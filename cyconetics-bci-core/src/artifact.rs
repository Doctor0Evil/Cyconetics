use std::fs;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};

use crate::error::CyconeticsBciError;

/// Simple local artifact cache abstraction.
///
/// In a full system, this would be backed by sovereign object storage and
/// DID/bostrom-verifiable indices; here we provide a filesystem-based skeleton.
pub struct LocalArtifactCache {
    root: PathBuf,
}

impl LocalArtifactCache {
    /// Initialize cache rooted at `~/.cyconetics/artifacts` or a custom path.
    pub fn new_default() -> Result<Self, CyconeticsBciError> {
        let base = dirs::home_dir()
            .ok_or_else(|| CyconeticsBciError::ConfigError("no home dir".into()))?;
        let root = base.join(".cyconetics").join("artifacts");
        fs::create_dir_all(&root)
            .map_err(|e| CyconeticsBciError::ArtifactError(e.to_string()))?;
        Ok(Self { root })
    }

    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, CyconeticsBciError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)
            .map_err(|e| CyconeticsBciError::ArtifactError(e.to_string()))?;
        Ok(Self { root })
    }

    fn artifact_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.json"))
    }

    pub fn put_json<T: Serialize>(&self, key: &str, value: &T) -> Result<(), CyconeticsBciError> {
        let path = self.artifact_path(key);
        let data = serde_json::to_vec_pretty(value)
            .map_err(|e| CyconeticsBciError::ArtifactError(e.to_string()))?;
        fs::write(path, data).map_err(|e| CyconeticsBciError::ArtifactError(e.to_string()))
    }

    pub fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<T, CyconeticsBciError> {
        let path = self.artifact_path(key);
        let data =
            fs::read(path).map_err(|e| CyconeticsBciError::ArtifactError(e.to_string()))?;
        let value: T = serde_json::from_slice(&data)
            .map_err(|e| CyconeticsBciError::ArtifactError(e.to_string()))?;
        Ok(value)
    }
}

/// Skeleton types for signing; to be wired into DID/bostrom/ALN stack.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignedArtifact<T> {
    pub payload: T,
    /// Signature bytes in hex/base64; semantics defined by external signer.
    pub signature: String,
    /// Public key / DID used.
    pub signer_id: String,
}

pub trait ArtifactSigner {
    fn sign(&self, data: &[u8]) -> Result<(String, String), CyconeticsBciError>;
}

pub trait ArtifactVerifier {
    fn verify(
        &self,
        data: &[u8],
        signature: &str,
        signer_id: &str,
    ) -> Result<(), CyconeticsBciError>;
}

impl<T: Serialize> SignedArtifact<T> {
    pub fn new(
        payload: T,
        signer: &dyn ArtifactSigner,
    ) -> Result<Self, CyconeticsBciError> {
        let data =
            serde_json::to_vec(&payload).map_err(|e| CyconeticsBciError::SigningError(e.to_string()))?;
        let (signature, signer_id) = signer.sign(&data)?;
        Ok(Self {
            payload,
            signature,
            signer_id,
        })
    }
}

impl<T: DeserializeOwned> SignedArtifact<T> {
    pub fn verify(
        &self,
        verifier: &dyn ArtifactVerifier,
    ) -> Result<T, CyconeticsBciError> {
        let data = serde_json::to_vec(&self.payload)
            .map_err(|e| CyconeticsBciError::SigningError(e.to_string()))?;
        verifier.verify(&data, &self.signature, &self.signer_id)?;
        Ok(self.payload.clone())
    }
}
