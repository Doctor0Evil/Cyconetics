use crate::LEDGER_ROH_MAX;
use crate::types::{EvidenceBundle, RoHBound, roh_from_biokarma};

impl RoHGuardedHostState {
    pub fn predict_roh(
        host_did: &str,
        upgrade_id: &str,
        evolution_id: &str,
        evidence: &EvidenceBundle,
    ) -> Result<(Self, UpgradeDecision), RoHPredictError> {
        evidence.validate()?;
        let roh = roh_from_biokarma(evidence);

        let token = if roh <= LEDGER_ROH_MAX as f32 {
            RoHBound::<30>::new(roh)
        } else {
            None
        };

        let decision = if roh < LEDGER_ROH_MAX as f32 {
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
