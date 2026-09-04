/// timeout/deadline surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeoutDeadlineSurface {
    /// command deadline.
    CommandDeadline,
    /// driver send timeout.
    DriverSendTimeout,
    /// driver receive timeout.
    DriverReceiveTimeout,
    /// persistence retry duration.
    PersistenceRetryDuration,
    /// packet/cache retention duration.
    PacketCacheRetentionDuration,
    /// runtime shutdown during scheduled action.
    RuntimeShutdownDuringScheduledAction,
}

/// retry / timeout / cancellation failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryTimeoutFailureKind {
    /// core command deadline exceeded.
    OperationDeadlineExceeded,
    /// operation cancelled after boundary.
    OperationCancelled,
    /// retry store bound exceeded.
    PersistenceRetryBoundExceeded,
    /// retry duration exceeded.
    PersistenceRetryDurationExceeded,
    /// packet/cache retention duration exceeded.
    RetentionDurationExceeded,
    /// network send retry exhausted.
    NetworkSendFailed,
    /// network receive retry exhausted.
    NetworkReceiveFailed,
    /// audit backlog bound reached.
    AuditBacklogBoundExceeded,
    /// runtime or driver shutdown.
    DriverShutdown,
    /// runtime task cancellation failed.
    RuntimeTaskCancelFailed,
    /// runtime task join/wait failed.
    RuntimeTaskJoinFailed,
    /// runtime task queue/mailbox/join bound exceeded.
    RuntimeTaskQueueBoundExceeded,
}

impl RetryTimeoutFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::OperationDeadlineExceeded => "operation_deadline_exceeded",
            Self::OperationCancelled => "operation_cancelled",
            Self::PersistenceRetryBoundExceeded => "persistence_retry_bound_exceeded",
            Self::PersistenceRetryDurationExceeded => "persistence_retry_duration_exceeded",
            Self::RetentionDurationExceeded => "retention_duration_exceeded",
            Self::NetworkSendFailed => "network_send_failed",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::AuditBacklogBoundExceeded => "audit_backlog_bound_exceeded",
            Self::DriverShutdown => "driver_shutdown",
            Self::RuntimeTaskCancelFailed => "runtime_task_cancel_failed",
            Self::RuntimeTaskJoinFailed => "runtime_task_join_failed",
            Self::RuntimeTaskQueueBoundExceeded => "runtime_task_queue_bound_exceeded",
        }
    }
}

impl TimeoutDeadlineSurface {
    /// owner です。
    pub const fn owner(self) -> RetryTimeoutOwner {
        match self {
            Self::CommandDeadline => RetryTimeoutOwner::Core,
            Self::DriverSendTimeout
            | Self::DriverReceiveTimeout
            | Self::PersistenceRetryDuration
            | Self::PacketCacheRetentionDuration
            | Self::RuntimeShutdownDuringScheduledAction => RetryTimeoutOwner::Driver,
        }
    }

    /// surface ごとの failure reason です。
    pub const fn failure_reason(self) -> RetryTimeoutFailureKind {
        match self {
            Self::CommandDeadline => RetryTimeoutFailureKind::OperationDeadlineExceeded,
            Self::DriverSendTimeout => RetryTimeoutFailureKind::NetworkSendFailed,
            Self::DriverReceiveTimeout => RetryTimeoutFailureKind::NetworkReceiveFailed,
            Self::PersistenceRetryDuration => {
                RetryTimeoutFailureKind::PersistenceRetryDurationExceeded
            }
            Self::PacketCacheRetentionDuration => {
                RetryTimeoutFailureKind::RetentionDurationExceeded
            }
            Self::RuntimeShutdownDuringScheduledAction => RetryTimeoutFailureKind::DriverShutdown,
        }
    }
}

/// cancellation source の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CancellationSource {
    /// client cancels before driver/core boundary.
    ClientBeforeDriverCoreBoundary,
    /// client cancels after command entered core.
    ClientAfterCoreEntry,
    /// entrypoint starts shutdown.
    EntrypointShutdown,
    /// runtime task cancelled.
    RuntimeTaskCancelled,
    /// test harness cancels.
    TestHarnessCancelled,
}

/// cancellation handling class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CancellationHandling {
    /// no domain state mutation.
    NoDomainStateMutation,
    /// preserve an already accepted/rejected decision result.
    PreservePriorDecisionResult,
    /// follow cross-plane shutdown drain canonical.
    FollowShutdownDrain,
    /// convert to operation_cancelled or driver_shutdown.
    ConvertToCatalogedRuntimeReason,
    /// classify as test timed-out/cancelled outcome.
    TestOutcomeOnly,
}

impl CancellationSource {
    /// cancellation handling です。
    pub const fn handling(self) -> CancellationHandling {
        match self {
            Self::ClientBeforeDriverCoreBoundary => CancellationHandling::NoDomainStateMutation,
            Self::ClientAfterCoreEntry => CancellationHandling::PreservePriorDecisionResult,
            Self::EntrypointShutdown => CancellationHandling::FollowShutdownDrain,
            Self::RuntimeTaskCancelled => CancellationHandling::ConvertToCatalogedRuntimeReason,
            Self::TestHarnessCancelled => CancellationHandling::TestOutcomeOnly,
        }
    }
}

/// retry を許可するための precondition set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RetryPreconditions {
    reason_retryable_or_policy_allowed: bool,
    command_idempotent_or_driver_local_before_domain_acceptance: bool,
    retry_bound_declared: bool,
    retry_duration_declared: bool,
    correlation_and_original_command_identity_preserved: bool,
    attempt_identity_distinguishes_first_and_retry: bool,
    privacy_redaction_boundary_preserved: bool,
}

impl RetryPreconditions {
    /// retry precondition set を作ります。
    pub const fn new(
        reason_retryable_or_policy_allowed: bool,
        command_idempotent_or_driver_local_before_domain_acceptance: bool,
        retry_bound_declared: bool,
        retry_duration_declared: bool,
        correlation_and_original_command_identity_preserved: bool,
        attempt_identity_distinguishes_first_and_retry: bool,
        privacy_redaction_boundary_preserved: bool,
    ) -> Self {
        Self {
            reason_retryable_or_policy_allowed,
            command_idempotent_or_driver_local_before_domain_acceptance,
            retry_bound_declared,
            retry_duration_declared,
            correlation_and_original_command_identity_preserved,
            attempt_identity_distinguishes_first_and_retry,
            privacy_redaction_boundary_preserved,
        }
    }

    /// retry を許可できるかどうかです。
    pub const fn allows_retry(self) -> bool {
        self.reason_retryable_or_policy_allowed
            && self.command_idempotent_or_driver_local_before_domain_acceptance
            && self.retry_bound_declared
            && self.retry_duration_declared
            && self.correlation_and_original_command_identity_preserved
            && self.attempt_identity_distinguishes_first_and_retry
            && self.privacy_redaction_boundary_preserved
    }
}

/// retry attempt の identity/observation shape です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryAttemptShape<TargetReference, ActorReference> {
    retry_class: RetryClass,
    correlation_id: CorrelationId,
    original_command_identity: Option<CommandIdentity<TargetReference, ActorReference>>,
    attempt_index: u64,
    preconditions: RetryPreconditions,
}

impl<TargetReference, ActorReference> RetryAttemptShape<TargetReference, ActorReference> {
    /// first attempt と retry attempt を区別できる shape を作ります。
    pub const fn new(
        retry_class: RetryClass,
        correlation_id: CorrelationId,
        original_command_identity: Option<CommandIdentity<TargetReference, ActorReference>>,
        attempt_index: u64,
        preconditions: RetryPreconditions,
    ) -> Self {
        Self {
            retry_class,
            correlation_id,
            original_command_identity,
            attempt_index,
            preconditions,
        }
    }
}

/// retry/timeout/cancellation 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedRetryTimeoutCancellationBehavior {
    /// automatic retry of non-idempotent domain command.
    AutomaticRetryOfNonIdempotentDomainCommand,
    /// driver retry changes domain decision.
    DriverRetryChangesDomainDecision,
    /// cancellation is treated as accepted domain lifecycle transition without core decision.
    CancellationAsAcceptedDomainTransition,
    /// timeout is reported as generic success or free-text failure.
    TimeoutAsGenericSuccessOrFreeTextFailure,
    /// unbounded retry loop or unbounded retry store.
    UnboundedRetry,
    /// test-only retry behavior is used as runtime policy.
    TestOnlyRetryAsRuntimePolicy,
    /// retry erases original correlation or first-attempt identity.
    RetryErasesOriginalCorrelationOrFirstAttemptIdentity,
    /// runtime task cancellation is treated as successful domain lifecycle transition.
    RuntimeTaskCancellationAsDomainSuccess,
}
