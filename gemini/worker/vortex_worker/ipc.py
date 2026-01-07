"""IPC Socket Module for Unix Domain Socket communication."""

import socket
import struct
from dataclasses import dataclass


@dataclass
class Job:
    """Job received from the Rust host."""

    job_id: str
    node_type: str
    params: dict
    input_tensors: list


@dataclass
class JobResult:
    """Result to send back to the Rust host."""

    job_id: str
    output_tensors: list
    success: bool
    error: str | None = None


class IPCSocket:
    """Unix Domain Socket client for IPC with Rust host.

    Protocol: Length-prefixed protobuf messages
    - 4 bytes (u32 BE): message length
    - N bytes: protobuf-encoded message
    """

    def __init__(self, path: str):
        self.path = path
        self.sock: socket.socket | None = None

    def connect(self) -> None:
        """Connect to the Rust host."""
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.path)
        self.sock.setblocking(False)

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
            raise RuntimeError("Socket not connected")

        import select

        timeout_sec = timeout_ms / 1000.0

        ready, _, _ = select.select([self.sock], [], [], timeout_sec)
        if not ready:
            return None

        # Read length prefix
        length_bytes = self._recv_exact(4)
        if not length_bytes:
            return None

        length = struct.unpack(">I", length_bytes)[0]

        # Read message
        data = self._recv_exact(length)
        if not data:
            return None

        # TODO: Decode protobuf message
        # For now, return placeholder
        return Job(
            job_id="placeholder",
            node_type="unknown",
            params={},
            input_tensors=[],
        )

    def send_result(self, result: JobResult) -> None:
        """Send job result back to host."""
        if not self.sock:
            raise RuntimeError("Socket not connected")

        # TODO: Encode protobuf message
        data = b""  # Placeholder

        # Send length prefix + data
        self.sock.sendall(struct.pack(">I", len(data)))
        self.sock.sendall(data)

    def send_error(self, job_id: str, error: str) -> None:
        """Send error response to host."""
        result = JobResult(
            job_id=job_id,
            output_tensors=[],
            success=False,
            error=error,
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
                return None

        return bytes(data)
