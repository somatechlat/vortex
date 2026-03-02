//! Execution Engine - Orchestrates graph execution from API to workers
//!
//! Connects all components: Scheduler → Arbiter → Supervisor → IPC

use crate::error::{VortexError, VortexResult};
use crate::graph::GraphDSL;
use crate::scheduler::{Scheduler, ExecutionPlan};
use crate::arbiter::Arbiter;
use crate::supervisor::{Supervisor, SupervisorTrait};
use crate::ipc::{IpcGateway, IpcConnection};
use crate::entities::run::{Model as RunModel, RunStatus};
use vortex_protocol::control::{TensorInput, TensorRef, TensorOutput, TensorOutputSpec, WorkerState};
use vortex_protocol::{JobRequest, JobResult};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio_util::sync::CancellationToken;
use crate::api::WsMessage;
use tokio::time::Duration;

use std::collections::HashMap;

/// Execution context for a single run
pub struct ExecutionContext {
    pub run_id: String,
    pub graph_id: String,
    pub graph: GraphDSL,
    pub execution_plan: ExecutionPlan,
    pub tx: Sender<WsMessage>,
    pub run_repo: Arc<crate::run_repo::RunRepository>,
    pub shm: Arc<crate::shm::SharedMemory>,
    pub cancel_token: CancellationToken,
    /// Registry of node outputs stored in SHM: (node_id, port_name) -> TensorOutput after each node completes
    pub output_registry: HashMap<(String, String), TensorOutput>,
}

impl ExecutionContext {
    /// Execute the graph to completion
    pub async fn execute(mut self) -> VortexResult<RunStatus> {
        tracing::info!(run_id = %self.run_id, "Starting execution context");

        // 1. Memory arbitration - check if we have enough VRAM
        let arbiter = Arbiter::new(8192);
        // In production: let evictions = arbiter.prepare_execution(&self.execution_plan.execution_order, current_vram)?;

        // 2. Spawn supervisor and workers
        let worker_path = std::env::var("VORTEX_WORKER_SCRIPT")
            .unwrap_or_else(|_| "worker/vortex_worker/__main__.py".to_string());

        let mut supervisor = Supervisor::new(&worker_path, self.shm.clone());

        // 3. Prepare IPC listener before worker boot so initial connect cannot race
        let socket_path = std::env::var("VORTEX_IPC_PATH")
            .unwrap_or_else(|_| "/tmp/vortex.sock".to_string());
        let mut gateway = IpcGateway::new(&socket_path);
        gateway.bind()?;

        // Spawn workers (1 per slot for now)
        let slot_id = 0; // Start with single worker
        let pid = supervisor.spawn_worker(slot_id)?;
        tracing::info!(pid = %pid, slot_id = %slot_id, "Spawned worker");

        // 4. Accept worker connection
        let mut conn = match gateway.accept() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(error = %e, "Failed to accept IPC connection");
                // Kill worker
                supervisor.kill_worker(pid)?;
                return Err(e);
            }
        };

        // T-03: Set 30-second read timeout for crash detection
        conn.set_read_timeout(Duration::from_secs(30))
            .map_err(|e| VortexError::Internal(format!("set_read_timeout: {e}")))?;

        // 4. Execute each node in order
        // Clone order to avoid borrow conflicts during mutable calls
        let execution_order: Vec<String> = self.execution_plan.execution_order.clone();

        for node_id in &execution_order {
            if self.cancel_token.is_cancelled() {
                let mut run = self.run_repo.get_by_id(&self.run_id).await
                    .map_err(|e| VortexError::Internal(e.to_string()))?
                    .ok_or_else(|| VortexError::Internal("Run not found".to_string()))?;
                run.status = RunStatus::Failed;
                run.error_json = Some(serde_json::json!({"error": "Run cancelled by user"}).to_string());
                run.completed_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| VortexError::Internal(format!("time error: {e}")))?
                        .as_secs() as i64,
                );
                self.run_repo.update(run).await
                    .map_err(|e| VortexError::Internal(e.to_string()))?;
                supervisor.shutdown()?;
                return Ok(RunStatus::Failed);
            }

            tracing::info!(node_id = %node_id, "Executing node");

            // Get node definition
            let node = self.graph.nodes.get(node_id)
                .ok_or_else(|| VortexError::Internal(format!("Node {} not found", node_id)))?;

            // 5. Check if node is an MCP tool - Proxy to Django if it is
            if node.op_type.starts_with("mcp.") || node.op_type.starts_with("odoo.") {
                tracing::info!(node_id = %node_id, op_type = %node.op_type, "Proxying to Django MCP Bridge");

                // Proxy to Django Admin API
                let mcp_url = format!("http://localhost:8000/api/mcp/call/{}", node.op_type);
                let client = reqwest::Client::new();
                let res = client.post(mcp_url)
                    .json(&node.params)
                    .send()
                    .await;

                match res {
                    Ok(resp) if resp.status().is_success() => {
                        let job_result = JobResult {
                            job_id: format!("{}_{}", self.run_id, node_id),
                            success: true,
                            outputs: vec![TensorOutput {
                                name: "result".into(),
                                tensor: None,
                            }],
                            error: None,
                            metrics: Some(vortex_protocol::control::JobMetrics {
                                execution_us: 1000,
                                peak_vram_bytes: 0,
                                tokens_processed: 0,
                            }),
                        };
                        self.handle_job_result(node_id, job_result, pid).await?;
                        continue;
                    }
                    Ok(resp) => {
                        self.handle_error(&node_id, format!("Django MCP Error: {}", resp.status())).await?;
                        supervisor.shutdown()?;
                        return Ok(RunStatus::Failed);
                    }
                    Err(e) => {
                        self.handle_error(&node_id, format!("Django Connectivity Error: {}", e)).await?;
                        supervisor.shutdown()?;
                        return Ok(RunStatus::Failed);
                    }
                }
            }

            // 6. Build JobRequest for Worker
            let job_request = self.build_job_request(node_id, &node.op_type, &node.params)?;

            // 7. Send via IPC
            conn.send_job_request(&job_request)
                .map_err(|e| VortexError::Internal(format!("IPC send failed: {}", e)))?;

            // 8. Wait for result (30s timeout set above; if worker crashes, read returns error)
            let job_result = conn.receive_job_result()
                .map_err(|e| {
                    supervisor.kill_worker(pid).ok();
                    VortexError::Internal(format!(
                        "Worker gone (pid={}, job={}_{}) — {}", pid, self.run_id, node_id, e
                    ))
                })?;

            self.handle_job_result(node_id, job_result, pid).await?;
        }

        // 9. All nodes complete - cleanup
        supervisor.shutdown()?;

        // 10. Mark run as complete
        let mut run = self.run_repo.get_by_id(&self.run_id).await
            .map_err(|e| VortexError::Internal(e.to_string()))?
            .ok_or_else(|| VortexError::Internal("Run not found".to_string()))?;

        run.status = RunStatus::Completed;
        run.completed_at = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| VortexError::Internal(format!("time error: {e}")))?
            .as_secs() as i64);

        self.run_repo.update(run).await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

        // 11. Send completion message
        self.tx.send(WsMessage::RunComplete {
            run_id: self.run_id.clone(),
            success: true,
            error: None,
        }).ok();

        Ok(RunStatus::Completed)
    }

    async fn handle_job_result(&mut self, node_id: &String, job_result: JobResult, pid: i32) -> VortexResult<()> {
        if job_result.success {
            self.tx.send(WsMessage::NodeComplete {
                run_id: self.run_id.clone(),
                node_id: node_id.clone(),
                duration_ms: job_result.metrics.as_ref().map(|m| m.execution_us / 1000).unwrap_or(0),
            }).ok();

            for output in job_result.outputs {
                self.output_registry.insert((node_id.clone(), output.name.clone()), output);
            }

            if let Some(metrics) = job_result.metrics {
                let _ = self.run_repo.insert_step(
                    &self.run_id,
                    node_id,
                    pid,
                    metrics.execution_us as i64,
                    (metrics.peak_vram_bytes / (1024 * 1024)) as i64,
                ).await;
            }
            Ok(())
        } else {
            let error_msg = job_result.error.as_ref()
                .map(|e| format!("{}: {}", e.code, e.message))
                .unwrap_or_else(|| "Unknown error".to_string());
            self.handle_error(node_id, error_msg).await
        }
    }

    async fn handle_error(&self, node_id: &str, error_msg: String) -> VortexResult<()> {
        self.tx.send(WsMessage::RunComplete {
            run_id: self.run_id.clone(),
            success: false,
            error: Some(error_msg.clone()),
        }).ok();

        let mut run = self.run_repo.get_by_id(&self.run_id).await
            .map_err(|e| VortexError::Internal(e.to_string()))?
            .ok_or_else(|| VortexError::Internal("Run not found".to_string()))?;

        run.status = RunStatus::Failed;
        run.error_json = Some(serde_json::json!({"error": error_msg}).to_string());
        run.completed_at = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| VortexError::Internal(format!("time error: {e}")))?
            .as_secs() as i64);

        self.run_repo.update(run).await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Build JobRequest from node data
    fn build_job_request(
        &self,
        node_id: &str,
        node_type: &str,
        params: &std::collections::HashMap<String, crate::graph::ParamValue>,
    ) -> VortexResult<JobRequest> {
        // Serialise params
        let params_json = serde_json::to_vec(params)
            .map_err(|e| VortexError::Internal(format!("param serialise: {e}")))?;

        // Resolve inputs: find every edge that feeds into this node
        let inputs: Vec<TensorInput> = self.graph.links
            .iter()
            .filter(|link| link.target.0 == node_id)
            .filter_map(|link| {
                let key = (link.source.0.clone(), link.source.1.clone());
                self.output_registry.get(&key).map(|out| TensorInput {
                    name: link.target.1.clone(),   // the port name on THIS node
                    tensor: out.tensor.clone(),     // TensorRef with SHM offset
                })
            })
            .collect();

        // Declare expected outputs from node type definition (use empty for now;
        // T-06 will populate this from ExecutorCapabilities)
        let outputs = vec![];

        Ok(JobRequest {
            job_id: format!("{}_{}", self.run_id, node_id),
            node_type: node_type.to_string(),
            params_json,
            inputs,
            outputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Node, Link};
    use crate::scheduler::ExecutionPlan;
    use tokio::sync::broadcast;

    #[test]
    fn test_output_registry_wires_inputs() {
        let mut graph = GraphDSL::new();
        graph.add_node(Node::new("A", "LoadModel"));
        graph.add_node(Node::new("B", "KSampler"));
        graph.add_link(("A".into(), "out".into()), ("B".into(), "in".into()));

        let (tx, _) = broadcast::channel(10);

        let mut shm_buffer = vec![0u8; 1024 * 1024];
        let shm = Arc::new(crate::shm::SharedMemory {
            base: shm_buffer.as_mut_ptr(),
            size: 1024 * 1024,
            fd: -1,
        });

        let mut ctx = ExecutionContext {
            run_id: "test_run".into(),
            graph_id: "test_graph".into(),
            graph,
            execution_plan: ExecutionPlan {
                execution_order: vec!["A".into(), "B".into()],
                node_hashes: HashMap::new(),
                dirty_nodes: vec!["A".into(), "B".into()],
                estimated_time_ms: 0,
            },
            tx,
            run_repo: Arc::new(crate::run_repo::RunRepository::new(Arc::new(crate::db::Database::new_mock()))),
            shm,
            cancel_token: CancellationToken::new(),
            output_registry: HashMap::new(),
        };

        // Mock a JobResult for A
        let out_tensor = TensorOutput {
            name: "out".into(),
            tensor: Some(TensorRef { offset: 0, size_bytes: 1024, dtype: 0, shape: vec![4, 64, 64] }),
        };
        ctx.output_registry.insert(("A".into(), "out".into()), out_tensor);

        // Call build_job_request for B
        let req = ctx.build_job_request("B", "KSampler", &HashMap::new()).unwrap();

        // Assert inputs
        assert_eq!(req.inputs.len(), 1);
        assert_eq!(req.inputs[0].name, "in");
        assert!(req.inputs[0].tensor.is_some());
    }
}
