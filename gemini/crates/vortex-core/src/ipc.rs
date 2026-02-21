//! IPC - Inter-Process Communication via Unix Domain Sockets
//!
//! Implements SRS Section 3.6.2 (IPC Gateway Trait)
//! Uses Protobuf messages with Big-Endian length prefix (4 bytes)
//!
//! Protocol Specification:
//! - 4 bytes (u32 Big-Endian): message length
//! - N bytes: Protobuf-encoded message

use crate::error::{VortexError, VortexResult};
use std::path::Path;
use vortex_protocol::{JobRequest, JobResult, Heartbeat, WorkerHandshake, HandshakeAck};
use prost::Message;

/// Default socket path
pub const SOCKET_PATH: &str = "/tmp/vortex.sock";

/// Per-slot socket path for worker pool
pub fn socket_path_for_slot(slot_id: u8) -> String {
    std::env::var("VORTEX_IPC_BASE")
        .unwrap_or_else(|_| "/tmp/vortex_slot".into())
        + &format!("_{}.sock", slot_id)
}

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// IPC Gateway for Unix Domain Socket communication
pub struct IpcGateway {
    socket_path: String,
    #[cfg(target_family = "unix")]
    listener: Option<std::os::unix::net::UnixListener>,
}

impl IpcGateway {
    /// Create a new IPC gateway (server mode)
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
            #[cfg(target_family = "unix")]
            listener: None,
        }
    }

    /// Bind and listen on the socket
    #[cfg(target_family = "unix")]
    pub fn bind(&mut self) -> VortexResult<()> {
        use std::os::unix::net::UnixListener;

        let path = Path::new(&self.socket_path);

        // Remove existing socket file
        if path.exists() {
            std::fs::remove_file(path).ok();
        }

        let listener = UnixListener::bind(path).map_err(|e| VortexError::BindError {
            path: self.socket_path.clone(),
            reason: e.to_string(),
        })?;

        self.listener = Some(listener);
        Ok(())
    }

    /// Stub for non-Unix
    #[cfg(not(target_family = "unix"))]
    pub fn bind(&mut self) -> VortexResult<()> {
        Err(VortexError::BindError {
            path: self.socket_path.clone(),
            reason: "Unix sockets not supported".to_string(),
        })
    }

    /// Accept a connection (blocking)
    #[cfg(target_family = "unix")]
    pub fn accept(&self) -> VortexResult<IpcConnection> {
        let listener = self.listener.as_ref().ok_or_else(|| {
            VortexError::Internal("Not bound".to_string())
        })?;

        let (stream, _addr) = listener.accept().map_err(|e| {
            VortexError::Io(e)
        })?;

        Ok(IpcConnection { stream })
    }

    #[cfg(not(target_family = "unix"))]
    pub fn accept(&self) -> VortexResult<IpcConnection> {
        Err(VortexError::Internal("Not supported".to_string()))
    }
}

/// A single IPC connection
#[cfg(target_family = "unix")]
pub struct IpcConnection {
    stream: std::os::unix::net::UnixStream,
}

#[cfg(target_family = "unix")]
impl IpcConnection {
    /// Send a protobuf message with Big-Endian length prefix
    pub fn send<M: Message>(&mut self, message: &M) -> VortexResult<()> {
        use std::io::Write;

        // Encode message
        let mut buf = Vec::with_capacity(message.encoded_len());
        message.encode(&mut buf).map_err(|e| {
            VortexError::IpcFailure { reason: format!("Encode failed: {}", e) }
        })?;

        // Create length prefix (4 bytes, Big-Endian)
        let len = buf.len() as u32;
        let len_bytes = len.to_be_bytes();

        // Send length + message
        self.stream.write_all(&len_bytes)?;
        self.stream.write_all(&buf)?;

        Ok(())
    }

    /// Receive a protobuf message (Big-Endian length prefix)
    pub fn receive<M: Message + Default>(&mut self) -> VortexResult<M> {
        use std::io::Read;

        // Read length prefix (4 bytes, Big-Endian)
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Validate length
        if len > 16 * 1024 * 1024 { // 16MB max
            return Err(VortexError::IpcFailure {
                reason: format!("Message too large: {} bytes", len)
            });
        }

        // Read payload
        let mut payload = vec![0u8; len];
        self.stream.read_exact(&mut payload)?;

        // Decode message
        let message = M::decode(&*payload).map_err(|e| {
            VortexError::IpcFailure { reason: format!("Decode failed: {}", e) }
        })?;

        Ok(message)
    }

    /// Send JobRequest
    pub fn send_job_request(&mut self, request: &JobRequest) -> VortexResult<()> {
        self.send(request)
    }

    /// Send JobResult
    pub fn send_job_result(&mut self, result: &JobResult) -> VortexResult<()> {
        self.send(result)
    }

    /// Send Heartbeat
    pub fn send_heartbeat(&mut self, heartbeat: &Heartbeat) -> VortexResult<()> {
        self.send(heartbeat)
    }

    /// Send WorkerHandshake
    pub fn send_handshake(&mut self, handshake: &WorkerHandshake) -> VortexResult<()> {
        self.send(handshake)
    }

    /// Receive JobRequest
    pub fn receive_job_request(&mut self) -> VortexResult<JobRequest> {
        self.receive()
    }

    /// Receive JobResult
    pub fn receive_job_result(&mut self) -> VortexResult<JobResult> {
        self.receive()
    }

    /// Receive Heartbeat
    pub fn receive_heartbeat(&mut self) -> VortexResult<Heartbeat> {
        self.receive()
    }

    /// Receive HandshakeAck
    pub fn receive_handshake_ack(&mut self) -> VortexResult<HandshakeAck> {
        self.receive()
    }

    /// Set a read timeout on the underlying socket.
    /// After this, receive() returns Err(VortexError::Io) on timeout.
    pub fn set_read_timeout(&self, timeout: std::time::Duration) -> VortexResult<()> {
        self.stream.set_read_timeout(Some(timeout))
            .map_err(VortexError::Io)
    }

    /// Get the peer PID (for authentication via SO_PEERCRED)
    #[cfg(target_os = "linux")]
    pub fn peer_pid(&self) -> Option<i32> {
        use std::os::unix::io::AsRawFd;

        let fd = self.stream.as_raw_fd();

        #[repr(C)]
        struct UcRed {
            pid: i32,
            uid: u32,
            gid: u32,
        }

        let mut cred: UcRed = unsafe { std::mem::zeroed() };
        let mut len = std::mem::size_of::<UcRed>() as u32;

        let result = unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                &mut cred as *mut UcRed as *mut libc::c_void,
                &mut len,
            )
        };

        if result == 0 {
            Some(cred.pid)
        } else {
            None
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn peer_pid(&self) -> Option<i32> {
        None
    }
}

#[cfg(not(target_family = "unix"))]
pub struct IpcConnection;

#[cfg(test)]
mod tests {
    use super::*;
    use vortex_protocol::control::{TensorInput, TensorRef, TensorOutputSpec};
    use vortex_protocol::JobRequest;

    #[test]
    fn test_protobuf_serialization() {
        // Create a JobRequest
        let request = JobRequest {
            job_id: "test-job-123".to_string(),
            node_type: "com.vortex.ksampler".to_string(),
            params_json: b"{\"steps\": 20}".to_vec(),
            inputs: vec![
                TensorInput {
                    name: "latent".to_string(),
                    tensor: Some(TensorRef {
                        offset: 0,
                        size_bytes: 1024,
                        dtype: 0, // f32
                        shape: vec![4, 64, 64],
                    }),
                }
            ],
            outputs: vec![
                TensorOutputSpec {
                    name: "output".to_string(),
                    dtype: 0,
                    expected_shape: vec![4, 64, 64],
                }
            ],
        };

        // Encode to bytes
        let mut buf = Vec::with_capacity(request.encoded_len());
        request.encode(&mut buf).unwrap();

        // Decode back
        let decoded = JobRequest::decode(&*buf).unwrap();

        assert_eq!(request.job_id, decoded.job_id);
        assert_eq!(request.node_type, decoded.node_type);
        assert_eq!(request.inputs.len(), decoded.inputs.len());
    }

    #[test]
    fn test_length_prefix() {
        // Test Big-Endian length encoding
        let len: u32 = 1024;
        let bytes = len.to_be_bytes();

        // Decode
        let decoded = u32::from_be_bytes(bytes);
        assert_eq!(len, decoded);
    }
}
