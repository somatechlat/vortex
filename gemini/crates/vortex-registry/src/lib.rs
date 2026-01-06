//! VORTEX Registry System
//!
//! Package manager for VORTEX custom nodes with:
//! - PubGrub dependency solver
//! - AST-based security scanner
//! - Virtual environment isolation

// Scaffold phase: allow dead_code until all modules are connected
#![allow(dead_code, unused_variables, clippy::new_without_default)]
#![allow(unexpected_cfgs)]

pub mod solver;
pub mod scanner;
pub mod venv;
pub mod manifest;
pub mod lockfile;

// CLI module only in binary
#[cfg(feature = "cli")]
pub mod cli;
