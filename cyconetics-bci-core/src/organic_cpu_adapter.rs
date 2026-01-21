use crate::cybostate::{CybostateFactorV1, CybostateFactorV2};
use crate::dcm::{CybostateSchemaVersion, DeviceCapabilityManifest};

#[derive(Debug, Clone)]
pub enum CybostateView {
    V1(CybostateFactorV1),
    V2(CybostateFactorV2),
}

impl CybostateView {
    pub fn roh(&self) -> f32 {
        match self {
            CybostateView::V1(v1) => v1.calculate_roh(),
            CybostateView::V2(v2) => v2.calculate_roh(),
        }
    }

    pub fn as_v1(&self) -> CybostateFactorV1 {
        match self {
            CybostateView::V1(v1) => *v1,
            CybostateView::V2(v2) => (*v2).into(),
        }
    }
}

/// Binds a device manifest to an appropriate cybostate representation.
pub struct OrganicCpuAdapter {
    pub manifest: DeviceCapabilityManifest,
}

impl OrganicCpuAdapter {
    pub fn new(manifest: DeviceCapabilityManifest) -> Self {
        Self { manifest }
    }

    pub fn interpret_state(&self, raw: &serde_json::Value) -> Option<CybostateView> {
        match self.manifest.cfschema_version {
            CybostateSchemaVersion::V1 => serde_json::from_value(raw.clone())
                .ok()
                .map(CybostateView::V1),
            CybostateSchemaVersion::V2 => serde_json::from_value(raw.clone())
                .ok()
                .map(CybostateView::V2),
        }
    }
}
