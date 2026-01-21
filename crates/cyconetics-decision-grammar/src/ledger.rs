use serde::{Deserialize, Serialize};

use crate::types::{
    DecisionKind, DecisionLedgerEntry, DecisionRecord, EvolutionAuditRecord,
};

/// Minimal in-memory ledger trait – real deployments should back this with
/// DID-signed, append-only storage under your Cosmos/ALN sidecar.
pub trait DecisionLedger {
    fn append(&mut self, entry: DecisionLedgerEntry);
    fn find_for_triplet(
        &self,
        host_did: &str,
        upgrade_id: &str,
        evolution_id: &str,
        kind: DecisionKind,
    ) -> Option<DecisionLedgerEntry>;
}

/// Simple in-memory implementation useful for tests/CI.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryDecisionLedger {
    entries: Vec<DecisionLedgerEntry>,
}

impl DecisionLedger for MemoryDecisionLedger {
    fn append(&mut self, entry: DecisionLedgerEntry) {
        self.entries.push(entry);
    }

    fn find_for_triplet(
        &self,
        host_did: &str,
        upgrade_id: &str,
        evolution_id: &str,
        kind: DecisionKind,
    ) -> Option<DecisionLedgerEntry> {
        self.entries
            .iter()
            .cloned()
            .find(|e| {
                e.host_did == host_did
                    && e.upgrade_id == upgrade_id
                    && e.evolution_id == evolution_id
                    && e.decision_kind == kind
            })
    }
}

/// Append-only evolution audit manipulations.
impl EvolutionAuditRecord {
    pub fn append_roh(&mut self, roh: f32) {
        self.roh_history.push(roh);
    }

    pub fn append_decision_hash(&mut self, hash: String) {
        self.decision_record_hashes.push(hash);
    }
}

/// CI/sidecar enforcement contracts.
pub trait SidecarGuard {
    /// Must be called before performing any transition from Proposed to
    /// {Approved, Authorized, Rejected, Escalated, Deferred}.
    fn ensure_transition_has_ledger_entry(
        &self,
        host_did: &str,
        upgrade_id: &str,
        evolution_id: &str,
        target: DecisionKind,
        expected_decider_did: &str,
    ) -> Result<(), SidecarError>;
}

impl SidecarGuard for MemoryDecisionLedger {
    fn ensure_transition_has_ledger_entry(
        &self,
        host_did: &str,
        upgrade_id: &str,
        evolution_id: &str,
        target: DecisionKind,
        expected_decider_did: &str,
    ) -> Result<(), SidecarError> {
        let entry = self
            .find_for_triplet(host_did, upgrade_id, evolution_id, target)
            .ok_or(SidecarError::MissingLedgerEntry)?;

        if entry.decided_by_did != expected_decider_did {
            return Err(SidecarError::DidMismatch {
                expected: expected_decider_did.to_string(),
                actual: entry.decided_by_did,
            });
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SidecarError {
    #[error("no DecisionLedgerEntry found for transition")]
    MissingLedgerEntry,

    #[error("DecisionLedgerEntry DID mismatch: expected {expected}, got {actual}")]
    DidMismatch { expected: String, actual: String },
}
