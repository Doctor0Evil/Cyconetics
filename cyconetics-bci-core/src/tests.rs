use chrono::Utc;
use uuid::Uuid;

use crate::abstraction::CyconeticsBciDevice;
use crate::create::{create_bci_device_driver, DriverModuleRef};
use crate::dcm::{
    BackendConfig, BackendKind, ChannelSpec, DeviceCapabilityManifest, PrivacyLevel,
    SamplingConstraints, SafetyFlags, SessionConstraints, XrGridBinding, Jurisdiction,
};

/// Build a minimal DCM for BrainFlow synthetic board (-1).
fn synthetic_board_manifest() -> DeviceCapabilityManifest {
    DeviceCapabilityManifest {
        id: Uuid::new_v4(),
        name: "brainflow_synthetic".into(),
        version: "0.1.0".into(),
        backend: BackendConfig {
            kind: BackendKind::BrainFlow,
            identifier: (-1).to_string(), // BrainFlow synthetic board ID
        },
        channels: vec![
            ChannelSpec {
                index: 0,
                label: "CH0".into(),
                unit: "uV".into(),
                closed_loop_safe: true,
            },
            ChannelSpec {
                index: 1,
                label: "CH1".into(),
                unit: "uV".into(),
                closed_loop_safe: true,
            },
        ],
        sampling: SamplingConstraints {
            min_hz: 10,
            max_hz: 512,
            default_hz: 250,
        },
        session: SessionConstraints {
            max_duration_secs: 600,
            min_rest_secs: 60,
        },
        jurisdictions: vec![Jurisdiction::UsAz],
        privacy: PrivacyLevel::Medium,
        safety: SafetyFlags {
            can_stimulate: false,
            medical_isolation_rated: false,
        },
        xr_grid: XrGridBinding {
            allowed_zones: vec!["AZ-PHX-XR-EEG-LOWRISK".into()],
            min_hazard_level: 1,
            max_hazard_level: 2,
        },
        tags: vec!["synthetic".into(), "dev".into()],
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn synthetic_board_end_to_end_stream() {
    let manifest = synthetic_board_manifest();
    manifest.validate().expect("manifest must be valid");

    let module_ref = DriverModuleRef {
        name: "brainflow_synthetic_module".into(),
        version: "0.1.0".into(),
    };

    let mut device: CyconeticsBciDevice =
        create_bci_device_driver(manifest, &module_ref).expect("driver creation failed");

    device
        .bci_stream_start(Some(250))
        .expect("failed to start stream");

    // Let BrainFlow synthetic board produce some data.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let frames = device.bci_snapshot(64).expect("failed to snapshot");
    assert!(!frames.is_empty(), "expected at least one frame");
    let frame = &frames[0];
    assert!(
        !frame.channels.is_empty(),
        "expected channels in snapshot frame"
    );

    device.bci_stream_stop().expect("failed to stop stream");
    device.shutdown().expect("failed to shutdown device");
}
