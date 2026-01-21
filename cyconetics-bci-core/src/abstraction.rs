use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dcm::DeviceCapabilityManifest;
use crate::device_layer::BrainFlowDevice;
use crate::error::CyconeticsBciError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BciFrame {
    pub timestamp: DateTime<Utc>,
    pub channels: Vec<f64>,
    /// Simple quality flag; can be extended to per-channel flags.
    pub quality_ok: bool,
    /// Arbitrary metadata derived from DCM or higher layers.
    pub metadata: Vec<(String, String)>,
}

pub struct CyconeticsBciDevice {
    manifest: DeviceCapabilityManifest,
    device: BrainFlowDevice,
}

impl CyconeticsBciDevice {
    pub fn new(device: BrainFlowDevice) -> Self {
        let manifest = device.manifest().clone();
        Self { manifest, device }
    }

    pub fn manifest(&self) -> &DeviceCapabilityManifest {
        &self.manifest
    }

    /// Entry point: begin streaming with manifest-constrained sampling rate.
    pub fn bci_stream_start(&mut self, sampling_hz: Option<u32>) -> Result<(), CyconeticsBciError> {
        // Additional policy checks can be inserted here (session length, jurisdiction, etc.).
        self.device.start_stream(sampling_hz)
    }

    pub fn bci_stream_stop(&mut self) -> Result<(), CyconeticsBciError> {
        self.device.stop_stream()
    }

    pub fn bci_snapshot(&mut self, num_samples: usize) -> Result<Vec<BciFrame>, CyconeticsBciError> {
        let raw = self.device.read_frame(Some(num_samples))?;
        let now = Utc::now();
        let mut frames = Vec::new();

        // BrainFlow returns channel-major data; here we transpose-ish into simple frames.
        // For now, compress into a single BciFrame with averaged channels.
        let mut aggregated = vec![0.0; raw.len()];
        let mut count = 0usize;
        for ch_idx in 0..raw.len() {
            for sample in &raw[ch_idx] {
                aggregated[ch_idx] += *sample;
            }
            if !raw[ch_idx].is_empty() {
                count = raw[ch_idx].len();
            }
        }

        if count > 0 {
            for v in &mut aggregated {
                *v /= count as f64;
            }
        }

        frames.push(BciFrame {
            timestamp: now,
            channels: aggregated,
            quality_ok: true,
            metadata: vec![("source".into(), "brainflow".into())],
        });

        Ok(frames)
    }

    pub fn shutdown(self) -> Result<(), CyconeticsBciError> {
        self.device.shutdown()
    }
}
