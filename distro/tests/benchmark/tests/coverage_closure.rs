//! Five-item coverage closure tests for benchmark threshold proof surfaces.

#[path = "../src/threshold.rs"]
mod threshold;

use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CorrelationId, CredentialRef, EndpointId, OpaqueReference,
    PacketId, ParticipantId, PermissionId, ReferenceAuthority, RoomId, RouteId, SessionId,
    StreamId,
};
use arcrtc_core_sfu::SfuDecisionKind;
use arcrtc_core_signaling::{SignalingCommandKind, SignalingEventKind};
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommandKind, TurnDecisionKind, TurnReferenceSet,
    TurnRequestedLifetimeSeconds, TurnTransactionId,
};
use arcrtc_distro_benchmark_tests::{
    build_benchmark_evidence_record, scenario_by_id, validate_benchmark_evidence_record,
    BenchmarkEvidenceValidationError, BenchmarkMeasurement, BenchmarkWorkload, BENCHMARK_SCENARIOS,
};
use arcrtc_distro_evidence::{
    validate_evidence_record, EvidenceValidationError, DistroCommandClass,
    DistroEnvironmentClass, DistroEvidenceReason, DistroEvidenceRecord,
    DistroLayer, DistroNonClaimScope, DistroPlane,
    DISTRO_COMMAND_ROOT,
};
use arcrtc_product_deployment::{
    build_product_runtime_profile, runtime::DistroRuntimeState as ProductRuntimeState,
    select_product_runtime, ProductHostClass, ProductRuntime, ProductRuntimeError,
    ProductRuntimeProfile,
};
use arcrtc_product_persistence_topology::{
    map_product_projection, ProductPersistenceRecordClass, ProductPersistenceTopologyError,
};
use arcrtc_product_rollback::{plan_drain, plan_restore, ProductDrainMode, ProductRestoreSource};
use arcrtc_reference_composition::ReferenceCompositionState;
use arcrtc_reference_ops::{
    build_reference_build_evidence_record, write_distro_evidence_record,
    DistroRuntimeState as ReferenceRuntimeState, DistroShutdownMode,
    ReferenceBuildEvidenceInput, ReferenceRuntime, ReferenceRuntimeError,
};
use arcrtc_reference_sfu::{
    apply_reference_sfu, state::SfuEndpointState, state::SfuRouteState, state::SfuSessionState,
    validate_reference_sfu_state, ReferenceEndpointState, ReferenceRouteState, ReferenceSfuAction,
    ReferenceSfuError, ReferenceSfuSessionState, ReferenceSfuState, ReferenceSfuSuppressionSource,
};
use arcrtc_reference_signaling::fixture_identity::FixtureSessionDescriptionDirection;
use arcrtc_reference_signaling::state::signaling_success_event_kind;
use arcrtc_reference_signaling::{
    apply_reference_signaling, build_kernel_signaling_command, project_reference_signaling_event,
    validate_reference_signaling_state, FixtureIceCandidate, FixtureSessionDescription,
    ReferenceParticipantPhase, ReferenceParticipantState, ReferenceRoomPhase, ReferenceRoomState,
    ReferenceSignalingCommandInput, ReferenceSignalingError, ReferenceSignalingPayload,
    ReferenceSignalingState,
};
use arcrtc_reference_turn::{
    apply_reference_turn, state::turn_decision_kind, state::AllocationState,
    state::ChannelBindState, state::PermissionState, validate_reference_turn_state,
    ReferenceChannelBindState, ReferencePermissionState, ReferenceTurnCommandInput,
    ReferenceTurnError, ReferenceTurnState,
};
use threshold::{evaluate_kpi_benchmark_threshold_satisfaction, BenchmarkThresholdVerdict};

fn base() -> DistroEvidenceRecord {
    DistroEvidenceRecord {
        correlation_id: "benchmark-coverage-closure".to_owned(),
        command:
            "cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios"
                .to_owned(),
        working_directory: DISTRO_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-distro-benchmark-tests".to_owned()),
        target_scope: "tests/benchmark".to_owned(),
        command_class: DistroCommandClass::Benchmark,
        distro_layer: DistroLayer::Reference,
        target_plane: DistroPlane::Signaling,
        environment_class: DistroEnvironmentClass::BenchmarkHost,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("BENCH-001:signaling_join_room_single".to_owned()),
        expected_outcome: "benchmark coverage closure measurement emitted".to_owned(),
        actual_outcome: "benchmark coverage closure measurement emitted".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: vec![
            DistroNonClaimScope::BenchmarkThresholdNotClaimed,
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "rerun when benchmark coverage closure changes".to_owned(),
    }
}

fn measurement(id: &str, name: &str, p95_ns: f64) -> BenchmarkMeasurement {
    BenchmarkMeasurement {
        scenario_id: id.to_owned(),
        scenario_name: name.to_owned(),
        criterion_group: "distro_benchmark_scenarios".to_owned(),
        sample_size: 100,
        warm_up_seconds: 3,
        measurement_seconds: 10,
        noise_threshold: 0.05,
        confidence_level: 0.95,
        significance_level: 0.05,
        median_ns: p95_ns,
        mean_ns: p95_ns,
        std_dev_ns: 0.0,
        p95_ns,
        throughput_items_per_second: 1.0,
    }
}

fn canonical_measurement() -> BenchmarkMeasurement {
    measurement("BENCH-001", "signaling_join_room_single", 1.0)
}

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("benchmark coverage reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

fn room(value: &str) -> RoomId {
    RoomId::new(accepted(value))
}

fn participant(value: &str) -> ParticipantId {
    ParticipantId::new(accepted(value))
}

fn session(value: &str) -> SessionId {
    SessionId::new(accepted(value))
}

fn endpoint(value: &str) -> EndpointId {
    EndpointId::new(accepted(value))
}

fn stream(value: &str) -> StreamId {
    StreamId::new(accepted(value))
}

fn route(value: &str) -> RouteId {
    RouteId::new(accepted(value))
}

fn allocation(value: &str) -> AllocationId {
    AllocationId::new(accepted(value))
}

fn permission(value: &str) -> PermissionId {
    PermissionId::new(accepted(value))
}

fn channel_bind(value: &str) -> ChannelBindId {
    ChannelBindId::new(accepted(value))
}

fn credential(value: &str) -> CredentialRef {
    CredentialRef::new(accepted(value))
}

fn packet(value: &str) -> PacketId {
    PacketId::new(accepted(value))
}

fn transaction(value: &str) -> TurnTransactionId {
    TurnTransactionId::new(accepted(value))
}

fn signaling_input(
    kind: SignalingCommandKind,
    payload: ReferenceSignalingPayload,
) -> ReferenceSignalingCommandInput {
    ReferenceSignalingCommandInput::new(
        cid("benchmark-signaling-correlation"),
        room("benchmark-signaling-room"),
        Some(participant("benchmark-signaling-participant")),
        kind,
        payload,
    )
}

fn turn_input(kind: TurnCommandKind) -> ReferenceTurnCommandInput {
    ReferenceTurnCommandInput {
        kind,
        transaction_id: transaction("benchmark-turn-transaction"),
        references: TurnReferenceSet::new(
            Some(allocation("benchmark-turn-allocation")),
            Some(permission("benchmark-turn-permission")),
            Some(channel_bind("benchmark-turn-channel")),
            Some(credential("benchmark-turn-credential")),
        ),
        allocation_id: Some(allocation("benchmark-turn-allocation")),
        permission_id: Some(permission("benchmark-turn-permission")),
        channel_bind_id: Some(channel_bind("benchmark-turn-channel")),
        credential_ref: Some(credential("benchmark-turn-credential")),
        peer_address: Some(CorePeerAddress::new("192.0.2.44:3478").expect("peer address")),
        requested_lifetime: Some(TurnRequestedLifetimeSeconds::try_new(600).expect("lifetime")),
        relay_packet_id: Some(packet("benchmark-turn-packet")),
    }
}

#[test]
fn benchmark_scenario_lookup_and_workload_descriptor_cover_present_and_absent_paths() {
    for scenario in BENCHMARK_SCENARIOS {
        let found = scenario_by_id(scenario.id).expect("scenario must be listed");
        assert_eq!(found.name, scenario.name);
        let workload = BenchmarkWorkload::from_scenario(found);
        assert_eq!(
            workload.workload_id,
            format!("actual-workload:{}:{}", found.id, found.name)
        );
        assert_eq!(workload.workload_summary, found.workload_summary);
    }

    assert!(scenario_by_id("BENCH-999").is_none());
}

#[test]
fn benchmark_threshold_verdict_covers_invalid_unknown_boundary_pass_and_fail() {
    assert_eq!(
        evaluate_kpi_benchmark_threshold_satisfaction(&measurement(
            "BENCH-001",
            "signaling_join_room_single",
            711.481,
        )),
        Ok(BenchmarkThresholdVerdict::Pass)
    );
    assert_eq!(
        evaluate_kpi_benchmark_threshold_satisfaction(&measurement(
            "BENCH-001",
            "signaling_join_room_single",
            711.482,
        )),
        Ok(BenchmarkThresholdVerdict::Fail)
    );
    assert_eq!(
        evaluate_kpi_benchmark_threshold_satisfaction(&measurement(
            "BENCH-001",
            "signaling_join_room_single",
            -1.0,
        )),
        Err("invalid-measured-p95")
    );
    assert_eq!(
        evaluate_kpi_benchmark_threshold_satisfaction(&measurement("BENCH-999", "unknown", 0.0,)),
        Err("threshold-not-found")
    );
}

#[test]
fn benchmark_evidence_validation_covers_each_criterion_config_branch() {
    for mutate in [
        |measurement: &mut BenchmarkMeasurement| measurement.sample_size = 101,
        |measurement: &mut BenchmarkMeasurement| measurement.warm_up_seconds = 4,
        |measurement: &mut BenchmarkMeasurement| measurement.measurement_seconds = 11,
        |measurement: &mut BenchmarkMeasurement| measurement.noise_threshold = 0.06,
        |measurement: &mut BenchmarkMeasurement| measurement.confidence_level = 0.90,
        |measurement: &mut BenchmarkMeasurement| measurement.significance_level = 0.01,
    ] {
        let mut measurement = canonical_measurement();
        mutate(&mut measurement);
        assert_eq!(
            build_benchmark_evidence_record(base(), measurement).unwrap_err(),
            BenchmarkEvidenceValidationError::CriterionConfigurationMismatch
        );
    }
}

#[test]
fn benchmark_evidence_validation_covers_timing_and_throughput_rejection_branches() {
    for mutate in [
        |measurement: &mut BenchmarkMeasurement| measurement.median_ns = -1.0,
        |measurement: &mut BenchmarkMeasurement| measurement.mean_ns = f64::INFINITY,
        |measurement: &mut BenchmarkMeasurement| measurement.std_dev_ns = f64::NAN,
        |measurement: &mut BenchmarkMeasurement| measurement.p95_ns = -0.1,
    ] {
        let mut measurement = canonical_measurement();
        mutate(&mut measurement);
        assert_eq!(
            build_benchmark_evidence_record(base(), measurement).unwrap_err(),
            BenchmarkEvidenceValidationError::InvalidTimingValue
        );
    }

    for throughput in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        let mut measurement = canonical_measurement();
        measurement.throughput_items_per_second = throughput;
        assert_eq!(
            build_benchmark_evidence_record(base(), measurement).unwrap_err(),
            BenchmarkEvidenceValidationError::InvalidThroughputValue
        );
    }
}

#[test]
fn benchmark_evidence_record_validation_rejects_projection_mutations_after_build() {
    let record = build_benchmark_evidence_record(base(), canonical_measurement())
        .expect("canonical benchmark evidence must build");

    let mut missing_threshold_non_claim = record.clone();
    missing_threshold_non_claim
        .base
        .non_claim_scope
        .retain(|scope| scope != &DistroNonClaimScope::BenchmarkThresholdNotClaimed);
    assert_eq!(
        validate_benchmark_evidence_record(&missing_threshold_non_claim),
        Err(BenchmarkEvidenceValidationError::Base(
            EvidenceValidationError::MissingRequiredNonClaimScope
        ))
    );

    let mut layer_mismatch = record.clone();
    layer_mismatch.base.distro_layer = DistroLayer::Product;
    assert_eq!(
        validate_benchmark_evidence_record(&layer_mismatch),
        Err(BenchmarkEvidenceValidationError::ScenarioLayerMismatch)
    );

    let mut plane_mismatch = record.clone();
    plane_mismatch.base.target_plane = DistroPlane::Turn;
    assert_eq!(
        validate_benchmark_evidence_record(&plane_mismatch),
        Err(BenchmarkEvidenceValidationError::ScenarioPlaneMismatch)
    );

    let mut scenario_missing = record.clone();
    scenario_missing.scenario_id = "BENCH-999".to_owned();
    assert_eq!(
        validate_benchmark_evidence_record(&scenario_missing),
        Err(BenchmarkEvidenceValidationError::ScenarioIdNotListed)
    );
}

#[test]
fn benchmark_base_evidence_validation_covers_shared_fail_closed_branches() {
    for (record, expected) in [
        {
            let mut record = base();
            record.correlation_id.clear();
            (record, EvidenceValidationError::EmptyCorrelationId)
        },
        {
            let mut record = base();
            record.working_directory = "/tmp".to_owned();
            (
                record,
                EvidenceValidationError::WorkingDirectoryOutsideDistro,
            )
        },
        {
            let mut record = base();
            record.actual_outcome.clear();
            (record, EvidenceValidationError::EmptyActualOutcome)
        },
        {
            let mut record = base();
            record.target_package = Some(" ".to_owned());
            (
                record,
                EvidenceValidationError::MissingRequiredTargetPackage,
            )
        },
        {
            let mut record = base();
            record.input_fixture_or_workload = Some(" ".to_owned());
            (
                record,
                EvidenceValidationError::MissingRequiredInputFixtureOrWorkload,
            )
        },
        {
            let mut record = base();
            record.kernel_reason = Some("unknown".to_owned());
            (record, EvidenceValidationError::ForbiddenUnclassifiedReason)
        },
        {
            let mut record = base();
            record.command = "cargo bench token=raw".to_owned();
            (record, EvidenceValidationError::RawSecretLikeValue)
        },
    ] {
        assert_eq!(validate_evidence_record(&record), Err(expected));
    }

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
            DistroCommandClass::RealDevice,
            vec![
                DistroNonClaimScope::NativeApplicationReadinessNotClaimed,
                DistroNonClaimScope::PublicDistributionReadinessNotClaimed,
                DistroNonClaimScope::ProductionReadinessNotClaimed,
                DistroNonClaimScope::LiveReadinessNotClaimed,
            ],
        ),
        (
            DistroCommandClass::ProductionReadiness,
            vec![
                DistroNonClaimScope::LiveReadinessNotClaimed,
                DistroNonClaimScope::KernelCompletionNotClaimed,
                DistroNonClaimScope::KernelFreezeNotClaimed,
            ],
        ),
        (
            DistroCommandClass::LiveReadiness,
            vec![
                DistroNonClaimScope::KernelCompletionNotClaimed,
                DistroNonClaimScope::KernelFreezeNotClaimed,
            ],
        ),
    ] {
        let mut record = base();
        record.command_class = command_class;
        record.non_claim_scope = scopes;
        assert_eq!(
            validate_evidence_record(&record),
            Ok(()),
            "{command_class:?}"
        );
    }
}

#[test]
fn benchmark_product_runtime_and_projection_cover_alternate_branches() {
    for (host_class, reason) in [
        (
            ProductHostClass::LocalSingleHost,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::ControlledMultiProcess,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::ProductionDeferred,
            DistroEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::ProductionAdmitted,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::LiveDeferred,
            DistroEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::LiveAdmitted,
            DistroEvidenceReason::DistroOk,
        ),
    ] {
        let profile = build_product_runtime_profile(host_class);
        assert_eq!(
            select_product_runtime(&profile).distro_reason,
            reason
        );
    }

    let mut runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    assert_eq!(
        runtime
            .start(cid("benchmark-product-runtime-start"))
            .expect("runtime must start")
            .state,
        ProductRuntimeState::Running
    );
    assert_eq!(
        runtime
            .drain(plan_drain(
                cid("benchmark-product-runtime-drain"),
                vec![DistroPlane::Deployment],
                ProductDrainMode::ControlledProduct,
            ))
            .expect("runtime must drain")
            .state,
        ProductRuntimeState::Stopped
    );
    assert_eq!(
        runtime
            .restore(plan_restore(
                cid("benchmark-product-runtime-restore"),
                ProductRestoreSource::EvidenceReport,
            ))
            .expect("runtime must restore")
            .state,
        ProductRuntimeState::Running
    );
    assert_eq!(runtime.state(), ProductRuntimeState::Running);
    assert_eq!(
        runtime.start(cid("benchmark-product-start-twice")),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );

    let mut premature_drain = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    assert_eq!(
        premature_drain.drain(plan_drain(
            cid("benchmark-product-premature-drain"),
            vec![DistroPlane::Deployment],
            ProductDrainMode::ControlledProduct,
        )),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );

    let mut running_restore = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    running_restore
        .start(cid("benchmark-product-running-restore-start"))
        .expect("runtime must start");
    assert_eq!(
        running_restore.restore(plan_restore(
            cid("benchmark-product-running-restore"),
            ProductRestoreSource::InMemoryProjection,
        )),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );

    let mut deferred_restore = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    assert_eq!(
        deferred_restore.restore(plan_restore(
            cid("benchmark-product-deferred-restore"),
            ProductRestoreSource::ProviderDeferred,
        )),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
    assert_eq!(
        plan_restore(
            cid("benchmark-product-live-restore-plan"),
            ProductRestoreSource::LiveEvidenceReport,
        )
        .distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let public_endpoint_mismatch = ProductRuntimeProfile {
        profile_name: "benchmark-public-mismatch",
        host_class: ProductHostClass::ProductionAdmitted,
        environment_class: DistroEnvironmentClass::ProductionDeferred,
        public_endpoint_claimed: true,
    };
    assert_eq!(
        ProductRuntime::new(public_endpoint_mismatch).start(cid("benchmark-public-mismatch")),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );

    assert_eq!(
        map_product_projection(
            DistroPlane::Signaling,
            ProductPersistenceRecordClass::AllocationProjection,
        ),
        Err(ProductPersistenceTopologyError::ProjectionMappingViolation)
    );
    assert_eq!(
        plan_drain(
            cid("benchmark-empty-drain"),
            Vec::new(),
            ProductDrainMode::ReferenceLocal,
        )
        .distro_reason,
        DistroEvidenceReason::RuntimeExecutorError
    );
    assert_eq!(
        plan_drain(
            cid("benchmark-deferred-drain"),
            vec![DistroPlane::Deployment],
            ProductDrainMode::ProductionDeferred,
        )
        .distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );
}

#[test]
fn benchmark_reference_ops_runtime_and_evidence_cover_fail_closed_branches() {
    let build = build_reference_build_evidence_record(ReferenceBuildEvidenceInput {
        correlation_id: cid("benchmark-build-evidence"),
        command: "cargo build --workspace --all-targets".to_owned(),
        target_package: "arcrtc-reference-ops".to_owned(),
        target_scope: "reference-distro/ops".to_owned(),
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        expected_outcome: "build succeeds".to_owned(),
        actual_outcome: "build succeeds".to_owned(),
        exit_status: 0,
    })
    .expect("reference build evidence must build");
    assert_eq!(build.command_class, DistroCommandClass::Build);
    let build_write = write_distro_evidence_record(&build)
        .expect("reference build evidence must be writable");
    assert!(build_write.output_path.is_file());

    let mut non_reference_writer = build.clone();
    non_reference_writer.distro_layer = DistroLayer::Product;
    assert_eq!(
        write_distro_evidence_record(&non_reference_writer),
        Err(ReferenceRuntimeError::CommandScopeMismatch)
    );

    let mut unsafe_dot = build.clone();
    unsafe_dot.correlation_id = "..".to_owned();
    assert_eq!(
        write_distro_evidence_record(&unsafe_dot),
        Err(ReferenceRuntimeError::CommandScopeMismatch)
    );

    let mut unsafe_path = build.clone();
    unsafe_path.correlation_id = "unsafe/path".to_owned();
    assert_eq!(
        write_distro_evidence_record(&unsafe_path),
        Err(ReferenceRuntimeError::CommandScopeMismatch)
    );

    let mut runtime = ReferenceRuntime::new(ReferenceCompositionState::default());
    assert_eq!(runtime.state(), ReferenceRuntimeState::Created);
    assert_eq!(runtime.composition(), &ReferenceCompositionState::default());
    assert_eq!(
        runtime
            .start(cid("benchmark-reference-runtime-start"))
            .expect("reference runtime must start")
            .state,
        ReferenceRuntimeState::Running
    );
    assert_eq!(
        runtime
            .shutdown(
                cid("benchmark-reference-runtime-shutdown"),
                DistroShutdownMode::GracefulLocal,
            )
            .expect("reference runtime must shutdown")
            .state,
        ReferenceRuntimeState::Stopped
    );
    assert_eq!(
        runtime.start(cid("benchmark-reference-runtime-restart")),
        Err(ReferenceRuntimeError::RuntimeExecutorError)
    );

    let mut premature_shutdown = ReferenceRuntime::new(ReferenceCompositionState::default());
    assert_eq!(
        premature_shutdown.shutdown(
            cid("benchmark-reference-runtime-premature-shutdown"),
            DistroShutdownMode::Immediate,
        ),
        Err(ReferenceRuntimeError::RuntimeExecutorError)
    );

    for mode in [
        DistroShutdownMode::Immediate,
        DistroShutdownMode::DrainThenStop,
    ] {
        let mut runtime = ReferenceRuntime::new(ReferenceCompositionState::default());
        runtime
            .start(cid("benchmark-reference-runtime-mode-start"))
            .expect("reference runtime must start");
        assert_eq!(
            runtime
                .shutdown(cid("benchmark-reference-runtime-mode-stop"), mode)
                .expect("reference runtime must stop")
                .state,
            ReferenceRuntimeState::Stopped
        );
    }
}

#[test]
fn benchmark_reference_signaling_covers_command_and_state_rejection_branches() {
    let mut state = ReferenceSignalingState::default();
    let join = signaling_input(
        SignalingCommandKind::JoinRoom,
        ReferenceSignalingPayload::JoinRoom,
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &join)
            .expect("join must succeed")
            .kind,
        SignalingEventKind::Joined
    );

    for (kind, payload, expected_kind) in [
        (
            SignalingCommandKind::SendOffer,
            ReferenceSignalingPayload::SendOffer {
                session: FixtureSessionDescription {
                    session_id: session("benchmark-offer-session"),
                    fixture_sdp_id: "offer".to_owned(),
                    direction: FixtureSessionDescriptionDirection::Offer,
                },
            },
            SignalingEventKind::OfferReceived,
        ),
        (
            SignalingCommandKind::SendAnswer,
            ReferenceSignalingPayload::SendAnswer {
                session: FixtureSessionDescription {
                    session_id: session("benchmark-answer-session"),
                    fixture_sdp_id: "answer".to_owned(),
                    direction: FixtureSessionDescriptionDirection::Answer,
                },
            },
            SignalingEventKind::AnswerReceived,
        ),
        (
            SignalingCommandKind::RequestTurnCredential,
            ReferenceSignalingPayload::RequestTurnCredential,
            SignalingEventKind::TurnCredentialAvailable,
        ),
        (
            SignalingCommandKind::AcknowledgeForward,
            ReferenceSignalingPayload::AcknowledgeForward,
            SignalingEventKind::Joined,
        ),
    ] {
        let input = signaling_input(kind, payload);
        assert_eq!(
            build_kernel_signaling_command(&input)
                .expect("kernel command must build")
                .kind(),
            kind
        );
        assert_eq!(
            apply_reference_signaling(&mut state, &input)
                .expect("signaling command must succeed")
                .kind,
            expected_kind
        );
    }

    let ice = signaling_input(
        SignalingCommandKind::SendIceCandidate,
        ReferenceSignalingPayload::SendIceCandidate {
            candidate: FixtureIceCandidate {
                session_id: session("benchmark-ice-session"),
                fixture_candidate_id: "candidate".to_owned(),
            },
        },
    );
    let ice_outcome =
        apply_reference_signaling(&mut state, &ice).expect("ICE candidate must succeed");
    assert_eq!(ice_outcome.kind, SignalingEventKind::IceCandidateReceived);
    assert_eq!(
        signaling_success_event_kind(ice_outcome.kind),
        SignalingEventKind::IceCandidateReceived
    );
    assert_eq!(
        project_reference_signaling_event(
            &ice_outcome,
            ReferenceSignalingPayload::SendIceCandidate {
                candidate: FixtureIceCandidate {
                    session_id: session("benchmark-ice-session"),
                    fixture_candidate_id: "candidate".to_owned(),
                },
            },
        )
        .kind(),
        SignalingEventKind::IceCandidateReceived
    );

    let invalid_offer = signaling_input(
        SignalingCommandKind::SendOffer,
        ReferenceSignalingPayload::SendAnswer {
            session: FixtureSessionDescription {
                session_id: session("benchmark-invalid-offer-session"),
                fixture_sdp_id: "answer-as-offer".to_owned(),
                direction: FixtureSessionDescriptionDirection::Answer,
            },
        },
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &invalid_offer),
        Err(ReferenceSignalingError::InvalidFixtureIdentity)
    );

    let invalid_answer = signaling_input(
        SignalingCommandKind::SendAnswer,
        ReferenceSignalingPayload::SendOffer {
            session: FixtureSessionDescription {
                session_id: session("benchmark-invalid-answer-session"),
                fixture_sdp_id: "offer-as-answer".to_owned(),
                direction: FixtureSessionDescriptionDirection::Offer,
            },
        },
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &invalid_answer),
        Err(ReferenceSignalingError::InvalidFixtureIdentity)
    );

    let invalid_ice = signaling_input(
        SignalingCommandKind::SendIceCandidate,
        ReferenceSignalingPayload::RequestTurnCredential,
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &invalid_ice),
        Err(ReferenceSignalingError::InvalidFixtureIdentity)
    );

    let leave = signaling_input(
        SignalingCommandKind::LeaveRoom,
        ReferenceSignalingPayload::LeaveRoom,
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &leave)
            .expect("leave must succeed")
            .kind,
        SignalingEventKind::ParticipantLeft
    );
    assert_eq!(
        apply_reference_signaling(
            &mut state,
            &signaling_input(
                SignalingCommandKind::RequestTurnCredential,
                ReferenceSignalingPayload::RequestTurnCredential,
            ),
        ),
        Err(ReferenceSignalingError::StateBoundaryViolation)
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &join)
            .expect("rejoin after leave must succeed")
            .kind,
        SignalingEventKind::Joined
    );

    assert_eq!(
        apply_reference_signaling(
            &mut state,
            &ReferenceSignalingCommandInput::new(
                cid("benchmark-signaling-missing-participant"),
                room("benchmark-signaling-room"),
                None,
                SignalingCommandKind::JoinRoom,
                ReferenceSignalingPayload::JoinRoom,
            ),
        ),
        Err(ReferenceSignalingError::StateBoundaryViolation)
    );

    let mut corrupt = state.clone();
    corrupt.rooms.insert(
        "orphan-room".to_owned(),
        ReferenceRoomState {
            room_id: room("orphan-room"),
            phase: ReferenceRoomPhase::Open,
            participant_ids: ["missing-participant".to_owned()].into_iter().collect(),
        },
    );
    assert_eq!(
        validate_reference_signaling_state(&corrupt),
        Err(ReferenceSignalingError::StateBoundaryViolation)
    );

    let mut participant_without_room = state.clone();
    participant_without_room.rooms.clear();
    assert_eq!(
        validate_reference_signaling_state(&participant_without_room),
        Err(ReferenceSignalingError::StateBoundaryViolation)
    );

    let mut joined_not_indexed = state.clone();
    joined_not_indexed
        .rooms
        .get_mut("benchmark-signaling-room")
        .expect("room must exist")
        .participant_ids
        .clear();
    assert_eq!(
        validate_reference_signaling_state(&joined_not_indexed),
        Err(ReferenceSignalingError::StateBoundaryViolation)
    );

    let mut closed_room = state.clone();
    closed_room
        .rooms
        .get_mut("benchmark-signaling-room")
        .expect("room must exist")
        .phase = ReferenceRoomPhase::Closed;
    assert_eq!(
        apply_reference_signaling(&mut closed_room, &join),
        Err(ReferenceSignalingError::StateBoundaryViolation)
    );

    let mut wrong_room = state.clone();
    wrong_room.participants.insert(
        "benchmark-signaling-participant".to_owned(),
        ReferenceParticipantState {
            participant_id: participant("benchmark-signaling-participant"),
            room_id: room("other-room"),
            phase: ReferenceParticipantPhase::Joined,
        },
    );
    assert_eq!(
        apply_reference_signaling(&mut wrong_room, &join),
        Err(ReferenceSignalingError::StateBoundaryViolation)
    );
}

#[test]
fn benchmark_reference_turn_covers_state_rejection_branches() {
    let mut state = ReferenceTurnState::default();
    assert_eq!(
        apply_reference_turn(&mut state, &turn_input(TurnCommandKind::Allocate))
            .expect("allocate must succeed")
            .kind,
        TurnDecisionKind::Allocation
    );
    assert_eq!(
        apply_reference_turn(&mut state, &turn_input(TurnCommandKind::Refresh))
            .expect("refresh must succeed")
            .kind,
        TurnDecisionKind::Refresh
    );
    assert_eq!(
        apply_reference_turn(&mut state, &turn_input(TurnCommandKind::CreatePermission))
            .expect("permission must succeed")
            .kind,
        TurnDecisionKind::Permission
    );
    assert_eq!(
        apply_reference_turn(&mut state, &turn_input(TurnCommandKind::ChannelBind))
            .expect("bind must succeed")
            .kind,
        TurnDecisionKind::ChannelBind
    );
    assert_eq!(
        apply_reference_turn(&mut state, &turn_input(TurnCommandKind::RelayData))
            .expect("relay must succeed")
            .kind,
        TurnDecisionKind::Relay
    );
    assert_eq!(
        turn_decision_kind(TurnDecisionKind::Relay),
        TurnDecisionKind::Relay
    );

    let mut missing_lifetime = turn_input(TurnCommandKind::Refresh);
    missing_lifetime.requested_lifetime = None;
    assert_eq!(
        apply_reference_turn(&mut state, &missing_lifetime),
        Err(ReferenceTurnError::StateBoundaryViolation)
    );

    let mut missing_credential = turn_input(TurnCommandKind::Allocate);
    missing_credential.credential_ref = None;
    assert_eq!(
        apply_reference_turn(&mut ReferenceTurnState::default(), &missing_credential),
        Err(ReferenceTurnError::InvalidFixtureCredential)
    );

    let mut corrupt_permission = state.clone();
    corrupt_permission.permissions.insert(
        "orphan-permission".to_owned(),
        ReferencePermissionState {
            permission_id: permission("orphan-permission"),
            allocation_id: allocation("missing-allocation"),
            peer_address: CorePeerAddress::new("192.0.2.88:3478").expect("peer address"),
            phase: PermissionState::Active,
        },
    );
    assert_eq!(
        validate_reference_turn_state(&corrupt_permission),
        Err(ReferenceTurnError::StateBoundaryViolation)
    );

    let mut corrupt_channel = state.clone();
    corrupt_channel.channel_binds.insert(
        "orphan-channel".to_owned(),
        ReferenceChannelBindState {
            channel_bind_id: channel_bind("orphan-channel"),
            permission_id: permission("missing-permission"),
            phase: ChannelBindState::Active,
        },
    );
    assert_eq!(
        validate_reference_turn_state(&corrupt_channel),
        Err(ReferenceTurnError::StateBoundaryViolation)
    );

    let mut revoked_permission = state.clone();
    revoked_permission
        .permissions
        .get_mut("benchmark-turn-permission")
        .expect("permission must exist")
        .phase = PermissionState::Revoked;
    assert_eq!(
        apply_reference_turn(
            &mut revoked_permission,
            &turn_input(TurnCommandKind::ChannelBind),
        ),
        Err(ReferenceTurnError::StateBoundaryViolation)
    );

    let mut missing_packet = turn_input(TurnCommandKind::RelayData);
    missing_packet.relay_packet_id = None;
    assert_eq!(
        apply_reference_turn(&mut state, &missing_packet),
        Err(ReferenceTurnError::StateBoundaryViolation)
    );

    let mut inactive_allocation = state.clone();
    inactive_allocation
        .allocations
        .get_mut("benchmark-turn-allocation")
        .expect("allocation must exist")
        .phase = AllocationState::Released;
    assert_eq!(
        apply_reference_turn(
            &mut inactive_allocation,
            &turn_input(TurnCommandKind::CreatePermission),
        ),
        Err(ReferenceTurnError::StateBoundaryViolation)
    );
}

#[test]
fn benchmark_reference_sfu_covers_state_rejection_branches() {
    let session_id = session("benchmark-sfu-session");
    let endpoint_id = endpoint("benchmark-sfu-endpoint");
    let stream_id = stream("benchmark-sfu-stream");
    let route_id = route("benchmark-sfu-route");
    let mut state = ReferenceSfuState::default();
    let mut reject_state = ReferenceSfuState::default();

    assert_eq!(
        apply_reference_sfu(
            &mut reject_state,
            &ReferenceSfuAction::RejectParticipant {
                session_id: session("benchmark-sfu-reject-session"),
                endpoint_id: endpoint("benchmark-sfu-reject-endpoint"),
            },
        )
        .expect("reject must succeed")
        .kind,
        SfuDecisionKind::ParticipantAdmission
    );
    assert_eq!(
        apply_reference_sfu(
            &mut reject_state,
            &ReferenceSfuAction::RejectParticipant {
                session_id: session("benchmark-sfu-reject-session"),
                endpoint_id: endpoint("benchmark-sfu-reject-endpoint"),
            },
        )
        .expect("idempotent reject must succeed")
        .kind,
        SfuDecisionKind::ParticipantAdmission
    );
    assert_eq!(
        apply_reference_sfu(
            &mut reject_state,
            &ReferenceSfuAction::AdmitParticipant {
                session_id: session("benchmark-sfu-reject-session"),
                endpoint_id: endpoint("benchmark-sfu-reject-endpoint"),
            },
        ),
        Err(ReferenceSfuError::StateBoundaryViolation)
    );

    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::AdmitParticipant {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
            },
        )
        .expect("admit must succeed")
        .kind,
        SfuDecisionKind::ParticipantAdmission
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::AdmitParticipant {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
            },
        )
        .expect("idempotent admit must succeed")
        .kind,
        SfuDecisionKind::ParticipantAdmission
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::PublishStream {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
                stream_id: stream_id.clone(),
            },
        )
        .expect("publish must succeed")
        .kind,
        SfuDecisionKind::Publication
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SubscribeRoute {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
                stream_id: stream_id.clone(),
                route_id: route_id.clone(),
            },
        )
        .expect("subscribe must succeed")
        .kind,
        SfuDecisionKind::Subscription
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SubscribeRoute {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
                stream_id: stream_id.clone(),
                route_id: route_id.clone(),
            },
        )
        .expect("idempotent candidate subscribe must succeed")
        .kind,
        SfuDecisionKind::Subscription
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SelectRoute {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
                stream_id: stream_id.clone(),
                route_id: route_id.clone(),
            },
        )
        .expect("select must succeed")
        .kind,
        SfuDecisionKind::RouteSelection
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SuppressForwarding {
                route_id: route_id.clone(),
                source: ReferenceSfuSuppressionSource::Quality,
            },
        )
        .expect("quality suppression must succeed")
        .kind,
        SfuDecisionKind::Forwarding
    );
    let mut backpressure_state = ReferenceSfuState::default();
    let backpressure_session = session("benchmark-sfu-backpressure-session");
    let backpressure_endpoint = endpoint("benchmark-sfu-backpressure-endpoint");
    let backpressure_stream = stream("benchmark-sfu-backpressure-stream");
    let backpressure_route = route("benchmark-sfu-backpressure-route");
    apply_reference_sfu(
        &mut backpressure_state,
        &ReferenceSfuAction::AdmitParticipant {
            session_id: backpressure_session.clone(),
            endpoint_id: backpressure_endpoint.clone(),
        },
    )
    .expect("backpressure admit must succeed");
    apply_reference_sfu(
        &mut backpressure_state,
        &ReferenceSfuAction::SubscribeRoute {
            session_id: backpressure_session.clone(),
            endpoint_id: backpressure_endpoint.clone(),
            stream_id: backpressure_stream.clone(),
            route_id: backpressure_route.clone(),
        },
    )
    .expect("backpressure subscribe must succeed");
    apply_reference_sfu(
        &mut backpressure_state,
        &ReferenceSfuAction::SelectRoute {
            session_id: backpressure_session,
            endpoint_id: backpressure_endpoint,
            stream_id: backpressure_stream,
            route_id: backpressure_route.clone(),
        },
    )
    .expect("backpressure select must succeed");
    assert_eq!(
        apply_reference_sfu(
            &mut backpressure_state,
            &ReferenceSfuAction::SuppressForwarding {
                route_id: backpressure_route,
                source: ReferenceSfuSuppressionSource::Backpressure,
            },
        )
        .expect("backpressure suppression must succeed")
        .kind,
        SfuDecisionKind::Forwarding
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::DropForwarding {
                route_id: route_id.clone(),
            },
        )
        .expect("drop must succeed")
        .kind,
        SfuDecisionKind::BackpressureAction
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::DropForwarding {
                route_id: route_id.clone(),
            },
        ),
        Err(ReferenceSfuError::StateBoundaryViolation)
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::CloseSession {
                session_id: session_id.clone(),
            },
        )
        .expect("close must succeed")
        .kind,
        SfuDecisionKind::DegradationRecovery
    );
    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::CloseSession {
                session_id: session_id.clone(),
            },
        ),
        Err(ReferenceSfuError::StateBoundaryViolation)
    );

    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::AdmitParticipant {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
            },
        ),
        Err(ReferenceSfuError::StateBoundaryViolation)
    );

    let mut corrupt_endpoint = ReferenceSfuState::default();
    corrupt_endpoint.endpoints.insert(
        "orphan-endpoint".to_owned(),
        ReferenceEndpointState {
            endpoint_id: endpoint("orphan-endpoint"),
            session_id: session("missing-session"),
            phase: SfuEndpointState::Admitted,
        },
    );
    assert_eq!(
        validate_reference_sfu_state(&corrupt_endpoint),
        Err(ReferenceSfuError::StateBoundaryViolation)
    );

    let mut corrupt_route = ReferenceSfuState::default();
    corrupt_route.sessions.insert(
        "benchmark-sfu-session".to_owned(),
        ReferenceSfuSessionState {
            session_id: session_id.clone(),
            phase: SfuSessionState::Open,
        },
    );
    corrupt_route.routes.insert(
        "orphan-route".to_owned(),
        ReferenceRouteState {
            route_id: route("orphan-route"),
            session_id,
            endpoint_id,
            stream_id,
            phase: SfuRouteState::Candidate,
        },
    );
    assert_eq!(
        validate_reference_sfu_state(&corrupt_route),
        Err(ReferenceSfuError::StateBoundaryViolation)
    );

    let mut corrupt_route_session = ReferenceSfuState::default();
    corrupt_route_session.sessions.insert(
        "benchmark-sfu-route-session".to_owned(),
        ReferenceSfuSessionState {
            session_id: session("benchmark-sfu-route-session"),
            phase: SfuSessionState::Open,
        },
    );
    corrupt_route_session.sessions.insert(
        "benchmark-sfu-endpoint-session".to_owned(),
        ReferenceSfuSessionState {
            session_id: session("benchmark-sfu-endpoint-session"),
            phase: SfuSessionState::Open,
        },
    );
    corrupt_route_session.endpoints.insert(
        "benchmark-sfu-mismatched-endpoint".to_owned(),
        ReferenceEndpointState {
            endpoint_id: endpoint("benchmark-sfu-mismatched-endpoint"),
            session_id: session("benchmark-sfu-endpoint-session"),
            phase: SfuEndpointState::Admitted,
        },
    );
    corrupt_route_session.routes.insert(
        "benchmark-sfu-mismatched-route".to_owned(),
        ReferenceRouteState {
            route_id: route("benchmark-sfu-mismatched-route"),
            session_id: session("benchmark-sfu-route-session"),
            endpoint_id: endpoint("benchmark-sfu-mismatched-endpoint"),
            stream_id: stream("benchmark-sfu-mismatched-stream"),
            phase: SfuRouteState::Candidate,
        },
    );
    assert_eq!(
        validate_reference_sfu_state(&corrupt_route_session),
        Err(ReferenceSfuError::StateBoundaryViolation)
    );
}
