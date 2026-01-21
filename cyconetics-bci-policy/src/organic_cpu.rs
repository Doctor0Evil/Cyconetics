use serde::{Deserialize, Serialize};
use crate::hci_profile::HciRiskLevel;
use crate::cybostate::CybostateFactorV1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RohBand {
    Green,
    Yellow,
    Red,
    HardReject,
}

pub fn classify_roh(r: f32) -> RohBand {
    if r >= 0.3 {
        RohBand::HardReject
    } else if r >= 0.25 {
        RohBand::Red
    } else if r >= 0.15 {
        RohBand::Yellow
    } else {
        RohBand::Green
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskClass {
    Exploratory,
    Critical,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub class: TaskClass,
    pub risk_level: HciRiskLevel,
    pub expected_duration_sec: u32,
    pub hci_risk_band: HciRiskBand,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchedulerDecision {
    Authorize,
    Defer,
    Reject,
    Escalate,
}

pub struct OrganicCpuScheduler {
    pub roh_threshold: f32,  // always 0.3
    pub knowledge_factor: f32,
}

impl OrganicCpuScheduler {
    pub fn new() -> Self {
        Self {
            roh_threshold: 0.3,
            knowledge_factor: 0.0,
        }
    }

    pub fn decide(
        &mut self,
        state: CybostateFactorV1,
        task: &ScheduledTask,
    ) -> SchedulerDecision {
        let roh = state.calculate_roh();
        let band = classify_roh(roh);

        if roh >= self.roh_threshold {
            // Fear response: escalate + log to Eibon trail.[file:3]
            return SchedulerDecision::Escalate;
        }

        use HciRiskLevel as HRL;
        use RohBand::*;
        use SchedulerDecision as SD;
        use TaskClass as TC;

        let decision = match (band, task.class, task.risk_level) {
            (Green, TC::Exploratory, _) => SD::Authorize,
            (Green, TC::Critical, HRL::Low) => SD::Authorize,
            (Green, TC::Critical, _) => SD::Defer,

            (Yellow, TC::Exploratory, HRL::Low | HRL::Medium) => SD::Authorize,
            (Yellow, TC::Exploratory, HRL::High) => SD::Defer,
            (Yellow, TC::Critical, _) => SD::Reject,

            (Red, TC::Maintenance, _) => SD::Authorize,
            (Red, _, _) => SD::Reject,

            (HardReject, _, _) => SD::Escalate,
        };

        if matches!(decision, SD::Authorize) {
            self.knowledge_factor += 0.01;
        }

        decision
    }
}
