//! System Deployment Configuration
//!
//! Hardware detection and system configuration.
//! Delegates to vortex-config for centralized settings (Rule 9).
//!
//! This module provides:
//! - HardwareCapabilities: Runtime hardware detection
//! - GpuDevice: GPU information
//! - SystemConfig: Aggregates vortex-config with hardware detection

use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

// Re-export from vortex-config - single source of truth (Rule 9)
pub use vortex_config::{
    ComputeMode, DeploymentMode, FeatureFlags, ResourceLimits,
};

// ═══════════════════════════════════════════════════════════════
//                    HARDWARE DETECTION
// ═══════════════════════════════════════════════════════════════

/// Detected hardware capabilities - unique to this module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    /// Number of CPU cores available
    pub cpu_cores: usize,
    /// Total RAM in bytes
    pub ram_bytes: u64,
    /// GPU devices detected
    pub gpus: Vec<GpuDevice>,
    /// Whether CUDA is available
    pub cuda_available: bool,
    /// CUDA version if available
    pub cuda_version: Option<String>,
}

/// GPU device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    /// Device index
    pub index: u32,
    /// Device name (e.g., "NVIDIA RTX 4090")
    pub name: String,
    /// VRAM in bytes
    pub vram_bytes: u64,
    /// Compute capability (e.g., "8.9")
    pub compute_capability: String,
}

impl HardwareCapabilities {
    /// Auto-detect hardware capabilities
    pub fn detect() -> Self {
        let cpu_cores = num_cpus::get();
        let ram_bytes = Self::detect_ram();
        let (gpus, cuda_available, cuda_version) = Self::detect_gpus();
        
        Self {
            cpu_cores,
            ram_bytes,
            gpus,
            cuda_available,
            cuda_version,
        }
    }
    
    /// Check if system has usable GPU
    pub fn has_gpu(&self) -> bool {
        self.cuda_available && !self.gpus.is_empty()
    }
    
    /// Get recommended compute mode based on hardware
    pub fn recommended_compute_mode(&self) -> ComputeMode {
        if self.has_gpu() && self.gpus.iter().any(|g| g.vram_bytes >= 8 * 1024 * 1024 * 1024) {
            ComputeMode::Gpu
        } else {
            ComputeMode::Cpu
        }
    }
    
    fn detect_ram() -> u64 {
        env::var("VORTEX_AVAILABLE_RAM_GB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(|gb| gb * 1024 * 1024 * 1024)
            .unwrap_or(8 * 1024 * 1024 * 1024) // Default 8GB
    }
    
    fn detect_gpus() -> (Vec<GpuDevice>, bool, Option<String>) {
        let output = Command::new("nvidia-smi")
            .args(["--query-gpu=index,name,memory.total,compute_cap", "--format=csv,noheader,nounits"])
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let gpus: Vec<GpuDevice> = stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                        if parts.len() >= 4 {
                            Some(GpuDevice {
                                index: parts[0].parse().unwrap_or(0),
                                name: parts[1].to_string(),
                                vram_bytes: parts[2].parse::<u64>().unwrap_or(0) * 1024 * 1024,
                                compute_capability: parts[3].to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                
                let cuda_version = Command::new("nvidia-smi")
                    .args(["--query-gpu=driver_version", "--format=csv,noheader"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string());
                
                let has_cuda = !gpus.is_empty();
                (gpus, has_cuda, cuda_version)
            }
            _ => (vec![], false, None),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    SYSTEM CONFIGURATION
// ═══════════════════════════════════════════════════════════════

/// System config aggregating vortex-config + hardware detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Deployment mode from vortex-config
    pub deployment_mode: DeploymentMode,
    /// Detected hardware (unique to this module)
    pub hardware: HardwareCapabilities,
    /// Compute mode from vortex-config
    pub compute_mode: ComputeMode,
    /// Log level
    pub log_level: String,
    /// Feature flags from vortex-config
    pub features: FeatureFlags,
    /// Resource limits from vortex-config
    pub limits: ResourceLimits,
}

impl SystemConfig {
    /// Build system configuration from vortex-config and hardware detection
    pub fn build() -> Self {
        let config = vortex_config::VortexConfig::from_env()
            .unwrap_or_else(|_| vortex_config::ConfigBuilder::sandbox().build().expect("default config"));
        
        let hardware = HardwareCapabilities::detect();
        let compute_mode = if hardware.has_gpu() {
            ComputeMode::Gpu
        } else {
            ComputeMode::Cpu
        };
        
        let log_level = match config.logging.level {
            vortex_config::LogLevel::Trace => "trace",
            vortex_config::LogLevel::Debug => "debug",
            vortex_config::LogLevel::Info => "info",
            vortex_config::LogLevel::Warn => "warn",
            vortex_config::LogLevel::Error => "error",
        }.to_string();
        
        Self {
            deployment_mode: config.mode,
            hardware,
            compute_mode,
            log_level,
            features: config.features,
            limits: config.resources,
        }
    }
    
    /// Log detected configuration
    pub fn log_config(&self) {
        tracing::info!(
            deployment_mode = ?self.deployment_mode,
            compute_mode = ?self.compute_mode,
            cpu_cores = self.hardware.cpu_cores,
            gpus = self.hardware.gpus.len(),
            cuda = self.hardware.cuda_available,
            "System configuration detected"
        );
    }
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let hw = HardwareCapabilities::detect();
        assert!(hw.cpu_cores > 0);
        assert!(hw.ram_bytes > 0);
    }
    
    #[test]
    fn test_system_config_build() {
        let config = SystemConfig::build();
        assert!(config.hardware.cpu_cores > 0);
    }
}
