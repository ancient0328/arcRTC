//! product monitoring evidence 境界を検査します。

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_distro_evidence::{
    DistroCommandClass, DistroEnvironmentClass, DistroEvidenceReason,
    DistroEvidenceRecord, DistroLayer, DistroNonClaimScope,
    DistroPlane, DISTRO_COMMAND_ROOT,
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
        target_scope: "product-distro/monitoring".to_owned(),
        target_plane: DistroPlane::Monitoring,
        toolchain_runtime_version: "rustc-test".to_owned(),
        expected_outcome: "build succeeds".to_owned(),
        actual_outcome: "build succeeds".to_owned(),
        exit_status: 0,
    }
}

fn product_test_record() -> DistroEvidenceRecord {
    DistroEvidenceRecord {
        correlation_id: "product-monitoring-test".to_owned(),
        command: "cargo test --workspace --all-targets".to_owned(),
        working_directory: DISTRO_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-product-monitoring".to_owned()),
        target_scope: "product-distro/monitoring".to_owned(),
        command_class: DistroCommandClass::Test,
        distro_layer: DistroLayer::Product,
        target_plane: DistroPlane::Monitoring,
        environment_class: DistroEnvironmentClass::ControlledProcess,
        toolchain_runtime_version: "rustc-test".to_owned(),
        input_fixture_or_workload: Some("product-monitoring-test".to_owned()),
        expected_outcome: "tests pass".to_owned(),
        actual_outcome: "tests pass".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: vec![
            DistroNonClaimScope::BehaviorCorrectnessNotClaimed,
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "product monitoring test command changes".to_owned(),
    }
}

#[test]
fn monitoring_evidence_accepts_only_product_build_or_test_records() {
    let build = build_product_build_evidence_record(build_input()).expect("build record");
    assert_eq!(build.command_class, DistroCommandClass::Build);
    assert!(build
        .non_claim_scope
        .contains(&DistroNonClaimScope::BehaviorCorrectnessNotClaimed));
    assert!(build
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
    assert!(build
        .non_claim_scope
        .contains(&DistroNonClaimScope::LiveReadinessNotClaimed));

    let test = build_product_evidence_record(product_test_record()).expect("test record");
    assert_eq!(test.command_class, DistroCommandClass::Test);
}

#[test]
fn monitoring_evidence_rejects_non_product_scope_and_command_shape() {
    let mut wrong_layer = product_test_record();
    wrong_layer.distro_layer = DistroLayer::Reference;
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
    wrong_build_command.command_class = DistroCommandClass::Build;
    wrong_build_command.command = "cargo build".to_owned();
    assert_eq!(
        build_product_evidence_record(wrong_build_command),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );

    let mut wrong_plane = product_test_record();
    wrong_plane.target_plane = DistroPlane::Ops;
    assert_eq!(
        build_product_evidence_record(wrong_plane),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );

    let mut wrong_scope = product_test_record();
    wrong_scope.target_scope = "reference-distro/monitoring".to_owned();
    assert_eq!(
        build_product_evidence_record(wrong_scope),
        Err(ProductMonitoringError::CommandScopeMismatch)
    );
}

#[test]
fn kpi_product_monitoring_maps_evidence_without_readiness_claim() {
    let observability = build_observability_record(
        cid("product-monitoring-observe"),
        DistroPlane::Monitoring,
        "product.controlled_process.command",
    );
    assert_eq!(
        observability.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    let empty_metric = build_observability_record(
        cid("product-monitoring-empty-metric"),
        DistroPlane::Monitoring,
        " ",
    );
    assert_eq!(
        empty_metric.distro_reason,
        DistroEvidenceReason::EvidenceFieldsIncomplete
    );

    let test_record = build_product_evidence_record(product_test_record())
        .expect("product test evidence record must be accepted");
    assert_eq!(test_record.command_class, DistroCommandClass::Test);
    assert_ne!(
        test_record.command_class,
        DistroCommandClass::ProductionReadiness
    );
    assert_ne!(
        test_record.command_class,
        DistroCommandClass::LiveReadiness
    );
    assert!(test_record
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
    assert!(test_record
        .non_claim_scope
        .contains(&DistroNonClaimScope::LiveReadinessNotClaimed));

    for command_class in [
        DistroCommandClass::ProductionReadiness,
        DistroCommandClass::LiveReadiness,
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
