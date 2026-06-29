//! real-device evidence extension record と validation 境界です。
#![allow(dead_code)]

use arcrtc_implementation_evidence::{
    validate_evidence_record, EvidenceValidationError, ImplementationCommandClass,
    ImplementationEvidenceRecord, ImplementationNonClaimScope, IMPLEMENTATIONS_EVIDENCE_ROOT,
};

/// admitted profile の閉集合です。
pub const REAL_DEVICE_PROFILE_REFERENCE_LOCAL: &str = "reference-local";

/// platform command をwrapper内で実行しないpreflight結果です。
pub const REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE: &str =
    "preflight:platform-command-admitted-unavailable";

/// browser wrapper-local capability のpreflight結果です。
pub const REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY: &str =
    "preflight:wrapper-local-capability-admitted";

/// platform command を実行した結果です。
pub const REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED: &str =
    "execution:platform-command-executed";

/// browser wrapper-local capability を実行結果として記録した状態です。
pub const REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY: &str =
    "execution:wrapper-local-capability-recorded";

/// real-device platform closed set です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RealDevicePlatform {
    /// Android platform です。
    Android,
    /// iOS platform です。
    Ios,
    /// browser platform です。
    Browser,
}

/// real-device class closed set です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RealDeviceClass {
    /// Android physical device です。
    AndroidPhysical,
    /// Android emulator です。
    AndroidEmulator,
    /// iOS physical device です。
    IosPhysical,
    /// iOS simulator です。
    IosSimulator,
    /// desktop browser です。
    DesktopBrowser,
    /// mobile browser です。
    MobileBrowser,
}

/// internal command class closed set です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealDeviceInternalCommandClass {
    /// Android inventory command です。
    AndroidDevice,
    /// iOS physical inventory command です。
    IosDevice,
    /// iOS simulator inventory command です。
    IosSimulator,
    /// browser inventory command です。
    Browser,
}

/// runtime version class closed set です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealDeviceVersionClass {
    /// Android API level です。
    AndroidApiLevel,
    /// iOS system version です。
    IosSystemVersion,
    /// browser version です。
    BrowserVersion,
}

/// execution surface closed set です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealDeviceExecutionSurface {
    /// native SDK command です。
    NativeSdkCommand,
    /// browser WebRTC command です。
    BrowserWebrtcCommand,
    /// wrapper local capability です。
    WrapperLocalCapability,
}

/// network class closed set です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealDeviceNetworkClass {
    /// local USB です。
    LocalUsb,
    /// emulator loopback です。
    EmulatorLoopback,
    /// simulator loopback です。
    SimulatorLoopback,
    /// local browser です。
    LocalBrowser,
    /// mobile browser emulation です。
    MobileBrowserEmulation,
}

/// real-device evidence extension record です。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RealDeviceEvidenceRecord {
    /// base evidence recordです。
    #[serde(flatten)]
    pub base: ImplementationEvidenceRecord,
    /// platform です。
    pub platform: RealDevicePlatform,
    /// device class です。
    pub device_class: RealDeviceClass,
    /// internal command class です。
    pub internal_command_class: RealDeviceInternalCommandClass,
    /// runtime version class です。
    pub runtime_version_class: RealDeviceVersionClass,
    /// execution surface です。
    pub execution_surface: RealDeviceExecutionSurface,
    /// network class です。
    pub network_class: RealDeviceNetworkClass,
    /// platform command です。
    pub platform_command: Option<String>,
    /// preflight outcome です。
    pub preflight_outcome: String,
    /// logs / metrics location です。
    pub logs_metrics_location: String,
    /// redacted device identifier です。
    pub redacted_device_identifier: Option<String>,
}

/// real-device evidence validation error です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RealDeviceEvidenceValidationError {
    /// base validation errorです。
    Base(EvidenceValidationError),
    /// command class mismatch です。
    CommandClassMismatch,
    /// platform / device class mismatch です。
    PlatformDeviceClassMismatch,
    /// internal command class mismatch です。
    InternalCommandClassMismatch,
    /// runtime version class mismatch です。
    RuntimeVersionClassMismatch,
    /// execution surface mismatch です。
    ExecutionSurfaceMismatch,
    /// network class mismatch です。
    NetworkClassMismatch,
    /// platform command missing です。
    PlatformCommandMissing,
    /// platform command unexpected です。
    PlatformCommandUnexpected,
    /// platform command not admitted です。
    PlatformCommandNotAdmitted,
    /// post execution field missing です。
    MissingPostExecutionField,
    /// logs metrics location empty です。
    EmptyLogsMetricsLocation,
    /// logs metrics location outside evidence dir です。
    LogsMetricsLocationOutsideEvidenceDir,
    /// device identifier empty です。
    EmptyDeviceIdentifier,
    /// device identifier format mismatch です。
    DeviceIdentifierFormatMismatch,
    /// device identifier unexpected です。
    UnexpectedDeviceIdentifier,
    /// unsafe device identifier です。
    UnsafeDeviceIdentifier,
    /// preflight outcome が閉集合外です。
    PreflightOutcomeNotAdmitted,
    /// profile が閉集合外です。
    ProfileNotAdmitted,
}

/// wrapper profile を閉集合で検証します。
pub fn validate_real_device_profile(
    profile: &str,
) -> Result<(), RealDeviceEvidenceValidationError> {
    if profile == REAL_DEVICE_PROFILE_REFERENCE_LOCAL {
        Ok(())
    } else {
        Err(RealDeviceEvidenceValidationError::ProfileNotAdmitted)
    }
}

/// real-device evidence record を検証します。
pub fn validate_real_device_evidence_record(
    record: &RealDeviceEvidenceRecord,
) -> Result<(), RealDeviceEvidenceValidationError> {
    validate_evidence_record(&record.base).map_err(RealDeviceEvidenceValidationError::Base)?;
    if record.base.command_class != ImplementationCommandClass::RealDevice {
        return Err(RealDeviceEvidenceValidationError::CommandClassMismatch);
    }
    for required in [
        ImplementationNonClaimScope::NativeApplicationReadinessNotClaimed,
        ImplementationNonClaimScope::PublicDistributionReadinessNotClaimed,
        ImplementationNonClaimScope::ProductionReadinessNotClaimed,
        ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ImplementationNonClaimScope::KernelCompletionNotClaimed,
    ] {
        if !record.base.non_claim_scope.contains(&required) {
            return Err(RealDeviceEvidenceValidationError::MissingPostExecutionField);
        }
    }
    validate_real_device_profile(
        record
            .base
            .input_fixture_or_workload
            .as_deref()
            .unwrap_or_default(),
    )?;
    validate_context(record)?;
    validate_post_execution_fields(record)?;
    Ok(())
}

/// KPI-T7 の bounded real-device evidence validation 境界です。
pub fn validate_kpi_bounded_real_device_evidence(
    record: &RealDeviceEvidenceRecord,
) -> Result<(), RealDeviceEvidenceValidationError> {
    validate_real_device_evidence_record(record)
}

/// KPI-T11 の real-device success evidence record builder 境界です。
pub fn build_kpi_real_device_success_evidence_record(
    record: RealDeviceEvidenceRecord,
) -> Result<RealDeviceEvidenceRecord, RealDeviceEvidenceValidationError> {
    validate_real_device_evidence_record(&record)?;
    Ok(record)
}

fn validate_context(
    record: &RealDeviceEvidenceRecord,
) -> Result<(), RealDeviceEvidenceValidationError> {
    let expected = expected_context(record.device_class)
        .ok_or(RealDeviceEvidenceValidationError::PlatformDeviceClassMismatch)?;
    if record.platform != expected.0 {
        return Err(RealDeviceEvidenceValidationError::PlatformDeviceClassMismatch);
    }
    if record.internal_command_class != expected.1 {
        return Err(RealDeviceEvidenceValidationError::InternalCommandClassMismatch);
    }
    if record.runtime_version_class != expected.2 {
        return Err(RealDeviceEvidenceValidationError::RuntimeVersionClassMismatch);
    }
    if record.execution_surface != expected.3 {
        return Err(RealDeviceEvidenceValidationError::ExecutionSurfaceMismatch);
    }
    if record.network_class != expected.4 {
        return Err(RealDeviceEvidenceValidationError::NetworkClassMismatch);
    }
    match (record.platform, record.platform_command.as_deref()) {
        (RealDevicePlatform::Android, Some("adb devices -l"))
        | (RealDevicePlatform::Ios, Some("xcrun xctrace list devices"))
        | (RealDevicePlatform::Ios, Some("xcrun simctl list devices")) => Ok(()),
        (RealDevicePlatform::Browser, None) => Ok(()),
        (RealDevicePlatform::Android | RealDevicePlatform::Ios, None) => {
            Err(RealDeviceEvidenceValidationError::PlatformCommandMissing)
        }
        (RealDevicePlatform::Browser, Some(_)) => {
            Err(RealDeviceEvidenceValidationError::PlatformCommandUnexpected)
        }
        _ => Err(RealDeviceEvidenceValidationError::PlatformCommandNotAdmitted),
    }
}

fn expected_context(
    device_class: RealDeviceClass,
) -> Option<(
    RealDevicePlatform,
    RealDeviceInternalCommandClass,
    RealDeviceVersionClass,
    RealDeviceExecutionSurface,
    RealDeviceNetworkClass,
)> {
    match device_class {
        RealDeviceClass::AndroidPhysical => Some((
            RealDevicePlatform::Android,
            RealDeviceInternalCommandClass::AndroidDevice,
            RealDeviceVersionClass::AndroidApiLevel,
            RealDeviceExecutionSurface::NativeSdkCommand,
            RealDeviceNetworkClass::LocalUsb,
        )),
        RealDeviceClass::AndroidEmulator => Some((
            RealDevicePlatform::Android,
            RealDeviceInternalCommandClass::AndroidDevice,
            RealDeviceVersionClass::AndroidApiLevel,
            RealDeviceExecutionSurface::NativeSdkCommand,
            RealDeviceNetworkClass::EmulatorLoopback,
        )),
        RealDeviceClass::IosPhysical => Some((
            RealDevicePlatform::Ios,
            RealDeviceInternalCommandClass::IosDevice,
            RealDeviceVersionClass::IosSystemVersion,
            RealDeviceExecutionSurface::NativeSdkCommand,
            RealDeviceNetworkClass::LocalUsb,
        )),
        RealDeviceClass::IosSimulator => Some((
            RealDevicePlatform::Ios,
            RealDeviceInternalCommandClass::IosSimulator,
            RealDeviceVersionClass::IosSystemVersion,
            RealDeviceExecutionSurface::NativeSdkCommand,
            RealDeviceNetworkClass::SimulatorLoopback,
        )),
        RealDeviceClass::DesktopBrowser => Some((
            RealDevicePlatform::Browser,
            RealDeviceInternalCommandClass::Browser,
            RealDeviceVersionClass::BrowserVersion,
            RealDeviceExecutionSurface::BrowserWebrtcCommand,
            RealDeviceNetworkClass::LocalBrowser,
        )),
        RealDeviceClass::MobileBrowser => Some((
            RealDevicePlatform::Browser,
            RealDeviceInternalCommandClass::Browser,
            RealDeviceVersionClass::BrowserVersion,
            RealDeviceExecutionSurface::BrowserWebrtcCommand,
            RealDeviceNetworkClass::MobileBrowserEmulation,
        )),
    }
}

fn validate_post_execution_fields(
    record: &RealDeviceEvidenceRecord,
) -> Result<(), RealDeviceEvidenceValidationError> {
    if record.preflight_outcome.trim().is_empty() {
        return Err(RealDeviceEvidenceValidationError::MissingPostExecutionField);
    }
    validate_preflight_outcome(record)?;
    if record.logs_metrics_location.trim().is_empty() {
        return Err(RealDeviceEvidenceValidationError::EmptyLogsMetricsLocation);
    }
    if !record
        .logs_metrics_location
        .starts_with(&format!("{IMPLEMENTATIONS_EVIDENCE_ROOT}/real-device/"))
    {
        return Err(RealDeviceEvidenceValidationError::LogsMetricsLocationOutsideEvidenceDir);
    }
    match (
        record.base.exit_status,
        record.redacted_device_identifier.as_deref(),
    ) {
        (Some(0), Some(identifier)) => validate_redacted_identifier(identifier),
        (Some(0), None) => Err(RealDeviceEvidenceValidationError::MissingPostExecutionField),
        (Some(_), Some(_)) => Err(RealDeviceEvidenceValidationError::UnexpectedDeviceIdentifier),
        (Some(_), None) => Ok(()),
        (None, _) => Err(RealDeviceEvidenceValidationError::MissingPostExecutionField),
    }
}

fn validate_preflight_outcome(
    record: &RealDeviceEvidenceRecord,
) -> Result<(), RealDeviceEvidenceValidationError> {
    match (
        record.platform_command.is_some(),
        record.preflight_outcome.as_str(),
    ) {
        (true, REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE)
        | (true, REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED)
        | (false, REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY)
        | (false, REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY) => Ok(()),
        _ => Err(RealDeviceEvidenceValidationError::PreflightOutcomeNotAdmitted),
    }
}

fn validate_redacted_identifier(identifier: &str) -> Result<(), RealDeviceEvidenceValidationError> {
    if identifier.trim().is_empty() {
        return Err(RealDeviceEvidenceValidationError::EmptyDeviceIdentifier);
    }
    if identifier == "redacted" {
        return Ok(());
    }
    if contains_raw_identifier_marker(identifier) {
        return Err(RealDeviceEvidenceValidationError::UnsafeDeviceIdentifier);
    }
    let Some(hash) = identifier.strip_prefix("sha256:") else {
        return Err(RealDeviceEvidenceValidationError::DeviceIdentifierFormatMismatch);
    };
    if hash.len() == 64
        && hash
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(RealDeviceEvidenceValidationError::DeviceIdentifierFormatMismatch)
    }
}

fn contains_raw_identifier_marker(identifier: &str) -> bool {
    let normalized = identifier.trim().to_ascii_lowercase();
    [
        "serial:",
        "serial=",
        "udid:",
        "udid=",
        "android_id:",
        "android_id=",
        "device_id:",
        "device_id=",
        "imei:",
        "imei=",
        "meid:",
        "meid=",
        "account:",
        "account=",
        "token:",
        "token=",
        "private_key:",
        "private_key=",
        "device_name:",
        "device_name=",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arcrtc_implementation_evidence::{
        ImplementationCommandClass, ImplementationEnvironmentClass, ImplementationEvidenceReason,
        ImplementationEvidenceRecord, ImplementationLayer, ImplementationNonClaimScope,
        ImplementationPlane, IMPLEMENTATIONS_COMMAND_ROOT,
    };

    fn base(exit_status: Option<i32>) -> ImplementationEvidenceRecord {
        ImplementationEvidenceRecord {
            correlation_id: "real-device-evidence-unit".to_owned(),
            command: "cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser".to_owned(),
            working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
            target_package: Some("arcrtc-implementation-real-device-tests".to_owned()),
            target_scope: "tests/real-device".to_owned(),
            command_class: ImplementationCommandClass::RealDevice,
            implementation_layer: ImplementationLayer::RealDevice,
            target_plane: ImplementationPlane::Ops,
            environment_class: ImplementationEnvironmentClass::RealDeviceBounded,
            toolchain_runtime_version: "real-device-unit-runtime".to_owned(),
            input_fixture_or_workload: Some(REAL_DEVICE_PROFILE_REFERENCE_LOCAL.to_owned()),
            expected_outcome: "unit evidence validates".to_owned(),
            actual_outcome: "unit evidence validates".to_owned(),
            exit_status,
            kernel_reason: None,
            implementation_reason: ImplementationEvidenceReason::ImplementationOk,
            non_claim_scope: vec![
                ImplementationNonClaimScope::NativeApplicationReadinessNotClaimed,
                ImplementationNonClaimScope::PublicDistributionReadinessNotClaimed,
                ImplementationNonClaimScope::ProductionReadinessNotClaimed,
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
                ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ],
            rerun_condition: "rerun when real-device evidence unit changes".to_owned(),
        }
    }

    fn browser_record(exit_status: Option<i32>) -> RealDeviceEvidenceRecord {
        RealDeviceEvidenceRecord {
            base: base(exit_status),
            platform: RealDevicePlatform::Browser,
            device_class: RealDeviceClass::DesktopBrowser,
            internal_command_class: RealDeviceInternalCommandClass::Browser,
            runtime_version_class: RealDeviceVersionClass::BrowserVersion,
            execution_surface: RealDeviceExecutionSurface::BrowserWebrtcCommand,
            network_class: RealDeviceNetworkClass::LocalBrowser,
            platform_command: None,
            preflight_outcome: REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY.to_owned(),
            logs_metrics_location: format!("{IMPLEMENTATIONS_EVIDENCE_ROOT}/real-device/unit.json"),
            redacted_device_identifier: (exit_status == Some(0)).then(|| "redacted".to_owned()),
        }
    }

    #[test]
    fn evidence_unit_covers_public_wrappers_and_required_scope_failures() {
        let record = browser_record(Some(0));
        assert_eq!(validate_kpi_bounded_real_device_evidence(&record), Ok(()));
        assert!(build_kpi_real_device_success_evidence_record(record.clone()).is_ok());

        let mut missing_scope = record.clone();
        missing_scope
            .base
            .non_claim_scope
            .retain(|scope| scope != &ImplementationNonClaimScope::KernelCompletionNotClaimed);
        assert_eq!(
            validate_real_device_evidence_record(&missing_scope),
            Err(RealDeviceEvidenceValidationError::MissingPostExecutionField)
        );

        let mut missing_profile = record;
        missing_profile.base.input_fixture_or_workload = Some("unsupported-profile".to_owned());
        assert_eq!(
            validate_real_device_evidence_record(&missing_profile),
            Err(RealDeviceEvidenceValidationError::ProfileNotAdmitted)
        );
    }

    #[test]
    fn evidence_unit_covers_context_and_post_execution_error_branches() {
        let mut missing_command = RealDeviceEvidenceRecord {
            platform: RealDevicePlatform::Android,
            device_class: RealDeviceClass::AndroidPhysical,
            internal_command_class: RealDeviceInternalCommandClass::AndroidDevice,
            runtime_version_class: RealDeviceVersionClass::AndroidApiLevel,
            execution_surface: RealDeviceExecutionSurface::NativeSdkCommand,
            network_class: RealDeviceNetworkClass::LocalUsb,
            platform_command: None,
            ..browser_record(Some(3))
        };
        missing_command.redacted_device_identifier = None;
        assert_eq!(
            validate_real_device_evidence_record(&missing_command),
            Err(RealDeviceEvidenceValidationError::PlatformCommandMissing)
        );

        let mut invalid_command = missing_command;
        invalid_command.platform_command = Some("adb shell getprop".to_owned());
        assert_eq!(
            validate_real_device_evidence_record(&invalid_command),
            Err(RealDeviceEvidenceValidationError::PlatformCommandNotAdmitted)
        );

        let mut outside_logs = browser_record(Some(0));
        outside_logs.logs_metrics_location = "/tmp/unit.json".to_owned();
        assert_eq!(
            validate_real_device_evidence_record(&outside_logs),
            Err(RealDeviceEvidenceValidationError::LogsMetricsLocationOutsideEvidenceDir)
        );

        let mut missing_exit = browser_record(None);
        missing_exit.redacted_device_identifier = Some("redacted".to_owned());
        assert_eq!(
            validate_real_device_evidence_record(&missing_exit),
            Err(RealDeviceEvidenceValidationError::Base(
                arcrtc_implementation_evidence::EvidenceValidationError::MissingRequiredExitStatus
            ))
        );

        let mut empty_identifier = browser_record(Some(0));
        empty_identifier.redacted_device_identifier = Some(" ".to_owned());
        assert_eq!(
            validate_real_device_evidence_record(&empty_identifier),
            Err(RealDeviceEvidenceValidationError::EmptyDeviceIdentifier)
        );

        let mut no_prefix = browser_record(Some(0));
        no_prefix.redacted_device_identifier = Some("safe-but-not-prefixed".to_owned());
        assert_eq!(
            validate_real_device_evidence_record(&no_prefix),
            Err(RealDeviceEvidenceValidationError::DeviceIdentifierFormatMismatch)
        );

        let mut invalid_hash = browser_record(Some(0));
        invalid_hash.redacted_device_identifier = Some(format!("sha256:{}", "z".repeat(64)));
        assert_eq!(
            validate_real_device_evidence_record(&invalid_hash),
            Err(RealDeviceEvidenceValidationError::DeviceIdentifierFormatMismatch)
        );
    }
}
