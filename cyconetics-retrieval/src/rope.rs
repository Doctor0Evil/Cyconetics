use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::retrieval::{PromptEnvelope, RetrievalDomain, KsrBand};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralRopeSegment {
    pub segment_id: Uuid,
    pub envelope: PromptEnvelope,
    pub ksrdelta: KsrBand,
    pub summary: String,
    pub quiz_math_score: f32,   // 0.0–1.0 trust in this segment’s facts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CyberCookbookDomain {
    DcmHciDesign,
    XrGridPolicy,
    RustWiring,
    DidRegistry,
    NeurorightsPolicy,
}

pub fn map_domain(envelope: &PromptEnvelope) -> CyberCookbookDomain {
    match envelope.domain {
        RetrievalDomain::DcmHciDesign => CyberCookbookDomain::DcmHciDesign,
        RetrievalDomain::XrGridPolicy => CyberCookbookDomain::XrGridPolicy,
        RetrievalDomain::RustWiring => CyberCookbookDomain::RustWiring,
        RetrievalDomain::DidRegistry => CyberCookbookDomain::DidRegistry,
        RetrievalDomain::NeurorightsPolicy => CyberCookbookDomain::NeurorightsPolicy,
    }
}
