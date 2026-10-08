//! Rongyász Spine 2048B mmap bus integration
//!
//! Provides shared memory interface using a fixed 2048-byte layout:
//! - 64B Header (magic "RGS_SPINE_2048V1", version 1, slot count, ring pointers)
//! - 512B PSI / Confidence Slots (32 x f64 slots)
//! - 1472B Ring Buffer (23 x 64B Event records)

use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SPINE_SIZE: usize = 2048;
pub const MAGIC: &[u8; 16] = b"RGS_SPINE_2048V1";
pub const MAX_EVENTS: usize = 23;
pub const EVENT_SIZE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveEventType {
    Gen = 1,
    Ltp = 2,
    Ltd = 3,
    Dream = 4,
    Pump = 5,
}

impl WaveEventType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Gen,
            2 => Self::Ltp,
            3 => Self::Ltd,
            4 => Self::Dream,
            _ => Self::Pump,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WaveEvent {
    pub event_type: WaveEventType,
    pub timestamp_ms: u64,
    pub amplitude: f32,
    pub payload: String,
}

pub struct SpineBus {
    pub path: PathBuf,
    mmap: Mutex<MmapMut>,
}

impl SpineBus {
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        #[allow(clippy::suspicious_open_options)]
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|e| format!("cannot open spine mmap file {}: {e}", path.display()))?;

        file.set_len(SPINE_SIZE as u64)
            .map_err(|e| format!("cannot resize spine file {}: {e}", path.display()))?;

        let mut mmap = unsafe {
            MmapMut::map_mut(&file)
                .map_err(|e| format!("cannot mmap spine file {}: {e}", path.display()))?
        };

        if &mmap[0..16] != MAGIC {
            // Initialize header
            mmap[0..16].copy_from_slice(MAGIC);
            mmap[16] = 1; // Version
            mmap[17..64].fill(0); // Clear pointers/slots
            mmap.flush().map_err(|e| format!("flush failed: {e}"))?;
        }

        Ok(Self {
            path,
            mmap: Mutex::new(mmap),
        })
    }

    pub fn default_bus() -> Result<Self, String> {
        let path = std::env::var_os("OCTOPUS_SPINE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("spine.rgs"));
        Self::open_or_create(path)
    }

    pub fn read_psi_slot(&self, index: usize) -> f64 {
        if index >= 32 {
            return 0.0;
        }
        let guard = self.mmap.lock().unwrap();
        let offset = 64 + index * 8;
        let bytes: [u8; 8] = guard[offset..offset + 8].try_into().unwrap();
        f64::from_le_bytes(bytes)
    }

    pub fn write_psi_slot(&self, index: usize, value: f64) {
        if index >= 32 {
            return;
        }
        let mut guard = self.mmap.lock().unwrap();
        let offset = 64 + index * 8;
        guard[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        let _ = guard.flush();
    }

    pub fn push_event(&self, event_type: WaveEventType, amplitude: f32, payload: &str) {
        let mut guard = self.mmap.lock().unwrap();
        let head = guard[18] as usize % MAX_EVENTS;
        let offset = 576 + head * EVENT_SIZE;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        guard[offset] = event_type as u8;
        guard[offset + 1..offset + 9].copy_from_slice(&now.to_le_bytes());
        guard[offset + 9..offset + 13].copy_from_slice(&amplitude.to_le_bytes());

        let payload_bytes = payload.as_bytes();
        let len = payload_bytes.len().min(50);
        guard[offset + 13] = len as u8;
        guard[offset + 14..offset + 14 + len].copy_from_slice(&payload_bytes[..len]);
        if len < 50 {
            guard[offset + 14 + len..offset + 64].fill(0);
        }

        guard[18] = ((head + 1) % MAX_EVENTS) as u8;
        let _ = guard.flush();
    }

    pub fn read_events(&self) -> Vec<WaveEvent> {
        let guard = self.mmap.lock().unwrap();
        let mut events = Vec::new();
        for i in 0..MAX_EVENTS {
            let offset = 576 + i * EVENT_SIZE;
            let event_type = guard[offset];
            if event_type == 0 {
                continue;
            }
            let timestamp_ms =
                u64::from_le_bytes(guard[offset + 1..offset + 9].try_into().unwrap());
            let amplitude = f32::from_le_bytes(guard[offset + 9..offset + 13].try_into().unwrap());
            let len = guard[offset + 13] as usize;
            let len = len.min(50);
            let payload =
                String::from_utf8_lossy(&guard[offset + 14..offset + 14 + len]).to_string();

            events.push(WaveEvent {
                event_type: WaveEventType::from_u8(event_type),
                timestamp_ms,
                amplitude,
                payload,
            });
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_spine_bus_creation_and_events() {
        let tmp = NamedTempFile::new().unwrap();
        let spine = SpineBus::open_or_create(tmp.path()).unwrap();

        spine.write_psi_slot(0, 0.95);
        assert!((spine.read_psi_slot(0) - 0.95).abs() < 1e-6);

        spine.push_event(WaveEventType::Gen, 0.85, "test-event");
        let events = spine.read_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, WaveEventType::Gen);
        assert_eq!(events[0].payload, "test-event");
    }
}
