//! System Deployment Configuration
//!
//! Handles deployment modes (SANDBOX/LIVE) and hardware auto-detection.
//! Part of the SaaS administration system.

use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

// ═══════════════════════════════════════════════════════════════
//                    DEPLOYMENT MODES
// ═══════════════════════════════════════════════════════════════

/// System-wide deployment mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentMode {
    /// Development/Testing - CPU inference, debug logging, relaxed limits
    Sandbox,
    /// Production - GPU inference (if available), error logging, strict limits
    Live,
}

impl DeploymentMode {
    pub fn from_env() -> Self {
        match env::var("VORTEX_DEPLOYMENT_MODE")
            .unwrap_or_else(|_| "sandbox".to_string())
            .to_lowercase()
            .as_str()
        {
            "live" | "production" | "prod" => Self::Live,
            _ => Self::Sandbox,
        }
    }
    
    pub fn is_sandbox(&self) -> bool {
        matches!(self, Self::Sandbox)
    }
    
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live)
    }
}

// ═══════════════════════════════════════════════════════════════
//                    HARDWARE DETECTION
// ═══════════════════════════════════════════════════════════════

/// Detected hardware capabilities
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
        // Use sysinfo crate in production
        // Fallback to env var or default
        env::var("VORTEX_AVAILABLE_RAM_GB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(|gb| gb * 1024 * 1024 * 1024)
            .unwrap_or(8 * 1024 * 1024 * 1024) // Default 8GB
    }
    
    fn detect_gpus() -> (Vec<GpuDevice>, bool, Option<String>) {
        // Try nvidia-smi for NVIDIA GPUs
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
                
                // Get CUDA version
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
//                    COMPUTE MODE
// ═══════════════════════════════════════════════════════════════

/// Compute mode for inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputeMode {
    Cpu,
    Gpu,
}

impl ComputeMode {
    pub fn from_env_or_detect(hw: &HardwareCapabilities) -> Self {
        // Check explicit override
        if let Ok(mode) = env::var("VORTEX_COMPUTE_MODE") {
            match mode.to_lowercase().as_str() {
                "gpu" | "cuda" => return Self::Gpu,
                "cpu" => return Self::Cpu,
                _ => {}
            }
        }
        
        // Auto-detect
        hw.recommended_compute_mode()
    }
}

// ═══════════════════════════════════════════════════════════════
//                    SYSTEM CONFIGURATION
// ═══════════════════════════════════════════════════════════════

/// Complete system deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Deployment mode (sandbox/live)
    pub deployment_mode: DeploymentMode,
    /// Detected hardware
    pub hardware: HardwareCapabilities,
    /// Compute mode (cpu/gpu)
    pub compute_mode: ComputeMode,
    /// Log level based on deployment mode
    pub log_level: String,
    /// Feature flags
    pub features: FeatureFlags,
    /// Resource limits
    pub limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub debug_panel: bool,
    pub admin_tools: bool,
    pub billing: bool,
    pub usage_limits: bool,
    pub swagger: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_concurrent_jobs: usize,
    pub max_vram_per_job_mb: u64,
    pub api_rate_limit: u32,
}

impl SystemConfig {
    /// Build system configuration from environment and hardware detection
    pub fn build() -> Self {
        let deployment_mode = DeploymentMode::from_env();
        let hardware = HardwareCapabilities::detect();
        let compute_mode = ComputeMode::from_env_or_detect(&hardware);
        
        let (log_level, features, limits) = match deployment_mode {
            DeploymentMode::Sandbox => (
                "debug".to_string(),
                FeatureFlags {
                    debug_panel: true,
                    admin_tools: true,
                    billing: false,
                    usage_limits: false,
                    swagger: true,
                },
                ResourceLimits {
                    max_concurrent_jobs: 2,
                    max_vram_per_job_mb: 4096,
                    api_rate_limit: 1000,
                },
            ),
            DeploymentMode::Live => (
                "error".to_string(),
                FeatureFlags {
                    debug_panel: false,
                    admin_tools: false,
                    billing: true,
                    usage_limits: true,
                    swagger: false,
                },
                ResourceLimits {
                    max_concurrent_jobs: 32,
                    max_vram_per_job_mb: 24576,
                    api_rate_limit: 100,
                },
            ),
        };
        
        Self {
            deployment_mode,
            hardware,
            compute_mode,
            log_level,
            features,
            limits,
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
    fn test_deployment_mode_from_env() {
        std::env::set_var("VORTEX_DEPLOYMENT_MODE", "live");
        assert_eq!(DeploymentMode::from_env(), DeploymentMode::Live);
        
        std::env::set_var("VORTEX_DEPLOYMENT_MODE", "sandbox");
        assert_eq!(DeploymentMode::from_env(), DeploymentMode::Sandbox);
    }
    
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
