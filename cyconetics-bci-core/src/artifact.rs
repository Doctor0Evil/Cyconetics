use std::fs;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};

use crate::error::CyconeticsBciError;

/// Simple local artifact cache abstraction.
pub struct LocalArtifactCache {
    root: PathBuf,
}

impl LocalArtifactCache {
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

/// DID-like identifier structure. In a real deployment, this would
/// conform to ALN/bostrom DID method specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycDid {
    pub did: String,
    pub public_key: Vec<u8>,
}

/// Signed artifact wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedArtifact<T> {
    pub payload: T,
    pub signature: String,
    pub signer: CycDid,
}

pub trait ArtifactSigner {
    fn sign(&self, data: &[u8]) -> Result<(String, CycDid), CyconeticsBciError>;
}

pub trait ArtifactVerifier {
    fn verify(&self, data: &[u8], signature: &str, signer: &CycDid)
        -> Result<(), CyconeticsBciError>;
}

impl<T: Serialize> SignedArtifact<T> {
    pub fn new(payload: T, signer: &dyn ArtifactSigner) -> Result<Self, CyconeticsBciError> {
        let data =
            serde_json::to_vec(&payload).map_err(|e| CyconeticsBciError::SigningError(e.to_string()))?;
        let (signature, did) = signer.sign(&data)?;
        Ok(Self {
            payload,
            signature,
            signer: did,
        })
    }
}

impl<T: DeserializeOwned + Clone> SignedArtifact<T> {
    pub fn verify(
        &self,
        verifier: &dyn ArtifactVerifier,
    ) -> Result<T, CyconeticsBciError> {
        let data = serde_json::to_vec(&self.payload)
            .map_err(|e| CyconeticsBciError::SigningError(e.to_string()))?;
        verifier.verify(&data, &self.signature, &self.signer)?;
        Ok(self.payload.clone())
    }
}
