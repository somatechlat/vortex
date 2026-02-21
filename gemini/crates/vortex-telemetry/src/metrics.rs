//! Prometheus Metrics

use prometheus::{Counter, Gauge, Histogram, Registry};
use std::sync::LazyLock;

/// Global metrics registry
pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

fn register_counter(name: &str, help: &str) -> Option<Counter> {
    let counter = match Counter::new(name, help) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, metric = %name, "Failed to initialize counter metric");
            return None;
        }
    };
    if let Err(e) = REGISTRY.register(Box::new(counter.clone())) {
        tracing::error!(error = %e, metric = %name, "Failed to register counter metric");
        return None;
    }
    Some(counter)
}

fn register_gauge(name: &str, help: &str) -> Option<Gauge> {
    let gauge = match Gauge::new(name, help) {
        Ok(g) => g,
        Err(e) => {
            tracing::error!(error = %e, metric = %name, "Failed to initialize gauge metric");
            return None;
        }
    };
    if let Err(e) = REGISTRY.register(Box::new(gauge.clone())) {
        tracing::error!(error = %e, metric = %name, "Failed to register gauge metric");
        return None;
    }
    Some(gauge)
}

fn register_histogram(name: &str, help: &str, buckets: Vec<f64>) -> Option<Histogram> {
    let opts = prometheus::HistogramOpts::new(name, help).buckets(buckets);
    let histogram = match Histogram::with_opts(opts) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(error = %e, metric = %name, "Failed to initialize histogram metric");
            return None;
        }
    };
    if let Err(e) = REGISTRY.register(Box::new(histogram.clone())) {
        tracing::error!(error = %e, metric = %name, "Failed to register histogram metric");
        return None;
    }
    Some(histogram)
}

/// Request counter
pub static REQUESTS_TOTAL: LazyLock<Option<Counter>> = LazyLock::new(|| {
    register_counter("vortex_requests_total", "Total requests processed")
});

/// Active workers gauge
pub static ACTIVE_WORKERS: LazyLock<Option<Gauge>> = LazyLock::new(|| {
    register_gauge("vortex_active_workers", "Number of active worker processes")
});

/// VRAM usage gauge
pub static VRAM_USED_BYTES: LazyLock<Option<Gauge>> = LazyLock::new(|| {
    register_gauge("vortex_vram_used_bytes", "VRAM bytes currently allocated")
});

/// Job execution duration histogram
pub static JOB_DURATION_SECONDS: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    register_histogram(
        "vortex_job_duration_seconds",
        "Job execution duration",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0],
    )
});

/// Encode all metrics to Prometheus text format
pub fn encode_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return format!("# metrics_encode_error {}\n", e);
    }
    match String::from_utf8(buffer) {
        Ok(s) => s,
        Err(e) => format!("# metrics_utf8_error {}\n", e),
    }
}
