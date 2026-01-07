//! VORTEX-GEN 3.0 Core Engine
//!
//! This crate implements the "Centaur" architecture:
//! - Rust Host for control plane (scheduling, memory arbitration)
//! - Python Workers for compute plane (ML inference)
//! - Zero-Copy IPC via POSIX Shared Memory

// Scaffold phase: allow dead_code until all modules are connected
#![allow(dead_code, unused_variables, unused_imports, clippy::new_without_default)]
#![allow(unexpected_cfgs)]
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐
//! │  Frontend   │────▶│  API Layer  │
//! │  (Svelte)   │     │  (HTTP/WS)  │
//! └─────────────┘     └──────┬──────┘
//!                            │
//!                     ┌──────▼──────┐
//!                     │ Core Engine │
//!                     │  (Salsa DB) │
//!                     └──────┬──────┘
//!                            │
//!              ┌─────────────┼─────────────┐
//!              │             │             │
//!       ┌──────▼──────┐ ┌────▼────┐ ┌──────▼──────┐
//!       │  Scheduler  │ │ Arbiter │ │ Supervisor  │
//!       │  (Kahn's)   │ │  (LFU)  │ │  (fork/IPC) │
//!       └─────────────┘ └─────────┘ └──────┬──────┘
//!                                          │
//!                                   ┌──────▼──────┐
//!                                   │   Worker    │
//!                                   │  (Python)   │
//!                                   └─────────────┘
//! ```

pub mod error;
pub mod graph;
pub mod scheduler;
pub mod arbiter;
pub mod ipc;
pub mod shm;
pub mod supervisor;
pub mod db;
pub mod api;
pub mod entities;
pub mod config;
pub mod settings;
pub mod tenant;
pub mod authz;
pub mod tenant_repo;
pub mod graph_repo;
pub mod run_repo;

pub use error::{VortexError, VortexResult};
pub use graph::{GraphDSL, Node, NodeID, Link};
pub use scheduler::Scheduler;
pub use arbiter::Arbiter;
pub use supervisor::Supervisor;
