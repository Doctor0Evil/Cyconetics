use serde::{Deserialize, Serialize};

use cyconetics_bci_core::dcm::{DeviceCapabilityManifest, Jurisdiction};
use cyconetics_bci_core::error::CyconeticsBciError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteProfile {
    pub id: String,
    pub label: String,
    pub jurisdictions: Vec<Jurisdiction>,
    pub max_hazard_level: u8,
    pub allowed_zone_prefixes: Vec<String>,
    /// Maximum allowed risk level label for this site (e.g., "moderate").
    pub max_risk_level: String,
}

impl SiteProfile {
    pub fn can_use_device_in_zone(
        &self,
        manifest: &DeviceCapabilityManifest,
        zone_id: &str,
        hazard_level: u8,
    ) -> Result<(), CyconeticsBciError> {
        use CyconeticsBciError::ManifestViolation;

        if hazard_level > self.max_hazard_level {
            return Err(ManifestViolation(format!(
                "Requested hazard_level {} exceeds site max {}",
                hazard_level, self.max_hazard_level
            )));
        }

        let mut prefix_ok = false;
        for p in &self.allowed_zone_prefixes {
            if zone_id.starts_with(p) {
                prefix_ok = true;
                break;
            }
        }
        if !prefix_ok {
            return Err(ManifestViolation(format!(
                "Zone '{}' is not recognized for site '{}'",
                zone_id, self.id
            )));
        }

        // Jurisdiction overlap.
        let mut jurisdiction_overlap = false;
        for j in &manifest.jurisdictions {
            if self.jurisdictions.contains(j) {
                jurisdiction_overlap = true;
                break;
            }
        }
        if !jurisdiction_overlap {
            return Err(ManifestViolation(
                "Device jurisdictions incompatible with site profile".into(),
            ));
        }

        // DCM’s own XR-grid constraints.
        if hazard_level < manifest.xr_grid.min_hazard_level
            || hazard_level > manifest.xr_grid.max_hazard_level
        {
            return Err(ManifestViolation(format!(
                "Hazard level {} outside device range [{}, {}]",
                hazard_level,
                manifest.xr_grid.min_hazard_level,
                manifest.xr_grid.max_hazard_level
            )));
        }

        if !manifest.xr_grid.allowed_zones.iter().any(|z| z == zone_id) {
            return Err(ManifestViolation(format!(
                "Zone '{}' not listed in device allowed_zones",
                zone_id
            )));
        }

        // K/S/R-based check: block devices above site’s max risk level.
        let device_level = manifest.risk_score.level.as_str();
        let max_level = self.max_risk_level.as_str();

        fn level_rank(level: &str) -> u8 {
            match level {
                "low" => 1,
                "moderate" => 2,
                "high" => 3,
                "extreme" => 4,
                _ => 4,
            }
        }

        if level_rank(device_level) > level_rank(max_level) {
            return Err(ManifestViolation(format!(
                "Device risk level '{}' exceeds site max '{}'",
                device_level, max_level
            )));
        }

        Ok(())
    }
}

pub fn site_profile_california() -> SiteProfile {
    SiteProfile {
        id: "US-CA-XRGRID-1".into(),
        label: "California XR Grid".into(),
        jurisdictions: vec![Jurisdiction::UsCa],
        max_hazard_level: 3,
        allowed_zone_prefixes: vec!["CA-".into(), "CA-LA-".into(), "CA-SF-".into()],
        max_risk_level: "moderate".into(),
    }
}

pub fn site_profile_arizona() -> SiteProfile {
    SiteProfile {
        id: "US-AZ-XRGRID-1".into(),
        label: "Arizona XR Grid".into(),
        jurisdictions: vec![Jurisdiction::UsAz],
        max_hazard_level: 3,
        allowed_zone_prefixes: vec!["AZ-PHX-".into(), "AZ-".into()],
        max_risk_level: "high".into(),
    }
}
