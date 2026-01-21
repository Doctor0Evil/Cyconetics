#![forbid(unsafe_code)]

pub mod types;
pub mod roles;
pub mod roh_guard;
pub mod ledger;
pub mod macros;

include!(concat!(env!("OUT_DIR"), "/aln_generated.rs"));
