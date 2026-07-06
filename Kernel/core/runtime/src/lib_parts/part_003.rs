/// runtime worker の bounded execution 条件です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeWorkerBound {
    queue_bound_declared: bool,
    join_wait_bound_declared: bool,
    memory_within_bound: bool,
}

impl RuntimeWorkerBound {
    /// worker queue / join-wait / memory bound の観測値を束ねます。
    pub const fn new(
        queue_bound_declared: bool,
        join_wait_bound_declared: bool,
        memory_within_bound: bool,
    ) -> Self {
        Self {
            queue_bound_declared,
            join_wait_bound_declared,
            memory_within_bound,
        }
    }
}

/// shutdown drain の runtime lifecycle 判定です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownDrainDecision {
    /// shutdown drain は不要です。
    NotRequired,
    /// runtime task は継続できます。
    Continue,
    /// shutdown drain により task は bounded cancel/join 対象です。
    Draining,
    /// shutdown drain は閉じた failure reason で拒否されました。
    Rejected(RuntimeFailureKind),
}

/// runtime backpressure の閉じた判定です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackpressureDecision {
    /// backpressure bound 内です。
    WithinBound,
    /// backpressure により task start は遅延扱いです。
    Delayed,
    /// backpressure により task lifecycle は拒否されました。
    Rejected(RuntimeFailureKind),
}

/// supervisor restart の観測です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupervisorRestartObservation {
    /// restart observation はありません。
    NotObserved,
    /// supervisor が restart を観測しました。
    RestartObserved,
    /// panic が観測されました。
    PanicObserved,
    /// supervision observation が失敗しました。
    Failed(RuntimeFailureKind),
}

/// runtime task lifecycle 判定入力です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeTaskLifecycleInput {
    startup_run_id: StartupRunId,
    correlation_id: Option<CorrelationId>,
    policy: RuntimeTaskLifecyclePolicy,
    materialized_task_reference: Option<&'static str>,
    worker_bound: RuntimeWorkerBound,
    shutdown_drain: ShutdownDrainDecision,
    backpressure: BackpressureDecision,
    supervisor_restart: SupervisorRestartObservation,
}

impl RuntimeTaskLifecycleInput {
    /// runtime task lifecycle の判定材料を束ねます。
    pub fn new(
        startup_run_id: StartupRunId,
        correlation_id: Option<CorrelationId>,
        policy: RuntimeTaskLifecyclePolicy,
        materialized_task_reference: Option<&'static str>,
        worker_bound: RuntimeWorkerBound,
        shutdown_drain: ShutdownDrainDecision,
        backpressure: BackpressureDecision,
        supervisor_restart: SupervisorRestartObservation,
    ) -> Self {
        Self {
            startup_run_id,
            correlation_id,
            policy,
            materialized_task_reference,
            worker_bound,
            shutdown_drain,
            backpressure,
            supervisor_restart,
        }
    }
}

/// runtime task lifecycle outcome を閉じた decision として返します。
///
/// runtime worker observation は domain state の所有者ではなく、task lifecycle の実行境界だけを表します。
pub fn decide_runtime_task_lifecycle(
    input: RuntimeTaskLifecycleInput,
) -> Result<RuntimeTaskLifecycleDecision, RuntimeTaskLifecycleDecisionError> {
    let failure = runtime_task_lifecycle_failure(&input);
    if let Some((outcome, reason)) = failure {
        return RuntimeTaskLifecycleDecision::try_new(
            input.startup_run_id,
            input.correlation_id,
            input.policy,
            input.materialized_task_reference,
            outcome,
            Some(reason),
        );
    }

    let outcome = match input.shutdown_drain {
        ShutdownDrainDecision::Draining => RuntimeTaskLifecycleOutcome::Cancelled,
        ShutdownDrainDecision::NotRequired | ShutdownDrainDecision::Continue => {
            if input.materialized_task_reference.is_some() {
                RuntimeTaskLifecycleOutcome::Spawned
            } else {
                RuntimeTaskLifecycleOutcome::Accepted
            }
        }
        ShutdownDrainDecision::Rejected(_) => unreachable!("rejected shutdown is handled above"),
    };

    RuntimeTaskLifecycleDecision::try_new(
        input.startup_run_id,
        input.correlation_id,
        input.policy,
        input.materialized_task_reference,
        outcome,
        None,
    )
}

fn runtime_task_lifecycle_failure(
    input: &RuntimeTaskLifecycleInput,
) -> Option<(RuntimeTaskLifecycleOutcome, RuntimeFailureKind)> {
    if input.policy.task_class.is_rejected_class() {
        return Some((
            RuntimeTaskLifecycleOutcome::Rejected,
            RuntimeFailureKind::RuntimeTaskClassNotAdmitted,
        ));
    }

    if !matches!(input.policy.task_class, RuntimeTaskClass::NoRuntimeTask)
        && !input.worker_bound.join_wait_bound_declared
    {
        return Some((
            RuntimeTaskLifecycleOutcome::Rejected,
            RuntimeFailureKind::RuntimeTaskSupervisionMissing,
        ));
    }

    if matches!(
        input.policy.task_class,
        RuntimeTaskClass::DriverIoWorker
            | RuntimeTaskClass::DriverPacketWorker
            | RuntimeTaskClass::DriverSinkWorker
    ) && !input.worker_bound.queue_bound_declared
    {
        return Some((
            RuntimeTaskLifecycleOutcome::Rejected,
            RuntimeFailureKind::RuntimeTaskQueueBoundExceeded,
        ));
    }

    if !input.worker_bound.memory_within_bound {
        return Some((
            RuntimeTaskLifecycleOutcome::Rejected,
            RuntimeFailureKind::MemoryPressureExceeded,
        ));
    }

    match input.shutdown_drain {
        ShutdownDrainDecision::Rejected(reason) => {
            return Some((RuntimeTaskLifecycleOutcome::Rejected, reason));
        }
        ShutdownDrainDecision::NotRequired
        | ShutdownDrainDecision::Continue
        | ShutdownDrainDecision::Draining => {}
    }

    match input.backpressure {
        BackpressureDecision::WithinBound => {}
        BackpressureDecision::Delayed => {
            return Some((
                RuntimeTaskLifecycleOutcome::Rejected,
                RuntimeFailureKind::RuntimeTaskQueueBoundExceeded,
            ));
        }
        BackpressureDecision::Rejected(reason) => {
            return Some((RuntimeTaskLifecycleOutcome::Rejected, reason));
        }
    }

    match input.supervisor_restart {
        SupervisorRestartObservation::NotObserved | SupervisorRestartObservation::RestartObserved => {
            None
        }
        SupervisorRestartObservation::PanicObserved => Some((
            RuntimeTaskLifecycleOutcome::PanicObserved,
            RuntimeFailureKind::RuntimeTaskPanicDetected,
        )),
        SupervisorRestartObservation::Failed(reason) => {
            Some((RuntimeTaskLifecycleOutcome::Failed, reason))
        }
    }
}
