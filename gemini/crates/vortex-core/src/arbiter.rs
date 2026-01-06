//! Memory Arbiter - VRAM management and eviction
//!
//! Implements SRS Section 3.2.3 [F-03] Memory Eviction Protocol

use crate::error::{VortexError, VortexResult};
use crate::graph::NodeID;
use std::collections::HashMap;

/// Unique identifier for a tensor in cache
pub type TensorID = String;

/// The Arbiter trait as defined in SRS Section 3.6.2
pub trait ArbiterTrait {
    /// Predicts the peak memory usage of a plan.
    fn predict_usage(&self, plan: &[NodeID]) -> u64;

    /// Determines which tensors to evict to free up `needed` bytes.
    /// Uses LFU (Least Future Used) strategy.
    fn plan_eviction(&self, current_vram: u64, needed: u64, plan: &[NodeID]) -> Vec<TensorID>;
}

/// Tensor metadata for memory tracking
#[derive(Debug, Clone)]
pub struct TensorInfo {
    pub id: TensorID,
    pub size_bytes: u64,
    pub dtype: String,
    pub shape: Vec<usize>,
    pub last_used_step: usize,
    pub next_use_step: Option<usize>,
}

/// Memory cost estimation for a node operation
#[derive(Debug, Clone)]
pub struct OperationCost {
    pub node_id: NodeID,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub peak_bytes: u64,
}

/// Default Arbiter implementation
pub struct Arbiter {
    /// VRAM limit in bytes
    vram_limit: u64,
    
    /// Known operation costs (op_type -> peak memory multiplier)
    cost_table: HashMap<String, f64>,
    
    /// Currently cached tensors
    tensor_cache: HashMap<TensorID, TensorInfo>,
    
    /// Current step in execution
    current_step: usize,
}

impl Arbiter {
    /// Create a new arbiter with the given VRAM limit
    pub fn new(vram_limit_mb: u64) -> Self {
        let mut cost_table = HashMap::new();
        
        // Default cost multipliers (output_size * multiplier = peak)
        cost_table.insert("Loader::Checkpoint".to_string(), 2.5);  // Model + buffers
        cost_table.insert("Loader::Image".to_string(), 1.5);
        cost_table.insert("Sampler::KSampler".to_string(), 3.0);  // Latent + intermediates
        cost_table.insert("Decoder::VAE".to_string(), 4.0);       // Latent -> Image
        cost_table.insert("Encoder::CLIP".to_string(), 1.2);
        
        Self {
            vram_limit: vram_limit_mb * 1024 * 1024,
            cost_table,
            tensor_cache: HashMap::new(),
            current_step: 0,
        }
    }

    /// Add a tensor to the cache
    pub fn cache_tensor(&mut self, tensor: TensorInfo) {
        self.tensor_cache.insert(tensor.id.clone(), tensor);
    }

    /// Remove a tensor from the cache
    pub fn evict_tensor(&mut self, tensor_id: &str) -> Option<TensorInfo> {
        self.tensor_cache.remove(tensor_id)
    }

    /// Get current cache usage in bytes
    pub fn current_cache_bytes(&self) -> u64 {
        self.tensor_cache.values().map(|t| t.size_bytes).sum()
    }

    /// Calculate the cost for a single operation
    pub fn estimate_operation_cost(&self, op_type: &str, output_bytes: u64) -> OperationCost {
        let multiplier = self.cost_table.get(op_type).copied().unwrap_or(1.5);
        let peak_bytes = (output_bytes as f64 * multiplier) as u64;
        
        OperationCost {
            node_id: String::new(),
            input_bytes: output_bytes, // Simplified
            output_bytes,
            peak_bytes,
        }
    }

    /// Calculate tensor size from shape and dtype
    pub fn calculate_tensor_size(shape: &[usize], dtype: &str) -> u64 {
        let elements: usize = shape.iter().product();
        let bytes_per_element = match dtype {
            "float32" | "f32" => 4,
            "float16" | "f16" => 2,
            "int64" | "i64" => 8,
            "int32" | "i32" => 4,
            "uint8" | "u8" => 1,
            _ => 4, // Default to float32
        };
        (elements * bytes_per_element) as u64
    }

    /// LFU eviction strategy - evict tensors with furthest future use
    fn select_eviction_candidates(
        &self,
        needed_bytes: u64,
        plan: &[NodeID],
    ) -> Vec<TensorID> {
        // Calculate future score for each cached tensor (distance to next usage)
        let mut scored: Vec<(TensorID, usize)> = self
            .tensor_cache
            .iter()
            .map(|(id, info)| {
                let score = info.next_use_step.unwrap_or(usize::MAX);
                (id.clone(), score)
            })
            .collect();
        
        // Sort by future score (descending) - evict furthest first
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Select enough tensors to free needed bytes
        let mut freed = 0u64;
        let mut to_evict = Vec::new();
        
        for (tensor_id, _) in scored {
            if freed >= needed_bytes {
                break;
            }
            if let Some(info) = self.tensor_cache.get(&tensor_id) {
                freed += info.size_bytes;
                to_evict.push(tensor_id);
            }
        }
        
        to_evict
    }

    /// Check if execution would cause OOM and return eviction plan
    pub fn prepare_execution(
        &self,
        plan: &[NodeID],
        current_vram: u64,
    ) -> VortexResult<Vec<TensorID>> {
        let predicted_peak = self.predict_usage(plan);
        let total = current_vram.saturating_add(predicted_peak);
        
        if total <= self.vram_limit {
            return Ok(Vec::new()); // No eviction needed
        }
        
        let needed = total - self.vram_limit;
        let evictions = self.plan_eviction(current_vram, needed, plan);
        
        // Check if eviction would free enough
        let evictable: u64 = evictions
            .iter()
            .filter_map(|id| self.tensor_cache.get(id))
            .map(|t| t.size_bytes)
            .sum();
        
        if evictable < needed {
            return Err(VortexError::ResourceExhausted {
                requested_mb: total / (1024 * 1024),
                limit_mb: self.vram_limit / (1024 * 1024),
            });
        }
        
        Ok(evictions)
    }
}

impl Default for Arbiter {
    fn default() -> Self {
        Self::new(8 * 1024) // Default 8GB
    }
}

impl ArbiterTrait for Arbiter {
    fn predict_usage(&self, plan: &[NodeID]) -> u64 {
        // Simplified prediction: sum of estimated peaks
        // In practice, this would use the cost table with actual node info
        plan.len() as u64 * 100 * 1024 * 1024 // 100MB per node estimate
    }

    fn plan_eviction(&self, current_vram: u64, needed: u64, plan: &[NodeID]) -> Vec<TensorID> {
        self.select_eviction_candidates(needed, plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_function() {
        // SRS VCRM: Target(1024,1024,RGBA) returns 4MB
        let shape = vec![1024, 1024, 4]; // RGBA
        let size = Arbiter::calculate_tensor_size(&shape, "uint8");
        
        assert_eq!(size, 4 * 1024 * 1024, "1024x1024 RGBA should be 4MB");
    }

    #[test]
    fn test_float16_size() {
        let shape = vec![1, 4, 64, 64]; // Latent
        let size = Arbiter::calculate_tensor_size(&shape, "float16");
        
        // 1 * 4 * 64 * 64 * 2 bytes = 32768 bytes
        assert_eq!(size, 32768);
    }

    #[test]
    fn test_eviction_selection() {
        let mut arbiter = Arbiter::new(1024); // 1GB limit
        
        // Add some tensors
        arbiter.cache_tensor(TensorInfo {
            id: "tensor_a".to_string(),
            size_bytes: 100 * 1024 * 1024, // 100MB
            dtype: "float32".to_string(),
            shape: vec![1024, 1024, 25],
            last_used_step: 0,
            next_use_step: Some(10), // Used soon
        });
        
        arbiter.cache_tensor(TensorInfo {
            id: "tensor_b".to_string(),
            size_bytes: 200 * 1024 * 1024, // 200MB
            dtype: "float32".to_string(),
            shape: vec![1024, 1024, 50],
            last_used_step: 0,
            next_use_step: Some(100), // Used later
        });
        
        let evictions = arbiter.plan_eviction(0, 150 * 1024 * 1024, &[]);
        
        // Should evict tensor_b first (further future use)
        assert!(evictions.contains(&"tensor_b".to_string()));
    }
}
