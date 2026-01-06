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

/// Initialize OpenTelemetry tracer (optional, if OTLP endpoint configured)
pub fn init_tracer(config: &OtelConfig) -> Option<opentelemetry::global::BoxedTracer> {
    config.otlp_endpoint.as_ref().map(|_endpoint| {
        // TODO: Configure OTLP exporter when endpoint is set
        opentelemetry::global::tracer("vortex")
    })
}
