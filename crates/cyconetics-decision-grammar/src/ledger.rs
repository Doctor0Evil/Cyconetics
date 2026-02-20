//! DecisionLedger and evidence bundle types for blockchain and audit.
//! All decisions are cryptographically stamped and immutably logged.

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;

/// Evidence bundle: 10-element collection of biomarkers for one decision
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceBiomarkers {
    pub il6_level: f32,           // Inflammatory marker (ng/mL)
    pub crp_level: f32,           // C-reactive protein (mg/L)
    pub cortisol_level: f32,      // Stress hormone (nmol/L)
    pub heart_rate: f32,          // BPM
    pub systolic_bp: f32,         // mmHg
    pub diastolic_bp: f32,        // mmHg
    pub core_temperature: f32,    // °C
    pub glucose_blood: f32,       // mg/dL
    pub lactate_level: f32,       // mmol/L (metabolic stress)
    pub oxygen_saturation: f32,   // %
}

/// EEG corridor state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceEeg {
    pub alpha_power: f32,         // 8–12 Hz relaxation
    pub theta_power: f32,         // 4–8 Hz attention
    pub beta_power: f32,          // 12–30 Hz cognitive
    pub gamma_power: f32,         // 30+ Hz, high activation
    pub eeg_corridor_state: String, // "nominal", "elevated", "critical"
}

/// Heart Rate Variability (HRV) indicator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceHrv {
    pub sdnn: f32,                // Standard deviation NN intervals
    pub rmssd: f32,               // Root mean square of successive differences
    pub hrv_index: f32,           // Composite HRV score
}

/// Complete evidence bundle for one decision
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub biomarkers: EvidenceBiomarkers,
    pub eeg_corridors: EvidenceEeg,
    pub hrv: EvidenceHrv,
    pub timestamp: i64,
    pub zone_id: String,
}

impl EvidenceBundle {
    pub fn new(zone_id: String) -> Self {
        EvidenceBundle {
            biomarkers: EvidenceBiomarkers::default(),
            eeg_corridors: EvidenceEeg::default(),
            hrv: EvidenceHrv::default(),
            timestamp: Utc::now().timestamp(),
            zone_id,
        }
    }

    /// Compute SHA256 hash of this bundle for blockchain stamping
    pub fn hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Check if any biomarker is critically high
    pub fn has_critical_biomarker(&self) -> bool {
        let b = &self.biomarkers;
        b.il6_level > 50.0          // Severe inflammation
            || b.cortisol_level > 500.0  // Extreme stress
            || b.heart_rate > 140.0  // Tachycardia
            || b.systolic_bp > 170.0 // Hypertensive crisis
            || b.core_temperature > 39.5 // Fever
            || b.glucose_blood < 70.0 || b.glucose_blood > 400.0 // Dangerous glucose
    }
}

/// Blood token spend proof (for CSP/Blood-linked decisions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodSpendProof {
    pub blood_tokens_spent: f32,
    pub blood_reserve_remaining: f32,
    pub homeostasis_protected: bool,  // true if reserved_for_homeostasis intact
    pub evidence_hash: String,
}

/// Entry in the decision ledger (blockchain shard)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLedgerEntry {
    pub key: DecisionLedgerKey,
    pub final_decision: DecisionKind,
    pub roh_band: RoHBand,
    pub evidence_bundle: EvidenceBundle,
    pub blood_coupling: Option<BloodSpendProof>,
    pub incident_flags: bool,           // true if organism distress detected (for BFC)
    pub created_at: i64,
    pub ledger_entry_hash: String,
}

impl DecisionLedgerEntry {
    pub fn new(
        key: DecisionLedgerKey,
        decision: DecisionKind,
        roh: RoHBand,
        evidence: EvidenceBundle,
    ) -> Self {
        let entry = DecisionLedgerEntry {
            key: key.clone(),
            final_decision: decision,
            roh_band: roh,
            evidence_bundle: evidence,
            blood_coupling: None,
            incident_flags: false,
            created_at: Utc::now().timestamp(),
            ledger_entry_hash: String::new(),
        };
        // Compute and set hash
        let mut entry = entry;
        entry.ledger_entry_hash = entry.compute_hash();
        entry
    }

    /// Compute SHA256 hash of this entire entry
    pub fn compute_hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Serialize for ALN shard commit
    pub fn to_aln_shard(&self) -> ALNDecisionLedgerV1 {
        ALNDecisionLedgerV1 {
            ledger_key: format!("{}", self.key),
            decision_verb: format!("{}", self.final_decision),
            roh_hex: format!("{:02X}{:02X}{:02X}", self.roh_band.knowledge, self.roh_band.social, self.roh_band.risk),
            evidence_hash: self.evidence_bundle.hash(),
            entry_hash: self.ledger_entry_hash.clone(),
            timestamp: self.created_at,
        }
    }
}

/// ALN shard representation of DecisionLedgerEntry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ALNDecisionLedgerV1 {
    pub ledger_key: String,      // host_did:upgrade_id:evolution_id
    pub decision_verb: String,   // "Approve", "Reject", etc.
    pub roh_hex: String,         // KSR band as hex
    pub evidence_hash: String,   // SHA256 of evidence bundle
    pub entry_hash: String,      // SHA256 of entire entry
    pub timestamp: i64,
}

// Re-export types needed by this module
use crate::types::{DecisionKind, DecisionLedgerKey, RoHBand};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_bundle_hash() {
        let bundle = EvidenceBundle::new("zone-phoenix-west".to_string());
        let hash = bundle.hash();
        assert_eq!(hash.len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_critical_biomarker_detection() {
        let mut bundle = EvidenceBundle::new("test-zone".to_string());
        bundle.biomarkers.il6_level = 60.0; // Critical
        assert!(bundle.has_critical_biomarker());

        let mut bundle2 = EvidenceBundle::new("test-zone".to_string());
        bundle2.biomarkers.il6_level = 10.0; // Normal
        assert!(!bundle2.has_critical_biomarker());
    }

    #[test]
    fn test_decision_ledger_entry() {
        let key = DecisionLedgerKey {
            host_did: "test-host".to_string(),
            upgrade_id: "upgrade-001".to_string(),
            evolution_id: "evo-001".to_string(),
        };
        let bundle = EvidenceBundle::new("test-zone".to_string());
        let entry = DecisionLedgerEntry::new(
            key,
            DecisionKind::Approve,
            RoHBand::default(),
            bundle,
        );
        assert!(!entry.ledger_entry_hash.is_empty());
    }
}
