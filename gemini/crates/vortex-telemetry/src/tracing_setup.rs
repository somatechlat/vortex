//! OpenTelemetry Tracing Setup

use opentelemetry_sdk::trace::Tracer;

/// OpenTelemetry configuration
pub struct OtelConfig {
    pub service_name: String,
    pub otlp_endpoint: Option<String>,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            service_name: "vortex-core".into(),
            otlp_endpoint: None,
        }
    }
}

/// Initialize OpenTelemetry tracer with unified logging integration
pub fn init_tracer(config: &OtelConfig) -> Option<Tracer> {
    config.otlp_endpoint.as_ref().and_then(|endpoint| {
        use opentelemetry_otlp::WithExportConfig;
        use tracing_subscriber::prelude::*;

        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint);

        let tracer = match opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(
                opentelemetry_sdk::trace::config()
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
                    ])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
        {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(error = %e, "Failed to initialize OTLP tracer");
                return None;
            }
        };

        // Unified subscriber setup
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.clone());
        let subscriber = tracing_subscriber::Registry::default()
            .with(telemetry)
            .with(tracing_subscriber::fmt::layer());

        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            tracing::warn!(error = %e, "Global subscriber already initialized; keeping existing subscriber");
        }

        Some(tracer)
    })
}
