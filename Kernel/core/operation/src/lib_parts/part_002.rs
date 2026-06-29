impl AtomicCommitStep {
    /// commit boundary ごとの outcome を作ります。
    pub const fn new(
        boundary: CommitBoundary,
        outcome: AtomicStepOutcome,
        failure_reason: Option<AtomicityFailureKind>,
    ) -> Self {
        Self {
            boundary,
            outcome,
            failure_reason,
        }
    }
}

/// compensation の owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompensationOwner {
    /// domain state changes require core/state-machine owner.
    CoreWhenDomainStateChanges,
    /// external resource cleanup is driver-owned.
    DriverForExternalResourceCleanup,
}

/// compensation の external response rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompensationExternalResponseRule {
    /// original accepted decision is preserved and later failure is separate.
    PreserveOriginalAcceptedDecision,
    /// compensation failure is projected as a later failure.
    ProjectLaterFailure,
    /// no external response is emitted by compensation.
    NoExternalResponse,
}

/// compensation の evidence adoption rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompensationEvidenceAdoptionRule {
    /// compensation result is required before close evidence can be adopted.
    RequireCompensationResultBeforeCloseEvidence,
    /// later failure is driver observation or evidence limitation only.
    DriverObservationOrEvidenceLimitation,
    /// affected scope is close-not-claimed.
    CloseNotClaimed,
}

/// compensation status です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompensationStatus {
    /// compensation is not required.
    NotRequired,
    /// compensation rule exists and is pending.
    RequiredAndAvailable,
    /// compensation required but no rule exists.
    RequiredButUnavailable,
    /// compensation executed.
    Executed,
    /// compensation execution failed.
    ExecutionFailed,
}

/// compensation は rollback ではなく、明示 rule を持つ follow-up decision です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompensationRule {
    triggering_failure: AtomicityFailureKind,
    affected_state_or_event: &'static str,
    owner: CompensationOwner,
    allowed_compensating_transition: &'static str,
    audit_event_relation: &'static str,
    external_response_rule: CompensationExternalResponseRule,
    evidence_adoption_rule: CompensationEvidenceAdoptionRule,
}

impl CompensationRule {
    /// compensation rule を作ります。
    pub const fn new(
        triggering_failure: AtomicityFailureKind,
        affected_state_or_event: &'static str,
        owner: CompensationOwner,
        allowed_compensating_transition: &'static str,
        audit_event_relation: &'static str,
        external_response_rule: CompensationExternalResponseRule,
        evidence_adoption_rule: CompensationEvidenceAdoptionRule,
    ) -> Self {
        Self {
            triggering_failure,
            affected_state_or_event,
            owner,
            allowed_compensating_transition,
            audit_event_relation,
            external_response_rule,
            evidence_adoption_rule,
        }
    }
}

/// atomicity/compensation decision の core 形状です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtomicityCompensationDecision {
    correlation_id: CorrelationId,
    atomicity_class: AtomicityClass,
    current_boundary: CommitBoundary,
    step_outcome: AtomicCommitStep,
    compensation_status: CompensationStatus,
}

impl AtomicityCompensationDecision {
    /// atomicity_compensation_decision に投影できる decision を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        atomicity_class: AtomicityClass,
        current_boundary: CommitBoundary,
        step_outcome: AtomicCommitStep,
        compensation_status: CompensationStatus,
    ) -> Self {
        Self {
            correlation_id,
            atomicity_class,
            current_boundary,
            step_outcome,
            compensation_status,
        }
    }

    /// audit event type code です。
    pub const fn audit_event_type(&self) -> &'static str {
        "atomicity_compensation_decision"
    }
}

/// partial success evidence に必要な分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartialSuccessEvidenceClass {
    /// per-step outcome があり close evidence 候補にできます。
    ClassifiedPerStep,
    /// partial success classification がないため採用不可です。
    MissingClassificationNotAdoptable,
    /// failed step の scope を close-not-claimed にします。
    CloseNotClaimedForFailedStep,
}

/// atomicity 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedAtomicityBehavior {
    /// driver DB transaction defines domain invariant.
    DriverTransactionDefinesDomainInvariant,
    /// port intent is treated as driver execution success.
    PortIntentAsDriverExecutionSuccess,
    /// external response success is treated as audit persistence success.
    ExternalResponseAsAuditPersistenceSuccess,
    /// compensation mutates state without core/state-machine rule.
    CompensationWithoutCoreStateMachineRule,
    /// partial success is hidden behind final success.
    PartialSuccessHiddenBehindFinalSuccess,
    /// failed audit/persistence step is used as close evidence.
    FailedAuditPersistenceAsCloseEvidence,
}

/// concurrency / ordering / lock boundary concern の owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderingOwner {
    /// core owns domain ordering validity.
    Core,
    /// core/signaling owns room and participant scopes.
    CoreSignaling,
    /// core/sfu owns session and endpoint scopes.
    CoreSfu,
    /// core/turn owns allocation and permission scopes.
    CoreTurn,
    /// driver owns packet lifecycle physical resources.
    Driver,
    /// core/audit owns hash-chain ordering semantics.
    CoreAudit,
    /// core/entrypoints boundary owns configuration validation sequence.
    CoreEntrypointsBoundary,
    /// driver or core implementation detail owns physical lock.
    CoreOrDriverImplementationDetail,
    /// entrypoints owns process lock only.
    Entrypoints,
    /// testing scope is not production semantics.
    TestingScope,
}

/// concurrency / ordering / lock boundary concern です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderingConcern {
    /// domain ordering validity.
    DomainOrderingValidity,
    /// aggregate serialization scope.
    AggregateSerializationScope,
    /// physical mutex / channel / actor mailbox.
    PhysicalLockOrMailbox,
    /// runtime scheduling.
    RuntimeScheduling,
    /// runtime task lifecycle.
    RuntimeTaskLifecycle,
    /// inbound wire ordering observation.
    InboundWireOrderingObservation,
    /// idempotency.
    Idempotency,
}

impl OrderingConcern {
    /// concern owner です。
    pub const fn owner(self) -> OrderingOwner {
        match self {
            Self::DomainOrderingValidity
            | Self::AggregateSerializationScope
            | Self::Idempotency => OrderingOwner::Core,
            Self::PhysicalLockOrMailbox => OrderingOwner::CoreOrDriverImplementationDetail,
            Self::RuntimeScheduling
            | Self::RuntimeTaskLifecycle
            | Self::InboundWireOrderingObservation => OrderingOwner::Driver,
        }
    }
}

/// v0.2 initial architecture が認める serialization scope です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SerializationScope {
    /// room state, participant membership, command idempotency.
    RoomScope,
    /// participant lifecycle within room.
    ParticipantScope,
    /// session admission and route lifecycle.
    SfuSessionScope,
    /// endpoint publication/subscription lifecycle.
    SfuEndpointScope,
    /// packet buffer lease, queue, cache, release.
    PacketLifecycleScope,
    /// allocation lifecycle and refresh.
    TurnAllocationScope,
    /// permission and relay authorization.
    TurnPermissionScope,
    /// hash-chain ordering.
    AuditChainScope,
    /// startup/wiring validation sequence.
    ConfigurationScope,
}

impl SerializationScope {
    /// serialization scope owner です。
    pub const fn owner(self) -> OrderingOwner {
        match self {
            Self::RoomScope | Self::ParticipantScope => OrderingOwner::CoreSignaling,
            Self::SfuSessionScope | Self::SfuEndpointScope => OrderingOwner::CoreSfu,
            Self::PacketLifecycleScope => OrderingOwner::Driver,
            Self::TurnAllocationScope | Self::TurnPermissionScope => OrderingOwner::CoreTurn,
            Self::AuditChainScope => OrderingOwner::CoreAudit,
            Self::ConfigurationScope => OrderingOwner::CoreEntrypointsBoundary,
        }
    }

    /// protected semantics の説明です。
    pub const fn protected_semantics(self) -> &'static str {
        match self {
            Self::RoomScope => "room state, participant membership, command idempotency",
            Self::ParticipantScope => "participant lifecycle within room",
            Self::SfuSessionScope => "session admission and route lifecycle",
            Self::SfuEndpointScope => "endpoint publication/subscription lifecycle",
            Self::PacketLifecycleScope => "packet buffer lease, queue, cache, release",
            Self::TurnAllocationScope => "allocation lifecycle and refresh",
            Self::TurnPermissionScope => "permission and relay authorization",
            Self::AuditChainScope => "hash-chain ordering",
            Self::ConfigurationScope => "startup/wiring validation sequence",
        }
    }
}

/// serialization scope に対応する typed reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SerializationScopeReference {
    /// room scope reference.
    Room(RoomId),
    /// participant scope reference.
    Participant(ParticipantId),
    /// SFU session scope reference.
    SfuSession(SessionId),
    /// SFU endpoint scope reference.
    SfuEndpoint(EndpointId),
    /// packet lifecycle scope reference.
    Packet(PacketId),
    /// TURN allocation scope reference.
    TurnAllocation(AllocationId),
    /// TURN permission scope reference.
    TurnPermission(PermissionId),
    /// audit chain scope reference.
    AuditChain(AuditEventId),
    /// configuration scope reference.
    Configuration(ConfigurationScopeRef),
}

/// ordering / concurrency failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderingFailureKind {
    /// protocol command order failure.
    CommandOrderViolation,
    /// idempotency duplicate rejection.
    DuplicateCommand,
    /// accepted candidates conflict on same serialization scope.
    ConcurrencyConflict,
    /// bounded serialization queue/lock admission exhausted.
    LockContentionBoundExceeded,
}

impl OrderingFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::CommandOrderViolation => "command_order_violation",
            Self::DuplicateCommand => "duplicate_command",
            Self::ConcurrencyConflict => "concurrency_conflict",
            Self::LockContentionBoundExceeded => "lock_contention_bound_exceeded",
        }
    }
}

/// physical lock/resource の allowed owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalLockResource {
    /// aggregate mutation guard.
    AggregateMutationGuard,
    /// driver queue/mutex.
    DriverQueueMutex,
    /// entrypoint-level process lock.
    EntrypointLevelProcessLock,
    /// test harness synchronization.
    TestHarnessSynchronization,
}

impl PhysicalLockResource {
    /// allowed owner です。
    pub const fn allowed_owner(self) -> OrderingOwner {
        match self {
            Self::AggregateMutationGuard => OrderingOwner::CoreOrDriverImplementationDetail,
            Self::DriverQueueMutex => OrderingOwner::Driver,
            Self::EntrypointLevelProcessLock => OrderingOwner::Entrypoints,
            Self::TestHarnessSynchronization => OrderingOwner::TestingScope,
        }
    }
}

/// serialization queue / lock wait の bound policy shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SerializationBoundPolicy {
    scope: SerializationScope,
    maximum_pending_commands_required: bool,
    maximum_wait_duration_required: bool,
    owner: OrderingOwner,
    failure_reason: OrderingFailureKind,
    audit_event_type: &'static str,
    cancellation_behavior: &'static str,
}

impl SerializationBoundPolicy {
    /// unbounded growth を避けるための bound policy shape を作ります。
    pub const fn new(
        scope: SerializationScope,
        maximum_pending_commands_required: bool,
        maximum_wait_duration_required: bool,
        owner: OrderingOwner,
        cancellation_behavior: &'static str,
    ) -> Self {
        Self {
            scope,
            maximum_pending_commands_required,
            maximum_wait_duration_required,
            owner,
            failure_reason: OrderingFailureKind::LockContentionBoundExceeded,
            audit_event_type: "resource_bound_decision",
            cancellation_behavior,
        }
    }
}

/// concurrency / ordering decision の core 形状です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderingDecision {
    correlation_id: CorrelationId,
    scope: SerializationScope,
    scope_reference: Option<SerializationScopeReference>,
    outcome: AtomicStepOutcome,
    reason: Option<OrderingFailureKind>,
}

impl OrderingDecision {
    /// ordering/concurrency decision を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        scope: SerializationScope,
        scope_reference: Option<SerializationScopeReference>,
        outcome: AtomicStepOutcome,
        reason: Option<OrderingFailureKind>,
    ) -> Self {
        Self {
            correlation_id,
            scope,
            scope_reference,
            outcome,
            reason,
        }
    }
}

/// concurrency / lock 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedOrderingLockBehavior {
    /// driver lock acquisition order defines domain command order.
    DriverLockOrderDefinesDomainOrder,
    /// runtime scheduler order replaces state machine precondition.
    RuntimeSchedulerOrderAsStateMachineAuthority,
    /// unbounded aggregate command queue.
    UnboundedAggregateCommandQueue,
    /// lock object becomes domain model or public API.
    LockObjectInDomainOrPublicApi,
    /// race conflict is resolved by last-writer-wins without core decision.
    LastWriterWinsWithoutCoreDecision,
    /// test synchronization behavior is treated as production semantics.
    TestSynchronizationAsProductionSemantics,
    /// task scheduling order is treated as serialization authority.
    TaskSchedulingAsSerializationAuthority,
}

/// retry / timeout / cancellation concern の owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryTimeoutOwner {
    /// core reason catalog owns retryability metadata.
    CoreReasonCatalog,
    /// core owns idempotency and deadline policy.
    Core,
    /// driver owns bounded operation retry and timer execution.
    Driver,
    /// driver/sdk owns client cancellation observation.
    DriverSdk,
    /// entrypoints initiate shutdown cancellation.
    Entrypoints,
    /// testing scope is not runtime evidence.
    TestingScope,
}

/// retry / timeout / cancellation concern の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryTimeoutConcern {
    /// retryability metadata.
    RetryabilityMetadata,
    /// domain command idempotency.
    DomainCommandIdempotency,
    /// driver operation retry execution.
    DriverOperationRetryExecution,
    /// command deadline policy.
    CommandDeadlinePolicy,
    /// timer/scheduler execution.
    TimerSchedulerExecution,
    /// client cancellation observation.
    ClientCancellationObservation,
    /// entrypoint shutdown cancellation.
    EntrypointShutdownCancellation,
}

impl RetryTimeoutConcern {
    /// concern owner です。
    pub const fn owner(self) -> RetryTimeoutOwner {
        match self {
            Self::RetryabilityMetadata => RetryTimeoutOwner::CoreReasonCatalog,
            Self::DomainCommandIdempotency | Self::CommandDeadlinePolicy => RetryTimeoutOwner::Core,
            Self::DriverOperationRetryExecution | Self::TimerSchedulerExecution => {
                RetryTimeoutOwner::Driver
            }
            Self::ClientCancellationObservation => RetryTimeoutOwner::DriverSdk,
            Self::EntrypointShutdownCancellation => RetryTimeoutOwner::Entrypoints,
        }
    }
}

/// v0.2 initial architecture が認める retry class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryClass {
    /// non-idempotent or policy-rejected command must not retry automatically.
    NoRetry,
    /// same correlation/command identity and idempotency rule required.
    IdempotentCommandReplay,
    /// send/receive operation retry, bounded and audited when failure affects decision.
    DriverTransportRetry,
    /// bounded retry store governed by persistence/resource Canonical.
    PersistenceRetry,
    /// bounded sink retry, not a substitute for audit evidence.
    AuditSinkRetry,
    /// testing-only retry behavior.
    TestOnlyRetry,
}

impl RetryClass {
    /// retry class owner です。
    pub const fn owner(self) -> RetryTimeoutOwner {
        match self {
            Self::NoRetry | Self::IdempotentCommandReplay => RetryTimeoutOwner::Core,
            Self::DriverTransportRetry | Self::PersistenceRetry | Self::AuditSinkRetry => {
                RetryTimeoutOwner::Driver
            }
            Self::TestOnlyRetry => RetryTimeoutOwner::TestingScope,
        }
    }

    /// runtime evidence として採用できる retry class です。
    pub const fn runtime_evidence_allowed(self) -> bool {
        match self {
            Self::TestOnlyRetry => false,
            Self::NoRetry
            | Self::IdempotentCommandReplay
            | Self::DriverTransportRetry
            | Self::PersistenceRetry
            | Self::AuditSinkRetry => true,
        }
    }
}

