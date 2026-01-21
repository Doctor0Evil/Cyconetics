use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dcm::{DeviceCapabilityManifest, Jurisdiction};
use crate::error::CyconeticsBciError;

/// XR grid zones where HCI exports are valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrZoneRef {
    /// Logical zone identifier (e.g., "XR-ZONE-CA-LAB-1").
    pub zone_id: String,
    /// Optional freeform description.
    pub description: Option<String>,
}

/// Categories of derived BCI state that HCI can consume.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HciStateKind {
    /// Very coarse, low-risk levels (e.g., low/med/high workload).
    CoarseCognitiveState,
    /// Simple discrete intents (e.g., A/B selector, left/right).
    DiscreteIntent,
    /// Binary engagement / disengagement flags.
    EngagementFlag,
    /// Quality indicators (e.g., "bad_contact", "artifact_heavy").
    SignalQualityFlag,
}

/// How often the state may be emitted into HCI/XR tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HciRateLimit {
    /// Maximum updates per second.
    pub max_hz: f32,
}

/// High-level risk rating for an exported state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HciRiskLevel {
    Low,
    Medium,
    High,
}

/// How the state is exposed on the internal bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HciExportChannel {
    /// Logical topic or stream name (e.g., "bci.hci.workload").
    pub topic: String,
    /// True if values are de-identified and aggregated.
    pub anonymized: bool,
    /// Optional notes (e.g., "no single-trial values; rolling average only").
    pub notes: Option<String>,
}

/// A single allowed HCI export mapping from BCI-derived state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HciExportRule {
    /// ID for this rule; can be logged in audit trails.
    pub id: Uuid,
    /// BCI-derived state category.
    pub kind: HciStateKind,
    /// Human-readable label (e.g., "workload_level").
    pub label: String,
    /// Jurisdictions where this export is allowed.
    pub jurisdictions: Vec<Jurisdiction>,
    /// XR zones where this export is allowed.
    pub xr_zones: Vec<XrZoneRef>,
    /// Risk rating used by policy engines.
    pub risk_level: HciRiskLevel,
    /// Update rate limit.
    pub rate_limit: HciRateLimit,
    /// Bus export configuration.
    pub export: HciExportChannel,
    /// Whether this rule requires explicit, per-session consent token.
    pub requires_explicit_consent: bool,
    /// If true, this export may NEVER be used for actuation/closed loop.
    pub no_closed_loop_use: bool,
}

/// Top-level HCI export profile bound to a specific DCM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HciExportProfile {
    /// Stable identifier for this profile.
    pub id: Uuid,
    /// Human-readable name (e.g., "OpenBCI-CA-Lab-Coarse-HCI").
    pub name: String,
    /// Version string for change management.
    pub version: String,
    /// The DCM this profile was authored against.
    pub device_manifest_id: Uuid,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Set of rules describing what may be exported to HCI/XR.
    pub rules: Vec<HciExportRule>,
}

impl HciExportProfile {
    /// Basic structural validation and cross-checks with the DCM.
    pub fn validate_against(
        &self,
        dcm: &DeviceCapabilityManifest,
    ) -> Result<(), CyconeticsBciError> {
        if self.device_manifest_id != dcm.id {
            return Err(CyconeticsBciError::ManifestViolation(
                "HCI profile device_manifest_id mismatch".into(),
            ));
        }
        if self.rules.is_empty() {
            return Err(CyconeticsBciError::ManifestViolation(
                "HCI profile must define at least one export rule".into(),
            ));
        }

        // Example policy: no High-risk exports for High-privacy devices.
        if matches!(dcm.privacy, crate::dcm::PrivacyLevel::High) {
            for rule in &self.rules {
                if matches!(rule.risk_level, HciRiskLevel::High) {
                    return Err(CyconeticsBciError::ManifestViolation(
                        "High-privacy DCM cannot have High-risk HCI export rules".into(),
                    ));
                }
            }
        }

        // Example policy: at least one jurisdiction overlap.
        for rule in &self.rules {
            if rule
                .jurisdictions
                .iter()
                .all(|j| !dcm.jurisdictions.contains(j))
            {
                return Err(CyconeticsBciError::ManifestViolation(
                    "HCI rule has no jurisdiction overlap with DCM".into(),
                ));
            }
        }

        Ok(())
    }
}
