#[derive(Clone, Copy, Debug)]
pub struct LifParams {
    pub tau_m: f32,
    pub v_rest: f32,
    pub r_m: f32,
    pub v_thresh: f32,
    pub v_reset: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct LifState {
    pub v: f32,
}

pub fn lif_step(
    state: LifState,
    params: LifParams,
    input_current: f32,
    dt: f32,
) -> (LifState, bool) {
    // implements: τ_m dV/dt = -(V - V_rest) + R_m I(t)
    // returns (new_state, fired)
    // body to be filled in repository
    unimplemented!()
}
