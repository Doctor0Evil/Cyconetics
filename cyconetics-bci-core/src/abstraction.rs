use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dcm::DeviceCapabilityManifest;
use crate::device_layer::BrainFlowDevice;
use crate::error::CyconeticsBciError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BciFrame {
    pub timestamp: DateTime<Utc>,
    pub channels: Vec<f64>,
    pub quality_ok: bool,
    pub metadata: Vec<(String, String)>,
}

pub struct CyconeticsBciDevice {
    manifest: DeviceCapabilityManifest,
    device: BrainFlowDevice,
    bound_zone: Option<String>,
    bound_hazard_level: Option<u8>,
}

impl CyconeticsBciDevice {
    pub fn new(device: BrainFlowDevice) -> Self {
        let manifest = device.manifest().clone();
        Self {
            manifest,
            device,
            bound_zone: None,
            bound_hazard_level: None,
        }
    }

    pub fn manifest(&self) -> &DeviceCapabilityManifest {
        &self.manifest
    }

    /// Bind this device instance to a specific XR-grid zone and hazard level.
    ///
    /// This must succeed before starting any stream or snapshot.
    pub fn bind_to_zone(
        &mut self,
        zone_id: &str,
        hazard_level: u8,
    ) -> Result<(), CyconeticsBciError> {
        use CyconeticsBciError::ManifestViolation;

        // Check that zone is allowed in the device's XR-grid manifest.
        if !self
            .manifest
            .xr_grid
            .allowed_zones
            .iter()
            .any(|z| z == zone_id)
        {
            return Err(ManifestViolation(format!(
                "Zone '{}' is not allowed for this device (allowed: {:?})",
                zone_id, self.manifest.xr_grid.allowed_zones
            )));
        }

        // Check hazard level against device's min/max hazard band.
        if hazard_level < self.manifest.xr_grid.min_hazard_level
            || hazard_level > self.manifest.xr_grid.max_hazard_level
        {
            return Err(ManifestViolation(format!(
                "Hazard level {} outside [{}, {}] for this device",
                hazard_level,
                self.manifest.xr_grid.min_hazard_level,
                self.manifest.xr_grid.max_hazard_level
            )));
        }

        self.bound_zone = Some(zone_id.to_string());
        self.bound_hazard_level = Some(hazard_level);
        Ok(())
    }

    fn ensure_bound(&self) -> Result<(), CyconeticsBciError> {
        if self.bound_zone.is_none() || self.bound_hazard_level.is_none() {
            return Err(CyconeticsBciError::ManifestViolation(
                "Device must be bound to an XR-grid zone before streaming or snapshot".into(),
            ));
        }
        Ok(())
    }

    /// Start streaming, enforcing XR-grid binding and manifest constraints.
    pub fn bci_stream_start(
        &mut self,
        sampling_hz: Option<u32>,
    ) -> Result<(), CyconeticsBciError> {
        self.ensure_bound()?;
        self.device.start_stream(sampling_hz)
    }

    pub fn bci_stream_stop(&mut self) -> Result<(), CyconeticsBciError> {
        self.device.stop_stream()
    }

    /// Take a snapshot of current data; requires a valid XR-grid binding.
    pub fn bci_snapshot(
        &mut self,
        num_samples: usize,
    ) -> Result<Vec<BciFrame>, CyconeticsBciError> {
        self.ensure_bound()?;
        let raw = self.device.read_frame(Some(num_samples))?;
        let now = Utc::now();
        let mut frames = Vec::new();

        // Aggregate per-channel values into a single frame (simple average).
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

        let mut metadata = vec![("source".into(), "brainflow".into())];
        if let Some(zone) = &self.bound_zone {
            metadata.push(("xr_zone".into(), zone.clone()));
        }
        if let Some(level) = self.bound_hazard_level {
            metadata.push(("xr_hazard_level".into(), level.to_string()));
        }

        frames.push(BciFrame {
            timestamp: now,
            channels: aggregated,
            quality_ok: true,
            metadata,
        });

        Ok(frames)
    }

    pub fn shutdown(self) -> Result<(), CyconeticsBciError> {
        self.device.shutdown()
    }
}
