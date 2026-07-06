/// rotation window boundary の opaque instant reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RotationInstantRef(OpaqueReference);

impl RotationInstantRef {
    /// accepted opaque reference から rotation instant reference を作ります。
    pub fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。clock/timestamp semantics は core/time 側で扱います。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// key rotation overlap policy の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyRotationOverlapPolicy {
    /// overlap を許可しません。
    NoOverlap,
    /// 明示された bounded overlap だけを許可します。
    BoundedOverlap,
}

/// key rotation window source model です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRotationWindow {
    active_from: RotationInstantRef,
    active_until: RotationInstantRef,
    overlap_policy: KeyRotationOverlapPolicy,
}

impl KeyRotationWindow {
    /// active interval と overlap policy を固定します。
    pub const fn new(
        active_from: RotationInstantRef,
        active_until: RotationInstantRef,
        overlap_policy: KeyRotationOverlapPolicy,
    ) -> Self {
        Self {
            active_from,
            active_until,
            overlap_policy,
        }
    }

    /// active_from reference です。
    pub const fn active_from(&self) -> &RotationInstantRef {
        &self.active_from
    }

    /// active_until reference です。
    pub const fn active_until(&self) -> &RotationInstantRef {
        &self.active_until
    }

    /// overlap policy です。
    pub const fn overlap_policy(&self) -> KeyRotationOverlapPolicy {
        self.overlap_policy
    }
}

/// key revocation list の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyRevocationListRef(OpaqueReference);

impl KeyRevocationListRef {
    /// accepted opaque reference から revocation list reference を作ります。
    pub fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。revocation list payload ではありません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// secret rotation decision の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecretRotationDecisionRef(OpaqueReference);

impl SecretRotationDecisionRef {
    /// accepted opaque reference から secret rotation decision reference を作ります。
    pub fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。raw secret や generation material ではありません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// key selection decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeySelectionDecision {
    /// key was selected.
    Selected,
    /// key selection was rejected by policy.
    Rejected,
    /// key rotation window is expired.
    Expired,
    /// key is revoked.
    Revoked,
    /// key selection could not be evaluated.
    Failed,
}

/// verification 用 key selection context です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySelectionVerificationContext {
    key_ref: Option<CredentialVerifierKeyRef>,
    rotation_window: KeyRotationWindow,
    revocation_list_ref: KeyRevocationListRef,
    rotation_decision_ref: SecretRotationDecisionRef,
    evaluation: KeySelectionEvaluation,
}

/// key selection evaluation flags です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeySelectionEvaluation {
    rejected_by_policy: bool,
    expired: bool,
    revoked: bool,
    evaluation_failed: bool,
}

impl KeySelectionEvaluation {
    /// key selection の評価結果を closed flags として保持します。
    pub const fn new(
        rejected_by_policy: bool,
        expired: bool,
        revoked: bool,
        evaluation_failed: bool,
    ) -> Self {
        Self {
            rejected_by_policy,
            expired,
            revoked,
            evaluation_failed,
        }
    }
}

impl KeySelectionVerificationContext {
    /// key selection に必要な opaque references と closed flags を束ねます。
    pub const fn new(
        key_ref: Option<CredentialVerifierKeyRef>,
        rotation_window: KeyRotationWindow,
        revocation_list_ref: KeyRevocationListRef,
        rotation_decision_ref: SecretRotationDecisionRef,
        evaluation: KeySelectionEvaluation,
    ) -> Self {
        Self {
            key_ref,
            rotation_window,
            revocation_list_ref,
            rotation_decision_ref,
            evaluation,
        }
    }

    /// rotation window です。
    pub const fn rotation_window(&self) -> &KeyRotationWindow {
        &self.rotation_window
    }

    /// revocation list reference です。
    pub const fn revocation_list_ref(&self) -> &KeyRevocationListRef {
        &self.revocation_list_ref
    }

    /// secret rotation decision reference です。
    pub const fn rotation_decision_ref(&self) -> &SecretRotationDecisionRef {
        &self.rotation_decision_ref
    }
}

/// verification 用 key selection を closed decision に写像します。
pub fn select_key_for_verification(context: KeySelectionVerificationContext) -> KeySelectionDecision {
    if context.evaluation.evaluation_failed {
        return KeySelectionDecision::Failed;
    }
    if context.key_ref.is_none() || context.evaluation.rejected_by_policy {
        return KeySelectionDecision::Rejected;
    }
    if context.evaluation.revoked {
        return KeySelectionDecision::Revoked;
    }
    if context.evaluation.expired {
        return KeySelectionDecision::Expired;
    }

    KeySelectionDecision::Selected
}
