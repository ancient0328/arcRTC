//! reference composition cross-plane binding 境界を検査します。

use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CorrelationId, CredentialRef, EndpointId, OpaqueReference,
    PacketId, ParticipantId, PermissionId, ReferenceAuthority, RoomId, RouteId, SessionId,
    StreamId,
};
use arcrtc_core_sfu::{
    PacketClass, PacketHeaderSemanticView, SfuDecisionKind, SfuModelKind, SfuReferenceSet,
};
use arcrtc_core_signaling::{SignalingCommandKind, SignalingEventKind};
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommandKind, TurnDecisionKind, TurnReferenceSet,
    TurnRequestedLifetimeSeconds, TurnTransactionId,
};
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroNonClaimScope, DistroPlane};
use arcrtc_reference_composition::{
    bind_signaling_to_sfu, bind_signaling_to_turn, run_reference_composition_step,
    validate_reference_composition_state, ReferenceCompositionError,
    ReferenceCompositionPlaneOutcome, ReferenceCompositionState, ReferenceCompositionStep,
    ReferenceCompositionStepInput,
};
use arcrtc_reference_sfu::{
    apply_reference_sfu, build_borrowed_packet_view, build_kernel_sfu_item, ReferenceSfuAction,
    ReferenceSfuContractInput, ReferenceSfuSuppressionSource,
};
use arcrtc_reference_signaling::fixture_identity::FixtureSessionDescriptionDirection;
use arcrtc_reference_signaling::{
    apply_reference_signaling, build_kernel_signaling_command, FixtureIceCandidate,
    FixtureSessionDescription, ReferenceSignalingCommandInput, ReferenceSignalingPayload,
};
use arcrtc_reference_turn::{
    apply_reference_turn, build_kernel_turn_command, ReferenceTurnCommandInput,
};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

fn room() -> RoomId {
    RoomId::new(accepted("composition-room"))
}

fn participant() -> ParticipantId {
    ParticipantId::new(accepted("composition-participant"))
}

fn session() -> SessionId {
    SessionId::new(accepted("composition-session"))
}

fn endpoint() -> EndpointId {
    EndpointId::new(accepted("composition-endpoint"))
}

fn stream() -> StreamId {
    StreamId::new(accepted("composition-stream"))
}

fn route() -> RouteId {
    RouteId::new(accepted("composition-route"))
}

fn allocation() -> AllocationId {
    AllocationId::new(accepted("composition-allocation"))
}

fn permission() -> PermissionId {
    PermissionId::new(accepted("composition-permission"))
}

fn channel_bind() -> ChannelBindId {
    ChannelBindId::new(accepted("composition-channel"))
}

fn packet() -> PacketId {
    PacketId::new(accepted("composition-packet"))
}

fn credential() -> CredentialRef {
    CredentialRef::new(accepted("composition-credential"))
}

fn references() -> TurnReferenceSet {
    TurnReferenceSet::new(
        Some(allocation()),
        Some(permission()),
        Some(channel_bind()),
        Some(credential()),
    )
}

fn turn_input(kind: TurnCommandKind) -> ReferenceTurnCommandInput {
    ReferenceTurnCommandInput {
        kind,
        transaction_id: TurnTransactionId::new(accepted("composition-turn-transaction")),
        references: references(),
        allocation_id: Some(allocation()),
        permission_id: Some(permission()),
        channel_bind_id: Some(channel_bind()),
        credential_ref: Some(credential()),
        peer_address: Some(CorePeerAddress::new("192.0.2.11:3478").expect("peer address")),
        requested_lifetime: Some(TurnRequestedLifetimeSeconds::try_new(600).expect("lifetime")),
        relay_packet_id: matches!(kind, TurnCommandKind::RelayData).then(packet),
    }
}

fn signaling_input(
    correlation_id: &str,
    kind: SignalingCommandKind,
    payload: ReferenceSignalingPayload,
) -> ReferenceSignalingCommandInput {
    ReferenceSignalingCommandInput::new(
        cid(correlation_id),
        room(),
        Some(participant()),
        kind,
        payload,
    )
}

fn kernel_driven_signaling_input(
    kind: SignalingCommandKind,
    payload: ReferenceSignalingPayload,
) -> ReferenceSignalingCommandInput {
    let requested = signaling_input("dcomp3-signaling-kernel", kind, payload);
    let kernel_command =
        build_kernel_signaling_command(&requested).expect("Kernel Signaling command must build");
    let subject = kernel_command.envelope().subject_references();
    ReferenceSignalingCommandInput::new(
        kernel_command.envelope().correlation_id().clone(),
        subject.room_id().clone(),
        subject.participant_id().cloned(),
        kernel_command.kind(),
        requested.payload.clone(),
    )
}

fn kernel_driven_turn_input(kind: TurnCommandKind) -> ReferenceTurnCommandInput {
    let requested = turn_input(kind);
    let kernel_command =
        build_kernel_turn_command(&requested).expect("Kernel TURN command must build");
    ReferenceTurnCommandInput {
        kind: kernel_command.kind(),
        transaction_id: kernel_command.transaction_id().clone(),
        references: kernel_command.references().clone(),
        allocation_id: requested.allocation_id,
        permission_id: requested.permission_id,
        channel_bind_id: requested.channel_bind_id,
        credential_ref: requested.credential_ref,
        peer_address: requested.peer_address,
        requested_lifetime: requested.requested_lifetime,
        relay_packet_id: requested.relay_packet_id,
    }
}

fn sfu_references() -> SfuReferenceSet {
    SfuReferenceSet::new(
        session(),
        Some(endpoint()),
        Some(stream()),
        Some(route()),
        Some(packet()),
    )
}

fn kernel_checked_sfu_action(model_kind: SfuModelKind, action: ReferenceSfuAction) -> ReferenceSfuAction {
    let raw_packet = [
        0x80, 0x60, 0x00, 0x2a, 0x00, 0x00, 0x03, 0xe8, 0, 0, 0, 7, 1, 2, 3, 4,
    ];
    let payload = &raw_packet[12..];
    let packet_id = packet();
    let stream_id = stream();
    let endpoint_id = endpoint();
    let packet_view = build_borrowed_packet_view(
        &packet_id,
        &stream_id,
        &endpoint_id,
        PacketHeaderSemanticView::new(PacketClass::Rtp, Some(42), Some(1000), Some(7)),
        &raw_packet,
        payload,
    );
    let packet_item = build_kernel_sfu_item(ReferenceSfuContractInput::new(
        SfuModelKind::BorrowedPacketAbstractView,
        sfu_references(),
        packet_view,
    ));
    assert_eq!(
        packet_item.model_kind(),
        SfuModelKind::BorrowedPacketAbstractView
    );
    let action_item = build_kernel_sfu_item(ReferenceSfuContractInput::new(
        model_kind,
        sfu_references(),
        action.clone(),
    ));
    assert_eq!(action_item.model_kind(), model_kind);
    // Kernel item は payload getter を公開しないため、ここでは constructor/model kind 接続を観測し、
    // 実行payloadは同じ reference action を composition step へ渡します。
    action
}

fn seed_composition_state() -> ReferenceCompositionState {
    let mut state = ReferenceCompositionState::default();

    let join = ReferenceSignalingCommandInput::new(
        cid("composition-signaling"),
        room(),
        Some(participant()),
        SignalingCommandKind::JoinRoom,
        ReferenceSignalingPayload::JoinRoom,
    );
    apply_reference_signaling(&mut state.signaling, &join).expect("signaling join must succeed");

    let allocated = apply_reference_turn(&mut state.turn, &turn_input(TurnCommandKind::Allocate))
        .expect("turn allocation must succeed");
    assert_eq!(allocated.kind, TurnDecisionKind::Allocation);

    let admitted = apply_reference_sfu(
        &mut state.sfu,
        &ReferenceSfuAction::AdmitParticipant {
            session_id: session(),
            endpoint_id: endpoint(),
        },
    )
    .expect("sfu admission must succeed");
    assert_eq!(admitted.kind, SfuDecisionKind::ParticipantAdmission);

    apply_reference_sfu(
        &mut state.sfu,
        &ReferenceSfuAction::SubscribeRoute {
            session_id: session(),
            endpoint_id: endpoint(),
            stream_id: stream(),
            route_id: route(),
        },
    )
    .expect("sfu route subscription must succeed");

    state
}

fn step(correlation_id: &str, step: ReferenceCompositionStep) -> ReferenceCompositionStepInput {
    ReferenceCompositionStepInput {
        correlation_id: cid(correlation_id),
        step,
    }
}

#[test]
fn kpi_reference_composition_executes_cross_plane_binding() {
    let mut state = ReferenceCompositionState::default();

    // KPI-T3-4 は plane output を runtime bridge で観測してから cross-plane binding へ接続する。
    let signaling = run_reference_composition_step(
        &mut state,
        step(
            "kpi-composition-signaling",
            ReferenceCompositionStep::ApplySignaling(ReferenceSignalingCommandInput::new(
                cid("kpi-composition-signaling-command"),
                room(),
                Some(participant()),
                SignalingCommandKind::JoinRoom,
                ReferenceSignalingPayload::JoinRoom,
            )),
        ),
    )
    .expect("signaling step must succeed");
    assert_eq!(signaling.applied_plane, DistroPlane::Signaling);
    assert!(matches!(
        signaling.plane_outcome,
        ReferenceCompositionPlaneOutcome::Signaling(_)
    ));

    let turn = run_reference_composition_step(
        &mut state,
        step(
            "kpi-composition-turn",
            ReferenceCompositionStep::ApplyTurn(turn_input(TurnCommandKind::Allocate)),
        ),
    )
    .expect("turn step must succeed");
    assert_eq!(turn.applied_plane, DistroPlane::Turn);
    assert!(matches!(
        turn.plane_outcome,
        ReferenceCompositionPlaneOutcome::Turn(_)
    ));

    let sfu_admission = run_reference_composition_step(
        &mut state,
        step(
            "kpi-composition-sfu-admission",
            ReferenceCompositionStep::ApplySfu(ReferenceSfuAction::AdmitParticipant {
                session_id: session(),
                endpoint_id: endpoint(),
            }),
        ),
    )
    .expect("sfu admission step must succeed");
    assert_eq!(sfu_admission.applied_plane, DistroPlane::Sfu);
    assert!(matches!(
        sfu_admission.plane_outcome,
        ReferenceCompositionPlaneOutcome::Sfu(_)
    ));

    run_reference_composition_step(
        &mut state,
        step(
            "kpi-composition-sfu-subscribe",
            ReferenceCompositionStep::ApplySfu(ReferenceSfuAction::SubscribeRoute {
                session_id: session(),
                endpoint_id: endpoint(),
                stream_id: stream(),
                route_id: route(),
            }),
        ),
    )
    .expect("sfu subscription step must succeed");

    let turn_binding = run_reference_composition_step(
        &mut state,
        step(
            "kpi-composition-bind-turn",
            ReferenceCompositionStep::BindSignalingToTurn {
                room_id: room(),
                allocation_id: allocation(),
            },
        ),
    )
    .expect("signaling to TURN binding must succeed");
    assert_eq!(turn_binding.applied_plane, DistroPlane::Composition);
    assert_eq!(
        turn_binding.non_claim_scope,
        vec![
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ]
    );

    let sfu_binding = run_reference_composition_step(
        &mut state,
        step(
            "kpi-composition-bind-sfu",
            ReferenceCompositionStep::BindSignalingToSfu {
                room_id: room(),
                session_id: session(),
                route_id: route(),
            },
        ),
    )
    .expect("signaling to SFU binding must succeed");
    assert_eq!(sfu_binding.applied_plane, DistroPlane::Composition);
    assert_eq!(state.bindings.len(), 2);

    run_reference_composition_step(
        &mut state,
        step(
            "kpi-composition-validate",
            ReferenceCompositionStep::Validate,
        ),
    )
    .expect("composition validation step must succeed");
    assert_eq!(validate_reference_composition_state(&state), Ok(()));
}

#[test]
fn dcomp3_reference_composition_executes_full_bounded_reference_flow() {
    let mut state = ReferenceCompositionState::default();

    // D-COMP-3 は個別planeの単体検査ではなく、Kernel public type を通した
    // reference-local flow が composition state に閉じることを一連のscenarioで固定する。
    let joined = run_reference_composition_step(
        &mut state,
        step(
            "dcomp3-signaling-join",
            ReferenceCompositionStep::ApplySignaling(kernel_driven_signaling_input(
                SignalingCommandKind::JoinRoom,
                ReferenceSignalingPayload::JoinRoom,
            )),
        ),
    )
    .expect("signaling join must succeed");
    assert_eq!(joined.applied_plane, DistroPlane::Signaling);
    assert!(matches!(
        joined.plane_outcome,
        ReferenceCompositionPlaneOutcome::Signaling(outcome)
            if outcome.kind == SignalingEventKind::Joined
    ));

    let offered = run_reference_composition_step(
        &mut state,
        step(
            "dcomp3-signaling-offer",
            ReferenceCompositionStep::ApplySignaling(kernel_driven_signaling_input(
                SignalingCommandKind::SendOffer,
                ReferenceSignalingPayload::SendOffer {
                    session: FixtureSessionDescription {
                        session_id: session(),
                        fixture_sdp_id: "dcomp3-offer-sdp".to_owned(),
                        direction: FixtureSessionDescriptionDirection::Offer,
                    },
                },
            )),
        ),
    )
    .expect("signaling offer must succeed");
    assert!(matches!(
        offered.plane_outcome,
        ReferenceCompositionPlaneOutcome::Signaling(outcome)
            if outcome.kind == SignalingEventKind::OfferReceived
    ));

    let candidate = run_reference_composition_step(
        &mut state,
        step(
            "dcomp3-signaling-candidate",
            ReferenceCompositionStep::ApplySignaling(kernel_driven_signaling_input(
                SignalingCommandKind::SendIceCandidate,
                ReferenceSignalingPayload::SendIceCandidate {
                    candidate: FixtureIceCandidate {
                        session_id: session(),
                        fixture_candidate_id: "dcomp3-candidate".to_owned(),
                    },
                },
            )),
        ),
    )
    .expect("signaling candidate must succeed");
    assert!(matches!(
        candidate.plane_outcome,
        ReferenceCompositionPlaneOutcome::Signaling(outcome)
            if outcome.kind == SignalingEventKind::IceCandidateReceived
    ));

    for (label, kind, expected) in [
        (
            "dcomp3-turn-allocation",
            TurnCommandKind::Allocate,
            TurnDecisionKind::Allocation,
        ),
        (
            "dcomp3-turn-permission",
            TurnCommandKind::CreatePermission,
            TurnDecisionKind::Permission,
        ),
        (
            "dcomp3-turn-channel-bind",
            TurnCommandKind::ChannelBind,
            TurnDecisionKind::ChannelBind,
        ),
        (
            "dcomp3-turn-relay",
            TurnCommandKind::RelayData,
            TurnDecisionKind::Relay,
        ),
    ] {
        let turn = run_reference_composition_step(
            &mut state,
            step(
                label,
                ReferenceCompositionStep::ApplyTurn(kernel_driven_turn_input(kind)),
            ),
        )
        .expect("TURN step must succeed");
        assert!(matches!(
            turn.plane_outcome,
            ReferenceCompositionPlaneOutcome::Turn(outcome) if outcome.kind == expected
        ));
    }

    for (label, model_kind, action, expected) in [
        (
            "dcomp3-sfu-admission",
            SfuModelKind::AdmissionDecision,
            ReferenceSfuAction::AdmitParticipant {
                session_id: session(),
                endpoint_id: endpoint(),
            },
            SfuDecisionKind::ParticipantAdmission,
        ),
        (
            "dcomp3-sfu-publication",
            SfuModelKind::Publication,
            ReferenceSfuAction::PublishStream {
                session_id: session(),
                endpoint_id: endpoint(),
                stream_id: stream(),
            },
            SfuDecisionKind::Publication,
        ),
        (
            "dcomp3-sfu-subscription",
            SfuModelKind::Subscription,
            ReferenceSfuAction::SubscribeRoute {
                session_id: session(),
                endpoint_id: endpoint(),
                stream_id: stream(),
                route_id: route(),
            },
            SfuDecisionKind::Subscription,
        ),
        (
            "dcomp3-sfu-forwarding-selection",
            SfuModelKind::RouteCandidate,
            ReferenceSfuAction::SelectRoute {
                session_id: session(),
                endpoint_id: endpoint(),
                stream_id: stream(),
                route_id: route(),
            },
            SfuDecisionKind::RouteSelection,
        ),
        (
            "dcomp3-sfu-forwarding",
            SfuModelKind::ForwardingIntent,
            ReferenceSfuAction::SuppressForwarding {
                route_id: route(),
                source: ReferenceSfuSuppressionSource::Backpressure,
            },
            SfuDecisionKind::Forwarding,
        ),
    ] {
        let sfu = run_reference_composition_step(
            &mut state,
            step(
                label,
                ReferenceCompositionStep::ApplySfu(kernel_checked_sfu_action(model_kind, action)),
            ),
        )
        .expect("SFU step must succeed");
        assert!(matches!(
            sfu.plane_outcome,
            ReferenceCompositionPlaneOutcome::Sfu(outcome) if outcome.kind == expected
        ));
    }

    run_reference_composition_step(
        &mut state,
        step(
            "dcomp3-bind-turn",
            ReferenceCompositionStep::BindSignalingToTurn {
                room_id: room(),
                allocation_id: allocation(),
            },
        ),
    )
    .expect("signaling to TURN binding must succeed");

    run_reference_composition_step(
        &mut state,
        step(
            "dcomp3-bind-sfu",
            ReferenceCompositionStep::BindSignalingToSfu {
                room_id: room(),
                session_id: session(),
                route_id: route(),
            },
        ),
    )
    .expect("signaling to SFU binding must succeed");

    let validated = run_reference_composition_step(
        &mut state,
        step("dcomp3-validate", ReferenceCompositionStep::Validate),
    )
    .expect("composition validation must succeed");
    assert_eq!(
        validated.non_claim_scope,
        vec![
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ]
    );
    assert_eq!(validate_reference_composition_state(&state), Ok(()));
    assert_eq!(state.signaling.rooms.len(), 1);
    assert_eq!(state.turn.allocations.len(), 1);
    assert_eq!(state.turn.permissions.len(), 1);
    assert_eq!(state.turn.channel_binds.len(), 1);
    assert_eq!(state.sfu.sessions.len(), 1);
    assert_eq!(state.sfu.endpoints.len(), 1);
    assert_eq!(state.sfu.routes.len(), 1);
    assert_eq!(state.bindings.len(), 2);
    println!(
        "D-COMP-3 reference composition bounded flow rooms={} allocations={} permissions={} channel_binds={} sfu_sessions={} routes={} bindings={}",
        state.signaling.rooms.len(),
        state.turn.allocations.len(),
        state.turn.permissions.len(),
        state.turn.channel_binds.len(),
        state.sfu.sessions.len(),
        state.sfu.routes.len(),
        state.bindings.len()
    );
}

#[test]
fn composition_binds_only_existing_cross_plane_state() {
    let mut state = seed_composition_state();

    let turn_outcome = bind_signaling_to_turn(
        &mut state,
        cid("composition-bind-turn"),
        room(),
        allocation(),
    )
    .expect("signaling to turn binding must succeed");
    assert_eq!(turn_outcome.distro_reason, DistroEvidenceReason::DistroOk);

    bind_signaling_to_sfu(
        &mut state,
        cid("composition-bind-sfu"),
        room(),
        session(),
        route(),
    )
    .expect("signaling to sfu binding must succeed");

    assert_eq!(state.bindings.len(), 2);
    assert_eq!(validate_reference_composition_state(&state), Ok(()));
}

#[test]
fn composition_rejects_orphan_cross_plane_bindings() {
    let mut empty_state = ReferenceCompositionState::default();

    assert_eq!(
        bind_signaling_to_turn(
            &mut empty_state,
            cid("composition-orphan-turn"),
            room(),
            allocation(),
        ),
        Err(ReferenceCompositionError::StateBoundaryViolation)
    );

    assert_eq!(
        bind_signaling_to_sfu(
            &mut empty_state,
            cid("composition-orphan-sfu"),
            room(),
            session(),
            route(),
        ),
        Err(ReferenceCompositionError::StateBoundaryViolation)
    );
}
