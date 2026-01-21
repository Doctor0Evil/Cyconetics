use serde::{Deserialize, Serialize};

use cyconetics_bci_core::dcm::{DeviceCapabilityManifest, Jurisdiction};
use cyconetics_bci_core::error::CyconeticsBciError;

/// Simple allowed risk bands for a site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskBandPolicy {
    /// Minimum allowed risk band (inclusive) for R.
    /// e.g., "low"
    pub min_band: String,
    /// Maximum allowed risk band (inclusive).
    /// e.g., "high"
    pub max_band: String,
}

impl RiskBandPolicy {
    fn band_rank(band: &str) -> u8 {
        match band {
            "low" => 0,
            "medium" => 1,
            "high" => 2,
            "extreme" => 3,
            _ => 4, // invalid band treated as worst.
        }
    }

    pub fn allows(&self, device_band: &str) -> bool {
        let d = Self::band_rank(device_band);
        let min = Self::band_rank(&self.min_band);
        let max = Self::band_rank(&self.max_band);
        d >= min && d <= max
    }
}

/// Site-specific profile (e.g., a CA lab vs AZ lab).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteProfile {
    pub id: String,
    pub label: String,
    pub jurisdictions: Vec<Jurisdiction>,
    pub max_hazard_level: u8,
    pub allowed_zone_prefixes: Vec<String>,
    /// Risk band policy for devices at this site.
    pub risk_policy: RiskBandPolicy,
}

impl SiteProfile {
    /// Compatibility check for DCM, zone, hazard level, and risk band.
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

        // Device XR-grid hazard bounds.
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

        // Enforce risk band policy: device R-band must fall into site's allowed band window.
        if !self
            .risk_policy
            .allows(&manifest.risk_score.risk_band)
        {
            return Err(ManifestViolation(format!(
                "Device risk band '{}' not allowed at site '{}' (allowed: {}..={})",
                manifest.risk_score.risk_band,
                self.id,
                self.risk_policy.min_band,
                self.risk_policy.max_band
            )));
        }

        Ok(())
    }
}

/// California site profile: accepts low and medium risk devices.
pub fn site_profile_california() -> SiteProfile {
    SiteProfile {
        id: "US-CA-XRGRID-1".into(),
        label: "California XR Grid".into(),
        jurisdictions: vec![Jurisdiction::UsCa],
        max_hazard_level: 3,
        allowed_zone_prefixes: vec!["CA-".into(), "CA-LA-".into(), "CA-SF-".into()],
        risk_policy: RiskBandPolicy {
            min_band: "low".into(),
            max_band: "medium".into(),
        },
    }
}

/// Arizona (Phoenix-focused) site profile: allows up to 'high' risk band.
pub fn site_profile_arizona() -> SiteProfile {
    SiteProfile {
        id: "US-AZ-XRGRID-1".into(),
        label: "Arizona XR Grid".into(),
        jurisdictions: vec![Jurisdiction::UsAz],
        max_hazard_level: 3,
        allowed_zone_prefixes: vec!["AZ-PHX-".into(), "AZ-".into()],
        risk_policy: RiskBandPolicy {
            min_band: "low".into(),
            max_band: "high".into(),
        },
    }
}
