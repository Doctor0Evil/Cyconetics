#[derive(Clone, Debug)]
pub struct NeurorightsProfile {
    pub id: String,
    pub version: String,
    pub anchor: String,
}

impl NeurorightsProfile {
    pub fn current() -> Self {
        Self {
            id: super::NEURORIGHTS_POLICY_ID.to_string(),
            version: super::NEURORIGHTS_POLICY_VERSION.to_string(),
            anchor: super::NEURORIGHTS_POLICY_ANCHOR.to_string(),
        }
    }
}
