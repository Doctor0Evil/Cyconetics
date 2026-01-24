use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalDomain {
    DcmHciDesign,
    XrGridPolicy,
    NeurorightsPolicy,
    RustWiring,
    DidRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalKind {
    RetrieveKnowledge,
    ThreatScan,
    RetrievePolicy,
    RetrieveDcmHci,
    NeuralRopeResearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceClass {
    StandardsSpec,
    ClinicalGuideline,
    GovernmentPolicy,
    RustCrateDocs,
    SovereignRegistryDoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalLimits {
    pub max_recursion_depth: u8,
    pub max_parallel_queries: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrZoneRef {
    pub zone_id: String,        // e.g. "XR-ZONE-AZ-PHX-1"
    pub jurisdiction: String,   // e.g. "US-AZ"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsrBand {
    pub k: u8,  // knowledge, 0x00–0xFF
    pub s: u8,  // social impact
    pub r: u8,  // risk of harm, must respect RoH ≤ 0x4C (~0.3)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedCodeActions {
    pub may_emit_rust_literals: bool,
    pub may_emit_manifests: bool,
    pub may_emit_policies: bool,
    pub may_touch_ffi: bool,    // always false for AI
    pub may_touch_io: bool,     // always false for AI
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEnvelope {
    pub id: Uuid,
    pub kind: RetrievalKind,
    pub domain: RetrievalDomain,
    pub xrzone: XrZoneRef,
    pub source_classes: Vec<SourceClass>,
    pub limits: RetrievalLimits,
    pub ksrestimate: KsrBand,
    pub allowed_code_actions: AllowedCodeActions,
}
