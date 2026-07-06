//! Five-item coverage closure tests for production-readiness proof surfaces.

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_distro_evidence::{
    validate_evidence_record, validate_readiness_evidence_record, EvidenceValidationError,
    DistroCommandClass, DistroEnvironmentClass, DistroEvidenceReason,
    DistroEvidenceRecord, DistroLayer, DistroNonClaimScope,
    DistroPlane, ReadinessAdmissionState, ReadinessClaim, ReadinessEvidenceRecord,
    ReadinessEvidenceValidationError, ReadinessValidationContext, DISTRO_COMMAND_ROOT,
};
use arcrtc_product_deployment::{
    build_product_runtime_profile, runtime::DistroRuntimeState, select_product_runtime,
    ProductHostClass, ProductRuntime, ProductRuntimeError, ProductRuntimeProfile,
};
use arcrtc_product_monitoring::{build_production_monitoring_probe, ProductMonitoringError};
use arcrtc_product_rollback::{
    plan_drain, plan_production_drain, plan_production_restore, plan_restore, ProductDrainMode,
    ProductRestoreSource, ProductRollbackError,
};

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(
        OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
            .expect("test correlation id must be accepted"),
    )
}

fn base(command_class: DistroCommandClass) -> DistroEvidenceRecord {
    DistroEvidenceRecord {
        correlation_id: "production-readiness-coverage-closure".to_owned(),
        command: "cargo test --manifest-path tests/production-readiness/Cargo.toml".to_owned(),
        working_directory: DISTRO_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-distro-production-readiness-tests".to_owned()),
        target_scope: "tests/production-readiness".to_owned(),
        command_class,
        distro_layer: DistroLayer::Readiness,
        target_plane: DistroPlane::Deployment,
        environment_class: DistroEnvironmentClass::ProductionDeferred,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("production-readiness-coverage-closure".to_owned()),
        expected_outcome: "production readiness coverage branches are closed".to_owned(),
        actual_outcome: "production readiness coverage branches are closed".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: vec![
            DistroNonClaimScope::LiveReadinessNotClaimed,
            DistroNonClaimScope::KernelCompletionNotClaimed,
            DistroNonClaimScope::KernelFreezeNotClaimed,
        ],
        rerun_condition: "rerun when production readiness coverage closure changes".to_owned(),
    }
}

fn context() -> ReadinessValidationContext {
    ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    }
}

fn prd_record(gate_id: &str, required_ref: &str) -> ReadinessEvidenceRecord {
    let mut record = ReadinessEvidenceRecord {
        base: base(DistroCommandClass::ProductionReadiness),
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

fn live_record(gate_id: &str, required_ref: &str) -> ReadinessEvidenceRecord {
    let mut record = ReadinessEvidenceRecord {
        base: base(DistroCommandClass::LiveReadiness),
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

#[test]
fn production_readiness_base_validation_covers_empty_required_fields() {
    for (record, expected) in [
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.correlation_id.clear();
            (record, EvidenceValidationError::EmptyCorrelationId)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.command.clear();
            (record, EvidenceValidationError::EmptyCommand)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.working_directory.clear();
            (record, EvidenceValidationError::EmptyWorkingDirectory)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.target_scope.clear();
            (record, EvidenceValidationError::EmptyTargetScope)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.toolchain_runtime_version.clear();
            (
                record,
                EvidenceValidationError::EmptyToolchainRuntimeVersion,
            )
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.expected_outcome.clear();
            (record, EvidenceValidationError::EmptyExpectedOutcome)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.actual_outcome.clear();
            (record, EvidenceValidationError::EmptyActualOutcome)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.rerun_condition.clear();
            (record, EvidenceValidationError::EmptyRerunCondition)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.non_claim_scope.clear();
            (record, EvidenceValidationError::EmptyNonClaimScope)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.target_package = Some("   ".to_owned());
            (
                record,
                EvidenceValidationError::MissingRequiredTargetPackage,
            )
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.input_fixture_or_workload = None;
            (
                record,
                EvidenceValidationError::MissingRequiredInputFixtureOrWorkload,
            )
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.exit_status = None;
            (record, EvidenceValidationError::MissingRequiredExitStatus)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record
                .non_claim_scope
                .retain(|scope| scope != &DistroNonClaimScope::KernelFreezeNotClaimed);
            (
                record,
                EvidenceValidationError::MissingRequiredNonClaimScope,
            )
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.kernel_reason = Some("Other".to_owned());
            (record, EvidenceValidationError::ForbiddenUnclassifiedReason)
        },
        {
            let mut record = base(DistroCommandClass::ProductionReadiness);
            record.target_package = Some("token=raw".to_owned());
            (record, EvidenceValidationError::RawSecretLikeValue)
        },
    ] {
        assert_eq!(validate_evidence_record(&record), Err(expected));
    }
}

#[test]
fn production_readiness_base_validation_covers_command_class_non_claim_shapes() {
    for (command_class, scopes) in [
        (
            DistroCommandClass::Format,
            vec![DistroNonClaimScope::CommandTargetSuccessNotClaimed],
        ),
        (
            DistroCommandClass::Build,
            vec![
                DistroNonClaimScope::BehaviorCorrectnessNotClaimed,
                DistroNonClaimScope::ProductionReadinessNotClaimed,
                DistroNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            DistroCommandClass::Test,
            vec![
                DistroNonClaimScope::ProductionReadinessNotClaimed,
                DistroNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            DistroCommandClass::Benchmark,
            vec![
                DistroNonClaimScope::BenchmarkThresholdNotClaimed,
                DistroNonClaimScope::ProductionReadinessNotClaimed,
                DistroNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            DistroCommandClass::RealDevice,
            vec![
                DistroNonClaimScope::NativeApplicationReadinessNotClaimed,
                DistroNonClaimScope::PublicDistributionReadinessNotClaimed,
                DistroNonClaimScope::ProductionReadinessNotClaimed,
                DistroNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
    ] {
        let mut record = base(command_class);
        record.non_claim_scope = scopes;
        assert_eq!(validate_evidence_record(&record), Ok(()));
    }
}

#[test]
fn production_readiness_required_prd_refs_accept_only_their_own_gate_field() {
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
            validate_readiness_evidence_record(&prd_record(gate_id, field), &context()),
            Ok(()),
            "{gate_id}"
        );

        let mut missing = prd_record(gate_id, field);
        match field {
            "build_evidence_ref" => missing.build_evidence_ref = None,
            "behavior_test_evidence_ref" => missing.behavior_test_evidence_ref = None,
            "auth_provider_admission_ref" => missing.auth_provider_admission_ref = None,
            "persistence_provider_admission_ref" => {
                missing.persistence_provider_admission_ref = None
            }
            "deployment_profile_ref" => missing.deployment_profile_ref = None,
            "monitoring_probe_ref" => missing.monitoring_probe_ref = None,
            "rollback_plan_ref" => missing.rollback_plan_ref = None,
            "security_scan_ref" => missing.security_scan_ref = None,
            "closed_gate_report_ref" => missing.closed_gate_report_ref = None,
            _ => unreachable!("unknown PRD field"),
        }
        assert_eq!(
            validate_readiness_evidence_record(&missing, &context()),
            Err(ReadinessEvidenceValidationError::MissingRequiredExtensionRef),
            "{gate_id}"
        );
    }
}

#[test]
fn production_readiness_shared_validator_covers_live_gate_set_without_live_claim() {
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
            validate_readiness_evidence_record(&live_record(gate_id, field), &context()),
            Ok(()),
            "{gate_id}"
        );
    }

    let mut unknown_live_gate = live_record("LIVE-001", "production_readiness_report_ref");
    unknown_live_gate.readiness_gate_id = "LIVE-999".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&unknown_live_gate, &context()),
        Err(ReadinessEvidenceValidationError::GateIdNotListed)
    );
}

#[test]
fn production_readiness_extension_covers_ref_and_authority_fail_closed_branches() {
    let mut command_mismatch = prd_record("PRD-001", "build_evidence_ref");
    command_mismatch.base.command_class = DistroCommandClass::LiveReadiness;
    assert_eq!(
        validate_readiness_evidence_record(&command_mismatch, &context()),
        Err(ReadinessEvidenceValidationError::CommandClassClaimMismatch)
    );

    let mut claim_mismatch = prd_record("PRD-001", "build_evidence_ref");
    claim_mismatch.readiness_gate_id = "LIVE-001".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&claim_mismatch, &context()),
        Err(ReadinessEvidenceValidationError::ReadinessGateClaimMismatch)
    );

    let mut unknown_gate = prd_record("PRD-001", "build_evidence_ref");
    unknown_gate.readiness_gate_id = "PRD-999".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&unknown_gate, &context()),
        Err(ReadinessEvidenceValidationError::GateIdNotListed)
    );

    let mut empty_common_ref = prd_record("PRD-001", "build_evidence_ref");
    empty_common_ref.readiness_adr_ref = " ".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&empty_common_ref, &context()),
        Err(ReadinessEvidenceValidationError::EmptyRequiredExtensionRef)
    );

    let mut mismatched_common_ref = prd_record("PRD-001", "build_evidence_ref");
    mismatched_common_ref.readiness_canonical_ref = "OTHER_AUTHORITY_MATRIX".to_owned();
    assert_eq!(
        validate_readiness_evidence_record(&mismatched_common_ref, &context()),
        Err(ReadinessEvidenceValidationError::ReadinessAuthorityRefMismatch)
    );

    let mut secret_ref = prd_record("PRD-008", "security_scan_ref");
    secret_ref.security_scan_ref = Some("token=raw-secret".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&secret_ref, &context()),
        Err(ReadinessEvidenceValidationError::SecretLikeExtensionRef)
    );

    let mut unexpected_ref = prd_record("PRD-001", "build_evidence_ref");
    unexpected_ref.monitoring_probe_ref = Some("coverage:unexpected".to_owned());
    assert_eq!(
        validate_readiness_evidence_record(&unexpected_ref, &context()),
        Err(ReadinessEvidenceValidationError::UnexpectedExtensionRefPopulated)
    );

    let mut no_auth_context = context();
    no_auth_context.auth_provider_authority = ReadinessAdmissionState::Absent;
    assert_eq!(
        validate_readiness_evidence_record(
            &prd_record("PRD-003", "auth_provider_admission_ref"),
            &no_auth_context,
        ),
        Err(ReadinessEvidenceValidationError::AuthProviderAuthorityNotAdmitted)
    );

    let mut no_persistence_context = context();
    no_persistence_context.persistence_provider_authority = ReadinessAdmissionState::Absent;
    assert_eq!(
        validate_readiness_evidence_record(
            &prd_record("PRD-004", "persistence_provider_admission_ref"),
            &no_persistence_context,
        ),
        Err(ReadinessEvidenceValidationError::PersistenceProviderAuthorityNotAdmitted)
    );
}

#[test]
fn production_readiness_probe_rejects_empty_metric_name() {
    assert_eq!(
        build_production_monitoring_probe(
            cid("production-coverage-empty-probe"),
            DistroPlane::Monitoring,
            " ",
        ),
        Err(ProductMonitoringError::EvidenceFieldsIncomplete)
    );
}

#[test]
fn production_readiness_drain_and_restore_plans_cover_deferred_reason_branches() {
    let empty_plan = plan_drain(
        cid("production-coverage-empty-drain"),
        Vec::new(),
        ProductDrainMode::ControlledProduct,
    );
    assert_eq!(
        empty_plan.distro_reason,
        DistroEvidenceReason::RuntimeExecutorError
    );

    let deferred_plan = plan_drain(
        cid("production-coverage-deferred-drain"),
        vec![DistroPlane::Deployment],
        ProductDrainMode::ProductionDeferred,
    );
    assert_eq!(
        deferred_plan.distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );

    let provider_restore = plan_restore(
        cid("production-coverage-provider-restore"),
        ProductRestoreSource::ProviderDeferred,
    );
    assert_eq!(
        provider_restore.distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );

    let in_memory_restore = plan_restore(
        cid("production-coverage-in-memory-restore"),
        ProductRestoreSource::InMemoryProjection,
    );
    assert_eq!(
        in_memory_restore.distro_reason,
        DistroEvidenceReason::DistroOk
    );
}

#[test]
fn production_readiness_product_runtime_profiles_cover_all_host_classes() {
    for (host_class, profile_name, environment_class, public_endpoint_claimed, reason) in [
        (
            ProductHostClass::LocalSingleHost,
            "product-local-single-host",
            DistroEnvironmentClass::LocalSingleHost,
            false,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::ControlledMultiProcess,
            "product-controlled-multi-process",
            DistroEnvironmentClass::ControlledProcess,
            false,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::ProductionDeferred,
            "product-production-deferred",
            DistroEnvironmentClass::ProductionDeferred,
            false,
            DistroEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::ProductionAdmitted,
            "product-production-admitted",
            DistroEnvironmentClass::ProductionDeferred,
            false,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::LiveDeferred,
            "product-live-deferred",
            DistroEnvironmentClass::LiveDeferred,
            false,
            DistroEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::LiveAdmitted,
            "product-live-admitted",
            DistroEnvironmentClass::LiveDeferred,
            true,
            DistroEvidenceReason::DistroOk,
        ),
    ] {
        let profile = build_product_runtime_profile(host_class);
        assert_eq!(profile.profile_name, profile_name);
        assert_eq!(profile.environment_class, environment_class);
        assert_eq!(profile.public_endpoint_claimed, public_endpoint_claimed);
        assert_eq!(
            select_product_runtime(&profile).distro_reason,
            reason
        );
    }
}

#[test]
fn production_readiness_product_runtime_lifecycle_covers_success_and_fail_closed() {
    let mut runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    assert_eq!(runtime.state(), DistroRuntimeState::Created);

    let started = runtime
        .start(cid("production-coverage-runtime-start"))
        .expect("local runtime must start");
    assert_eq!(started.state, DistroRuntimeState::Running);

    let drain = plan_drain(
        cid("production-coverage-runtime-drain"),
        vec![DistroPlane::Deployment],
        ProductDrainMode::ControlledProduct,
    );
    let drained = runtime.drain(drain).expect("running runtime must drain");
    assert_eq!(drained.state, DistroRuntimeState::Stopped);

    let restored = runtime
        .restore(plan_restore(
            cid("production-coverage-runtime-restore"),
            ProductRestoreSource::EvidenceReport,
        ))
        .expect("stopped runtime must restore");
    assert_eq!(restored.state, DistroRuntimeState::Running);
    assert!(restored
        .non_claim_scope
        .contains(&DistroNonClaimScope::BehaviorCorrectnessNotClaimed));

    assert_eq!(
        runtime.start(cid("production-coverage-start-twice")),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );
    assert_eq!(runtime.state(), DistroRuntimeState::Failed);

    let mut deferred_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::ProductionDeferred,
    ));
    assert_eq!(
        deferred_runtime.start(cid("production-coverage-deferred-start")),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );

    let mut created_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    assert_eq!(
        created_runtime.drain(plan_drain(
            cid("production-coverage-drain-before-start"),
            vec![DistroPlane::Deployment],
            ProductDrainMode::ControlledProduct,
        )),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );

    let mut running_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    running_runtime
        .start(cid("production-coverage-running-runtime"))
        .expect("local runtime must start");
    assert_eq!(
        running_runtime.restore(plan_restore(
            cid("production-coverage-restore-while-running"),
            ProductRestoreSource::EvidenceReport,
        )),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );

    let public_endpoint_mismatch = ProductRuntimeProfile {
        profile_name: "production-coverage-public-mismatch",
        host_class: ProductHostClass::ProductionAdmitted,
        environment_class: DistroEnvironmentClass::ProductionDeferred,
        public_endpoint_claimed: true,
    };
    let mut mismatch_runtime = ProductRuntime::new(public_endpoint_mismatch);
    assert_eq!(
        mismatch_runtime.start(cid("production-coverage-public-mismatch")),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
}

#[test]
fn production_readiness_operation_wrappers_cover_error_mappings() {
    let production_drain = plan_production_drain(
        cid("production-coverage-operation-drain"),
        vec![DistroPlane::Deployment],
    )
    .expect("production drain must be admitted for controlled product");
    assert_eq!(
        production_drain.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    assert_eq!(
        plan_production_drain(cid("production-coverage-empty-operation-drain"), Vec::new()),
        Err(ProductRollbackError::RuntimeExecutorError)
    );

    let production_restore = plan_production_restore(cid("production-coverage-operation-restore"))
        .expect("production restore must use evidence report source");
    assert_eq!(
        production_restore.distro_reason,
        DistroEvidenceReason::DistroOk
    );
}
