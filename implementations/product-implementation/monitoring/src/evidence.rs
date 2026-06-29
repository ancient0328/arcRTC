//! product evidence record の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{
    validate_evidence_record, ImplementationCommandClass, ImplementationEnvironmentClass,
    ImplementationEvidenceReason, ImplementationEvidenceRecord, ImplementationLayer,
    ImplementationNonClaimScope, ImplementationPlane, IMPLEMENTATIONS_COMMAND_ROOT,
};

use crate::error::ProductMonitoringError;

/// product evidence record は shared evidence record のaliasです。
pub type ProductEvidenceRecord = ImplementationEvidenceRecord;

/// product build command evidence の入力です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductBuildEvidenceInput {
    /// command / report / span を接続する相関IDです。
    pub correlation_id: CorrelationId,
    /// 実行した build command です。
    pub command: String,
    /// build target packageです。
    pub target_package: String,
    /// build target scopeです。
    pub target_scope: String,
    /// build target planeです。
    pub target_plane: ImplementationPlane,
    /// toolchain / runtime version表記です。
    pub toolchain_runtime_version: String,
    /// 期待結果です。
    pub expected_outcome: String,
    /// 実結果です。
    pub actual_outcome: String,
    /// process exit statusです。
    pub exit_status: i32,
}

/// product evidence recordを検証済みrecordとして返します。
pub fn build_product_evidence_record(
    record: ImplementationEvidenceRecord,
) -> Result<ProductEvidenceRecord, ProductMonitoringError> {
    validate_evidence_record(&record)
        .map_err(|_| ProductMonitoringError::EvidenceFieldsIncomplete)?;
    validate_product_monitoring_evidence_ownership(&record)?;
    Ok(record)
}

/// product build command evidence recordを作ります。
pub fn build_product_build_evidence_record(
    input: ProductBuildEvidenceInput,
) -> Result<ProductEvidenceRecord, ProductMonitoringError> {
    build_product_evidence_record(ImplementationEvidenceRecord {
        correlation_id: input.correlation_id.as_str().to_owned(),
        command: input.command,
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some(input.target_package),
        target_scope: input.target_scope,
        command_class: ImplementationCommandClass::Build,
        implementation_layer: ImplementationLayer::Product,
        target_plane: input.target_plane,
        environment_class: ImplementationEnvironmentClass::ControlledProcess,
        toolchain_runtime_version: input.toolchain_runtime_version,
        input_fixture_or_workload: None,
        expected_outcome: input.expected_outcome,
        actual_outcome: input.actual_outcome,
        exit_status: Some(input.exit_status),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "product source or build command change".to_owned(),
    })
}

fn validate_product_monitoring_evidence_ownership(
    record: &ImplementationEvidenceRecord,
) -> Result<(), ProductMonitoringError> {
    if record.implementation_layer != ImplementationLayer::Product {
        return Err(ProductMonitoringError::CommandScopeMismatch);
    }
    if !matches!(
        record.command_class,
        ImplementationCommandClass::Build | ImplementationCommandClass::Test
    ) {
        return Err(ProductMonitoringError::CommandScopeMismatch);
    }
    if !is_product_command_class_shape(record.command_class, &record.command) {
        return Err(ProductMonitoringError::CommandScopeMismatch);
    }
    if matches!(
        record.target_plane,
        ImplementationPlane::Composition | ImplementationPlane::Ops
    ) {
        return Err(ProductMonitoringError::CommandScopeMismatch);
    }
    if !record
        .target_package
        .as_deref()
        .is_some_and(|package| package.starts_with("arcrtc-product-"))
    {
        return Err(ProductMonitoringError::CommandScopeMismatch);
    }
    if !record.target_scope.starts_with("product-implementation/") {
        return Err(ProductMonitoringError::CommandScopeMismatch);
    }
    Ok(())
}

fn is_product_command_class_shape(
    command_class: ImplementationCommandClass,
    command: &str,
) -> bool {
    let trimmed = command.trim_start();
    match command_class {
        ImplementationCommandClass::Build => trimmed == "cargo build --workspace --all-targets",
        ImplementationCommandClass::Test => trimmed == "cargo test --workspace --all-targets",
        _ => false,
    }
}
