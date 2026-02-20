//! Role-based decision authority traits and implementations.
//! Each role can only emit allowed decision verbs for its domain.

use crate::types::*;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Context for a decision: host state, evidence, and authorization chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub host_did: String,
    pub upgrade_id: String,
    pub evolution_id: String,
    pub host_state: RoHGuardedHostState,
    pub current_roh: f32,
    pub predicted_post_roh: f32,
    pub evidence_bundle_hash: Option<String>,
    pub zone_id: String,
    pub decision_timestamp: i64,
}

impl fmt::Display for DecisionContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DecisionContext {{ host={}, upgrade={}, zone={}, current_roh={:.3}, predicted={:.3} }}",
            self.host_did, self.upgrade_id, self.zone_id, self.current_roh, self.predicted_post_roh
        )
    }
}

/// Error types for decision role violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleViolation {
    UnauthorizedVerb(String),
    RoHViolation(f32, f32),             // current, ceiling
    MissingHostVeto,
    HostSelfVetoOverridden,
    InvalidBloodTokenUsage,
    NeuroconsentMissing,
}

impl fmt::Display for RoleViolation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RoleViolation::UnauthorizedVerb(v) => write!(f, "UnauthorizedVerb: {}", v),
            RoleViolation::RoHViolation(curr, ceil) => {
                write!(f, "RoHViolation: {} > {}", curr, ceil)
            },
            RoleViolation::MissingHostVeto => write!(f, "MissingHostVeto"),
            RoleViolation::HostSelfVetoOverridden => write!(f, "HostSelfVetoOverridden"),
            RoleViolation::InvalidBloodTokenUsage => write!(f, "InvalidBloodTokenUsage"),
            RoleViolation::NeuroconsentMissing => write!(f, "NeuroconsentMissing"),
        }
    }
}

/// HostSelf role: the augmented citizen has veto power over all decisions affecting their body/brain
pub trait HostSelfDecider {
    /// Host can ALWAYS reject (structural veto)
    fn reject(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;

    /// Host can override a rejected decision to authorize (but requires explicit DID signature)
    fn override_reject_to_authorize(
        &self,
        ctx: &DecisionContext,
        reason: &str,
    ) -> Result<DecisionRecord, RoleViolation>;

    /// Host can defer for any reason
    fn defer(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;

    /// Host can approve low-risk upgrades (RoH < 0.15)
    fn approve_low_risk(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;

    /// Host can escalate to NeurorightsBoard
    fn escalate_to_neurorights(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;
}

/// NeurorightsDecider: independent neurorights board for protection
pub trait NeurorightsDecider {
    /// Can approve if RoH < 0.2 and no identity-risk present
    fn approve_safe_upgrade(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;

    /// Can reject for any neurorights concern
    fn reject_neurorights_violation(
        &self,
        ctx: &DecisionContext,
        reason: &str,
    ) -> Result<DecisionRecord, RoleViolation>;

    /// Can escalate to GovSafetyOS for complex decisions
    fn escalate_to_safety(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;

    /// Can audit BFC broadcasts for ecological consent
    fn audit_biofield_broadcast(
        &self,
        ctx: &DecisionContext,
        proposal: &BFCBroadcastProposal,
    ) -> Result<DecisionRecord, RoleViolation>;
}

/// SafetyDecider: autonomous safety daemon for hemodynamic/metabolic/thermal constraints
pub trait SafetyDecider {
    /// Can reject if any safety envelope is exceeded
    fn reject_safety_violation(
        &self,
        ctx: &DecisionContext,
        violation: &str,
    ) -> Result<DecisionRecord, RoleViolation>;

    /// Can escalate for human review
    fn escalate_to_humans(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;

    /// Cannot approve (passive gatekeeper only)
    /// But returns Ok(Defer) if all envelopes are clear
    fn defer_pending_other_roles(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;
}

/// GovSafetyOS: governmental/jurisdictional oversight for multi-zone decisions
pub trait GovSafetyDecider {
    /// Can approve for jurisdictional compliance, but only after NeurorightsDecider approves
    fn approve_jurisdiction_compliant(
        &self,
        ctx: &DecisionContext,
        jurisdiction: &str,
    ) -> Result<DecisionRecord, RoleViolation>;

    /// Can reject for policy violations
    fn reject_policy_violation(
        &self,
        ctx: &DecisionContext,
        policy: &str,
    ) -> Result<DecisionRecord, RoleViolation>;

    /// Can escalate to higher authority
    fn escalate_to_executive(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation>;
}

/// Helper: construct a decision record from a role decision
pub fn decision_record_from_role(
    ctx: &DecisionContext,
    kind: DecisionKind,
    ksr: KsrBand,
) -> DecisionRecord {
    DecisionRecord {
        host_did: ctx.host_did.clone(),
        upgrade_id: ctx.upgrade_id.clone(),
        evolution_id: ctx.evolution_id.clone(),
        decision: kind,
        pre_roh: ctx.current_roh,
        post_roh: ctx.predicted_post_roh,
        brain_specs: ctx.host_state.brain_specs.clone(),
        host_budget: ctx.host_state.host_budget.clone(),
        bci_snapshot: ctx.host_state.bci_snapshot.clone(),
        ksr_band: ksr,
        ledger_key: DecisionLedgerKey {
            host_did: ctx.host_did.clone(),
            upgrade_id: ctx.upgrade_id.clone(),
            evolution_id: ctx.evolution_id.clone(),
        },
        timestamp: ctx.decision_timestamp,
    }
}

/// Concrete implementation of HostSelfDecider (template; actual impl is user-provided)
pub struct HostSelfImpl {
    pub host_did: String,
    pub authorized: bool,
}

impl HostSelfDecider for HostSelfImpl {
    fn reject(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation> {
        let record = decision_record_from_role(ctx, DecisionKind::Reject, KsrBand::default());
        Ok(record)
    }

    fn override_reject_to_authorize(
        &self,
        ctx: &DecisionContext,
        _reason: &str,
    ) -> Result<DecisionRecord, RoleViolation> {
        if ctx.predicted_post_roh > 0.3 {
            return Err(RoleViolation::RoHViolation(ctx.predicted_post_roh, 0.3));
        }
        let mut ksr = KsrBand::default();
        ksr.risk = 0x28; // Higher risk for host override
        let record = decision_record_from_role(ctx, DecisionKind::Authorize, ksr);
        Ok(record)
    }

    fn defer(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation> {
        let record = decision_record_from_role(ctx, DecisionKind::Defer, KsrBand::default());
        Ok(record)
    }

    fn approve_low_risk(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation> {
        if ctx.predicted_post_roh > 0.15 {
            return Err(RoleViolation::RoHViolation(ctx.predicted_post_roh, 0.15));
        }
        let record = decision_record_from_role(ctx, DecisionKind::Approve, KsrBand::default());
        Ok(record)
    }

    fn escalate_to_neurorights(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation> {
        let mut ksr = KsrBand::default();
        ksr.risk = 0x2B;
        let record = decision_record_from_role(ctx, DecisionKind::Escalate, ksr);
        Ok(record)
    }
}

/// Concrete implementation of SafetyDecider
pub struct SafetyDaemonImpl {
    pub enabled: bool,
}

impl SafetyDecider for SafetyDaemonImpl {
    fn reject_safety_violation(
        &self,
        ctx: &DecisionContext,
        _violation: &str,
    ) -> Result<DecisionRecord, RoleViolation> {
        let record = decision_record_from_role(ctx, DecisionKind::Reject, KsrBand::default());
        Ok(record)
    }

    fn escalate_to_humans(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation> {
        let mut ksr = KsrBand::default();
        ksr.risk = 0x29;
        let record = decision_record_from_role(ctx, DecisionKind::Escalate, ksr);
        Ok(record)
    }

    fn defer_pending_other_roles(&self, ctx: &DecisionContext) -> Result<DecisionRecord, RoleViolation> {
        let record = decision_record_from_role(ctx, DecisionKind::Defer, KsrBand::default());
        Ok(record)
    }
}
