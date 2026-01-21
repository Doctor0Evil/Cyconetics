use std::sync::Arc;
use serde_json::json;
use crate::domain::{
    PromptEnvelope, Metadata, RiskAssessment,
    Intent, SubjectTag, PurposeTag, CodexType, SecurityLevel,
};
use crate::logging::{LogSink, LogEvent};
use crate::tools::{ToolAdapter, ToolError};

/// Central router state.
pub struct CyberRetrievalRouter {
    tools: Vec<Arc<dyn ToolAdapter>>,
    log_sink: Arc<dyn LogSink>,
    risk_threshold: f32, // e.g. 0.3
}

impl CyberRetrievalRouter {
    pub fn new(
        tools: Vec<Arc<dyn ToolAdapter>>,
        log_sink: Arc<dyn LogSink>,
        risk_threshold: f32,
    ) -> Self {
        Self { tools, log_sink, risk_threshold }
    }

    /// Entry point: handle a normalized envelope.
    pub async fn handle(&self, envelope: PromptEnvelope) -> Result<serde_json::Value, ToolError> {
        let metadata = self.derive_metadata(&envelope);
        let risk = self.assess_risk(&envelope, &metadata);

        // Red-flag path: block if above threshold.
        if risk.risk_score >= self.risk_threshold || risk.red_flag {
            let result = json!({
                "status": "blocked",
                "reason": "Risk threshold exceeded",
                "trace_id": envelope.trace_id,
            });

            let event = self.build_log_event(&envelope, &metadata, &risk, Some(&result), "blocked");
            let _ = self.log_sink.append(&event);
            return Err(ToolError::Denied("Risk threshold exceeded".into()));
        }

        // Deterministic tool selection based on intent + subject.
        let tool = self.select_tool(&envelope, &metadata)?;

        let result = tool.execute(&envelope, &metadata, &risk).await?;

        let event = self.build_log_event(&envelope, &metadata, &risk, Some(&result), tool.name());
        let _ = self.log_sink.append(&event);

        Ok(result)
    }

    fn derive_metadata(&self, envelope: &PromptEnvelope) -> Metadata {
        // Deterministic mapping from intent/args to metadata.
        let subject = SubjectTag::Other;
        let purpose = PurposeTag::Other;
        let codex_type = match envelope.intent {
            Intent::Governance => CodexType::PolicyDraft,
            Intent::Analyze | Intent::Retrieve => CodexType::ResearchSpec,
            Intent::Plan | Intent::Simulate => CodexType::DataOnChainRef,
            Intent::Unknown => CodexType::LogEvent,
        };

        // Very conservative defaults.
        Metadata {
            codex_type,
            drive_path: format!(
                "Drive:/Cyber-Retrieval/Logs/{:?}/{}",
                envelope.created_at,
                envelope.trace_id
            ),
            subject,
            purpose,
            has_pii: false,
            bio_risk_flag: false,
            policy_relevant: matches!(envelope.intent, Intent::Governance),
        }
    }

    fn assess_risk(&self, _envelope: &PromptEnvelope, metadata: &Metadata) -> RiskAssessment {
        // High-level, non-procedural risk estimate.
        let mut score: f32 = 0.05;

        if metadata.bio_risk_flag {
            score = 0.25;
        }

        let red_flag = score >= 0.3;

        RiskAssessment {
            risk_score: score,
            red_flag,
            rationale: "High-level conceptual guidance only; no operational bio steps.".into(),
        }
    }

    fn select_tool(
        &self,
        envelope: &PromptEnvelope,
        metadata: &Metadata,
    ) -> Result<Arc<dyn ToolAdapter>, ToolError> {
        // Simple deterministic routing example: extend as needed.
        let target_name = match (envelope.intent, metadata.codex_type) {
            (Intent::Governance, _) => "governance_registry",
            (Intent::Retrieve, _) => "drive_reader",
            (Intent::Analyze, _) => "analysis_engine",
            (Intent::Plan, _) | (Intent::Simulate, _) => "simulation_adapter",
            (Intent::Unknown, _) => "logging_adapter",
        };

        self.tools
            .iter()
            .find(|t| t.name() == target_name)
            .cloned()
            .ok_or_else(|| ToolError::Internal(format!("No tool registered for {}", target_name)))
    }

    fn build_log_event(
        &self,
        envelope: &PromptEnvelope,
        metadata: &Metadata,
        risk: &RiskAssessment,
        result: Option<&serde_json::Value>,
        cmd: &str,
    ) -> LogEvent {
        let params = envelope.args.clone();
        let result_ref = result.map(|r| format!("inline:{}", envelope.trace_id));

        LogEvent {
            trace_id: envelope.trace_id.clone(),
            user_did: envelope.identity.user_did.clone(),
            cmd: cmd.to_string(),
            params,
            result_ref,
            timestamp: envelope.created_at,
            metadata: metadata.clone(),
            risk: risk.clone(),
            authorship: envelope.identity.clone(),
        }
    }
}
