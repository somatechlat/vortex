//! Prometheus Metrics

use prometheus::{Counter, Gauge, Histogram, Registry};
use std::sync::LazyLock;

/// Global metrics registry
pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

/// Request counter
pub static REQUESTS_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    let counter = Counter::new("vortex_requests_total", "Total requests processed").unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

/// Active workers gauge
pub static ACTIVE_WORKERS: LazyLock<Gauge> = LazyLock::new(|| {
    let gauge = Gauge::new("vortex_active_workers", "Number of active worker processes").unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

/// VRAM usage gauge
pub static VRAM_USED_BYTES: LazyLock<Gauge> = LazyLock::new(|| {
    let gauge = Gauge::new("vortex_vram_used_bytes", "VRAM bytes currently allocated").unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

/// Job execution duration histogram
pub static JOB_DURATION_SECONDS: LazyLock<Histogram> = LazyLock::new(|| {
    let histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new("vortex_job_duration_seconds", "Job execution duration")
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0]),
    ).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

/// Encode all metrics to Prometheus text format
pub fn encode_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
