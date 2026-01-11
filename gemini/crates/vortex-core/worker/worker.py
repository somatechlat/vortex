#!/usr/bin/env python3
"""
VORTEX Worker - Python Compute Process

Implements SRS Section 3.6.1 (Executor ABC) and Section 3.10 (Worker Execution Flow)

This worker:
1. Connects to the Host via Unix Domain Socket
2. Maps shared memory for Zero-Copy tensor access
3. Executes jobs dispatched by the Supervisor
4. Reports results back over IPC
"""

import argparse
import json
import logging
import mmap
import os
import signal
import socket
import struct
import sys
import time
import traceback
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import IntEnum
from pathlib import Path
from typing import Any, Dict, List, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='[WORKER %(process)d] %(asctime)s %(levelname)s: %(message)s'
)
logger = logging.getLogger(__name__)

# ============================================================================
# Constants (matching SRS Section 3.4)
# ============================================================================

SOCKET_PATH = "/tmp/vortex.sock"
SHM_NAME = "/vortex-shm"
PROTOCOL_VERSION = 1

# Worker status (matching shm.rs)
class WorkerStatus(IntEnum):
    IDLE = 0
    BUSY = 1
    DEAD = 2
    BOOTING = 3


# ============================================================================
# IPC Protocol (matching ipc.rs)
# ============================================================================

@dataclass
class ControlPacket:
    request_id: str
    timestamp: int
    payload: Dict[str, Any]
    
    def to_bytes(self) -> bytes:
        data = json.dumps({
            "request_id": self.request_id,
            "timestamp": self.timestamp,
            "payload": self.payload
        }).encode('utf-8')
        return struct.pack('<I', len(data)) + data
    
    @classmethod
    def from_bytes(cls, data: bytes) -> 'ControlPacket':
        length = struct.unpack('<I', data[:4])[0]
        payload = json.loads(data[4:4+length])
        return cls(
            request_id=payload['request_id'],
            timestamp=payload['timestamp'],
            payload=payload['payload']
        )


def recv_packet(sock: socket.socket) -> ControlPacket:
    """Receive a length-prefixed packet"""
    # Read length (4 bytes)
    len_data = sock.recv(4)
    if len(len_data) < 4:
        raise ConnectionError("Connection closed")
    
    length = struct.unpack('<I', len_data)[0]
    
    # Read payload
    payload = b''
    while len(payload) < length:
        chunk = sock.recv(length - len(payload))
        if not chunk:
            raise ConnectionError("Connection closed")
        payload += chunk
    
    return ControlPacket.from_bytes(len_data + payload)


def send_packet(sock: socket.socket, packet: ControlPacket):
    """Send a length-prefixed packet"""
    sock.sendall(packet.to_bytes())


# ============================================================================
# Executor Interface (SRS Section 3.6.1)
# ============================================================================

class Executor(ABC):
    """Abstract base class for all compute nodes"""
    
    @abstractmethod
    def execute(
        self, 
        inputs: Dict[str, Any], 
        params: Dict[str, Any]
    ) -> Any:
        """
        Core logic for the node.
        
        Args:
            inputs: Map of Input Name -> Tensor (Zero-Copy from SHM)
            params: Dictionary of configuration literals
            
        Returns:
            The output Tensor (to be exported to SHM)
        """
        pass
    
    def cleanup(self):
        """Optional hook for releasing heavy resources (e.g., Models)."""
        pass


# ============================================================================
# Built-in Executors
# ============================================================================

class PassthroughExecutor(Executor):
    """Simple passthrough for testing"""
    
    def execute(self, inputs: Dict[str, Any], params: Dict[str, Any]) -> Any:
        # Just return the first input
        if inputs:
            return next(iter(inputs.values()))
        return None


class AddExecutor(Executor):
    """Add two tensors"""
    
    def execute(self, inputs: Dict[str, Any], params: Dict[str, Any]) -> Any:
        try:
            import torch
            a = inputs.get('a')
            b = inputs.get('b')
            if a is not None and b is not None:
                return torch.add(a, b)
            return a or b
        except ImportError:
            # Fallback for non-PyTorch
            a = inputs.get('a', 0)
            b = inputs.get('b', 0)
            return a + b


# Executor registry
EXECUTORS: Dict[str, type] = {
    "Test::Passthrough": PassthroughExecutor,
    "Math::Add": AddExecutor,
}


def get_executor(op_type: str) -> Optional[Executor]:
    """Get an executor instance by operation type"""
    executor_class = EXECUTORS.get(op_type)
    if executor_class:
        return executor_class()
    return None


# ============================================================================
# Shared Memory Access (SRS Section 3.5.2)
# ============================================================================

class SharedMemoryAccess:
    """Zero-Copy access to shared memory"""
    
    def __init__(self, shm_name: str):
        self.shm_name = shm_name
        self.mm: Optional[mmap.mmap] = None
        self.fd: Optional[int] = None
    
    def open(self):
        """Open and map the shared memory region"""
        import ctypes
        
        # Open shared memory
        # Note: On macOS, shm_open uses /dev/shm-style naming differently
        shm_path = f"/dev/shm{self.shm_name}" if sys.platform == 'linux' else self.shm_name
        
        try:
            # Try POSIX shm_open via ctypes
            libc = ctypes.CDLL('libc.so.6' if sys.platform == 'linux' else 'libc.dylib')
            
            shm_open = libc.shm_open
            shm_open.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.c_int]
            shm_open.restype = ctypes.c_int
            
            O_RDWR = 0x0002
            self.fd = shm_open(self.shm_name.encode(), O_RDWR, 0o600)
            
            if self.fd < 0:
                raise OSError(f"shm_open failed for {self.shm_name}")
            
            # Map the memory
            self.mm = mmap.mmap(self.fd, 0, access=mmap.ACCESS_WRITE)
            
        except Exception as e:
            logger.warning(f"Failed to open SHM: {e}")
            self.mm = None
    
    def read_header(self) -> Dict[str, Any]:
        """Read the shared memory header"""
        if not self.mm:
            return {}
        
        self.mm.seek(0)
        magic = struct.unpack('<Q', self.mm.read(8))[0]
        version = struct.unpack('<I', self.mm.read(4))[0]
        
        return {
            'magic': hex(magic),
            'version': version,
            'valid': magic == 0x5654583300000001
        }
    
    def close(self):
        """Close the mapping"""
        if self.mm:
            self.mm.close()
        if self.fd is not None:
            os.close(self.fd)


# ============================================================================
# Worker Main Loop (SRS Section 3.10 - Worker Execution Flow)
# ============================================================================

class Worker:
    """Main worker process"""
    
    def __init__(self, slot_id: int, socket_path: str, shm_name: str):
        self.slot_id = slot_id
        self.socket_path = socket_path
        self.shm_name = shm_name
        self.sock: Optional[socket.socket] = None
        self.shm: Optional[SharedMemoryAccess] = None
        self.running = False
        self.worker_id = f"worker_{slot_id}_{os.getpid()}"
    
    def connect(self):
        """Connect to the host supervisor"""
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.socket_path)
        logger.info(f"Connected to {self.socket_path}")
        
        # Send handshake
        packet = ControlPacket(
            request_id=f"hs_{time.time_ns()}",
            timestamp=int(time.time() * 1000),
            payload={
                "type": "Handshake",
                "protocol_version": PROTOCOL_VERSION,
                "worker_id": self.worker_id,
                "capabilities": self._detect_capabilities()
            }
        )
        send_packet(self.sock, packet)
        
        # Wait for ack
        ack = recv_packet(self.sock)
        logger.info(f"Received handshake ack: {ack.payload}")
    
    def _detect_capabilities(self) -> List[str]:
        """Detect available capabilities (CUDA, etc.)"""
        caps = []
        
        try:
            import torch
            if torch.cuda.is_available():
                caps.append("CUDA")
                caps.append(f"GPU:{torch.cuda.get_device_name(0)}")
            if hasattr(torch.backends, 'mps') and torch.backends.mps.is_available():
                caps.append("MPS")
            caps.append("TORCH")
        except ImportError:
            pass
        
        try:
            import numpy
            caps.append("NUMPY")
        except ImportError:
            pass
        
        return caps
    
    def run(self):
        """Main event loop"""
        self.running = True
        
        # Open shared memory
        self.shm = SharedMemoryAccess(self.shm_name)
        self.shm.open()
        
        logger.info(f"Worker {self.worker_id} starting main loop")
        
        while self.running:
            try:
                packet = recv_packet(self.sock)
                self._handle_packet(packet)
            except ConnectionError:
                logger.warning("Connection lost")
                break
            except Exception as e:
                logger.error(f"Error in main loop: {e}")
                traceback.print_exc()
    
    def _handle_packet(self, packet: ControlPacket):
        """Handle incoming packets"""
        payload = packet.payload
        ptype = payload.get('type')
        
        if ptype == 'JobSubmit':
            self._execute_job(packet)
        elif ptype == 'JobCancel':
            logger.info(f"Job cancelled: {payload.get('job_id')}")
        elif ptype == 'Heartbeat':
            # Respond to heartbeat
            response = ControlPacket(
                request_id=packet.request_id,
                timestamp=int(time.time() * 1000),
                payload={"type": "Heartbeat", "worker_id": self.worker_id}
            )
            send_packet(self.sock, response)
    
    def _execute_job(self, packet: ControlPacket):
        """Execute a job"""
        payload = packet.payload
        job_id = payload.get('job_id')
        op_type = payload.get('op_type')
        params = payload.get('params', {})
        
        logger.info(f"Executing job {job_id}: {op_type}")
        start_time = time.perf_counter_ns()
        
        try:
            # Get executor
            executor = get_executor(op_type)
            if not executor:
                raise ValueError(f"Unknown op_type: {op_type}")
            
            # Execute (TODO: Add proper input handling from SHM)
            result = executor.execute({}, params)
            
            duration_us = (time.perf_counter_ns() - start_time) // 1000
            
            # Send success result
            response = ControlPacket(
                request_id=packet.request_id,
                timestamp=int(time.time() * 1000),
                payload={
                    "type": "JobResult",
                    "job_id": job_id,
                    "success": True,
                    "output_handle": None,  # TODO: Write to SHM
                    "duration_us": duration_us,
                    "peak_vram_mb": 0  # TODO: Track VRAM
                }
            )
            send_packet(self.sock, response)
            
        except Exception as e:
            duration_us = (time.perf_counter_ns() - start_time) // 1000
            logger.error(f"Job {job_id} failed: {e}")
            
            # Send error result
            response = ControlPacket(
                request_id=packet.request_id,
                timestamp=int(time.time() * 1000),
                payload={
                    "type": "JobResult",
                    "job_id": job_id,
                    "success": False,
                    "error_message": str(e),
                    "duration_us": duration_us,
                    "peak_vram_mb": 0
                }
            )
            send_packet(self.sock, response)
    
    def shutdown(self):
        """Clean shutdown"""
        self.running = False
        if self.shm:
            self.shm.close()
        if self.sock:
            self.sock.close()


# ============================================================================
# Entry Point
# ============================================================================

def main():
    parser = argparse.ArgumentParser(description='VORTEX Worker Process')
    parser.add_argument('--slot-id', type=int, required=True, help='Worker slot ID')
    parser.add_argument('--shm-name', default=SHM_NAME, help='Shared memory name')
    parser.add_argument('--socket', default=SOCKET_PATH, help='Socket path')
    
    args = parser.parse_args()
    
    logger.info(f"Starting worker: slot={args.slot_id}, shm={args.shm_name}")
    
    worker = Worker(
        slot_id=args.slot_id,
        socket_path=args.socket,
        shm_name=args.shm_name
    )
    
    # Handle signals
    def handle_signal(signum, frame):
        logger.info(f"Received signal {signum}, shutting down")
        worker.shutdown()
        sys.exit(0)
    
    signal.signal(signal.SIGTERM, handle_signal)
    signal.signal(signal.SIGINT, handle_signal)
    
    try:
        worker.connect()
        worker.run()
    except Exception as e:
        logger.error(f"Worker failed: {e}")
        traceback.print_exc()
        sys.exit(1)
    finally:
        worker.shutdown()


if __name__ == '__main__':
    main()
