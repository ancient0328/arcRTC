/// credential integrity policy reference の評価状態です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialPolicyReferenceState {
    /// policy reference is present and admissible.
    Present,
    /// policy reference is present but rejected.
    Rejected,
    /// policy reference cannot be evaluated.
    Unavailable,
}

/// verified credential の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerifiedCredentialRef {
    value: OpaqueReference,
    state: CredentialPolicyReferenceState,
}

impl VerifiedCredentialRef {
    /// verified credential reference と policy evaluation state を保持します。
    pub const fn new(value: OpaqueReference, state: CredentialPolicyReferenceState) -> Self {
        Self { value, state }
    }

    /// opaque value です。raw credential ではありません。
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// message integrity policy の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageIntegrityPolicyRef {
    value: OpaqueReference,
    state: CredentialPolicyReferenceState,
}

impl MessageIntegrityPolicyRef {
    /// message integrity policy reference と evaluation state を保持します。
    pub const fn new(value: OpaqueReference, state: CredentialPolicyReferenceState) -> Self {
        Self { value, state }
    }

    /// opaque value です。HMAC secret ではありません。
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// nonce policy の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoncePolicyRef {
    value: OpaqueReference,
    state: CredentialPolicyReferenceState,
}

impl NoncePolicyRef {
    /// nonce policy reference と evaluation state を保持します。
    pub const fn new(value: OpaqueReference, state: CredentialPolicyReferenceState) -> Self {
        Self { value, state }
    }

    /// opaque value です。nonce 本体ではありません。
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// TURN credential integrity policy input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnCredentialPolicyInput {
    verified_credential_ref: VerifiedCredentialRef,
    integrity_policy_ref: MessageIntegrityPolicyRef,
    nonce_policy_ref: NoncePolicyRef,
}

impl TurnCredentialPolicyInput {
    /// verified credential / integrity policy / nonce policy の references を束ねます。
    pub const fn new(
        verified_credential_ref: VerifiedCredentialRef,
        integrity_policy_ref: MessageIntegrityPolicyRef,
        nonce_policy_ref: NoncePolicyRef,
    ) -> Self {
        Self {
            verified_credential_ref,
            integrity_policy_ref,
            nonce_policy_ref,
        }
    }
}

/// credential integrity policy decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialIntegrityPolicyDecision {
    /// credential integrity policy accepted.
    Accepted,
    /// credential integrity policy rejected.
    Rejected,
    /// credential integrity policy could not be evaluated.
    Failed,
}

/// TURN credential integrity policy input を closed decision に写像します。
pub fn decide_credential_integrity_policy(
    input: TurnCredentialPolicyInput,
) -> CredentialIntegrityPolicyDecision {
    let states = [
        input.verified_credential_ref.state,
        input.integrity_policy_ref.state,
        input.nonce_policy_ref.state,
    ];

    let mut index = 0;
    while index < states.len() {
        match states[index] {
            CredentialPolicyReferenceState::Unavailable => {
                return CredentialIntegrityPolicyDecision::Failed;
            }
            CredentialPolicyReferenceState::Rejected => {
                return CredentialIntegrityPolicyDecision::Rejected;
            }
            CredentialPolicyReferenceState::Present => {}
        }
        index += 1;
    }

    CredentialIntegrityPolicyDecision::Accepted
}
