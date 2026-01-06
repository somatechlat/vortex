//! Error types for VORTEX Core Engine
//!
//! All errors are defined as variants of `VortexError` enum.
//! Error codes follow the registry in SRS Section 4.2.

use thiserror::Error;

/// Result type alias for VORTEX operations
pub type VortexResult<T> = Result<T, VortexError>;

/// Core error enum following SRS Error Code Registry
#[derive(Error, Debug)]
pub enum VortexError {
    // === Graph Compilation Errors (VE-0xx) ===
    
    /// VE-001: Graph contains a cycle (A->B->A)
    #[error("VE-001 CycleDetected: Graph contains a cycle involving nodes: {nodes:?}")]
    CycleDetected { nodes: Vec<String> },

    /// VE-002: Output type incompatible with Input type
    #[error("VE-002 TypeMismatch: Cannot connect {source_type} to {target_type}")]
    TypeMismatch {
        source_type: String,
        target_type: String,
        source_node: String,
        target_node: String,
    },

    /// VE-003: VRAM request exceeds hardware limit
    #[error("VE-003 ResourceExhausted: Requested {requested_mb}MB exceeds limit {limit_mb}MB")]
    ResourceExhausted {
        requested_mb: u64,
        limit_mb: u64,
    },

    /// VE-004: Worker process died unexpectedly
    #[error("VE-004 WorkerGone: Worker PID {pid} exited with code {exit_code}")]
    WorkerGone {
        pid: i32,
        exit_code: i32,
        job_id: Option<String>,
    },

    /// VE-005: Hash integrity check failed
    #[error("VE-005 IntegrityError: Hash mismatch for {resource}")]
    IntegrityError { resource: String },

    // === System Errors (SYS-0xx) ===
    
    /// SYS-001: Shared memory open failed
    #[error("SYS-001 ShmFailure: shm_open failed: {reason}")]
    ShmFailure { reason: String },

    /// SYS-002: Socket bind failed
    #[error("SYS-002 BindError: Cannot bind to {path}: {reason}")]
    BindError { path: String, reason: String },

    /// SYS-003: Protocol version mismatch
    #[error("SYS-003 VersionMismatch: Expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },

    // === Database Errors ===
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // === IO Errors ===
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // === Serialization Errors ===
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // === Generic Errors ===
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl VortexError {
    /// Returns the error code string (e.g., "VE-001")
    pub fn code(&self) -> &'static str {
        match self {
            VortexError::CycleDetected { .. } => "VE-001",
            VortexError::TypeMismatch { .. } => "VE-002",
            VortexError::ResourceExhausted { .. } => "VE-003",
            VortexError::WorkerGone { .. } => "VE-004",
            VortexError::IntegrityError { .. } => "VE-005",
            VortexError::ShmFailure { .. } => "SYS-001",
            VortexError::BindError { .. } => "SYS-002",
            VortexError::VersionMismatch { .. } => "SYS-003",
            VortexError::Database(_) => "DB-001",
            VortexError::Io(_) => "IO-001",
            VortexError::Json(_) => "JSON-001",
            VortexError::Internal(_) => "INT-001",
        }
    }

    /// Returns true if this error should trigger worker respawn
    pub fn should_respawn_worker(&self) -> bool {
        matches!(self, VortexError::WorkerGone { .. })
    }
}
