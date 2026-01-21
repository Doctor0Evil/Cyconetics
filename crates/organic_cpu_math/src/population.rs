#[derive(Clone, Copy, Debug)]
pub struct WilsonCowanParams {
    pub tau_e: f32,
    pub tau_i: f32,
    pub w_ee: f32,
    pub w_ei: f32,
    pub w_ie: f32,
    pub w_ii: f32,
    pub p_e: f32,
    pub p_i: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct WilsonCowanState {
    pub e: f32,
    pub i: f32,
}

pub fn sigmoid(x: f32, beta: f32, theta: f32) -> f32 {
    // S(x) = 1 / (1 + exp(-β (x - θ)))
    unimplemented!()
}

pub fn wilson_cowan_step(
    state: WilsonCowanState,
    params: WilsonCowanParams,
    beta: f32,
    theta: f32,
    dt: f32,
) -> WilsonCowanState {
    // τ_E dE/dt = -E + S(w_EE E - w_EI I + P_E)
    // τ_I dI/dt = -I + S(w_IE E - w_II I + P_I)
    unimplemented!()
}
