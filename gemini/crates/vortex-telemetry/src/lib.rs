//! VORTEX Telemetry
//!
//! Unified observability layer providing:
//! - Structured JSON logging
//! - OpenTelemetry tracing
//! - Prometheus metrics

pub mod logging;
pub mod tracing_setup;
pub mod metrics;

use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize telemetry with default configuration
pub fn init() {
    init_with_level(Level::INFO);
}

/// Initialize telemetry with specific log level
pub fn init_with_level(level: Level) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level.to_string()));
    
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true);
    
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
    
    tracing::info!("VORTEX telemetry initialized");
}
