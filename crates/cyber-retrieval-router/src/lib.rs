use neurorights_firewall::{NeurorightsBound, NeurorightsEnvelope};
use cyber_retrieval_types::PromptEnvelope;

pub struct CyberRetrievalRouter;

impl CyberRetrievalRouter {
    // Neurorights-bound entry for augmented-citizen flows.
    pub async fn handle_citizen_request(
        &self,
        env: NeurorightsBound<PromptEnvelope, NeurorightsEnvelope>,
    ) -> Result<serde_json::Value, RouterError> {
        let inner = env.inner();

        // From this point on, all downstream tooling is guaranteed to see
        // a neurorights-bound PromptEnvelope with the citizen_v1 profile.
        match inner.intent.as_str() {
            "cyber_retrieval.intent.fetch_record" => {
                self.handle_fetch_record(inner).await
            }
            "cyber_retrieval.intent.plan_action" => {
                self.handle_plan_action(inner).await
            }
            _ => Err(RouterError::UnknownIntent(inner.intent.clone())),
        }
    }

    async fn handle_fetch_record(
        &self,
        env: &PromptEnvelope,
    ) -> Result<serde_json::Value, RouterError> {
        // Normal business logic; env.neurorights_profile is already set.
        Ok(serde_json::json!({ "status": "ok", "trace_id": env.trace_id }))
    }

    async fn handle_plan_action(
        &self,
        env: &PromptEnvelope,
    ) -> Result<serde_json::Value, RouterError> {
        Ok(serde_json::json!({ "status": "planned", "trace_id": env.trace_id }))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RouterError {
    #[error("unknown intent {0}")]
    UnknownIntent(String),
}
