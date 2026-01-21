use serde::{Deserialize, Serialize};

use cyconetics_bci_core::dcm::{DeviceCapabilityManifest, Jurisdiction};
use cyconetics_bci_core::error::CyconeticsBciError;

/// Site-specific profile (e.g., CA lab vs AZ lab).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteProfile {
    pub id: String,
    pub label: String,
    pub jurisdictions: Vec<Jurisdiction>,
    /// Default maximum hazard level allowed at this site.
    pub max_hazard_level: u8,
    /// Zone ID prefixes that are recognized for this site.
    pub allowed_zone_prefixes: Vec<String>,
}

impl SiteProfile {
    /// Check if a device (via its manifest) may be used in a zone at a given hazard level.
    ///
    /// This combines site-level constraints (jurisdiction, zone prefix, max hazard)
    /// with device-level XR-grid constraints from the DCM.
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

        // Zone must match one of the site's known prefixes.
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

        // Require at least one overlapping jurisdiction between device and site.
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

        // Device-level XR-grid hazard band.
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

        // Device must explicitly list the zone in its allowed_zones.
        if !manifest.xr_grid.allowed_zones.iter().any(|z| z == zone_id) {
            return Err(ManifestViolation(format!(
                "Zone '{}' not listed in device allowed_zones",
                zone_id
            )));
        }

        Ok(())
    }
}

/// Safety-oriented California site profile.
/// Modeled on general biosafety + lab zoning concepts rather than specific statutes.
pub fn site_profile_california() -> SiteProfile {
    SiteProfile {
        id: "US-CA-XRGRID-1".into(),
        label: "California XR Grid".into(),
        jurisdictions: vec![Jurisdiction::UsCa],
        max_hazard_level: 3,
        allowed_zone_prefixes: vec![
            "CA-".into(),
            "CA-LA-".into(),
            "CA-SF-".into(),
        ],
    }
}

/// Phoenix-focused Arizona site profile.
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
