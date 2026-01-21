use serde::{Deserialize, Serialize};
use neurorights_firewall::{NeurorightsProfile, HasNeurorightsProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub user_did: String,
    pub aln: String,
    pub bostrom_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,
    pub trace_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Governance {
    pub eibon_label: String,
    pub policy_scope: String,
    pub jurisdiction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEnvelope {
    pub trace_id: String,
    pub intent: String,
    pub args: serde_json::Value,
    pub security_level: String,
    pub identity: Identity,
    pub provenance: Provenance,
    pub governance: Governance,
    pub neurorights_profile: NeurorightsProfile,
}

impl HasNeurorightsProfile for PromptEnvelope {
    fn set_neurorights_profile(&mut self, profile: NeurorightsProfile) {
        self.neurorights_profile = profile;
    }
}

/// Deterministic normalization into a PromptEnvelope.
pub fn normalize_prompt(
    raw_text: &str,
    identity: Identity,
    governance: Governance,
    anchor: &str,
) -> PromptEnvelope {
    use sha3::{Digest, Sha3_256};

    let mut hasher = Sha3_256::new();
    hasher.update(raw_text.as_bytes());
    hasher.update(identity.user_did.as_bytes());
    let trace_id = format!("hex:{}", hex::encode(hasher.finalize()));

    let profile = NeurorightsProfile::citizen_v1(anchor.to_owned());

    PromptEnvelope {
        trace_id,
        intent: "cyber_retrieval.intent.unknown".into(),
        args: serde_json::Value::Null,
        security_level: "citizen-default".into(),
        identity,
        provenance: Provenance {
            source: "cyber-retrieval.input".into(),
            trace_chain: Vec::new(),
        },
        governance,
        neurorights_profile: profile,
    }
}
