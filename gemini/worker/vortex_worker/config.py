"""Worker Configuration."""

import os
from dataclasses import dataclass


@dataclass
class WorkerConfig:
    """Configuration for VORTEX worker."""

    slot_id: int
    shm_name: str
    ipc_path: str
    debug: bool = False

    @classmethod
    def from_env(cls) -> "WorkerConfig":
        """Load configuration from environment variables."""
        return cls(
            slot_id=int(os.getenv("VORTEX_SLOT_ID", "0")),
            shm_name=os.getenv("VORTEX_SHM_NAME", "/vortex-shm"),
            ipc_path=os.getenv("VORTEX_IPC_PATH", "/tmp/vortex.sock"),  # nosec B108
            debug=os.getenv("VORTEX_DEBUG", "").lower() in ("1", "true"),
        )
