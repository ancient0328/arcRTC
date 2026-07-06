/// TURN nonce evaluation state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnNonceEvaluationState {
    /// nonce is accepted for replay evaluation.
    Present,
    /// nonce is malformed.
    Malformed,
    /// nonce has already been used.
    Replayed,
    /// nonce evaluation failed.
    Unavailable,
}

/// TURN nonce policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnNoncePolicy {
    replay_window_declared: bool,
    amplification_bound_declared: bool,
    amplification_within_bound: bool,
}

impl TurnNoncePolicy {
    /// nonce replay window と amplification bound の宣言状態を保持します。
    pub const fn new(
        replay_window_declared: bool,
        amplification_bound_declared: bool,
        amplification_within_bound: bool,
    ) -> Self {
        Self {
            replay_window_declared,
            amplification_bound_declared,
            amplification_within_bound,
        }
    }
}

/// TURN nonce の opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TurnNonceRef {
    value: OpaqueReference,
    state: TurnNonceEvaluationState,
    policy: TurnNoncePolicy,
}

impl TurnNonceRef {
    /// nonce reference と evaluation state / policy を保持します。
    pub const fn new(
        value: OpaqueReference,
        state: TurnNonceEvaluationState,
        policy: TurnNoncePolicy,
    ) -> Self {
        Self {
            value,
            state,
            policy,
        }
    }

    /// opaque value です。nonce secret ではありません。
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// TURN replay decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnReplayDecision {
    /// replay checks accepted.
    Accepted,
    /// replay checks rejected.
    Rejected(TurnFailureKind),
}

/// amplification bound decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmplificationBoundDecision {
    /// amplification is within bound.
    WithinBound,
    /// amplification bound rejected the packet.
    Rejected(TurnFailureKind),
}

/// TURN security classification input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnSecurityClassificationInput {
    credential_integrity_policy_ref: MessageIntegrityPolicyRef,
    packet_ref: PacketId,
    nonce_ref: TurnNonceRef,
}

impl TurnSecurityClassificationInput {
    /// credential integrity policy / packet / nonce references を束ねます。
    pub const fn new(
        credential_integrity_policy_ref: MessageIntegrityPolicyRef,
        packet_ref: PacketId,
        nonce_ref: TurnNonceRef,
    ) -> Self {
        Self {
            credential_integrity_policy_ref,
            packet_ref,
            nonce_ref,
        }
    }

    /// packet reference です。
    pub const fn packet_ref(&self) -> &PacketId {
        &self.packet_ref
    }

    /// credential integrity policy reference です。
    pub const fn credential_integrity_policy_ref(&self) -> &MessageIntegrityPolicyRef {
        &self.credential_integrity_policy_ref
    }
}

/// TURN security classification decision です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnSecurityClassificationDecision {
    replay_decision: TurnReplayDecision,
    amplification_decision: AmplificationBoundDecision,
    reason: Option<TurnFailureKind>,
}

impl TurnSecurityClassificationDecision {
    /// replay / amplification decisions と closed reason を保持します。
    pub const fn new(
        replay_decision: TurnReplayDecision,
        amplification_decision: AmplificationBoundDecision,
        reason: Option<TurnFailureKind>,
    ) -> Self {
        Self {
            replay_decision,
            amplification_decision,
            reason,
        }
    }
}

/// malformed / replay / amplification failure を closed reason に写像します。
pub fn classify_turn_security(
    input: TurnSecurityClassificationInput,
) -> TurnSecurityClassificationDecision {
    match input.nonce_ref.state {
        TurnNonceEvaluationState::Malformed => {
            return TurnSecurityClassificationDecision::new(
                TurnReplayDecision::Rejected(TurnFailureKind::MalformedTurnMessage),
                AmplificationBoundDecision::Rejected(TurnFailureKind::MalformedTurnMessage),
                Some(TurnFailureKind::MalformedTurnMessage),
            );
        }
        TurnNonceEvaluationState::Replayed => {
            return TurnSecurityClassificationDecision::new(
                TurnReplayDecision::Rejected(TurnFailureKind::CredentialInvalid),
                AmplificationBoundDecision::WithinBound,
                Some(TurnFailureKind::CredentialInvalid),
            );
        }
        TurnNonceEvaluationState::Unavailable => {
            return TurnSecurityClassificationDecision::new(
                TurnReplayDecision::Rejected(TurnFailureKind::CredentialMissing),
                AmplificationBoundDecision::Rejected(TurnFailureKind::CredentialMissing),
                Some(TurnFailureKind::CredentialMissing),
            );
        }
        TurnNonceEvaluationState::Present => {}
    }

    if !input.nonce_ref.policy.replay_window_declared {
        return TurnSecurityClassificationDecision::new(
            TurnReplayDecision::Rejected(TurnFailureKind::CredentialInvalid),
            AmplificationBoundDecision::WithinBound,
            Some(TurnFailureKind::CredentialInvalid),
        );
    }
    if !input.nonce_ref.policy.amplification_bound_declared
        || !input.nonce_ref.policy.amplification_within_bound
    {
        return TurnSecurityClassificationDecision::new(
            TurnReplayDecision::Accepted,
            AmplificationBoundDecision::Rejected(TurnFailureKind::TurnRelayQueueBoundExceeded),
            Some(TurnFailureKind::TurnRelayQueueBoundExceeded),
        );
    }

    TurnSecurityClassificationDecision::new(
        TurnReplayDecision::Accepted,
        AmplificationBoundDecision::WithinBound,
        None,
    )
}
