use crate::domain::Identity;

/// Static config for this deployment / operator group.
pub struct AuthorshipConfig {
    pub default_aln: Option<String>,
    pub default_bostrom: Option<String>,
}

impl AuthorshipConfig {
    pub fn new(default_aln: Option<String>, default_bostrom: Option<String>) -> Self {
        Self { default_aln, default_bostrom }
    }

    /// Build an Identity from a user DID and optional overrides.
    pub fn make_identity(
        &self,
        user_did: impl Into<String>,
        aln_override: Option<String>,
        bostrom_override: Option<String>,
    ) -> Identity {
        Identity {
            user_did: user_did.into(),
            aln: aln_override.or_else(|| self.default_aln.clone()),
            bostrom_address: bostrom_override.or_else(|| self.default_bostrom.clone()),
        }
    }
}
