#![allow(non_snake_case)]

use arcrtc_core_identity::{OpaqueReference, ParticipantId, ReferenceAuthority, RoomId};
use arcrtc_core_security::{CredentialPolicyReferenceState, VerifiedCredentialRef};
use arcrtc_core_signaling::{
    apply_membership_command, decide_join_admission, IdempotentCommandDecision,
    JoinAdmissionDecision, JoinAdmissionInput, LeaveDecision, MembershipCommand,
    MembershipCommandDecision, ParticipantLifecycleState, RejectedJoinReason,
    RoomAdmissionPolicyRef, RoomAdmissionPolicyState, RoomMembershipState, SignalingFailureKind,
    TimeoutDecision,
};

// Test Roadmap の named assertion rule に合わせ、二重アンダースコア名を維持します。

fn reference(value: &str, authority: ReferenceAuthority) -> OpaqueReference {
    OpaqueReference::accept(value, authority).expect("test fixture uses accepted references")
}

fn room_ref() -> RoomId {
    RoomId::new(reference("room:sig-01", ReferenceAuthority::CorePolicy))
}

fn participant_ref() -> ParticipantId {
    ParticipantId::new(reference(
        "participant:sig-01",
        ReferenceAuthority::CorePolicy,
    ))
}

fn credential_ref() -> VerifiedCredentialRef {
    VerifiedCredentialRef::new(
        reference(
            "credential:sig-01",
            ReferenceAuthority::DriverCredentialConversion,
        ),
        CredentialPolicyReferenceState::Present,
    )
}

fn membership(state: ParticipantLifecycleState) -> RoomMembershipState {
    RoomMembershipState::new(room_ref(), participant_ref(), state, None)
}

#[test]
fn t_sig_01_join_success_lifecycle() {
    assert_t_sig_01__duplicate_idempotent_command();
    assert_t_sig_01__leave();
    assert_t_sig_01__timeout();
    assert_t_sig_01__rejected_credential();
}

fn assert_t_sig_01__duplicate_idempotent_command() {
    let first = apply_membership_command(
        membership(ParticipantLifecycleState::Active),
        MembershipCommand::FirstCommand,
    );
    assert_eq!(
        first.decision(),
        MembershipCommandDecision::Idempotent(IdempotentCommandDecision::FirstCommand)
    );

    let duplicate =
        apply_membership_command(first.state().clone(), MembershipCommand::DuplicateCommand);
    assert_eq!(
        duplicate.decision(),
        MembershipCommandDecision::Idempotent(IdempotentCommandDecision::DuplicateObserved)
    );
}

fn assert_t_sig_01__leave() {
    let result = apply_membership_command(
        membership(ParticipantLifecycleState::Active),
        MembershipCommand::Leave,
    );

    assert_eq!(
        result.decision(),
        MembershipCommandDecision::Leave(LeaveDecision::Accepted)
    );
    assert_eq!(
        result.state().lifecycle_state(),
        ParticipantLifecycleState::Leaving
    );
}

fn assert_t_sig_01__timeout() {
    let result = apply_membership_command(
        membership(ParticipantLifecycleState::Active),
        MembershipCommand::Timeout,
    );

    assert_eq!(
        result.decision(),
        MembershipCommandDecision::Timeout(TimeoutDecision::TimedOut)
    );
    assert_eq!(
        result.state().lifecycle_state(),
        ParticipantLifecycleState::TimedOut
    );
}

fn assert_t_sig_01__rejected_credential() {
    let decision = decide_join_admission(JoinAdmissionInput::new(
        credential_ref(),
        room_ref(),
        participant_ref(),
        RoomAdmissionPolicyRef::new(
            reference("admission:sig-01", ReferenceAuthority::CorePolicy),
            RoomAdmissionPolicyState::NotAcceptingJoin,
        ),
    ));

    assert_eq!(
        decision,
        JoinAdmissionDecision::Rejected(RejectedJoinReason::new(
            SignalingFailureKind::RoomNotAcceptingJoin
        ))
    );
}
