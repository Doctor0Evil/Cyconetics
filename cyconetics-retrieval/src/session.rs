use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rope::NeuralRopeSegment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RopeSegmentKind {
    ActiveRetrieval,
    SummaryCooldown,
    PolicyCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: Uuid,
    pub segments: Vec<(NeuralRopeSegment, RopeSegmentKind)>,
    pub accumulated_risk: f32,
    pub high_r_segments: u8,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            session_id: Uuid::new_v4(),
            segments: Vec::new(),
            accumulated_risk: 0.0,
            high_r_segments: 0,
        }
    }

    pub fn can_accept_active(&self, next_r_band: u8) -> bool {
        let r_norm = (next_r_band as f32) / 255.0;
        let projected = self.accumulated_risk + r_norm;
        projected < 0.3 && self.high_r_segments < 4
    }

    pub fn push_active(&mut self, seg: NeuralRopeSegment) {
        let r_norm = (seg.ksrdelta.r as f32) / 255.0;
        if r_norm > 0.20 {
            self.high_r_segments += 1;
        }
        self.accumulated_risk += r_norm;
        self.segments.push((seg, RopeSegmentKind::ActiveRetrieval));
    }

    pub fn push_cooldown(&mut self, text: &str) {
        let cooldown = NeuralRopeSegment {
            segment_id: Uuid::new_v4(),
            envelope: self
                .segments
                .last()
                .map(|(s, _)| s.envelope.clone())
                .expect("cooldown needs an initial envelope"),
            ksrdelta: crate::retrieval::KsrBand { k: 0x10, s: 0x60, r: 0x05 },
            summary: text.to_string(),
            quiz_math_score: 1.0,
        };
        self.accumulated_risk *= 0.8;
        self.segments.push((cooldown, RopeSegmentKind::SummaryCooldown));
    }
}
