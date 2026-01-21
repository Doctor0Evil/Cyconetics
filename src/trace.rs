use serde_json::Value;

/// Very simple deterministic trace-id from user_did + prompt + timestamp bucket.
/// Replace with any stable, non-cryptographic scheme your governance approves.
pub fn make_trace_id(user_did: &str, prompt: &str, bucket: &str) -> String {
    let input = format!("{}|{}|{}", user_did, prompt, bucket);
    // Simple, stable folding into hex-like string.
    let mut acc: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        acc = acc.wrapping_mul(1099511628211);
        acc ^= *b as u64;
    }
    format!("0x{:016x}", acc)
}

/// Helper to build a small args object deterministically.
pub fn make_args(prompt: &str, extra: Option<Value>) -> Value {
    let base = serde_json::json!({
        "prompt": prompt,
    });
    match extra {
        Some(v) => {
            serde_json::json!({
                "base": base,
                "extra": v,
            })
        }
        None => base,
    }
}
