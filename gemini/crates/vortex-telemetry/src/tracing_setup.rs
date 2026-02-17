//! OpenTelemetry Tracing Setup

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
pub fn init_tracer(config: &OtelConfig) -> Option<opentelemetry::sdk::trace::Tracer> {
    config.otlp_endpoint.as_ref().map(|endpoint| {
        use opentelemetry_otlp::WithExportConfig;
        use tracing_subscriber::prelude::*;

        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint);

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(
                opentelemetry::sdk::trace::config()
                    .with_resource(opentelemetry::sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
                    ])),
            )
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("Failed to initialize OTLP tracer");

        // Unified subscriber setup
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.clone());
        let subscriber = tracing_subscriber::Registry::default()
            .with(telemetry)
            .with(tracing_subscriber::fmt::layer());

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global subscriber");

        tracer
    })
}
