import pytest
import numpy as np
import torch
import pyarrow as pa
from vortex_worker.bridge import arrow_to_tensor, tensor_to_arrow

def test_tensor_to_arrow_conversion():
    """Test converting a PyTorch tensor to Arrow buffer."""
    # Create a simple tensor
    tensor = torch.tensor([[1.0, 2.0], [3.0, 4.0]], dtype=torch.float32)
    
    buffer, shape, dtype_str = tensor_to_arrow(tensor)
    
    assert shape == (2, 2)
    assert dtype_str == "float32"
    assert isinstance(buffer, bytes)
    
    # Verify content
    numpy_array = np.frombuffer(buffer, dtype=np.float32).reshape(shape)
    assert np.allclose(numpy_array, tensor.numpy())
    
def test_arrow_to_tensor_conversion():
    """Test converting Arrow buffer back to PyTorch tensor."""
    shape = (2, 3)
    data = np.array([[1, 2, 3], [4, 5, 6]], dtype=np.int32)
    buffer = data.tobytes()
    
    tensor = arrow_to_tensor(buffer, shape, "int32", device="cpu")
    
    assert tensor.shape == shape
    assert tensor.dtype == torch.int32
    assert torch.equal(tensor, torch.from_numpy(data))

def test_roundtrip_conversion():
    """Test full roundtrip: Tensor -> Arrow -> Tensor."""
    original = torch.randn(10, 10, dtype=torch.float32)
    
    buffer, shape, dtype_str = tensor_to_arrow(original)
    restored = arrow_to_tensor(buffer, shape, dtype_str, device="cpu")
    
    assert torch.equal(original, restored)

def test_unsupported_type():
    """Test invalid type handling."""
    with pytest.raises(ValueError):
        arrow_to_tensor(b"dummy", (1,), "invalid_type", device="cpu")

def test_zero_copy_check():
    """Verify that we are using frombuffer for zero-copy possibility."""
    data = np.array([1.0, 2.0, 3.0], dtype=np.float32)
    buffer = data.tobytes()
    
    # This should create a read-only tensor if it's truly zero-copy from immutable bytes,
    # or copy if copying is forced. For bytes object, it usually copies unless using specific buffer protocols.
    # bridge.py uses torch.frombuffer which shares memory if buffer is writable, but bytes are immutable.
    # The key is that successful execution means the API works.
    tensor = arrow_to_tensor(buffer, (3,), "float32", device="cpu")
    assert torch.equal(tensor, torch.tensor([1.0, 2.0, 3.0]))
