"""VORTEX Worker Entry Point - P2 Real Executors Implementation."""

import logging
import os
import signal
import sys
import time
from typing import NoReturn

from .config import (
    HEARTBEAT_INTERVAL_MS,
    JOB_TIMEOUT_MS,
    SHM_SIZE_BYTES,
    WorkerConfig,
    runtime_default_device,
)
from .ipc import IPCSocket, Job, JobResult
from .sandbox import enable_sandbox
from .secrets import get_vault_secret
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
    # Fetch critical secrets from Vault before config init
    db_password = get_vault_secret("vortex/prod", "db_password", default=os.getenv("VORTEX_DB_PASSWORD"))

    config = WorkerConfig.from_env()
    # Inject Vault secrets into config
    if db_password:
        config.db_password = db_password

    setup_logging(config)

    logger.info(f"VORTEX Worker starting (slot={config.slot_id})")

    # Enable security sandbox P3
    try:
        enable_sandbox()
    except Exception as e:
        logger.error(f"Failed to enable sandbox: {e}")
        sys.exit(1)

    shutdown = False

    def handle_signal(signum: int, frame) -> None:
        nonlocal shutdown
        logger.info(f"Received signal {signum}, shutting down...")
        shutdown = True

    signal.signal(signal.SIGTERM, handle_signal)
    signal.signal(signal.SIGINT, handle_signal)

    try:
        SHM_SIZE = SHM_SIZE_BYTES
        try:
            shm = ShmArena(config.shm_name, size=SHM_SIZE)
            logger.info(f"Created SHM arena: {config.shm_name} ({SHM_SIZE} bytes)")
        except Exception:
            shm = ShmArena(config.shm_name)
            logger.info(f"Connected to existing SHM arena: {config.shm_name}")

        shm.register_worker(config.slot_id)
        shm.set_worker_status(config.slot_id, 2)

        ipc: IPCSocket | None = None
        try:
            ipc = IPCSocket(config.ipc_path)
            ipc.connect()
            logger.info(f"Connected to IPC socket: {config.ipc_path}")
        except FileNotFoundError:
            ipc = None
            logger.warning(f"IPC socket not found: {config.ipc_path} - standalone mode")
        except Exception as ipc_err:
            ipc = None
            logger.warning(f"IPC connection failed: {ipc_err} - standalone mode")

        logger.info("Entering main event loop")
        while not shutdown:
            shm.update_heartbeat(config.slot_id)

            if ipc is None:
                time.sleep(HEARTBEAT_INTERVAL_MS / 1000)
                continue

            job = ipc.receive(timeout_ms=JOB_TIMEOUT_MS)

            if job is None:
                continue

            logger.info(f"Received job: {job.job_id} for node type: {job.node_type}")

            shm.set_worker_status(config.slot_id, 3)

            try:
                result = execute_job(job, shm, runtime_default_device(config.runtime_mode))
                ipc.send_result(result)
                logger.info(f"Job completed: {job.job_id}")
            except Exception as e:
                logger.error(f"Job failed: {e}")
                shm.set_worker_status(config.slot_id, 4)
                ipc.send_error(job.job_id, str(e))
            finally:
                shm.set_worker_status(config.slot_id, 2)

        logger.info("Worker shutdown complete")

    except Exception as e:
        logger.exception(f"Fatal error: {e}")
        sys.exit(1)

    sys.exit(0)


def execute_job(job: Job, shm: ShmArena, default_device: str) -> JobResult:
    """Execute a compute job using real executors."""
    import json

    # Import extras to register cloud executors
    from . import extras
    from .executor import ExecutorRegistry, TensorHandle

    logger.info(f"Executing job: {job.job_id} (type: {job.node_type})")

    try:
        params = {}
        if job.params_json:
            try:
                params = json.loads(job.params_json)
            except json.JSONDecodeError:
                logger.warning("Invalid JSON params")
                params = {}

        inputs = {}
        for name, input_spec in job.inputs.items():
            if isinstance(input_spec, dict):
                offset = input_spec.get("offset", 0)
                shape = tuple(input_spec.get("shape", []))
                dtype = input_spec.get("dtype", "float32")
                device = input_spec.get("device", default_device)

                if offset == 0 and not shape:
                    inputs[name] = TensorHandle(
                        offset=0, shape=(), dtype=dtype, device=device
                    )
                else:
                    inputs[name] = TensorHandle(
                        offset=offset, shape=shape, dtype=dtype, device=device
                    )
            else:
                logger.warning(f"Unexpected input format for {name}")

        executor_cls = ExecutorRegistry.get(job.node_type)
        if executor_cls is None:
            return JobResult(
                job_id=job.job_id,
                success=False,
                outputs=[],
                error={
                    "code": "EXECUTOR_NOT_FOUND",
                    "message": f"No executor for {job.node_type}",
                },
            )

        start_time = time.perf_counter_ns()
        executor = executor_cls(shm)
        result = executor.execute(inputs, params)
        exec_time_us = (time.perf_counter_ns() - start_time) // 1000

        outputs = []
        for name, handle in result.outputs.items():
            output_spec = {
                "name": name,
                "offset": handle.offset,
                "dtype": handle.dtype,
                "shape": list(handle.shape),
                "device": handle.device,
            }

            if handle.offset > 0 and handle.shape:
                try:
                    tensor_info = shm.get_tensor_info(handle.offset)
                    output_spec["size_bytes"] = tensor_info["total_bytes"]
                except Exception:
                    output_spec["size_bytes"] = 0

            outputs.append(output_spec)

        metrics = {
            "execution_us": (
                result.duration_us if result.duration_us > 0 else exec_time_us
            ),
            "peak_vram_bytes": result.peak_vram_mb * 1024 * 1024,
        }

        if result.success:
            return JobResult(
                job_id=job.job_id,
                success=True,
                outputs=outputs,
                metrics=metrics,
            )
        else:
            return JobResult(
                job_id=job.job_id,
                success=False,
                outputs=[],
                error={
                    "code": "EXECUTION_ERROR",
                    "message": result.error or "Unknown error",
                    "traceback": "",
                },
                metrics=metrics,
            )

    except Exception as e:
        logger.exception(f"Job execution failed: {e}")
        return JobResult(
            job_id=job.job_id,
            success=False,
            outputs=[],
            error={
                "code": "WORKER_ERROR",
                "message": str(e),
                "traceback": "",
            },
            metrics={"execution_us": 0, "peak_vram_bytes": 0},
        )


if __name__ == "__main__":
    main()
