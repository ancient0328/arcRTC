/// room admission policy の閉集合状態です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomAdmissionPolicyState {
    /// join admission can proceed.
    Open,
    /// room policy rejects new joins.
    NotAcceptingJoin,
    /// room capacity has been reached.
    CapacityExceeded,
    /// participant admission capacity has been reached.
    AdmissionCapacityExceeded,
}

/// room admission policy の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomAdmissionPolicyRef {
    value: OpaqueReference,
    state: RoomAdmissionPolicyState,
}

impl RoomAdmissionPolicyRef {
    /// admission policy reference と closed policy state を保持します。
    pub const fn new(value: OpaqueReference, state: RoomAdmissionPolicyState) -> Self {
        Self { value, state }
    }

    /// opaque value です。policy payload ではありません。
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// join admission input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinAdmissionInput {
    verified_credential_ref: VerifiedCredentialRef,
    room_ref: RoomId,
    participant_ref: ParticipantId,
    admission_policy_ref: RoomAdmissionPolicyRef,
}

impl JoinAdmissionInput {
    /// raw credential を含まない join admission input を作ります。
    pub const fn new(
        verified_credential_ref: VerifiedCredentialRef,
        room_ref: RoomId,
        participant_ref: ParticipantId,
        admission_policy_ref: RoomAdmissionPolicyRef,
    ) -> Self {
        Self {
            verified_credential_ref,
            room_ref,
            participant_ref,
            admission_policy_ref,
        }
    }

    /// verified credential reference です。
    pub const fn verified_credential_ref(&self) -> &VerifiedCredentialRef {
        &self.verified_credential_ref
    }

    /// room reference です。
    pub const fn room_ref(&self) -> &RoomId {
        &self.room_ref
    }
}

/// accepted participant reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AcceptedParticipantRef(ParticipantId);

impl AcceptedParticipantRef {
    /// participant reference を accepted join result として保持します。
    pub const fn new(participant_ref: ParticipantId) -> Self {
        Self(participant_ref)
    }

    /// participant reference です。
    pub const fn participant_ref(&self) -> &ParticipantId {
        &self.0
    }
}

/// rejected join reason です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RejectedJoinReason {
    kind: SignalingFailureKind,
}

impl RejectedJoinReason {
    /// closed signaling failure kind から rejected join reason を作ります。
    pub const fn new(kind: SignalingFailureKind) -> Self {
        Self { kind }
    }

    /// rejected join reason kind です。
    pub const fn kind(&self) -> SignalingFailureKind {
        self.kind
    }
}

/// join admission decision の閉集合です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinAdmissionDecision {
    /// join accepted.
    Accepted(AcceptedParticipantRef),
    /// join rejected with closed reason.
    Rejected(RejectedJoinReason),
}

/// verified credential reference と room admission policy から join admission を決定します。
pub fn decide_join_admission(input: JoinAdmissionInput) -> JoinAdmissionDecision {
    match input.admission_policy_ref.state {
        RoomAdmissionPolicyState::Open => {
            JoinAdmissionDecision::Accepted(AcceptedParticipantRef::new(input.participant_ref))
        }
        RoomAdmissionPolicyState::NotAcceptingJoin => JoinAdmissionDecision::Rejected(
            RejectedJoinReason::new(SignalingFailureKind::RoomNotAcceptingJoin),
        ),
        RoomAdmissionPolicyState::CapacityExceeded => JoinAdmissionDecision::Rejected(
            RejectedJoinReason::new(SignalingFailureKind::RoomCapacityExceeded),
        ),
        RoomAdmissionPolicyState::AdmissionCapacityExceeded => JoinAdmissionDecision::Rejected(
            RejectedJoinReason::new(SignalingFailureKind::AdmissionCapacityExceeded),
        ),
    }
}
