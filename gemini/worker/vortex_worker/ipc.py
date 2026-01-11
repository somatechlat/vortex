"""IPC Socket Module for Unix Domain Socket communication.

Protocol Specification:
- 4 bytes (u32 Big-Endian): message length
- N bytes: Protobuf-encoded message

Uses the same protobuf schemas as the Rust host (vortex-protocol).
"""

import socket
import struct
from dataclasses import dataclass
from typing import Optional, Dict, List, Any

# Import generated protobuf classes
from .generated import control


@dataclass
class Job:
    """Job received from the Rust host (wrapper for JobRequest proto)."""
    job_id: str
    node_type: str
    params_json: bytes  # Raw JSON bytes from proto
    inputs: Dict[str, Any]  # TensorInput mapping
    outputs: List[Dict[str, Any]]  # Output specs


@dataclass
class JobResult:
    """Result to send back (wrapper for JobResult proto)."""
    job_id: str
    success: bool
    outputs: List[Dict[str, Any]]
    error: Optional[Dict[str, str]] = None
    metrics: Optional[Dict[str, Any]] = None


class IPCSocket:
    """Unix Domain Socket client for IPC with Rust host.

    Protocol: Length-prefixed protobuf messages
    - 4 bytes (u32 BE): message length
    - N bytes: protobuf-encoded message
    """

    def __init__(self, path: str):
        self.path = path
        self.sock: Optional[socket.socket] = None

    def connect(self) -> None:
        """Connect to the Rust host."""
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.path)
        self.sock.setblocking(False)
        print(f"[IPC] Connected to {self.path}")

    def close(self) -> None:
        """Close the connection."""
        if self.sock:
            self.sock.close()
            self.sock = None

    def receive(self, timeout_ms: int = 1000) -> Optional[Job]:
        """Receive a job from the host.

        Returns None on timeout.
        """
        if not self.sock:
            raise RuntimeError("Socket not connected")

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
                        'offset': tensor_input.tensor.offset,
                        'size_bytes': tensor_input.tensor.size_bytes,
                        'dtype': tensor_input.tensor.dtype,
                        'shape': list(tensor_input.tensor.shape),
                    }
            
            outputs = []
            for spec in request.outputs:
                outputs.append({
                    'name': spec.name,
                    'dtype': spec.dtype,
                    'expected_shape': list(spec.expected_shape),
                })
            
            job = Job(
                job_id=request.job_id,
                node_type=request.node_type,
                params_json=request.params_json,
                inputs=inputs,
                outputs=outputs,
            )
            
            print(f"[IPC] Received job: {job.job_id} for node type: {job.node_type}")
            return job
            
        except Exception as e:
            print(f"[IPC] Decode error: {e}")
            import traceback
            traceback.print_exc()
            return None

    def send_result(self, result: JobResult) -> None:
        """Send job result back to host."""
        if not self.sock:
            raise RuntimeError("Socket not connected")

        # Create JobResult protobuf
        proto_result = control.JobResult()
        proto_result.job_id = result.job_id
        proto_result.success = result.success
        
        # Add outputs
        for output in result.outputs:
            tensor_output = proto_result.outputs.add()
            tensor_output.name = output['name']
            tensor_ref = tensor_output.tensor
            tensor_ref.offset = output.get('offset', 0)
            tensor_ref.size_bytes = output.get('size_bytes', 0)
            tensor_ref.dtype = output.get('dtype', 0)
            if 'shape' in output:
                tensor_ref.shape.extend(output['shape'])
        
        # Add error if present
        if result.error:
            proto_error = proto_result.error
            proto_error.code = result.error.get('code', 'UNKNOWN')
            proto_error.message = result.error.get('message', '')
            proto_error.traceback = result.error.get('traceback', '')
        
        # Add metrics if present
        if result.metrics:
            proto_metrics = proto_result.metrics
            proto_metrics.execution_us = result.metrics.get('execution_us', 0)
            proto_metrics.peak_vram_bytes = result.metrics.get('peak_vram_bytes', 0)
            proto_metrics.tokens_processed = result.metrics.get('tokens_processed', 0)
        
        # Serialize
        data = proto_result.SerializeToString()
        
        # Send length prefix (Big-Endian) + data
        length = struct.pack(">I", len(data))
        self.sock.sendall(length + data)
        print(f"[IPC] Sent result for job: {result.job_id}, success: {result.success}")

    def send_error(self, job_id: str, error: str) -> None:
        """Send error response to host."""
        result = JobResult(
            job_id=job_id,
            success=False,
            outputs=[],
            error={"code": "WORKER_ERROR", "message": error},
        )
        self.send_result(result)

    def _recv_exact(self, n: int) -> Optional[bytes]:
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
