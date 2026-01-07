//! VORTEX-GEN 3.0 Core Engine
//!
//! Entry point for the VORTEX core service.
//! Adheres to Rule 122 (10-Cycle Resiliency) and Rule 9 (Centralized Config).

use vortex_core::error::VortexResult;

#[tokio::main]
async fn main() -> VortexResult<()> {
    // 1. Initialize Tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("vortex=info".parse().unwrap())
        )
        .init();

    tracing::info!("Starting VORTEX Core Engine (Native Mode)");

    // 2. Delegate to Server Module for Resilient Startup
    // This handles Config (Rule 9), DB Connect (Rule 122), and Migration (Rule 106).
    vortex_core::server::start_server().await
}
