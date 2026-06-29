//! Five-item coverage closure tests for real-device success proof surfaces.

#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/command_output.rs"]
mod command_output;
#[path = "../src/device_observation.rs"]
mod device_observation;
#[path = "../src/dispatch.rs"]
mod dispatch;
#[path = "../src/error.rs"]
mod error;
#[path = "../src/evidence.rs"]
mod evidence;
#[path = "../src/redaction.rs"]
mod redaction;
#[path = "../src/wrapper.rs"]
mod wrapper;

use arcrtc_implementation_evidence::{
    validate_evidence_record, EvidenceValidationError, ImplementationCommandClass,
    ImplementationEnvironmentClass, ImplementationEvidenceReason, ImplementationEvidenceRecord,
    ImplementationLayer, ImplementationNonClaimScope, ImplementationPlane,
    IMPLEMENTATIONS_COMMAND_ROOT, IMPLEMENTATIONS_EVIDENCE_ROOT,
};
use cli::{CliDeviceClass, CliPlatform};
use command_output::{
    RealDeviceCommandExitStatus, RealDeviceCommandOutput, RealDeviceCommandUnavailable,
};
use device_observation::{
    parse_real_device_observation, RealDeviceObservation, RealDeviceObservationError,
};
use dispatch::dispatch_real_device_command;
use evidence::{
    validate_real_device_evidence_record, validate_real_device_profile, RealDeviceClass,
    RealDeviceEvidenceRecord, RealDeviceEvidenceValidationError, RealDeviceExecutionSurface,
    RealDeviceInternalCommandClass, RealDeviceNetworkClass, RealDevicePlatform,
    RealDeviceVersionClass, REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED,
    REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY, REAL_DEVICE_PROFILE_REFERENCE_LOCAL,
};
use redaction::{
    redact_real_device_identifier, reject_raw_real_device_identifier_marker,
    RealDeviceRedactionError,
};
use wrapper::{
    actual_outcome_for_exit, build_kpi_real_device_success_record, build_real_device_record,
    implementation_reason_for_exit, planned_exit_for_dispatch, preflight_outcome_for_dispatch,
    RealDeviceWrapperExit,
};

fn browser_record() -> RealDeviceEvidenceRecord {
    let dispatch =
        dispatch_real_device_command(CliPlatform::Browser, CliDeviceClass::DesktopBrowser)
            .expect("browser dispatch");
    build_real_device_record(
        dispatch,
        REAL_DEVICE_PROFILE_REFERENCE_LOCAL.to_owned(),
        RealDeviceWrapperExit::Success,
    )
}

fn android_record() -> RealDeviceEvidenceRecord {
    let dispatch =
        dispatch_real_device_command(CliPlatform::Android, CliDeviceClass::AndroidPhysical)
            .expect("android dispatch");
    build_real_device_record(
        dispatch,
        REAL_DEVICE_PROFILE_REFERENCE_LOCAL.to_owned(),
        RealDeviceWrapperExit::PlatformCommandUnavailable,
    )
}

fn observed(dispatch: &dispatch::RealDeviceDispatch) -> RealDeviceObservation {
    RealDeviceObservation {
        observed_device_class: dispatch.device_class,
        runtime_version_class: dispatch.runtime_version_class,
        network_class: dispatch.network_class,
        toolchain_runtime_version: "real-device-runtime".to_owned(),
        redacted_device_identifier: "redacted".to_owned(),
    }
}

fn base_evidence(command_class: ImplementationCommandClass) -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "real-device-coverage-closure".to_owned(),
        command: "cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-real-device-tests".to_owned()),
        target_scope: "tests/real-device".to_owned(),
        command_class,
        implementation_layer: ImplementationLayer::Readiness,
        target_plane: ImplementationPlane::Ops,
        environment_class: ImplementationEnvironmentClass::LocalSingleHost,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("real-device-coverage-closure".to_owned()),
        expected_outcome: "real-device coverage branches close".to_owned(),
        actual_outcome: "real-device coverage branches close".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::NativeApplicationReadinessNotClaimed,
            ImplementationNonClaimScope::PublicDistributionReadinessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "rerun when real-device coverage closure changes".to_owned(),
    }
}

#[test]
fn real_device_command_output_covers_status_reason_and_summary_branches() {
    for (status, code, reason, is_success) in [
        (
            RealDeviceCommandExitStatus::Success,
            0,
            ImplementationEvidenceReason::ImplementationOk,
            true,
        ),
        (
            RealDeviceCommandExitStatus::RequiredDeviceNotObserved,
            2,
            ImplementationEvidenceReason::RealDeviceScopeMismatch,
            false,
        ),
        (
            RealDeviceCommandExitStatus::PlatformCommandUnavailable,
            3,
            ImplementationEvidenceReason::RuntimeExecutorError,
            false,
        ),
        (
            RealDeviceCommandExitStatus::EvidenceFieldsIncomplete,
            4,
            ImplementationEvidenceReason::EvidenceFieldsIncomplete,
            false,
        ),
        (
            RealDeviceCommandExitStatus::CommandScopeMismatch,
            5,
            ImplementationEvidenceReason::CommandScopeMismatch,
            false,
        ),
    ] {
        assert_eq!(status.code(), code);
        assert_eq!(status.implementation_reason(), reason);
        assert_eq!(status.is_success(), is_success);
    }

    let summary = RealDeviceCommandOutput::success("  line 1\n\nline 2  ", " err ", " runtime ");
    assert_eq!(summary.stdout_summary, "line 1 | line 2");
    assert_eq!(summary.stderr_summary, "err");
    assert_eq!(summary.toolchain_runtime_version, "runtime");

    let long = "x".repeat(600);
    let truncated = RealDeviceCommandOutput::success(long, "", "runtime");
    assert!(truncated.stdout_summary.ends_with("..."));
    assert!(truncated.stdout_summary.len() <= 515);

    for reason in [
        RealDeviceCommandUnavailable::MissingProgram,
        RealDeviceCommandUnavailable::SpawnFailed,
        RealDeviceCommandUnavailable::NonZeroExit,
        RealDeviceCommandUnavailable::TimedOut,
        RealDeviceCommandUnavailable::CommandNotAdmitted,
    ] {
        let unavailable =
            RealDeviceCommandOutput::unavailable(reason, "", "", Some(127), "runtime");
        assert_eq!(
            unavailable.exit_status,
            RealDeviceCommandExitStatus::PlatformCommandUnavailable
        );
        assert_eq!(unavailable.unavailable, Some(reason));
    }

    assert_eq!(
        RealDeviceCommandOutput::success("no device", "", "runtime")
            .required_device_not_observed()
            .exit_status,
        RealDeviceCommandExitStatus::RequiredDeviceNotObserved
    );
    assert_eq!(
        RealDeviceCommandOutput::evidence_fields_incomplete("runtime").exit_status,
        RealDeviceCommandExitStatus::EvidenceFieldsIncomplete
    );
}

#[test]
fn real_device_observation_covers_platform_success_and_failure_branches() {
    let android =
        dispatch_real_device_command(CliPlatform::Android, CliDeviceClass::AndroidPhysical)
            .expect("android dispatch");
    let android_observation = parse_real_device_observation(
        &android,
        &RealDeviceCommandOutput::success(
            "List of devices attached | ABC123 device product:pixel",
            "",
            "adb devices -l",
        ),
    )
    .expect("android physical must be observed");
    assert_eq!(
        android_observation.observed_device_class,
        RealDeviceClass::AndroidPhysical
    );

    assert_eq!(
        parse_real_device_observation(
            &android,
            &RealDeviceCommandOutput::unavailable(
                RealDeviceCommandUnavailable::MissingProgram,
                "",
                "",
                None,
                "adb devices -l",
            ),
        ),
        Err(RealDeviceObservationError::CommandNotSuccessful)
    );
    assert_eq!(
        parse_real_device_observation(
            &android,
            &RealDeviceCommandOutput::success(
                "List of devices attached | emulator-5554 device product:sdk",
                "",
                "adb devices -l",
            ),
        ),
        Err(RealDeviceObservationError::RequiredDeviceNotObserved)
    );

    let ios = dispatch_real_device_command(CliPlatform::Ios, CliDeviceClass::IosPhysical)
        .expect("ios physical dispatch");
    assert!(parse_real_device_observation(
        &ios,
        &RealDeviceCommandOutput::success(
            "== Devices == | User iPhone (26.5) | == Simulators ==",
            "",
            "xcrun xctrace list devices",
        ),
    )
    .is_ok());

    let browser = dispatch_real_device_command(CliPlatform::Browser, CliDeviceClass::MobileBrowser)
        .expect("browser dispatch");
    assert!(parse_real_device_observation(
        &browser,
        &RealDeviceCommandOutput::success("", "", "browser runtime"),
    )
    .is_ok());
}

#[test]
fn real_device_redaction_rejects_raw_markers_and_accepts_redacted_sentinel() {
    assert_eq!(
        redact_real_device_identifier(None),
        Ok("redacted".to_owned())
    );
    assert_eq!(
        redact_real_device_identifier(Some("opaque-safe-label")),
        Ok("redacted".to_owned())
    );

    for marker in [
        "serial:ABC",
        "udid=ABC",
        "android_id:ABC",
        "device_id=ABC",
        "imei:ABC",
        "meid=ABC",
        "account:ABC",
        "token=ABC",
        "private_key:ABC",
        "device_name=ABC",
    ] {
        assert_eq!(
            reject_raw_real_device_identifier_marker(marker),
            Err(RealDeviceRedactionError::RawIdentifierMarker),
            "{marker}"
        );
        assert_eq!(
            redact_real_device_identifier(Some(marker)),
            Err(RealDeviceRedactionError::RawIdentifierMarker),
            "{marker}"
        );
    }
}

#[test]
fn real_device_evidence_validation_covers_context_and_command_branches() {
    assert_eq!(
        validate_real_device_evidence_record(&browser_record()),
        Ok(())
    );
    assert_eq!(
        validate_real_device_evidence_record(&android_record()),
        Ok(())
    );
    assert_eq!(
        validate_real_device_profile("unexpected-profile"),
        Err(RealDeviceEvidenceValidationError::ProfileNotAdmitted)
    );

    let mut wrong_command = browser_record();
    wrong_command.base.command_class = ImplementationCommandClass::Test;
    assert_eq!(
        validate_real_device_evidence_record(&wrong_command),
        Err(RealDeviceEvidenceValidationError::CommandClassMismatch)
    );

    let mut missing_scope = browser_record();
    missing_scope
        .base
        .non_claim_scope
        .retain(|scope| scope != &ImplementationNonClaimScope::KernelCompletionNotClaimed);
    assert_eq!(
        validate_real_device_evidence_record(&missing_scope),
        Err(RealDeviceEvidenceValidationError::MissingPostExecutionField)
    );

    let mut platform_mismatch = browser_record();
    platform_mismatch.platform = RealDevicePlatform::Android;
    assert_eq!(
        validate_real_device_evidence_record(&platform_mismatch),
        Err(RealDeviceEvidenceValidationError::PlatformDeviceClassMismatch)
    );

    let mut internal_mismatch = browser_record();
    internal_mismatch.internal_command_class = RealDeviceInternalCommandClass::AndroidDevice;
    assert_eq!(
        validate_real_device_evidence_record(&internal_mismatch),
        Err(RealDeviceEvidenceValidationError::InternalCommandClassMismatch)
    );

    let mut runtime_mismatch = browser_record();
    runtime_mismatch.runtime_version_class = RealDeviceVersionClass::AndroidApiLevel;
    assert_eq!(
        validate_real_device_evidence_record(&runtime_mismatch),
        Err(RealDeviceEvidenceValidationError::RuntimeVersionClassMismatch)
    );

    let mut execution_mismatch = browser_record();
    execution_mismatch.execution_surface = RealDeviceExecutionSurface::NativeSdkCommand;
    assert_eq!(
        validate_real_device_evidence_record(&execution_mismatch),
        Err(RealDeviceEvidenceValidationError::ExecutionSurfaceMismatch)
    );

    let mut network_mismatch = browser_record();
    network_mismatch.network_class = RealDeviceNetworkClass::LocalUsb;
    assert_eq!(
        validate_real_device_evidence_record(&network_mismatch),
        Err(RealDeviceEvidenceValidationError::NetworkClassMismatch)
    );

    let mut missing_command = android_record();
    missing_command.platform_command = None;
    assert_eq!(
        validate_real_device_evidence_record(&missing_command),
        Err(RealDeviceEvidenceValidationError::PlatformCommandMissing)
    );

    let mut unexpected_command = browser_record();
    unexpected_command.platform_command = Some("browser --list".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&unexpected_command),
        Err(RealDeviceEvidenceValidationError::PlatformCommandUnexpected)
    );

    let mut command_not_admitted = android_record();
    command_not_admitted.platform_command = Some("adb shell getprop".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&command_not_admitted),
        Err(RealDeviceEvidenceValidationError::PlatformCommandNotAdmitted)
    );
}

#[test]
fn real_device_evidence_validation_covers_post_execution_and_identifier_branches() {
    let mut invalid_preflight = browser_record();
    invalid_preflight.preflight_outcome =
        REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED.to_owned();
    assert_eq!(
        validate_real_device_evidence_record(&invalid_preflight),
        Err(RealDeviceEvidenceValidationError::PreflightOutcomeNotAdmitted)
    );

    let mut empty_logs = browser_record();
    empty_logs.logs_metrics_location.clear();
    assert_eq!(
        validate_real_device_evidence_record(&empty_logs),
        Err(RealDeviceEvidenceValidationError::EmptyLogsMetricsLocation)
    );

    let mut outside_logs = browser_record();
    outside_logs.logs_metrics_location = "/tmp/real-device.json".to_owned();
    assert_eq!(
        validate_real_device_evidence_record(&outside_logs),
        Err(RealDeviceEvidenceValidationError::LogsMetricsLocationOutsideEvidenceDir)
    );

    let mut empty_identifier = browser_record();
    empty_identifier.redacted_device_identifier = Some(" ".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&empty_identifier),
        Err(RealDeviceEvidenceValidationError::EmptyDeviceIdentifier)
    );

    let mut sha_identifier = browser_record();
    sha_identifier.redacted_device_identifier = Some(format!("sha256:{}", "a".repeat(64)));
    assert_eq!(
        validate_real_device_evidence_record(&sha_identifier),
        Ok(())
    );

    let mut uppercase_identifier = browser_record();
    uppercase_identifier.redacted_device_identifier = Some(format!("sha256:{}", "A".repeat(64)));
    assert_eq!(
        validate_real_device_evidence_record(&uppercase_identifier),
        Err(RealDeviceEvidenceValidationError::DeviceIdentifierFormatMismatch)
    );

    let mut raw_identifier = browser_record();
    raw_identifier.redacted_device_identifier = Some("device_id=ABC123".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&raw_identifier),
        Err(RealDeviceEvidenceValidationError::UnsafeDeviceIdentifier)
    );

    let mut missing_identifier = browser_record();
    missing_identifier.redacted_device_identifier = None;
    assert_eq!(
        validate_real_device_evidence_record(&missing_identifier),
        Err(RealDeviceEvidenceValidationError::MissingPostExecutionField)
    );

    let mut unexpected_identifier = android_record();
    unexpected_identifier.redacted_device_identifier = Some("redacted".to_owned());
    assert_eq!(
        validate_real_device_evidence_record(&unexpected_identifier),
        Err(RealDeviceEvidenceValidationError::UnexpectedDeviceIdentifier)
    );

    let mut missing_exit = browser_record();
    missing_exit.base.exit_status = None;
    assert_eq!(
        validate_real_device_evidence_record(&missing_exit),
        Err(RealDeviceEvidenceValidationError::Base(
            arcrtc_implementation_evidence::EvidenceValidationError::MissingRequiredExitStatus
        ))
    );

    let mut missing_preflight = browser_record();
    missing_preflight.preflight_outcome.clear();
    assert_eq!(
        validate_real_device_evidence_record(&missing_preflight),
        Err(RealDeviceEvidenceValidationError::MissingPostExecutionField)
    );

    assert!(browser_record()
        .logs_metrics_location
        .starts_with(&format!("{IMPLEMENTATIONS_EVIDENCE_ROOT}/real-device/")));
}

#[test]
fn real_device_wrapper_success_record_covers_success_outcome_matrix() {
    for (platform, device_class, output, observation_expected) in [
        (
            CliPlatform::Android,
            CliDeviceClass::AndroidPhysical,
            RealDeviceCommandOutput::success(
                "List of devices attached | ABC123 device product:pixel",
                "",
                "adb devices -l",
            ),
            true,
        ),
        (
            CliPlatform::Android,
            CliDeviceClass::AndroidEmulator,
            RealDeviceCommandOutput::success(
                "List of devices attached | emulator-5554 device product:sdk",
                "",
                "adb devices -l",
            ),
            true,
        ),
        (
            CliPlatform::Ios,
            CliDeviceClass::IosSimulator,
            RealDeviceCommandOutput::success(
                "== Devices == | iPhone 16 Pro (ABC) (Booted)",
                "",
                "xcrun simctl list devices",
            ),
            true,
        ),
        (
            CliPlatform::Browser,
            CliDeviceClass::DesktopBrowser,
            RealDeviceCommandOutput::success("", "", "browser runtime"),
            true,
        ),
        (
            CliPlatform::Browser,
            CliDeviceClass::MobileBrowser,
            RealDeviceCommandOutput::evidence_fields_incomplete("browser runtime"),
            false,
        ),
    ] {
        let dispatch = dispatch_real_device_command(platform, device_class).expect("dispatch");
        let observation = observation_expected.then(|| observed(&dispatch));
        let record = build_kpi_real_device_success_record(
            dispatch.clone(),
            REAL_DEVICE_PROFILE_REFERENCE_LOCAL.to_owned(),
            output.clone(),
            observation,
        );
        assert_eq!(
            record.base.implementation_reason,
            output.exit_status.implementation_reason()
        );
        assert_eq!(
            record.preflight_outcome,
            if dispatch.platform_command.is_some()
                && matches!(
                    output.exit_status,
                    RealDeviceCommandExitStatus::Success
                        | RealDeviceCommandExitStatus::RequiredDeviceNotObserved
                )
            {
                "execution:platform-command-executed"
            } else if output.exit_status == RealDeviceCommandExitStatus::Success {
                REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY
            } else {
                preflight_outcome_for_dispatch(&dispatch)
            }
        );
    }
}

#[test]
fn real_device_wrapper_exit_mapping_covers_outcome_reason_helpers() {
    for exit in [
        RealDeviceWrapperExit::Success,
        RealDeviceWrapperExit::ScopeMismatch,
        RealDeviceWrapperExit::PlatformCommandUnavailable,
        RealDeviceWrapperExit::EvidenceValidationFailure,
        RealDeviceWrapperExit::CommandScopeMismatch,
    ] {
        assert!(!actual_outcome_for_exit(exit).is_empty());
        assert_eq!(
            implementation_reason_for_exit(exit),
            match exit {
                RealDeviceWrapperExit::Success => ImplementationEvidenceReason::ImplementationOk,
                RealDeviceWrapperExit::ScopeMismatch => {
                    ImplementationEvidenceReason::RealDeviceScopeMismatch
                }
                RealDeviceWrapperExit::PlatformCommandUnavailable => {
                    ImplementationEvidenceReason::RuntimeExecutorError
                }
                RealDeviceWrapperExit::EvidenceValidationFailure => {
                    ImplementationEvidenceReason::EvidenceFieldsIncomplete
                }
                RealDeviceWrapperExit::CommandScopeMismatch => {
                    ImplementationEvidenceReason::CommandScopeMismatch
                }
            }
        );
    }

    let browser =
        dispatch_real_device_command(CliPlatform::Browser, CliDeviceClass::DesktopBrowser)
            .expect("browser dispatch");
    assert_eq!(
        planned_exit_for_dispatch(&browser),
        RealDeviceWrapperExit::Success
    );
    let android =
        dispatch_real_device_command(CliPlatform::Android, CliDeviceClass::AndroidPhysical)
            .expect("android dispatch");
    assert_eq!(
        planned_exit_for_dispatch(&android),
        RealDeviceWrapperExit::PlatformCommandUnavailable
    );
}

#[test]
fn real_device_base_evidence_validation_covers_shared_fail_closed_branches() {
    for (record, expected) in [
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.correlation_id.clear();
            (record, EvidenceValidationError::EmptyCorrelationId)
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.command.clear();
            (record, EvidenceValidationError::EmptyCommand)
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.working_directory.clear();
            (record, EvidenceValidationError::EmptyWorkingDirectory)
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.working_directory = "/tmp".to_owned();
            (
                record,
                EvidenceValidationError::WorkingDirectoryOutsideImplementations,
            )
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.actual_outcome.clear();
            (record, EvidenceValidationError::EmptyActualOutcome)
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.rerun_condition.clear();
            (record, EvidenceValidationError::EmptyRerunCondition)
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.target_package = Some(" ".to_owned());
            (
                record,
                EvidenceValidationError::MissingRequiredTargetPackage,
            )
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.input_fixture_or_workload = None;
            (
                record,
                EvidenceValidationError::MissingRequiredInputFixtureOrWorkload,
            )
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.kernel_reason = Some("other".to_owned());
            (record, EvidenceValidationError::ForbiddenUnclassifiedReason)
        },
        {
            let mut record = base_evidence(ImplementationCommandClass::RealDevice);
            record.target_scope = "raw_packet=payload".to_owned();
            (record, EvidenceValidationError::RawSecretLikeValue)
        },
    ] {
        assert_eq!(validate_evidence_record(&record), Err(expected));
    }

    for (command_class, scopes) in [
        (
            ImplementationCommandClass::Format,
            vec![ImplementationNonClaimScope::CommandTargetSuccessNotClaimed],
        ),
        (
            ImplementationCommandClass::Build,
            vec![
                ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
                ImplementationNonClaimScope::ProductionReadinessNotClaimed,
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            ImplementationCommandClass::Test,
            vec![
                ImplementationNonClaimScope::ProductionReadinessNotClaimed,
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            ImplementationCommandClass::Benchmark,
            vec![
                ImplementationNonClaimScope::BenchmarkThresholdNotClaimed,
                ImplementationNonClaimScope::ProductionReadinessNotClaimed,
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            ImplementationCommandClass::ProductionReadiness,
            vec![
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
                ImplementationNonClaimScope::KernelCompletionNotClaimed,
                ImplementationNonClaimScope::KernelFreezeNotClaimed,
            ],
        ),
        (
            ImplementationCommandClass::LiveReadiness,
            vec![
                ImplementationNonClaimScope::KernelCompletionNotClaimed,
                ImplementationNonClaimScope::KernelFreezeNotClaimed,
            ],
        ),
    ] {
        let mut record = base_evidence(command_class);
        record.non_claim_scope = scopes;
        assert_eq!(
            validate_evidence_record(&record),
            Ok(()),
            "{command_class:?}"
        );
    }
}
