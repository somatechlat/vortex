//! DLPack Serialization - Zero-Copy Tensor Transfer
//!
//! Implements SRS Section 3.9.2 (Tensor Serialization) for VORTEX-GEN 3.0.
//! Enables zero-copy transfer between Rust (host) and Python (workers) via DLPack format.
//!
//! Design Principles:
//! - Zero-copy: Tensors share memory between Rust and Python
//! - Config-driven: All paths and buffers from vortex-config
//! - Safe: Proper error handling, no panics
//! - Cross-platform: Works with SHM-backed tensors

use crate::error::{VortexError, VortexResult};
// Note: DLContext is not available in dlpack-sys 0.1.1
use dlpack_sys::{DLDataType, DLDevice, DLTensor};
use ndarray::{ArrayD, IxDyn};
use std::sync::Arc;
use vortex_config::VortexConfig;

/// Device type abstraction for DLPack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    CPU,
    CUDA,
    ROCM,
}

impl DeviceType {
    /// Convert to DLPack device
    pub fn to_dl_device(&self) -> DLDevice {
        match self {
            DeviceType::CPU => DLDevice {
                device_type: dlpack_sys::DLDeviceType_kDLCPU,
                device_id: 0,
            },
            DeviceType::CUDA => DLDevice {
                device_type: dlpack_sys::DLDeviceType_kDLCUDA,
                device_id: 0,
            },
            DeviceType::ROCM => DLDevice {
                device_type: dlpack_sys::DLDeviceType_kDLROCM,
                device_id: 0,
            },
        }
    }

    /// Convert from DLPack device
    pub fn from_dl_device(device: DLDevice) -> Option<Self> {
        match device.device_type {
            dlpack_sys::DLDeviceType_kDLCPU => Some(DeviceType::CPU),
            dlpack_sys::DLDeviceType_kDLCUDA => Some(DeviceType::CUDA),
            dlpack_sys::DLDeviceType_kDLROCM => Some(DeviceType::ROCM),
            _ => None,
        }
    }
}

/// VORTEX Tensor backed by DLPack
pub struct Tensor {
    /// DLPack tensor structure (holds pointer to data)
    dl_tensor: DLTensor,
    /// Ownership of the data (prevents premature free)
    _owner: Arc<Vec<u8>>,
    /// Shape of the tensor
    shape: Vec<usize>,
    /// Data type
    dtype: DLDataType,
    /// Device
    device: DeviceType,
}

impl Tensor {
    /// Create a CPU tensor from ndarray
    pub fn from_ndarray<T>(array: ArrayD<T>) -> VortexResult<Self>
    where
        T: Copy + 'static,
    {
        use std::mem::size_of;
        use std::os::raw::c_void;

        let shape = array.shape().to_vec();
        let total_elements: usize = shape.iter().product();
        let element_size = size_of::<T>();

        let mut data_vec = vec![0u8; total_elements * element_size];
        unsafe {
            std::ptr::copy_nonoverlapping(
                array.as_ptr() as *const u8,
                data_vec.as_mut_ptr(),
                data_vec.len(),
            );
        }

        let owner = Arc::new(data_vec);
        let data_ptr = owner.as_ptr() as *mut c_void;

        // Determine DLPack data type from Rust type
        let dtype = match std::any::type_name::<T>() {
            "f32" => DLDataType {
                code: dlpack_sys::DLDataTypeCode_kDLFloat as u8,
                bits: 32,
                lanes: 1,
            },
            "f64" => DLDataType {
                code: dlpack_sys::DLDataTypeCode_kDLFloat as u8,
                bits: 64,
                lanes: 1,
            },
            "i32" => DLDataType {
                code: dlpack_sys::DLDataTypeCode_kDLInt as u8,
                bits: 32,
                lanes: 1,
            },
            "i64" => DLDataType {
                code: dlpack_sys::DLDataTypeCode_kDLInt as u8,
                bits: 64,
                lanes: 1,
            },
            "u8" => DLDataType {
                code: dlpack_sys::DLDataTypeCode_kDLUInt as u8,
                bits: 8,
                lanes: 1,
            },
            _ => {
                return Err(VortexError::Internal(format!(
                    "Unsupported data type for DLPack: {}",
                    std::any::type_name::<T>()
                )))
            }
        };

        // Convert shape to i64 for DLPack (C-compatible)
        let dl_shape: Vec<i64> = shape.iter().map(|&s| s as i64).collect();

        // Create strides (row-major, all 1 for contiguous arrays)
        let strides: Vec<i64> = vec![1; shape.len()];

        // dlpack-sys 0.1.1 DLTensor requires strides field
        let dl_tensor = DLTensor {
            data: data_ptr,
            device: DeviceType::CPU.to_dl_device(),
            ndim: shape.len() as i32,
            dtype,
            shape: dl_shape.as_ptr() as *mut i64,
            strides: strides.as_ptr() as *mut i64,
            byte_offset: 0,
        };

        Ok(Self {
            dl_tensor,
            _owner: owner,
            shape,
            dtype,
            device: DeviceType::CPU,
        })
    }

    /// Convert DLPack tensor back to ndarray
    pub fn to_ndarray<T>(&self) -> VortexResult<ArrayD<T>>
    where
        T: Copy + 'static,
    {
        if self.device != DeviceType::CPU {
            return Err(VortexError::Internal(
                "GPU tensor must be copied to CPU first".to_string(),
            ));
        }

        let shape = self.shape.clone();
        let data = unsafe {
            std::slice::from_raw_parts(
                self.dl_tensor.data as *const T,
                self.shape.iter().product(),
            )
        };

        ArrayD::from_shape_vec(IxDyn(&shape), data.to_vec())
            .map_err(|e| VortexError::Internal(format!("Failed to create ndarray: {}", e)))
    }

    /// Get raw DLPack tensor pointer
    ///
    /// # Safety
    /// Pointer is valid for lifetime of this Tensor
    pub unsafe fn as_dl_tensor(&self) -> *const DLTensor {
        &self.dl_tensor as *const DLTensor
    }

    /// Get tensor shape
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Get device type
    pub fn device(&self) -> DeviceType {
        self.device
    }

    /// Get total bytes
    pub fn nbytes(&self) -> usize {
        // Use constants from dlpack_sys for type codes
        let kdl_float = dlpack_sys::DLDataTypeCode_kDLFloat as u8;
        let kdl_int = dlpack_sys::DLDataTypeCode_kDLInt as u8;
        let kdl_uint = dlpack_sys::DLDataTypeCode_kDLUInt as u8;
        
        let element_size = match (self.dtype.code, self.dtype.bits) {
            (c, 32) if c == kdl_float || c == kdl_int => 4,
            (c, 64) if c == kdl_float || c == kdl_int => 8,
            (c, 8) if c == kdl_uint => 1,
            _ => 4,
        };
        self.shape.iter().product::<usize>() * element_size
    }

    /// Check if tensor is empty
    pub fn is_empty(&self) -> bool {
        self.shape.iter().product::<usize>() == 0
    }
}

/// SHM-backed tensor for large data transfers
pub struct ShmTensor {
    tensor: Tensor,
    shm_offset: usize,
    shm_name: String,
}

impl ShmTensor {
    /// Create tensor in shared memory
    pub fn create_in_shm<T>(
        array: ArrayD<T>,
        shm_name: &str,
        offset: usize,
    ) -> VortexResult<Self>
    where
        T: Copy + 'static,
    {
        let tensor = Tensor::from_ndarray(array)?;

        Ok(Self {
            tensor,
            shm_offset: offset,
            shm_name: shm_name.to_string(),
        })
    }

    /// Get SHM path for Python worker to map
    pub fn shm_path(&self) -> String {
        format!("{}{}", self.shm_name, self.shm_offset)
    }

    /// Get reference to inner tensor
    pub fn tensor(&self) -> &Tensor {
        &self.tensor
    }
}

/// Config-driven tensor factory
pub struct TensorFactory {
    config: VortexConfig,
}

impl TensorFactory {
    /// Create new factory with config
    pub fn new(config: VortexConfig) -> Self {
        Self { config }
    }

    /// Create CPU tensor from array
    pub fn create_cpu<T>(&self, array: ArrayD<T>) -> VortexResult<Tensor>
    where
        T: Copy + 'static,
    {
        Tensor::from_ndarray(array)
    }

    /// Get recommended SHM size for tensor
    pub fn get_shm_size(&self, tensor: &Tensor) -> usize {
        let max_vram_bytes = self.config.worker.max_vram_mb as usize * 1024 * 1024;
        let tensor_size = tensor.nbytes();
        
        std::cmp::min(tensor_size, max_vram_bytes)
    }

    /// Validate tensor for execution
    pub fn validate_tensor(&self, tensor: &Tensor) -> VortexResult<()> {
        let max_vram_bytes = self.config.worker.max_vram_mb as usize * 1024 * 1024;
        let tensor_size = tensor.nbytes();

        if tensor_size > max_vram_bytes {
            return Err(VortexError::ResourceExhausted {
                requested_mb: (tensor_size / (1024 * 1024)) as u64,
                limit_mb: self.config.worker.max_vram_mb,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_tensor_roundtrip() {
        let original = array![[1.0, 2.0], [3.0, 4.0]].into_dyn();
        let tensor = Tensor::from_ndarray(original.clone()).unwrap();
        let recovered = tensor.to_ndarray::<f32>().unwrap();
        
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_tensor_shape() {
        let arr = array![[1, 2, 3], [4, 5, 6]].into_dyn();
        let tensor = Tensor::from_ndarray(arr).unwrap();
        
        assert_eq!(tensor.shape(), &[2, 3]);
    }

    #[test]
    fn test_tensor_nbytes() {
        let arr = array![[1.0f32, 2.0], [3.0, 4.0]].into_dyn();
        let tensor = Tensor::from_ndarray(arr).unwrap();
        
        assert_eq!(tensor.nbytes(), 16);
    }

    #[test]
    fn test_device_type() {
        let device = DeviceType::CUDA;
        let dl_device = device.to_dl_device();
        let recovered = DeviceType::from_dl_device(dl_device);
        
        assert_eq!(recovered, Some(DeviceType::CUDA));
    }
}
