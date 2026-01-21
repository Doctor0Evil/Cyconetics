#![forbid(unsafe_code)]

// Neurorights policy surface.
mod envelope;
mod profile;
mod bound;
mod sealed;
mod version;

pub use envelope::NeurorightsEnvelope;
pub use profile::NeurorightsProfile;
pub use bound::NeurorightsBound;
pub use sealed::NeurorightsMarkerSealed;
pub use version::{
    NEURORIGHTS_POLICY_ANCHOR,
    NEURORIGHTS_POLICY_ID,
    NEURORIGHTS_POLICY_VERSION,
};

// Decision grammar / RoH surface.
pub mod types;
pub mod roles;
pub mod roh_guard;
pub mod ledger;
pub mod macros;

// Generated from decision.ledger.entry.v1.aln at build time.
include!(concat!(env!("OUT_DIR"), "/aln_generated.rs"));
