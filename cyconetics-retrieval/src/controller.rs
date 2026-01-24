use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::retrieval::{PromptEnvelope, SourceClass, KsrBand};
use crate::rope::NeuralRopeSegment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedFact {
    pub id: Uuid,
    pub source_url: String,
    pub source_class: SourceClass,
    pub content_hash: [u8; 32],
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalBatchResult {
    pub envelope: PromptEnvelope,
    pub segments: Vec<NeuralRopeSegment>,
}

pub struct RetrievalController {
    pub roh_ceiling: f32, // must correspond to RoH ≤ 0.3
}

impl RetrievalController {
    pub fn new() -> Self {
        Self { roh_ceiling: 0.3 }
    }

    /// Generate heterogeneous queries from a natural-language need.
    pub fn build_diverse_queries(&self, envelope: &PromptEnvelope, need: &str) -> Vec<String> {
        let mut queries = Vec::new();
        match envelope.domain {
            crate::retrieval::RetrievalDomain::DcmHciDesign => {
                queries.push(format!("{} clinical EEG DCM manifest", need));
                queries.push(format!("{} BCI HCI export profile standard", need));
                queries.push(format!("{} IEC safety leakage current EEG", need));
            }
            crate::retrieval::RetrievalDomain::XrGridPolicy => {
                queries.push(format!("{} XR zoning US-AZ policy", need));
                queries.push(format!("{} lab safety OSHA EEG", need));
                queries.push(format!("{} neurorights RoH 0.3 threshold", need));
            }
            _ => {
                queries.push(need.to_string());
            }
        }
        queries.truncate(envelope.limits.max_parallel_queries as usize);
        queries
    }

    /// Enforce source-class diversity at selection time.
    pub fn enforce_source_diversity(
        &self,
        mut facts: Vec<RetrievedFact>,
    ) -> Vec<RetrievedFact> {
        facts.sort_by_key(|f| f.source_class.clone() as u8);
        let mut seen_classes = Vec::new();
        let mut out = Vec::new();
        for f in facts {
            if !seen_classes.contains(&f.source_class) {
                seen_classes.push(f.source_class.clone());
                out.push(f);
            }
        }
        out
    }

    /// Remove duplicate/near-duplicate facts using content hashes.
    pub fn deduplicate(&self, mut facts: Vec<RetrievedFact>) -> Vec<RetrievedFact> {
        facts.sort_by_key(|f| f.content_hash);
        facts.dedup_by(|a, b| a.content_hash == b.content_hash);
        facts
    }

    /// Lightweight quiz_math over a batch of facts to assign KSR deltas.
    pub fn quiz_math_score(&self, facts: &[RetrievedFact]) -> (KsrBand, f32) {
        if facts.is_empty() {
            return (
                KsrBand { k: 0, s: 0x40, r: 0x10 },
                0.0,
            );
        }
        let mut agreement = 0usize;
        for f in facts {
            if f.text.len() > 64 {
                agreement += 1;
            }
        }
        let consistency = agreement as f32 / facts.len() as f32;
        let k = (0xC0 as f32 * consistency) as u8;
        let s = 0x70;
        let r = 0x20;
        (KsrBand { k, s, r }, consistency)
    }

    /// Build a NeuralRope segment from a batch of vetted facts.
    pub fn build_segment(
        &self,
        envelope: PromptEnvelope,
        facts: Vec<RetrievedFact>,
    ) -> RetrievalBatchResult {
        let facts = self.deduplicate(self.enforce_source_diversity(facts));
        let (ksr_delta, quiz_score) = self.quiz_math_score(&facts);
        let summary = format!(
            "Batch of {} facts; quiz_math consistency {:.2}",
            facts.len(),
            quiz_score
        );

        let segment = NeuralRopeSegment {
            segment_id: Uuid::new_v4(),
            envelope,
            ksrdelta: ksr_delta,
            summary,
            quiz_math_score: quiz_score,
        };

        RetrievalBatchResult {
            envelope: segment.envelope.clone(),
            segments: vec![segment],
        }
    }
}
