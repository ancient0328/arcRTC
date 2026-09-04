// core/command は command、decision、event、response の core surface です。
//
// external request/response 型はここへ入れず、driver が解釈できる自由形式 result も返しません。

use arcrtc_core_identity::{AuditEventId, CorrelationId};

/// core command package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreCommandSurface;

/// v0.2 initial architecture で許可された result shape class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResultShapeClass {
    /// driver conversion 後の core-owned command envelope です。
    CommandEnvelope,
    /// command に対する authoritative outcome です。
    UseCaseDecision,
    /// state transition または decision fact です。
    DomainEvent,
    /// driver execution request です。I/O 成功を意味しません。
    PortIntent,
    /// concrete execution result から変換された core-owned observation です。
    DriverObservation,
    /// audit event model への projection です。
    AuditProjection,
    /// external surface へ返す projection model です。
    ExternalResponseModel,
}

/// command の意味名です。wire 名や HTTP path は driver 側に置きます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandType(&'static str);

impl CommandType {
    /// core-owned command type 名を作ります。
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// command type 名です。
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

/// command version です。protocol versioning の詳細判断は core/protocol が所有します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandVersion(u32);

impl CommandVersion {
    /// command shape 内で扱う version value です。
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// raw version number です。
    pub const fn value(&self) -> u32 {
        self.0
    }
}

/// command/decision が対象にする core surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetSurface {
    /// Signaling core surface です。
    Signaling,
    /// SFU core surface です。
    Sfu,
    /// TURN core surface です。
    Turn,
    /// Transport core surface です。
    Transport,
    /// Security core surface です。
    Security,
    /// Audit core surface です。
    Audit,
    /// Quality core surface です。
    Quality,
    /// Port boundary surface です。
    Ports,
    /// Configuration core surface です。
    Configuration,
    /// Feature admission core surface です。
    Features,
}

/// driver conversion 後に core use case が受け取る command envelope です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEnvelope<SubjectReferences> {
    correlation_id: CorrelationId,
    command_type: CommandType,
    version: CommandVersion,
    target_surface: TargetSurface,
    subject_references: SubjectReferences,
}

impl<SubjectReferences> CommandEnvelope<SubjectReferences> {
    /// core-owned command envelope を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        command_type: CommandType,
        version: CommandVersion,
        target_surface: TargetSurface,
        subject_references: SubjectReferences,
    ) -> Self {
        Self {
            correlation_id,
            command_type,
            version,
            target_surface,
            subject_references,
        }
    }

    /// command/event chain の correlation ID です。
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    /// command type です。
    pub const fn command_type(&self) -> CommandType {
        self.command_type
    }

    /// command version です。
    pub const fn version(&self) -> CommandVersion {
        self.version
    }

    /// target surface です。
    pub const fn target_surface(&self) -> TargetSurface {
        self.target_surface
    }

    /// command subject references です。
    pub const fn subject_references(&self) -> &SubjectReferences {
        &self.subject_references
    }
}

/// audit-compatible authoritative outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UseCaseOutcome {
    /// accepted outcome です。
    Accepted,
    /// allowed outcome です。
    Allowed,
    /// forwarded outcome です。
    Forwarded,
    /// selected outcome です。
    Selected,
    /// released outcome です。
    Released,
    /// closed_success outcome です。
    ClosedSuccess,
    /// idempotent_observed outcome です。
    IdempotentObserved,
    /// within_bound_observed outcome です。
    WithinBoundObserved,
    /// rejected outcome です。
    Rejected,
    /// denied outcome です。
    Denied,
    /// protocol_violation outcome です。
    ProtocolViolation,
    /// suppressed outcome です。
    Suppressed,
    /// dropped outcome です。
    Dropped,
    /// expired outcome です。
    Expired,
    /// revoked outcome です。
    Revoked,
    /// shed outcome です。
    Shed,
    /// delayed outcome です。
    Delayed,
    /// degraded outcome です。
    Degraded,
    /// failed outcome です。
    Failed,
    /// converted_failure outcome です。
    ConvertedFailure,
    /// closed_by_policy outcome です。
    ClosedByPolicy,
}

impl UseCaseOutcome {
    /// reason を持たない success observation かどうかです。
    pub const fn is_success(self) -> bool {
        matches!(
            self,
            Self::Accepted
                | Self::Allowed
                | Self::Forwarded
                | Self::Selected
                | Self::Released
                | Self::ClosedSuccess
                | Self::IdempotentObserved
                | Self::WithinBoundObserved
        )
    }

    /// cataloged reason が必須かどうかです。
    pub const fn requires_reason(self) -> bool {
        !self.is_success()
    }
}

/// decision に添付する reason slot です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionReason<Reason> {
    /// success outcome にだけ許可される reason 不在です。
    Absent,
    /// non-success outcome に必須の cataloged reason です。
    Cataloged(Reason),
}

/// state transition の有無を表す summary です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateTransitionSummary {
    /// state transition はありません。
    NoStateChange,
    /// state transition があり、詳細は各 state machine source が所有します。
    Changed(&'static str),
}

/// driver に依頼する execution intent です。実行成功は表しません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortIntent {
    intent_type: &'static str,
    target_surface: TargetSurface,
}

impl PortIntent {
    /// port intent の意味名と target surface を固定します。
    pub const fn new(intent_type: &'static str, target_surface: TargetSurface) -> Self {
        Self {
            intent_type,
            target_surface,
        }
    }

    /// intent type 名です。
    pub const fn intent_type(&self) -> &'static str {
        self.intent_type
    }

    /// port intent の target surface です。
    pub const fn target_surface(&self) -> TargetSurface {
        self.target_surface
    }
}

/// audit projection が必要かどうかを示します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditProjectionRequirement {
    /// audit projection が必要です。
    Required,
    /// audit projection は不要です。
    NotRequired,
}

/// command に対する authoritative decision です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseCaseDecision<Reason> {
    correlation_id: CorrelationId,
    command_type: CommandType,
    target_surface: TargetSurface,
    outcome: UseCaseOutcome,
    reason: DecisionReason<Reason>,
    state_transition: StateTransitionSummary,
    port_intents: Vec<PortIntent>,
    audit_projection: AuditProjectionRequirement,
}

/// use case decision 生成時の未検査入力です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseCaseDecisionInput<Reason> {
    pub correlation_id: CorrelationId,
    pub command_type: CommandType,
    pub target_surface: TargetSurface,
    pub outcome: UseCaseOutcome,
    pub reason: DecisionReason<Reason>,
    pub state_transition: StateTransitionSummary,
    pub port_intents: Vec<PortIntent>,
    pub audit_projection: AuditProjectionRequirement,
}

impl<Reason> UseCaseDecision<Reason> {
    /// use case decision を reason rule に従って生成します。
    pub fn new(input: UseCaseDecisionInput<Reason>) -> Result<Self, DecisionShapeError> {
        let UseCaseDecisionInput {
            correlation_id,
            command_type,
            target_surface,
            outcome,
            reason,
            state_transition,
            port_intents,
            audit_projection,
        } = input;

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
            correlation_id,
            command_type,
            target_surface,
            outcome,
            reason,
            state_transition,
            port_intents,
            audit_projection,
        })
    }

    /// command/event chain の correlation ID です。
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    /// command type です。
    pub const fn command_type(&self) -> CommandType {
        self.command_type
    }

    /// target surface です。
    pub const fn target_surface(&self) -> TargetSurface {
        self.target_surface
    }

    /// authoritative outcome です。
    pub const fn outcome(&self) -> UseCaseOutcome {
        self.outcome
    }

    /// decision reason slot です。
    pub const fn reason(&self) -> &DecisionReason<Reason> {
        &self.reason
    }

    /// state transition summary です。
    pub const fn state_transition(&self) -> StateTransitionSummary {
        self.state_transition
    }

    /// driver execution request の一覧です。
    pub const fn port_intents(&self) -> &Vec<PortIntent> {
        &self.port_intents
    }

    /// audit projection requirement です。
    pub const fn audit_projection(&self) -> AuditProjectionRequirement {
        self.audit_projection
    }

}

/// decision shape rule 違反です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecisionShapeError {
    /// success outcome が fake reason を持っています。
    SuccessMustNotCarryReason,
    /// non-success outcome に cataloged reason がありません。
    NonSuccessRequiresReason,
}

/// state transition または decision fact を表す domain event です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainEvent<Payload> {
    correlation_id: CorrelationId,
    event_type: &'static str,
    payload: Payload,
}

impl<Payload> DomainEvent<Payload> {
    /// domain event を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        event_type: &'static str,
        payload: Payload,
    ) -> Self {
        Self {
            correlation_id,
            event_type,
            payload,
        }
    }

    /// command/event chain の correlation ID です。
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    /// event type 名です。
    pub const fn event_type(&self) -> &'static str {
        self.event_type
    }

    /// event payload です。
    pub const fn payload(&self) -> &Payload {
        &self.payload
    }
}

/// driver execution result を core-owned observation に変換した形です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverObservation<Reason> {
    correlation_id: CorrelationId,
    outcome: UseCaseOutcome,
    reason: DecisionReason<Reason>,
}

impl<Reason> DriverObservation<Reason> {
    /// driver observation を decision と同じ reason rule で作ります。
    pub fn new(
        correlation_id: CorrelationId,
        outcome: UseCaseOutcome,
        reason: DecisionReason<Reason>,
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
            correlation_id,
            outcome,
            reason,
        })
    }
}

/// audit event model へ渡す closed projection です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditProjection<Reason> {
    correlation_id: CorrelationId,
    event_type: &'static str,
    outcome: UseCaseOutcome,
    reason: DecisionReason<Reason>,
}

impl<Reason> AuditProjection<Reason> {
    /// audit projection を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        event_type: &'static str,
        outcome: UseCaseOutcome,
        reason: DecisionReason<Reason>,
    ) -> Self {
        Self {
            correlation_id,
            event_type,
            outcome,
            reason,
        }
    }
}

/// external response は authoritative decision の projection であり、外部 status ではありません。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalResponseModel<Reason> {
    correlation_id: CorrelationId,
    outcome: UseCaseOutcome,
    reason: DecisionReason<Reason>,
}

impl<Reason> ExternalResponseModel<Reason> {
    /// external response projection model を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        outcome: UseCaseOutcome,
        reason: DecisionReason<Reason>,
    ) -> Self {
        Self {
            correlation_id,
            outcome,
            reason,
        }
    }
}

/// correlation tracing は duplicate 判定そのものではないことを明示する trace 型です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationTrace {
    correlation_id: CorrelationId,
}
