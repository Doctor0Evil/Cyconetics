use neurorights_core::{NeurorightsBound, NeurorightsEnvelope};
use neurorights_firewall::PromptEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::pipeline::{OrganicCpuConfig, run_organic_cpu_sim};

#[derive(Debug, Deserialize)]
struct SimArgs {
    lif_input: f32,
    lif: OrganicLifConfig,
    wc: OrganicWcConfig,
    ddm: OrganicDdmConfig,
    dt: f32,
    steps: usize,
}

#[derive(Debug, Deserialize)]
struct OrganicLifConfig {
    tau_m: f32,
    v_rest: f32,
    r_m: f32,
    v_thresh: f32,
    v_reset: f32,
}

#[derive(Debug, Deserialize)]
struct OrganicWcConfig {
    tau_e: f32,
    tau_i: f32,
    w_ee: f32,
    w_ei: f32,
    w_ie: f32,
    w_ii: f32,
    p_e: f32,
    p_i: f32,
}

#[derive(Debug, Deserialize)]
struct OrganicDdmConfig {
    drift: f32,
    noise: f32,
    upper: f32,
    lower: f32,
}

#[derive(Debug, Serialize)]
pub struct SimResponse {
    pub last_lif_v: f32,
    pub last_wc_e: f32,
    pub last_wc_i: f32,
    pub decision: String,
}

pub fn handle_organic_cpu_sim(
    bound: NeurorightsBound<PromptEnvelope, NeurorightsEnvelope>,
) -> Result<SimResponse, String> {
    // Enforced at type level: this can only be called with a neurorights-verified envelope. [file:3]

    // Read args from PromptEnvelope (retrieval-only)
    let args: &Value = &bound.payload.args;
    let sim_args: SimArgs = serde_json::from_value(args.clone())
        .map_err(|e| format!("bad args: {e}"))?;

    let cfg = OrganicCpuConfig {
        lif: organic_cpu_math::LifParams {
            tau_m: sim_args.lif.tau_m,
            v_rest: sim_args.lif.v_rest,
            r_m: sim_args.lif.r_m,
            v_thresh: sim_args.lif.v_thresh,
            v_reset: sim_args.lif.v_reset,
        },
        wc: organic_cpu_math::WilsonCowanParams {
            tau_e: sim_args.wc.tau_e,
            tau_i: sim_args.wc.tau_i,
            w_ee: sim_args.wc.w_ee,
            w_ei: sim_args.wc.w_ei,
            w_ie: sim_args.wc.w_ie,
            w_ii: sim_args.wc.w_ii,
            p_e: sim_args.wc.p_e,
            p_i: sim_args.wc.p_i,
        },
        ddm: organic_cpu_math::DdmParams {
            drift: sim_args.ddm.drift,
            noise: sim_args.ddm.noise,
            upper: sim_args.ddm.upper,
            lower: sim_args.ddm.lower,
        },
        dt: sim_args.dt,
        steps: sim_args.steps,
    };

    // Use deterministic or sandboxed noise; here, zeroed for governance-safe default.
    let noise = vec![0.0_f32; cfg.steps];

    let result = run_organic_cpu_sim(&cfg, sim_args.lif_input, &noise);

    Ok(SimResponse {
        last_lif_v: result.last_lif_v,
        last_wc_e: result.last_wc_e,
        last_wc_i: result.last_wc_i,
        decision: format!("{:?}", result.decision),
    })
}
