use organic_cpu_math::{
    LifParams, LifState, lif_step,
    WilsonCowanParams, WilsonCowanState, wilson_cowan_step,
    DdmParams, DdmState, ddm_step, DecisionOutcome,
};

#[derive(Clone, Debug)]
pub struct OrganicCpuConfig {
    pub lif: LifParams,
    pub wc: WilsonCowanParams,
    pub ddm: DdmParams,
    pub dt: f32,
    pub steps: usize,
}

#[derive(Clone, Debug)]
pub struct OrganicCpuResult {
    pub last_lif_v: f32,
    pub last_wc_e: f32,
    pub last_wc_i: f32,
    pub decision: DecisionOutcome,
}

pub fn run_organic_cpu_sim(
    cfg: &OrganicCpuConfig,
    lif_input: f32,
    noise_samples: &[f32],
) -> OrganicCpuResult {
    let mut lif_state = LifState { v: cfg.lif.v_rest };
    let mut wc_state = WilsonCowanState { e: 0.0, i: 0.0 };
    let mut ddm_state = DdmState { x: 0.0, decided: false };
    let mut decision = DecisionOutcome::Undecided;

    for t in 0..cfg.steps {
        let (_fired_state, _spike) = lif_step(lif_state, cfg.lif, lif_input, cfg.dt);
        lif_state = _fired_state;

        wc_state = wilson_cowan_step(wc_state, cfg.wc, 1.0, 0.0, cfg.dt);

        let noise = noise_samples.get(t).copied().unwrap_or(0.0);
        let (new_ddm, outcome) = ddm_step(ddm_state, cfg.ddm, cfg.dt, noise);
        ddm_state = new_ddm;
        if let DecisionOutcome::Undecided = decision {
            decision = outcome;
        }
    }

    OrganicCpuResult {
        last_lif_v: lif_state.v,
        last_wc_e: wc_state.e,
        last_wc_i: wc_state.i,
        decision,
    }
}
