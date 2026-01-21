use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Jurisdiction {
    #[serde(rename = "US-CA")]
    UsCa,
    #[serde(rename = "US-AZ")]
    UsAz,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConstraints {
    pub min_hz: u32,
    pub max_hz: u32,
    pub default_hz: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConstraints {
    pub max_duration_secs: u32,
    pub min_rest_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSpec {
    pub index: u16,
    pub label: String,
    pub unit: String,
    pub closed_loop_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    BrainFlow,
    LslSource,
    VendorC,
}

/// XR-grid zoning metadata for this device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrGridBinding {
    /// Logical zone IDs (e.g., "AZ-LAB-1-XR-EEG-LOWRISK").
    pub allowed_zones: Vec<String>,
    /// Minimum hazard level this device may be used with (e.g., BSL-like tier).
    pub min_hazard_level: u8,
    /// Maximum hazard level (helps constrain where the device is allowed).
    pub max_hazard_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub kind: BackendKind,
    pub identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyFlags {
    pub can_stimulate: bool,
    pub medical_isolation_rated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilityManifest {
    pub id: Uuid,
    pub name: String,
    pub version: String,

    pub backend: BackendConfig,

    pub channels: Vec<ChannelSpec>,
    pub sampling: SamplingConstraints,
    pub session: SessionConstraints,

    pub jurisdictions: Vec<Jurisdiction>,
    pub privacy: PrivacyLevel,
    pub safety: SafetyFlags,

    /// XR-grid binding information.
    pub xr_grid: XrGridBinding,

    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl DeviceCapabilityManifest {
    pub fn validate(&self) -> Result<(), crate::error::CyconeticsBciError> {
        use crate::error::CyconeticsBciError;

        if self.channels.is_empty() {
            return Err(CyconeticsBciError::ManifestViolation(
                "DCM must declare at least one channel".into(),
            ));
        }

        if self.sampling.min_hz == 0
            || self.sampling.max_hz == 0
            || self.sampling.min_hz > self.sampling.max_hz
        {
            return Err(CyconeticsBciError::ManifestViolation(
                "Invalid sampling constraints".into(),
            ));
        }

        if self.sampling.default_hz < self.sampling.min_hz
            || self.sampling.default_hz > self.sampling.max_hz
        {
            return Err(CyconeticsBciError::ManifestViolation(
                "Default sampling rate out of bounds".into(),
            ));
        }

        if self.session.max_duration_secs == 0 {
            return Err(CyconeticsBciError::ManifestViolation(
                "Session max_duration_secs must be > 0".into(),
            ));
        }

        if self.xr_grid.allowed_zones.is_empty() {
            return Err(CyconeticsBciError::ManifestViolation(
                "XR-grid configuration must declare at least one allowed zone".into(),
            ));
        }

        if self.xr_grid.min_hazard_level > self.xr_grid.max_hazard_level {
            return Err(CyconeticsBciError::ManifestViolation(
                "XR-grid hazard levels are inconsistent".into(),
            ));
        }

        Ok(())
    }
}
