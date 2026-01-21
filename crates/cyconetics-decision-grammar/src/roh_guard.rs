use serde::{Deserialize, Serialize};

use crate::types::{EvidenceBundle, RoHBound, roh_from_biokarma};
use crate::types::DecisionKind;

/// High-level decision outcome for an upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeDecision {
    Approved,
    Denied,
}

/// RoH-guarded state for a host and candidate upgrade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoHGuardedHostState {
    pub host_did: String,
    pub upgrade_id: String,
    pub evolution_id: String,

    pub last_observed_roh: f32,
    pub predicted_roh: f32,
    pub roh_token: Option<RoHBound<30>>,
}

impl RoHGuardedHostState {
    /// Compute predicted RoH from evidence and return decision + possible RoHBound token.
    /// If predicted RoH >= 0.3, returns Denied and None for RoHBound<30>.
    pub fn predict_roh(
        host_did: &str,
        upgrade_id: &str,
        evolution_id: &str,
        evidence: &EvidenceBundle,
    ) -> Result<(Self, UpgradeDecision), RoHPredictError> {
        evidence.validate()?;
        let roh = roh_from_biokarma(evidence);
        let token = RoHBound::<30>::new(roh);

        let decision = if roh < 0.30 {
            UpgradeDecision::Approved
        } else {
            UpgradeDecision::Denied
        };

        Ok((
            RoHGuardedHostState {
                host_did: host_did.into(),
                upgrade_id: upgrade_id.into(),
                evolution_id: evolution_id.into(),
                last_observed_roh: roh,
                predicted_roh: roh,
                roh_token: token,
            },
            decision,
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RoHPredictError {
    #[error(transparent)]
    Evidence(#[from] crate::types::EvidenceError),
}
