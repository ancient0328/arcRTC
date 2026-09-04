impl CorrelationTrace {
    /// command/event chain の tracing reference を保持します。
    pub const fn new(correlation_id: CorrelationId) -> Self {
        Self { correlation_id }
    }

    /// tracing 用 correlation ID です。
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }
}

/// v0.2 initial architecture で許可された idempotency class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdempotencyClass {
    /// duplicate must be rejected or state machine が再評価します。
    NonIdempotentCommand,
    /// same command identity and same canonical payload digest の duplicate です。
    IdempotentSamePayload,
    /// same idempotency scope だが payload digest または target が異なる conflict です。
    IdempotentConflict,
    /// prior response may be replayed as external projection の候補です。
    ResponseReplayCandidate,
    /// SDK reconnect 後の client-local pending command replay です。
    SdkPendingCommandReplay,
    /// server side missed event replay です。別 policy がなければ禁止です。
    ServerEventReplay,
}

/// idempotency scope を表す core-owned opaque value です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyScope(String);

impl IdempotencyScope {
    /// scope value を作ります。
    pub fn new(value: impl Into<String>) -> Result<Self, IdempotencyValueError> {
        let value = value.into();
        validate_idempotency_value(&value)?;
        Ok(Self(value))
    }

    /// scope value です。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// explicit idempotency key です。CorrelationId とは別物です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// idempotency key を作ります。
    pub fn new(value: impl Into<String>) -> Result<Self, IdempotencyValueError> {
        let value = value.into();
        validate_idempotency_value(&value)?;
        Ok(Self(value))
    }

    /// idempotency key value です。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// payload semantics に影響する場合だけ使う canonical payload digest です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticPayloadDigest {
    algorithm: &'static str,
    digest: Vec<u8>,
}

impl SemanticPayloadDigest {
    /// canonical serialization 後の digest を保持します。
    pub fn new(algorithm: &'static str, digest: Vec<u8>) -> Result<Self, IdempotencyValueError> {
        if algorithm.is_empty() {
            return Err(IdempotencyValueError::Empty);
        }
        if algorithm.chars().any(char::is_control) {
            return Err(IdempotencyValueError::ControlCharacter);
        }
        if digest.is_empty() {
            return Err(IdempotencyValueError::Empty);
        }
        Ok(Self { algorithm, digest })
    }

    /// digest algorithm 名です。
    pub const fn algorithm(&self) -> &'static str {
        self.algorithm
    }

    /// digest bytes です。payload 本体ではありません。
    pub fn digest(&self) -> &[u8] {
        &self.digest
    }
}

/// replay window policy の境界です。具体的な時計や duration は core/time が所有します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayWindow {
    /// response replay は許可されません。
    NotReplayable,
    /// 明示された policy label の window 内だけ replay 候補になります。
    Bounded(&'static str),
}

/// idempotent command identity を構成する core-owned 要素です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandIdentity<TargetReference, ActorReference> {
    command_type: CommandType,
    scope: IdempotencyScope,
    target_reference: TargetReference,
    actor_reference: Option<ActorReference>,
    idempotency_key: IdempotencyKey,
    payload_digest: Option<SemanticPayloadDigest>,
    replay_window: ReplayWindow,
}

impl<TargetReference, ActorReference> CommandIdentity<TargetReference, ActorReference> {
    /// command identity を明示的な scope/key/payload digest で作ります。
    pub const fn new(
        command_type: CommandType,
        scope: IdempotencyScope,
        target_reference: TargetReference,
        actor_reference: Option<ActorReference>,
        idempotency_key: IdempotencyKey,
        payload_digest: Option<SemanticPayloadDigest>,
        replay_window: ReplayWindow,
    ) -> Self {
        Self {
            command_type,
            scope,
            target_reference,
            actor_reference,
            idempotency_key,
            payload_digest,
            replay_window,
        }
    }

    /// command type です。
    pub const fn command_type(&self) -> CommandType {
        self.command_type
    }

    /// idempotency scope です。
    pub const fn scope(&self) -> &IdempotencyScope {
        &self.scope
    }

    /// target reference です。
    pub const fn target_reference(&self) -> &TargetReference {
        &self.target_reference
    }

    /// actor / participant reference です。
    pub const fn actor_reference(&self) -> Option<&ActorReference> {
        self.actor_reference.as_ref()
    }

    /// explicit idempotency key です。
    pub const fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    /// semantic payload digest です。
    pub const fn payload_digest(&self) -> Option<&SemanticPayloadDigest> {
        self.payload_digest.as_ref()
    }

    /// replay window policy です。
    pub const fn replay_window(&self) -> ReplayWindow {
        self.replay_window
    }
}

/// response replay が参照する prior result です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PriorDecisionReference {
    prior_correlation_id: CorrelationId,
    prior_audit_event_id: Option<AuditEventId>,
}

impl PriorDecisionReference {
    /// prior decision の trace / audit reference を保持します。
    pub const fn new(
        prior_correlation_id: CorrelationId,
        prior_audit_event_id: Option<AuditEventId>,
    ) -> Self {
        Self {
            prior_correlation_id,
            prior_audit_event_id,
        }
    }

    /// prior decision の correlation ID です。
    pub const fn prior_correlation_id(&self) -> &CorrelationId {
        &self.prior_correlation_id
    }

    /// prior decision の audit event reference です。
    pub const fn prior_audit_event_id(&self) -> Option<&AuditEventId> {
        self.prior_audit_event_id.as_ref()
    }
}

/// idempotency / replay evaluation の結果です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyDecision<Reason> {
    class: IdempotencyClass,
    outcome: UseCaseOutcome,
    reason: DecisionReason<Reason>,
    prior_decision: Option<PriorDecisionReference>,
}

impl<Reason> IdempotencyDecision<Reason> {
    /// idempotency decision を outcome reason rule に従って作ります。
    pub fn new(
        class: IdempotencyClass,
        outcome: UseCaseOutcome,
        reason: DecisionReason<Reason>,
        prior_decision: Option<PriorDecisionReference>,
    ) -> Result<Self, DecisionShapeError> {
        match (outcome.requires_reason(), &reason) {
            (false, DecisionReason::Cataloged(_)) => {
                return Err(DecisionShapeError::SuccessMustNotCarryReason);
            }
            (true, DecisionReason::Absent) => {
                return Err(DecisionShapeError::NonSuccessRequiresReason);
            }
            _ => {}
        }
        Ok(Self {
            class,
            outcome,
            reason,
            prior_decision,
        })
    }

    /// idempotency class です。
    pub const fn class(&self) -> IdempotencyClass {
        self.class
    }

    /// authoritative outcome です。
    pub const fn outcome(&self) -> UseCaseOutcome {
        self.outcome
    }

    /// cataloged reason slot です。
    pub const fn reason(&self) -> &DecisionReason<Reason> {
        &self.reason
    }

    /// response replay が参照する prior decision です。
    pub const fn prior_decision(&self) -> Option<&PriorDecisionReference> {
        self.prior_decision.as_ref()
    }
}

/// idempotency / replay failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdempotencyFailureKind {
    /// correlation ID missing before core entry です。
    MissingCorrelationId,
    /// response/event chain の correlation mismatch です。
    CorrelationMismatch,
    /// duplicate rejected by idempotency rule です。
    DuplicateCommand,
    /// duplicate payload または target mismatch です。
    IdempotencyPayloadMismatch,
    /// replay window expired です。
    IdempotencyWindowExpired,
    /// command class が replay を許可しません。
    ReplayNotAllowed,
    /// response replay cache unavailable です。
    ResponseReplayNotAvailable,
    /// canonical payload digest を生成できません。
    CanonicalSerializationFailed,
}

impl IdempotencyFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::MissingCorrelationId => "missing_correlation_id",
            Self::CorrelationMismatch => "correlation_mismatch",
            Self::DuplicateCommand => "duplicate_command",
            Self::IdempotencyPayloadMismatch => "idempotency_payload_mismatch",
            Self::IdempotencyWindowExpired => "idempotency_window_expired",
            Self::ReplayNotAllowed => "replay_not_allowed",
            Self::ResponseReplayNotAvailable => "response_replay_not_available",
            Self::CanonicalSerializationFailed => "canonical_serialization_failed",
        }
    }
}

/// idempotency value の最小構文エラーです。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdempotencyValueError {
    /// 空 value は scope/key/digest identifier になりません。
    Empty,
    /// 制御文字を含む value は採用しません。
    ControlCharacter,
}

fn validate_idempotency_value(value: &str) -> Result<(), IdempotencyValueError> {
    if value.is_empty() {
        return Err(IdempotencyValueError::Empty);
    }
    if value.chars().any(char::is_control) {
        return Err(IdempotencyValueError::ControlCharacter);
    }
    Ok(())
}
