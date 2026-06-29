//! production readiness fail-closed 境界を検査します。

use arcrtc_implementation_evidence::{
    validate_readiness_evidence_record, EvidenceValidationError, ImplementationCommandClass,
    ImplementationEnvironmentClass, ImplementationEvidenceReason, ImplementationEvidenceRecord,
    ImplementationLayer, ImplementationNonClaimScope, ImplementationPlane, ReadinessAdmissionState,
    ReadinessClaim, ReadinessEvidenceRecord, ReadinessEvidenceValidationError,
    ReadinessValidationContext, IMPLEMENTATIONS_COMMAND_ROOT,
};

fn base() -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "production-readiness-fail-closed".to_owned(),
        command: "cargo test --manifest-path tests/production-readiness/Cargo.toml".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-production-readiness-tests".to_owned()),
        target_scope: "tests/production-readiness".to_owned(),
        command_class: ImplementationCommandClass::ProductionReadiness,
        implementation_layer: ImplementationLayer::Readiness,
        target_plane: ImplementationPlane::Deployment,
        environment_class: ImplementationEnvironmentClass::ProductionDeferred,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("production-readiness-fail-closed".to_owned()),
        expected_outcome: "readiness extension rejects missing authority".to_owned(),
        actual_outcome: "readiness extension rejects missing authority".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
        rerun_condition: "rerun when readiness matrix changes".to_owned(),
    }
}

fn provider_record(gate_id: &str) -> ReadinessEvidenceRecord {
    ReadinessEvidenceRecord {
        base: base(),
        readiness_claim: ReadinessClaim::ProductionReadiness,
        readiness_gate_id: gate_id.to_owned(),
        readiness_adr_ref:
            "READINESS_CLAIM_BOUNDARY"
                .to_owned(),
        readiness_canonical_ref: "READINESS_MATRIX"
            .to_owned(),
        build_evidence_ref: None,
        behavior_test_evidence_ref: None,
        auth_provider_admission_ref: (gate_id == "PRD-003")
            .then(|| "canonical:production-auth-provider-admission".to_owned()),
        persistence_provider_admission_ref: (gate_id == "PRD-004")
            .then(|| "canonical:production-persistence-provider-admission".to_owned()),
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
    }
}

fn live_record_in_production_file(gate_id: &str) -> ReadinessEvidenceRecord {
    let mut record = provider_record(gate_id);
    record.base.command_class = ImplementationCommandClass::LiveReadiness;
    record.base.non_claim_scope = vec![
        ImplementationNonClaimScope::KernelCompletionNotClaimed,
        ImplementationNonClaimScope::KernelFreezeNotClaimed,
    ];
    record.readiness_claim = ReadinessClaim::LiveReadiness;
    record.auth_provider_admission_ref = None;
    record.persistence_provider_admission_ref = None;
    record
}

fn production_context() -> ReadinessValidationContext {
    ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    }
}

#[test]
fn provider_dependent_production_gates_fail_closed_without_provider_authority() {
    let context_without_auth = ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Absent,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Absent,
        production_readiness_report: ReadinessAdmissionState::Absent,
        public_traversal_authority: ReadinessAdmissionState::Absent,
    };
    assert_eq!(
        validate_readiness_evidence_record(&provider_record("PRD-003"), &context_without_auth),
        Err(ReadinessEvidenceValidationError::AuthProviderAuthorityNotAdmitted)
    );

    let context_without_persistence = ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Absent,
        live_endpoint_authority: ReadinessAdmissionState::Absent,
        production_readiness_report: ReadinessAdmissionState::Absent,
        public_traversal_authority: ReadinessAdmissionState::Absent,
    };
    assert_eq!(
        validate_readiness_evidence_record(
            &provider_record("PRD-004"),
            &context_without_persistence
        ),
        Err(ReadinessEvidenceValidationError::PersistenceProviderAuthorityNotAdmitted)
    );
}

#[test]
fn production_claim_rejects_live_gate_ids() {
    let context = production_context();
    for gate_id in [
        "LIVE-001", "LIVE-002", "LIVE-003", "LIVE-004", "LIVE-005", "LIVE-006", "LIVE-007",
        "LIVE-008",
    ] {
        let mut record = provider_record("PRD-003");
        record.readiness_gate_id = gate_id.to_owned();
        assert_eq!(
            validate_readiness_evidence_record(&record, &context),
            Err(ReadinessEvidenceValidationError::ReadinessGateClaimMismatch),
            "{gate_id}"
        );
    }

    let mut wrong_ref_and_gate = provider_record("PRD-003");
    wrong_ref_and_gate.readiness_gate_id = "LIVE-002".to_owned();
    wrong_ref_and_gate.readiness_adr_ref = "OTHER_ADR_BOUNDARY".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&wrong_ref_and_gate, &context),
        Err(ReadinessEvidenceValidationError::ReadinessGateClaimMismatch)
    );
}

#[test]
fn production_readiness_asserts_unknown_and_live_authority_fail_closed_branches() {
    let context = production_context();
    let mut unknown_gate = provider_record("PRD-003");
    unknown_gate.readiness_gate_id = "PRD-999".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&unknown_gate, &context),
        Err(ReadinessEvidenceValidationError::GateIdNotListed)
    );

    let mut live_endpoint_gate = live_record_in_production_file("LIVE-002");
    live_endpoint_gate.live_endpoint_evidence_ref = Some("report:endpoint".to_owned());
    let context_without_live = ReadinessValidationContext {
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
        live_endpoint_gate.readiness_gate_id = gate_id.to_owned();
        assert_eq!(
            validate_readiness_evidence_record(&live_endpoint_gate, &context_without_live),
            Err(ReadinessEvidenceValidationError::LiveEndpointAuthorityNotAdmitted),
            "{gate_id}"
        );
    }

    let mut live_production_gate = live_record_in_production_file("LIVE-001");
    live_production_gate.production_readiness_report_ref = Some("report:production".to_owned());
    let context_without_production = ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Absent,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    };
    assert_eq!(
        validate_readiness_evidence_record(&live_production_gate, &context_without_production),
        Err(ReadinessEvidenceValidationError::ProductionReadinessReportNotAdmitted)
    );

    let mut live_traversal_gate = live_record_in_production_file("LIVE-003");
    live_traversal_gate.public_traversal_evidence_ref = Some("report:traversal".to_owned());
    let context_without_traversal = ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Absent,
    };
    assert_eq!(
        validate_readiness_evidence_record(&live_traversal_gate, &context_without_traversal),
        Err(ReadinessEvidenceValidationError::PublicTraversalAuthorityNotAdmitted)
    );
}

#[test]
fn kpi_production_readiness_remains_fail_closed_without_dedicated_closed_gate() {
    let context = production_context();
    let missing_closed_gate = provider_record("PRD-009");

    assert_eq!(
        validate_readiness_evidence_record(&missing_closed_gate, &context),
        Err(ReadinessEvidenceValidationError::MissingRequiredExtensionRef)
    );

    let mut wrong_gate_shape = provider_record("PRD-003");
    wrong_gate_shape.closed_gate_report_ref = Some("CLOSED_GATE_REPORT".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&wrong_gate_shape, &context),
        Err(ReadinessEvidenceValidationError::UnexpectedExtensionRefPopulated)
    );
}

#[test]
fn production_readiness_rejects_command_claim_mismatch_and_empty_required_refs() {
    let context = production_context();

    let mut wrong_command_class = provider_record("PRD-003");
    wrong_command_class.base.command_class = ImplementationCommandClass::LiveReadiness;
    assert_eq!(
        validate_readiness_evidence_record(&wrong_command_class, &context),
        Err(ReadinessEvidenceValidationError::CommandClassClaimMismatch)
    );

    let mut empty_authority_ref = provider_record("PRD-003");
    empty_authority_ref.readiness_adr_ref.clear();
    assert_eq!(
        validate_readiness_evidence_record(&empty_authority_ref, &context),
        Err(ReadinessEvidenceValidationError::EmptyRequiredExtensionRef)
    );

    let mut empty_gate_ref = provider_record("PRD-003");
    empty_gate_ref.auth_provider_admission_ref = Some("   ".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&empty_gate_ref, &context),
        Err(ReadinessEvidenceValidationError::EmptyRequiredExtensionRef)
    );
}

#[test]
fn production_readiness_rejects_missing_or_extra_extension_refs() {
    let context = production_context();

    let mut missing_required = provider_record("PRD-003");
    missing_required.auth_provider_admission_ref = None;
    assert_eq!(
        validate_readiness_evidence_record(&missing_required, &context),
        Err(ReadinessEvidenceValidationError::MissingRequiredExtensionRef)
    );

    let mut unexpected_extra = provider_record("PRD-003");
    unexpected_extra.deployment_profile_ref = Some("report:deployment".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&unexpected_extra, &context),
        Err(ReadinessEvidenceValidationError::UnexpectedExtensionRefPopulated)
    );
}

#[test]
fn production_readiness_rejects_authority_ref_secret_and_non_claim_scope_violations() {
    let context = production_context();

    let mut wrong_authority = provider_record("PRD-003");
    wrong_authority.readiness_canonical_ref = "OTHER_AUTHORITY_MATRIX".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&wrong_authority, &context),
        Err(ReadinessEvidenceValidationError::ReadinessAuthorityRefMismatch)
    );

    let mut secret_like_ref = provider_record("PRD-003");
    secret_like_ref.auth_provider_admission_ref = Some("token=abc".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&secret_like_ref, &context),
        Err(ReadinessEvidenceValidationError::SecretLikeExtensionRef)
    );

    let mut missing_non_claim = provider_record("PRD-003");
    missing_non_claim.base.non_claim_scope.clear();
    assert_eq!(
        validate_readiness_evidence_record(&missing_non_claim, &context),
        Err(ReadinessEvidenceValidationError::Base(
            EvidenceValidationError::EmptyNonClaimScope
        ))
    );
}
