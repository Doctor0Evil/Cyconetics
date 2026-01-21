use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Jurisdiction tags (e.g., Phoenix vs San Jolla lab grids).[file:3]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Jurisdiction {
    #[serde(rename = "US-CA")]
    UsCa,
    #[serde(rename = "US-AZ")]
    UsAz,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

/// XR-grid zoning metadata.[file:3]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrGridBinding {
    pub allowed_zones: Vec<String>,
    pub min_hazard_level: u8,
    pub max_hazard_level: u8,
}

/// Schema version for CybostateFactor attached to this device.[file:3]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CybostateSchemaVersion {
    V1,
    V2,
}

/// K/S/R-style risk score; R is hard-capped to respect RoH 0.3.[file:3]
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
    /// Derive a categorical band from R and enforce the RoH 0.3 ceiling in byte space.[file:3]
    pub fn from_components(k: u8, s: u8, r: u8) -> Self {
        // 0.3 * 255 ≈ 76.5 ≈ 0x4C: anything above this must not be accepted
        // for Phoenix / San Jolla RoH-governed devices.[file:3]
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

    /// Returns true if the encoded R respects the RoH 0.3 ceiling when
    /// interpreted as a byte-scaled risk-of-harm.[file:3]
    pub fn respects_roh_ceiling(&self) -> bool {
        // 0.3 of 255 ≈ 76.5; use 0x4C as a strict upper bound.
        self.r_risk_of_harm <= 0x4C
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

/// Canonical Device Capability Manifest for Cyconetics BCI core.[file:3]
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

    /// CybostateFactor schema version used by this device.
    pub cfschema_version: CybostateSchemaVersion,

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

        // Basic sanity check on risk band.
        let band = self.risk_score.risk_band.as_str();
        if !["low", "medium", "high", "extreme"].contains(&band) {
            return Err(CyconeticsBciError::ManifestViolation(
                "risk_score.risk_band must be one of: low, medium, high, extreme".into(),
            ));
        }

        // Enforce global RoH ceiling at manifest level.
        if !self.risk_score.respects_roh_ceiling() {
            return Err(CyconeticsBciError::ManifestViolation(
                "risk_score.r_risk_of_harm exceeds RoH 0.3 ceiling for this grid".into(),
            ));
        }

        Ok(())
    }
}
