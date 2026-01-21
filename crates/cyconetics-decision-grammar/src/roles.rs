use serde::{Deserialize, Serialize};

use crate::types::{DecisionKind, DecisionRecord, RoHBound};

/// Base identity/host types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostIdentity {
    pub host_did: String,
    pub aln: String,
    pub bostrom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeContext {
    pub upgrade_id: String,
    pub evolution_id: String,
    pub zone_id: String,
    pub scheduler_id: String,
}

/// Role marker traits.
/// These should be implemented only in trusted crates (e.g., via macros).
pub trait NeurorightsDecider {
    fn decide_neurorights(
        &self,
        host: &HostIdentity,
        ctx: &UpgradeContext,
        roh: f32,
        roh_token: Option<RoHBound<30>>,
    ) -> DecisionRecord;
}

pub trait SafetyDecider {
    fn decide_safety(
        &self,
        host: &HostIdentity,
        ctx: &UpgradeContext,
        roh: f32,
        roh_token: Option<RoHBound<30>>,
    ) -> DecisionRecord;
}

pub trait HostSelfDecider {
    fn decide_host_self(
        &self,
        host: &HostIdentity,
        ctx: &UpgradeContext,
        roh: f32,
        roh_token: Option<RoHBound<30>>,
    ) -> DecisionRecord;
}

/// Example default implementations can be generated via macros in macros.rs.
/// Below is a simple wrapper used as a placeholder.
pub struct HostSelf;

impl HostSelfDecider for HostSelf {
    fn decide_host_self(
        &self,
        host: &HostIdentity,
        ctx: &UpgradeContext,
        roh: f32,
        roh_token: Option<RoHBound<30>>,
    ) -> DecisionRecord {
        DecisionRecord {
            host_did: host.host_did.clone(),
            upgrade_id: ctx.upgrade_id.clone(),
            evolution_id: ctx.evolution_id.clone(),
            kind: DecisionKind::Rejected, // host veto by default
            decided_by_role: "HostSelf".into(),
            decided_by_did: host.host_did.clone(),
            predicted_roh: roh,
            roh_token,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            evidence_hash: "0xHOST_VETO".into(),
        }
    }
}
