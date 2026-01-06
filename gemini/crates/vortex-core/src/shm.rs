//! Shared Memory - POSIX SHM implementation for Zero-Copy transport
//!
//! Implements SRS Section 3.4.1 (Shared Memory Header) and Section 3.9.1 (Offset Map)

use crate::error::{VortexError, VortexResult};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Shared memory region name
pub const SHM_NAME: &str = "/vtx_shm";

/// Default SHM size (64GB)
pub const SHM_SIZE: usize = 64 * 1024 * 1024 * 1024;

/// Magic bytes for validation
pub const MAGIC_BYTES: u64 = 0x5654_5833_0000_0001; // "VTX3" + version 1

/// Maximum number of worker slots
pub const MAX_WORKERS: usize = 256;

/// Offset where worker slots begin
pub const SLOTS_OFFSET: usize = 0x40;

/// Size of each worker slot (64 bytes, cache-line aligned)
pub const SLOT_SIZE: usize = 64;

/// Worker status values (SRS Section 3.4.1)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    Idle = 0,
    Busy = 1,
    Dead = 2,
    Booting = 3,
}

impl From<u32> for WorkerStatus {
    fn from(v: u32) -> Self {
        match v {
            0 => WorkerStatus::Idle,
            1 => WorkerStatus::Busy,
            2 => WorkerStatus::Dead,
            3 => WorkerStatus::Booting,
            _ => WorkerStatus::Dead,
        }
    }
}

/// Shared Memory Header (SRS Section 3.4.1, 3.9.1)
///
/// Offset Map:
/// - 0x0000: magic_bytes (u64)
/// - 0x0008: version (u32)
/// - 0x000C: flags (atomic_u32)
/// - 0x0010: clock_tick (atomic_u64)
/// - 0x0018: reserved[40]
/// - 0x0040: slots[256]
#[repr(C, align(64))]
pub struct ShmHeader {
    /// Magic bytes for validation (0x5654_5833_0000_0001)
    pub magic_bytes: u64,
    
    /// Protocol version (must match host and worker)
    pub version: u32,
    
    /// System state flags (Bit 0: SYSTEM_READY, Bit 1: MAINTENANCE)
    pub flags: AtomicU32,
    
    /// Global monotonic clock (1 tick = 1ms)
    pub clock_tick: AtomicU64,
    
    /// Reserved padding for cache line alignment
    pub reserved: [u8; 40],
}

/// Worker Slot structure (64 bytes, cache-line aligned)
#[repr(C, align(64))]
pub struct WorkerSlot {
    /// OS Process ID (0 = Empty)
    pub pid: AtomicU32,
    
    /// Worker status (0=IDLE, 1=BUSY, 2=DEAD, 3=BOOTING)
    pub status: AtomicU32,
    
    /// Current job ID (pointer/offset)
    pub current_job_id: AtomicU64,
    
    /// Last heartbeat timestamp
    pub last_heartbeat: AtomicU64,
    
    /// Padding to 64 bytes
    pub padding: [u8; 40],
}

impl ShmHeader {
    /// Validate the magic bytes
    pub fn is_valid(&self) -> bool {
        self.magic_bytes == MAGIC_BYTES
    }
    
    /// Get the current clock tick
    pub fn clock(&self) -> u64 {
        self.clock_tick.load(Ordering::Acquire)
    }
    
    /// Increment the clock tick
    pub fn tick(&self) -> u64 {
        self.clock_tick.fetch_add(1, Ordering::AcqRel)
    }
    
    /// Check if system is ready
    pub fn is_ready(&self) -> bool {
        self.flags.load(Ordering::Acquire) & 0x01 != 0
    }
    
    /// Set system ready flag
    pub fn set_ready(&self, ready: bool) {
        if ready {
            self.flags.fetch_or(0x01, Ordering::Release);
        } else {
            self.flags.fetch_and(!0x01, Ordering::Release);
        }
    }
}

impl WorkerSlot {
    /// Create a new empty slot
    pub const fn new() -> Self {
        Self {
            pid: AtomicU32::new(0),
            status: AtomicU32::new(WorkerStatus::Idle as u32),
            current_job_id: AtomicU64::new(0),
            last_heartbeat: AtomicU64::new(0),
            padding: [0u8; 40],
        }
    }
    
    /// Check if slot is empty
    pub fn is_empty(&self) -> bool {
        self.pid.load(Ordering::Acquire) == 0
    }
    
    /// Get the current status
    pub fn get_status(&self) -> WorkerStatus {
        WorkerStatus::from(self.status.load(Ordering::Acquire))
    }
    
    /// Set the status atomically
    pub fn set_status(&self, status: WorkerStatus) {
        self.status.store(status as u32, Ordering::Release);
    }
    
    /// Claim this slot for a new worker
    pub fn claim(&self, pid: u32) -> bool {
        // Try to atomically set PID from 0
        self.pid.compare_exchange(0, pid, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }
    
    /// Release this slot
    pub fn release(&self) {
        self.pid.store(0, Ordering::Release);
        self.status.store(WorkerStatus::Idle as u32, Ordering::Release);
        self.current_job_id.store(0, Ordering::Release);
    }
    
    /// Update heartbeat
    pub fn heartbeat(&self, tick: u64) {
        self.last_heartbeat.store(tick, Ordering::Release);
    }
    
    /// Check if worker is alive (heartbeat within threshold)
    pub fn is_alive(&self, current_tick: u64, threshold: u64) -> bool {
        let last = self.last_heartbeat.load(Ordering::Acquire);
        current_tick.saturating_sub(last) < threshold
    }
}

/// Shared Memory Manager
pub struct SharedMemory {
    /// Base address of mapped memory
    base: *mut u8,
    
    /// Size of the mapping
    size: usize,
    
    /// File descriptor (for cleanup)
    #[cfg(target_family = "unix")]
    fd: std::os::unix::io::RawFd,
}

unsafe impl Send for SharedMemory {}
unsafe impl Sync for SharedMemory {}

impl SharedMemory {
    /// Open or create shared memory region
    #[cfg(target_family = "unix")]
    pub fn open(create: bool) -> VortexResult<Self> {
        use nix::fcntl::OFlag;
        use nix::sys::mman::{mmap, shm_open, MapFlags, ProtFlags};
        use nix::sys::stat::Mode;
        use nix::unistd::ftruncate;
        use std::ffi::CString;
        use std::num::NonZeroUsize;

        let name = CString::new(SHM_NAME).unwrap();
        
        let flags = if create {
            OFlag::O_CREAT | OFlag::O_RDWR
        } else {
            OFlag::O_RDWR
        };
        
        let fd = shm_open(
            name.as_c_str(),
            flags,
            Mode::S_IRUSR | Mode::S_IWUSR,
        ).map_err(|e| VortexError::ShmFailure {
            reason: e.to_string(),
        })?;
        
        if create {
            ftruncate(&fd, SHM_SIZE as i64).map_err(|e| VortexError::ShmFailure {
                reason: format!("ftruncate failed: {}", e),
            })?;
        }
        
        let ptr = unsafe {
            mmap(
                None,
                NonZeroUsize::new(SHM_SIZE).unwrap(),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                &fd,
                0,
            ).map_err(|e| VortexError::ShmFailure {
                reason: format!("mmap failed: {}", e),
            })?
        };
        
        let base = ptr.as_ptr() as *mut u8;
        
        // Initialize header if creating
        if create {
            let header = unsafe { &mut *(base as *mut ShmHeader) };
            header.magic_bytes = MAGIC_BYTES;
            header.version = 1;
            header.flags = AtomicU32::new(0);
            header.clock_tick = AtomicU64::new(0);
            header.reserved = [0u8; 40];
        }
        
        Ok(Self {
            base,
            size: SHM_SIZE,
            fd: std::os::unix::io::AsRawFd::as_raw_fd(&fd),
        })
    }
    
    /// Stub for non-Unix platforms
    #[cfg(not(target_family = "unix"))]
    pub fn open(_create: bool) -> VortexResult<Self> {
        Err(VortexError::ShmFailure {
            reason: "Shared memory only supported on Unix".to_string(),
        })
    }
    
    /// Get a reference to the header
    pub fn header(&self) -> &ShmHeader {
        unsafe { &*(self.base as *const ShmHeader) }
    }
    
    /// Get a mutable reference to the header
    pub fn header_mut(&mut self) -> &mut ShmHeader {
        unsafe { &mut *(self.base as *mut ShmHeader) }
    }
    
    /// Get a reference to a worker slot
    pub fn slot(&self, index: usize) -> Option<&WorkerSlot> {
        if index >= MAX_WORKERS {
            return None;
        }
        let offset = SLOTS_OFFSET + (index * SLOT_SIZE);
        unsafe {
            Some(&*((self.base.add(offset)) as *const WorkerSlot))
        }
    }
    
    /// Get a mutable reference to a worker slot
    pub fn slot_mut(&mut self, index: usize) -> Option<&mut WorkerSlot> {
        if index >= MAX_WORKERS {
            return None;
        }
        let offset = SLOTS_OFFSET + (index * SLOT_SIZE);
        unsafe {
            Some(&mut *((self.base.add(offset)) as *mut WorkerSlot))
        }
    }
    
    /// Find an empty slot and claim it for the given PID
    pub fn claim_slot(&self, pid: u32) -> Option<usize> {
        for i in 0..MAX_WORKERS {
            if let Some(slot) = self.slot(i) {
                if slot.claim(pid) {
                    return Some(i);
                }
            }
        }
        None
    }
    
    /// Find slot by PID
    pub fn find_slot_by_pid(&self, pid: u32) -> Option<usize> {
        for i in 0..MAX_WORKERS {
            if let Some(slot) = self.slot(i) {
                if slot.pid.load(Ordering::Acquire) == pid {
                    return Some(i);
                }
            }
        }
        None
    }
    
    /// Allocate space in the data region
    /// Returns offset from base
    pub fn allocate(&mut self, size: usize, alignment: usize) -> Option<usize> {
        // Data region starts after slots
        let data_start = SLOTS_OFFSET + (MAX_WORKERS * SLOT_SIZE);
        
        // TODO: Implement proper arena allocator
        // For now, just return the start of data region
        Some(data_start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        assert_eq!(std::mem::size_of::<ShmHeader>(), 64);
    }

    #[test]
    fn test_slot_size() {
        assert_eq!(std::mem::size_of::<WorkerSlot>(), 64);
    }

    #[test]
    fn test_worker_status() {
        let slot = WorkerSlot::new();
        assert!(slot.is_empty());
        assert_eq!(slot.get_status(), WorkerStatus::Idle);
        
        slot.set_status(WorkerStatus::Busy);
        assert_eq!(slot.get_status(), WorkerStatus::Busy);
    }
}
