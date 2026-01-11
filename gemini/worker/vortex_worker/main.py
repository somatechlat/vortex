"""VORTEX Worker Entry Point.

This module provides the main entry point for the VORTEX compute worker.
The worker connects to the Rust host via Unix Domain Socket and processes
compute jobs using the shared memory arena.

Protocol: Protobuf over UDS with Big-Endian length prefix
"""

import logging
import signal
import sys
from typing import NoReturn, Optional

from .config import WorkerConfig
from .ipc import IPCSocket, Job, JobResult
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
        # Connect/create shared memory arena
        SHM_SIZE = 64 * 1024 * 1024  # 64 MB
        try:
            shm = ShmArena(config.shm_name, size=SHM_SIZE)
            logger.info(f"Created SHM arena: {config.shm_name} ({SHM_SIZE} bytes)")
        except Exception:
            # Try to open existing if create fails
            shm = ShmArena(config.shm_name)
            logger.info(f"Connected to existing SHM arena: {config.shm_name}")

        # Register worker slot
        shm.register_worker(config.slot_id)
        shm.set_status(config.slot_id, 2)  # IDLE

        # Try to connect to IPC socket
        ipc: Optional[IPCSocket] = None
        try:
            ipc = IPCSocket(config.ipc_path)
            ipc.connect()
            logger.info(f"Connected to IPC socket: {config.ipc_path}")
        except FileNotFoundError:
            ipc = None
            logger.warning(f"IPC socket not found: {config.ipc_path} - running in standalone mode")
        except Exception as ipc_err:
            ipc = None
            logger.warning(f"IPC connection failed: {ipc_err} - running in standalone mode")

        # Main event loop
        logger.info("Entering main event loop")
        while not shutdown:
            # Update heartbeat
            shm.update_heartbeat(config.slot_id)

            # If no IPC, just keep heartbeat alive
            if ipc is None:
                import time
                time.sleep(1)
                continue

            # Wait for job from host
            job = ipc.receive(timeout_ms=1000)

            if job is None:
                # No job, continue heartbeat polling
                continue

            logger.info(f"Received job: {job.job_id} for node type: {job.node_type}")

            # Mark as busy
            shm.set_status(config.slot_id, 3)  # BUSY

            try:
                # Execute the job
                result = execute_job(job, shm)
                
                # Send result back
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


def execute_job(job: Job, shm: ShmArena) -> JobResult:
    """Execute a compute job.
    
    This is the main execution entry point. It:
    1. Looks up the executor for the node type
    2. Loads input tensors from shared memory
    3. Executes the operation
    4. Writes output tensors to shared memory
    5. Returns metrics and results
    
    Args:
        job: Job request from host
        shm: Shared memory arena for tensor data
        
    Returns:
        JobResult with execution status and output references
    """
    logger.info(f"Executing job: {job.job_id} (type: {job.node_type})")
    
    # In production, this would:
    # 1. Look up executor from registry
    # 2. Load inputs from SHM
    # 3. Execute: result = executor.execute(job.params, inputs)
    # 4. Allocate output in SHM
    # 5. Serialize result to SHM
    # 6. Return JobResult with output refs
    
    # For now, stub implementation
    import time
    time.sleep(0.1)  # Simulate work
    
    return JobResult(
        job_id=job.job_id,
        success=True,
        outputs=[],  # Would contain tensor refs
        metrics={
            "execution_us": 100000,  # 100ms
            "peak_vram_bytes": 0,
            "tokens_processed": 0,
        },
    )


if __name__ == "__main__":
    main()
