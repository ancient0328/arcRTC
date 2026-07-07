/// runtime task lifecycle outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskLifecycleOutcome {
    /// accepted.
    Accepted,
    /// rejected.
    Rejected,
    /// spawned.
    Spawned,
    /// joined.
    Joined,
    /// cancelled.
    Cancelled,
    /// failed.
    Failed,
    /// panic observed.
    PanicObserved,
    /// close-not-claimed.
    CloseNotClaimed,
}

/// RuntimePort が core へ返せる opaque task observation です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeTaskLifecycleDecision {
    startup_run_id: StartupRunId,
    correlation_id: Option<CorrelationId>,
    policy: RuntimeTaskLifecyclePolicy,
    materialized_task_reference: Option<&'static str>,
    outcome: RuntimeTaskLifecycleOutcome,
    reason: Option<RuntimeFailureKind>,
}

/// runtime task lifecycle decision の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskLifecycleDecisionError {
    /// non-success outcome requires cataloged reason.
    ReasonRequired,
    /// success outcome must not carry fake reason.
    ReasonMustBeAbsentForSuccess,
}

impl RuntimeTaskLifecycleDecision {
    /// runtime_task_lifecycle_decision に投影できる decision を作ります。
    pub fn try_new(
        startup_run_id: StartupRunId,
        correlation_id: Option<CorrelationId>,
        policy: RuntimeTaskLifecyclePolicy,
        materialized_task_reference: Option<&'static str>,
        outcome: RuntimeTaskLifecycleOutcome,
        reason: Option<RuntimeFailureKind>,
    ) -> Result<Self, RuntimeTaskLifecycleDecisionError> {
        match outcome {
            RuntimeTaskLifecycleOutcome::Accepted
            | RuntimeTaskLifecycleOutcome::Spawned
            | RuntimeTaskLifecycleOutcome::Joined
            | RuntimeTaskLifecycleOutcome::Cancelled
                if reason.is_some() =>
            {
                return Err(RuntimeTaskLifecycleDecisionError::ReasonMustBeAbsentForSuccess);
            }
            RuntimeTaskLifecycleOutcome::Rejected
            | RuntimeTaskLifecycleOutcome::Failed
            | RuntimeTaskLifecycleOutcome::PanicObserved
            | RuntimeTaskLifecycleOutcome::CloseNotClaimed
                if reason.is_none() =>
            {
                return Err(RuntimeTaskLifecycleDecisionError::ReasonRequired);
            }
            _ => {}
        }

        Ok(Self {
            startup_run_id,
            correlation_id,
            policy,
            materialized_task_reference,
            outcome,
            reason,
        })
    }

    /// audit event type code です。
    pub const fn audit_event_type(&self) -> &'static str {
        "runtime_task_lifecycle_decision"
    }

    /// lifecycle outcome です。
    pub const fn outcome(&self) -> RuntimeTaskLifecycleOutcome {
        self.outcome
    }

    /// rejected/failed reason です。
    pub const fn reason(&self) -> Option<RuntimeFailureKind> {
        self.reason
    }
}

/// task cancellation surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskCancellationSurface {
    /// before driver/core conversion.
    BeforeDriverCoreConversion,
    /// after command entered core.
    AfterCommandEnteredCore,
    /// during shutdown/drain.
    DuringShutdownDrain,
    /// during driver queue/cache execution.
    DuringDriverQueueCacheExecution,
    /// during test harness timeout.
    DuringTestHarnessTimeout,
}

impl TaskCancellationSurface {
    /// cancellation relation です。
    pub const fn required_relation(self) -> &'static str {
        match self {
            Self::BeforeDriverCoreConversion => "driver-local cancellation; no domain mutation",
            Self::AfterCommandEnteredCore => "prior core decision evidence remains authoritative",
            Self::DuringShutdownDrain => "follows shutdown drain canonical",
            Self::DuringDriverQueueCacheExecution => {
                "driver failure/shutdown/resource reason; domain decision is not rewritten"
            }
            Self::DuringTestHarnessTimeout => "testing evidence records timeout/cancel class",
        }
    }
}

/// task/worker lifecycle 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedRuntimeTaskWorkerBehavior {
    /// concrete runtime task handle appears in core public API or domain state.
    ConcreteRuntimeTaskHandleInCoreApi,
    /// detached task is spawned without admitted supervision scope.
    DetachedTaskWithoutAdmittedSupervision,
    /// worker completion is treated as domain decision.
    WorkerCompletionAsDomainDecision,
    /// task cancellation rewrites prior accepted/rejected domain decision.
    TaskCancellationRewritesPriorDecision,
    /// task panic is treated as graceful shutdown or recovery success.
    TaskPanicAsGracefulShutdownOrRecovery,
    /// unbounded task queue, mailbox, join wait, or restart loop is allowed.
    UnboundedTaskQueueMailboxJoinOrRestart,
    /// entrypoints supervisor restart is used as runtime readiness or domain restore evidence.
    SupervisorRestartAsReadinessOrRestoreEvidence,
    /// driver worker owns domain semantics.
    DriverWorkerOwnsDomainSemantics,
}
