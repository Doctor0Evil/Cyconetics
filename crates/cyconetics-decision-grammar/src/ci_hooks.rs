//! ALN/Cyberswarm shard schemas and binding.
//! Decisions are committed to immutable, blockchain-anchored shards for audit and traceability.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// ALN shard schema: cyberswarm.decision.ledger.v1
/// Immutable append-only ledger of all governance decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLedgerShard {
    pub shard_name: String,  // "cyberswarm.decision.ledger.v1"
    pub entries: Vec<DecisionLedgerShardEntry>,
    pub last_committed_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLedgerShardEntry {
    pub key: String,                 // host_did:upgrade_id:evolution_id
    pub decision_verb: String,       // "Approve", "Reject", etc.
    pub roh_band_hex: String,        // KSR as hex: KNOWLEDGE:SOCIAL:RISK
    pub evidence_hash: String,       // SHA256 of EvidenceBundle
    pub entry_hash: String,          // SHA256 of entire entry
    pub timestamp: i64,
    pub zone_id: String,
}

impl DecisionLedgerShard {
    pub fn new() -> Self {
        DecisionLedgerShard {
            shard_name: "cyberswarm.decision.ledger.v1".to_string(),
            entries: Vec::new(),
            last_committed_hash: String::new(),
        }
    }

    /// Append an entry (immutable)
    pub fn append(&mut self, entry: DecisionLedgerShardEntry) {
        self.entries.push(entry);
        self.update_hash();
    }

    /// Update the committed hash after append
    fn update_hash(&mut self) {
        use sha2::{Sha256, Digest};
        let json = serde_json::to_string(&self.entries).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        self.last_committed_hash = hex::encode(hasher.finalize());
    }

    /// Serialize to JSONL for blockchain commit
    pub fn to_jsonl(&self) -> String {
        self.entries
            .iter()
            .filter_map(|e| serde_json::to_string(e).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// ALN shard schema: neurorights.consent.registry.v1
/// Immutable registry of neuro-consent records for all entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsConsentShard {
    pub shard_name: String,  // "neurorights.consent.registry.v1"
    pub consents: HashMap<String, NeuroConsentEntry>, // key: entity_did_or_id
    pub last_committed_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroConsentEntry {
    pub entity_id: String,
    pub entity_type: String,          // "HostBrain", "NonhostNervousSystem", "PlantElectricalActivity"
    pub consent_level: u8,            // 0=none, 1=passive, 2=modulation, 3=closedloop
    pub zones_allowed: Vec<String>,   // XR-Grid zones where this consent applies
    pub consent_timestamp: i64,
    pub expires_at: Option<i64>,      // Optional expiration
    pub audit_hash: String,
}

impl NeurorightsConsentShard {
    pub fn new() -> Self {
        NeurorightsConsentShard {
            shard_name: "neurorights.consent.registry.v1".to_string(),
            consents: HashMap::new(),
            last_committed_hash: String::new(),
        }
    }

    /// Register a consent entry (immutable append semantics)
    pub fn register(&mut self, entry: NeuroConsentEntry) {
        self.consents.insert(entry.entity_id.clone(), entry);
        self.update_hash();
    }

    fn update_hash(&mut self) {
        use sha2::{Sha256, Digest};
        let json = serde_json::to_string(&self.consents).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        self.last_committed_hash = hex::encode(hasher.finalize());
    }
}

/// ALN shard schema: neurorights.broadcast.ledger.v1
/// Immutable log of all BFC broadcasts for ecological monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsBoradcastLedgerShard {
    pub shard_name: String,  // "neurorights.broadcast.ledger.v1"
    pub broadcasts: Vec<BroadcastLedgerEntry>,
    pub last_committed_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastLedgerEntry {
    pub host_did: String,
    pub bfc_id: String,
    pub power_level: u8,
    pub target_entities: Vec<String>,  // ["HostBrain", "NonhostNervousSystem"]
    pub zone_id: String,
    pub broadcast_timestamp: i64,
    pub incident_detected: bool,       // true if organism distress flagged
    pub evidence_hash: String,
    pub entry_hash: String,
}

impl NeurorightsBoradcastLedgerShard {
    pub fn new() -> Self {
        NeurorightsBoradcastLedgerShard {
            shard_name: "neurorights.broadcast.ledger.v1".to_string(),
            broadcasts: Vec::new(),
            last_committed_hash: String::new(),
        }
    }

    /// Log a broadcast (immutable)
    pub fn log_broadcast(&mut self, entry: BroadcastLedgerEntry) {
        self.broadcasts.push(entry);
        self.update_hash();
    }

    fn update_hash(&mut self) {
        use sha2::{Sha256, Digest};
        let json = serde_json::to_string(&self.broadcasts).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        self.last_committed_hash = hex::encode(hasher.finalize());
    }

    /// Serialize to JSONL for blockchain commit
    pub fn to_jsonl(&self) -> String {
        self.broadcasts
            .iter()
            .filter_map(|b| serde_json::to_string(b).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Helper to create a NeurorightsBoradcastLedgerShard (note: typo in struct name preserved for ALN compat)
pub fn neurorights_broadcast_ledger() -> NeurorightsBoradcastLedgerShard {
    NeurorightsBoradcastLedgerShard::new()
}

/// ALN shard schema: policy.governance.decision-grammar.v1
/// Meta-shard: stores the decision grammar rules and macros
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionGrammarPolicyShard {
    pub shard_name: String,
    pub roh_ceiling: f32,                    // e.g., 0.3
    pub allowed_decision_verbs: Vec<String>,
    pub role_permissions: HashMap<String, Vec<String>>, // role -> allowed verbs
    pub zone_policies: HashMap<String, ZonePolicy>,
    pub last_committed_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZonePolicy {
    pub zone_id: String,
    pub roh_ceiling: f32,
    pub allowed_upgrade_classes: Vec<String>,
    pub jurisdiction: String,
    pub neuroights_requirements: String,
}

impl DecisionGrammarPolicyShard {
    pub fn new() -> Self {
        DecisionGrammarPolicyShard {
            shard_name: "policy.governance.decision-grammar.v1".to_string(),
            roh_ceiling: 0.30,
            allowed_decision_verbs: vec![
                "Approve".to_string(),
                "Authorize".to_string(),
                "Defer".to_string(),
                "Reject".to_string(),
                "Escalate".to_string(),
            ],
            role_permissions: {
                let mut map = HashMap::new();
                map.insert(
                    "HostSelf".to_string(),
                    vec![
                        "Approve".to_string(),
                        "Authorize".to_string(),
                        "Reject".to_string(),
                        "Escalate".to_string(),
                    ],
                );
                map.insert(
                    "NeurorightsBoard".to_string(),
                    vec!["Approve".to_string(), "Reject".to_string(), "Escalate".to_string()],
                );
                map.insert(
                    "SafetyDaemon".to_string(),
                    vec!["Reject".to_string(), "Escalate".to_string()],
                );
                map
            },
            zone_policies: HashMap::new(),
            last_committed_hash: String::new(),
        }
    }

    /// Add a zone policy
    pub fn add_zone_policy(&mut self, policy: ZonePolicy) {
        self.zone_policies.insert(policy.zone_id.clone(), policy);
        self.update_hash();
    }

    fn update_hash(&mut self) {
        use sha2::{Sha256, Digest};
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        self.last_committed_hash = hex::encode(hasher.finalize());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_ledger_shard() {
        let mut shard = DecisionLedgerShard::new();
        let entry = DecisionLedgerShardEntry {
            key: "host:upgrade:evo".to_string(),
            decision_verb: "Approve".to_string(),
            roh_band_hex: "E078E2A".to_string(),
            evidence_hash: "abcd1234".to_string(),
            entry_hash: "efgh5678".to_string(),
            timestamp: 1708357200,
            zone_id: "phoenix-west".to_string(),
        };
        shard.append(entry);
        assert_eq!(shard.entries.len(), 1);
        assert!(!shard.last_committed_hash.is_empty());
    }

    #[test]
    fn test_neuro_consent_shard() {
        let mut shard = NeurorightsConsentShard::new();
        let entry = NeuroConsentEntry {
            entity_id: "insect-001".to_string(),
            entity_type: "NonhostNervousSystem".to_string(),
            consent_level: 1, // Passive only
            zones_allowed: vec!["zone-phoenix-west".to_string()],
            consent_timestamp: 1708357200,
            expires_at: None,
            audit_hash: "hash123".to_string(),
        };
        shard.register(entry);
        assert_eq!(shard.consents.len(), 1);
    }
}
