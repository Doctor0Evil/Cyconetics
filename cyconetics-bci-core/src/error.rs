use thiserror::Error;

#[derive(Debug, Error)]
pub enum CyconeticsBciError {
    #[error("Manifest violation: {0}")]
    ManifestViolation(String),

    #[error("Device error: {0}")]
    DeviceError(String),

    #[error("Artifact error: {0}")]
    ArtifactError(String),

    #[error("Signing / verification error: {0}")]
    SigningError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
