use async_trait::async_trait;
use serde_json::json;
use crate::domain::{PromptEnvelope, Metadata, RiskAssessment};
use crate::tools::{ToolAdapter, ToolError};

pub struct DriveReaderAdapter;

#[async_trait]
impl ToolAdapter for DriveReaderAdapter {
    fn name(&self) -> &'static str {
        "drive_reader"
    }

    async fn execute(
        &self,
        _envelope: &PromptEnvelope,
        _metadata: &Metadata,
        _risk: &RiskAssessment,
    ) -> Result<serde_json::Value, ToolError> {
        // Stub: implement safe, read-only drive access here.
        Ok(json!({
            "status": "ok",
            "mode": "read-only",
            "note": "DriveReaderAdapter stub; implement safe retrieval."
        }))
    }
}
