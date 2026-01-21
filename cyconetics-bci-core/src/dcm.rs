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
    /// Minimal identity risk (e.g., anonymous task markers only)
    Low,
    /// Potentially linkable to an individual with side information
    Medium,
    /// Directly identifying or clinically sensitive
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConstraints {
    pub min_hz: u32,
    pub max_hz: u32,
    /// Default sampling rate in Hz to use if caller does not specify.
    pub default_hz: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConstraints {
    /// Maximum continuous session duration in seconds.
    pub max_duration_secs: u32,
    /// Minimum rest interval between sessions in seconds.
    pub min_rest_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSpec {
    pub index: u16,
    pub label: String,
    pub unit: String,
    /// Is this channel safe for closed-loop control usage?
    pub closed_loop_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    BrainFlow,
    LslSource,
    VendorC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub kind: BackendKind,
    /// For BrainFlow: board_id; for vendor: driver name; for LSL: stream type.
    pub identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyFlags {
    /// True if device can perform stimulation, not just read.
    pub can_stimulate: bool,
    /// True if hardware isolation is explicitly rated for patient connection.
    pub medical_isolation_rated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilityManifest {
    /// Stable manifest identifier (could be ALN / bostrom hash).
    pub id: Uuid,
    pub name: String,
    pub version: String,

    pub backend: BackendConfig,

    /// Physical/operational constraints
    pub channels: Vec<ChannelSpec>,
    pub sampling: SamplingConstraints,
    pub session: SessionConstraints,

    /// Jurisdictional and privacy metadata
    pub jurisdictions: Vec<Jurisdiction>,
    pub privacy: PrivacyLevel,
    pub safety: SafetyFlags,

    /// Arbitrary tags and notes (e.g., "research-only", "no-clinical-use").
    pub tags: Vec<String>,

    /// Creation time of this manifest.
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
        Ok(())
    }
}
