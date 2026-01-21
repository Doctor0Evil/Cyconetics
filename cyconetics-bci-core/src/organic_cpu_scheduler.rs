use serde::{Deserialize, Serialize};
use crate::hci_profile::HciRiskLevel;

/// Represents the current biophysical state of the augmented citizen.
#
pub struct CybostateFactor {
    pub physiological_load: f32, // 0.0 to 1.0
    pub cognitive_load: f32,     // 0.0 to 1.0
    pub security_integrity: f32, // 0.0 to 1.0
}

impl CybostateFactor {
    /// Calculates the aggregate Risk-of-Harm (RoH) index.
    pub fn calculate_roh(&self) -> f32 {
        (self.physiological_load * 0.5) + (self.cognitive_load * 0.3) + ((1.0 - self.security_integrity) * 0.2)
    }
}

/// The core scheduler responsible for gating neural tasks.
pub struct OrganicCpuScheduler {
    pub roh_threshold: f32, // Strictly capped at 0.3
    pub knowledge_factor: f32,
}

impl OrganicCpuScheduler {
    pub fn new() -> Self {
        Self {
            roh_threshold: 0.3,
            knowledge_factor: 0.0,
        }
    }

    /// Evaluates if a proposed data-retrieval action is safe to execute.
    pub fn evaluate_task(&mut self, state: &CybostateFactor, task_risk: HciRiskLevel) -> Result<String, String> {
        let current_roh = state.calculate_roh();
        
        // Hex-stamp: 0xCYC0-ROH-VAL-PRO
        if current_roh >= self.roh_threshold {
            return Err("REJECTED: Risk-of-Harm threshold exceeded (RoH >= 0.3). Avoiding negative-karma reflection.".into());
        }

        match task_risk {
            HciRiskLevel::High if current_roh > 0.2 => {
                Err("REJECTED: High-risk task requested with elevated current RoH.".into())
            },
            _ => {
                self.knowledge_factor += 0.01; // Increment knowledge contribution
                Ok(format!("AUTHORIZED: Executing task. Current RoH: {:.2}, Knowledge Factor: {:.2}", current_roh, self.knowledge_factor))
            }
        }
    }
}
