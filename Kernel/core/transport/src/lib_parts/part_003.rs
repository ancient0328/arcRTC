/// ICE candidate と transport session の関係を表す opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransportIceRelationRef(OpaqueReference);

impl TransportIceRelationRef {
    /// accepted opaque reference から transport ICE relation reference を作ります。
    pub fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。connectivity proof として扱いません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// transport policy の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransportPolicyRef {
    value: OpaqueReference,
    state: TransportPolicyState,
}

impl TransportPolicyRef {
    /// transport policy reference と closed state を保持します。
    pub const fn new(value: OpaqueReference, state: TransportPolicyState) -> Self {
        Self { value, state }
    }

    /// opaque value です。policy payload ではありません。
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// transport policy state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportPolicyState {
    /// candidate relation can be accepted.
    AllowsCandidateRelation,
    /// candidate relation is rejected by policy.
    RejectsCandidateRelation,
    /// policy cannot be evaluated.
    PolicyUnavailable,
}

/// transport candidate relation input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportCandidateRelationInput {
    relation_ref: TransportIceRelationRef,
    policy_ref: TransportPolicyRef,
    candidate_ref: IceCandidateRef,
}

impl TransportCandidateRelationInput {
    /// relation / policy / candidate references を束ねます。
    pub const fn new(
        relation_ref: TransportIceRelationRef,
        policy_ref: TransportPolicyRef,
        candidate_ref: IceCandidateRef,
    ) -> Self {
        Self {
            relation_ref,
            policy_ref,
            candidate_ref,
        }
    }

    /// candidate reference です。
    pub const fn candidate_ref(&self) -> &IceCandidateRef {
        &self.candidate_ref
    }
}

/// transport candidate relation rejection です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportCandidateRelationRejection {
    reason: IceFailureKind,
}

impl TransportCandidateRelationRejection {
    /// closed ICE failure reason を保持します。
    pub const fn new(reason: IceFailureKind) -> Self {
        Self { reason }
    }

    /// rejection reason です。
    pub const fn reason(&self) -> IceFailureKind {
        self.reason
    }
}

/// transport candidate relation decision の閉集合です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportCandidateRelationDecision {
    /// relation accepted.
    Accepted(TransportIceRelationRef),
    /// relation rejected.
    Rejected(TransportCandidateRelationRejection),
}

/// ICE candidate を transport relation に閉集合で関連付けます。
pub fn relate_ice_candidate_to_transport(
    input: TransportCandidateRelationInput,
) -> TransportCandidateRelationDecision {
    match input.policy_ref.state {
        TransportPolicyState::AllowsCandidateRelation => {
            TransportCandidateRelationDecision::Accepted(input.relation_ref)
        }
        TransportPolicyState::RejectsCandidateRelation => {
            TransportCandidateRelationDecision::Rejected(TransportCandidateRelationRejection::new(
                IceFailureKind::IceCandidatePolicyViolation,
            ))
        }
        TransportPolicyState::PolicyUnavailable => {
            TransportCandidateRelationDecision::Rejected(TransportCandidateRelationRejection::new(
                IceFailureKind::IceCandidateMappingInvalid,
            ))
        }
    }
}
