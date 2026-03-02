"""IPC Socket Module for Unix Domain Socket communication.

Protocol Specification:
- 4 bytes (u32 Big-Endian): message length
- N bytes: Protobuf-encoded message

Uses the same protobuf schemas as the Rust host (vortex-protocol).
"""

import socket
import struct
import logging
import time
from dataclasses import dataclass
from typing import Any

# Import generated protobuf classes
from .generated import control

logger = logging.getLogger(__name__)


@dataclass
class Job:
    """Job received from the Rust host (wrapper for JobRequest proto)."""

    job_id: str
    node_type: str
    params_json: bytes  # Raw JSON bytes from proto
    inputs: dict[str, Any]  # TensorInput mapping
    outputs: list[dict[str, Any]]  # Output specs


@dataclass
class JobResult:
    """Result to send back (wrapper for JobResult proto)."""

    job_id: str
    success: bool
    outputs: list[dict[str, Any]]
    error: dict[str, str] | None = None
    metrics: dict[str, Any] | None = None


class IPCSocket:
    """Unix Domain Socket client for IPC with Rust host.

    Protocol: Length-prefixed protobuf messages
    - 4 bytes (u32 BE): message length
    - N bytes: protobuf-encoded message
    """

    def __init__(self, path: str, max_retries: int = 10):
        self.path = path
        self.sock: socket.socket | None = None
        self.max_retries = max_retries

    def connect(self) -> None:
        """Connect to the Rust host with exponential backoff."""
        delay = 0.5
        for attempt in range(1, self.max_retries + 1):
            try:
                self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                self.sock.connect(self.path)
                self.sock.setblocking(False)
                logger.info("IPC connected: %s (attempt %d)", self.path, attempt)
                return
            except (ConnectionRefusedError, FileNotFoundError, OSError) as e:
                logger.warning(
                    "IPC connect failed (attempt %d/%d): %s. Retrying in %.1fs...",
                    attempt, self.max_retries, e, delay,
                )
                if self.sock:
                    self.sock.close()
                    self.sock = None
                time.sleep(delay)
                delay = min(delay * 2, 30.0)

        raise ConnectionError(
            f"IPC connection to {self.path} failed after {self.max_retries} attempts"
        )

    def _reconnect(self) -> None:
        """Attempt to reconnect after a socket failure."""
        logger.warning("IPC connection lost, attempting reconnect...")
        self.close()
        self.connect()

    def close(self) -> None:
        """Close the connection."""
        if self.sock:
            self.sock.close()
            self.sock = None

    def receive(self, timeout_ms: int = 1000) -> Job | None:
        """Receive a job from the host.

        Returns None on timeout.
        """
        if not self.sock:
            self._reconnect()

        import select

        timeout_sec = timeout_ms / 1000.0

        ready, _, _ = select.select([self.sock], [], [], timeout_sec)
        if not ready:
            return None

        # Read length prefix (4 bytes, Big-Endian)
        length_bytes = self._recv_exact(4)
        if not length_bytes:
            return None

        length = struct.unpack(">I", length_bytes)[0]

        # Read message
        data = self._recv_exact(length)
        if not data:
            return None

        # Decode protobuf message using generated class
        try:
            request = control.JobRequest()
            request.ParseFromString(data)

            # Convert to Job wrapper
            inputs = {}
            for tensor_input in request.inputs:
                if tensor_input.tensor:
                    inputs[tensor_input.name] = {
                        "offset": tensor_input.tensor.offset,
                        "size_bytes": tensor_input.tensor.size_bytes,
                        "dtype": tensor_input.tensor.dtype,
                        "shape": list(tensor_input.tensor.shape),
                    }

            outputs = []
            for spec in request.outputs:
                outputs.append(
                    {
                        "name": spec.name,
                        "dtype": spec.dtype,
                        "expected_shape": list(spec.expected_shape),
                    }
                )

            job = Job(
                job_id=request.job_id,
                node_type=request.node_type,
                params_json=request.params_json,
                inputs=inputs,
                outputs=outputs,
            )

            logger.debug("IPC received job_id=%s node_type=%s", job.job_id, job.node_type)
            return job

        except Exception as e:
            logger.exception("IPC decode error: %s", e)
            return None

    def send_result(self, result: JobResult) -> None:
        """Send job result back to host."""
        if not self.sock:
            self._reconnect()

        # Create JobResult protobuf
        proto_result = control.JobResult()
        proto_result.job_id = result.job_id
        proto_result.success = result.success

        # Add outputs
        for output in result.outputs:
            tensor_output = proto_result.outputs.add()
            tensor_output.name = output["name"]
            tensor_ref = tensor_output.tensor
            tensor_ref.offset = output.get("offset", 0)
            tensor_ref.size_bytes = output.get("size_bytes", 0)
            tensor_ref.dtype = output.get("dtype", 0)
            if "shape" in output:
                tensor_ref.shape.extend(output["shape"])

        # Add error if present
        if result.error:
            proto_error = proto_result.error
            proto_error.code = result.error.get("code", "UNKNOWN")
            proto_error.message = result.error.get("message", "")
            proto_error.traceback = result.error.get("traceback", "")

        # Add metrics if present
        if result.metrics:
            proto_metrics = proto_result.metrics
            proto_metrics.execution_us = result.metrics.get("execution_us", 0)
            proto_metrics.peak_vram_bytes = result.metrics.get("peak_vram_bytes", 0)
            proto_metrics.tokens_processed = result.metrics.get("tokens_processed", 0)

        # Serialize
        data = proto_result.SerializeToString()

        # Send length prefix (Big-Endian) + data
        length = struct.pack(">I", len(data))
        self.sock.sendall(length + data)
        logger.debug("IPC sent result job_id=%s success=%s", result.job_id, result.success)

    def send_error(self, job_id: str, error: str) -> None:
        """Send error response to host."""
        result = JobResult(
            job_id=job_id,
            success=False,
            outputs=[],
            error={"code": "WORKER_ERROR", "message": error},
        )
        self.send_result(result)

    def _recv_exact(self, n: int) -> bytes | None:
        """Receive exactly n bytes."""
        if not self.sock:
            return None

        data = bytearray()
        while len(data) < n:
            try:
                chunk = self.sock.recv(n - len(data))
                if not chunk:
                    return None
                data.extend(chunk)
            except BlockingIOError:
                # Would block, but we might have partial data
                if len(data) == 0:
                    return None
                # Continue waiting for remaining bytes
                continue

        return bytes(data)
