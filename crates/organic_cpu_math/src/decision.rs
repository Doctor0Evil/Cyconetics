#[derive(Clone, Copy, Debug)]
pub struct DdmParams {
    pub drift: f32,   // μ
    pub noise: f32,   // σ
    pub upper: f32,   // upper bound
    pub lower: f32,   // lower bound
}

#[derive(Clone, Copy, Debug)]
pub struct DdmState {
    pub x: f32,
    pub decided: bool,
}

pub enum DecisionOutcome {
    Upper,
    Lower,
    Undecided,
}

pub fn ddm_step(
    state: DdmState,
    params: DdmParams,
    dt: f32,
    gaussian_noise_sample: f32,
) -> (DdmState, DecisionOutcome) {
    // dx_t = μ dt + σ sqrt(dt) ξ
    // with absorbing bounds at upper/lower
    unimplemented!()
}
