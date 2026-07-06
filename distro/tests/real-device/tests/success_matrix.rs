//! KPI-011 real-device success matrix の required device class set を検査します。

use std::collections::BTreeSet;

#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/dispatch.rs"]
mod dispatch;
#[path = "../src/error.rs"]
mod error;
#[path = "../src/evidence.rs"]
mod evidence;
#[path = "../src/redaction.rs"]
mod redaction;
#[path = "../src/success.rs"]
mod success;

use arcrtc_distro_evidence::{
    DistroCommandClass, DistroEnvironmentClass, DistroEvidenceReason,
    DistroEvidenceRecord, DistroLayer, DistroNonClaimScope,
    DistroPlane, DISTRO_COMMAND_ROOT, DISTRO_EVIDENCE_ROOT,
};
use cli::{CliDeviceClass, CliPlatform};
use dispatch::dispatch_kpi_real_device_success_command;
use evidence::{
    RealDeviceClass, RealDeviceEvidenceRecord, RealDeviceInternalCommandClass,
    RealDeviceNetworkClass, RealDevicePlatform, RealDeviceVersionClass,
    REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED,
    REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY,
};
use success::{
    validate_real_device_success_evidence, RealDeviceSuccessValidationError,
    RealDeviceSuccessVerdict,
};

#[derive(Clone, Copy)]
struct SuccessMatrixRow {
    kpi_row_id: &'static str,
    platform: CliPlatform,
    cli_device_class: CliDeviceClass,
    platform_group: &'static str,
    wrapper_command: &'static str,
    device_class: RealDeviceClass,
    internal_command_class: RealDeviceInternalCommandClass,
    runtime_version_class: RealDeviceVersionClass,
    network_class: RealDeviceNetworkClass,
    required_exit_status: i32,
}

fn success_matrix_rows() -> [SuccessMatrixRow; 6] {
    [
        SuccessMatrixRow {
            kpi_row_id: "KPI-011-ANDROID-PHYSICAL",
            platform: CliPlatform::Android,
            cli_device_class: CliDeviceClass::AndroidPhysical,
            platform_group: "Android",
            wrapper_command: "cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-physical",
            device_class: RealDeviceClass::AndroidPhysical,
            internal_command_class: RealDeviceInternalCommandClass::AndroidDevice,
            runtime_version_class: RealDeviceVersionClass::AndroidApiLevel,
            network_class: RealDeviceNetworkClass::LocalUsb,
            required_exit_status: 0,
        },
        SuccessMatrixRow {
            kpi_row_id: "KPI-011-ANDROID-EMULATOR",
            platform: CliPlatform::Android,
            cli_device_class: CliDeviceClass::AndroidEmulator,
            platform_group: "Android",
            wrapper_command: "cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-emulator",
            device_class: RealDeviceClass::AndroidEmulator,
            internal_command_class: RealDeviceInternalCommandClass::AndroidDevice,
            runtime_version_class: RealDeviceVersionClass::AndroidApiLevel,
            network_class: RealDeviceNetworkClass::EmulatorLoopback,
            required_exit_status: 0,
        },
        SuccessMatrixRow {
            kpi_row_id: "KPI-011-IOS-PHYSICAL",
            platform: CliPlatform::Ios,
            cli_device_class: CliDeviceClass::IosPhysical,
            platform_group: "iOS",
            wrapper_command: "cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-physical",
            device_class: RealDeviceClass::IosPhysical,
            internal_command_class: RealDeviceInternalCommandClass::IosDevice,
            runtime_version_class: RealDeviceVersionClass::IosSystemVersion,
            network_class: RealDeviceNetworkClass::LocalUsb,
            required_exit_status: 0,
        },
        SuccessMatrixRow {
            kpi_row_id: "KPI-011-IOS-SIMULATOR",
            platform: CliPlatform::Ios,
            cli_device_class: CliDeviceClass::IosSimulator,
            platform_group: "iOS",
            wrapper_command: "cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-simulator",
            device_class: RealDeviceClass::IosSimulator,
            internal_command_class: RealDeviceInternalCommandClass::IosSimulator,
            runtime_version_class: RealDeviceVersionClass::IosSystemVersion,
            network_class: RealDeviceNetworkClass::SimulatorLoopback,
            required_exit_status: 0,
        },
        SuccessMatrixRow {
            kpi_row_id: "KPI-011-BROWSER-DESKTOP",
            platform: CliPlatform::Browser,
            cli_device_class: CliDeviceClass::DesktopBrowser,
            platform_group: "browser",
            wrapper_command: "cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser",
            device_class: RealDeviceClass::DesktopBrowser,
            internal_command_class: RealDeviceInternalCommandClass::Browser,
            runtime_version_class: RealDeviceVersionClass::BrowserVersion,
            network_class: RealDeviceNetworkClass::LocalBrowser,
            required_exit_status: 0,
        },
        SuccessMatrixRow {
            kpi_row_id: "KPI-011-BROWSER-MOBILE",
            platform: CliPlatform::Browser,
            cli_device_class: CliDeviceClass::MobileBrowser,
            platform_group: "browser",
            wrapper_command: "cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class mobile-browser",
            device_class: RealDeviceClass::MobileBrowser,
            internal_command_class: RealDeviceInternalCommandClass::Browser,
            runtime_version_class: RealDeviceVersionClass::BrowserVersion,
            network_class: RealDeviceNetworkClass::MobileBrowserEmulation,
            required_exit_status: 0,
        },
    ]
}

#[test]
fn kpi_real_device_success_matrix_accepts_required_device_class_set() {
    let rows = success_matrix_rows();
    assert_eq!(rows.len(), 6);

    let mut row_ids = BTreeSet::new();
    let mut platform_groups = BTreeSet::new();

    for row in rows {
        assert!(row_ids.insert(row.kpi_row_id));
        platform_groups.insert(row.platform_group);
        assert_eq!(row.required_exit_status, 0);
        assert!(row
            .wrapper_command
            .starts_with("cargo run --manifest-path tests/real-device/Cargo.toml -- "));
        assert!(row.wrapper_command.contains("--profile reference-local"));

        let dispatch = dispatch_kpi_real_device_success_command(row.platform, row.cli_device_class)
            .expect("required device class must be admitted");
        assert_eq!(dispatch.device_class, row.device_class);
        assert_eq!(dispatch.internal_command_class, row.internal_command_class);
        assert_eq!(dispatch.runtime_version_class, row.runtime_version_class);
        assert_eq!(dispatch.network_class, row.network_class);

        match dispatch.platform {
            RealDevicePlatform::Android | RealDevicePlatform::Ios => {
                assert!(dispatch.platform_command.is_some());
            }
            RealDevicePlatform::Browser => {
                assert!(dispatch.platform_command.is_none());
            }
        }
    }

    assert_eq!(
        platform_groups,
        BTreeSet::from(["Android", "browser", "iOS"])
    );
    let records = rows
        .iter()
        .map(|row| success_record(*row, 0))
        .collect::<Vec<_>>();
    assert_eq!(
        validate_real_device_success_evidence(&records),
        Ok(RealDeviceSuccessVerdict::Pass)
    );
}

#[test]
fn kpi_real_device_success_matrix_rejects_missing_required_row_or_nonzero_exit() {
    let rows = success_matrix_rows();
    let records = rows
        .iter()
        .map(|row| success_record(*row, 0))
        .collect::<Vec<_>>();
    assert_eq!(
        validate_real_device_success_evidence(&records),
        Ok(RealDeviceSuccessVerdict::Pass)
    );

    let missing_row = records[..5].to_vec();
    assert_eq!(
        validate_real_device_success_evidence(&missing_row),
        Err(RealDeviceSuccessValidationError::MissingRequiredRow)
    );

    let mut nonzero_row = records.clone();
    nonzero_row[0] = success_record(rows[0], 2);
    assert_eq!(
        validate_real_device_success_evidence(&nonzero_row),
        Err(RealDeviceSuccessValidationError::NonZeroExitStatus)
    );

    let mut missing_redacted_identifier = records.clone();
    missing_redacted_identifier[1].redacted_device_identifier = None;
    assert_eq!(
        validate_real_device_success_evidence(&missing_redacted_identifier),
        Err(RealDeviceSuccessValidationError::MissingRedactedIdentifier)
    );

    let mut raw_identifier = records;
    raw_identifier[2].redacted_device_identifier = Some("serial=ABC123".to_owned());
    assert!(matches!(
        validate_real_device_success_evidence(&raw_identifier),
        Err(RealDeviceSuccessValidationError::RawIdentifierMarker(_))
    ));
}

fn success_record(row: SuccessMatrixRow, exit_status: i32) -> RealDeviceEvidenceRecord {
    RealDeviceEvidenceRecord {
        base: DistroEvidenceRecord {
            correlation_id: row.kpi_row_id.to_owned(),
            command: row.wrapper_command.to_owned(),
            working_directory: DISTRO_COMMAND_ROOT.to_owned(),
            target_package: Some("arcrtc-distro-real-device-tests".to_owned()),
            target_scope: "tests/real-device".to_owned(),
            command_class: DistroCommandClass::RealDevice,
            distro_layer: DistroLayer::RealDevice,
            target_plane: DistroPlane::Ops,
            environment_class: DistroEnvironmentClass::RealDeviceBounded,
            toolchain_runtime_version: "real-device-success-test-runtime".to_owned(),
            input_fixture_or_workload: Some("reference-local".to_owned()),
            expected_outcome: "six required real-device wrapper row records exit 0 evidence"
                .to_owned(),
            actual_outcome: if exit_status == 0 {
                "real-device success evidence row observed with raw identifier absent"
            } else {
                "required real-device class not observed"
            }
            .to_owned(),
            exit_status: Some(exit_status),
            kernel_reason: None,
            distro_reason: if exit_status == 0 {
                DistroEvidenceReason::DistroOk
            } else {
                DistroEvidenceReason::RealDeviceScopeMismatch
            },
            non_claim_scope: vec![
                DistroNonClaimScope::NativeApplicationReadinessNotClaimed,
                DistroNonClaimScope::PublicDistributionReadinessNotClaimed,
                DistroNonClaimScope::ProductionReadinessNotClaimed,
                DistroNonClaimScope::LiveReadinessNotClaimed,
                DistroNonClaimScope::KernelCompletionNotClaimed,
            ],
            rerun_condition:
                "rerun when real-device platform command output or device class changes".to_owned(),
        },
        platform: row.platform.into(),
        device_class: row.device_class,
        internal_command_class: row.internal_command_class,
        runtime_version_class: row.runtime_version_class,
        execution_surface: match row.platform {
            CliPlatform::Android | CliPlatform::Ios => {
                evidence::RealDeviceExecutionSurface::NativeSdkCommand
            }
            CliPlatform::Browser => evidence::RealDeviceExecutionSurface::BrowserWebrtcCommand,
        },
        network_class: row.network_class,
        platform_command: match row.platform {
            CliPlatform::Android => Some("adb devices -l".to_owned()),
            CliPlatform::Ios if row.device_class == RealDeviceClass::IosPhysical => {
                Some("xcrun xctrace list devices".to_owned())
            }
            CliPlatform::Ios => Some("xcrun simctl list devices".to_owned()),
            CliPlatform::Browser => None,
        },
        preflight_outcome: match row.platform {
            CliPlatform::Android | CliPlatform::Ios => {
                REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED.to_owned()
            }
            CliPlatform::Browser => REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY.to_owned(),
        },
        logs_metrics_location: format!(
            "{DISTRO_EVIDENCE_ROOT}/real-device/{}.json",
            row.kpi_row_id.to_ascii_lowercase()
        ),
        redacted_device_identifier: (exit_status == 0).then(|| "redacted".to_owned()),
    }
}

impl From<CliPlatform> for RealDevicePlatform {
    fn from(value: CliPlatform) -> Self {
        match value {
            CliPlatform::Android => Self::Android,
            CliPlatform::Ios => Self::Ios,
            CliPlatform::Browser => Self::Browser,
        }
    }
}
