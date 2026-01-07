"""Worker Configuration."""

import os
import re
from dataclasses import dataclass


def parse_slot_id(value: str) -> int:
    """Parse slot ID from env var - handles pod names like 'worker-abc-123'."""
    if not value:
        return 0
    # Direct integer
    if value.isdigit():
        return int(value)
    # Extract numeric suffix from pod name
    match = re.search(r"(\d+)$", value)
    if match:
        return int(match.group(1)) % 256  # Keep in byte range
    # Hash-based fallback for non-numeric pod names
    return hash(value) % 256


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
        raw_slot = os.getenv("VORTEX_SLOT_ID", "0")
        return cls(
            slot_id=parse_slot_id(raw_slot),
            shm_name=os.getenv("VORTEX_SHM_NAME", "/vortex-shm"),
            ipc_path=os.getenv("VORTEX_IPC_PATH", "/tmp/vortex.sock"),  # nosec B108
            debug=os.getenv("VORTEX_DEBUG", "").lower() in ("1", "true"),
        )
