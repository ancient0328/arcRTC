/// TURN client の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TurnClientRef(OpaqueReference);

impl TurnClientRef {
    /// accepted opaque reference から TURN client reference を作ります。
    pub fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。network address ではありません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// allocation policy state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllocationPolicyState {
    /// allocation can be accepted.
    AllowsAllocation,
    /// allocation is rejected by policy.
    RejectsAllocation,
    /// allocation policy cannot be evaluated.
    PolicyUnavailable,
}

/// allocation policy の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AllocationPolicyRef {
    value: OpaqueReference,
    state: AllocationPolicyState,
}

impl AllocationPolicyRef {
    /// allocation policy reference と closed state を保持します。
    pub const fn new(value: OpaqueReference, state: AllocationPolicyState) -> Self {
        Self { value, state }
    }

    /// opaque value です。policy payload ではありません。
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// allocation lifetime policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllocationLifetimePolicy {
    /// requested lifetime is accepted.
    Accepted,
    /// requested lifetime is expired.
    Expired,
    /// requested lifetime is rejected by policy.
    Rejected,
}

/// credentialed TURN allocation input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialedAllocateInput {
    verified_credential_ref: VerifiedCredentialRef,
    client_ref: TurnClientRef,
    requested_lifetime: TurnRequestedLifetimeSeconds,
    allocation_policy_ref: AllocationPolicyRef,
}

impl CredentialedAllocateInput {
    /// verified credential と allocation policy を持つ allocation input を作ります。
    pub const fn new(
        verified_credential_ref: VerifiedCredentialRef,
        client_ref: TurnClientRef,
        requested_lifetime: TurnRequestedLifetimeSeconds,
        allocation_policy_ref: AllocationPolicyRef,
    ) -> Self {
        Self {
            verified_credential_ref,
            client_ref,
            requested_lifetime,
            allocation_policy_ref,
        }
    }

    /// verified credential reference です。
    pub const fn verified_credential_ref(&self) -> &VerifiedCredentialRef {
        &self.verified_credential_ref
    }
}

/// active allocation state です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveAllocationState {
    client_ref: TurnClientRef,
    requested_lifetime: TurnRequestedLifetimeSeconds,
}

impl ActiveAllocationState {
    /// active allocation state を作ります。
    pub const fn new(
        client_ref: TurnClientRef,
        requested_lifetime: TurnRequestedLifetimeSeconds,
    ) -> Self {
        Self {
            client_ref,
            requested_lifetime,
        }
    }

    /// client reference です。
    pub const fn client_ref(&self) -> &TurnClientRef {
        &self.client_ref
    }
}

/// TURN allocation rejection reason です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnAllocationRejectionReason {
    kind: TurnFailureKind,
}

impl TurnAllocationRejectionReason {
    /// closed TURN failure kind から allocation rejection reason を作ります。
    pub const fn new(kind: TurnFailureKind) -> Self {
        Self { kind }
    }
}

/// TURN allocation decision の閉集合です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnAllocationDecision {
    /// allocation accepted.
    Accepted(ActiveAllocationState),
    /// allocation rejected.
    Rejected(TurnAllocationRejectionReason),
    /// allocation expired.
    Expired(TurnAllocationRejectionReason),
}

/// credentialed allocation input を allocation decision に写像します。
pub fn decide_turn_allocation(input: CredentialedAllocateInput) -> TurnAllocationDecision {
    match input.allocation_policy_ref.state {
        AllocationPolicyState::PolicyUnavailable | AllocationPolicyState::RejectsAllocation => {
            return TurnAllocationDecision::Rejected(TurnAllocationRejectionReason::new(
                TurnFailureKind::CredentialInvalid,
            ));
        }
        AllocationPolicyState::AllowsAllocation => {}
    }

    let lifetime_policy = if input.requested_lifetime.as_u32() == 0 {
        AllocationLifetimePolicy::Expired
    } else {
        AllocationLifetimePolicy::Accepted
    };

    match lifetime_policy {
        AllocationLifetimePolicy::Accepted => TurnAllocationDecision::Accepted(
            ActiveAllocationState::new(input.client_ref, input.requested_lifetime),
        ),
        AllocationLifetimePolicy::Expired => TurnAllocationDecision::Expired(
            TurnAllocationRejectionReason::new(TurnFailureKind::AllocationLifetimeExceeded),
        ),
        AllocationLifetimePolicy::Rejected => TurnAllocationDecision::Rejected(
            TurnAllocationRejectionReason::new(TurnFailureKind::TurnLifetimeViolation),
        ),
    }
}
