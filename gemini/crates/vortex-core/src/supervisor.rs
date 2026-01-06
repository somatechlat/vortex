//! Supervisor - Worker process management
//!
//! Implements SRS Section 3.6.1 (Supervisor Trait) and Section 3.7.1 (Worker Lifecycle)

use crate::error::{VortexError, VortexResult};
use crate::shm::WorkerStatus;
use std::collections::HashMap;
use std::process::{Child, Command};

/// Health status of a worker
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unresponsive,
    Dead,
}

/// Supervisor trait as defined in SRS Section 3.6.1
pub trait SupervisorTrait {
    /// Spawns a new worker logic process.
    /// Returns the PID and the assigned SHM Slot ID.
    fn spawn_worker(&mut self, slot_id: u8) -> VortexResult<i32>;
    
    /// Handles a SIGCHLD signal (Process Death).
    /// Returns `true` if the worker was critical and needs immediate replacement.
    fn handle_crash(&mut self, pid: i32, exit_code: i32) -> bool;
    
    /// Sends a Heartbeat Ping over IPC.
    fn check_health(&self, pid: i32) -> HealthStatus;
}

/// Worker process information
#[derive(Debug)]
pub struct WorkerInfo {
    pub pid: i32,
    pub slot_id: u8,
    pub child: Option<Child>,
    pub status: WorkerStatus,
    pub current_job: Option<String>,
    pub spawn_time: std::time::Instant,
}

/// Worker Supervisor implementation
pub struct Supervisor {
    /// Path to Python worker executable
    worker_script: String,
    
    /// Python interpreter path
    python_path: String,
    
    /// Map of PID -> WorkerInfo
    workers: HashMap<i32, WorkerInfo>,
    
    /// Shared memory reference (todo: should be Arc)
    shm_name: String,
    
    /// Maximum number of workers
    max_workers: usize,
    
    /// Heartbeat timeout (ms)
    heartbeat_timeout: u64,
}

impl Supervisor {
    /// Create a new supervisor
    pub fn new(worker_script: impl Into<String>) -> Self {
        Self {
            worker_script: worker_script.into(),
            python_path: "python3".to_string(),
            workers: HashMap::new(),
            shm_name: crate::shm::SHM_NAME.to_string(),
            max_workers: 4,
            heartbeat_timeout: 5000, // 5 seconds
        }
    }
    
    /// Set the Python interpreter path
    pub fn with_python(mut self, path: impl Into<String>) -> Self {
        self.python_path = path.into();
        self
    }
    
    /// Set maximum number of workers
    pub fn with_max_workers(mut self, max: usize) -> Self {
        self.max_workers = max;
        self
    }
    
    /// Get number of active workers
    pub fn active_count(&self) -> usize {
        self.workers.len()
    }
    
    /// Get worker by PID
    pub fn get_worker(&self, pid: i32) -> Option<&WorkerInfo> {
        self.workers.get(&pid)
    }
    
    /// Get worker by slot ID
    pub fn get_worker_by_slot(&self, slot_id: u8) -> Option<&WorkerInfo> {
        self.workers.values().find(|w| w.slot_id == slot_id)
    }
    
    /// Spawn a new worker process
    fn do_spawn(&mut self, slot_id: u8) -> VortexResult<i32> {
        // Build command
        let mut cmd = Command::new(&self.python_path);
        cmd.arg(&self.worker_script)
            .arg("--slot-id")
            .arg(slot_id.to_string())
            .arg("--shm-name")
            .arg(&self.shm_name);
        
        // Spawn process
        let child = cmd.spawn().map_err(|e| VortexError::Internal(
            format!("Failed to spawn worker: {}", e)
        ))?;
        
        let pid = child.id() as i32;
        
        // Register worker
        self.workers.insert(pid, WorkerInfo {
            pid,
            slot_id,
            child: Some(child),
            status: WorkerStatus::Booting,
            current_job: None,
            spawn_time: std::time::Instant::now(),
        });
        
        tracing::info!("Spawned worker PID {} in slot {}", pid, slot_id);
        
        Ok(pid)
    }
    
    /// Check for and handle dead workers
    pub fn reap_workers(&mut self) -> Vec<(i32, i32)> {
        let mut dead = Vec::new();
        
        for (pid, worker) in self.workers.iter_mut() {
            if let Some(ref mut child) = worker.child {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let code = status.code().unwrap_or(-1);
                        dead.push((*pid, code));
                    }
                    Ok(None) => {
                        // Still running
                    }
                    Err(e) => {
                        tracing::error!("Error checking worker {}: {}", pid, e);
                    }
                }
            }
        }
        
        // Remove dead workers
        for (pid, code) in &dead {
            self.workers.remove(pid);
            tracing::warn!("Worker {} exited with code {}", pid, code);
        }
        
        dead
    }
    
    /// Kill a worker by PID
    pub fn kill_worker(&mut self, pid: i32) -> VortexResult<()> {
        if let Some(mut worker) = self.workers.remove(&pid) {
            if let Some(ref mut child) = worker.child {
                child.kill().map_err(|e| VortexError::Internal(
                    format!("Failed to kill worker {}: {}", pid, e)
                ))?;
            }
        }
        Ok(())
    }
    
    /// Kill all workers
    pub fn shutdown(&mut self) -> VortexResult<()> {
        let pids: Vec<i32> = self.workers.keys().copied().collect();
        for pid in pids {
            self.kill_worker(pid)?;
        }
        Ok(())
    }
}

impl SupervisorTrait for Supervisor {
    fn spawn_worker(&mut self, slot_id: u8) -> VortexResult<i32> {
        if self.workers.len() >= self.max_workers {
            return Err(VortexError::ResourceExhausted {
                requested_mb: 0,
                limit_mb: 0,
            });
        }
        
        self.do_spawn(slot_id)
    }
    
    fn handle_crash(&mut self, pid: i32, exit_code: i32) -> bool {
        if let Some(worker) = self.workers.remove(&pid) {
            tracing::error!(
                "Worker {} crashed with code {} (job: {:?})",
                pid,
                exit_code,
                worker.current_job
            );
            
            // Critical if it was processing a job
            worker.current_job.is_some()
        } else {
            false
        }
    }
    
    fn check_health(&self, pid: i32) -> HealthStatus {
        match self.workers.get(&pid) {
            Some(worker) => {
                match worker.status {
                    WorkerStatus::Idle | WorkerStatus::Busy => HealthStatus::Healthy,
                    WorkerStatus::Booting => HealthStatus::Healthy, // Give time to boot
                    WorkerStatus::Dead => HealthStatus::Dead,
                }
            }
            None => HealthStatus::Dead,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_creation() {
        let sup = Supervisor::new("worker.py")
            .with_max_workers(8)
            .with_python("/usr/bin/python3");
        
        assert_eq!(sup.max_workers, 8);
        assert_eq!(sup.active_count(), 0);
    }
}
