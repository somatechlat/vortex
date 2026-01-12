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
use vortex_protocol::control::{TensorInput, TensorRef, TensorOutputSpec, WorkerState};
use vortex_protocol::{JobRequest, JobResult};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use crate::api::WsMessage;
use tokio::time::{sleep, Duration};

/// Execution context for a single run
pub struct ExecutionContext {
    pub run_id: String,
    pub graph_id: String,
    pub graph: GraphDSL,
    pub execution_plan: ExecutionPlan,
    pub tx: Sender<WsMessage>,
    pub run_repo: Arc<crate::run_repo::RunRepository>,
}

impl ExecutionContext {
    /// Execute the graph to completion
    pub async fn execute(self) -> VortexResult<RunStatus> {
        tracing::info!(run_id = %self.run_id, "Starting execution context");

        // 1. Memory arbitration - check if we have enough VRAM
        let arbiter = Arbiter::new(8192);
        // In production: let evictions = arbiter.prepare_execution(&self.execution_plan.execution_order, current_vram)?;
        
        // 2. Spawn supervisor and workers
        let worker_path = std::env::var("VORTEX_WORKER_SCRIPT")
            .unwrap_or_else(|_| "worker/vortex_worker/__main__.py".to_string());
            
        let mut supervisor = Supervisor::new(&worker_path);
        
        // Spawn workers (1 per slot for now)
        let slot_id = 0; // Start with single worker
        let pid = supervisor.spawn_worker(slot_id)?;
        tracing::info!(pid = %pid, slot_id = %slot_id, "Spawned worker");

        // 3. Connect to IPC socket
        let socket_path = std::env::var("VORTEX_IPC_PATH")
            .unwrap_or_else(|_| "/tmp/vortex.sock".to_string());
            
        // Wait for worker to bind socket
        sleep(Duration::from_millis(500)).await;
        
        let gateway = IpcGateway::new(&socket_path);
        let mut conn = match gateway.accept() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(error = %e, "Failed to accept IPC connection");
                // Kill worker
                supervisor.kill_worker(pid)?;
                return Err(e);
            }
        };

        // 4. Execute each node in order
        for node_id in &self.execution_plan.execution_order {
            tracing::info!(node_id = %node_id, "Executing node");
            
            // Get node definition
            let node = self.graph.nodes.get(node_id)
                .ok_or_else(|| VortexError::Internal(format!("Node {} not found", node_id)))?;

            // 5. Build JobRequest
            let job_request = self.build_job_request(node_id, &node.op_type, &node.params)?;
            
            // 6. Send via IPC
            conn.send_job_request(&job_request)
                .map_err(|e| VortexError::Internal(format!("IPC send failed: {}", e)))?;

            // 7. Wait for result
            let job_result = conn.receive_job_result()
                .map_err(|e| VortexError::Internal(format!("IPC receive failed: {}", e)))?;

            // 8. Update progress
            if job_result.success {
                self.tx.send(WsMessage::NodeComplete {
                    run_id: self.run_id.clone(),
                    node_id: node_id.clone(),
                    duration_ms: job_result.metrics.as_ref().map(|m| m.execution_us / 1000).unwrap_or(0),
                }).ok();
                
                // Log metrics
                if let Some(metrics) = job_result.metrics {
                    tracing::info!(
                        node = %node_id,
                        duration_us = %metrics.execution_us,
                        peak_vram = %metrics.peak_vram_bytes,
                        "Node completed"
                    );
                    
                    // Store in run_steps table
                    let _ = self.run_repo.insert_step(
                        &self.run_id,
                        node_id,
                        pid,
                        metrics.execution_us as i64,
                        (metrics.peak_vram_bytes / (1024 * 1024)) as i64,
                    ).await;
                }
            } else {
                // Job failed
                let error_msg = job_result.error.as_ref()
                    .map(|e| format!("{}: {}", e.code, e.message))
                    .unwrap_or_else(|| "Unknown error".to_string());
                
                self.tx.send(WsMessage::RunComplete {
                    run_id: self.run_id.clone(),
                    success: false,
                    error: Some(error_msg.clone()),
                }).ok();
                
                // Update run status to FAILED
                let mut run = self.run_repo.get_by_id(&self.run_id).await
                    .map_err(|e| VortexError::Internal(e.to_string()))?
                    .ok_or_else(|| VortexError::Internal("Run not found".to_string()))?;
                
                run.status = RunStatus::Failed;
                run.error_json = Some(serde_json::json!({"error": error_msg}).to_string());
                run.completed_at = Some(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64);
                
                self.run_repo.update(run).await
                    .map_err(|e| VortexError::Internal(e.to_string()))?;
                
                // Cleanup and return
                supervisor.shutdown()?;
                return Ok(RunStatus::Failed);
            }
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
            .unwrap()
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

    /// Build JobRequest from node data
    fn build_job_request(
        &self,
        node_id: &str,
        node_type: &str,
        params: &std::collections::HashMap<String, crate::graph::ParamValue>,
    ) -> VortexResult<JobRequest> {
        // Convert params to JSON bytes
        let params_json = serde_json::to_vec(params)
            .map_err(|e| VortexError::Internal(format!("Param serialization failed: {}", e)))?;

        // Build inputs (for now, empty - would read from previous nodes)
        let inputs = vec![];

        // Build outputs (for now, empty - would be based on node type)
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
