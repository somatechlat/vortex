//! VORTEX Core Units
//!
//! Native processing units for generative AI workflows.

pub mod loader;
pub mod sampler;
pub mod encoder;

use vortex_protocol::graph::NodeDef;

/// Trait for native VORTEX units.
pub trait VortexUnit: Send + Sync {
    /// Get the static definition for this unit.
    fn definition(&self) -> NodeDef;
}
