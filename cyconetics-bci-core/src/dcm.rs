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

/// XR-grid zoning metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrGridBinding {
    pub allowed_zones: Vec<String>,
    pub min_hazard_level: u8,
    pub max_hazard_level: u8,
}

/// Simple K/S/R-style risk score, following risk-matrix concepts
/// (likelihood x consequence) from biosafety guidance.[web:121][web:122][web:128][web:125]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    /// 0–255: higher means more useful knowledge / maturity.
    pub k_usefulness: u8,
    /// 0–255: higher means larger social impact (positive or broad).
    pub s_social_impact: u8,
    /// 0–255: higher means higher risk-of-harm.
    pub r_risk_of_harm: u8,
    /// Optional categorical band, derived from R.
    /// e.g., "low", "medium", "high", "extreme".
    pub risk_band: String,
}

impl RiskScore {
    /// Simple helper to derive a risk band from R, inspired by
    /// 3–4 level matrices in biosafety risk assessment (low/medium/high/extreme).[web:122][web:125]
    pub fn from_components(k: u8, s: u8, r: u8) -> Self {
        let risk_band = if r <= 0x40 {
            "low".to_string()
        } else if r <= 0x80 {
            "medium".to_string()
        } else if r <= 0xC0 {
            "high".to_string()
        } else {
            "extreme".to_string()
        };
        Self {
            k_usefulness: k,
            s_social_impact: s,
            r_risk_of_harm: r,
            risk_band,
        }
    }
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

    pub xr_grid: XrGridBinding,

    /// K/S/R risk score attached to this device/driver.
    pub risk_score: RiskScore,

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

        // Basic sanity check on risk band (optional).
        let band = self.risk_score.risk_band.as_str();
        if !["low", "medium", "high", "extreme"].contains(&band) {
            return Err(CyconeticsBciError::ManifestViolation(
                "risk_score.risk_band must be one of: low, medium, high, extreme".into(),
            ));
        }

        Ok(())
    }
}
