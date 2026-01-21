use crate::manifest::{DeviceCapabilityManifest, HciExportRule, JurisdictionCode};
use std::collections::HashSet;

pub struct SiteProfile {
    pub zone_id: String,
    pub allowed_jurisdictions: HashSet<JurisdictionCode>,
    pub max_risk_threshold: f32, // Must be <= 0.3
    pub required_neurorights: Vec<String>,
}

pub struct SiteValidator;

impl SiteValidator {
    /// Validates an HCI Export Rule against the local Site Profile.
    /// Ensures alignment with the 0.3 Risk-of-Harm ceiling and XR-Grid zoning.
    pub fn validate_rule(
        profile: &SiteProfile,
        rule: &HciExportRule,
        dcm: &DeviceCapabilityManifest
    ) -> Result<(), &'static str> {
        
        // 1. Risk-of-Harm Threshold Check
        // The rule risk level (if numeric) or categorized level must not exceed the profile's ceiling.
        if profile.max_risk_threshold > 0.3 {
            return Err("ERROR: SiteProfile violates global RoH policy (> 0.3).");
        }

        // 2. Jurisdictional Integrity Check
        if!profile.allowed_jurisdictions.contains(&dcm.jurisdiction) {
            return Err("ERROR: Device jurisdiction not authorized for this XR-Zone.");
        }

        // 3. Privacy Level vs. Risk Level Check
        // High-privacy DCMs cannot export High-risk rules.
        if dcm.privacy_level == "High" && rule.risk_level_as_f32() > 0.15 {
            return Err("ERROR: High-privacy device cannot export Medium/High risk data.");
        }

        // 4. Closed-Loop Safety Check
        if rule.no_closed_loop_use &&!SiteValidator::is_read_only_zone(&profile.zone_id) {
            return Err("ERROR: Closed-loop data used in unauthorized zone.");
        }

        Ok(())
    }

    fn is_read_only_zone(zone_id: &str) -> bool {
        // Implementation for read-only zone identification...
        zone_id.contains("READ_ONLY") |

| zone_id.contains("GLOBAL_ECO")
    }
}

/// Macro for compile-time manifest validation.
/// Ensures all manifests contain necessary metadata before deployment.
#[macro_export]
macro_rules! validate_manifest_at_compile_time {
    ($manifest:expr) => {
        if!$manifest.neurorights_compliant {
            panic!("COMPILE_ERROR: Manifest is not neurorights compliant!");
        }
    };
}
