use serde::{Deserialize, Serialize};

use cyconetics_bci_core::dcm::{DeviceCapabilityManifest, Jurisdiction};
use cyconetics_bci_core::error::CyconeticsBciError;

/// Site-specific profile (e.g., a CA lab vs AZ lab).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteProfile {
    pub id: String,
    /// Human-readable label, e.g., "California XR Grid".
    pub label: String,
    /// Jurisdictions this site is considered to operate under.
    pub jurisdictions: Vec<Jurisdiction>,
    /// Default maximum hazard level allowed at this site.
    pub max_hazard_level: u8,
    /// Zone ID prefixes considered valid for this site.
    pub allowed_zone_prefixes: Vec<String>,
}

impl SiteProfile {
    /// Basic compatibility check for a DCM, zone, and hazard level.
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

        // Ensure device jurisdictions intersect with site jurisdictions.
        let site_jurisdictions = &self.jurisdictions;
        let mut jurisdiction_overlap = false;
        for j in &manifest.jurisdictions {
            if site_jurisdictions.contains(j) {
                jurisdiction_overlap = true;
                break;
            }
        }
        if !jurisdiction_overlap {
            return Err(ManifestViolation(
                "Device jurisdictions incompatible with site profile".into(),
            ));
        }

        // Defer to DCM XR-grid constraints as well (min/max hazard & allowed_zones).
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

        Ok(())
    }
}

/// Pre-configured California site profile.
///
/// This is a best-effort, safety-oriented mapping rather than a normative
/// legal encoding; it follows general BSL-lab layout and zoning practices.
pub fn site_profile_california() -> SiteProfile {
    SiteProfile {
        id: "US-CA-XRGRID-1".into(),
        label: "California XR Grid".into(),
        jurisdictions: vec![Jurisdiction::UsCa],
        max_hazard_level: 3,
        allowed_zone_prefixes: vec![
            "CA-".into(),         // general CA zones
            "CA-LA-".into(),      // city-specific
            "CA-SF-".into(),
        ],
    }
}

/// Pre-configured Arizona site profile (Phoenix-focused).
pub fn site_profile_arizona() -> SiteProfile {
    SiteProfile {
        id: "US-AZ-XRGRID-1".into(),
        label: "Arizona XR Grid".into(),
        jurisdictions: vec![Jurisdiction::UsAz],
        max_hazard_level: 3,
        allowed_zone_prefixes: vec![
            "AZ-PHX-".into(),
            "AZ-".into(),
        ],
    }
}
