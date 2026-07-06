//! reference Signaling fixture identity 境界を検査します。

use arcrtc_core_identity::{
    CorrelationId, OpaqueReference, ParticipantId, ReferenceAuthority, RoomId, SessionId,
};
use arcrtc_core_signaling::SignalingCommandKind;
use arcrtc_reference_signaling::fixture_identity::FixtureSessionDescriptionDirection;
use arcrtc_reference_signaling::{
    apply_reference_signaling, FixtureIceCandidate, FixtureSessionDescription,
    ReferenceSignalingCommandInput, ReferenceSignalingError, ReferenceSignalingPayload,
    ReferenceSignalingState,
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
        cid("fixture-correlation"),
        room("fixture-room"),
        Some(participant("fixture-participant")),
        kind,
        payload,
    )
}

#[test]
fn signaling_fixture_types_expose_only_fixture_identifiers() {
    let session_description = FixtureSessionDescription {
        session_id: session("fixture-session"),
        fixture_sdp_id: "offer-fixture-id".to_owned(),
        direction: FixtureSessionDescriptionDirection::Offer,
    };
    let candidate = FixtureIceCandidate {
        session_id: session("fixture-session"),
        fixture_candidate_id: "candidate-fixture-id".to_owned(),
    };

    assert_eq!(session_description.fixture_sdp_id, "offer-fixture-id");
    assert_eq!(candidate.fixture_candidate_id, "candidate-fixture-id");
}

#[test]
fn signaling_fixture_rejects_offer_and_answer_direction_mismatch() {
    let mut state = ReferenceSignalingState::default();
    apply_reference_signaling(
        &mut state,
        &input(
            SignalingCommandKind::JoinRoom,
            ReferenceSignalingPayload::JoinRoom,
        ),
    )
    .expect("join must succeed before fixture direction checks");

    let answer_as_offer = input(
        SignalingCommandKind::SendOffer,
        ReferenceSignalingPayload::SendOffer {
            session: FixtureSessionDescription {
                session_id: session("fixture-offer-session"),
                fixture_sdp_id: "answer-as-offer".to_owned(),
                direction: FixtureSessionDescriptionDirection::Answer,
            },
        },
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &answer_as_offer),
        Err(ReferenceSignalingError::InvalidFixtureIdentity)
    );

    let offer_as_answer = input(
        SignalingCommandKind::SendAnswer,
        ReferenceSignalingPayload::SendAnswer {
            session: FixtureSessionDescription {
                session_id: session("fixture-answer-session"),
                fixture_sdp_id: "offer-as-answer".to_owned(),
                direction: FixtureSessionDescriptionDirection::Offer,
            },
        },
    );
    assert_eq!(
        apply_reference_signaling(&mut state, &offer_as_answer),
        Err(ReferenceSignalingError::InvalidFixtureIdentity)
    );
}
