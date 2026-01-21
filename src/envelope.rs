use crate::NeurorightsMarkerSealed;

#[derive(Clone, Debug)]
pub struct NeurorightsEnvelope {
    pub policy_id: &'static str,
    pub policy_version: &'static str,
    pub policy_anchor: &'static str,

    // Invariants mirrored from ALN shard.
    pub max_inner_state_score: u32,
    pub allow_neurocoercion: bool,
}

impl NeurorightsEnvelope {
    pub const fn compiled() -> Self {
        Self {
            policy_id: crate::NEURORIGHTS_POLICY_ID,
            policy_version: crate::NEURORIGHTS_POLICY_VERSION,
            policy_anchor: crate::NEURORIGHTS_POLICY_ANCHOR,
            max_inner_state_score: crate::MAX_INNER_STATE_SCORE,
            allow_neurocoercion: crate::ALLOW_NEUROCOERCION,
        }
    }
}

// Seal the envelope type.
impl NeurorightsMarkerSealed for NeurorightsEnvelope {}
