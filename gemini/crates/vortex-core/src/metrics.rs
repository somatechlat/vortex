//! Metrics Collection for Prometheus Export

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug)]
pub struct MetricsCollector {
    pub total_jobs: AtomicU64,
    pub successful_jobs: AtomicU64,
    pub failed_jobs: AtomicU64,
    pub total_execution_time_us: AtomicU64,
    pub total_vram_usage: AtomicU64,
    pub active_workers: AtomicU64,
    pub start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            total_jobs: AtomicU64::new(0),
            successful_jobs: AtomicU64::new(0),
            failed_jobs: AtomicU64::new(0),
            total_execution_time_us: AtomicU64::new(0),
            total_vram_usage: AtomicU64::new(0),
            active_workers: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn record_job_started(&self) {
        self.total_jobs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_job_completed(&self, success: bool, execution_time_us: u64, peak_vram: u64) {
        if success {
            self.successful_jobs.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_jobs.fetch_add(1, Ordering::Relaxed);
        }
        self.total_execution_time_us.fetch_add(execution_time_us, Ordering::Relaxed);
        self.total_vram_usage.fetch_max(peak_vram, Ordering::Relaxed);
    }

    pub fn set_active_workers(&self, count: u64) {
        self.active_workers.store(count, Ordering::Relaxed);
    }

    pub fn get_prometheus_metrics(&self) -> String {
        let uptime = self.start_time.elapsed().as_secs();
        let total = self.total_jobs.load(Ordering::Relaxed);
        let success = self.successful_jobs.load(Ordering::Relaxed);
        let failed = self.failed_jobs.load(Ordering::Relaxed);
        let total_time = self.total_execution_time_us.load(Ordering::Relaxed);
        let avg_time = if total > 0 { total_time / total } else { 0 };
        let vram = self.total_vram_usage.load(Ordering::Relaxed);
        let workers = self.active_workers.load(Ordering::Relaxed);

        format!(
            "# HELP vortex_uptime_seconds Total uptime in seconds
# TYPE vortex_uptime_seconds gauge
vortex_uptime_seconds {}

# HELP vortex_jobs_total Total number of jobs processed
# TYPE vortex_jobs_total counter
vortex_jobs_total {}

# HELP vortex_jobs_success_total Successful jobs
# TYPE vortex_jobs_success_total counter
vortex_jobs_success_total {}

# HELP vortex_jobs_failed_total Failed jobs
# TYPE vortex_jobs_failed_total counter
vortex_jobs_failed_total {}

# HELP vortex_avg_execution_time_us Average execution time in microseconds
# TYPE vortex_avg_execution_time_us gauge
vortex_avg_execution_time_us {}

# HELP vortex_peak_vram_bytes Peak VRAM usage
# TYPE vortex_peak_vram_bytes gauge
vortex_peak_vram_bytes {}

# HELP vortex_active_workers Number of active workers
# TYPE vortex_active_workers gauge
vortex_active_workers {}

# HELP vortex_info VORTEX version and build info
# TYPE vortex_info info
vortex_info{{version=\"{}\"}} 1
",
            uptime, total, success, failed, avg_time, vram, workers, env!("CARGO_PKG_VERSION")
        )
    }
}
