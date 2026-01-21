#![forbid(unsafe_code)]

use uuid::Uuid;

use crate::types::{
    DecisionKind,
    DecisionRecord,
    DecisionLedgerEntry,
    EvidenceBundle,
    EvolutionAuditRecord,
    RoHBound,
};
use crate::roh_guard::{RoHGuardedHostState, UpgradeDecision};
use crate::roles::{HostIdentity, UpgradeContext, HostSelfDecider, HostSelf};
use crate::ledger::{DecisionLedger, MemoryDecisionLedger, SidecarGuard};
use crate::macros::{scheduler_policy, evolutiongraph};

use cybernetic_cookbook::website::{PageBlueprint, WebsiteAsset, commit_page_blueprint};

/// Phoenix → San Jolla upgrade descriptor.
#[derive(Clone, Debug)]
pub struct PhoenixToSanJollaUpgrade {
    pub upgrade_id: Uuid,
    pub evolution_id: Uuid,
    pub description: String,
    /// RoH cost estimate per session for this upgrade.
    pub roh_delta: f32,
}

impl PhoenixToSanJollaUpgrade {
    pub fn new(description: &str, roh_delta: f32) -> Self {
        Self {
            upgrade_id: Uuid::new_v4(),
            evolution_id: Uuid::new_v4(),
            description: description.to_owned(),
            roh_delta,
        }
    }
}

/// Construct a RoHGuardedHostState for a Phoenix host.
pub fn mk_phx_host_state(current_roh: f32) -> RoHGuardedHostState {
    RoHGuardedHostState {
        host_did: "did:bostrom:phx-host-001".into(),
        upgrade_id: String::new(),
        evolution_id: String::new(),
        last_observed_roh: current_roh,
        predicted_roh: current_roh,
        roh_token: None,
    }
}

/// Simple helper: compute predicted RoH from delta and try to bind RoHBound<30>.
fn predict_roh_for_task(
    state: &RoHGuardedHostState,
    roh_delta: f32,
) -> (f32, Option<RoHBound<30>>) {
    let predicted = (state.last_observed_roh + roh_delta).min(1.0);
    let token = RoHBound::<30>::new(predicted);
    (predicted, token)
}

/// Host/zone policy state machine for Phoenix → San Jolla.
/// Only allows Authorize/Approve if predicted_roh < 0.30 and a RoHBound<30> exists.
#[scheduler_policy(host = "Phoenix", zone = "XR-ZONE-CA-SJO")]
pub struct PhoenixToSanJollaPolicy;

/// Static evolution graph – build will fail if unsafe paths are added once the
/// macro is fully extended.
evolutiongraph! {
    evolution PhoenixSanJollaBCIPath {
        node Proposed;
        node SafetyReviewed;
        node Approved;

        edge Proposed      -> SafetyReviewed { kind: Escalated, roh_delta: 0.00 }
        edge SafetyReviewed-> Approved       { kind: Authorized, roh_delta: 0.05 }
    }
}

/// End-to-end Phoenix → San Jolla upgrade flow.
pub fn run_phx_sjo_upgrade_flow(
    website_asset: &mut WebsiteAsset,
) -> Result<(), String> {
    // 1. Initial Phoenix host state
    let mut host_state = mk_phx_host_state(0.18);

    // 2. Proposed upgrade descriptor
    let upgrade = PhoenixToSanJollaUpgrade::new(
        "Enable San Jolla CA-mode lab-grid profile for host.",
        0.05,
    );

    host_state.upgrade_id = upgrade.upgrade_id.to_string();
    host_state.evolution_id = upgrade.evolution_id.to_string();

    // 3. Predict RoH and try to bind RoHBound<30>
    let (predicted_roh, roh_token) = predict_roh_for_task(&host_state, upgrade.roh_delta);
    host_state.predicted_roh = predicted_roh;
    host_state.roh_token = roh_token;

    // If predicted RoH ≥ 0.30, deny by construction.
    if predicted_roh >= 0.30 || host_state.roh_token.is_none() {
        return Err(format!(
            "REJECTED: predicted RoH {:.3} would exceed 0.30 ceiling",
            predicted_roh
        ));
    }

    // 4. Apply scheduler policy (Phoenix → San Jolla)
    let policy = PhoenixToSanJollaPolicy;
    let decision = policy.decide(&host_state);

    if decision != UpgradeDecision::Approved {
        return Err(format!(
            "REJECTED BY POLICY: PhoenixToSanJollaPolicy denied (predicted RoH {:.3})",
            predicted_roh
        ));
    }

    // 5. HostSelf veto surface – host can still reject.
    let host_identity = HostIdentity {
        host_did: host_state.host_did.clone(),
        aln: "aln:phoenix-grid".into(),
        bostrom: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into(),
    };
    let ctx = UpgradeContext {
        upgrade_id: host_state.upgrade_id.clone(),
        evolution_id: host_state.evolution_id.clone(),
        zone_id: "XR-ZONE-CA-SJO".into(),
        scheduler_id: "PHX-SJO-ROUTER-01".into(),
    };

    let host_self = HostSelf;
    let host_decision_record =
        host_self.decide_host_self(&host_identity, &ctx, predicted_roh, host_state.roh_token);

    if matches!(host_decision_record.kind, DecisionKind::Rejected | DecisionKind::Deferred) {
        return Err(format!(
            "HOST-SELF VETO: {:?}",
            host_decision_record.kind
        ));
    }

    // 6. Build DecisionRecord for Proposed -> Approved
    let decision_record = DecisionRecord {
        host_did: host_state.host_did.clone(),
        upgrade_id: host_state.upgrade_id.clone(),
        evolution_id: host_state.evolution_id.clone(),
        kind: DecisionKind::Approved,
        decided_by_role: "PhoenixToSanJollaPolicy+HostSelf".into(),
        decided_by_did: host_state.host_did.clone(),
        predicted_roh,
        roh_token: host_state.roh_token,
        timestamp_ms: chrono::Utc::now().timestamp_millis(),
        evidence_hash: "0xCYC0-PHX-SJO-ROH30".into(),
    };

    // 7. Construct EvidenceBundle and ledger entry
    let evidence = EvidenceBundle {
        biokarma: vec![0.0; 10], // placeholder; real corridor data goes here
        observables: vec![0.0; 10],
    };
    evidence.validate().map_err(|e| e.to_string())?;

    let ledger_entry = DecisionLedgerEntry {
        host_did: decision_record.host_did.clone(),
        upgrade_id: decision_record.upgrade_id.clone(),
        evolution_id: decision_record.evolution_id.clone(),
        decision_kind: decision_record.kind,
        decision_record_hash: decision_record.evidence_hash.clone(),
        decided_by_did: decision_record.decided_by_did.clone(),
        decided_by_role: decision_record.decided_by_role.clone(),
        zone_id: ctx.zone_id.clone(),
        scheduler_id: ctx.scheduler_id.clone(),
        roh_at_decision: predicted_roh,
        roh_delta: upgrade.roh_delta,
        biokarma_snapshot_id: "BK-SNAPSHOT-PHX-SJO-001".into(),
        biomarker_envelope_id: "BIO-ENV-PHX-SJO-001".into(),
        created_at_ms: decision_record.timestamp_ms,
        hexstamp: "0xCYC0-PHX-SJO-ROH30-LEDGER-COOKBOOK-v1".into(),
    };

    // 8. Append to in-memory ledger and enforce CI/sidecar contract
    let mut ledger = MemoryDecisionLedger::default();
    ledger.append(ledger_entry.clone());

    ledger
        .ensure_transition_has_ledger_entry(
            &decision_record.host_did,
            &decision_record.upgrade_id,
            &decision_record.evolution_id,
            DecisionKind::Approved,
            &decision_record.decided_by_did,
        )
        .map_err(|e| format!("Sidecar guard failed: {e}"))?;

    // 9. Update EvolutionAuditRecord (append-only decision hash history)
    let mut audit = EvolutionAuditRecord {
        host_did: host_state.host_did.clone(),
        upgrade_id: host_state.upgrade_id.clone(),
        evolution_id: host_state.evolution_id.clone(),
        active: true,
        roh_history: vec![host_state.last_observed_roh],
        decision_record_hashes: vec![],
    };
    audit.append_roh(predicted_roh);
    audit.append_decision_hash(decision_record.evidence_hash.clone());

    // 10. Cookbook “page commit” describing the evolution
    let page_blueprint = PageBlueprint {
        id: format!(
            "phx-sjo-{}-{}",
            decision_record.upgrade_id, decision_record.evolution_id
        ),
        path: "/decisions/phx-sjo-upgrade".into(),
        class: "governance-record".into(),
        hexstamp: "0xCYC0-PHX-SJO-ROH30-LEDGER-COOKBOOK-v1".into(),
        knowledge_factor: crate::DEFAULT_KNOWLEDGE_FACTOR,
        risk_of_harm: crate::DEFAULT_RISK_OF_HARM,
        cybostate_factor: crate::DEFAULT_CYBOSTATE_FACTOR,
    };

    commit_page_blueprint(website_asset, page_blueprint);

    Ok(())
}
