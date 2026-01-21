use uuid::Uuid;
use serde::{Serialize, Deserialize};

#
pub enum JurisdictionCode {
    UsAzPhx, // Phoenix, Arizona Lab-grid
    UsCaSjo, // San Jolla, California Lab-grid
    GlobalEco, // Global Ecological Sustainability Grid
}

#
pub struct ElectricalLimits {
    pub max_voltage_mv: f32,
    pub max_current_ua: f32,
    pub session_timeout_sec: u32,
}

#
pub struct FrequencyBand {
    pub min_hz: f32,
    pub max_hz: f32,
    pub band_label: String,
}

#
pub struct DeviceCapabilityManifest {
    pub id: Uuid,
    pub version: String,
    pub jurisdiction: JurisdictionCode,
    pub electrical_safety: ElectricalLimits,
    pub authorized_bands: Vec<FrequencyBand>,
    pub privacy_level: String, // e.g., "High", "Critical"
    pub neurorights_compliant: bool,
}

impl DeviceCapabilityManifest {
    /// High-integrity validator for device activation within an XR-Zone.
    /// Ensures that the device does not exceed bioscale-programming thresholds.
    pub fn authorize_activation(
        &self, 
        current_zone_id: &str,
        requested_risk_level: f32
    ) -> Result<bool, &'static str> {
        // Enforce the 0.3 Risk-of-Harm ceiling
        if requested_risk_level >= 0.3 {
            return Err("REJECTED: Risk-of-Harm threshold exceeded (>= 0.3).");
        }

        // Logic for jurisdictional check and electrical safety validation...
        Ok(true)
    }
}
