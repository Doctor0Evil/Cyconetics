use serde::{Deserialize, Serialize};

/// High-level decision kinds in the Cyconetics safety spine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionKind {
    Proposed,
    Approved,
    Authorized,
    Rejected,
    Escalated,
    Deferred,
}

/// RoH-bounded token. Type parameter N encodes the integer ceiling in centi-RoH (e.g. 30 -> 0.30).
/// This is integrated with the safety spine so that any state requiring RoH<=0.3
/// must carry RoHBound<30>.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoHBound<const N: u8> {
    /// Risk-of-harm in [0.0, 1.0] but constrained at construction.
    roh: f32,
}

impl<const N: u8> RoHBound<N> {
    pub fn new(roh: f32) -> Option<Self> {
        let ceiling = (N as f32) / 100.0;
        if roh <= ceiling {
            Some(Self { roh })
        } else {
            None
        }
    }

    pub fn value(&self) -> f32 {
        self.roh
    }
}

/// A typed decision record that is pushed into the decision ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub host_did: String,
    pub upgrade_id: String,
    pub evolution_id: String,

    pub kind: DecisionKind,
    pub decided_by_role: String, // e.g., "NeurorightsDecider", "SafetyDecider", "HostSelf"
    pub decided_by_did: String,

    pub predicted_roh: f32,
    pub roh_token: Option<RoHBound<30>>,

    pub timestamp_ms: i64,
    pub evidence_hash: String,
}

/// Static shard schema mirror for DecisionLedgerEntry (ALN-rooted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLedgerEntry {
    pub host_did: String,
    pub upgrade_id: String,
    pub evolution_id: String,

    pub decision_kind: DecisionKind,
    pub decision_record_hash: String,

    pub decided_by_did: String,
    pub decided_by_role: String,

    pub zone_id: String,
    pub scheduler_id: String,

    pub roh_at_decision: f32,
    pub roh_delta: f32,

    pub biokarma_snapshot_id: String,
    pub biomarker_envelope_id: String,

    pub created_at_ms: i64,
    pub hexstamp: String,
}

/// Extended evolution audit record with non-erasable decision hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionAuditRecord {
    pub host_did: String,
    pub upgrade_id: String,
    pub evolution_id: String,

    pub active: bool,
    pub roh_history: Vec<f32>,
    pub decision_record_hashes: Vec<String>, // append-only
}

/// Evidence bundles for prediction functions – must have >=10 elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// For example: rolling biokarma scores, each in [-1.0, 1.0].
    pub biokarma: Vec<f32>,
    /// Optional additional observables (heart rate variability, EEG quality score, etc.).
    pub observables: Vec<f32>,
}

impl EvidenceBundle {
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.biokarma.len() < 10 {
            return Err(EvidenceError::TooFewBiokarma {
                len: self.biokarma.len(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    #[error("biokarma evidence length {len} < 10 (bundle too small)")]
    TooFewBiokarma { len: usize },
}

/// Economic/biomarker profile for blood token reserves (corridors / sessions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodTokenReserveProfile {
    pub host_did: String,
    pub corridor_id: String,
    pub session_id: String,

    pub max_blood_tokens: u64,
    pub reserved_tokens: u64,
    pub biomarker_envelope_id: String,
}

/// Proof that a spend was made within allowed reserve + RoH constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodSpendProof {
    pub host_did: String,
    pub corridor_id: String,
    pub session_id: String,

    pub spent_tokens: u64,
    pub remaining_tokens: u64,

    pub biomarker_envelope_id: String,
    pub decision_record_hash: String,
    pub roh_bound: RoHBound<30>,
}

/// Simple function mapping a biokarma bundle to predicted RoH.
/// This must only be called after EvidenceBundle::validate().
pub fn roh_from_biokarma(bundle: &EvidenceBundle) -> f32 {
    // Example heuristic: higher absolute biokarma variance -> higher RoH
    let avg = bundle.biokarma.iter().copied().sum::<f32>() / (bundle.biokarma.len() as f32);
    let var = bundle
        .biokarma
        .iter()
        .map(|v| {
            let d = v - avg;
            d * d
        })
        .sum::<f32>()
        / (bundle.biokarma.len() as f32);

    // Normalize variance into [0, 1] band via a soft compression
    (var / 10.0).min(1.0)
}
