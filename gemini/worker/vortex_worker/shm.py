"""Shared memory arena module with DLPack support."""

import ctypes
import mmap
import time
import os
import logging
try:
    import posix_ipc
except ImportError:
    posix_ipc = None

try:
    import torch
    DLPACK_AVAILABLE = True
except ImportError:
    DLPACK_AVAILABLE = False

logger = logging.getLogger(__name__)
if not DLPACK_AVAILABLE:
    logger.warning("torch not available; DLPack features disabled")

class WorkerSlot(ctypes.Structure):
    _fields_ = [
        ("pid", ctypes.c_uint32),
        ("status", ctypes.c_uint32),
        ("current_job_id", ctypes.c_uint64),
        ("last_heartbeat", ctypes.c_uint64),
        ("padding", ctypes.c_uint8 * 40),
    ]

assert ctypes.sizeof(WorkerSlot) == 64, "WorkerSlot must be 64 bytes"

class ShmHeader(ctypes.Structure):
    MAX_WORKERS = 256
    MAGIC = 0x5654_5833_0000_0001

    _fields_ = [
        ("magic", ctypes.c_uint64),
        ("version", ctypes.c_uint32),
        ("flags", ctypes.c_uint32),
        ("clock_tick", ctypes.c_uint64),
        ("reserved", ctypes.c_uint8 * 40),
    ]

assert ctypes.sizeof(ShmHeader) == 64

class TensorHeader(ctypes.Structure):
    _pack_ = 64
    _fields_ = [
        ("magic", ctypes.c_uint64),
        ("dtype_code", ctypes.c_uint8),
        ("dtype_bits", ctypes.c_uint8),
        ("dtype_lanes", ctypes.c_uint8),
        ("reserved1", ctypes.c_uint8),
        ("device_type", ctypes.c_uint8),
        ("device_id", ctypes.c_uint16),
        ("ndim", ctypes.c_uint8),
        ("shape", ctypes.c_int64 * 8),
        ("data_bytes", ctypes.c_uint64),
        ("data_offset", ctypes.c_uint64),
        ("reserved2", ctypes.c_uint8 * 16),
    ]

TENSOR_MAGIC = 0x5453_4552_4F56_5458
assert ctypes.sizeof(TensorHeader) >= 112, f"TensorHeader too small: {ctypes.sizeof(TensorHeader)}"

class ShmArena:
    SHM_NAME = "/vortex-shm"
    SLOTS_OFFSET = 0x40
    TENSOR_DATA_OFFSET = 0x4000

    def __init__(self, name: str | None = None, size: int = 0):
        if posix_ipc is None:
            raise RuntimeError("posix_ipc not available")
        
        self.name = name or self.SHM_NAME
        self._created = False
        
        # Use os constants (already imported at module level)
        O_RDWR = os.O_RDWR
        O_CREAT = os.O_CREAT
        
        if size > 0:
            self.shm = posix_ipc.SharedMemory(self.name, O_CREAT | O_RDWR, size=size)
            self._created = True
        else:
            self.shm = posix_ipc.SharedMemory(self.name, O_RDWR)
        
        self.mm = mmap.mmap(self.shm.fd, self.shm.size)
        self.header = ShmHeader.from_buffer(self.mm)
        
        if self._created and self.header.magic == 0:
            self.header.magic = ShmHeader.MAGIC
            self.header.version = 1
            self.header.flags = 0
            self.header.clock_tick = 0
            for i in range(40):
                self.header.reserved[i] = 0
        
        if self.header.magic != ShmHeader.MAGIC:
            raise RuntimeError(f"Invalid SHM magic: {self.header.magic}, expected {ShmHeader.MAGIC}")
        
        if not DLPACK_AVAILABLE:
            logger.warning("DLPack not available")

    def close(self) -> None:
        self.mm.close()
        self.shm.close_fd()

    def _get_slot_offset(self, slot_id: int) -> int:
        if slot_id >= ShmHeader.MAX_WORKERS:
            raise ValueError(f"Slot ID {slot_id} exceeds max")
        return self.SLOTS_OFFSET + (slot_id * ctypes.sizeof(WorkerSlot))

    def _get_slot(self, slot_id: int) -> WorkerSlot:
        offset = self._get_slot_offset(slot_id)
        slot_bytes = self.mm[offset : offset + ctypes.sizeof(WorkerSlot)]
        return WorkerSlot.from_buffer_copy(slot_bytes)

    def _set_slot(self, slot_id: int, slot: WorkerSlot) -> None:
        offset = self._get_slot_offset(slot_id)
        slot_bytes = bytes(slot)
        self.mm[offset : offset + len(slot_bytes)] = slot_bytes

    def register_worker(self, slot_id: int) -> None:
        # os.getpid() works because os was imported at module level
        slot = self._get_slot(slot_id)
        slot.pid = os.getpid()
        slot.status = 1
        slot.current_job_id = 0
        slot.last_heartbeat = int(time.time() * 1000)
        self._set_slot(slot_id, slot)

    def set_worker_status(self, slot_id: int, status: int) -> None:
        slot = self._get_slot(slot_id)
        slot.status = status
        self._set_slot(slot_id, slot)

    def update_heartbeat(self, slot_id: int) -> None:
        slot = self._get_slot(slot_id)
        slot.last_heartbeat = int(time.time() * 1000)
        self._set_slot(slot_id, slot)

    def store_tensor(self, tensor) -> int:
        if not DLPACK_AVAILABLE:
            raise RuntimeError("DLPack not available")

        shape = list(tensor.shape)
        dtype = tensor.dtype
        device = tensor.device

        data_size = tensor.element_size() * tensor.numel()
        header_size = ctypes.sizeof(TensorHeader)
        total_size = header_size + data_size

        if not hasattr(self, "_alloc_ptr"):
            self._alloc_ptr = self.TENSOR_DATA_OFFSET

        current_offset = self._alloc_ptr
        available = self.mm.size() - current_offset

        if total_size > available:
            raise RuntimeError(f"SHM overflow: need {total_size}, available {available}")

        header = TensorHeader()
        header.magic = TENSOR_MAGIC

        if dtype == torch.float32:
            header.dtype_code = 2
            header.dtype_bits = 32
            header.dtype_lanes = 1
        elif dtype == torch.float16:
            header.dtype_code = 2
            header.dtype_bits = 16
            header.dtype_lanes = 1
        elif dtype == torch.bfloat16:
            header.dtype_code = 2
            header.dtype_bits = 16
            header.dtype_lanes = 1
        elif dtype == torch.int32:
            header.dtype_code = 1
            header.dtype_bits = 32
            header.dtype_lanes = 1
        elif dtype == torch.int64:
            header.dtype_code = 1
            header.dtype_bits = 64
            header.dtype_lanes = 1
        elif dtype == torch.uint8:
            header.dtype_code = 0
            header.dtype_bits = 8
            header.dtype_lanes = 1
        else:
            raise ValueError(f"Unsupported dtype: {dtype}")

        if device.type == "cpu":
            header.device_type = 1
            header.device_id = 0
        elif device.type == "cuda":
            header.device_type = 2
            header.device_id = device.index if device.index else 0
        else:
            raise ValueError(f"Unsupported device: {device.type}")

        header.ndim = len(shape)
        header.data_bytes = data_size
        header.data_offset = current_offset + header_size

        for i in range(8):
            header.shape[i] = shape[i] if i < len(shape) else 0

        header.reserved2 = bytes(16)

        header_bytes = bytes(header)
        self.mm[current_offset : current_offset + len(header_bytes)] = header_bytes

        if device.type != "cpu":
            tensor = tensor.cpu()

        tensor_np = tensor.numpy()
        tensor_data = tensor_np.tobytes()
        self.mm[header.data_offset : header.data_offset + data_size] = tensor_data

        self._alloc_ptr = current_offset + total_size

        return current_offset

    def load_tensor(self, header_offset: int):
        if not DLPACK_AVAILABLE:
            raise RuntimeError("DLPack not available")

        import numpy as np

        header_bytes = self.mm[
            header_offset : header_offset + ctypes.sizeof(TensorHeader)
        ]
        header = TensorHeader.from_buffer_copy(header_bytes)

        if header.magic != TENSOR_MAGIC:
            raise RuntimeError(f"Invalid tensor magic: {hex(header.magic)}")

        shape = tuple(header.shape[i] for i in range(header.ndim))

        if header.dtype_code == 2 and header.dtype_bits == 32:
            dtype = np.float32
        elif header.dtype_code == 2 and header.dtype_bits == 16:
            dtype = np.float16
        elif header.dtype_code == 1 and header.dtype_bits == 32:
            dtype = np.int32
        elif header.dtype_code == 1 and header.dtype_bits == 64:
            dtype = np.int64
        elif header.dtype_code == 0 and header.dtype_bits == 8:
            dtype = np.uint8
        else:
            raise ValueError(
                f"Unsupported DLPack dtype: code={header.dtype_code}, bits={header.dtype_bits}"
            )

        data_bytes = self.mm[
            header.data_offset : header.data_offset + header.data_bytes
        ]
        array = np.frombuffer(data_bytes, dtype=dtype).reshape(shape)
        tensor = torch.from_numpy(array).clone()

        return tensor

    def get_tensor_info(self, header_offset: int) -> dict:
        header_bytes = self.mm[
            header_offset : header_offset + ctypes.sizeof(TensorHeader)
        ]
        header = TensorHeader.from_buffer_copy(header_bytes)

        if header.magic != TENSOR_MAGIC:
            raise RuntimeError("Invalid magic")

        return {
            "shape": tuple(header.shape[i] for i in range(header.ndim)),
            "dtype_code": header.dtype_code,
            "dtype_bits": header.dtype_bits,
            "dtype_lanes": header.dtype_lanes,
            "device_type": header.device_type,
            "device_id": header.device_id,
            "total_bytes": header.data_bytes,
            "data_offset": header.data_offset,
        }

    def get_tensor_data(self, offset: int, shape: tuple, dtype: str) -> bytes:
        header_bytes = self.mm[offset : offset + ctypes.sizeof(TensorHeader)]
        header = TensorHeader.from_buffer_copy(header_bytes)

        if header.magic != TENSOR_MAGIC:
            raise RuntimeError("Invalid tensor magic")

        data_offset = header.data_offset
        data_size = header.data_bytes

        return self.mm[data_offset : data_offset + data_size]

    def get_tensor_data_mut(self, offset: int, size: int) -> memoryview:
        return memoryview(self.mm)[offset : offset + size]

    def allocate_tensor(
        self,
        dtype_code: int,
        dtype_bits: int,
        dtype_lanes: int,
        device_type: int,
        device_id: int,
        shape: list,
        data_bytes: int,
    ) -> int:
        header_size = ctypes.sizeof(TensorHeader)
        total_size = header_size + data_bytes

        if not hasattr(self, "_alloc_ptr"):
            self._alloc_ptr = self.TENSOR_DATA_OFFSET

        current_offset = self._alloc_ptr
        available = self.mm.size() - current_offset

        if total_size > available:
            raise RuntimeError(f"SHM overflow: need {total_size}, available {available}")

        header = TensorHeader()
        header.magic = TENSOR_MAGIC
        header.dtype_code = dtype_code
        header.dtype_bits = dtype_bits
        header.dtype_lanes = dtype_lanes
        header.device_type = device_type
        header.device_id = device_id
        header.ndim = len(shape)
        header.data_bytes = data_bytes
        header.data_offset = current_offset + header_size

        for i, dim in enumerate(shape):
            if i < 8:
                header.shape[i] = dim

        header.reserved2 = bytes(16)

        header_bytes = bytes(header)
        self.mm[current_offset : current_offset + len(header_bytes)] = header_bytes

        self._alloc_ptr = current_offset + total_size

        return current_offset
