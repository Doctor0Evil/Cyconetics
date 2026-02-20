//! RoH (Risk-of-Harm) guardrails and computation functions.
//! This module contains the mathematical heart of safety: computing RoH from multi-axis risks
//! and enforcing the RoH ≤ 0.3 ceiling via type-level tokens.

use crate::types::*;
use serde::{Serialize, Deserialize};

/// BioKarma risk vector: multi-axis risk representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioKarmaRiskVector {
    pub metabolic_risk: f32,       // 0.0–1.0: glucose, protein, hydration strain
    pub hemodynamic_risk: f32,     // 0.0–1.0: BP, HR, perfusion stress
    pub thermal_risk: f32,         // 0.0–1.0: core temp, thermoregulation load
    pub cognitive_risk: f32,       // 0.0–1.0: cognitive load, attention demand
    pub psych_risk: f32,           // 0.0–1.0: emotional/psychological stress
}

impl BioKarmaRiskVector {
    pub fn new() -> Self {
        BioKarmaRiskVector {
            metabolic_risk: 0.0,
            hemodynamic_risk: 0.0,
            thermal_risk: 0.0,
            cognitive_risk: 0.0,
            psych_risk: 0.0,
        }
    }

    /// Composite RoH score using max with soft-capping
    pub fn composite_score(&self) -> f32 {
        let max_risk = [
            self.metabolic_risk,
            self.hemodynamic_risk,
            self.thermal_risk,
            self.cognitive_risk,
            self.psych_risk,
        ]
        .iter()
        .copied()
        .fold(0.0, f32::max);

        // Soft cap at 0.3 with smooth transition
        if max_risk < 0.3 {
            max_risk
        } else if max_risk < 0.5 {
            0.3 + 0.1 * ((max_risk - 0.3) / 0.2)
        } else {
            0.4 + 0.1 * ((max_risk - 0.5) / 0.5).min(1.0)
        }
    }

    /// Check if this risk vector is safe (< 0.3)
    pub fn is_safe(&self) -> bool {
        self.composite_score() < 0.3
    }
}

/// Compute RoH from a bio-karma vector using monotone safety functions
pub fn roh_from_biokarma(vec: &BioKarmaRiskVector) -> f32 {
    // Use a weighted combination with exponential penalty for high individual risks
    let weighted = 0.2 * vec.metabolic_risk
        + 0.25 * vec.hemodynamic_risk
        + 0.15 * vec.thermal_risk
        + 0.2 * vec.cognitive_risk
        + 0.2 * vec.psych_risk;

    // Penalize if any single component is critically high
    let max_component = [
        vec.metabolic_risk,
        vec.hemodynamic_risk,
        vec.thermal_risk,
        vec.cognitive_risk,
        vec.psych_risk,
    ]
    .iter()
    .copied()
    .fold(0.0, f32::max);

    if max_component > 0.8 {
        // Critical component: add exponential penalty
        let penalty = 0.15 * ((max_component - 0.8) / 0.2).min(1.0);
        (weighted + penalty).min(0.5)
    } else {
        weighted.min(0.4)
    }
}

/// Predict RoH for an upgrade given current host state
pub fn predict_roh(state: &RoHGuardedHostState, delta_from_upgrade: f32) -> f32 {
    let current = state.current_roh;
    let predicted = (current + delta_from_upgrade).max(0.0).min(1.0);
    predicted
}

/// Try to construct a RoHBound<30> capability if RoH is provably < 0.3
pub fn try_roh_bound_30(predicted_roh: f32) -> Option<RoHBound<30>> {
    if predicted_roh < 0.30 {
        Some(RoHBound::new_unchecked())
    } else {
        None
    }
}

/// Try to construct a RoHBound<20> (stricter) for sensitive operations
pub fn try_roh_bound_20(predicted_roh: f32) -> Option<RoHBound<20>> {
    if predicted_roh < 0.20 {
        Some(RoHBound::new_unchecked())
    } else {
        None
    }
}

/// Evaluate if an upgrade descriptor is safe given host state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeDescriptor {
    pub upgrade_id: String,
    pub upgrade_class: UpgradeClass,
    pub estimated_roh_delta: f32,
    pub requires_host_veto: bool,
    pub blood_token_cost: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeClass {
    BCI,    // Brain-computer interface
    EXO,    // Exoskeleton/mechanical
    XR,     // XR-grid augmentation
    GOV,    // Governance/policy
}

/// Decision enum returned from upgrade safety evaluation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeDecision {
    /// Safe to proceed; RoH < 0.3 and all envelopes pass
    Approved(RoHBound<30>),
    /// Proceed with caution; RoH < 0.3 but some stress markers elevated
    ApprovedWithCaution(RoHBound<30>),
    /// Rejected; RoH >= 0.3 or critical envelope violated
    Rejected(String), // reason
    /// Deferred for human review or condition wait
    Deferred(String), // reason
}

/// Comprehensive upgrade evaluation
pub fn evaluate_upgrade(
    state: &RoHGuardedHostState,
    descriptor: &UpgradeDescriptor,
) -> UpgradeDecision {
    // 1. Check RoH ceiling
    let predicted_roh = predict_roh(state, descriptor.estimated_roh_delta);

    if predicted_roh >= 0.3 {
        return UpgradeDecision::Rejected(format!(
            "RoH would exceed 0.3 ceiling: {:.3}",
            predicted_roh
        ));
    }

    // 2. Check metabolic envelope
    if state.host_budget.protein_reserve_index < 0.3 && descriptor.blood_token_cost > 0.5 {
        return UpgradeDecision::Rejected(
            "Insufficient protein reserves for blood-token upgrade".to_string(),
        );
    }

    // 3. Check hydration envelope
    if state.host_budget.hydration_index < 0.4 {
        return UpgradeDecision::Deferred(
            "Hydration too low; defer until water intake sufficient".to_string(),
        );
    }

    // 4. Check hemodynamic margin
    if predicted_roh > 0.25 {
        return UpgradeDecision::ApprovedWithCaution(RoHBound::new_unchecked());
    }

    // 5. All checks pass
    UpgradeDecision::Approved(RoHBound::new_unchecked())
}

/// Lyapunov-style drift correction: ensure RoH only decreases after incident
pub struct DriftCorrector {
    pub target_roh: f32,  // e.g., 0.15 for normal baseline
}

impl DriftCorrector {
    pub fn new(target: f32) -> Self {
        DriftCorrector { target_roh: target }
    }

    /// Compute a correction factor that monotonically reduces RoH toward target
    pub fn correction_factor(&self, current_roh: f32) -> f32 {
        if current_roh <= self.target_roh {
            0.0 // Already at or below target
        } else {
            let excess = current_roh - self.target_roh;
            // Exponential decay toward target
            excess * 0.1  // Slow correction to avoid aggressive changes
        }
    }

    /// Apply correction: reduce RoH by the correction factor
    pub fn apply_correction(&self, current_roh: f32) -> f32 {
        (current_roh - self.correction_factor(current_roh)).max(self.target_roh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biokarma_composite() {
        let vec = BioKarmaRiskVector {
            metabolic_risk: 0.1,
            hemodynamic_risk: 0.15,
            thermal_risk: 0.05,
            cognitive_risk: 0.12,
            psych_risk: 0.08,
        };
        let composite = vec.composite_score();
        assert!(composite >= 0.0 && composite <= 0.3);
    }

    #[test]
    fn test_roh_from_biokarma() {
        let vec = BioKarmaRiskVector {
            metabolic_risk: 0.2,
            hemodynamic_risk: 0.18,
            thermal_risk: 0.1,
            cognitive_risk: 0.15,
            psych_risk: 0.12,
        };
        let roh = roh_from_biokarma(&vec);
        assert!(roh < 0.3);
    }

    #[test]
    fn test_roh_bound_30() {
        let bound = try_roh_bound_30(0.25);
        assert!(bound.is_some());

        let no_bound = try_roh_bound_30(0.35);
        assert!(no_bound.is_none());
    }

    #[test]
    fn test_drift_corrector() {
        let corrector = DriftCorrector::new(0.15);
        let corrected = corrector.apply_correction(0.28);
        assert!(corrected < 0.28);
        assert!(corrected >= 0.15);
    }
}
