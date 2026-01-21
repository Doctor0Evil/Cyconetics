use std::{env, fs, path::PathBuf};

fn main() {
    // Path to ALN shards; configurable so Phoenix/San Jolla can point to their own copies.
    let aln_dir = env::var("ALN_DIR").unwrap_or_else(|_| "aln".to_string());

    let ledger_path = PathBuf::from(&aln_dir).join("decision.ledger.entry.v1.aln");
    println!("cargo:rerun-if-changed={}", ledger_path.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let dest_path = out_dir.join("aln_generated.rs");

    let ledger_yaml =
        fs::read_to_string(&ledger_path).expect("failed to read decision.ledger.entry.v1.aln");

    let generated =
        generate_ledger_code(&ledger_yaml).expect("failed to generate ALN-derived Rust code");

    fs::write(&dest_path, generated).expect("failed to write aln_generated.rs");
}

fn generate_ledger_code(yaml: &str) -> Result<String, String> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct LedgerRoot {
        id: String,
        version: String,
        hexstamp: String,
        fields: LedgerFields,
    }

    #[derive(Deserialize)]
    struct LedgerFields {
        roh_at_decision: f64,
        roh_delta: f64,
    }

    let root: LedgerRoot =
        serde_yaml::from_str(yaml).map_err(|e| format!("ledger yaml parse error: {e}"))?;

    let code = format!(
        r#"
/// ALN id for DecisionLedgerEntry shard.
pub const DECISION_LEDGER_ENTRY_ID: &str = "{id}";
/// Version for DecisionLedgerEntry shard.
pub const DECISION_LEDGER_ENTRY_VERSION: &str = "{version}";
/// Hex-stamp snapshot for DecisionLedgerEntry schema.
pub const DECISION_LEDGER_ENTRY_HEXSTAMP: &str = "{hexstamp}";

/// Maximum allowed risk-of-harm at decision time for any upgrade path.
/// This should align with the global RoH ceiling (0.3) enforced in the safety spine.
pub const LEDGER_ROH_MAX: f64 = 0.30;

/// Default numeric hints from the ALN shard (for tests and CI sanity checks).
pub const LEDGER_DEFAULT_ROH_AT_DECISION: f64 = {roh_at};
pub const LEDGER_DEFAULT_ROH_DELTA: f64 = {roh_delta};
"#,
        id = root.id,
        version = root.version,
        hexstamp = root.hexstamp,
        roh_at = root.fields.roh_at_decision,
        roh_delta = root.fields.roh_delta,
    );

    Ok(code)
}
