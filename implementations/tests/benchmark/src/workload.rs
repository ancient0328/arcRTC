//! deterministic benchmark workload dispatch です。

use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CorrelationId, CredentialRef, EndpointId, OpaqueReference,
    PacketId, ParticipantId, PermissionId, ReferenceAuthority, RoomId, RouteId, SessionId,
    StreamId,
};
use arcrtc_core_sfu::{PacketClass, PacketHeaderSemanticView, SfuModelKind, SfuReferenceSet};
use arcrtc_core_signaling::SignalingCommandKind;
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommandKind, TurnReferenceSet, TurnRequestedLifetimeSeconds,
    TurnTransactionId,
};
use arcrtc_implementation_evidence::{ImplementationEvidenceReason, ImplementationPlane};
use arcrtc_product_deployment::{
    build_product_runtime_profile, select_product_runtime, ProductHostClass,
};
use arcrtc_product_persistence_topology::{map_product_projection, ProductPersistenceRecordClass};
use arcrtc_product_policy::{evaluate_product_auth_policy, ProductAction, ProductPolicyInput};
use arcrtc_product_rollback::{plan_drain, ProductDrainMode};
use arcrtc_reference_composition::{
    bind_signaling_to_sfu, bind_signaling_to_turn, validate_reference_composition_state,
    ReferenceCompositionState,
};
use arcrtc_reference_ops::{
    build_reference_build_evidence_record, ImplementationShutdownMode, ReferenceBuildEvidenceInput,
    ReferenceRuntime,
};
use arcrtc_reference_sfu::{
    apply_reference_sfu, build_borrowed_packet_view, build_kernel_sfu_item,
    validate_reference_sfu_state, ReferenceSfuAction, ReferenceSfuContractInput, ReferenceSfuState,
    ReferenceSfuSuppressionSource,
};
use arcrtc_reference_signaling::fixture_identity::FixtureSessionDescriptionDirection;
use arcrtc_reference_signaling::{
    apply_reference_signaling, build_kernel_signaling_command, validate_reference_signaling_state,
    FixtureIceCandidate, FixtureSessionDescription, ReferenceSignalingCommandInput,
    ReferenceSignalingPayload, ReferenceSignalingState,
};
use arcrtc_reference_turn::{
    apply_reference_turn, build_kernel_turn_command, validate_reference_turn_state,
    ReferenceTurnCommandInput, ReferenceTurnState,
};

use crate::scenario::BenchmarkScenario;

/// benchmark workload descriptor です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkWorkload {
    /// workload idです。
    pub workload_id: String,
    /// workload summaryです。
    pub workload_summary: String,
}

impl BenchmarkWorkload {
    /// scenario 定義から workload descriptor を作ります。
    pub fn from_scenario(scenario: &BenchmarkScenario) -> Self {
        Self {
            workload_id: workload_id_for_scenario(scenario),
            workload_summary: scenario.workload_summary.to_owned(),
        }
    }
}

/// KPI benchmark workload execution resultです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelProductionBenchmarkWorkload {
    /// workload idです。
    pub workload_id: String,
    /// 完了したwork item数です。
    pub completed_items: usize,
}

/// scenario に対応する actual workload id を返します。
pub fn workload_id_for_scenario(scenario: &BenchmarkScenario) -> String {
    format!("actual-workload:{}:{}", scenario.id, scenario.name)
}

/// Roadmap KPI-T6 の actual workload dispatch です。
pub fn execute_kpi_actual_workload(
    scenario: &BenchmarkScenario,
) -> KernelProductionBenchmarkWorkload {
    let completed_items = match scenario.id {
        "BENCH-001" => signaling_join_room_single(),
        "BENCH-002" => signaling_join_room_batch(),
        "BENCH-003" => signaling_offer_answer_candidate(),
        "BENCH-004" => signaling_turn_credential_request(),
        "BENCH-005" => turn_allocate_single(),
        "BENCH-006" => turn_permission_batch(),
        "BENCH-007" => turn_channel_bind_batch(),
        "BENCH-008" => turn_relay_data_path(),
        "BENCH-009" => sfu_contract_item_create(),
        "BENCH-010" => sfu_borrowed_packet_view(),
        "BENCH-011" => sfu_route_select_batch(),
        "BENCH-012" => sfu_packet_fanout_intent(),
        "BENCH-013" => composition_signaling_turn_binding(),
        "BENCH-014" => composition_signaling_sfu_binding(),
        "BENCH-015" => runtime_start_shutdown(),
        "BENCH-016" => evidence_json_record_write(),
        "BENCH-017" => product_policy_evaluate(),
        "BENCH-018" => product_projection_mapping(),
        "BENCH-019" => product_runtime_select(),
        "BENCH-020" => product_drain_plan_create(),
        _ => 0,
    };
    KernelProductionBenchmarkWorkload {
        workload_id: workload_id_for_scenario(scenario),
        completed_items,
    }
}

fn accepted(value: impl Into<String>) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("benchmark fixture reference must be accepted")
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

fn packet(value: &str) -> PacketId {
    PacketId::new(accepted(value))
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

fn signaling_input(
    suffix: &str,
    kind: SignalingCommandKind,
    payload: ReferenceSignalingPayload,
) -> ReferenceSignalingCommandInput {
    ReferenceSignalingCommandInput::new(
        cid(&format!("bench-signaling-{suffix}")),
        room(&format!("bench-room-{suffix}")),
        Some(participant(&format!("bench-participant-{suffix}"))),
        kind,
        payload,
    )
}

fn signaling_join_room_single() -> usize {
    let mut state = ReferenceSignalingState::default();
    let input = signaling_input(
        "single",
        SignalingCommandKind::JoinRoom,
        ReferenceSignalingPayload::JoinRoom,
    );
    build_kernel_signaling_command(&input).expect("kernel signaling command");
    apply_reference_signaling(&mut state, &input).expect("join room");
    validate_reference_signaling_state(&state).expect("valid signaling state");
    state.participants.len()
}

fn signaling_join_room_batch() -> usize {
    let mut completed = 0;
    for room_index in 0..8 {
        let mut state = ReferenceSignalingState::default();
        for participant_index in 0..6 {
            let suffix = format!("{room_index}-{participant_index}");
            let input = signaling_input(
                &suffix,
                SignalingCommandKind::JoinRoom,
                ReferenceSignalingPayload::JoinRoom,
            );
            apply_reference_signaling(&mut state, &input).expect("batch join room");
            completed += 1;
        }
        validate_reference_signaling_state(&state).expect("valid batch signaling state");
    }
    completed
}

fn signaling_offer_answer_candidate() -> usize {
    let mut state = ReferenceSignalingState::default();
    let join = signaling_input(
        "pair",
        SignalingCommandKind::JoinRoom,
        ReferenceSignalingPayload::JoinRoom,
    );
    apply_reference_signaling(&mut state, &join).expect("join before offer");
    let session_id = session("bench-signaling-session");
    let offer = ReferenceSignalingCommandInput::new(
        cid("bench-offer"),
        join.room_id.clone(),
        join.participant_id.clone(),
        SignalingCommandKind::SendOffer,
        ReferenceSignalingPayload::SendOffer {
            session: FixtureSessionDescription {
                session_id: session_id.clone(),
                fixture_sdp_id: "bench-offer-sdp".to_owned(),
                direction: FixtureSessionDescriptionDirection::Offer,
            },
        },
    );
    let answer = ReferenceSignalingCommandInput::new(
        cid("bench-answer"),
        join.room_id.clone(),
        join.participant_id.clone(),
        SignalingCommandKind::SendAnswer,
        ReferenceSignalingPayload::SendAnswer {
            session: FixtureSessionDescription {
                session_id: session_id.clone(),
                fixture_sdp_id: "bench-answer-sdp".to_owned(),
                direction: FixtureSessionDescriptionDirection::Answer,
            },
        },
    );
    apply_reference_signaling(&mut state, &offer).expect("offer");
    apply_reference_signaling(&mut state, &answer).expect("answer");
    for index in 0..4 {
        let candidate = ReferenceSignalingCommandInput::new(
            cid(&format!("bench-candidate-{index}")),
            join.room_id.clone(),
            join.participant_id.clone(),
            SignalingCommandKind::SendIceCandidate,
            ReferenceSignalingPayload::SendIceCandidate {
                candidate: FixtureIceCandidate {
                    session_id: session_id.clone(),
                    fixture_candidate_id: format!("bench-candidate-{index}"),
                },
            },
        );
        apply_reference_signaling(&mut state, &candidate).expect("candidate");
    }
    6
}

fn signaling_turn_credential_request() -> usize {
    let mut state = ReferenceSignalingState::default();
    let join = signaling_input(
        "turn-credential",
        SignalingCommandKind::JoinRoom,
        ReferenceSignalingPayload::JoinRoom,
    );
    apply_reference_signaling(&mut state, &join).expect("join before credential");
    let request = ReferenceSignalingCommandInput::new(
        cid("bench-turn-credential"),
        join.room_id.clone(),
        join.participant_id.clone(),
        SignalingCommandKind::RequestTurnCredential,
        ReferenceSignalingPayload::RequestTurnCredential,
    );
    apply_reference_signaling(&mut state, &request).expect("turn credential request");
    2
}

fn turn_input(suffix: &str, kind: TurnCommandKind) -> ReferenceTurnCommandInput {
    let allocation_id = allocation(&format!("bench-allocation-{suffix}"));
    let permission_id = permission(&format!("bench-permission-{suffix}"));
    let channel_bind_id = channel_bind(&format!("bench-channel-{suffix}"));
    let credential_ref = credential(&format!("bench-credential-{suffix}"));
    ReferenceTurnCommandInput {
        kind,
        transaction_id: TurnTransactionId::new(accepted(format!("bench-transaction-{suffix}"))),
        references: TurnReferenceSet::new(
            Some(allocation_id.clone()),
            Some(permission_id.clone()),
            Some(channel_bind_id.clone()),
            Some(credential_ref.clone()),
        ),
        allocation_id: Some(allocation_id),
        permission_id: Some(permission_id),
        channel_bind_id: Some(channel_bind_id),
        credential_ref: Some(credential_ref),
        peer_address: Some(
            CorePeerAddress::new(format!("192.0.2.{}", suffix.len() + 1)).expect("peer address"),
        ),
        requested_lifetime: Some(TurnRequestedLifetimeSeconds::try_new(600).expect("lifetime")),
        relay_packet_id: Some(packet(&format!("bench-turn-packet-{suffix}"))),
    }
}

fn turn_allocate_single() -> usize {
    let mut state = ReferenceTurnState::default();
    let input = turn_input("single", TurnCommandKind::Allocate);
    build_kernel_turn_command(&input).expect("kernel turn command");
    apply_reference_turn(&mut state, &input).expect("allocate");
    validate_reference_turn_state(&state).expect("valid turn state");
    1
}

fn turn_permission_batch() -> usize {
    let mut completed = 0;
    for index in 0..8 {
        let mut state = ReferenceTurnState::default();
        let suffix = format!("permission-{index}");
        let allocate = turn_input(&suffix, TurnCommandKind::Allocate);
        apply_reference_turn(&mut state, &allocate).expect("allocate before permission");
        for _permission_index in 0..2 {
            let input = turn_input(&suffix, TurnCommandKind::CreatePermission);
            apply_reference_turn(&mut state, &input).expect("permission");
            completed += 1;
        }
    }
    completed
}

fn turn_channel_bind_batch() -> usize {
    let mut completed = 0;
    for index in 0..16 {
        let mut state = ReferenceTurnState::default();
        let allocate = turn_input(&format!("channel-{index}"), TurnCommandKind::Allocate);
        let permission_input = turn_input(
            &format!("channel-{index}"),
            TurnCommandKind::CreatePermission,
        );
        let bind_input = turn_input(&format!("channel-{index}"), TurnCommandKind::ChannelBind);
        apply_reference_turn(&mut state, &allocate).expect("allocate before channel");
        apply_reference_turn(&mut state, &permission_input).expect("permission before channel");
        apply_reference_turn(&mut state, &bind_input).expect("channel bind");
        completed += 1;
    }
    completed
}

fn turn_relay_data_path() -> usize {
    let mut state = ReferenceTurnState::default();
    let allocate = turn_input("relay", TurnCommandKind::Allocate);
    let permission_input = turn_input("relay", TurnCommandKind::CreatePermission);
    let relay = turn_input("relay", TurnCommandKind::RelayData);
    apply_reference_turn(&mut state, &allocate).expect("allocate before relay");
    apply_reference_turn(&mut state, &permission_input).expect("permission before relay");
    apply_reference_turn(&mut state, &relay).expect("relay data");
    1
}

fn sfu_refs(suffix: &str) -> (SessionId, EndpointId, StreamId, RouteId, PacketId) {
    (
        session(&format!("bench-session-{suffix}")),
        endpoint(&format!("bench-endpoint-{suffix}")),
        stream(&format!("bench-stream-{suffix}")),
        route(&format!("bench-route-{suffix}")),
        packet(&format!("bench-packet-{suffix}")),
    )
}

fn sfu_contract_item_create() -> usize {
    let (session_id, endpoint_id, stream_id, route_id, packet_id) = sfu_refs("contract");
    let input = ReferenceSfuContractInput::new(
        SfuModelKind::RouteCandidate,
        SfuReferenceSet::new(
            session_id,
            Some(endpoint_id),
            Some(stream_id),
            Some(route_id),
            Some(packet_id),
        ),
        "route-candidate",
    );
    let item = build_kernel_sfu_item(input);
    usize::from(item.model_kind() == SfuModelKind::RouteCandidate)
}

fn sfu_borrowed_packet_view() -> usize {
    let (_, endpoint_id, stream_id, _, packet_id) = sfu_refs("packet");
    let raw = [0_u8; 1200];
    let payload = &raw[12..];
    let view = build_borrowed_packet_view(
        &packet_id,
        &stream_id,
        &endpoint_id,
        PacketHeaderSemanticView::new(PacketClass::Rtp, Some(1), Some(2), Some(3)),
        &raw,
        payload,
    );
    view.payload_len()
}

fn sfu_route_select_batch() -> usize {
    let mut state = ReferenceSfuState::default();
    let mut completed = 0;
    for index in 0..24 {
        let (session_id, endpoint_id, stream_id, route_id, _) = sfu_refs(&format!("route-{index}"));
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::AdmitParticipant {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
            },
        )
        .expect("admit");
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SubscribeRoute {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
                stream_id: stream_id.clone(),
                route_id: route_id.clone(),
            },
        )
        .expect("subscribe");
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SelectRoute {
                session_id,
                endpoint_id,
                stream_id,
                route_id,
            },
        )
        .expect("select");
        completed += 1;
    }
    validate_reference_sfu_state(&state).expect("valid sfu state");
    completed
}

fn sfu_packet_fanout_intent() -> usize {
    let mut state = ReferenceSfuState::default();
    for index in 0..6 {
        let (session_id, endpoint_id, stream_id, route_id, _) =
            sfu_refs(&format!("fanout-{index}"));
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::AdmitParticipant {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
            },
        )
        .expect("admit");
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SubscribeRoute {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
                stream_id: stream_id.clone(),
                route_id: route_id.clone(),
            },
        )
        .expect("subscribe");
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SelectRoute {
                session_id,
                endpoint_id,
                stream_id,
                route_id: route_id.clone(),
            },
        )
        .expect("select");
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SuppressForwarding {
                route_id,
                source: ReferenceSfuSuppressionSource::Backpressure,
            },
        )
        .expect("suppress");
    }
    6
}

fn composition_signaling_turn_binding() -> usize {
    let mut state = ReferenceCompositionState::default();
    for index in 0..8 {
        let room_id = room(&format!("bench-composition-room-{index}"));
        let join = ReferenceSignalingCommandInput::new(
            cid(&format!("bench-composition-join-{index}")),
            room_id.clone(),
            Some(participant(&format!(
                "bench-composition-participant-{index}"
            ))),
            SignalingCommandKind::JoinRoom,
            ReferenceSignalingPayload::JoinRoom,
        );
        apply_reference_signaling(&mut state.signaling, &join).expect("composition join");
        let allocation_input = turn_input(
            &format!("composition-turn-{index}"),
            TurnCommandKind::Allocate,
        );
        let allocation_id = allocation_input
            .allocation_id
            .clone()
            .expect("allocation id");
        apply_reference_turn(&mut state.turn, &allocation_input).expect("composition allocation");
        bind_signaling_to_turn(
            &mut state,
            cid(&format!("bench-composition-turn-{index}")),
            room_id,
            allocation_id,
        )
        .expect("signaling to turn binding");
    }
    validate_reference_composition_state(&state).expect("valid composition state");
    8
}

fn composition_signaling_sfu_binding() -> usize {
    let mut state = ReferenceCompositionState::default();
    for index in 0..24 {
        let room_id = room(&format!("bench-composition-sfu-room-{index}"));
        let session_id = session(&format!("bench-composition-session-{index}"));
        let endpoint_id = endpoint(&format!("bench-composition-endpoint-{index}"));
        let stream_id = stream(&format!("bench-composition-stream-{index}"));
        let route_id = route(&format!("bench-composition-route-{index}"));
        let join = ReferenceSignalingCommandInput::new(
            cid(&format!("bench-composition-sfu-join-{index}")),
            room_id.clone(),
            Some(participant(&format!(
                "bench-composition-sfu-participant-{index}"
            ))),
            SignalingCommandKind::JoinRoom,
            ReferenceSignalingPayload::JoinRoom,
        );
        apply_reference_signaling(&mut state.signaling, &join).expect("composition sfu join");
        apply_reference_sfu(
            &mut state.sfu,
            &ReferenceSfuAction::AdmitParticipant {
                session_id: session_id.clone(),
                endpoint_id: endpoint_id.clone(),
            },
        )
        .expect("composition sfu admit");
        apply_reference_sfu(
            &mut state.sfu,
            &ReferenceSfuAction::SubscribeRoute {
                session_id: session_id.clone(),
                endpoint_id,
                stream_id,
                route_id: route_id.clone(),
            },
        )
        .expect("composition sfu subscribe");
        bind_signaling_to_sfu(
            &mut state,
            cid(&format!("bench-composition-sfu-{index}")),
            room_id,
            session_id,
            route_id,
        )
        .expect("signaling to sfu binding");
    }
    validate_reference_composition_state(&state).expect("valid composition state");
    24
}

fn runtime_start_shutdown() -> usize {
    let mut runtime = ReferenceRuntime::new(ReferenceCompositionState::default());
    runtime
        .start(cid("bench-runtime-start"))
        .expect("runtime start");
    runtime
        .shutdown(
            cid("bench-runtime-stop"),
            ImplementationShutdownMode::GracefulLocal,
        )
        .expect("runtime stop");
    2
}

fn evidence_json_record_write() -> usize {
    let record = build_reference_build_evidence_record(ReferenceBuildEvidenceInput {
        correlation_id: cid("bench-evidence-json"),
        command: "cargo build --workspace --all-targets".to_owned(),
        target_package: "arcrtc-reference-ops".to_owned(),
        target_scope: "reference-implementation/ops".to_owned(),
        toolchain_runtime_version: "rustc-benchmark".to_owned(),
        expected_outcome: "build succeeds".to_owned(),
        actual_outcome: "build succeeds".to_owned(),
        exit_status: 0,
    })
    .expect("reference evidence record");
    serde_json::to_vec(&record).expect("evidence json").len()
}

fn product_policy_evaluate() -> usize {
    let mut completed = 0;
    for (plane, action) in [
        (ImplementationPlane::Signaling, ProductAction::Join),
        (ImplementationPlane::Turn, ProductAction::Relay),
        (ImplementationPlane::Sfu, ProductAction::Publish),
    ] {
        evaluate_product_auth_policy(&ProductPolicyInput {
            correlation_id: cid("bench-product-policy"),
            target_plane: plane,
            fixture_identity: Some("bench-fixture-user".to_owned()),
            requested_action: action,
        })
        .expect("product policy");
        completed += 1;
    }
    completed
}

fn product_projection_mapping() -> usize {
    for (plane, record_class) in [
        (
            ImplementationPlane::Signaling,
            ProductPersistenceRecordClass::SessionProjection,
        ),
        (
            ImplementationPlane::Turn,
            ProductPersistenceRecordClass::AllocationProjection,
        ),
        (
            ImplementationPlane::Sfu,
            ProductPersistenceRecordClass::RouteProjection,
        ),
        (
            ImplementationPlane::Monitoring,
            ProductPersistenceRecordClass::EvidenceProjection,
        ),
    ] {
        map_product_projection(plane, record_class).expect("product projection");
    }
    4
}

fn product_runtime_select() -> usize {
    let profile = build_product_runtime_profile(ProductHostClass::LocalSingleHost);
    let selection = select_product_runtime(&profile);
    usize::from(selection.implementation_reason == ImplementationEvidenceReason::ImplementationOk)
}

fn product_drain_plan_create() -> usize {
    let plan = plan_drain(
        cid("bench-product-drain"),
        vec![
            ImplementationPlane::Signaling,
            ImplementationPlane::Turn,
            ImplementationPlane::Sfu,
        ],
        ProductDrainMode::ControlledProduct,
    );
    plan.planes.len()
}
