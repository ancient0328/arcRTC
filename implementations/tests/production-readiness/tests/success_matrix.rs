//! production readiness success matrix を検査します。

use std::collections::BTreeSet;

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_implementation_evidence::{
    validate_readiness_evidence_record, ImplementationCommandClass, ImplementationEnvironmentClass,
    ImplementationEvidenceReason, ImplementationEvidenceRecord, ImplementationLayer,
    ImplementationNonClaimScope, ImplementationPlane, ReadinessAdmissionState, ReadinessClaim,
    ReadinessEvidenceRecord, ReadinessValidationContext, IMPLEMENTATIONS_COMMAND_ROOT,
};
use arcrtc_product_deployment::{
    build_product_production_profile, select_product_runtime, ProductHostClass,
};
use arcrtc_product_monitoring::build_production_monitoring_probe;
use arcrtc_product_persistence_topology::{
    admit_product_persistence_provider, ProductPersistenceProviderClass,
    ProductPersistenceTopologyError,
};
use arcrtc_product_policy::{
    admit_product_auth_provider, ProductAuthProviderClass, ProductPolicyError,
};
use arcrtc_product_rollback::{
    plan_production_drain, plan_production_restore, ProductRollbackError,
};

struct ProductionReadinessGateFixture {
    gate_id: &'static str,
    required_ref_field: &'static str,
    ref_value: &'static str,
    failure_classification: &'static str,
    rerun_condition: &'static str,
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(
        OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
            .expect("test correlation id must be accepted"),
    )
}

fn base() -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "production-readiness-success-matrix".to_owned(),
        command: "cargo test --manifest-path tests/production-readiness/Cargo.toml kpi_production_readiness_success_matrix_accepts_prd_001_through_prd_009".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-production-readiness-tests".to_owned()),
        target_scope: "tests/production-readiness".to_owned(),
        command_class: ImplementationCommandClass::ProductionReadiness,
        implementation_layer: ImplementationLayer::Readiness,
        target_plane: ImplementationPlane::Deployment,
        environment_class: ImplementationEnvironmentClass::ProductionDeferred,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("PRD-001 through PRD-009 success matrix".to_owned()),
        expected_outcome: "production readiness success matrix accepts PRD-001 through PRD-009".to_owned(),
        actual_outcome: "production readiness success matrix accepts PRD-001 through PRD-009".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
        rerun_condition: "rerun when production readiness matrix or provider admission changes"
            .to_owned(),
    }
}

fn admitted_context() -> ReadinessValidationContext {
    ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Absent,
        production_readiness_report: ReadinessAdmissionState::Absent,
        public_traversal_authority: ReadinessAdmissionState::Absent,
    }
}

fn fixtures() -> Vec<ProductionReadinessGateFixture> {
    vec![
        ProductionReadinessGateFixture {
            gate_id: "PRD-001",
            required_ref_field: "build_evidence_ref",
            ref_value: "PRODUCT_WORKSPACE_BUILD_EVIDENCE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "product workspace build command changes",
        },
        ProductionReadinessGateFixture {
            gate_id: "PRD-002",
            required_ref_field: "behavior_test_evidence_ref",
            ref_value: "PRODUCT_BEHAVIOR_TEST_EVIDENCE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "product behavior test command changes",
        },
        ProductionReadinessGateFixture {
            gate_id: "PRD-003",
            required_ref_field: "auth_provider_admission_ref",
            ref_value: "AUTH_PROVIDER_ADMISSION",
            failure_classification: "READINESS_NOT_ADMITTED",
            rerun_condition: "auth provider admission changes",
        },
        ProductionReadinessGateFixture {
            gate_id: "PRD-004",
            required_ref_field: "persistence_provider_admission_ref",
            ref_value: "PERSISTENCE_PROVIDER_ADMISSION",
            failure_classification: "READINESS_NOT_ADMITTED",
            rerun_condition: "persistence provider admission changes",
        },
        ProductionReadinessGateFixture {
            gate_id: "PRD-005",
            required_ref_field: "deployment_profile_ref",
            ref_value: "PRODUCTION_RUNTIME_PROFILE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "production profile source changes",
        },
        ProductionReadinessGateFixture {
            gate_id: "PRD-006",
            required_ref_field: "monitoring_probe_ref",
            ref_value: "PRODUCTION_MONITORING_PROBE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "production monitoring probe changes",
        },
        ProductionReadinessGateFixture {
            gate_id: "PRD-007",
            required_ref_field: "rollback_plan_ref",
            ref_value: "PRODUCTION_OPERATION_PLAN",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "production rollback plan changes",
        },
        ProductionReadinessGateFixture {
            gate_id: "PRD-008",
            required_ref_field: "security_scan_ref",
            ref_value: "SECURITY_SCAN_EVIDENCE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "production readiness security scan scope changes",
        },
        ProductionReadinessGateFixture {
            gate_id: "PRD-009",
            required_ref_field: "closed_gate_report_ref",
            ref_value: "CLOSED_GATE_REPORT_EVALUATION",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "production readiness Closed Gate report changes",
        },
    ]
}

fn record(fixture: &ProductionReadinessGateFixture) -> ReadinessEvidenceRecord {
    let mut record = ReadinessEvidenceRecord {
        base: base(),
        readiness_claim: ReadinessClaim::ProductionReadiness,
        readiness_gate_id: fixture.gate_id.to_owned(),
        readiness_adr_ref:
            "READINESS_CLAIM_BOUNDARY"
                .to_owned(),
        readiness_canonical_ref: "READINESS_MATRIX"
            .to_owned(),
        build_evidence_ref: None,
        behavior_test_evidence_ref: None,
        auth_provider_admission_ref: None,
        persistence_provider_admission_ref: None,
        deployment_profile_ref: None,
        monitoring_probe_ref: None,
        rollback_plan_ref: None,
        security_scan_ref: None,
        production_readiness_report_ref: None,
        live_endpoint_evidence_ref: None,
        public_traversal_evidence_ref: None,
        rollback_drain_execution_ref: None,
        shutdown_drain_evidence_ref: None,
        restore_evidence_ref: None,
        closed_gate_report_ref: None,
    };
    let ref_value = Some(fixture.ref_value.to_owned());
    match fixture.required_ref_field {
        "build_evidence_ref" => record.build_evidence_ref = ref_value,
        "behavior_test_evidence_ref" => record.behavior_test_evidence_ref = ref_value,
        "auth_provider_admission_ref" => record.auth_provider_admission_ref = ref_value,
        "persistence_provider_admission_ref" => {
            record.persistence_provider_admission_ref = ref_value;
        }
        "deployment_profile_ref" => record.deployment_profile_ref = ref_value,
        "monitoring_probe_ref" => record.monitoring_probe_ref = ref_value,
        "rollback_plan_ref" => record.rollback_plan_ref = ref_value,
        "security_scan_ref" => record.security_scan_ref = ref_value,
        "closed_gate_report_ref" => record.closed_gate_report_ref = ref_value,
        _ => unreachable!("unknown production readiness ref field"),
    }
    record
}

#[test]
fn kpi_production_readiness_success_matrix_accepts_prd_001_through_prd_009() {
    let fixtures = fixtures();
    let gate_ids = fixtures
        .iter()
        .map(|fixture| fixture.gate_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        gate_ids,
        BTreeSet::from([
            "PRD-001", "PRD-002", "PRD-003", "PRD-004", "PRD-005", "PRD-006", "PRD-007", "PRD-008",
            "PRD-009",
        ])
    );

    for fixture in &fixtures {
        assert_eq!(
            validate_readiness_evidence_record(&record(fixture), &admitted_context()),
            Ok(()),
            "{}",
            fixture.gate_id
        );
        assert!(!fixture.failure_classification.trim().is_empty());
        assert!(!fixture.rerun_condition.trim().is_empty());
        assert!(!fixture.ref_value.trim().is_empty());
    }
}

#[test]
fn production_provider_source_accepts_only_admitted_provider_classes() {
    let auth = admit_product_auth_provider(ProductAuthProviderClass::ControlledProductionIdentity)
        .expect("auth provider must be admitted");
    assert_eq!(
        auth.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );
    assert_eq!(
        admit_product_auth_provider(ProductAuthProviderClass::NotAdmitted),
        Err(ProductPolicyError::ReadinessNotAdmitted)
    );

    let persistence = admit_product_persistence_provider(
        ProductPersistenceProviderClass::ControlledProjectionStore,
    )
    .expect("persistence provider must be admitted");
    assert_eq!(
        persistence.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );
    assert_eq!(
        admit_product_persistence_provider(ProductPersistenceProviderClass::NotAdmitted),
        Err(ProductPersistenceTopologyError::ProviderNotAdmitted)
    );
}

#[test]
fn production_profile_probe_and_operation_sources_are_admitted_without_live_claim() {
    let profile = build_product_production_profile();
    assert_eq!(profile.host_class, ProductHostClass::ProductionAdmitted);
    assert!(!profile.public_endpoint_claimed);
    assert_eq!(
        select_product_runtime(&profile).implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );

    let probe = build_production_monitoring_probe(
        cid("production-monitoring-probe"),
        ImplementationPlane::Monitoring,
        "product.production.readiness",
    )
    .expect("production monitoring probe must be admitted");
    assert_eq!(
        probe.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );

    let drain = plan_production_drain(
        cid("production-drain-plan"),
        vec![
            ImplementationPlane::Signaling,
            ImplementationPlane::Turn,
            ImplementationPlane::Sfu,
        ],
    )
    .expect("production drain plan must be admitted");
    assert_eq!(
        drain.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );

    let restore = plan_production_restore(cid("production-restore-plan"))
        .expect("production restore plan must be admitted");
    assert_eq!(
        restore.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );

    assert_eq!(
        plan_production_drain(cid("production-empty-drain-plan"), Vec::new()),
        Err(ProductRollbackError::RuntimeExecutorError)
    );
}
