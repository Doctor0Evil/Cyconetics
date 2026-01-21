use std::time::Duration;

use brainflow::{
    board_shim::{self, BoardShim},
    brainflow_input_params::BrainFlowInputParamsBuilder,
    BoardIds,
};

use crate::dcm::{BackendKind, DeviceCapabilityManifest};
use crate::error::CyconeticsBciError;

pub struct BrainFlowDevice {
    manifest: DeviceCapabilityManifest,
    board: BoardShim,
}

impl BrainFlowDevice {
    pub fn new(manifest: DeviceCapabilityManifest) -> Result<Self, CyconeticsBciError> {
        manifest.validate()?;

        if manifest.backend.kind != BackendKind::BrainFlow {
            return Err(CyconeticsBciError::ManifestViolation(
                "BrainFlowDevice requires BackendKind::BrainFlow".into(),
            ));
        }

        // Convert manifest.backend.identifier -> BrainFlow board_id
        // For now assume identifier is numeric board_id.
        let board_id: i32 = manifest
            .backend
            .identifier
            .parse()
            .map_err(|e| CyconeticsBciError::ConfigError(e.to_string()))?;

        let params = BrainFlowInputParamsBuilder::default().build();

        board_shim::enable_dev_board_logger()
            .map_err(|e| CyconeticsBciError::DeviceError(format!("{:?}", e)))?;

        let board = BoardShim::new(BoardIds::from(board_id), params)
            .map_err(|e| CyconeticsBciError::DeviceError(format!("{:?}", e)))?;

        Ok(Self { manifest, board })
    }

    pub fn prepare(&mut self) -> Result<(), CyconeticsBciError> {
        self.board
            .prepare_session()
            .map_err(|e| CyconeticsBciError::DeviceError(format!("{:?}", e)))
    }

    pub fn start_stream(&mut self, sampling_hz: Option<u32>) -> Result<(), CyconeticsBciError> {
        let hz = sampling_hz.unwrap_or(self.manifest.sampling.default_hz);
        if hz < self.manifest.sampling.min_hz || hz > self.manifest.sampling.max_hz {
            return Err(CyconeticsBciError::ManifestViolation(format!(
                "Requested sampling rate {} Hz is outside [{}, {}] Hz",
                hz, self.manifest.sampling.min_hz, self.manifest.sampling.max_hz
            )));
        }

        // Buffer size calculation can be tuned; 60 seconds of data as example.
        let buffer_size = (hz as usize) * 60 * self.manifest.channels.len();

        self.board
            .start_stream(buffer_size as i32, "")
            .map_err(|e| CyconeticsBciError::DeviceError(format!("{:?}", e)))
    }

    pub fn stop_stream(&mut self) -> Result<(), CyconeticsBciError> {
        self.board
            .stop_stream()
            .map_err(|e| CyconeticsBciError::DeviceError(format!("{:?}", e)))
    }

    pub fn read_frame(&mut self, num_samples: Option<usize>) -> Result<Vec<Vec<f64>>, CyconeticsBciError> {
        // Use BrainFlow ring buffer access; cap by manifest-defined channel count.
        let ns = num_samples.unwrap_or(32);
        let data = self
            .board
            .get_board_data(Some(ns as i32))
            .map_err(|e| CyconeticsBciError::DeviceError(format!("{:?}", e)))?;

        let mut truncated = Vec::new();
        let max_channels = self.manifest.channels.len().min(data.len());
        for ch in 0..max_channels {
            truncated.push(data[ch].clone());
        }
        Ok(truncated)
    }

    pub fn shutdown(mut self) -> Result<(), CyconeticsBciError> {
        self.board
            .release_session()
            .map_err(|e| CyconeticsBciError::DeviceError(format!("{:?}", e)))
    }

    pub fn manifest(&self) -> &DeviceCapabilityManifest {
        &self.manifest
    }
}
