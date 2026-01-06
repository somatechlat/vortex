"""VORTEX Worker Entry Point.

This module provides the main entry point for the VORTEX compute worker.
The worker connects to the Rust host via Unix Domain Socket and processes
compute jobs using the shared memory arena.
"""

import logging
import signal
import sys
from typing import NoReturn

from .config import WorkerConfig
from .ipc import IPCSocket
from .shm import ShmArena

logger = logging.getLogger(__name__)


def setup_logging(config: WorkerConfig) -> None:
    """Configure structured logging."""
    level = logging.DEBUG if config.debug else logging.INFO
    logging.basicConfig(
        level=level,
        format='{"time":"%(asctime)s","level":"%(levelname)s","msg":"%(message)s"}',
        stream=sys.stdout,
    )


def main() -> NoReturn:
    """Main worker entry point."""
    # Load configuration from environment
    config = WorkerConfig.from_env()
    setup_logging(config)

    logger.info(f"VORTEX Worker starting (slot={config.slot_id})")

    # Setup signal handlers
    shutdown = False

    def handle_signal(signum: int, frame) -> None:
        nonlocal shutdown
        logger.info(f"Received signal {signum}, shutting down...")
        shutdown = True

    signal.signal(signal.SIGTERM, handle_signal)
    signal.signal(signal.SIGINT, handle_signal)

    try:
        # Connect to shared memory
        shm = ShmArena(config.shm_name)
        logger.info(f"Connected to SHM arena: {config.shm_name}")

        # Connect to IPC socket
        ipc = IPCSocket(config.ipc_path)
        ipc.connect()
        logger.info(f"Connected to IPC socket: {config.ipc_path}")

        # Register worker slot
        shm.register_worker(config.slot_id)

        # Main event loop
        logger.info("Entering main event loop")
        while not shutdown:
            # Wait for job from host
            job = ipc.receive(timeout_ms=1000)

            if job is None:
                # Update heartbeat
                shm.update_heartbeat(config.slot_id)
                continue

            logger.info(f"Received job: {job.job_id}")

            # Mark as busy
            shm.set_status(config.slot_id, 3)  # BUSY

            try:
                # TODO: Execute job
                result = execute_job(job, shm)
                ipc.send_result(result)
                logger.info(f"Job completed: {job.job_id}")
            except Exception as e:
                logger.error(f"Job failed: {e}")
                shm.set_status(config.slot_id, 4)  # ERROR
                ipc.send_error(job.job_id, str(e))
            finally:
                shm.set_status(config.slot_id, 2)  # IDLE

        logger.info("Worker shutdown complete")

    except Exception as e:
        logger.exception(f"Fatal error: {e}")
        sys.exit(1)

    sys.exit(0)


def execute_job(job, shm: ShmArena):
    """Execute a compute job.

    TODO: Implement job execution with executor registry.
    """
    # Placeholder - will be implemented in Phase 3
    pass


if __name__ == "__main__":
    main()
