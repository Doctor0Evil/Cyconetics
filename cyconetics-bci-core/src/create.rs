use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::artifact::{ArtifactSigner, LocalArtifactCache, SignedArtifact};
use crate::dcm::DeviceCapabilityManifest;
use crate::device_layer::BrainFlowDevice;
use crate::error::CyconeticsBciError;
use crate::abstraction::CyconeticsBciDevice;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverModuleRef {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredDriver {
    pub id: Uuid,
    pub manifest_id: Uuid,
    pub module: DriverModuleRef,
    /// DID of creator.
    pub created_by_did: String,
}

pub struct DriverRegistry {
    cache: LocalArtifactCache,
}

impl DriverRegistry {
    pub fn new_default() -> Result<Self, CyconeticsBciError> {
        Ok(Self {
            cache: LocalArtifactCache::new_default()?,
        })
    }

    pub fn register_driver(
        &self,
        signed_manifest: SignedArtifact<DeviceCapabilityManifest>,
        module_ref: DriverModuleRef,
    ) -> Result<RegisteredDriver, CyconeticsBciError> {
        let manifest = signed_manifest.payload;
        manifest.validate()?;

        let driver_id = Uuid::new_v4();
        let registered = RegisteredDriver {
            id: driver_id,
            manifest_id: manifest.id,
            module: module_ref,
            created_by_did: signed_manifest.signer.did.clone(),
        };

        let manifest_key = format!("manifest-{}", manifest.id);
        let registry_key = format!("driver-{}", driver_id);

        self.cache.put_json(&manifest_key, &manifest)?;
        self.cache.put_json(&registry_key, &registered)?;

        Ok(registered)
    }

    pub fn load_manifest(
        &self,
        manifest_id: Uuid,
    ) -> Result<DeviceCapabilityManifest, CyconeticsBciError> {
        let key = format!("manifest-{manifest_id}");
        self.cache.get_json(&key)
    }
}

/// High-level device constructor.
///
/// In a full pipeline, this function is one stage in a larger
/// CI-like process; here it performs the runtime wiring only.
pub fn create_bci_device_driver(
    manifest: DeviceCapabilityManifest,
    _driver_module_ref: &DriverModuleRef,
) -> Result<CyconeticsBciDevice, CyconeticsBciError> {
    let device = BrainFlowDevice::new(manifest)?;
    let device = CyconeticsBciDevice::new(device);
    Ok(device)
}
