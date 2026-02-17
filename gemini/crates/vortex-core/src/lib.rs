//! VORTEX-GEN 3.0 Core Engine
//!
//! This crate implements the "Centaur" architecture:
//! - Rust Host for control plane (scheduling, memory arbitration)
//! - Python Workers for compute plane (ML inference)
//! - Zero-Copy IPC via POSIX Shared Memory

// Scaffold phase: allow dead_code until all modules are connected
#![allow(dead_code, unused_variables, unused_imports, clippy::new_without_default)]
#![allow(unexpected_cfgs)]

// ═══════════════════════════════════════════════════════════════
// Core Modules
// ═══════════════════════════════════════════════════════════════

pub mod error;
pub mod config;

// ═══════════════════════════════════════════════════════════════
// Data Models & Protocols
// ═══════════════════════════════════════════════════════════════

pub mod graph;
pub mod serialization;
pub mod shm;

// ═══════════════════════════════════════════════════════════════
// Execution & Scheduling
// ═══════════════════════════════════════════════════════════════

pub mod scheduler;
pub mod arbiter;
pub mod supervisor;
pub mod execution;

// ═══════════════════════════════════════════════════════════════
// IPC & Networking
// ═══════════════════════════════════════════════════════════════

pub mod ipc;

// ═══════════════════════════════════════════════════════════════
// Database & Persistence
// ═══════════════════════════════════════════════════════════════

pub mod db;
pub mod entities;
pub mod tenant_repo;
pub mod graph_repo;
pub mod run_repo;
pub mod tenant;

// ═══════════════════════════════════════════════════════════════
// API & Web Services
// ═══════════════════════════════════════════════════════════════

pub mod authz;
pub mod api;
pub mod server;
pub mod mcp_registry;
pub mod units;

// ═══════════════════════════════════════════════════════════════
// Monitoring
// ═══════════════════════════════════════════════════════════════

pub mod metrics;
