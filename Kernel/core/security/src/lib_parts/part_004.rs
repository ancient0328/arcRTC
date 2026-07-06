/// verifier profile の core-owned opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerifierProfileRef(OpaqueReference);

impl VerifierProfileRef {
    /// accepted opaque reference から verifier profile reference を作ります。
    pub fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。profile 内容や key material として扱いません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// credential subject の core-owned opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CredentialSubjectRef(OpaqueReference);

impl CredentialSubjectRef {
    /// accepted opaque reference から credential subject reference を作ります。
    pub fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。application user identity と同一視しません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// verifier key の core-owned opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CredentialVerifierKeyRef(OpaqueReference);

impl CredentialVerifierKeyRef {
    /// accepted opaque reference から verifier key reference を作ります。
    pub fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。raw key material として扱いません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// key selection の core-facing context です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySelectionContext {
    subject_ref: Option<CredentialSubjectRef>,
    key_ref: Option<CredentialVerifierKeyRef>,
    selected: bool,
}

impl KeySelectionContext {
    /// verifier backend が変換した subject/key reference と selection 結果を保持します。
    pub const fn new(
        subject_ref: Option<CredentialSubjectRef>,
        key_ref: Option<CredentialVerifierKeyRef>,
        selected: bool,
    ) -> Self {
        Self {
            subject_ref,
            key_ref,
            selected,
        }
    }
}

/// credential revocation state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialRevocationState {
    /// revocation list 上で失効していません。
    NotRevoked,
    /// revocation list 上で失効しています。
    Revoked,
    /// revocation state を判定できません。
    Unavailable,
}

/// verifier backend に渡す core-owned input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialVerifierPortInput {
    credential_ref: CredentialRef,
    verifier_profile_ref: VerifierProfileRef,
    key_selection_context: KeySelectionContext,
    revocation_state: CredentialRevocationState,
}

impl CredentialVerifierPortInput {
    /// raw credential を含まない verifier input を作ります。
    pub fn new(
        credential_ref: CredentialRef,
        verifier_profile_ref: VerifierProfileRef,
        key_selection_context: KeySelectionContext,
        revocation_state: CredentialRevocationState,
    ) -> Self {
        Self {
            credential_ref,
            verifier_profile_ref,
            key_selection_context,
            revocation_state,
        }
    }

    /// credential reference です。
    pub const fn credential_ref(&self) -> &CredentialRef {
        &self.credential_ref
    }

    /// verifier profile reference です。
    pub const fn verifier_profile_ref(&self) -> &VerifierProfileRef {
        &self.verifier_profile_ref
    }
}

/// credential verification decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialVerificationDecision {
    /// verification accepted.
    Accepted,
    /// verification rejected.
    Rejected,
}

/// verifier backend から返す core-owned output です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialVerifierPortOutput {
    decision: CredentialVerificationDecision,
    subject_ref: CredentialSubjectRef,
    key_ref: CredentialVerifierKeyRef,
}

impl CredentialVerifierPortOutput {
    /// verifier decision と opaque references を束ねます。
    pub const fn new(
        decision: CredentialVerificationDecision,
        subject_ref: CredentialSubjectRef,
        key_ref: CredentialVerifierKeyRef,
    ) -> Self {
        Self {
            decision,
            subject_ref,
            key_ref,
        }
    }

    /// verification decision です。
    pub const fn decision(&self) -> CredentialVerificationDecision {
        self.decision
    }

    /// credential subject reference です。
    pub const fn subject_ref(&self) -> &CredentialSubjectRef {
        &self.subject_ref
    }

    /// verifier key reference です。
    pub const fn key_ref(&self) -> &CredentialVerifierKeyRef {
        &self.key_ref
    }
}

/// credential verifier failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialVerifierFailureKind {
    /// subject reference がありません。
    SubjectRefMissing,
    /// verifier key reference がありません。
    KeyRefMissing,
    /// key selection が成立していません。
    KeySelectionRejected,
    /// revocation state を判定できません。
    RevocationStateUnavailable,
    /// verifier key material の read に失敗しました。
    KeyMaterialReadFailed,
    /// verifier key material の driver-local parse に失敗しました。
    KeyMaterialParseFailed,
    /// concrete crypto backend が verification を実行できません。
    CryptoBackendRejected,
}

/// verifier input を accepted/rejected/failure の閉集合に写像します。
pub fn decide_credential_verification(
    input: CredentialVerifierPortInput,
) -> Result<CredentialVerifierPortOutput, CredentialVerifierFailureKind> {
    let CredentialVerifierPortInput {
        credential_ref: _credential_ref,
        verifier_profile_ref: _verifier_profile_ref,
        key_selection_context,
        revocation_state,
    } = input;

    let subject_ref = key_selection_context
        .subject_ref
        .ok_or(CredentialVerifierFailureKind::SubjectRefMissing)?;
    let key_ref = key_selection_context
        .key_ref
        .ok_or(CredentialVerifierFailureKind::KeyRefMissing)?;

    if !key_selection_context.selected {
        return Err(CredentialVerifierFailureKind::KeySelectionRejected);
    }

    match revocation_state {
        CredentialRevocationState::NotRevoked => Ok(CredentialVerifierPortOutput::new(
            CredentialVerificationDecision::Accepted,
            subject_ref,
            key_ref,
        )),
        CredentialRevocationState::Revoked => Ok(CredentialVerifierPortOutput::new(
            CredentialVerificationDecision::Rejected,
            subject_ref,
            key_ref,
        )),
        CredentialRevocationState::Unavailable => {
            Err(CredentialVerifierFailureKind::RevocationStateUnavailable)
        }
    }
}
