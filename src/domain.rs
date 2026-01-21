use std::time::{SystemTime};
use serde::{Serialize, Deserialize};

/// High-level intent for a neural syscall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    Retrieve,
    Analyze,
    Plan,
    Simulate,
    Governance,
    Unknown,
}

/// Security level for the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Public,
    Restricted,
    Sensitive,
}

/// High-level codex type for produced assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodexType {
    ResearchSpec,
    PolicyDraft,
    CodeRust,
    DataOnChainRef,
    LogEvent,
}

/// Subject tags for cybernetic research (no wet-lab).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubjectTag {
    Prosthetics,
    NeuralInterfaces,
    Governance,
    Ethics,
    Simulation,
    Other,
}

/// Declared purpose of the retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PurposeTag {
    Education,
    Policy,
    Ethics,
    Simulation,
    Monitoring,
    Other,
}

/// DID / ALN / Bostrom authorship & identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub user_did: String,
    pub aln: Option<String>,
    pub bostrom_address: Option<String>,
}

/// Structured prompt envelope (neural syscall).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEnvelope {
    pub trace_id: String,
    pub intent: Intent,
    pub args: serde_json::Value,
    pub security_level: SecurityLevel,
    pub identity: Identity,
    pub created_at: SystemTime,
}

/// Metadata derived from prompt + router analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub codex_type: CodexType,
    pub drive_path: String,
    pub subject: SubjectTag,
    pub purpose: PurposeTag,
    pub has_pii: bool,
    pub bio_risk_flag: bool,
    pub policy_relevant: bool,
}

/// Risk analysis result (bounded 0.0–1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_score: f32,
    pub red_flag: bool,
    pub rationale: String,
}
