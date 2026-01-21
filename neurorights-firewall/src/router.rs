use neurorights_core::{NeurorightsBound, NeurorightsEnvelope, NeurorightsProfile};

#[derive(Clone, Debug)]
pub struct PromptEnvelope {
    pub trace_id: String,
    pub intent: String,          // integrate with your actual Intent enum
    pub args: serde_json::Value,
    pub security_level: String,  // integrate with real type
    pub identity: String,        // DID / ALN / Bostrom identity
    pub provenance: String,
    pub governance: String,
    pub neurorights_profile: NeurorightsProfile,
}

/// Construct a bound envelope from a raw `PromptEnvelope`.
/// This is the only allowed entry path for router handlers.
pub fn wrap_prompt(env: PromptEnvelope) -> NeurorightsBound<PromptEnvelope, NeurorightsEnvelope> {
    let profile = &env.neurorights_profile;

    assert_eq!(
        profile.id,
        neurorights_core::NEURORIGHTS_POLICY_ID,
        "PromptEnvelope neurorights_profile.id mismatch"
    );
    assert_eq!(
        profile.version,
        neurorights_core::NEURORIGHTS_POLICY_VERSION,
        "PromptEnvelope neurorights_profile.version mismatch"
    );

    let envelope = NeurorightsEnvelope::compiled();
    NeurorightsBound::new(env, envelope)
}
