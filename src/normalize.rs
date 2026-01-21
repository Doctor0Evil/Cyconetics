use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;
use crate::domain::{PromptEnvelope, Intent, SecurityLevel};
use crate::authorship::AuthorshipConfig;
use crate::trace::{make_trace_id, make_args};

/// High-level input from an augmented-citizen / system.
pub struct RawPrompt<'a> {
    pub user_did: &'a str,
    pub text: &'a str,
    pub security_level: SecurityLevel,
    pub intent_hint: Option<Intent>,
    pub extra_args: Option<Value>,
}

/// Deterministic mapping: (RawPrompt + config + time bucket) → PromptEnvelope.
pub fn normalize_prompt(
    raw: RawPrompt,
    authorship_cfg: &AuthorshipConfig,
) -> PromptEnvelope {
    let now = SystemTime::now();
    let ts = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    // Bucket for trace stability (e.g. per day).
    let bucket = ts / 86_400;

    let intent = infer_intent(raw.text, raw.intent_hint);
    let args = make_args(raw.text, raw.extra_args);
    let trace_id = make_trace_id(raw.user_did, raw.text, &bucket.to_string());
    let identity = authorship_cfg.make_identity(raw.user_did, None, None);

    PromptEnvelope {
        trace_id,
        intent,
        args,
        security_level: raw.security_level,
        identity,
        created_at: now,
    }
}

/// Very conservative, deterministic intent inference.
fn infer_intent(text: &str, hint: Option<Intent>) -> Intent {
    if let Some(h) = hint {
        return h;
    }

    // Minimal keyword-based mapping; extend as needed.
    let lower = text.to_ascii_lowercase();

    if lower.contains("policy") || lower.contains("governance") {
        Intent::Governance
    } else if lower.contains("simulate") || lower.contains("simulation") {
        Intent::Simulate
    } else if lower.contains("plan") {
        Intent::Plan
    } else if lower.contains("analyz") {
        Intent::Analyze
    } else if lower.contains("retrieve") || lower.contains("lookup") || lower.contains("fetch") {
        Intent::Retrieve
    } else {
        Intent::Unknown
    }
}
