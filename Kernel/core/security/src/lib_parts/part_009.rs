/// mutual trust proof の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MutualTrustProofClass {
    /// mTLS proof.
    Mtls,
    /// signed assertion proof.
    SignedAssertion,
    /// pinned key proof.
    PinnedKey,
}

impl MutualTrustProofClass {
    /// mutual trust proof class の stable code です。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mtls => "mtls",
            Self::SignedAssertion => "signed_assertion",
            Self::PinnedKey => "pinned_key",
        }
    }
}

/// secure media peer verification decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecureMediaPeerVerificationDecision {
    /// secure media peer proof を受理します。
    Accepted,
    /// secure media peer proof を拒否します。
    Rejected,
}

/// internal service identity decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceIdentityDecision {
    /// internal service identity を受理します。
    Accepted,
    /// internal service identity を拒否します。
    Rejected,
}

/// internal service identity input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalServiceIdentityInput {
    /// internal service identity reference です。
    pub service_identity_ref: &'static str,
    /// mutual trust proof class です。
    pub mutual_trust_proof_class: MutualTrustProofClass,
    /// trust policy reference です。
    pub trust_policy_ref: &'static str,
}

impl InternalServiceIdentityInput {
    /// internal service identity input を束ねます。
    pub const fn new(
        service_identity_ref: &'static str,
        mutual_trust_proof_class: MutualTrustProofClass,
        trust_policy_ref: &'static str,
    ) -> Self {
        Self {
            service_identity_ref,
            mutual_trust_proof_class,
            trust_policy_ref,
        }
    }
}

/// secure media peer verification input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecureMediaPeerVerificationInput {
    /// secure media peer reference です。
    pub peer_ref: &'static str,
    /// mutual trust proof class です。
    pub mutual_trust_proof_class: MutualTrustProofClass,
    /// trust policy reference です。
    pub trust_policy_ref: &'static str,
}

impl SecureMediaPeerVerificationInput {
    /// secure media peer verification input を束ねます。
    pub const fn new(
        peer_ref: &'static str,
        mutual_trust_proof_class: MutualTrustProofClass,
        trust_policy_ref: &'static str,
    ) -> Self {
        Self {
            peer_ref,
            mutual_trust_proof_class,
            trust_policy_ref,
        }
    }
}

/// internal service identity を closed decision へ写像します。
///
/// target domain authorization は返しません。
pub fn decide_internal_service_identity(
    input: InternalServiceIdentityInput,
) -> InternalServiceIdentityDecision {
    if !input.service_identity_ref.is_empty()
        && !input.trust_policy_ref.is_empty()
        && !input.mutual_trust_proof_class.as_str().is_empty()
    {
        InternalServiceIdentityDecision::Accepted
    } else {
        InternalServiceIdentityDecision::Rejected
    }
}

/// secure media peer proof を closed decision へ写像します。
pub fn decide_secure_media_peer_verification(
    input: SecureMediaPeerVerificationInput,
) -> SecureMediaPeerVerificationDecision {
    if !input.peer_ref.is_empty()
        && !input.trust_policy_ref.is_empty()
        && !input.mutual_trust_proof_class.as_str().is_empty()
    {
        SecureMediaPeerVerificationDecision::Accepted
    } else {
        SecureMediaPeerVerificationDecision::Rejected
    }
}
