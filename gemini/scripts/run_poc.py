"""
VORTEX POC: "The Laughing Flowers"
Executes a multi-modal generation job via the VORTEX Hybrid Worker.
"""

import sys
import os
import time
import json
import logging
import base64
import numpy as np
import threading

# Add worker path to sys.path to import modules
WORKER_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "../worker")
sys.path.append(WORKER_PATH)

from vortex_worker.ipc import IPCSocket, Job, JobResult
from vortex_worker.config import WorkerConfig

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("poc")

def main():
    logger.info("VORTEX POC: The Laughing Flowers (real flow)")

    if len(sys.argv) < 2:
        logger.error("Usage: python run_poc.py <SAGEMAKER_ENDPOINT_NAME>")
        sys.exit(1)

    endpoint_name = sys.argv[1]
    # Set the env var for the local worker process to pick up
    os.environ["VORTEX_SAGEMAKER_ENDPOINT"] = endpoint_name

    # 1. Connect to Local Worker IPC
    config = WorkerConfig.from_env()
    ipc_path = config.ipc_path

    if not os.path.exists(ipc_path):
        logger.error(f"Worker IPC socket not found at {ipc_path}. Please start the local worker first!")
        sys.exit(1)

    logger.info(f"Connecting to Worker IPC: {ipc_path}")
    ipc = IPCSocket(ipc_path)
    ipc.connect()

    # helper to run a job
    def run_job(step_name, remote_op, params, inputs={}):
        job_id = f"poc-{step_name}-{int(time.time())}"
        logger.info(f"Dispatching step: {step_name} [{remote_op}]")

        # We wrap the real operation in Cloud::SageMaker
        payload_params = params.copy()
        payload_params["remote_op_type"] = remote_op
        # Pass the endpoint name dynamically to the worker
        payload_params["endpoint_name"] = endpoint_name

        job = Job(
            job_id=job_id,
            node_type="Cloud::SageMaker",
            params_json=json.dumps(payload_params),
            inputs=inputs
        )

        ipc.send(job)

        # Waiting Logic
        start_time = time.time()
        while True:
            if time.time() - start_time > 300:
                raise TimeoutError(f"Step {step_name} timed out")

            result = ipc.receive_result(timeout_ms=1000)
            if result and result.job_id == job_id:
                if not result.success:
                    raise RuntimeError(f"Step {step_name} failed: {result.error}")

                logger.info(f"Step {step_name} succeeded")
                return result
            time.sleep(0.1)

    try:
        # STEP 1: Load Model (Real Checkpoint Loader)
        # Using standard SD1.5 ID widely available or pre-cached in container
        run_job(
            "LoadModel",
            "Loader::Checkpoint",
            {"ckpt_name": "runwayml/stable-diffusion-v1-5"}
        )

        # STEP 2: Generate Latents (Real KSampler)
        # We don't send latents, we expect KSampler to generate noise if input missing (as per implementation)
        result = run_job(
            "Generate",
            "Sampler::KSampler",
            {
                "prompt": "A field of sunflowers, cinematic lighting",
                "steps": 20,
                "cfg": 7.5,
                "seed": 42
            }
        )

        logger.info(f"POC completed. Generated {len(result.outputs)} latent tensors.")
        logger.info(f"Metrics: {result.metrics}")

    except Exception as e:
        logger.error(f"POC failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
