//! real-device wrapper の exit mapping と evidence construction 境界です。
#![allow(dead_code)]

use arcrtc_distro_evidence::{
    DistroCommandClass, DistroEnvironmentClass, DistroEvidenceReason,
    DistroEvidenceRecord, DistroLayer, DistroNonClaimScope,
    DistroPlane, DISTRO_COMMAND_ROOT, DISTRO_EVIDENCE_ROOT,
};

use crate::{
    command_output::{RealDeviceCommandExitStatus, RealDeviceCommandOutput},
    device_observation::RealDeviceObservation,
    dispatch::RealDeviceDispatch,
    evidence::{
        validate_real_device_evidence_record, RealDeviceEvidenceRecord,
        RealDeviceEvidenceValidationError, REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED,
        REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY,
        REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE,
        REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
    },
};

/// wrapper の閉じた exit 分類です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealDeviceWrapperExit {
    /// wrapper-local command 成功です。
    Success,
    /// scope mismatch です。
    ScopeMismatch,
    /// platform command は許可済みだが、この bounded wrapper では実行しません。
    PlatformCommandUnavailable,
    /// evidence validation failure です。
    EvidenceValidationFailure,
    /// command scope mismatch です。
    CommandScopeMismatch,
}

impl RealDeviceWrapperExit {
    /// process exit code へ変換します。
    pub const fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::ScopeMismatch => 2,
            Self::PlatformCommandUnavailable => 3,
            Self::EvidenceValidationFailure => 4,
            Self::CommandScopeMismatch => 5,
        }
    }
}

impl From<RealDeviceCommandExitStatus> for RealDeviceWrapperExit {
    fn from(value: RealDeviceCommandExitStatus) -> Self {
        match value {
            RealDeviceCommandExitStatus::Success => Self::Success,
            RealDeviceCommandExitStatus::RequiredDeviceNotObserved => Self::ScopeMismatch,
            RealDeviceCommandExitStatus::PlatformCommandUnavailable => {
                Self::PlatformCommandUnavailable
            }
            RealDeviceCommandExitStatus::EvidenceFieldsIncomplete => {
                Self::EvidenceValidationFailure
            }
            RealDeviceCommandExitStatus::CommandScopeMismatch => Self::CommandScopeMismatch,
        }
    }
}

/// wrapper exit code の閉集合です。
pub const fn all_wrapper_exit_codes() -> [i32; 5] {
    [
        RealDeviceWrapperExit::Success.code(),
        RealDeviceWrapperExit::ScopeMismatch.code(),
        RealDeviceWrapperExit::PlatformCommandUnavailable.code(),
        RealDeviceWrapperExit::EvidenceValidationFailure.code(),
        RealDeviceWrapperExit::CommandScopeMismatch.code(),
    ]
}

/// 現在の working directory が distro root かを検査します。
pub fn is_current_working_directory_distro_root() -> bool {
    std::env::current_dir()
        .ok()
        .is_some_and(|path| path.ends_with(DISTRO_COMMAND_ROOT))
}

/// dispatch 結果から wrapper の予定 exit を決めます。
pub const fn planned_exit_for_dispatch(dispatch: &RealDeviceDispatch) -> RealDeviceWrapperExit {
    if dispatch.platform_command.is_some() {
        RealDeviceWrapperExit::PlatformCommandUnavailable
    } else {
        RealDeviceWrapperExit::Success
    }
}

/// planned exit に対応する実装 reason です。
pub const fn distro_reason_for_exit(
    planned_exit: RealDeviceWrapperExit,
) -> DistroEvidenceReason {
    match planned_exit {
        RealDeviceWrapperExit::Success => DistroEvidenceReason::DistroOk,
        RealDeviceWrapperExit::ScopeMismatch => {
            DistroEvidenceReason::RealDeviceScopeMismatch
        }
        RealDeviceWrapperExit::PlatformCommandUnavailable => {
            DistroEvidenceReason::RuntimeExecutorError
        }
        RealDeviceWrapperExit::EvidenceValidationFailure => {
            DistroEvidenceReason::EvidenceFieldsIncomplete
        }
        RealDeviceWrapperExit::CommandScopeMismatch => {
            DistroEvidenceReason::CommandScopeMismatch
        }
    }
}

/// planned exit に対応する actual outcome です。
pub const fn actual_outcome_for_exit(planned_exit: RealDeviceWrapperExit) -> &'static str {
    match planned_exit {
        RealDeviceWrapperExit::Success => "wrapper-local capability recorded",
        RealDeviceWrapperExit::ScopeMismatch => "platform and device class scope mismatch",
        RealDeviceWrapperExit::PlatformCommandUnavailable => {
            "platform command unavailable in bounded wrapper"
        }
        RealDeviceWrapperExit::EvidenceValidationFailure => "evidence validation failed",
        RealDeviceWrapperExit::CommandScopeMismatch => {
            "command working directory outside distro"
        }
    }
}

/// dispatch context に対するpreflight outcomeの閉集合です。
pub const fn preflight_outcome_for_dispatch(dispatch: &RealDeviceDispatch) -> &'static str {
    if dispatch.platform_command.is_some() {
        REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE
    } else {
        REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY
    }
}

/// dispatch と profile から evidence record を作ります。
pub fn build_real_device_record(
    dispatch: RealDeviceDispatch,
    profile: String,
    planned_exit: RealDeviceWrapperExit,
) -> RealDeviceEvidenceRecord {
    let exit_status = planned_exit.code();
    RealDeviceEvidenceRecord {
        base: DistroEvidenceRecord {
            correlation_id: "real-device-wrapper-preflight".to_owned(),
            command: "cargo run --manifest-path tests/real-device/Cargo.toml -- <platform> --profile reference-local --device-class <device-class>".to_owned(),
            working_directory: DISTRO_COMMAND_ROOT.to_owned(),
            target_package: Some("arcrtc-distro-real-device-tests".to_owned()),
            target_scope: "tests/real-device".to_owned(),
            command_class: DistroCommandClass::RealDevice,
            distro_layer: DistroLayer::RealDevice,
            target_plane: DistroPlane::Ops,
            environment_class: DistroEnvironmentClass::RealDeviceBounded,
            toolchain_runtime_version: "rustc 1.96".to_owned(),
            input_fixture_or_workload: Some(profile),
            expected_outcome: "wrapper records command exit status and evidence JSON".to_owned(),
            actual_outcome: actual_outcome_for_exit(planned_exit).to_owned(),
            exit_status: Some(exit_status),
            kernel_reason: None,
            distro_reason: distro_reason_for_exit(planned_exit),
            non_claim_scope: vec![
                DistroNonClaimScope::NativeApplicationReadinessNotClaimed,
                DistroNonClaimScope::PublicDistributionReadinessNotClaimed,
                DistroNonClaimScope::ProductionReadinessNotClaimed,
                DistroNonClaimScope::LiveReadinessNotClaimed,
                DistroNonClaimScope::KernelCompletionNotClaimed,
            ],
            rerun_condition: "rerun when wrapper command or device class changes".to_owned(),
        },
        platform: dispatch.platform,
        device_class: dispatch.device_class,
        internal_command_class: dispatch.internal_command_class,
        runtime_version_class: dispatch.runtime_version_class,
        execution_surface: dispatch.execution_surface,
        network_class: dispatch.network_class,
        platform_command: dispatch.platform_command.map(str::to_owned),
        preflight_outcome: preflight_outcome_for_dispatch(&dispatch).to_owned(),
        logs_metrics_location: format!("{DISTRO_EVIDENCE_ROOT}/real-device/wrapper.json"),
        redacted_device_identifier: (exit_status == 0).then(|| "redacted".to_owned()),
    }
}

/// KPI-T7 の bounded real-device evidence record builder 境界です。
pub fn build_kpi_bounded_real_device_record(
    dispatch: RealDeviceDispatch,
    profile: String,
    planned_exit: RealDeviceWrapperExit,
) -> RealDeviceEvidenceRecord {
    build_real_device_record(dispatch, profile, planned_exit)
}

/// KPI-T11 の real-device success record を組み立てます。
pub fn build_kpi_real_device_success_record(
    dispatch: RealDeviceDispatch,
    profile: String,
    command_output: RealDeviceCommandOutput,
    observation: Option<RealDeviceObservation>,
) -> RealDeviceEvidenceRecord {
    let exit_status = command_output.exit_status.code();
    let redacted_device_identifier = observation
        .as_ref()
        .map(|observation| observation.redacted_device_identifier.clone());
    let toolchain_runtime_version = observation
        .as_ref()
        .map(|observation| observation.toolchain_runtime_version.clone())
        .unwrap_or_else(|| command_output.toolchain_runtime_version.clone());
    RealDeviceEvidenceRecord {
        base: DistroEvidenceRecord {
            correlation_id: correlation_id_for_dispatch(&dispatch).to_owned(),
            command: wrapper_command_for_dispatch(&dispatch).to_owned(),
            working_directory: DISTRO_COMMAND_ROOT.to_owned(),
            target_package: Some("arcrtc-distro-real-device-tests".to_owned()),
            target_scope: "tests/real-device".to_owned(),
            command_class: DistroCommandClass::RealDevice,
            distro_layer: DistroLayer::RealDevice,
            target_plane: DistroPlane::Ops,
            environment_class: DistroEnvironmentClass::RealDeviceBounded,
            toolchain_runtime_version,
            input_fixture_or_workload: Some(profile),
            expected_outcome: "six required real-device wrapper row records exit 0 evidence"
                .to_owned(),
            actual_outcome: actual_outcome_for_command_output(command_output.exit_status)
                .to_owned(),
            exit_status: Some(exit_status),
            kernel_reason: None,
            distro_reason: command_output.exit_status.distro_reason(),
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
        platform: dispatch.platform,
        device_class: dispatch.device_class,
        internal_command_class: dispatch.internal_command_class,
        runtime_version_class: dispatch.runtime_version_class,
        execution_surface: dispatch.execution_surface,
        network_class: dispatch.network_class,
        platform_command: dispatch.platform_command.map(str::to_owned),
        preflight_outcome: success_outcome_for_dispatch(&dispatch, command_output.exit_status)
            .to_owned(),
        logs_metrics_location: format!(
            "{DISTRO_EVIDENCE_ROOT}/real-device/{}.json",
            evidence_slug_for_dispatch(&dispatch)
        ),
        redacted_device_identifier,
    }
}

/// evidence validation result を最終 exit code に変換します。
pub fn validated_exit_code(
    record: &RealDeviceEvidenceRecord,
    planned_exit: RealDeviceWrapperExit,
) -> Result<i32, RealDeviceEvidenceValidationError> {
    validate_real_device_evidence_record(record)?;
    Ok(planned_exit.code())
}

const fn actual_outcome_for_command_output(
    exit_status: RealDeviceCommandExitStatus,
) -> &'static str {
    match exit_status {
        RealDeviceCommandExitStatus::Success => {
            "real-device success evidence row observed with raw identifier absent"
        }
        RealDeviceCommandExitStatus::RequiredDeviceNotObserved => {
            "required real-device class not observed"
        }
        RealDeviceCommandExitStatus::PlatformCommandUnavailable => {
            "platform command unavailable for real-device success row"
        }
        RealDeviceCommandExitStatus::EvidenceFieldsIncomplete => {
            "real-device success evidence fields incomplete"
        }
        RealDeviceCommandExitStatus::CommandScopeMismatch => {
            "real-device wrapper command scope mismatch"
        }
    }
}

const fn success_outcome_for_dispatch(
    dispatch: &RealDeviceDispatch,
    exit_status: RealDeviceCommandExitStatus,
) -> &'static str {
    match (dispatch.platform_command.is_some(), exit_status) {
        (true, RealDeviceCommandExitStatus::Success)
        | (true, RealDeviceCommandExitStatus::RequiredDeviceNotObserved) => {
            REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED
        }
        (true, _) => REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE,
        (false, RealDeviceCommandExitStatus::Success) => {
            REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY
        }
        (false, _) => REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
    }
}

fn wrapper_command_for_dispatch(dispatch: &RealDeviceDispatch) -> &'static str {
    match dispatch.device_class {
        crate::evidence::RealDeviceClass::AndroidPhysical => "cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-physical",
        crate::evidence::RealDeviceClass::AndroidEmulator => "cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-emulator",
        crate::evidence::RealDeviceClass::IosPhysical => "cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-physical",
        crate::evidence::RealDeviceClass::IosSimulator => "cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-simulator",
        crate::evidence::RealDeviceClass::DesktopBrowser => "cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser",
        crate::evidence::RealDeviceClass::MobileBrowser => "cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class mobile-browser",
    }
}

fn correlation_id_for_dispatch(dispatch: &RealDeviceDispatch) -> &'static str {
    match dispatch.device_class {
        crate::evidence::RealDeviceClass::AndroidPhysical => "KPI-011-ANDROID-PHYSICAL",
        crate::evidence::RealDeviceClass::AndroidEmulator => "KPI-011-ANDROID-EMULATOR",
        crate::evidence::RealDeviceClass::IosPhysical => "KPI-011-IOS-PHYSICAL",
        crate::evidence::RealDeviceClass::IosSimulator => "KPI-011-IOS-SIMULATOR",
        crate::evidence::RealDeviceClass::DesktopBrowser => "KPI-011-BROWSER-DESKTOP",
        crate::evidence::RealDeviceClass::MobileBrowser => "KPI-011-BROWSER-MOBILE",
    }
}

fn evidence_slug_for_dispatch(dispatch: &RealDeviceDispatch) -> &'static str {
    match dispatch.device_class {
        crate::evidence::RealDeviceClass::AndroidPhysical => "android-physical",
        crate::evidence::RealDeviceClass::AndroidEmulator => "android-emulator",
        crate::evidence::RealDeviceClass::IosPhysical => "ios-physical",
        crate::evidence::RealDeviceClass::IosSimulator => "ios-simulator",
        crate::evidence::RealDeviceClass::DesktopBrowser => "browser-desktop",
        crate::evidence::RealDeviceClass::MobileBrowser => "browser-mobile",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        command_output::{RealDeviceCommandExitStatus, RealDeviceCommandOutput},
        dispatch::dispatch_real_device_command,
        evidence::{RealDeviceClass, REAL_DEVICE_PROFILE_REFERENCE_LOCAL},
    };

    fn dispatch_for(device_class: RealDeviceClass) -> RealDeviceDispatch {
        match device_class {
            RealDeviceClass::AndroidPhysical => dispatch_real_device_command(
                crate::cli::CliPlatform::Android,
                crate::cli::CliDeviceClass::AndroidPhysical,
            ),
            RealDeviceClass::AndroidEmulator => dispatch_real_device_command(
                crate::cli::CliPlatform::Android,
                crate::cli::CliDeviceClass::AndroidEmulator,
            ),
            RealDeviceClass::IosPhysical => dispatch_real_device_command(
                crate::cli::CliPlatform::Ios,
                crate::cli::CliDeviceClass::IosPhysical,
            ),
            RealDeviceClass::IosSimulator => dispatch_real_device_command(
                crate::cli::CliPlatform::Ios,
                crate::cli::CliDeviceClass::IosSimulator,
            ),
            RealDeviceClass::DesktopBrowser => dispatch_real_device_command(
                crate::cli::CliPlatform::Browser,
                crate::cli::CliDeviceClass::DesktopBrowser,
            ),
            RealDeviceClass::MobileBrowser => dispatch_real_device_command(
                crate::cli::CliPlatform::Browser,
                crate::cli::CliDeviceClass::MobileBrowser,
            ),
        }
        .expect("dispatch must be admitted")
    }

    #[test]
    fn wrapper_unit_covers_bounded_record_builder_and_dispatch_mappings() {
        for (class, command_suffix, correlation, slug) in [
            (
                RealDeviceClass::AndroidPhysical,
                "android-physical",
                "KPI-011-ANDROID-PHYSICAL",
                "android-physical",
            ),
            (
                RealDeviceClass::AndroidEmulator,
                "android-emulator",
                "KPI-011-ANDROID-EMULATOR",
                "android-emulator",
            ),
            (
                RealDeviceClass::IosPhysical,
                "ios-physical",
                "KPI-011-IOS-PHYSICAL",
                "ios-physical",
            ),
            (
                RealDeviceClass::IosSimulator,
                "ios-simulator",
                "KPI-011-IOS-SIMULATOR",
                "ios-simulator",
            ),
            (
                RealDeviceClass::DesktopBrowser,
                "desktop-browser",
                "KPI-011-BROWSER-DESKTOP",
                "browser-desktop",
            ),
            (
                RealDeviceClass::MobileBrowser,
                "mobile-browser",
                "KPI-011-BROWSER-MOBILE",
                "browser-mobile",
            ),
        ] {
            let dispatch = dispatch_for(class);
            assert!(wrapper_command_for_dispatch(&dispatch).contains(command_suffix));
            assert_eq!(correlation_id_for_dispatch(&dispatch), correlation);
            assert_eq!(evidence_slug_for_dispatch(&dispatch), slug);

            let record = build_kpi_bounded_real_device_record(
                dispatch.clone(),
                REAL_DEVICE_PROFILE_REFERENCE_LOCAL.to_owned(),
                planned_exit_for_dispatch(&dispatch),
            );
            assert_eq!(
                validated_exit_code(&record, planned_exit_for_dispatch(&dispatch)),
                Ok(planned_exit_for_dispatch(&dispatch).code())
            );
        }
    }

    #[test]
    fn wrapper_unit_covers_success_record_outcome_mappings() {
        let browser = dispatch_for(RealDeviceClass::DesktopBrowser);
        for (status, expected_outcome, expected_preflight) in [
            (
                RealDeviceCommandExitStatus::Success,
                "real-device success evidence row observed with raw identifier absent",
                REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY,
            ),
            (
                RealDeviceCommandExitStatus::RequiredDeviceNotObserved,
                "required real-device class not observed",
                REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
            ),
            (
                RealDeviceCommandExitStatus::PlatformCommandUnavailable,
                "platform command unavailable for real-device success row",
                REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
            ),
            (
                RealDeviceCommandExitStatus::EvidenceFieldsIncomplete,
                "real-device success evidence fields incomplete",
                REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
            ),
            (
                RealDeviceCommandExitStatus::CommandScopeMismatch,
                "real-device wrapper command scope mismatch",
                REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
            ),
        ] {
            let output = match status {
                RealDeviceCommandExitStatus::Success => {
                    RealDeviceCommandOutput::success("", "", "browser runtime")
                }
                RealDeviceCommandExitStatus::RequiredDeviceNotObserved => {
                    RealDeviceCommandOutput::success("", "", "browser runtime")
                        .required_device_not_observed()
                }
                RealDeviceCommandExitStatus::PlatformCommandUnavailable => {
                    RealDeviceCommandOutput::unavailable(
                        crate::command_output::RealDeviceCommandUnavailable::MissingProgram,
                        "",
                        "",
                        None,
                        "browser runtime",
                    )
                }
                RealDeviceCommandExitStatus::EvidenceFieldsIncomplete => {
                    RealDeviceCommandOutput::evidence_fields_incomplete("browser runtime")
                }
                RealDeviceCommandExitStatus::CommandScopeMismatch => RealDeviceCommandOutput {
                    exit_status: RealDeviceCommandExitStatus::CommandScopeMismatch,
                    raw_exit_status: None,
                    stdout_summary: String::new(),
                    stderr_summary: String::new(),
                    toolchain_runtime_version: "browser runtime".to_owned(),
                    unavailable: None,
                },
            };
            let record = build_kpi_real_device_success_record(
                browser.clone(),
                REAL_DEVICE_PROFILE_REFERENCE_LOCAL.to_owned(),
                output,
                None,
            );
            assert_eq!(record.base.actual_outcome, expected_outcome);
            assert_eq!(record.preflight_outcome, expected_preflight);
        }
    }
}
