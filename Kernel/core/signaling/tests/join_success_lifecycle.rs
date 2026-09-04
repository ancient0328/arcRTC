use arcrtc_core_identity::{OpaqueReference, ParticipantId, ReferenceAuthority, RoomId};
use arcrtc_core_security::{CredentialPolicyReferenceState, VerifiedCredentialRef};
use arcrtc_core_signaling::{
    apply_membership_command, decide_join_admission, IdempotentCommandDecision,
    JoinAdmissionDecision, JoinAdmissionInput, LeaveDecision, MembershipCommand,
    MembershipCommandDecision, ParticipantLifecycleState, RejectedJoinReason,
    RoomAdmissionPolicyRef, RoomAdmissionPolicyState, RoomMembershipState, SignalingFailureKind,
    TimeoutDecision,
};

fn reference(value: &str, authority: ReferenceAuthority) -> OpaqueReference {
    OpaqueReference::accept(value, authority).expect("test fixture uses accepted references")
}

fn room_ref() -> RoomId {
    RoomId::new(reference(
        "room:signaling-lifecycle",
        ReferenceAuthority::CorePolicy,
    ))
}

fn participant_ref() -> ParticipantId {
    ParticipantId::new(reference(
        "participant:signaling-lifecycle",
        ReferenceAuthority::CorePolicy,
    ))
}

fn credential_ref() -> VerifiedCredentialRef {
    VerifiedCredentialRef::new(
        reference(
            "credential:signaling-lifecycle",
            ReferenceAuthority::DriverCredentialConversion,
        ),
        CredentialPolicyReferenceState::Present,
    )
}

fn membership(state: ParticipantLifecycleState) -> RoomMembershipState {
    RoomMembershipState::new(room_ref(), participant_ref(), state, None)
}

#[test]
fn join_success_lifecycle_preserves_state_transition() {
    assert_duplicate_idempotent_command();
    assert_leave();
    assert_timeout();
    assert_rejected_credential();
}

fn assert_duplicate_idempotent_command() {
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

fn assert_leave() {
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

fn assert_timeout() {
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

fn assert_rejected_credential() {
    let decision = decide_join_admission(JoinAdmissionInput::new(
        credential_ref(),
        room_ref(),
        participant_ref(),
        RoomAdmissionPolicyRef::new(
            reference(
                "admission:signaling-lifecycle",
                ReferenceAuthority::CorePolicy,
            ),
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
