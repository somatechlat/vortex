"""Shared Memory Arena Module.

Provides Python bindings to the VORTEX shared memory arena,
matching the Rust ShmHeader and WorkerSlot structures exactly.
"""

import ctypes
import mmap
import time

# ═══════════════════════════════════════════════════════════════
#                    CTYPES STRUCTURES
# Must match Rust structs exactly!
# ═══════════════════════════════════════════════════════════════

class WorkerSlot(ctypes.Structure):
    """Worker slot in shared memory (64 bytes).

    Must match Rust:
    ```
    #[repr(C)]
    pub struct WorkerSlot {
        pub pid: i32,
        pub status: u32,
        pub last_heartbeat: u64,
        pub current_job: u64,
        pub progress: f32,
        pub reserved: [u8; 36],
    }
    ```
    """
    _fields_ = [
        ("pid", ctypes.c_int32),           # Process ID
        ("status", ctypes.c_uint32),        # WorkerStatus enum
        ("last_heartbeat", ctypes.c_uint64), # Unix timestamp ms
        ("current_job", ctypes.c_uint64),   # Current job ID
        ("progress", ctypes.c_float),       # 0.0 - 1.0
        ("reserved", ctypes.c_uint8 * 36),  # Padding to 64 bytes
    ]

assert ctypes.sizeof(WorkerSlot) == 64, "WorkerSlot must be 64 bytes"


class ShmHeader(ctypes.Structure):
    """Shared memory header (16384 bytes total with slots).

    Layout:
    - Header: 64 bytes
    - Worker slots: 256 * 64 = 16320 bytes
    - Tensor arena starts at offset 16384
    """
    MAX_WORKERS = 256
    MAGIC = 0x5654_5833_0000_0001  # "VTX3" + version

    _fields_ = [
        ("magic", ctypes.c_uint64),
        ("version", ctypes.c_uint32),
        ("num_workers", ctypes.c_uint32),
        ("arena_size", ctypes.c_uint64),
        ("arena_used", ctypes.c_uint64),
        ("lock", ctypes.c_uint32),
        ("reserved", ctypes.c_uint8 * 28),
        ("slots", WorkerSlot * MAX_WORKERS),
    ]


# ═══════════════════════════════════════════════════════════════
#                    SHM ARENA CLASS
# ═══════════════════════════════════════════════════════════════

class ShmArena:
    """Python interface to the VORTEX shared memory arena."""

    def __init__(self, name: str, size: int = 0):
        """Open or create shared memory arena.

        Args:
            name: POSIX shared memory name (e.g., "/vortex-shm")
            size: Size in bytes (0 = open existing)
        """
        import posix_ipc

        self.name = name
        self._created = False

        # Open or create
        if size > 0:
            self.shm = posix_ipc.SharedMemory(
                name,
                posix_ipc.O_CREAT | posix_ipc.O_RDWR,
                size=size,
            )
            self._created = True
        else:
            self.shm = posix_ipc.SharedMemory(name, posix_ipc.O_RDWR)

        # Memory map
        self.mm = mmap.mmap(self.shm.fd, self.shm.size)

        # Cast to header structure
        self.header = ShmHeader.from_buffer(self.mm)

        # Initialize if newly created (magic will be 0)
        if self._created and self.header.magic == 0:
            self.header.magic = ShmHeader.MAGIC
            self.header.version = 1
            self.header.num_workers = 0
            self.header.arena_size = size
            self.header.arena_used = 0
            self.header.lock = 0

        # Validate magic
        if self.header.magic != ShmHeader.MAGIC:
            raise RuntimeError(
                f"Invalid SHM magic: {self.header.magic:#x}, "
                f"expected {ShmHeader.MAGIC:#x}"
            )

    def close(self) -> None:
        """Close the shared memory mapping."""
        self.mm.close()
        self.shm.close_fd()

    def register_worker(self, slot_id: int) -> None:
        """Register this worker in the given slot."""
        import os

        if slot_id >= ShmHeader.MAX_WORKERS:
            raise ValueError(f"Slot ID {slot_id} exceeds max {ShmHeader.MAX_WORKERS}")

        slot = self.header.slots[slot_id]
        slot.pid = os.getpid()
        slot.status = 1  # BOOTING
        slot.last_heartbeat = int(time.time() * 1000)
        slot.progress = 0.0

    def set_status(self, slot_id: int, status: int) -> None:
        """Set worker status.

        Status values:
        - 0: DEAD
        - 1: BOOTING
        - 2: IDLE
        - 3: BUSY
        - 4: ERROR
        """
        self.header.slots[slot_id].status = status

    def update_heartbeat(self, slot_id: int) -> None:
        """Update worker heartbeat timestamp."""
        self.header.slots[slot_id].last_heartbeat = int(time.time() * 1000)

    def set_progress(self, slot_id: int, progress: float) -> None:
        """Set worker job progress (0.0 - 1.0)."""
        self.header.slots[slot_id].progress = max(0.0, min(1.0, progress))

    def get_tensor_offset(self) -> int:
        """Get the offset where tensor data begins."""
        return ctypes.sizeof(ShmHeader)
