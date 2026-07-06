//! live readiness success matrix を検査します。

use std::collections::BTreeSet;

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_distro_evidence::{
    validate_readiness_evidence_record, DistroCommandClass, DistroEnvironmentClass,
    DistroEvidenceReason, DistroEvidenceRecord, DistroLayer,
    DistroNonClaimScope, DistroPlane, ReadinessAdmissionState, ReadinessClaim,
    ReadinessEvidenceRecord, ReadinessValidationContext, DISTRO_COMMAND_ROOT,
};
use arcrtc_product_deployment::{
    admit_product_live_endpoint, admit_public_traversal, build_product_live_profile,
    select_product_runtime, ProductHostClass, ProductLiveEndpointClass,
    ProductPublicTraversalClass, ProductRuntimeError,
};
use arcrtc_product_monitoring::build_live_monitoring_probe;
use arcrtc_product_rollback::{
    execute_live_restore, execute_live_shutdown_drain, ProductRollbackError,
};
use arcrtc_product_sfu::{build_live_product_sfu_runtime, ProductSfuError};
use arcrtc_product_signaling::{build_live_product_signaling_runtime, ProductSignalingError};
use arcrtc_product_turn::{build_live_product_turn_runtime, ProductTurnError};

struct LiveReadinessGateFixture {
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

fn base() -> DistroEvidenceRecord {
    DistroEvidenceRecord {
        correlation_id: "live-readiness-success-matrix".to_owned(),
        command: "cargo test --manifest-path tests/live/Cargo.toml kpi_live_readiness_success_matrix_accepts_live_001_through_live_008".to_owned(),
        working_directory: DISTRO_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-distro-live-tests".to_owned()),
        target_scope: "tests/live".to_owned(),
        command_class: DistroCommandClass::LiveReadiness,
        distro_layer: DistroLayer::Readiness,
        target_plane: DistroPlane::Deployment,
        environment_class: DistroEnvironmentClass::LiveDeferred,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("LIVE-001 through LIVE-008 success matrix".to_owned()),
        expected_outcome: "live readiness success matrix accepts LIVE-001 through LIVE-008"
            .to_owned(),
        actual_outcome: "live readiness success matrix accepts LIVE-001 through LIVE-008"
            .to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: vec![
            DistroNonClaimScope::KernelCompletionNotClaimed,
            DistroNonClaimScope::KernelFreezeNotClaimed,
        ],
        rerun_condition: "rerun when live readiness matrix or live endpoint traversal changes"
            .to_owned(),
    }
}

fn admitted_context() -> ReadinessValidationContext {
    ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    }
}

fn fixtures() -> Vec<LiveReadinessGateFixture> {
    vec![
        LiveReadinessGateFixture {
            gate_id: "LIVE-001",
            required_ref_field: "production_readiness_report_ref",
            ref_value: "PRODUCTION_READINESS_REPORT",
            failure_classification: "READINESS_NOT_ADMITTED",
            rerun_condition: "production readiness success evidence changes",
        },
        LiveReadinessGateFixture {
            gate_id: "LIVE-002",
            required_ref_field: "live_endpoint_evidence_ref",
            ref_value: "LIVE_ENDPOINT_EVIDENCE",
            failure_classification: "READINESS_NOT_ADMITTED",
            rerun_condition: "live endpoint admission changes",
        },
        LiveReadinessGateFixture {
            gate_id: "LIVE-003",
            required_ref_field: "public_traversal_evidence_ref",
            ref_value: "PUBLIC_TRAVERSAL_EVIDENCE",
            failure_classification: "READINESS_NOT_ADMITTED",
            rerun_condition: "public traversal admission changes",
        },
        LiveReadinessGateFixture {
            gate_id: "LIVE-004",
            required_ref_field: "monitoring_probe_ref",
            ref_value: "MONITORING_PROBE_EVIDENCE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "live monitoring probe changes",
        },
        LiveReadinessGateFixture {
            gate_id: "LIVE-005",
            required_ref_field: "rollback_drain_execution_ref",
            ref_value: "ROLLBACK_DRAIN_EXECUTION_EVIDENCE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "live drain operation changes",
        },
        LiveReadinessGateFixture {
            gate_id: "LIVE-006",
            required_ref_field: "shutdown_drain_evidence_ref",
            ref_value: "SHUTDOWN_DRAIN_EVIDENCE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "live shutdown evidence changes",
        },
        LiveReadinessGateFixture {
            gate_id: "LIVE-007",
            required_ref_field: "restore_evidence_ref",
            ref_value: "RESTORE_EVIDENCE",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "live restore operation changes",
        },
        LiveReadinessGateFixture {
            gate_id: "LIVE-008",
            required_ref_field: "closed_gate_report_ref",
            ref_value: "CLOSED_GATE_REPORT",
            failure_classification: "EVIDENCE_FIELDS_INCOMPLETE",
            rerun_condition: "live readiness Closed Gate report changes",
        },
    ]
}

fn record(fixture: &LiveReadinessGateFixture) -> ReadinessEvidenceRecord {
    let mut record = ReadinessEvidenceRecord {
        base: base(),
        readiness_claim: ReadinessClaim::LiveReadiness,
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
        "production_readiness_report_ref" => record.production_readiness_report_ref = ref_value,
        "live_endpoint_evidence_ref" => record.live_endpoint_evidence_ref = ref_value,
        "public_traversal_evidence_ref" => record.public_traversal_evidence_ref = ref_value,
        "monitoring_probe_ref" => record.monitoring_probe_ref = ref_value,
        "rollback_drain_execution_ref" => record.rollback_drain_execution_ref = ref_value,
        "shutdown_drain_evidence_ref" => record.shutdown_drain_evidence_ref = ref_value,
        "restore_evidence_ref" => record.restore_evidence_ref = ref_value,
        "closed_gate_report_ref" => record.closed_gate_report_ref = ref_value,
        _ => unreachable!("unknown live readiness ref field"),
    }
    record
}

#[test]
fn kpi_live_readiness_success_matrix_accepts_live_001_through_live_008() {
    let fixtures = fixtures();
    let gate_ids = fixtures
        .iter()
        .map(|fixture| fixture.gate_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        gate_ids,
        BTreeSet::from([
            "LIVE-001", "LIVE-002", "LIVE-003", "LIVE-004", "LIVE-005", "LIVE-006", "LIVE-007",
            "LIVE-008",
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
fn live_endpoint_traversal_sources_accept_only_admitted_classes() {
    let endpoint = admit_product_live_endpoint(ProductLiveEndpointClass::ControlledPublicEndpoint)
        .expect("live endpoint must be admitted");
    assert_eq!(
        endpoint.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert_eq!(
        admit_product_live_endpoint(ProductLiveEndpointClass::NotAdmitted),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );

    let traversal = admit_public_traversal(ProductPublicTraversalClass::ControlledPublicTraversal)
        .expect("public traversal must be admitted");
    assert_eq!(
        traversal.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert_eq!(
        admit_public_traversal(ProductPublicTraversalClass::NotAdmitted),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );

    let profile = build_product_live_profile(&endpoint).expect("live profile must be admitted");
    assert_eq!(profile.host_class, ProductHostClass::LiveAdmitted);
    assert!(profile.public_endpoint_claimed);
    assert_eq!(
        select_product_runtime(&profile).distro_reason,
        DistroEvidenceReason::DistroOk
    );
}

#[test]
fn live_probe_operation_and_plane_sources_are_admitted_by_live_endpoint_ref() {
    let probe = build_live_monitoring_probe(
        cid("live-monitoring-probe"),
        DistroPlane::Monitoring,
        "product.live.readiness",
    )
    .expect("live monitoring probe must be admitted");
    assert_eq!(
        probe.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let drain = execute_live_shutdown_drain(
        cid("live-shutdown-drain"),
        vec![
            DistroPlane::Signaling,
            DistroPlane::Turn,
            DistroPlane::Sfu,
        ],
    )
    .expect("live shutdown drain must be admitted");
    assert_eq!(
        drain.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert_eq!(
        execute_live_shutdown_drain(cid("live-empty-drain"), Vec::new()),
        Err(ProductRollbackError::RuntimeExecutorError)
    );

    let restore = execute_live_restore(cid("live-restore")).expect("live restore must be admitted");
    assert_eq!(
        restore.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let endpoint_ref = "LIVE_ENDPOINT_EVIDENCE";
    let signaling = build_live_product_signaling_runtime(endpoint_ref)
        .expect("live signaling runtime must be admitted");
    assert!(signaling.public_endpoint_claimed);
    assert_eq!(
        signaling.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert_eq!(
        build_live_product_signaling_runtime(" "),
        Err(ProductSignalingError::ReadinessNotAdmitted)
    );

    let turn =
        build_live_product_turn_runtime(endpoint_ref).expect("live turn runtime must be admitted");
    assert!(turn.relay_public_endpoint_claimed);
    assert_eq!(
        turn.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert_eq!(
        build_live_product_turn_runtime(" "),
        Err(ProductTurnError::ReadinessNotAdmitted)
    );

    let sfu =
        build_live_product_sfu_runtime(endpoint_ref).expect("live sfu runtime must be admitted");
    assert!(sfu.media_public_endpoint_claimed);
    assert_eq!(
        sfu.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert_eq!(
        build_live_product_sfu_runtime(" "),
        Err(ProductSfuError::ReadinessNotAdmitted)
    );
}
