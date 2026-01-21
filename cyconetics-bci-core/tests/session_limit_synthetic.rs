use chrono::Utc;
use uuid::Uuid;

use cyconetics_bci_core::abstraction::CyconeticsBciDevice;
use cyconetics_bci_core::create::{create_bci_device_driver, DriverModuleRef};
use cyconetics_bci_core::dcm::{
    BackendConfig, BackendKind, ChannelSpec, DeviceCapabilityManifest, Jurisdiction,
    PrivacyLevel, SamplingConstraints, SafetyFlags, SessionConstraints, XrGridBinding,
};

fn short_session_manifest() -> DeviceCapabilityManifest {
    DeviceCapabilityManifest {
        id: Uuid::new_v4(),
        name: "brainflow_synthetic_short_session".into(),
        version: "0.1.0".into(),
        backend: BackendConfig {
            kind: BackendKind::BrainFlow,
            identifier: (-1).to_string(),
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
            max_duration_secs: 1, // very short to trigger limit in test
            min_rest_secs: 0,
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
        tags: vec!["synthetic".into(), "test".into()],
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn session_duration_limit_enforced() {
    let manifest = short_session_manifest();
    manifest.validate().expect("manifest must be valid");

    let module_ref = DriverModuleRef {
        name: "brainflow_synthetic_module_short".into(),
        version: "0.1.0".into(),
    };

    let mut device: CyconeticsBciDevice =
        create_bci_device_driver(manifest, &module_ref).expect("driver creation failed");

    device
        .bind_to_zone("AZ-PHX-XR-EEG-LOWRISK", 1)
        .expect("binding must succeed");

    device
        .bci_stream_start(Some(250))
        .expect("failed to start stream");

    // Stay within limit: snapshot should succeed.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    device.bci_snapshot(32).expect("snapshot within limit should succeed");

    // Exceed limit: wait beyond 1 second total.
    tokio::time::sleep(std::time::Duration::from_millis(750)).await;

    // Now any further snapshot or restart should fail with a manifest violation.
    let too_long = device.bci_snapshot(32);
    assert!(too_long.is_err(), "expected session limit to be enforced");

    device.bci_stream_stop().ok();
    device.shutdown().ok();
}
