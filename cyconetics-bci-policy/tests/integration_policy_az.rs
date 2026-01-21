use chrono::Utc;
use uuid::Uuid;

use cyconetics_bci_core::abstraction::CyconeticsBciDevice;
use cyconetics_bci_core::create::{create_bci_device_driver, DriverModuleRef};
use cyconetics_bci_core::dcm::{
    BackendConfig, BackendKind, ChannelSpec, DeviceCapabilityManifest, Jurisdiction,
    PrivacyLevel, SamplingConstraints, SafetyFlags, SessionConstraints, XrGridBinding,
};
use cyconetics_bci_policy::site::{site_profile_arizona, SiteProfile};

fn synthetic_manifest_az() -> DeviceCapabilityManifest {
    DeviceCapabilityManifest {
        id: Uuid::new_v4(),
        name: "brainflow_synthetic_az".into(),
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
async fn az_site_policy_and_binding() {
    let manifest = synthetic_manifest_az();
    manifest.validate().expect("manifest must be valid");

    let site: SiteProfile = site_profile_arizona();

    let zone_id = "AZ-PHX-XR-EEG-LOWRISK";
    let hazard_level = 2;

    // Policy check at site level.
    site.can_use_device_in_zone(&manifest, zone_id, hazard_level)
        .expect("policy should allow this usage");

    let module_ref = DriverModuleRef {
        name: "brainflow_synthetic_module".into(),
        version: "0.1.0".into(),
    };

    let mut device: CyconeticsBciDevice =
        create_bci_device_driver(manifest, &module_ref).expect("driver creation failed");

    // XR-grid binding at device level.
    device
        .bind_to_zone(zone_id, hazard_level)
        .expect("binding must succeed");

    device
        .bci_stream_start(Some(250))
        .expect("failed to start stream");

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let frames = device.bci_snapshot(64).expect("failed to snapshot");
    assert!(!frames.is_empty());
    assert!(!frames[0].channels.is_empty());

    device.bci_stream_stop().expect("failed to stop stream");
    device.shutdown().expect("failed to shutdown");
}
