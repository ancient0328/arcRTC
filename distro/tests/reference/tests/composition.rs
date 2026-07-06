//! reference composition cross-plane binding 境界を検査します。

use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CorrelationId, CredentialRef, EndpointId, OpaqueReference,
    ParticipantId, PermissionId, ReferenceAuthority, RoomId, RouteId, SessionId, StreamId,
};
use arcrtc_core_sfu::SfuDecisionKind;
use arcrtc_core_signaling::SignalingCommandKind;
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommandKind, TurnDecisionKind, TurnReferenceSet,
    TurnRequestedLifetimeSeconds, TurnTransactionId,
};
use arcrtc_distro_evidence::{
    DistroEvidenceReason, DistroNonClaimScope, DistroPlane,
};
use arcrtc_reference_composition::{
    bind_signaling_to_sfu, bind_signaling_to_turn, run_reference_composition_step,
    validate_reference_composition_state, ReferenceCompositionError,
    ReferenceCompositionPlaneOutcome, ReferenceCompositionState, ReferenceCompositionStep,
    ReferenceCompositionStepInput,
};
use arcrtc_reference_sfu::{apply_reference_sfu, ReferenceSfuAction};
use arcrtc_reference_signaling::{
    apply_reference_signaling, ReferenceSignalingCommandInput, ReferenceSignalingPayload,
};
use arcrtc_reference_turn::{apply_reference_turn, ReferenceTurnCommandInput};

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
        relay_packet_id: None,
    }
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
fn composition_binds_only_existing_cross_plane_state() {
    let mut state = seed_composition_state();

    let turn_outcome = bind_signaling_to_turn(
        &mut state,
        cid("composition-bind-turn"),
        room(),
        allocation(),
    )
    .expect("signaling to turn binding must succeed");
    assert_eq!(
        turn_outcome.distro_reason,
        DistroEvidenceReason::DistroOk
    );

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
