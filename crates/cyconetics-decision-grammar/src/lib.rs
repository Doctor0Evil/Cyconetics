//! # Cyconetics Decision Grammar
//!
//! A typed, neurorights-bound decision framework for brain-computer interface (BCI) governance.
//! Enforces RoH ≤ 0.3, host sovereignty, and ecological consent through compile-time and runtime
//! invariants, procedural macros, and ALN shard binding.
//!
//! ## Core Principles
//!
//! 1. **Type-driven governance**: Decision verbs and roles are encoded in Rust types, making invalid
//!    decisions structurally impossible.
//! 2. **RoH ≤ 0.3 guardrail**: Every upgrade path must prove it remains below this ceiling.
//! 3. **Host veto**: HostSelf role has unremovable veto paths for all high-impact decisions.
//! 4. **Incident-driven tightening**: SafetyEpoch logs trigger automatic policy updates when near-misses occur.
//! 5. **Blockchain-authored**: All decisions are stamped into cyberswarm.decision.ledger.v1 for audit.

pub mod types;
pub mod roles;
pub mod roh_guard;
pub mod ledger;
pub mod validators;
pub mod aln_shards;
pub mod ci_hooks;

// Re-export macros
pub use cyconetics_decision_grammar_macros::*;

// Re-export key types
pub use types::{
    DecisionKind, DecisionRecord, RoHBound, RoHGuardedHostState, KsrBand, NeurorightsTag,
    NeuroEntityType, NeuroConsentRecord, BFCBroadcastProposal,
};
pub use roles::{
    DecisionContext, NeurorightsDecider, SafetyDecider, HostSelfDecider, GovSafetyDecider,
};
pub use roh_guard::{BrainSpecs, predict_roh, roh_from_biokarma};
pub use ledger::{
    DecisionLedgerEntry, DecisionLedgerKey, EvidenceBundle, EvidenceBiomarkers,
    BloodSpendProof,
};
pub use validators::{
    validate_roh_complies_with_zone, validate_decision_record, NeuroConsentViolation,
};
pub use aln_shards::{DecisionLedgerShard, NeurorightsConsentShard, neurorights_broadcast_ledger};

use std::collections::HashMap;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use chrono::Utc;

/// Global decision registry for audit and traceability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRegistry {
    /// Immutable append-only log of all decisions
    pub decisions: Vec<DecisionRecord>,
    /// Index for fast lookup by host_did, upgrade_id
    pub index: HashMap<String, Vec<usize>>,
    /// Last known SafetyEpoch that tightened policy
    pub last_policy_tightening: Option<String>,
}

impl DecisionRegistry {
    pub fn new() -> Self {
        Self {
            decisions: Vec::new(),
            index: HashMap::new(),
            last_policy_tightening: None,
        }
    }

    /// Append a decision record (append-only, never overwrite)
    pub fn append(&mut self, record: DecisionRecord) -> String {
        let idx = self.decisions.len();
        let key = format!("{}:{}:{}", record.host_did, record.upgrade_id, record.evolution_id);
        self.index.entry(key).or_insert_with(Vec::new).push(idx);
        self.decisions.push(record);
        
        // Return hash of this entry for blockchain stamping
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&self.decisions[idx]).unwrap().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Fetch all decisions for a given host_did and upgrade_id
    pub fn lookup(&self, host_did: &str, upgrade_id: &str) -> Vec<&DecisionRecord> {
        let key = format!("{}:{}", host_did, upgrade_id);
        self.index
            .get(&key)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.decisions.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Serialize for blockchain commit
    pub fn to_jsonl(&self) -> String {
        self.decisions
            .iter()
            .filter_map(|d| serde_json::to_string(d).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Safety-Epoch incident tracker for policy tightening
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyEpochLog {
    pub epoch_id: String,
    pub timestamp: i64,
    pub incident_type: IncidentType,
    pub affected_zones: Vec<String>,
    pub roh_measurement: f32,
    pub policy_action: Option<PolicyTightening>,
    pub evidence_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentType {
    RoHTrendingHigh,      // RoH approaching 0.3
    HemodynamicDeviation, // BP/HR/perfusion out of safe range
    MetabolicStrain,      // Glucose/hydration/protein depleted
    NeuroconsentViolation, // Non-host entity modulation attempt
    BloodTokenOverdraw,   // CSP spending exceeded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTightening {
    pub old_roh_ceiling: f32,
    pub new_roh_ceiling: f32,
    pub affected_upgrade_classes: Vec<String>,
    pub reason: String,
}

/// Helper to compute combined RoH from multi-axis risk
pub fn compute_incident_roh(
    base_roh: f32,
    hemodynamic_stress: f32,
    metabolic_strain: f32,
    psych_density: f32,
) -> f32 {
    // RoH = max(base, h_stress, m_strain, p_density) with soft capping
    let combined = [base_roh, hemodynamic_stress, metabolic_strain, psych_density]
        .iter()
        .copied()
        .fold(0.0, f32::max);
    
    // Soft cap at 0.3 with smooth transition
    if combined < 0.3 {
        combined
    } else if combined < 0.5 {
        0.3 + 0.1 * ((combined - 0.3) / 0.2)
    } else {
        0.4 + 0.1 * ((combined - 0.5) / 0.5).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_registry_append() {
        let mut registry = DecisionRegistry::new();
        let record = DecisionRecord {
            host_did: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
            upgrade_id: "upgrade-001".to_string(),
            evolution_id: "evo-001".to_string(),
            decision: DecisionKind::Authorize,
            pre_roh: 0.15,
            post_roh: 0.22,
            brain_specs: BrainSpecs::default(),
            host_budget: Default::default(),
            bci_snapshot: Default::default(),
            ksr_band: KsrBand::default(),
            ledger_key: DecisionLedgerKey {
                host_did: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
                upgrade_id: "upgrade-001".to_string(),
                evolution_id: "evo-001".to_string(),
            },
            timestamp: Utc::now().timestamp(),
        };

        let hash = registry.append(record);
        assert!(!hash.is_empty());
        assert_eq!(registry.decisions.len(), 1);
    }

    #[test]
    fn test_roh_computation() {
        let roh = compute_incident_roh(0.15, 0.12, 0.18, 0.20);
        assert!(roh >= 0.0 && roh <= 0.5);
    }
}
