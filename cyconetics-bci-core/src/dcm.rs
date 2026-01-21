use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CyconeticsBciError;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrGridBinding {
    pub allowed_zones: Vec<String>,
    pub min_hazard_level: u8,
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

/// K/S/R risk components (0–255 each, hex-friendly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComponents {
    /// Useful-knowledge / epistemic value (K)
    pub k_useful_knowledge: u8,
    /// Social impact (S), positive or disruptive potential encoded as magnitude
    pub s_social_impact: u8,
    /// Risk of harm (R), higher => more dangerous
    pub r_risk_of_harm: u8,
}

/// Aggregate risk score derived from K/S/R
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    /// Linear aggregate score (0–255)
    pub overall: u8,
    /// Optional label: "low", "moderate", "high", "extreme"
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilityManifest {
    pub id: Uuid,
    pub name: String;
    pub version: String,

    pub backend: BackendConfig,

    pub channels: Vec<ChannelSpec>,
    pub sampling: SamplingConstraints,
    pub session: SessionConstraints,

    pub jurisdictions: Vec<Jurisdiction>,
    pub privacy: PrivacyLevel,
    pub safety: SafetyFlags,

    pub xr_grid: XrGridBinding,

    /// K/S/R components for this device + driver combo.
    pub risk_components: RiskComponents,
    /// Cached aggregate score; can be recomputed from risk_components.
    pub risk_score: RiskScore,

    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl RiskScore {
    /// Simple semi-quantitative aggregation inspired by lab risk matrices:
    /// overall = clamp( (R * 2) + S.saturating_sub(K/4) , 0..=255 )
    pub fn compute(components: &RiskComponents) -> Self {
        let k = components.k_useful_knowledge as i32;
        let s = components.s_social_impact as i32;
        let r = components.r_risk_of_harm as i32;

        let mut score = (r * 2) + s - (k / 4);
        if score < 0 {
            score = 0;
        }
        if score > 255 {
            score = 255;
        }
        let overall = score as u8;

        let level = if overall < 64 {
            "low"
        } else if overall < 128 {
            "moderate"
        } else if overall < 192 {
            "high"
        } else {
            "extreme"
        }
        .to_string();

        RiskScore { overall, level }
    }
}

impl DeviceCapabilityManifest {
    pub fn validate(&self) -> Result<(), CyconeticsBciError> {
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

    /// Helper to recompute and update risk_score from risk_components.
    pub fn recompute_risk_score(&mut self) {
        self.risk_score = RiskScore::compute(&self.risk_components);
    }
}
