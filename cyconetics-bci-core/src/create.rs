use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::artifact::{ArtifactSigner, LocalArtifactCache, SignedArtifact};
use crate::dcm::DeviceCapabilityManifest;
use crate::device_layer::BrainFlowDevice;
use crate::error::CyconeticsBciError;
use crate::abstraction::CyconeticsBciDevice;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverModuleRef {
    /// Logical name for driver module (e.g., "brainflow_synthetic").
    pub name: String,
    /// Version of the driver implementation (not the manifest).
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredDriver {
    pub id: Uuid,
    pub manifest_id: Uuid,
    pub module: DriverModuleRef,
    pub created_by: String,
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
        signer_id: &str,
    ) -> Result<RegisteredDriver, CyconeticsBciError> {
        // In a full implementation, verify signature and policies.
        if signed_manifest.signer_id != signer_id {
            return Err(CyconeticsBciError::SigningError(
                "signer mismatch".into(),
            ));
        }

        let manifest = signed_manifest.payload;
        manifest.validate()?;

        let driver_id = Uuid::new_v4();
        let registered = RegisteredDriver {
            id: driver_id,
            manifest_id: manifest.id,
            module: module_ref,
            created_by: signed_manifest.signer_id.clone(),
        };

        // Persist both manifest and registry entry as immutable artifacts.
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

/// High-level helper: create a CyconeticsBciDevice from a manifest + module ref.
///
/// This is the concrete realization of:
///   create_bci_device_driver(manifest, driver_module_ref)
pub fn create_bci_device_driver(
    manifest: DeviceCapabilityManifest,
    _driver_module_ref: &DriverModuleRef,
) -> Result<CyconeticsBciDevice, CyconeticsBciError> {
    // In a full pipeline:
    // 1. Run static analysis / linting against template + DCM.
    // 2. Build driver module and sign artifact.
    // 3. Register into sovereign registry.
    // Here we construct the runtime object directly.

    let device = BrainFlowDevice::new(manifest)?;
    let mut device = CyconeticsBciDevice::new(device);
    // Prepare session early; caller can still decide when to start streaming.
    device.bci_stream_stop().ok(); // ensures clean state if needed
    Ok(device)
}
