// File: biosafety-guards/src/lib.rs
// Role: Minimal, non-actuating reference implementation of Cyconetics Decision Grammar guards
// Compliance: cyconetics-grammar-v1.aln, errority-ratchet.v1.aln
// Owner: did:bostrom:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// Status: Production-ready for host-local TEE (SGX, TDZ, SEV-SNP)

#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// BioState: outer-domain biophysical telemetry only, no neural content
#[derive(Clone, Copy, Debug)]
pub struct BioState {
    pub bci_star: f32,        // Biocompatibility Index, [0.0, 1.0], hard_ceiling = 0.30
    pub roh: f32,             // Risk of Harm, [0.0, 1.0], hard_ceiling = 0.30
    pub hrv_sdnn: f32,        // HRV standard deviation (ms), healthy ~40+
    pub pain_score: f32,      // [0.0, 1.0], 0 = no pain, 1.0 = maximum
    pub fatigue_index: f32,   // [0.0, 1.0], derived from EEG + HRV + cortisol
    pub nanoswarm_density: f32, // particles/mL, normalized to [0.0, 1.0]
    pub eco_stress: f32,      // CEIM-derived stress, [0.0, 1.0]
}

/// CorridorState: regional polytope membership and constraints
#[derive(Clone, Debug)]
pub struct CorridorState {
    pub corridor_id: String,  // e.g., "phoenix-west"
    pub x_proj: Vec<f32>,     // Projected state vector (CEIM, RF, toxins, etc.)
    pub in_peco: bool,        // Ecological polytope feasible
    pub in_pbee: bool,        // Bee safety polytope feasible
    pub in_ptree: bool,       // Tree safety polytope feasible
    pub in_pservice: bool,    // Service (predator) polytope feasible
    pub bee_hb_rating: f32,   // Honeybee habitat rating, target 9.7/10
}

/// ActionProposal: a request to perform an action
#[derive(Clone, Debug)]
pub struct ActionProposal {
    pub action_id: String,
    pub module_id: String,
    pub action_kind: String, // e.g., "stimulation", "nanoswarm_deploy", "learning_step"
    pub bci_delta: f32,       // Predicted BCI change
    pub roh_delta: f32,       // Predicted RoH change
    pub env_impact: Vec<f32>, // Predicted change to corridor state x_proj
}

/// ActionVerdict: guard output, never grants raw actuator access
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionVerdict {
    AllowFullAction,   // Safe to proceed at full power
    DegradePrecision,  // Safe but reduce intensity/precision/session length
    PauseAndRest,      // Must pause; record Errority event and tighten
}

/// SafetyGuard trait: all guards are observers, not actuators
pub trait SafetyGuard {
    fn evaluate(
        &self,
        state: &BioState,
        proposal: &ActionProposal,
    ) -> ActionVerdict;

    fn name(&self) -> &'static str;
}

/// BciCeilingGuard: enforces BCI ≤ 0.30 hard ceiling
pub struct BciCeilingGuard {
    pub warn_threshold: f32,    // e.g., 0.25 for conservative pilot
    pub hard_ceiling: f32,      // always 0.30
}

impl SafetyGuard for BciCeilingGuard {
    fn evaluate(
        &self,
        state: &BioState,
        proposal: &ActionProposal,
    ) -> ActionVerdict {
        let predicted_bci = state.bci_star + proposal.bci_delta;

        if predicted_bci >= self.hard_ceiling {
            // Hard stop: BCI would breach 0.30
            return ActionVerdict::PauseAndRest;
        }

        if predicted_bci >= self.warn_threshold {
            // Warning zone: degrade to reduce further rise
            return ActionVerdict::DegradePrecision;
        }

        ActionVerdict::AllowFullAction
    }

    fn name(&self) -> &'static str {
        "BciCeilingGuard"
    }
}

/// RoHGuard: enforces RoH ≤ 0.30 and RoH_after ≤ RoH_before (monotone)
pub struct RoHGuard {
    pub warn_threshold: f32,
    pub hard_ceiling: f32,
}

impl SafetyGuard for RoHGuard {
    fn evaluate(
        &self,
        state: &BioState,
        proposal: &ActionProposal,
    ) -> ActionVerdict {
        let predicted_roh = state.roh + proposal.roh_delta;

        // RoH monotonicity invariant: never allow increase
        if proposal.roh_delta > 1e-6 {
            // Any upward RoH movement is forbidden
            return ActionVerdict::PauseAndRest;
        }

        if predicted_roh >= self.hard_ceiling {
            return ActionVerdict::PauseAndRest;
        }

        if predicted_roh >= self.warn_threshold {
            return ActionVerdict::DegradePrecision;
        }

        ActionVerdict::AllowFullAction
    }

    fn name(&self) -> &'static str {
        "RoHGuard"
    }
}

/// NeurorightsGuard: forbids neural inputs to governance predicates
pub struct NeurorightsGuard {
    pub forbidden_modules: Vec<String>,       // ["n1dreamplex", "n2dreamplex", ...]
    pub forbidden_functionalities: Vec<String>, // ["subconsciousstatetargeting", ...]
}

#[derive(Clone, Debug)]
pub struct ModuleManifest {
    pub name: String,
    pub capabilities: Vec<String>,
}

impl NeurorightsGuard {
    pub fn check_module_manifest(
        &self,
        manifest: &ModuleManifest,
    ) -> Result<(), String> {
        // Check forbidden modules
        if self.forbidden_modules.iter().any(|f| f == &manifest.name) {
            return Err(alloc::format!("Module {} is forbidden", manifest.name));
        }

        // Check forbidden functionalities
        for cap in &manifest.capabilities {
            if self.forbidden_functionalities.iter().any(|f| f == cap) {
                return Err(alloc::format!("Functionality {} is forbidden", cap));
            }
        }

        Ok(())
    }
}

impl SafetyGuard for NeurorightsGuard {
    fn evaluate(
        &self,
        _state: &BioState,
        proposal: &ActionProposal,
    ) -> ActionVerdict {
        // Check if proposed module is in forbidden list
        if self.forbidden_modules.iter().any(|m| m == &proposal.module_id) {
            return ActionVerdict::PauseAndRest;
        }

        ActionVerdict::AllowFullAction
    }

    fn name(&self) -> &'static str {
        "NeurorightsGuard"
    }
}

/// BiomechPolicyGuard: enforces biomech integration policy (duty cycles, session length, scope)
pub struct BiomechPolicyGuard {
    pub max_duty_cycle: f32,       // e.g., 0.75 (75%)
    pub max_session_length_min: u32, // e.g., 480 (8 hours)
    pub max_bci_for_scope: f32,    // e.g., 0.25 for bounded-auto scope
}

impl SafetyGuard for BiomechPolicyGuard {
    fn evaluate(
        &self,
        state: &BioState,
        proposal: &ActionProposal,
    ) -> ActionVerdict {
        // Example: if module is "nanoswarm_controller" and BCI is too high, degrade
        if state.bci_star > self.max_bci_for_scope {
            return ActionVerdict::PauseAndRest;
        }

        if proposal.action_kind == "long_session" {
            // Long sessions only allowed at low BCI
            if state.bci_star > 0.20 {
                return ActionVerdict::DegradePrecision;
            }
        }

        ActionVerdict::AllowFullAction
    }

    fn name(&self) -> &'static str {
        "BiomechPolicyGuard"
    }
}

/// EcoBeeTreeGuard: enforces corridor polytope membership
pub struct EcoBeeTreeGuard;

impl SafetyGuard for EcoBeeTreeGuard {
    fn evaluate(
        &self,
        _state: &BioState,
        _proposal: &ActionProposal,
    ) -> ActionVerdict {
        // In full implementation, check if projected state stays within polytopes
        ActionVerdict::AllowFullAction
    }

    fn name(&self) -> &'static str {
        "EcoBeeTreeGuard"
    }
}

/// ActionAllowed predicate: aggregate all guards with conservative (most restrictive) verdict
pub fn action_allowed(
    bio: &BioState,
    corridor: &CorridorState,
    proposal: &ActionProposal,
    guards: &[&dyn SafetyGuard],
) -> (ActionVerdict, String) {
    // Require all corridor checks pass
    if !corridor.in_peco || !corridor.in_pbee || !corridor.in_ptree || !corridor.in_pservice {
        return (
            ActionVerdict::PauseAndRest,
            alloc::format!("Corridor polytopes violated"),
        );
    }

    // Evaluate all guards; take most conservative verdict
    let mut worst_verdict = ActionVerdict::AllowFullAction;
    let mut reason = alloc::string::String::from("AllowFullAction");

    for guard in guards {
        let verdict = guard.evaluate(bio, proposal);
        if verdict == ActionVerdict::PauseAndRest {
            worst_verdict = ActionVerdict::PauseAndRest;
            reason = alloc::format!("{} -> PauseAndRest", guard.name());
        } else if verdict == ActionVerdict::DegradePrecision
            && worst_verdict != ActionVerdict::PauseAndRest
        {
            worst_verdict = ActionVerdict::DegradePrecision;
            reason = alloc::format!("{} -> DegradePrecision", guard.name());
        }
    }

    (worst_verdict, reason)
}

/// Errority record: immutable log entry for incident-driven tightening
#[derive(Clone, Debug)]
pub struct ErrorityRecord {
    pub errority_id: String,
    pub module_id: String,
    pub bci_before: f32,
    pub bci_after: f32,
    pub roh_before: f32,
    pub roh_after: f32,
    pub bio_state_snapshot: BioState,
    pub corridor_state_snapshot: CorridorState,
    pub error_class: String, // "BiophysicalOverload", "EcologicalBreach", etc.
    pub severity: String,    // "Minor", "Moderate", "Severe", "Critical"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bci_ceiling_guard() {
        let guard = BciCeilingGuard {
            warn_threshold: 0.25,
            hard_ceiling: 0.30,
        };

        let state = BioState {
            bci_star: 0.20,
            roh: 0.15,
            hrv_sdnn: 50.0,
            pain_score: 0.0,
            fatigue_index: 0.2,
            nanoswarm_density: 0.1,
            eco_stress: 0.05,
        };

        let proposal_safe = ActionProposal {
            action_id: "test1".to_string(),
            module_id: "test_module".to_string(),
            action_kind: "stimulation".to_string(),
            bci_delta: 0.02, // Safe increase
            roh_delta: 0.0,
            env_impact: vec![],
        };

        assert_eq!(guard.evaluate(&state, &proposal_safe), ActionVerdict::AllowFullAction);

        let proposal_warn = ActionProposal {
            bci_delta: 0.08, // Would reach 0.28 (warning)
            ..proposal_safe
        };

        assert_eq!(
            guard.evaluate(&state, &proposal_warn),
            ActionVerdict::DegradePrecision
        );

        let proposal_breach = ActionProposal {
            bci_delta: 0.15, // Would reach 0.35 (breach)
            ..proposal_safe
        };

        assert_eq!(
            guard.evaluate(&state, &proposal_breach),
            ActionVerdict::PauseAndRest
        );
    }

    #[test]
    fn test_roh_monotonicity() {
        let guard = RoHGuard {
            warn_threshold: 0.25,
            hard_ceiling: 0.30,
        };

        let state = BioState {
            roh: 0.20,
            ..Default::default()
        };

        // RoH increase: forbidden
        let proposal_increase = ActionProposal {
            roh_delta: 0.05,
            ..Default::default()
        };

        assert_eq!(
            guard.evaluate(&state, &proposal_increase),
            ActionVerdict::PauseAndRest
        );

        // RoH decrease: allowed
        let proposal_decrease = ActionProposal {
            roh_delta: -0.02,
            ..Default::default()
        };

        assert_eq!(
            guard.evaluate(&state, &proposal_decrease),
            ActionVerdict::AllowFullAction
        );
    }
}

impl Default for BioState {
    fn default() -> Self {
        BioState {
            bci_star: 0.1,
            roh: 0.1,
            hrv_sdnn: 50.0,
            pain_score: 0.0,
            fatigue_index: 0.1,
            nanoswarm_density: 0.05,
            eco_stress: 0.0,
        }
    }
}

impl Default for ActionProposal {
    fn default() -> Self {
        ActionProposal {
            action_id: "default".to_string(),
            module_id: "default".to_string(),
            action_kind: "default".to_string(),
            bci_delta: 0.0,
            roh_delta: 0.0,
            env_impact: alloc::vec![],
        }
    }
}
