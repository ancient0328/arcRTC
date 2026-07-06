//! live readiness fail-closed 境界を検査します。

use arcrtc_distro_evidence::{
    validate_readiness_evidence_record, EvidenceValidationError, DistroCommandClass,
    DistroEnvironmentClass, DistroEvidenceReason, DistroEvidenceRecord,
    DistroLayer, DistroNonClaimScope, DistroPlane, ReadinessAdmissionState,
    ReadinessClaim, ReadinessEvidenceRecord, ReadinessEvidenceValidationError,
    ReadinessValidationContext, DISTRO_COMMAND_ROOT,
};

fn base() -> DistroEvidenceRecord {
    DistroEvidenceRecord {
        correlation_id: "live-readiness-fail-closed".to_owned(),
        command: "cargo test --manifest-path tests/live/Cargo.toml".to_owned(),
        working_directory: DISTRO_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-distro-live-tests".to_owned()),
        target_scope: "tests/live".to_owned(),
        command_class: DistroCommandClass::LiveReadiness,
        distro_layer: DistroLayer::Readiness,
        target_plane: DistroPlane::Deployment,
        environment_class: DistroEnvironmentClass::LiveDeferred,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("live-readiness-fail-closed".to_owned()),
        expected_outcome: "live readiness extension rejects missing authority".to_owned(),
        actual_outcome: "live readiness extension rejects missing authority".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: vec![
            DistroNonClaimScope::LiveReadinessNotClaimed,
            DistroNonClaimScope::KernelCompletionNotClaimed,
            DistroNonClaimScope::KernelFreezeNotClaimed,
        ],
        rerun_condition: "rerun when live readiness matrix changes".to_owned(),
    }
}

fn live_record(gate_id: &str) -> ReadinessEvidenceRecord {
    let mut record = ReadinessEvidenceRecord {
        base: base(),
        readiness_claim: ReadinessClaim::LiveReadiness,
        readiness_gate_id: gate_id.to_owned(),
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
    match gate_id {
        "LIVE-001" => record.production_readiness_report_ref = Some("report:production".to_owned()),
        "LIVE-003" => record.public_traversal_evidence_ref = Some("report:traversal".to_owned()),
        _ => record.live_endpoint_evidence_ref = Some("report:endpoint".to_owned()),
    }
    record
}

fn production_record_in_live_file(gate_id: &str) -> ReadinessEvidenceRecord {
    let mut record = live_record(gate_id);
    record.base.command_class = DistroCommandClass::ProductionReadiness;
    record.base.non_claim_scope = vec![
        DistroNonClaimScope::LiveReadinessNotClaimed,
        DistroNonClaimScope::KernelCompletionNotClaimed,
        DistroNonClaimScope::KernelFreezeNotClaimed,
    ];
    record.readiness_claim = ReadinessClaim::ProductionReadiness;
    record.live_endpoint_evidence_ref = None;
    record.production_readiness_report_ref = None;
    record.public_traversal_evidence_ref = None;
    record
}

fn live_context() -> ReadinessValidationContext {
    ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    }
}

#[test]
fn live_readiness_fails_closed_without_live_endpoint_authority() {
    let context = ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Absent,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    };
    for gate_id in [
        "LIVE-001", "LIVE-002", "LIVE-003", "LIVE-004", "LIVE-005", "LIVE-006", "LIVE-007",
        "LIVE-008",
    ] {
        assert_eq!(
            validate_readiness_evidence_record(&live_record(gate_id), &context),
            Err(ReadinessEvidenceValidationError::LiveEndpointAuthorityNotAdmitted),
            "{gate_id}"
        );
    }
}

#[test]
fn live_readiness_asserts_provider_authority_fail_closed_branch() {
    let context = ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Absent,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    };
    let mut record = production_record_in_live_file("PRD-003");
    record.auth_provider_admission_ref =
        Some("canonical:production-auth-provider-admission".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&record, &context),
        Err(ReadinessEvidenceValidationError::AuthProviderAuthorityNotAdmitted)
    );
}

#[test]
fn live_readiness_fails_closed_without_production_or_public_traversal_authority() {
    let context_without_production = ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Absent,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    };
    assert_eq!(
        validate_readiness_evidence_record(&live_record("LIVE-001"), &context_without_production),
        Err(ReadinessEvidenceValidationError::ProductionReadinessReportNotAdmitted)
    );

    let context_without_traversal = ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Absent,
    };
    assert_eq!(
        validate_readiness_evidence_record(&live_record("LIVE-003"), &context_without_traversal),
        Err(ReadinessEvidenceValidationError::PublicTraversalAuthorityNotAdmitted)
    );
}

#[test]
fn kpi_live_readiness_remains_fail_closed_without_dedicated_closed_gate() {
    let context = live_context();
    let mut missing_closed_gate = live_record("LIVE-008");
    missing_closed_gate.live_endpoint_evidence_ref = None;

    assert_eq!(
        validate_readiness_evidence_record(&missing_closed_gate, &context),
        Err(ReadinessEvidenceValidationError::MissingRequiredExtensionRef)
    );

    let mut wrong_gate_shape = live_record("LIVE-002");
    wrong_gate_shape.closed_gate_report_ref = Some("CLOSED_GATE_REPORT".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&wrong_gate_shape, &context),
        Err(ReadinessEvidenceValidationError::UnexpectedExtensionRefPopulated)
    );
}

#[test]
fn live_claim_rejects_production_gate_ids() {
    let context = live_context();
    for gate_id in [
        "PRD-001", "PRD-002", "PRD-003", "PRD-004", "PRD-005", "PRD-006", "PRD-007", "PRD-008",
        "PRD-009",
    ] {
        let mut record = live_record("LIVE-001");
        record.readiness_gate_id = gate_id.to_owned();
        assert_eq!(
            validate_readiness_evidence_record(&record, &context),
            Err(ReadinessEvidenceValidationError::ReadinessGateClaimMismatch),
            "{gate_id}"
        );
    }

    let mut wrong_ref_and_gate = live_record("LIVE-002");
    wrong_ref_and_gate.readiness_gate_id = "PRD-003".to_owned();
    wrong_ref_and_gate.readiness_canonical_ref = "OTHER_MATRIX".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&wrong_ref_and_gate, &context),
        Err(ReadinessEvidenceValidationError::ReadinessGateClaimMismatch)
    );
}

#[test]
fn live_readiness_rejects_command_claim_mismatch_and_empty_required_refs() {
    let context = live_context();

    let mut wrong_command_class = live_record("LIVE-002");
    wrong_command_class.base.command_class = DistroCommandClass::ProductionReadiness;
    wrong_command_class
        .base
        .non_claim_scope
        .push(DistroNonClaimScope::LiveReadinessNotClaimed);
    assert_eq!(
        validate_readiness_evidence_record(&wrong_command_class, &context),
        Err(ReadinessEvidenceValidationError::CommandClassClaimMismatch)
    );

    let mut empty_authority_ref = live_record("LIVE-002");
    empty_authority_ref.readiness_canonical_ref.clear();
    assert_eq!(
        validate_readiness_evidence_record(&empty_authority_ref, &context),
        Err(ReadinessEvidenceValidationError::EmptyRequiredExtensionRef)
    );

    let mut empty_gate_ref = live_record("LIVE-002");
    empty_gate_ref.live_endpoint_evidence_ref = Some("   ".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&empty_gate_ref, &context),
        Err(ReadinessEvidenceValidationError::EmptyRequiredExtensionRef)
    );
}

#[test]
fn live_readiness_rejects_missing_or_extra_extension_refs() {
    let context = live_context();

    let mut missing_required = live_record("LIVE-002");
    missing_required.live_endpoint_evidence_ref = None;
    assert_eq!(
        validate_readiness_evidence_record(&missing_required, &context),
        Err(ReadinessEvidenceValidationError::MissingRequiredExtensionRef)
    );

    let mut unexpected_extra = live_record("LIVE-002");
    unexpected_extra.monitoring_probe_ref = Some("report:monitoring".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&unexpected_extra, &context),
        Err(ReadinessEvidenceValidationError::UnexpectedExtensionRefPopulated)
    );
}

#[test]
fn live_readiness_rejects_authority_ref_secret_gate_and_non_claim_scope_violations() {
    let context = live_context();

    let mut wrong_authority = live_record("LIVE-002");
    wrong_authority.readiness_adr_ref = "OTHER_BOUNDARY".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&wrong_authority, &context),
        Err(ReadinessEvidenceValidationError::ReadinessAuthorityRefMismatch)
    );

    let mut secret_like_ref = live_record("LIVE-002");
    secret_like_ref.live_endpoint_evidence_ref = Some("secret=endpoint".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&secret_like_ref, &context),
        Err(ReadinessEvidenceValidationError::SecretLikeExtensionRef)
    );

    let mut unknown_gate = live_record("LIVE-999");
    unknown_gate.live_endpoint_evidence_ref = None;
    assert_eq!(
        validate_readiness_evidence_record(&unknown_gate, &context),
        Err(ReadinessEvidenceValidationError::GateIdNotListed)
    );

    let mut missing_non_claim = live_record("LIVE-002");
    missing_non_claim.base.non_claim_scope.clear();
    assert_eq!(
        validate_readiness_evidence_record(&missing_non_claim, &context),
        Err(ReadinessEvidenceValidationError::Base(
            EvidenceValidationError::EmptyNonClaimScope
        ))
    );
}
