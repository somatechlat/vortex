//! VORTEX Protocol Types
//!
//! Shared protocol types and protobuf definitions for VORTEX-GEN 3.0.
//! This crate provides the contract between all VORTEX components.

// Scaffold phase: allow dead_code until all modules are connected
#![allow(dead_code, unused_imports, clippy::new_without_default)]

use std::fmt;

// ═══════════════════════════════════════════════════════════════
//                    GENERATED PROTOBUFS
// ═══════════════════════════════════════════════════════════════

/// Control messages (Host ↔ Worker)
pub mod control {
    include!("generated/vortex.control.rs");
}

/// Graph definitions
pub mod graph {
    include!("generated/vortex.graph.rs");
}

/// Worker-specific types
pub mod worker {
    include!("generated/vortex.worker.rs");
}

// ═══════════════════════════════════════════════════════════════
//                    CORE TYPES
// ═══════════════════════════════════════════════════════════════

/// Unique node identifier within a graph
#[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeID(pub uuid::Uuid);

impl NodeID {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node:{}", &self.0.to_string()[..8])
    }
}

impl fmt::Debug for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeID({})", &self.0.to_string()[..8])
    }
}

/// Unique graph identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct GraphID(pub uuid::Uuid);

impl GraphID {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for GraphID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "graph:{}", &self.0.to_string()[..8])
    }
}

impl fmt::Debug for GraphID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GraphID({})", &self.0.to_string()[..8])
    }
}

/// Unique job identifier for execution tracking
#[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct JobID(pub uuid::Uuid);

impl JobID {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for JobID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "job:{}", &self.0.to_string()[..8])
    }
}

impl fmt::Debug for JobID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JobID({})", &self.0.to_string()[..8])
    }
}

/// Unique tensor identifier for memory tracking
#[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TensorID(pub uuid::Uuid);

impl TensorID {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for TensorID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tensor:{}", &self.0.to_string()[..8])
    }
}

impl fmt::Debug for TensorID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TensorID({})", &self.0.to_string()[..8])
    }
}

// ═══════════════════════════════════════════════════════════════
//                    DATA TYPES
// ═══════════════════════════════════════════════════════════════

/// Tensor data types (matches Signal Bus lanes)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum DataType {
    Latent = 0,
    Image = 1,
    Model = 2,
    Clip = 3,
    Vae = 4,
    Conditioning = 5,
    Mask = 6,
    ControlNet = 7,
}

impl DataType {
    /// Get the CSS color for this data type (Signal Bus visualization)
    pub fn color(&self) -> &'static str {
        match self {
            DataType::Latent => "#a855f7",       // Purple
            DataType::Image => "#ef4444",         // Red
            DataType::Model => "#3b82f6",         // Blue
            DataType::Clip => "#22c55e",          // Green
            DataType::Vae => "#f97316",           // Orange
            DataType::Conditioning => "#06b6d4",  // Cyan
            DataType::Mask => "#f59e0b",          // Amber
            DataType::ControlNet => "#ec4899",    // Pink
        }
    }
    
    /// Get the lane index for Signal Bus routing
    pub fn lane(&self) -> usize {
        *self as usize
    }
}

// ═══════════════════════════════════════════════════════════════
//                    CONSTANTS
// ═══════════════════════════════════════════════════════════════

/// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum nodes per graph
pub const MAX_NODES: usize = 10_000;

/// Maximum workers per host
pub const MAX_WORKERS: usize = 256;

/// Default heartbeat interval (ms)
pub const HEARTBEAT_INTERVAL_MS: u64 = 1000;

/// Heartbeat timeout (dead if no heartbeat in this time)
pub const HEARTBEAT_TIMEOUT_MS: u64 = 5000;

/// Maximum message size (16MB)
pub const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

/// Shared memory arena name
pub const SHM_NAME: &str = "/vortex-shm";

/// IPC socket path
pub const IPC_SOCKET_PATH: &str = "/tmp/vortex.sock";

// ═══════════════════════════════════════════════════════════════
//                    ERROR TYPES
// ═══════════════════════════════════════════════════════════════

/// VORTEX Error Registry (VE-XXX codes per SRS Section 4.2)
#[derive(Debug, thiserror::Error, Clone, serde::Serialize, serde::Deserialize)]
pub enum VortexError {
    // Graph Compilation (VE-0xx)
    #[error("VE-001 CycleDetected: Graph contains a cycle involving nodes: {nodes:?}")]
    CycleDetected { nodes: Vec<String> },

    #[error("VE-002 TypeMismatch: Cannot connect {source_type} to {target_type}")]
    TypeMismatch {
        source_type: String,
        target_type: String,
    },

    #[error("VE-003 ResourceExhausted: Requested {requested_mb}MB exceeds limit {limit_mb}MB")]
    ResourceExhausted { requested_mb: u64, limit_mb: u64 },

    #[error("VE-004 WorkerGone: Worker PID {pid} died unexpectedly")]
    WorkerGone { pid: i32, job_id: Option<String> },

    #[error("VE-005 IntegrityError: Hash mismatch for {resource}")]
    IntegrityError { resource: String },

    // System Errors (SYS-xxx)
    #[error("SYS-001 ShmFailure: Shared memory error: {reason}")]
    ShmFailure { reason: String },

    #[error("SYS-002 IpcFailure: IPC error: {reason}")]
    IpcFailure { reason: String },

    #[error("SYS-003 VersionMismatch: Expected v{expected}, got v{actual}")]
    VersionMismatch { expected: u32, actual: u32 },

    // Registry Errors (REG-xxx)
    #[error("REG-001 PackageNotFound: Package {name} not found")]
    PackageNotFound { name: String },

    #[error("REG-002 DependencyConflict: {reason}")]
    DependencyConflict { reason: String },

    #[error("REG-003 SecurityViolation: {code} - {description}")]
    SecurityViolation { code: String, description: String },
}

impl VortexError {
    /// Get the error code string
    pub fn code(&self) -> &'static str {
        match self {
            VortexError::CycleDetected { .. } => "VE-001",
            VortexError::TypeMismatch { .. } => "VE-002",
            VortexError::ResourceExhausted { .. } => "VE-003",
            VortexError::WorkerGone { .. } => "VE-004",
            VortexError::IntegrityError { .. } => "VE-005",
            VortexError::ShmFailure { .. } => "SYS-001",
            VortexError::IpcFailure { .. } => "SYS-002",
            VortexError::VersionMismatch { .. } => "SYS-003",
            VortexError::PackageNotFound { .. } => "REG-001",
            VortexError::DependencyConflict { .. } => "REG-002",
            VortexError::SecurityViolation { .. } => "REG-003",
        }
    }
}

/// Result type alias
pub type VortexResult<T> = Result<T, VortexError>;

// Re-export common types
pub use control::{JobRequest, JobResult, Heartbeat, WorkerHandshake, HandshakeAck};
pub use graph::{Graph, Node, Edge, NodeDef, PortDef, ExecutionPlan, DataType as ProtoDataType};
pub use worker::{WorkerStatus, WorkerPhase, ExecutorInfo, SecurityViolation as ProtoSecurityViolation};
