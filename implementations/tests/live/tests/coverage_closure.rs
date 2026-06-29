//! Five-item coverage closure tests for live-readiness proof surfaces.

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_implementation_evidence::{
    validate_evidence_record, validate_readiness_evidence_record, EvidenceValidationError,
    ImplementationCommandClass, ImplementationEnvironmentClass, ImplementationEvidenceReason,
    ImplementationEvidenceRecord, ImplementationLayer, ImplementationNonClaimScope,
    ImplementationPlane, ReadinessAdmissionState, ReadinessClaim, ReadinessEvidenceRecord,
    ReadinessEvidenceValidationError, ReadinessValidationContext, IMPLEMENTATIONS_COMMAND_ROOT,
};
use arcrtc_product_deployment::{
    admit_product_live_endpoint, admit_public_traversal, build_product_live_profile,
    build_product_runtime_profile, runtime::ImplementationRuntimeState, select_product_runtime,
    ProductHostClass, ProductLiveEndpointAdmission, ProductLiveEndpointClass,
    ProductPublicTraversalClass, ProductRuntime, ProductRuntimeError, ProductRuntimeProfile,
};
use arcrtc_product_monitoring::{build_live_monitoring_probe, ProductMonitoringError};
use arcrtc_product_rollback::{
    execute_live_restore, execute_live_shutdown_drain, plan_drain, plan_restore, ProductDrainMode,
    ProductRestoreSource, ProductRollbackError,
};
use arcrtc_product_sfu::build_product_sfu_runtime;
use arcrtc_product_signaling::build_product_signaling_runtime;
use arcrtc_product_turn::build_product_turn_runtime;

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(
        OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
            .expect("test correlation id must be accepted"),
    )
}

fn base(command_class: ImplementationCommandClass) -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "live-readiness-coverage-closure".to_owned(),
        command: "cargo test --manifest-path tests/live/Cargo.toml".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-live-tests".to_owned()),
        target_scope: "tests/live".to_owned(),
        command_class,
        implementation_layer: ImplementationLayer::Readiness,
        target_plane: ImplementationPlane::Deployment,
        environment_class: ImplementationEnvironmentClass::LiveDeferred,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("live-readiness-coverage-closure".to_owned()),
        expected_outcome: "live readiness coverage branches are closed".to_owned(),
        actual_outcome: "live readiness coverage branches are closed".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
        rerun_condition: "rerun when live readiness coverage closure changes".to_owned(),
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

fn live_record(gate_id: &str, required_ref: &str) -> ReadinessEvidenceRecord {
    let mut record = ReadinessEvidenceRecord {
        base: base(ImplementationCommandClass::LiveReadiness),
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
    let value = Some(format!("coverage:{gate_id}:{required_ref}"));
    match required_ref {
        "production_readiness_report_ref" => record.production_readiness_report_ref = value,
        "live_endpoint_evidence_ref" => record.live_endpoint_evidence_ref = value,
        "public_traversal_evidence_ref" => record.public_traversal_evidence_ref = value,
        "monitoring_probe_ref" => record.monitoring_probe_ref = value,
        "rollback_drain_execution_ref" => record.rollback_drain_execution_ref = value,
        "shutdown_drain_evidence_ref" => record.shutdown_drain_evidence_ref = value,
        "restore_evidence_ref" => record.restore_evidence_ref = value,
        "closed_gate_report_ref" => record.closed_gate_report_ref = value,
        _ => unreachable!("unknown LIVE field"),
    }
    record
}

fn prd_record(gate_id: &str, required_ref: &str) -> ReadinessEvidenceRecord {
    let mut base = base(ImplementationCommandClass::ProductionReadiness);
    base.non_claim_scope = vec![
        ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ImplementationNonClaimScope::KernelCompletionNotClaimed,
        ImplementationNonClaimScope::KernelFreezeNotClaimed,
    ];
    let mut record = ReadinessEvidenceRecord {
        base,
        readiness_claim: ReadinessClaim::ProductionReadiness,
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
    let value = Some(format!("coverage:{gate_id}:{required_ref}"));
    match required_ref {
        "build_evidence_ref" => record.build_evidence_ref = value,
        "behavior_test_evidence_ref" => record.behavior_test_evidence_ref = value,
        "auth_provider_admission_ref" => record.auth_provider_admission_ref = value,
        "persistence_provider_admission_ref" => {
            record.persistence_provider_admission_ref = value;
        }
        "deployment_profile_ref" => record.deployment_profile_ref = value,
        "monitoring_probe_ref" => record.monitoring_probe_ref = value,
        "rollback_plan_ref" => record.rollback_plan_ref = value,
        "security_scan_ref" => record.security_scan_ref = value,
        "closed_gate_report_ref" => record.closed_gate_report_ref = value,
        _ => unreachable!("unknown PRD field"),
    }
    record
}

#[test]
fn live_readiness_base_validation_covers_command_required_fields() {
    for (record, expected) in [
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.correlation_id.clear();
            (record, EvidenceValidationError::EmptyCorrelationId)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.command.clear();
            (record, EvidenceValidationError::EmptyCommand)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.working_directory.clear();
            (record, EvidenceValidationError::EmptyWorkingDirectory)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.target_scope.clear();
            (record, EvidenceValidationError::EmptyTargetScope)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.toolchain_runtime_version.clear();
            (
                record,
                EvidenceValidationError::EmptyToolchainRuntimeVersion,
            )
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.expected_outcome.clear();
            (record, EvidenceValidationError::EmptyExpectedOutcome)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.actual_outcome.clear();
            (record, EvidenceValidationError::EmptyActualOutcome)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.rerun_condition.clear();
            (record, EvidenceValidationError::EmptyRerunCondition)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.input_fixture_or_workload = Some(" ".to_owned());
            (
                record,
                EvidenceValidationError::MissingRequiredInputFixtureOrWorkload,
            )
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.exit_status = None;
            (record, EvidenceValidationError::MissingRequiredExitStatus)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.non_claim_scope.clear();
            (record, EvidenceValidationError::EmptyNonClaimScope)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.working_directory = "/tmp".to_owned();
            (
                record,
                EvidenceValidationError::WorkingDirectoryOutsideImplementations,
            )
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.target_package = None;
            (
                record,
                EvidenceValidationError::MissingRequiredTargetPackage,
            )
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record
                .non_claim_scope
                .retain(|scope| scope != &ImplementationNonClaimScope::KernelFreezeNotClaimed);
            (
                record,
                EvidenceValidationError::MissingRequiredNonClaimScope,
            )
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.kernel_reason = Some("unknown".to_owned());
            (record, EvidenceValidationError::ForbiddenUnclassifiedReason)
        },
        {
            let mut record = base(ImplementationCommandClass::LiveReadiness);
            record.command = "cargo test token=raw".to_owned();
            (record, EvidenceValidationError::RawSecretLikeValue)
        },
    ] {
        assert_eq!(validate_evidence_record(&record), Err(expected));
    }
}

#[test]
fn live_readiness_base_validation_covers_command_class_non_claim_shapes() {
    for (command_class, scopes) in [
        (
            ImplementationCommandClass::Format,
            vec![ImplementationNonClaimScope::CommandTargetSuccessNotClaimed],
        ),
        (
            ImplementationCommandClass::Build,
            vec![
                ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
                ImplementationNonClaimScope::ProductionReadinessNotClaimed,
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            ImplementationCommandClass::Test,
            vec![
                ImplementationNonClaimScope::ProductionReadinessNotClaimed,
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            ImplementationCommandClass::Benchmark,
            vec![
                ImplementationNonClaimScope::BenchmarkThresholdNotClaimed,
                ImplementationNonClaimScope::ProductionReadinessNotClaimed,
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            ImplementationCommandClass::RealDevice,
            vec![
                ImplementationNonClaimScope::NativeApplicationReadinessNotClaimed,
                ImplementationNonClaimScope::PublicDistributionReadinessNotClaimed,
                ImplementationNonClaimScope::ProductionReadinessNotClaimed,
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            ImplementationCommandClass::ProductionReadiness,
            vec![
                ImplementationNonClaimScope::LiveReadinessNotClaimed,
                ImplementationNonClaimScope::KernelCompletionNotClaimed,
                ImplementationNonClaimScope::KernelFreezeNotClaimed,
            ],
        ),
    ] {
        let mut record = base(command_class);
        record.non_claim_scope = scopes;
        assert_eq!(validate_evidence_record(&record), Ok(()));
    }
}

#[test]
fn live_readiness_required_refs_accept_only_their_own_gate_field() {
    for (gate_id, field) in [
        ("LIVE-001", "production_readiness_report_ref"),
        ("LIVE-002", "live_endpoint_evidence_ref"),
        ("LIVE-003", "public_traversal_evidence_ref"),
        ("LIVE-004", "monitoring_probe_ref"),
        ("LIVE-005", "rollback_drain_execution_ref"),
        ("LIVE-006", "shutdown_drain_evidence_ref"),
        ("LIVE-007", "restore_evidence_ref"),
        ("LIVE-008", "closed_gate_report_ref"),
    ] {
        assert_eq!(
            validate_readiness_evidence_record(&live_record(gate_id, field), &admitted_context()),
            Ok(()),
            "{gate_id}"
        );

        let mut missing = live_record(gate_id, field);
        match field {
            "production_readiness_report_ref" => missing.production_readiness_report_ref = None,
            "live_endpoint_evidence_ref" => missing.live_endpoint_evidence_ref = None,
            "public_traversal_evidence_ref" => missing.public_traversal_evidence_ref = None,
            "monitoring_probe_ref" => missing.monitoring_probe_ref = None,
            "rollback_drain_execution_ref" => missing.rollback_drain_execution_ref = None,
            "shutdown_drain_evidence_ref" => missing.shutdown_drain_evidence_ref = None,
            "restore_evidence_ref" => missing.restore_evidence_ref = None,
            "closed_gate_report_ref" => missing.closed_gate_report_ref = None,
            _ => unreachable!("unknown LIVE field"),
        }
        assert_eq!(
            validate_readiness_evidence_record(&missing, &admitted_context()),
            Err(ReadinessEvidenceValidationError::MissingRequiredExtensionRef),
            "{gate_id}"
        );
    }
}

#[test]
fn live_readiness_shared_validator_covers_production_gate_set_without_production_claim() {
    for (gate_id, field) in [
        ("PRD-001", "build_evidence_ref"),
        ("PRD-002", "behavior_test_evidence_ref"),
        ("PRD-003", "auth_provider_admission_ref"),
        ("PRD-004", "persistence_provider_admission_ref"),
        ("PRD-005", "deployment_profile_ref"),
        ("PRD-006", "monitoring_probe_ref"),
        ("PRD-007", "rollback_plan_ref"),
        ("PRD-008", "security_scan_ref"),
        ("PRD-009", "closed_gate_report_ref"),
    ] {
        assert_eq!(
            validate_readiness_evidence_record(&prd_record(gate_id, field), &admitted_context()),
            Ok(()),
            "{gate_id}"
        );
    }

    let mut no_auth = admitted_context();
    no_auth.auth_provider_authority = ReadinessAdmissionState::Absent;
    assert_eq!(
        validate_readiness_evidence_record(
            &prd_record("PRD-003", "auth_provider_admission_ref"),
            &no_auth,
        ),
        Err(ReadinessEvidenceValidationError::AuthProviderAuthorityNotAdmitted)
    );

    let mut no_persistence = admitted_context();
    no_persistence.persistence_provider_authority = ReadinessAdmissionState::Absent;
    assert_eq!(
        validate_readiness_evidence_record(
            &prd_record("PRD-004", "persistence_provider_admission_ref"),
            &no_persistence,
        ),
        Err(ReadinessEvidenceValidationError::PersistenceProviderAuthorityNotAdmitted)
    );

    let mut unknown_prd_gate = prd_record("PRD-001", "build_evidence_ref");
    unknown_prd_gate.readiness_gate_id = "PRD-999".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&unknown_prd_gate, &admitted_context()),
        Err(ReadinessEvidenceValidationError::GateIdNotListed)
    );
}

#[test]
fn live_readiness_extension_covers_authority_and_ref_fail_closed_branches() {
    let mut command_mismatch = live_record("LIVE-001", "production_readiness_report_ref");
    command_mismatch.readiness_claim = ReadinessClaim::ProductionReadiness;
    assert_eq!(
        validate_readiness_evidence_record(&command_mismatch, &admitted_context()),
        Err(ReadinessEvidenceValidationError::CommandClassClaimMismatch)
    );

    let mut claim_mismatch = live_record("LIVE-001", "production_readiness_report_ref");
    claim_mismatch.readiness_gate_id = "PRD-001".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&claim_mismatch, &admitted_context()),
        Err(ReadinessEvidenceValidationError::ReadinessGateClaimMismatch)
    );

    let mut empty_optional_ref = live_record("LIVE-004", "monitoring_probe_ref");
    empty_optional_ref.monitoring_probe_ref = Some(" ".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&empty_optional_ref, &admitted_context()),
        Err(ReadinessEvidenceValidationError::EmptyRequiredExtensionRef)
    );

    let mut secret_ref = live_record("LIVE-003", "public_traversal_evidence_ref");
    secret_ref.public_traversal_evidence_ref = Some("token=raw".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&secret_ref, &admitted_context()),
        Err(ReadinessEvidenceValidationError::SecretLikeExtensionRef)
    );

    let mut unexpected_ref = live_record("LIVE-002", "live_endpoint_evidence_ref");
    unexpected_ref.restore_evidence_ref = Some("coverage:unexpected".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&unexpected_ref, &admitted_context()),
        Err(ReadinessEvidenceValidationError::UnexpectedExtensionRefPopulated)
    );

    let mut no_live_authority = admitted_context();
    no_live_authority.live_endpoint_authority = ReadinessAdmissionState::Absent;
    assert_eq!(
        validate_readiness_evidence_record(
            &live_record("LIVE-002", "live_endpoint_evidence_ref"),
            &no_live_authority,
        ),
        Err(ReadinessEvidenceValidationError::LiveEndpointAuthorityNotAdmitted)
    );

    let mut no_production_report = admitted_context();
    no_production_report.production_readiness_report = ReadinessAdmissionState::Absent;
    assert_eq!(
        validate_readiness_evidence_record(
            &live_record("LIVE-001", "production_readiness_report_ref"),
            &no_production_report,
        ),
        Err(ReadinessEvidenceValidationError::ProductionReadinessReportNotAdmitted)
    );

    let mut no_public_traversal = admitted_context();
    no_public_traversal.public_traversal_authority = ReadinessAdmissionState::Absent;
    assert_eq!(
        validate_readiness_evidence_record(
            &live_record("LIVE-003", "public_traversal_evidence_ref"),
            &no_public_traversal,
        ),
        Err(ReadinessEvidenceValidationError::PublicTraversalAuthorityNotAdmitted)
    );
}

#[test]
fn live_readiness_endpoint_profile_and_drain_cover_admitted_and_deferred_branches() {
    let endpoint = admit_product_live_endpoint(ProductLiveEndpointClass::ControlledPublicEndpoint)
        .expect("controlled public endpoint is admitted");
    let profile = build_product_live_profile(&endpoint).expect("live profile is admitted");
    assert!(profile.public_endpoint_claimed);

    assert_eq!(
        admit_product_live_endpoint(ProductLiveEndpointClass::NotAdmitted),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
    assert_eq!(
        admit_public_traversal(ProductPublicTraversalClass::NotAdmitted),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );

    let rejected_profile = ProductLiveEndpointAdmission {
        endpoint_class: ProductLiveEndpointClass::ControlledPublicEndpoint,
        endpoint_boundary: "controlled-live-public-endpoint",
        live_endpoint_evidence_boundary: "live-endpoint-evidence-ref-required",
        implementation_reason: ImplementationEvidenceReason::ReadinessNotAdmitted,
    };
    assert_eq!(
        build_product_live_profile(&rejected_profile),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );

    assert_eq!(
        build_live_monitoring_probe(
            cid("live-coverage-empty-probe"),
            ImplementationPlane::Monitoring,
            " ",
        ),
        Err(ProductMonitoringError::EvidenceFieldsIncomplete)
    );

    let deferred = plan_drain(
        cid("live-coverage-deferred-drain"),
        vec![ImplementationPlane::Sfu],
        ProductDrainMode::ProductionDeferred,
    );
    assert_eq!(
        deferred.implementation_reason,
        ImplementationEvidenceReason::ReadinessNotAdmitted
    );

    assert_eq!(
        execute_live_shutdown_drain(cid("live-coverage-empty-drain"), Vec::new()),
        Err(arcrtc_product_rollback::ProductRollbackError::RuntimeExecutorError)
    );
}

#[test]
fn live_readiness_product_runtime_profiles_cover_all_host_classes() {
    for (host_class, profile_name, environment_class, public_endpoint_claimed, reason) in [
        (
            ProductHostClass::LocalSingleHost,
            "product-local-single-host",
            ImplementationEnvironmentClass::LocalSingleHost,
            false,
            ImplementationEvidenceReason::ImplementationOk,
        ),
        (
            ProductHostClass::ControlledMultiProcess,
            "product-controlled-multi-process",
            ImplementationEnvironmentClass::ControlledProcess,
            false,
            ImplementationEvidenceReason::ImplementationOk,
        ),
        (
            ProductHostClass::ProductionDeferred,
            "product-production-deferred",
            ImplementationEnvironmentClass::ProductionDeferred,
            false,
            ImplementationEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::ProductionAdmitted,
            "product-production-admitted",
            ImplementationEnvironmentClass::ProductionDeferred,
            false,
            ImplementationEvidenceReason::ImplementationOk,
        ),
        (
            ProductHostClass::LiveDeferred,
            "product-live-deferred",
            ImplementationEnvironmentClass::LiveDeferred,
            false,
            ImplementationEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::LiveAdmitted,
            "product-live-admitted",
            ImplementationEnvironmentClass::LiveDeferred,
            true,
            ImplementationEvidenceReason::ImplementationOk,
        ),
    ] {
        let profile = build_product_runtime_profile(host_class);
        assert_eq!(profile.profile_name, profile_name);
        assert_eq!(profile.environment_class, environment_class);
        assert_eq!(profile.public_endpoint_claimed, public_endpoint_claimed);
        assert_eq!(
            select_product_runtime(&profile).implementation_reason,
            reason
        );
    }
}

#[test]
fn live_readiness_product_runtime_lifecycle_covers_success_and_fail_closed() {
    let mut runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LiveAdmitted,
    ));
    assert_eq!(runtime.state(), ImplementationRuntimeState::Created);

    let started = runtime
        .start(cid("live-coverage-runtime-start"))
        .expect("live admitted runtime must start");
    assert_eq!(started.state, ImplementationRuntimeState::Running);

    let drained = runtime
        .drain(plan_drain(
            cid("live-coverage-runtime-drain"),
            vec![ImplementationPlane::Deployment],
            ProductDrainMode::LiveAdmitted,
        ))
        .expect("running runtime must drain");
    assert_eq!(drained.state, ImplementationRuntimeState::Stopped);

    let restored = runtime
        .restore(plan_restore(
            cid("live-coverage-runtime-restore"),
            ProductRestoreSource::LiveEvidenceReport,
        ))
        .expect("stopped runtime must restore");
    assert_eq!(restored.state, ImplementationRuntimeState::Running);

    assert_eq!(
        runtime.start(cid("live-coverage-start-twice")),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );

    let mut deferred_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LiveDeferred,
    ));
    assert_eq!(
        deferred_runtime.start(cid("live-coverage-deferred-start")),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );

    let mut created_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    assert_eq!(
        created_runtime.drain(plan_drain(
            cid("live-coverage-drain-before-start"),
            vec![ImplementationPlane::Deployment],
            ProductDrainMode::LiveAdmitted,
        )),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );

    let mut running_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LiveAdmitted,
    ));
    running_runtime
        .start(cid("live-coverage-running-runtime"))
        .expect("live admitted runtime must start");
    assert_eq!(
        running_runtime.restore(plan_restore(
            cid("live-coverage-restore-while-running"),
            ProductRestoreSource::LiveEvidenceReport,
        )),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );

    let public_endpoint_mismatch = ProductRuntimeProfile {
        profile_name: "live-coverage-public-mismatch",
        host_class: ProductHostClass::ProductionAdmitted,
        environment_class: ImplementationEnvironmentClass::ProductionDeferred,
        public_endpoint_claimed: true,
    };
    let mut mismatch_runtime = ProductRuntime::new(public_endpoint_mismatch);
    assert_eq!(
        mismatch_runtime.start(cid("live-coverage-public-mismatch")),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
}

#[test]
fn live_readiness_product_plane_descriptors_cover_non_live_runtime_builders() {
    let signaling = build_product_signaling_runtime();
    assert_eq!(signaling.target_plane, ImplementationPlane::Signaling);
    assert!(!signaling.public_endpoint_claimed);
    assert!(signaling.live_endpoint_evidence_ref.is_none());

    let turn = build_product_turn_runtime();
    assert_eq!(turn.target_plane, ImplementationPlane::Turn);
    assert!(!turn.relay_public_endpoint_claimed);
    assert!(turn.live_endpoint_evidence_ref.is_none());

    let sfu = build_product_sfu_runtime();
    assert_eq!(sfu.target_plane, ImplementationPlane::Sfu);
    assert!(!sfu.media_public_endpoint_claimed);
    assert!(sfu.live_endpoint_evidence_ref.is_none());
}

#[test]
fn live_readiness_operation_wrappers_cover_success_and_error_mappings() {
    let shutdown = execute_live_shutdown_drain(
        cid("live-coverage-operation-shutdown"),
        vec![ImplementationPlane::Deployment],
    )
    .expect("live shutdown drain must be admitted");
    assert_eq!(
        shutdown.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );

    assert_eq!(
        execute_live_shutdown_drain(cid("live-coverage-operation-empty-drain"), Vec::new()),
        Err(ProductRollbackError::RuntimeExecutorError)
    );

    let restore = execute_live_restore(cid("live-coverage-operation-restore"))
        .expect("live restore must use live evidence report source");
    assert_eq!(
        restore.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );
}
