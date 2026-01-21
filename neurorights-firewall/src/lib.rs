#![forbid(unsafe_code)]

pub mod router;
pub mod audit;
pub mod ci_guards;

pub use router::wrap_prompt;
pub use audit::{Authorship, EvidenceStamp};
