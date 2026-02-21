"""Worker Configuration."""

import os
import re
from dataclasses import dataclass
from typing import Optional


def parse_slot_id(value: str) -> int:
    """Parse slot ID from env var - handles pod names like 'worker-abc-123'."""
    if not value:
        return 0
    if value.isdigit():
        return int(value)
    match = re.search(r"(\d+)$", value)
    if match:
        return int(match.group(1)) % 256
    return hash(value) % 256


# Centralized constants
SHM_SIZE_BYTES = 64 * 1024 * 1024  # 64 MB
HEARTBEAT_INTERVAL_MS = 1000
JOB_TIMEOUT_MS = 1000


def runtime_default_device(runtime_mode: str) -> str:
    """Return default tensor device for the selected runtime mode."""
    return "cpu" if runtime_mode == "cpu" else "cuda"


@dataclass
class WorkerConfig:
    """Configuration for VORTEX worker."""

    slot_id: int
    shm_name: str
    ipc_path: str
    runtime_mode: str = "cpu"
    db_password: Optional[str] = None
    debug: bool = False

    @classmethod
    def from_env(cls) -> "WorkerConfig":
        """Load configuration from environment variables."""
        raw_slot = os.getenv("VORTEX_SLOT_ID", "0")
        return cls(
            slot_id=parse_slot_id(raw_slot),
            shm_name=os.getenv("VORTEX_SHM_NAME", "/vortex-shm"),
            ipc_path=os.getenv("VORTEX_IPC_PATH", "/tmp/vortex.sock"),
            runtime_mode=os.getenv("VORTEX_RUNTIME_MODE", "cpu").lower(),
            debug=os.getenv("VORTEX_DEBUG", "").lower() in ("1", "true"),
        )
