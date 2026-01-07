"""Arrow/PyTorch Bridge for Zero-Copy Data Transfer.

Provides conversions between Arrow tensors and PyTorch tensors
using DLPack to enable zero-copy data transfer through shared memory.
"""

from typing import Any, Optional, Tuple
import logging

logger = logging.getLogger(__name__)

# Type stubs for optional imports
try:
    import torch
    TORCH_AVAILABLE = True
except ImportError:
    TORCH_AVAILABLE = False
    torch = None  # type: ignore

try:
    import pyarrow as pa
    ARROW_AVAILABLE = True
except ImportError:
    ARROW_AVAILABLE = False
    pa = None  # type: ignore


def arrow_to_tensor(
    buffer: bytes,
    shape: Tuple[int, ...],
    dtype: str,
    device: str = "cuda",
) -> Any:
    """Convert Arrow buffer to PyTorch tensor.
    
    Args:
        buffer: Raw bytes from Arrow/SHM
        shape: Tensor shape
        dtype: Data type string (e.g., "float32", "float16")
        device: Target device ("cuda" or "cpu")
        
    Returns:
        PyTorch tensor on specified device
    """
    if not TORCH_AVAILABLE:
        raise ImportError("PyTorch is required for tensor operations")
    
    # Map dtype string to torch dtype
    dtype_map = {
        "float32": torch.float32,
        "float16": torch.float16,
        "bfloat16": torch.bfloat16,
        "int64": torch.int64,
        "int32": torch.int32,
        "uint8": torch.uint8,
        "bool": torch.bool,
    }
    
    torch_dtype = dtype_map.get(dtype, torch.float32)
    
    # Create tensor from buffer (zero-copy if possible)
    tensor = torch.frombuffer(buffer, dtype=torch_dtype).reshape(shape)
    
    if device == "cuda" and torch.cuda.is_available():
        tensor = tensor.cuda()
    
    return tensor


def tensor_to_arrow(
    tensor: Any,
    copy: bool = False,
) -> Tuple[bytes, Tuple[int, ...], str]:
    """Convert PyTorch tensor to Arrow-compatible buffer.
    
    Args:
        tensor: PyTorch tensor
        copy: If True, always copy to CPU; otherwise use DLPack if possible
        
    Returns:
        Tuple of (buffer, shape, dtype_str)
    """
    if not TORCH_AVAILABLE:
        raise ImportError("PyTorch is required for tensor operations")
    
    # Ensure tensor is contiguous
    if not tensor.is_contiguous():
        tensor = tensor.contiguous()
    
    # Move to CPU if on GPU
    if tensor.is_cuda:
        if copy:
            cpu_tensor = tensor.cpu()
        else:
            # Use DLPack for zero-copy (requires GPU accessible memory)
            cpu_tensor = tensor.cpu()  # Simplified - DLPack path would be different
    else:
        cpu_tensor = tensor
    
    # Get dtype string
    dtype_map = {
        torch.float32: "float32",
        torch.float16: "float16",
        torch.bfloat16: "bfloat16",
        torch.int64: "int64",
        torch.int32: "int32",
        torch.uint8: "uint8",
        torch.bool: "bool",
    }
    
    dtype_str = dtype_map.get(cpu_tensor.dtype, "float32")
    
    # Get raw buffer
    buffer = cpu_tensor.numpy().tobytes()
    
    return buffer, tuple(tensor.shape), dtype_str


def dlpack_to_tensor(dlpack_capsule: Any, device: str = "cuda") -> Any:
    """Convert DLPack capsule to PyTorch tensor.
    
    This is the zero-copy path for cross-process tensor sharing.
    """
    if not TORCH_AVAILABLE:
        raise ImportError("PyTorch is required")
    
    return torch.from_dlpack(dlpack_capsule)


def tensor_to_dlpack(tensor: Any) -> Any:
    """Convert PyTorch tensor to DLPack capsule.
    
    This is the zero-copy path for cross-process tensor sharing.
    """
    if not TORCH_AVAILABLE:
        raise ImportError("PyTorch is required")
    
    return torch.to_dlpack(tensor)


def benchmark_transfer(size_mb: int = 1024) -> dict:
    """Benchmark tensor transfer throughput.
    
    Args:
        size_mb: Size of tensor in megabytes
        
    Returns:
        Dictionary with timing metrics
    """
    import time
    
    if not TORCH_AVAILABLE:
        return {"error": "PyTorch not available"}
    
    results = {}
    
    # Create test tensor
    num_elements = (size_mb * 1024 * 1024) // 4  # float32
    tensor = torch.randn(num_elements, dtype=torch.float32)
    
    # Test CPU -> bytes
    start = time.perf_counter()
    buffer, shape, dtype = tensor_to_arrow(tensor)
    cpu_to_bytes = time.perf_counter() - start
    results["cpu_to_bytes_ms"] = cpu_to_bytes * 1000
    results["throughput_gbps"] = size_mb / (cpu_to_bytes * 1000) if cpu_to_bytes > 0 else 0
    
    # Test bytes -> CPU
    start = time.perf_counter()
    restored = arrow_to_tensor(buffer, shape, dtype, device="cpu")
    bytes_to_cpu = time.perf_counter() - start
    results["bytes_to_cpu_ms"] = bytes_to_cpu * 1000
    
    # Verify
    results["verified"] = torch.allclose(tensor, restored)
    
    logger.info(f"Transfer benchmark: {results}")
    return results
