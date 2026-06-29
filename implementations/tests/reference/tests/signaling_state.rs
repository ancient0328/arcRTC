//! reference Signaling state mutation 境界を直接実行して検査します。

use arcrtc_core_identity::{
    CorrelationId, OpaqueReference, ParticipantId, ReferenceAuthority, RoomId, SessionId,
};
use arcrtc_core_signaling::{SignalingCommandKind, SignalingEventKind};
use arcrtc_reference_signaling::fixture_identity::FixtureSessionDescriptionDirection;
use arcrtc_reference_signaling::{
    apply_reference_signaling, build_kernel_signaling_command, validate_reference_signaling_state,
    FixtureSessionDescription, ReferenceParticipantPhase, ReferenceSignalingCommandInput,
    ReferenceSignalingError, ReferenceSignalingPayload, ReferenceSignalingState,
};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
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

fn input(
    kind: SignalingCommandKind,
    payload: ReferenceSignalingPayload,
) -> ReferenceSignalingCommandInput {
    ReferenceSignalingCommandInput::new(
        cid("sig-correlation"),
        room("sig-room"),
        Some(participant("sig-participant")),
        kind,
        payload,
    )
}

#[test]
fn kpi_reference_signaling_executes_kernel_contract_driven_state_transition() {
    let mut state = ReferenceSignalingState::default();
    let requested = input(
        SignalingCommandKind::JoinRoom,
        ReferenceSignalingPayload::JoinRoom,
    );
    let kernel_command =
        build_kernel_signaling_command(&requested).expect("Kernel command builder must succeed");
    let subject = kernel_command.envelope().subject_references();

    // KPI-T3-1 は Kernel command builder output を reference transition の起点として固定する。
    let transition_input = ReferenceSignalingCommandInput::new(
        kernel_command.envelope().correlation_id().clone(),
        subject.room_id().clone(),
        subject.participant_id().cloned(),
        kernel_command.kind(),
        ReferenceSignalingPayload::JoinRoom,
    );

    assert_eq!(
        kernel_command.envelope().command_type().as_str(),
        "reference.signaling.join_room"
    );
    assert_eq!(kernel_command.envelope().version().value(), 1);
    let joined = apply_reference_signaling(&mut state, &transition_input)
        .expect("Kernel-derived join transition must succeed");

    assert_eq!(joined.kind, SignalingEventKind::Joined);
    assert_eq!(joined.correlation_id, requested.correlation_id);
    assert_eq!(joined.room_id, requested.room_id);
    assert_eq!(joined.participant_id, requested.participant_id);
    assert_eq!(
        state
            .participants
            .get("sig-participant")
            .expect("participant state must exist")
            .phase,
        ReferenceParticipantPhase::Joined
    );
    assert_eq!(validate_reference_signaling_state(&state), Ok(()));
}

#[test]
fn signaling_state_applies_join_leave_and_validates_in_memory_invariants() {
    let mut state = ReferenceSignalingState::default();

    let join = input(
        SignalingCommandKind::JoinRoom,
        ReferenceSignalingPayload::JoinRoom,
    );
    let joined = apply_reference_signaling(&mut state, &join).expect("join must succeed");
    assert_eq!(joined.kind, SignalingEventKind::Joined);
    assert_eq!(
        state
            .participants
            .get("sig-participant")
            .expect("participant state must exist")
            .phase,
        ReferenceParticipantPhase::Joined
    );

    let leave = input(
        SignalingCommandKind::LeaveRoom,
        ReferenceSignalingPayload::LeaveRoom,
    );
    let left = apply_reference_signaling(&mut state, &leave).expect("leave must succeed");
    assert_eq!(left.kind, SignalingEventKind::ParticipantLeft);
    assert_eq!(
        state
            .participants
            .get("sig-participant")
            .expect("participant state must exist")
            .phase,
        ReferenceParticipantPhase::Left
    );
    assert_eq!(validate_reference_signaling_state(&state), Ok(()));
}

#[test]
fn signaling_state_rejects_fixture_direction_mismatch_before_event_projection() {
    let mut state = ReferenceSignalingState::default();
    let join = input(
        SignalingCommandKind::JoinRoom,
        ReferenceSignalingPayload::JoinRoom,
    );
    apply_reference_signaling(&mut state, &join).expect("join must succeed");

    let invalid_offer = input(
        SignalingCommandKind::SendOffer,
        ReferenceSignalingPayload::SendOffer {
            session: FixtureSessionDescription {
                session_id: session("sig-session"),
                fixture_sdp_id: "answer-as-offer".to_owned(),
                direction: FixtureSessionDescriptionDirection::Answer,
            },
        },
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &invalid_offer),
        Err(ReferenceSignalingError::InvalidFixtureIdentity)
    );
}
