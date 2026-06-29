//! bounded real-device wrapper command の entrypoint です。

mod cli;
mod command_output;
mod device_observation;
mod dispatch;
mod error;
mod evidence;
mod platform_executor;
mod redaction;
mod wrapper;

use clap::Parser;
use cli::RealDeviceCli;
use device_observation::{parse_real_device_observation, RealDeviceObservationError};
use dispatch::dispatch_kpi_real_device_success_command;
use evidence::{build_kpi_real_device_success_evidence_record, validate_real_device_profile};
use platform_executor::execute_real_device_platform_command;
use wrapper::{
    all_wrapper_exit_codes, build_kpi_real_device_success_record,
    is_current_working_directory_implementations_root, RealDeviceWrapperExit,
};

fn main() {
    debug_assert_eq!(all_wrapper_exit_codes(), [0, 2, 3, 4, 5]);
    if !is_current_working_directory_implementations_root() {
        std::process::exit(RealDeviceWrapperExit::CommandScopeMismatch.code());
    }
    let exit_status = run_real_device_wrapper(RealDeviceCli::parse());
    std::process::exit(exit_status);
}

/// real-device wrapper の CLI orchestration 境界です。
pub fn run_real_device_wrapper(cli: RealDeviceCli) -> i32 {
    if !is_current_working_directory_implementations_root() {
        return RealDeviceWrapperExit::CommandScopeMismatch.code();
    }
    if validate_real_device_profile(&cli.profile).is_err() {
        return RealDeviceWrapperExit::EvidenceValidationFailure.code();
    }
    let dispatch = match dispatch_kpi_real_device_success_command(cli.platform, cli.device_class) {
        Ok(dispatch) => dispatch,
        Err(_) => return RealDeviceWrapperExit::ScopeMismatch.code(),
    };
    let mut command_output = execute_real_device_platform_command(&dispatch);
    let observation = match parse_real_device_observation(&dispatch, &command_output) {
        Ok(observation) => Some(observation),
        Err(RealDeviceObservationError::RequiredDeviceNotObserved) => {
            command_output = command_output.required_device_not_observed();
            None
        }
        Err(_) => None,
    };
    let record =
        build_kpi_real_device_success_record(dispatch, cli.profile, command_output, observation);
    let exit_status = record
        .base
        .exit_status
        .unwrap_or(RealDeviceWrapperExit::EvidenceValidationFailure.code());
    let record = match build_kpi_real_device_success_evidence_record(record) {
        Ok(record) => record,
        Err(_) => return RealDeviceWrapperExit::EvidenceValidationFailure.code(),
    };
    match serde_json::to_string_pretty(&record) {
        Ok(json) => println!("{json}"),
        Err(_) => return RealDeviceWrapperExit::EvidenceValidationFailure.code(),
    }
    exit_status
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::{
        path::PathBuf,
        sync::{Mutex, OnceLock},
    };

    use super::{
        cli::{CliDeviceClass, CliPlatform, RealDeviceCli},
        command_output::{
            RealDeviceCommandExitStatus, RealDeviceCommandOutput, RealDeviceCommandUnavailable,
        },
        device_observation::parse_real_device_observation,
        dispatch::dispatch_real_device_command,
        evidence::{
            validate_real_device_evidence_record, RealDeviceClass, RealDeviceEvidenceRecord,
            RealDeviceEvidenceValidationError, RealDeviceExecutionSurface,
            RealDeviceInternalCommandClass, RealDeviceNetworkClass, RealDevicePlatform,
            RealDeviceVersionClass, REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED,
            REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY, REAL_DEVICE_PROFILE_REFERENCE_LOCAL,
        },
        redaction::{
            redact_real_device_identifier, reject_raw_real_device_identifier_marker,
            RealDeviceRedactionError,
        },
        wrapper::{
            actual_outcome_for_exit, build_real_device_record, implementation_reason_for_exit,
            planned_exit_for_dispatch, preflight_outcome_for_dispatch, RealDeviceWrapperExit,
        },
    };
    use arcrtc_implementation_evidence::{
        ImplementationCommandClass, ImplementationEvidenceReason, ImplementationNonClaimScope,
    };

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn implementations_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("implementations root must exist")
    }

    struct CurrentDirRestore(PathBuf);

    impl Drop for CurrentDirRestore {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }

    fn output_for(device_class: CliDeviceClass) -> RealDeviceCommandOutput {
        match device_class {
            CliDeviceClass::AndroidPhysical => RealDeviceCommandOutput::success(
                "List of devices attached | ABC123 device product:pixel",
                "",
                "adb devices -l",
            ),
            CliDeviceClass::AndroidEmulator => RealDeviceCommandOutput::success(
                "List of devices attached | emulator-5554 device product:sdk",
                "",
                "adb devices -l",
            ),
            CliDeviceClass::IosPhysical => RealDeviceCommandOutput::success(
                "== Devices == | User iPhone (26.5) | == Simulators ==",
                "",
                "xcrun xctrace list devices",
            ),
            CliDeviceClass::IosSimulator => RealDeviceCommandOutput::success(
                "== Devices == | iPhone 16 Pro (ABC) (Booted)",
                "",
                "xcrun simctl list devices",
            ),
            CliDeviceClass::DesktopBrowser | CliDeviceClass::MobileBrowser => {
                RealDeviceCommandOutput::success("", "", "browser runtime")
            }
        }
    }

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

    #[test]
    fn unit_command_output_and_wrapper_helpers_cover_closed_sets() {
        for (status, code, reason, success) in [
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
            let status = black_box(status);
            assert_eq!(status.code(), code);
            assert_eq!(status.implementation_reason(), reason);
            assert_eq!(status.is_success(), success);
        }

        for exit in [
            RealDeviceWrapperExit::Success,
            RealDeviceWrapperExit::ScopeMismatch,
            RealDeviceWrapperExit::PlatformCommandUnavailable,
            RealDeviceWrapperExit::EvidenceValidationFailure,
            RealDeviceWrapperExit::CommandScopeMismatch,
        ] {
            let exit = black_box(exit);
            assert!(!actual_outcome_for_exit(exit).is_empty());
            assert_eq!(
                RealDeviceWrapperExit::from(match exit {
                    RealDeviceWrapperExit::Success => RealDeviceCommandExitStatus::Success,
                    RealDeviceWrapperExit::ScopeMismatch => {
                        RealDeviceCommandExitStatus::RequiredDeviceNotObserved
                    }
                    RealDeviceWrapperExit::PlatformCommandUnavailable => {
                        RealDeviceCommandExitStatus::PlatformCommandUnavailable
                    }
                    RealDeviceWrapperExit::EvidenceValidationFailure => {
                        RealDeviceCommandExitStatus::EvidenceFieldsIncomplete
                    }
                    RealDeviceWrapperExit::CommandScopeMismatch => {
                        RealDeviceCommandExitStatus::CommandScopeMismatch
                    }
                }),
                exit
            );
            assert_eq!(
                implementation_reason_for_exit(exit),
                match exit {
                    RealDeviceWrapperExit::Success =>
                        ImplementationEvidenceReason::ImplementationOk,
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

        let summary = RealDeviceCommandOutput::success(" a\n\n b ", " err ", " runtime ");
        assert_eq!(summary.stdout_summary, "a | b");
        assert_eq!(
            summary.required_device_not_observed().exit_status,
            RealDeviceCommandExitStatus::RequiredDeviceNotObserved
        );
        assert_eq!(
            RealDeviceCommandOutput::evidence_fields_incomplete("runtime").exit_status,
            RealDeviceCommandExitStatus::EvidenceFieldsIncomplete
        );
        for reason in [
            RealDeviceCommandUnavailable::MissingProgram,
            RealDeviceCommandUnavailable::SpawnFailed,
            RealDeviceCommandUnavailable::NonZeroExit,
            RealDeviceCommandUnavailable::TimedOut,
            RealDeviceCommandUnavailable::CommandNotAdmitted,
        ] {
            assert_eq!(
                RealDeviceCommandOutput::unavailable(reason, "", "", None, "runtime").unavailable,
                Some(reason)
            );
        }
    }

    #[test]
    fn unit_dispatch_observation_and_redaction_cover_closed_sets() {
        for (platform, device_class, expected_class) in [
            (
                CliPlatform::Android,
                CliDeviceClass::AndroidPhysical,
                RealDeviceClass::AndroidPhysical,
            ),
            (
                CliPlatform::Android,
                CliDeviceClass::AndroidEmulator,
                RealDeviceClass::AndroidEmulator,
            ),
            (
                CliPlatform::Ios,
                CliDeviceClass::IosPhysical,
                RealDeviceClass::IosPhysical,
            ),
            (
                CliPlatform::Ios,
                CliDeviceClass::IosSimulator,
                RealDeviceClass::IosSimulator,
            ),
            (
                CliPlatform::Browser,
                CliDeviceClass::DesktopBrowser,
                RealDeviceClass::DesktopBrowser,
            ),
            (
                CliPlatform::Browser,
                CliDeviceClass::MobileBrowser,
                RealDeviceClass::MobileBrowser,
            ),
        ] {
            let dispatch = dispatch_real_device_command(platform, device_class).expect("dispatch");
            assert_eq!(dispatch.device_class, expected_class);
            assert!(planned_exit_for_dispatch(&dispatch).code() <= 3);
            assert!(!preflight_outcome_for_dispatch(&dispatch).is_empty());
            let output = output_for(device_class);
            assert!(parse_real_device_observation(&dispatch, &output).is_ok());
        }
        assert!(
            dispatch_real_device_command(CliPlatform::Android, CliDeviceClass::IosPhysical)
                .is_err()
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
                Err(RealDeviceRedactionError::RawIdentifierMarker)
            );
            assert_eq!(
                redact_real_device_identifier(Some(marker)),
                Err(RealDeviceRedactionError::RawIdentifierMarker)
            );
        }
        assert_eq!(
            redact_real_device_identifier(None),
            Ok("redacted".to_owned())
        );
        assert_eq!(
            reject_raw_real_device_identifier_marker("safe-label"),
            Ok(())
        );
        assert_eq!(
            redact_real_device_identifier(Some("safe-label")),
            Ok("redacted".to_owned())
        );
    }

    #[test]
    fn unit_evidence_validation_covers_context_and_post_execution_matrix() {
        let mut record = browser_record();
        record.base.command_class = ImplementationCommandClass::Test;
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::CommandClassMismatch)
        );

        let mut record = browser_record();
        record.platform = RealDevicePlatform::Android;
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::PlatformDeviceClassMismatch)
        );

        let mut record = browser_record();
        record.internal_command_class = RealDeviceInternalCommandClass::AndroidDevice;
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::InternalCommandClassMismatch)
        );

        let mut record = browser_record();
        record.runtime_version_class = RealDeviceVersionClass::AndroidApiLevel;
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::RuntimeVersionClassMismatch)
        );

        let mut record = browser_record();
        record.execution_surface = RealDeviceExecutionSurface::NativeSdkCommand;
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::ExecutionSurfaceMismatch)
        );

        let mut record = browser_record();
        record.network_class = RealDeviceNetworkClass::LocalUsb;
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::NetworkClassMismatch)
        );

        let mut record = browser_record();
        record.platform_command = Some("browser --list".to_owned());
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::PlatformCommandUnexpected)
        );

        let mut record = browser_record();
        record.preflight_outcome = REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED.to_owned();
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::PreflightOutcomeNotAdmitted)
        );

        let mut record = browser_record();
        record.redacted_device_identifier = Some(format!("sha256:{}", "a".repeat(64)));
        assert_eq!(validate_real_device_evidence_record(&record), Ok(()));

        let mut record = browser_record();
        record.redacted_device_identifier = Some("device_id=ABC".to_owned());
        assert_eq!(
            validate_real_device_evidence_record(&record),
            Err(RealDeviceEvidenceValidationError::UnsafeDeviceIdentifier)
        );
    }

    #[test]
    fn unit_wrapper_orchestration_covers_scope_and_profile_fail_closed() {
        let _guard = cwd_lock().lock().expect("cwd lock");
        let _restore = CurrentDirRestore(std::env::current_dir().expect("current dir"));

        let cli = RealDeviceCli {
            platform: CliPlatform::Browser,
            profile: REAL_DEVICE_PROFILE_REFERENCE_LOCAL.to_owned(),
            device_class: CliDeviceClass::DesktopBrowser,
        };
        std::env::set_current_dir(std::env::temp_dir()).expect("temp cwd");
        let exit = super::run_real_device_wrapper(cli);
        assert_eq!(exit, RealDeviceWrapperExit::CommandScopeMismatch.code());

        std::env::set_current_dir(implementations_root()).expect("implementations cwd");
        let cli = RealDeviceCli {
            platform: CliPlatform::Browser,
            profile: REAL_DEVICE_PROFILE_REFERENCE_LOCAL.to_owned(),
            device_class: CliDeviceClass::DesktopBrowser,
        };
        assert_eq!(
            super::run_real_device_wrapper(cli),
            RealDeviceWrapperExit::Success.code()
        );

        let invalid = RealDeviceCli {
            platform: CliPlatform::Browser,
            profile: "serial=ABC".to_owned(),
            device_class: CliDeviceClass::DesktopBrowser,
        };
        let exit = super::run_real_device_wrapper(invalid);
        assert_eq!(
            exit,
            RealDeviceWrapperExit::EvidenceValidationFailure.code()
        );

        let record = browser_record();
        assert_eq!(
            record.preflight_outcome,
            REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY
        );
        assert!(record
            .base
            .non_claim_scope
            .contains(&ImplementationNonClaimScope::KernelCompletionNotClaimed));
    }
}
