//! base evidence record のvalidation境界です。

use crate::record::{
    ImplementationCommandClass, ImplementationEvidenceRecord, ImplementationNonClaimScope,
};

/// implementations command の固定root（repository-root 相対）です。
pub const IMPLEMENTATIONS_COMMAND_ROOT: &str = "implementations";

/// implementations command が証跡を書き込める固定target root（repository-root 相対）です。
pub const IMPLEMENTATIONS_TARGET_ROOT: &str = "implementations/target";

/// implementations command evidence の固定root（repository-root 相対）です。
pub const IMPLEMENTATIONS_EVIDENCE_ROOT: &str = "implementations/target/implementation-evidence";

/// base evidence validation の閉集合errorです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceValidationError {
    /// correlation id が空です。
    EmptyCorrelationId,
    /// command が空です。
    EmptyCommand,
    /// working directory が空です。
    EmptyWorkingDirectory,
    /// working directory が implementations root 外です。
    WorkingDirectoryOutsideImplementations,
    /// target scope が空です。
    EmptyTargetScope,
    /// toolchain runtime version が空です。
    EmptyToolchainRuntimeVersion,
    /// expected outcome が空です。
    EmptyExpectedOutcome,
    /// actual outcome が空です。
    EmptyActualOutcome,
    /// rerun condition が空です。
    EmptyRerunCondition,
    /// non-claim scope が空です。
    EmptyNonClaimScope,
    /// command class が要求する target package がありません。
    MissingRequiredTargetPackage,
    /// command class が要求する fixture/workload がありません。
    MissingRequiredInputFixtureOrWorkload,
    /// command class が要求する exit status がありません。
    MissingRequiredExitStatus,
    /// command class が要求する non-claim scope がありません。
    MissingRequiredNonClaimScope,
    /// readiness admission が不足しています。
    InvalidReadinessAdmission,
    /// secret-like value または raw payload marker が含まれています。
    RawSecretLikeValue,
    /// 未分類 reason sentinel が含まれています。
    ForbiddenUnclassifiedReason,
}

/// base evidence record を検証します。
pub fn validate_evidence_record(
    record: &ImplementationEvidenceRecord,
) -> Result<(), EvidenceValidationError> {
    if record.correlation_id.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyCorrelationId);
    }
    if record.command.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyCommand);
    }
    if record.working_directory.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyWorkingDirectory);
    }
    if !is_under_implementations_root(&record.working_directory) {
        return Err(EvidenceValidationError::WorkingDirectoryOutsideImplementations);
    }
    if record.target_scope.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyTargetScope);
    }
    if record.toolchain_runtime_version.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyToolchainRuntimeVersion);
    }
    if record.expected_outcome.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyExpectedOutcome);
    }
    if record.actual_outcome.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyActualOutcome);
    }
    if record.rerun_condition.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyRerunCondition);
    }
    if record.non_claim_scope.is_empty() {
        return Err(EvidenceValidationError::EmptyNonClaimScope);
    }
    validate_command_class_required_fields(record)?;
    validate_required_non_claim_scope(record)?;
    validate_no_unclassified_reason(record)?;
    validate_no_secret_like_values(record)?;
    Ok(())
}

fn validate_command_class_required_fields(
    record: &ImplementationEvidenceRecord,
) -> Result<(), EvidenceValidationError> {
    if requires_target_package(record.command_class)
        && is_empty_option(record.target_package.as_deref())
    {
        return Err(EvidenceValidationError::MissingRequiredTargetPackage);
    }
    if requires_input_fixture_or_workload(record.command_class)
        && is_empty_option(record.input_fixture_or_workload.as_deref())
    {
        return Err(EvidenceValidationError::MissingRequiredInputFixtureOrWorkload);
    }
    if record.exit_status.is_none() {
        return Err(EvidenceValidationError::MissingRequiredExitStatus);
    }
    Ok(())
}

const fn requires_target_package(command_class: ImplementationCommandClass) -> bool {
    matches!(
        command_class,
        ImplementationCommandClass::Build
            | ImplementationCommandClass::Test
            | ImplementationCommandClass::Benchmark
            | ImplementationCommandClass::RealDevice
            | ImplementationCommandClass::ProductionReadiness
            | ImplementationCommandClass::LiveReadiness
    )
}

const fn requires_input_fixture_or_workload(command_class: ImplementationCommandClass) -> bool {
    matches!(
        command_class,
        ImplementationCommandClass::Test
            | ImplementationCommandClass::Benchmark
            | ImplementationCommandClass::RealDevice
            | ImplementationCommandClass::ProductionReadiness
            | ImplementationCommandClass::LiveReadiness
    )
}

fn validate_required_non_claim_scope(
    record: &ImplementationEvidenceRecord,
) -> Result<(), EvidenceValidationError> {
    let required: &[ImplementationNonClaimScope] = match record.command_class {
        ImplementationCommandClass::Format => {
            &[ImplementationNonClaimScope::CommandTargetSuccessNotClaimed]
        }
        ImplementationCommandClass::Build => &[
            ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        ImplementationCommandClass::Test => &[
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        ImplementationCommandClass::Benchmark => &[
            ImplementationNonClaimScope::BenchmarkThresholdNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        ImplementationCommandClass::RealDevice => &[
            ImplementationNonClaimScope::NativeApplicationReadinessNotClaimed,
            ImplementationNonClaimScope::PublicDistributionReadinessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        ImplementationCommandClass::ProductionReadiness => &[
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
        ImplementationCommandClass::LiveReadiness => &[
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
    };

    for required_scope in required {
        if !record.non_claim_scope.contains(required_scope) {
            return Err(EvidenceValidationError::MissingRequiredNonClaimScope);
        }
    }
    Ok(())
}

fn validate_no_secret_like_values(
    record: &ImplementationEvidenceRecord,
) -> Result<(), EvidenceValidationError> {
    let required_fields = [
        record.correlation_id.as_str(),
        record.command.as_str(),
        record.working_directory.as_str(),
        record.target_scope.as_str(),
        record.toolchain_runtime_version.as_str(),
        record.expected_outcome.as_str(),
        record.actual_outcome.as_str(),
        record.rerun_condition.as_str(),
    ];
    for value in required_fields {
        reject_secret_like(value)?;
    }
    for value in [
        record.target_package.as_deref(),
        record.input_fixture_or_workload.as_deref(),
        record.kernel_reason.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        reject_secret_like(value)?;
    }
    Ok(())
}

fn validate_no_unclassified_reason(
    record: &ImplementationEvidenceRecord,
) -> Result<(), EvidenceValidationError> {
    if let Some(kernel_reason) = record.kernel_reason.as_deref() {
        reject_unclassified_reason(kernel_reason)?;
    }
    Ok(())
}

fn reject_unclassified_reason(value: &str) -> Result<(), EvidenceValidationError> {
    // Kernel reason catalog は implementations 側で所有しないが、
    // evidence として未分類 sentinel を採用することはここで fail-closed にする。
    let normalized = value.trim().to_ascii_lowercase();
    if matches!(normalized.as_str(), "unknown" | "other") {
        return Err(EvidenceValidationError::ForbiddenUnclassifiedReason);
    }
    Ok(())
}

pub(crate) fn reject_secret_like(value: &str) -> Result<(), EvidenceValidationError> {
    let lower = value.to_ascii_lowercase();
    for marker in [
        "secret=",
        "token=",
        "private_key",
        "-----begin",
        "packet_payload=",
        "raw_packet=",
        "authorization:",
        "bearer ",
    ] {
        if lower.contains(marker) {
            return Err(EvidenceValidationError::RawSecretLikeValue);
        }
    }
    Ok(())
}

fn is_empty_option(value: Option<&str>) -> bool {
    value.map(str::trim).unwrap_or_default().is_empty()
}

fn is_under_implementations_root(value: &str) -> bool {
    value.trim() == IMPLEMENTATIONS_COMMAND_ROOT
}
