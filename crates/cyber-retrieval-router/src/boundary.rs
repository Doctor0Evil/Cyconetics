use neurorights_firewall::{NeurorightsBound, NeurorightsEnvelope, NeurorightsProfile};
use cyber_retrieval_types::{PromptEnvelope, normalize_prompt, Identity, Governance};
use crate::CyberRetrievalRouter;

pub async fn entry_from_http(
    router: &CyberRetrievalRouter,
    raw_text: String,
    user_did: String,
    aln: String,
    bostrom_address: String,
) -> Result<serde_json::Value, crate::RouterError> {
    let identity = Identity { user_did, aln, bostrom_address };
    let governance = Governance {
        eibon_label: "Eibon:Experimental".into(),
        policy_scope: "Cyber-Retrieval.NeuroFirewall".into(),
        jurisdiction: "phoenix-az-us".into(),
    };

    let mut env = normalize_prompt(
        &raw_text,
        identity,
        governance,
        "did:web:cybercore-brain.org#neurorights",
    );

    // If needed, adjust profile anchor before binding.
    env.neurorights_profile = NeurorightsProfile::citizen_v1(
        "did:web:cybercore-brain.org#neurorights",
    );

    let bound: NeurorightsBound<PromptEnvelope, NeurorightsEnvelope> =
        NeurorightsBound::new(env);

    router.handle_citizen_request(bound).await
}
