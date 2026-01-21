use async_trait::async_trait;
use serde_json::Value;
use crate::domain::{PromptEnvelope, Metadata, RiskAssessment};

/// Trait for any tool adapter (drive, registry, chain, etc.).
#[async_trait]
pub trait ToolAdapter: Send + Sync {
    fn name(&self) -> &'static str;

    async fn execute(
        &self,
        envelope: &PromptEnvelope,
        metadata: &Metadata,
        risk: &RiskAssessment,
    ) -> Result<Value, ToolError>;
}

#[derive(Debug)]
pub enum ToolError {
    Denied(String),
    Internal(String),
}
