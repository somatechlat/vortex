//! IPC - Inter-Process Communication via Unix Domain Sockets
//!
//! Implements SRS Section 3.6.2 (IPC Gateway Trait)

use crate::error::{VortexError, VortexResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Default socket path
pub const SOCKET_PATH: &str = "/tmp/vtx.sock";

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Control packet types matching SRS Section 3.4.2 (Protobuf Definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPacket {
    pub request_id: String,
    pub timestamp: i64,
    pub payload: PacketPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PacketPayload {
    Handshake(Handshake),
    HandshakeAck(HandshakeAck),
    JobSubmit(JobSubmit),
    JobResult(JobResult),
    JobCancel(JobCancel),
    Heartbeat(Heartbeat),
    Error(ErrorPayload),
}

/// Worker handshake message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    pub protocol_version: u32,
    pub worker_id: String,
    pub capabilities: Vec<String>,
}

/// Handshake acknowledgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeAck {
    pub slot_id: u8,
    pub shm_name: String,
}

/// Job submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSubmit {
    pub job_id: String,
    pub node_id: String,
    pub op_type: String,
    pub input_handles: Vec<u64>,  // Offsets in SHM
    pub params: serde_json::Value,
}

/// Job result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub success: bool,
    pub output_handle: Option<u64>,
    pub error_message: Option<String>,
    pub duration_us: u64,
    pub peak_vram_mb: u64,
}

/// Job cancellation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCancel {
    pub job_id: String,
}

/// Heartbeat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub worker_id: String,
    pub timestamp: i64,
}

/// Error payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

impl ControlPacket {
    /// Create a new packet with auto-generated request ID
    pub fn new(payload: PacketPayload) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono_timestamp(),
            payload,
        }
    }
    
    /// Serialize to bytes (length-prefixed JSON)
    pub fn to_bytes(&self) -> VortexResult<Vec<u8>> {
        let json = serde_json::to_vec(self)?;
        let len = json.len() as u32;
        
        let mut buf = Vec::with_capacity(4 + json.len());
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&json);
        
        Ok(buf)
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> VortexResult<Self> {
        if data.len() < 4 {
            return Err(VortexError::Internal("Packet too short".to_string()));
        }
        
        let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if data.len() < 4 + len {
            return Err(VortexError::Internal("Incomplete packet".to_string()));
        }
        
        let packet = serde_json::from_slice(&data[4..4 + len])?;
        Ok(packet)
    }
}

/// Get current timestamp in milliseconds
fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

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
    /// Send a packet
    pub fn send(&mut self, packet: &ControlPacket) -> VortexResult<()> {
        use std::io::Write;
        
        let bytes = packet.to_bytes()?;
        self.stream.write_all(&bytes)?;
        Ok(())
    }
    
    /// Receive a packet (blocking)
    pub fn recv(&mut self) -> VortexResult<ControlPacket> {
        use std::io::Read;
        
        // Read length prefix
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;
        
        // Read payload
        let mut payload = vec![0u8; len];
        self.stream.read_exact(&mut payload)?;
        
        // Prepend length for parsing
        let mut full = Vec::with_capacity(4 + len);
        full.extend_from_slice(&len_buf);
        full.extend_from_slice(&payload);
        
        ControlPacket::from_bytes(&full)
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

    #[test]
    fn test_packet_serialization() {
        let packet = ControlPacket::new(PacketPayload::Handshake(Handshake {
            protocol_version: 1,
            worker_id: "worker_0".to_string(),
            capabilities: vec!["CUDA".to_string()],
        }));
        
        let bytes = packet.to_bytes().unwrap();
        let decoded = ControlPacket::from_bytes(&bytes).unwrap();
        
        assert_eq!(packet.request_id, decoded.request_id);
    }
}
