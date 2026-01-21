// Example artifact: Refined CybostateFactorV2 struct
/// Represents the user's real-time state, broken down into components relevant for RoH calculation.
pub struct CybostateFactorV2 {
    pub physiological_load: PhysiologicalMetrics,
    pub cognitive_load: CognitiveMetrics,
    pub security_integrity: f32, // Normalized score from 0.0 to 1.0
}

pub struct PhysiologicalMetrics {
    pub hr_mean: f32,        // Mean heart rate in BPM
    pub hrv_rmssd: f32,      // Root Mean Square of Successive Differences in ms
    pub eda_mean: f32,       // Mean skin conductance in µS
}

pub struct CognitiveMetrics {
    pub tbr_frontal_midline: f32, // Theta/Beta Ratio from Fz electrode
    pub alpha_asymmetry: f32,    // Left frontal alpha - Right frontal alpha power
    pub gamma_power: f32,        // Average gamma power across frontal sensors
}
