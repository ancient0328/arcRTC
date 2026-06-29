//! product monitoring evidence 境界を検査します。

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_implementation_evidence::{
    ImplementationCommandClass, ImplementationEnvironmentClass, ImplementationEvidenceReason,
    ImplementationEvidenceRecord, ImplementationLayer, ImplementationNonClaimScope,
    ImplementationPlane, IMPLEMENTATIONS_COMMAND_ROOT,
};
use arcrtc_product_monitoring::{
    build_observability_record, build_product_build_evidence_record, build_product_evidence_record,
    ProductBuildEvidenceInput, ProductMonitoringError,
};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

fn build_input() -> ProductBuildEvidenceInput {
    ProductBuildEvidenceInput {
        correlation_id: cid("product-monitoring-build"),
        command: "cargo build --workspace --all-targets".to_owned(),
        target_package: "arcrtc-product-monitoring".to_owned(),
        target_scope: "product-implementation/monitoring".to_owned(),
        target_plane: ImplementationPlane::Monitoring,
        toolchain_runtime_version: "rustc-test".to_owned(),
        expected_outcome: "build succeeds".to_owned(),
        actual_outcome: "build succeeds".to_owned(),
        exit_status: 0,
    }
}

fn product_test_record() -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "product-monitoring-test".to_owned(),
        command: "cargo test --workspace --all-targets".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-product-monitoring".to_owned()),
        target_scope: "product-implementation/monitoring".to_owned(),
        command_class: ImplementationCommandClass::Test,
        implementation_layer: ImplementationLayer::Product,
        target_plane: ImplementationPlane::Monitoring,
        environment_class: ImplementationEnvironmentClass::ControlledProcess,
        toolchain_runtime_version: "rustc-test".to_owned(),
        input_fixture_or_workload: Some("product-monitoring-test".to_owned()),
        expected_outcome: "tests pass".to_owned(),
        actual_outcome: "tests pass".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "product monitoring test command changes".to_owned(),
    }
}

#[test]
fn monitoring_evidence_accepts_only_product_build_or_test_records() {
    let build = build_product_build_evidence_record(build_input()).expect("build record");
    assert_eq!(build.command_class, ImplementationCommandClass::Build);
    assert!(build
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed));
    assert!(build
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::ProductionReadinessNotClaimed));
    assert!(build
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::LiveReadinessNotClaimed));

    let test = build_product_evidence_record(product_test_record()).expect("test record");
    assert_eq!(test.command_class, ImplementationCommandClass::Test);
}

#[test]
fn monitoring_evidence_rejects_non_product_scope_and_command_shape() {
    let mut wrong_layer = product_test_record();
    wrong_layer.implementation_layer = ImplementationLayer::Reference;
    assert_eq!(
        build_product_evidence_record(wrong_layer),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );

    let mut wrong_package = product_test_record();
    wrong_package.target_package = Some("arcrtc-reference-monitoring".to_owned());
    assert_eq!(
        build_product_evidence_record(wrong_package),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );

    let mut wrong_command = product_test_record();
    wrong_command.command = "cargo bench --workspace".to_owned();
    assert_eq!(
        build_product_evidence_record(wrong_command),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );

    let mut wrong_build_command = product_test_record();
    wrong_build_command.command_class = ImplementationCommandClass::Build;
    wrong_build_command.command = "cargo build".to_owned();
    assert_eq!(
        build_product_evidence_record(wrong_build_command),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );

    let mut wrong_plane = product_test_record();
    wrong_plane.target_plane = ImplementationPlane::Ops;
    assert_eq!(
        build_product_evidence_record(wrong_plane),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );

    let mut wrong_scope = product_test_record();
    wrong_scope.target_scope = "reference-implementation/monitoring".to_owned();
    assert_eq!(
        build_product_evidence_record(wrong_scope),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );
}

#[test]
fn kpi_product_monitoring_maps_evidence_without_readiness_claim() {
    let observability = build_observability_record(
        cid("product-monitoring-observe"),
        ImplementationPlane::Monitoring,
        "product.controlled_process.command",
    );
    assert_eq!(
        observability.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );
    let empty_metric = build_observability_record(
        cid("product-monitoring-empty-metric"),
        ImplementationPlane::Monitoring,
        " ",
    );
    assert_eq!(
        empty_metric.implementation_reason,
        ImplementationEvidenceReason::EvidenceFieldsIncomplete
    );

    let test_record = build_product_evidence_record(product_test_record())
        .expect("product test evidence record must be accepted");
    assert_eq!(test_record.command_class, ImplementationCommandClass::Test);
    assert_ne!(
        test_record.command_class,
        ImplementationCommandClass::ProductionReadiness
    );
    assert_ne!(
        test_record.command_class,
        ImplementationCommandClass::LiveReadiness
    );
    assert!(test_record
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::ProductionReadinessNotClaimed));
    assert!(test_record
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::LiveReadinessNotClaimed));

    for command_class in [
        ImplementationCommandClass::ProductionReadiness,
        ImplementationCommandClass::LiveReadiness,
    ] {
        let mut readiness_record = product_test_record();
        readiness_record.command_class = command_class;
        let result = build_product_evidence_record(readiness_record);
        // readiness class は shared schema または product ownership のどちらでも閉じられます。
        assert!(matches!(
            result,
            Err(ProductMonitoringError::EvidenceFieldsIncomplete
                | ProductMonitoringError::CommandScopeMismatch)
        ));
    }
}
