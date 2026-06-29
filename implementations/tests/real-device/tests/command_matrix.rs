//! real-device command matrix 境界を検査します。

#[path = "../src/evidence.rs"]
mod evidence;

use arcrtc_implementation_evidence::{
    ImplementationCommandClass, ImplementationEnvironmentClass, ImplementationEvidenceReason,
    ImplementationEvidenceRecord, ImplementationLayer, ImplementationNonClaimScope,
    ImplementationPlane, IMPLEMENTATIONS_COMMAND_ROOT, IMPLEMENTATIONS_EVIDENCE_ROOT,
};
use evidence::{
    validate_kpi_bounded_real_device_evidence, validate_real_device_evidence_record,
    RealDeviceClass, RealDeviceEvidenceRecord, RealDeviceEvidenceValidationError,
    RealDeviceExecutionSurface, RealDeviceInternalCommandClass, RealDeviceNetworkClass,
    RealDevicePlatform, RealDeviceVersionClass, REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE,
    REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
};

fn base(exit_status: i32) -> ImplementationEvidenceRecord {
    let (actual_outcome, implementation_reason) = match exit_status {
        0 => (
            "wrapper-local capability recorded",
            ImplementationEvidenceReason::ImplementationOk,
        ),
        3 => (
            "platform command unavailable in bounded wrapper",
            ImplementationEvidenceReason::RuntimeExecutorError,
        ),
        _ => (
            "wrapper evidence validation failed",
            ImplementationEvidenceReason::EvidenceFieldsIncomplete,
        ),
    };
    ImplementationEvidenceRecord {
        correlation_id: "real-device-command-matrix".to_owned(),
        command: "cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-real-device-tests".to_owned()),
        target_scope: "tests/real-device".to_owned(),
        command_class: ImplementationCommandClass::RealDevice,
        implementation_layer: ImplementationLayer::RealDevice,
        target_plane: ImplementationPlane::Ops,
        environment_class: ImplementationEnvironmentClass::RealDeviceBounded,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("reference-local".to_owned()),
        expected_outcome: "wrapper records command exit status and evidence JSON".to_owned(),
        actual_outcome: actual_outcome.to_owned(),
        exit_status: Some(exit_status),
        kernel_reason: None,
        implementation_reason,
        non_claim_scope: vec![
            ImplementationNonClaimScope::NativeApplicationReadinessNotClaimed,
            ImplementationNonClaimScope::PublicDistributionReadinessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
        ],
        rerun_condition: "rerun when wrapper command changes".to_owned(),
    }
}

fn browser_record(exit_status: i32) -> RealDeviceEvidenceRecord {
    RealDeviceEvidenceRecord {
        base: base(exit_status),
        platform: RealDevicePlatform::Browser,
        device_class: RealDeviceClass::DesktopBrowser,
        internal_command_class: RealDeviceInternalCommandClass::Browser,
        runtime_version_class: RealDeviceVersionClass::BrowserVersion,
        execution_surface: RealDeviceExecutionSurface::BrowserWebrtcCommand,
        network_class: RealDeviceNetworkClass::LocalBrowser,
        platform_command: None,
        preflight_outcome: REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY.to_owned(),
        logs_metrics_location: format!("{IMPLEMENTATIONS_EVIDENCE_ROOT}/real-device/browser.json"),
        redacted_device_identifier: (exit_status == 0).then(|| "redacted".to_owned()),
    }
}

fn android_record(exit_status: i32) -> RealDeviceEvidenceRecord {
    RealDeviceEvidenceRecord {
        base: base(exit_status),
        platform: RealDevicePlatform::Android,
        device_class: RealDeviceClass::AndroidPhysical,
        internal_command_class: RealDeviceInternalCommandClass::AndroidDevice,
        runtime_version_class: RealDeviceVersionClass::AndroidApiLevel,
        execution_surface: RealDeviceExecutionSurface::NativeSdkCommand,
        network_class: RealDeviceNetworkClass::LocalUsb,
        platform_command: Some("adb devices -l".to_owned()),
        preflight_outcome: REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE.to_owned(),
        logs_metrics_location: format!("{IMPLEMENTATIONS_EVIDENCE_ROOT}/real-device/android.json"),
        redacted_device_identifier: (exit_status == 0).then(|| "redacted".to_owned()),
    }
}

fn android_emulator_record(exit_status: i32) -> RealDeviceEvidenceRecord {
    RealDeviceEvidenceRecord {
        device_class: RealDeviceClass::AndroidEmulator,
        network_class: RealDeviceNetworkClass::EmulatorLoopback,
        ..android_record(exit_status)
    }
}

fn ios_physical_record(exit_status: i32) -> RealDeviceEvidenceRecord {
    RealDeviceEvidenceRecord {
        base: base(exit_status),
        platform: RealDevicePlatform::Ios,
        device_class: RealDeviceClass::IosPhysical,
        internal_command_class: RealDeviceInternalCommandClass::IosDevice,
        runtime_version_class: RealDeviceVersionClass::IosSystemVersion,
        execution_surface: RealDeviceExecutionSurface::NativeSdkCommand,
        network_class: RealDeviceNetworkClass::LocalUsb,
        platform_command: Some("xcrun xctrace list devices".to_owned()),
        preflight_outcome: REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE.to_owned(),
        logs_metrics_location: format!("{IMPLEMENTATIONS_EVIDENCE_ROOT}/real-device/ios.json"),
        redacted_device_identifier: (exit_status == 0).then(|| "redacted".to_owned()),
    }
}

fn ios_simulator_record(exit_status: i32) -> RealDeviceEvidenceRecord {
    RealDeviceEvidenceRecord {
        device_class: RealDeviceClass::IosSimulator,
        internal_command_class: RealDeviceInternalCommandClass::IosSimulator,
        network_class: RealDeviceNetworkClass::SimulatorLoopback,
        platform_command: Some("xcrun simctl list devices".to_owned()),
        ..ios_physical_record(exit_status)
    }
}

fn mobile_browser_record(exit_status: i32) -> RealDeviceEvidenceRecord {
    RealDeviceEvidenceRecord {
        device_class: RealDeviceClass::MobileBrowser,
        network_class: RealDeviceNetworkClass::MobileBrowserEmulation,
        ..browser_record(exit_status)
    }
}

#[test]
fn kpi_real_device_command_matrix_requires_bounded_evidence_fields() {
    for record in [
        android_record(3),
        android_emulator_record(3),
        ios_physical_record(3),
        ios_simulator_record(3),
        browser_record(0),
        mobile_browser_record(0),
    ] {
        let expected_exit_status = record.base.exit_status;
        let expected_preflight = if record.platform_command.is_some() {
            REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE
        } else {
            REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY
        };
        assert_eq!(
            record.base.command_class,
            ImplementationCommandClass::RealDevice
        );
        assert_eq!(
            record.base.environment_class,
            ImplementationEnvironmentClass::RealDeviceBounded
        );
        assert!(expected_exit_status.is_some());
        match expected_exit_status {
            Some(0) => {
                assert_eq!(
                    record.base.actual_outcome,
                    "wrapper-local capability recorded"
                );
                assert_eq!(
                    record.base.implementation_reason,
                    ImplementationEvidenceReason::ImplementationOk
                );
            }
            Some(3) => {
                assert_eq!(
                    record.base.actual_outcome,
                    "platform command unavailable in bounded wrapper"
                );
                assert_eq!(
                    record.base.implementation_reason,
                    ImplementationEvidenceReason::RuntimeExecutorError
                );
            }
            other => panic!("unexpected real-device exit status {other:?}"),
        }
        assert_eq!(record.preflight_outcome, expected_preflight);
        assert!(record
            .logs_metrics_location
            .starts_with(&format!("{IMPLEMENTATIONS_EVIDENCE_ROOT}/real-device/")));
        assert_eq!(validate_kpi_bounded_real_device_evidence(&record), Ok(()));
    }
}

#[test]
fn real_device_evidence_accepts_canonical_browser_record() {
    assert_eq!(
        validate_real_device_evidence_record(&browser_record(0)),
        Ok(())
    );

    let mut hashed_identifier = browser_record(0);
    hashed_identifier.redacted_device_identifier = Some(format!("sha256:{}", "0".repeat(64)));
    assert_eq!(
        validate_real_device_evidence_record(&hashed_identifier),
        Ok(())
    );
}

#[test]
fn real_device_evidence_rejects_context_mismatches() {
    let mut wrong_command_class = browser_record(0);
    wrong_command_class.base.command_class = ImplementationCommandClass::Build;
    wrong_command_class
        .base
        .non_claim_scope
        .push(ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed);
    assert_eq!(
        validate_real_device_evidence_record(&wrong_command_class),
        Err(RealDeviceEvidenceValidationError::CommandClassMismatch)
    );

    let mut missing_native_non_claim = browser_record(0);
    missing_native_non_claim
        .base
        .non_claim_scope
        .retain(|scope| {
            *scope != ImplementationNonClaimScope::NativeApplicationReadinessNotClaimed
        });
    assert_eq!(
        validate_real_device_evidence_record(&missing_native_non_claim),
        Err(RealDeviceEvidenceValidationError::Base(
            arcrtc_implementation_evidence::EvidenceValidationError::MissingRequiredNonClaimScope
        ))
    );

    let mut missing_kernel_non_claim = browser_record(0);
    missing_kernel_non_claim
        .base
        .non_claim_scope
        .retain(|scope| *scope != ImplementationNonClaimScope::KernelCompletionNotClaimed);
    assert_eq!(
        validate_real_device_evidence_record(&missing_kernel_non_claim),
        Err(RealDeviceEvidenceValidationError::MissingPostExecutionField)
    );

    let mut wrong_platform = browser_record(0);
    wrong_platform.platform = RealDevicePlatform::Android;
    assert_eq!(
        validate_real_device_evidence_record(&wrong_platform),
        Err(RealDeviceEvidenceValidationError::PlatformDeviceClassMismatch)
    );

    let mut wrong_internal = browser_record(0);
    wrong_internal.internal_command_class = RealDeviceInternalCommandClass::AndroidDevice;
    assert_eq!(
        validate_real_device_evidence_record(&wrong_internal),
        Err(RealDeviceEvidenceValidationError::InternalCommandClassMismatch)
    );

    let mut wrong_runtime = browser_record(0);
    wrong_runtime.runtime_version_class = RealDeviceVersionClass::AndroidApiLevel;
    assert_eq!(
        validate_real_device_evidence_record(&wrong_runtime),
        Err(RealDeviceEvidenceValidationError::RuntimeVersionClassMismatch)
    );

    let mut wrong_surface = browser_record(0);
    wrong_surface.execution_surface = RealDeviceExecutionSurface::NativeSdkCommand;
    assert_eq!(
        validate_real_device_evidence_record(&wrong_surface),
        Err(RealDeviceEvidenceValidationError::ExecutionSurfaceMismatch)
    );

    let mut wrong_network = browser_record(0);
    wrong_network.network_class = RealDeviceNetworkClass::LocalUsb;
    assert_eq!(
        validate_real_device_evidence_record(&wrong_network),
        Err(RealDeviceEvidenceValidationError::NetworkClassMismatch)
    );
}

#[test]
fn real_device_evidence_rejects_platform_command_mismatches() {
    let mut missing_platform_command = android_record(2);
    missing_platform_command.platform_command = None;
    assert_eq!(
        validate_real_device_evidence_record(&missing_platform_command),
        Err(RealDeviceEvidenceValidationError::PlatformCommandMissing)
    );

    let mut unexpected_platform_command = browser_record(0);
    unexpected_platform_command.platform_command = Some("playwright test".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&unexpected_platform_command),
        Err(RealDeviceEvidenceValidationError::PlatformCommandUnexpected)
    );

    let mut not_admitted_platform_command = android_record(2);
    not_admitted_platform_command.platform_command = Some("adb shell getprop".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&not_admitted_platform_command),
        Err(RealDeviceEvidenceValidationError::PlatformCommandNotAdmitted)
    );
}

#[test]
fn real_device_evidence_rejects_identifier_requiredness_violations() {
    let mut missing_identifier = browser_record(0);
    missing_identifier.redacted_device_identifier = None;
    assert_eq!(
        validate_real_device_evidence_record(&missing_identifier),
        Err(RealDeviceEvidenceValidationError::MissingPostExecutionField)
    );

    let mut unexpected_identifier = browser_record(2);
    unexpected_identifier.redacted_device_identifier = Some("redacted".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&unexpected_identifier),
        Err(RealDeviceEvidenceValidationError::UnexpectedDeviceIdentifier)
    );
}

#[test]
fn real_device_evidence_rejects_post_execution_field_violations() {
    let mut missing_preflight = browser_record(0);
    missing_preflight.preflight_outcome.clear();
    assert_eq!(
        validate_real_device_evidence_record(&missing_preflight),
        Err(RealDeviceEvidenceValidationError::MissingPostExecutionField)
    );

    let mut unknown_preflight = browser_record(0);
    unknown_preflight.preflight_outcome = "preflight fields recorded".to_owned();
    assert_eq!(
        validate_real_device_evidence_record(&unknown_preflight),
        Err(RealDeviceEvidenceValidationError::PreflightOutcomeNotAdmitted)
    );

    let mut missing_exit_status = browser_record(0);
    missing_exit_status.base.exit_status = None;
    assert_eq!(
        validate_real_device_evidence_record(&missing_exit_status),
        Err(RealDeviceEvidenceValidationError::Base(
            arcrtc_implementation_evidence::EvidenceValidationError::MissingRequiredExitStatus
        ))
    );

    let mut empty_logs = browser_record(0);
    empty_logs.logs_metrics_location.clear();
    assert_eq!(
        validate_real_device_evidence_record(&empty_logs),
        Err(RealDeviceEvidenceValidationError::EmptyLogsMetricsLocation)
    );

    let mut outside_logs = browser_record(0);
    outside_logs.logs_metrics_location = "/tmp/real-device/browser.json".to_owned();
    assert_eq!(
        validate_real_device_evidence_record(&outside_logs),
        Err(RealDeviceEvidenceValidationError::LogsMetricsLocationOutsideEvidenceDir)
    );

    let mut empty_identifier = browser_record(0);
    empty_identifier.redacted_device_identifier = Some("   ".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&empty_identifier),
        Err(RealDeviceEvidenceValidationError::EmptyDeviceIdentifier)
    );

    let mut unsafe_identifier = browser_record(0);
    unsafe_identifier.redacted_device_identifier = Some("serial=ABC123".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&unsafe_identifier),
        Err(RealDeviceEvidenceValidationError::UnsafeDeviceIdentifier)
    );

    let mut invalid_identifier_format = browser_record(0);
    invalid_identifier_format.redacted_device_identifier = Some("sha256:ABC".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&invalid_identifier_format),
        Err(RealDeviceEvidenceValidationError::DeviceIdentifierFormatMismatch)
    );
}
