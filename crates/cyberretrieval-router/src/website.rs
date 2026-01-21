use neurorights_core::{NeurorightsBound, NeurorightsEnvelope};
use neurorights_firewall::PromptEnvelope;

use cyberretrieval_website_governance::{
    handlers::{handle_website_governance, WebsiteGovArgs, WebsiteGovEnvelope},
    risk::RiskEnvelope,
};

pub fn website_page_route(
    env: NeurorightsBound<PromptEnvelope<WebsiteGovArgs>, NeurorightsEnvelope>,
) -> Result<(), WebsiteRouteError> {
    // Construct default RiskEnvelope for this operation
    let risk = RiskEnvelope::default("0x4F91C7AB39D62E11");

    // ALN-derived governance constraints and RoH ceiling are enforced
    let decision = handle_website_governance(env, risk)?;

    if !decision.allowed {
        return Err(WebsiteRouteError::Denied);
    }

    // At this point, you can call into the Cookbook pipeline to materialize
    // the Markdown spec / page blueprint, still under neurorights envelopes.

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum WebsiteRouteError {
    #[error("website governance decision denied")]
    Denied,

    #[error(transparent)]
    Governance(#[from] cyberretrieval_website_governance::handlers::GovernanceError),
}
