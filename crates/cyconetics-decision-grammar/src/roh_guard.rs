use serde::{Deserialize, Serialize};

use crate::LEDGER_ROH_MAX;
use crate::types::{EvidenceBundle, EvidenceError, RoHBound, roh_from_biokarma};

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
    /// If predicted RoH >= LEDGER_ROH_MAX, returns Denied and None for RoHBound<30>.
    pub fn predict_roh(
        host_did: &str,
        upgrade_id: &str,
        evolution_id: &str,
        evidence: &EvidenceBundle,
    ) -> Result<(Self, UpgradeDecision), RoHPredictError> {
        evidence.validate()?;

        let roh = roh_from_biokarma(evidence);
        let ceiling = LEDGER_ROH_MAX as f32;

        let token = if roh <= ceiling {
            RoHBound::<30>::new(roh)
        } else {
            None
        };

        let decision = if roh < ceiling {
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
    Evidence(#[from] EvidenceError),
}
