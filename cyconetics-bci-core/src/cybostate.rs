use serde::{Deserialize, Serialize};

/// Legacy, already deployed shape (V1).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CybostateFactorV1 {
    pub physiological_load: f32, // 0.0–1.0
    pub cognitive_load: f32,     // 0.0–1.0
    pub security_integrity: f32, // 0.0–1.0 (1.0 = fully intact)
}

impl CybostateFactorV1 {
    pub fn calculate_roh(&self) -> f32 {
        // Mirrors existing weights in organiccpuscheduler.rs.[file:3]
        self.physiological_load * 0.5
            + self.cognitive_load * 0.3
            + (1.0 - self.security_integrity) * 0.2
    }
}

/// RoH‑native, metric‑grounded V2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CybostateFactorV2 {
    pub physiological: PhysiologicalMetrics,
    pub cognitive: CognitiveMetrics,
    /// 0.0–1.0 anomaly / integrity score from security telemetry.
    pub security_integrity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalMetrics {
    pub hr_mean_bpm: f32,
    pub hrv_rmssd_ms: f32,
    pub eda_mean_microsiemens: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveMetrics {
    pub tbr_frontal_midline: f32,
    pub alpha_asymmetry: f32,
    pub fm_theta_power: f32,
    pub de_complexity: f32,
}

impl CybostateFactorV2 {
    /// Normalizes rich metrics into V1‑style loads, then computes RoH.
    pub fn calculate_roh(&self) -> f32 {
        let phys_load = normalize_physiological(&self.physiological);
        let cog_load = normalize_cognitive(&self.cognitive);
        phys_load * 0.5 + cog_load * 0.3 + (1.0 - self.security_integrity) * 0.2
    }
}

fn normalize_physiological(p: &PhysiologicalMetrics) -> f32 {
    // Placeholder normalization; tuned during lab‑grid calibration.[file:3]
    let hr_term = (p.hr_mean_bpm / 100.0).clamp(0.0, 1.0);
    let hrv_term = (1.0 - (p.hrv_rmssd_ms / 80.0)).clamp(0.0, 1.0);
    let eda_term = (p.eda_mean_microsiemens / 20.0).clamp(0.0, 1.0);
    (hr_term + hrv_term + eda_term) / 3.0
}

fn normalize_cognitive(c: &CognitiveMetrics) -> f32 {
    let tbr_term = (c.tbr_frontal_midline / 4.0).clamp(0.0, 1.0);
    let theta_term = (c.fm_theta_power / 1.5).clamp(0.0, 1.0);
    let de_term = (c.de_complexity / 5.0).clamp(0.0, 1.0);
    let asym_term = ((c.alpha_asymmetry.abs()) / 5.0).clamp(0.0, 1.0);
    (tbr_term + theta_term + de_term + asym_term) / 4.0
}

impl From<CybostateFactorV2> for CybostateFactorV1 {
    fn from(v2: CybostateFactorV2) -> Self {
        let phys_load = normalize_physiological(&v2.physiological);
        let cog_load = normalize_cognitive(&v2.cognitive);
        CybostateFactorV1 {
            physiological_load: phys_load,
            cognitive_load: cog_load,
            security_integrity: v2.security_integrity,
        }
    }
}
