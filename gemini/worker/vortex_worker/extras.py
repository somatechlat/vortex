"""
VORTEX Cloud Integration (AWS/SageMaker)
"""

import os
import json
import logging
import time
import base64
import io
import numpy as np
from typing import Any, Dict

from .executor import AbstractExecutor, ExecutionResult, TensorHandle, ExecutorRegistry
from .shm import ShmArena

logger = logging.getLogger(__name__)

@ExecutorRegistry.register("Cloud::SageMaker")
class SageMakerExecutor(AbstractExecutor):
    """
    Executor that offloads inference to an AWS SageMaker Serverless Endpoint.
    Handles Zero-Copy -> JSON -> Zero-Copy transition.
    """
    OP_TYPE = "Cloud::SageMaker"
    # Mapping for cloud-specific operations
    INPUT_TYPES = {"input": "ANY"}
    OUTPUT_TYPES = {"output": "ANY"}

    def __init__(self, shm_arena: ShmArena):
        super().__init__(shm_arena)
        self.region = os.getenv("AWS_REGION", "us-east-1")
        self.endpoint_name = os.getenv("VORTEX_SAGEMAKER_ENDPOINT", "vortex-serverless-endpoint")

        # Lazy initialization of boto3 client to avoid overhead on startup if not used
        self._sm_client = None

    @property
    def sm_client(self):
        if self._sm_client is None:
            import boto3
            self._sm_client = boto3.client("sagemaker-runtime", region_name=self.region)
        return self._sm_client

    def execute(
        self,
        inputs: Dict[str, TensorHandle],
        params: Dict[str, Any],
    ) -> ExecutionResult:
        # Allow dynamic endpoint targeting from params (e.g. from POC script)
        target_endpoint = params.get("endpoint_name", self.endpoint_name)

        logger.info(f"Offloading job to SageMaker Endpoint: {target_endpoint}")
        start_time = time.perf_counter_ns()

        try:
            # 1. Serialize Tensors (SHM -> Base64)
            # We convert tensors to numpy, then to base64 strings for JSON payload.
            # Efficiency Note: For massive tensors, we should use S3 presigned URLs,
            # but for Serverless Inference (max 6GB RAM), base64 is acceptable for latents/images.
            serialized_inputs = {}
            for name, handle in inputs.items():
                if handle.offset == 0:
                    continue

                tensor = self.get_tensor(handle)
                # Move to CPU and numpy
                np_array = tensor.cpu().numpy()

                # Serialize
                buffer = io.BytesIO()
                np.save(buffer, np_array)
                b64_data = base64.b64encode(buffer.getvalue()).decode("utf-8")

                serialized_inputs[name] = {
                    "shape": list(np_array.shape),
                    "dtype": str(np_array.dtype),
                    "data": b64_data
                }

            payload = {
                "inputs": serialized_inputs,
                "params": params,
                "op_type": params.get("remote_op_type", "unknown")
            }

            # 2. Invoke Endpoint
            response = self.sm_client.invoke_endpoint(
                EndpointName=target_endpoint,
                ContentType="application/json",
                Body=json.dumps(payload).encode("utf-8")
            )

            # 3. Deserialize Response
            response_body = response['Body'].read().decode("utf-8")
            result_data = json.loads(response_body)

            if "error" in result_data:
                raise RuntimeError(f"Remote Error: {result_data['error']}")

            # 4. Write back to SHM
            outputs = {}
            import torch

            for name, tensor_data in result_data.get("outputs", {}).items():
                b64_data = tensor_data["data"]
                dtype_str = tensor_data["dtype"]
                shape = tuple(tensor_data["shape"])

                # Decode
                buffer = io.BytesIO(base64.b64decode(b64_data))
                np_array = np.load(buffer)

                # Convert to Torch
                tensor = torch.from_numpy(np_array)

                # Store in SHM
                # Note: Remote tensors usually come back to CPU first, then we might move to CUDA if needed
                # But here we store as-is.
                output_handle = self.put_tensor(tensor, device="cpu") # Default to CPU for safety
                outputs[name] = output_handle

            duration_us = (time.perf_counter_ns() - start_time) // 1000

            return ExecutionResult(
                success=True,
                outputs=outputs,
                duration_us=duration_us,
                peak_vram_mb=0, # Serverless
            )

        except Exception as e:
            logger.exception(f"SageMaker invocation failed: {e}")
            return ExecutionResult(
                success=False,
                outputs={},
                duration_us=(time.perf_counter_ns() - start_time) // 1000,
                peak_vram_mb=0,
                error=str(e)
            )
