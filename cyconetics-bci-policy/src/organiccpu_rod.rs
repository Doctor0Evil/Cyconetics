pub fn decide_with_rod(
    &mut self,
    ts_sec: u64,
    lifeforce_band: LifeforceBandState,
    psychrisk_scalar: f32,      // fear / ROH-like
    pain_corridor_scalar: f32,  // pain corridor
    compute_strain_scalar: f32, // cognitive load
    task: ScheduledTask,
) -> (SchedulerDecision, RodScalar) {
    // 1) Update pain debt and compute ROD
    let inputs = PainDebtInputs {
        tssec: ts_sec,
        lifeforceband: lifeforce_band,
        psychriskscalar: psychrisk_scalar,
        paincorridorscalar: pain_corridor_scalar,
        computestrainscalar: compute_strain_scalar,
    };
    self.paindebt.update(inputs);
    let rod = RodScalar::compute_rod(self.paindebt, self.neurorightsbudgets, self.rodpolicyprofile);

    // 2) Hard vetoes: Lifeforce HardStop or ROD == 1.0
    if matches!(lifeforce_band, LifeforceBandState::HardStop) || (rod.value - RODMAX).abs() <= 1e-6 {
        return (SchedulerDecision::Reject, rod);
    }

    // 3) Compute ROH and classify bands
    let roh_value = /* CybostateFactorV2::calculate_roh(...) */;
    let roh_band  = classify_roh(roh_value);
    let roh_ceiling: f32 = 0.3;

    // 4) High ROD = conservation mode
    let rod_band = self.classify_rod(rod);
    if matches!(rod_band, RodBand::High) {
        return match task.class {
            TaskClass::Maintenance => {
                if roh_value > roh_ceiling {
                    (SchedulerDecision::Escalate, rod)
                } else {
                    (SchedulerDecision::Authorize, rod)
                }
            }
            _ => (SchedulerDecision::Defer, rod),
        };
    }

    // 5) Narrow “training under stress” corridor:
    // allow brief ROH > 0.3 only when ROD is Low and task is time-bounded + recovery-booked.
    if roh_value > roh_ceiling {
        if matches!(rod_band, RodBand::Low) && task.expected_duration_sec <= 30 && self.has_mandatory_recovery_window(&task) {
            return (SchedulerDecision::AuthorizeWithRecovery, rod);
        }
        return (SchedulerDecision::Escalate, rod);
    }

    // 6) Normal operations shaped by ROH × ROD bands
    let decision = match (roh_band, rod_band, task.class) {
        (RohBand::Green, RodBand::Low, _)               => SchedulerDecision::Authorize,
        (RohBand::Green, RodBand::Medium, TaskClass::Critical) => SchedulerDecision::Defer,
        (RohBand::Green, RodBand::Medium, _)            => SchedulerDecision::Authorize,
        (RohBand::Yellow, RodBand::Low, TaskClass::Maintenance) => SchedulerDecision::Authorize,
        (RohBand::Yellow, RodBand::Low, _)              => SchedulerDecision::Defer,
        (RohBand::Yellow, RodBand::Medium, _)           => SchedulerDecision::Defer,
        _                                               => SchedulerDecision::Reject,
    };

    (decision, rod)
}
