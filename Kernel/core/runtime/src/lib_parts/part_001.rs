// core/runtime は ClockPort、RandomPort、RuntimePort の core-owned abstraction surface です。
//
// ここでは tokio/browser/native runtime、OS RNG、system clock の実装を持ちません。
// driver が observation/execution を提供し、core は意味論と failure mapping だけを所有します。

use arcrtc_core_identity::{ConfigurationScopeRef, CorrelationId, StartupRunId};

/// core runtime package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreRuntimeSurface;

/// runtime abstraction surface の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeAbstractionSurface {
    /// current time abstraction, monotonic comparison input, deadline semantics.
    ClockPort,
    /// nonce / opaque ID / challenge entropy contract.
    RandomPort,
    /// timer / spawn / cancellation contract.
    RuntimePort,
}

/// core と driver の ownership tuple です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeOwnership {
    core_owns: &'static str,
    driver_owns: &'static str,
}

impl RuntimeOwnership {
    /// ownership tuple を作ります。
    pub const fn new(core_owns: &'static str, driver_owns: &'static str) -> Self {
        Self {
            core_owns,
            driver_owns,
        }
    }
}

impl RuntimeAbstractionSurface {
    /// Canonical の ownership tuple です。
    pub const fn ownership(self) -> RuntimeOwnership {
        match self {
            Self::ClockPort => RuntimeOwnership::new(
                "current time abstraction, monotonic comparison input, deadline semantics",
                "system clock / test clock implementation",
            ),
            Self::RandomPort => RuntimeOwnership::new(
                "nonce / opaque ID / challenge entropy contract",
                "OS RNG / deterministic test RNG implementation",
            ),
            Self::RuntimePort => RuntimeOwnership::new(
                "timer / spawn / cancellation contract",
                "tokio or other runtime execution",
            ),
        }
    }
}

/// ClockPort が提供する observation class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockObservationClass {
    /// current time abstraction.
    CurrentTime,
    /// monotonic comparison input.
    MonotonicComparisonInput,
    /// deadline comparison input.
    DeadlineComparisonInput,
    /// expiry comparison input.
    ExpiryComparisonInput,
}

/// core policy が time observation を使って評価する expiry/deadline target です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimePolicyTarget {
    /// TURN allocation expiry.
    TurnAllocationExpiry,
    /// TURN permission expiry.
    TurnPermissionExpiry,
    /// TURN channel bind expiry.
    TurnChannelBindExpiry,
    /// Signaling room lifecycle.
    SignalingRoomLifecycle,
    /// resource retention.
    ResourceRetention,
    /// configuration startup timeout.
    ConfigurationStartupTimeout,
    /// command deadline.
    CommandDeadline,
}

impl TimePolicyTarget {
    /// target ごとの cataloged reason です。
    pub const fn failure_reason(self) -> &'static str {
        match self {
            Self::TurnAllocationExpiry => "allocation_lifetime_exceeded",
            Self::TurnPermissionExpiry => "permission_lifetime_exceeded",
            Self::TurnChannelBindExpiry => "channel_bind_lifetime_exceeded",
            Self::SignalingRoomLifecycle => "room_lifetime_exceeded",
            Self::ResourceRetention => "retention_duration_exceeded",
            Self::ConfigurationStartupTimeout => "runtime_config_invalid",
            Self::CommandDeadline => "operation_deadline_exceeded",
        }
    }
}

/// RandomPort output の core-owned use class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RandomnessUseClass {
    /// nonce.
    Nonce,
    /// opaque ID.
    OpaqueId,
    /// challenge entropy.
    Challenge,
    /// reference stability material.
    ReferenceStability,
}

/// RandomPort output contract です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RandomnessContract {
    use_class: RandomnessUseClass,
    driver_supplies_entropy: bool,
    core_owns_identity_meaning: bool,
    external_identity_encoding_allowed: bool,
}

impl RandomnessContract {
    /// randomness output を opaque input として扱う contract を作ります。
    pub const fn opaque(use_class: RandomnessUseClass) -> Self {
        Self {
            use_class,
            driver_supplies_entropy: true,
            core_owns_identity_meaning: true,
            external_identity_encoding_allowed: false,
        }
    }
}

/// RuntimePort operation class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeOperationClass {
    /// timer scheduling.
    Timer,
    /// task spawn scheduling.
    Spawn,
    /// cancellation request.
    Cancellation,
    /// shutdown observation.
    ShutdownObservation,
}

/// RuntimePort が返してよい opaque reference / observation class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeOutputClass {
    /// concrete runtime handle ではない schedule reference.
    OpaqueScheduleReference,
    /// concrete task handle ではない task reference.
    OpaqueTaskReference,
    /// cancellation observation.
    CancellationObservation,
    /// shutdown observation.
    ShutdownObservation,
}

/// runtime/clock/randomness failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeFailureKind {
    /// required runtime configuration missing.
    RuntimeConfigMissing,
    /// runtime cannot initialize selected driver/entrypoint.
    RuntimeConfigInvalid,
    /// runtime task class is not admitted.
    RuntimeTaskClassNotAdmitted,
    /// runtime task owner/supervision scope is invalid.
    RuntimeTaskOwnerViolation,
    /// task has no admitted supervision scope.
    RuntimeTaskSupervisionMissing,
    /// runtime cannot spawn required task.
    RuntimeTaskSpawnFailed,
    /// task join/wait observation failed.
    RuntimeTaskJoinFailed,
    /// task cancellation failed or could not be observed.
    RuntimeTaskCancelFailed,
    /// task panic was observed.
    RuntimeTaskPanicDetected,
    /// driver runtime is shutting down.
    DriverShutdown,
    /// timer/queue resource bound exceeded.
    RuntimeTaskQueueBoundExceeded,
    /// memory pressure bound exceeded.
    MemoryPressureExceeded,
}

impl RuntimeFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::RuntimeTaskClassNotAdmitted => "runtime_task_class_not_admitted",
            Self::RuntimeTaskOwnerViolation => "runtime_task_owner_violation",
            Self::RuntimeTaskSupervisionMissing => "runtime_task_supervision_missing",
            Self::RuntimeTaskSpawnFailed => "runtime_task_spawn_failed",
            Self::RuntimeTaskJoinFailed => "runtime_task_join_failed",
            Self::RuntimeTaskCancelFailed => "runtime_task_cancel_failed",
            Self::RuntimeTaskPanicDetected => "runtime_task_panic_detected",
            Self::DriverShutdown => "driver_shutdown",
            Self::RuntimeTaskQueueBoundExceeded => "runtime_task_queue_bound_exceeded",
            Self::MemoryPressureExceeded => "memory_pressure_exceeded",
        }
    }
}

/// runtime configuration selection の core-facing shape です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeConfigurationSelection {
    startup_run_id: StartupRunId,
    configuration_scope: ConfigurationScopeRef,
    selected_surface: RuntimeAbstractionSurface,
}

impl RuntimeConfigurationSelection {
    /// entrypoints wiring から core validation へ渡す runtime configuration selection です。
    pub const fn new(
        startup_run_id: StartupRunId,
        configuration_scope: ConfigurationScopeRef,
        selected_surface: RuntimeAbstractionSurface,
    ) -> Self {
        Self {
            startup_run_id,
            configuration_scope,
            selected_surface,
        }
    }
}

/// runtime/clock/randomness 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedRuntimeClockRandomnessBehavior {
    /// core imports concrete runtime handle.
    CoreImportsConcreteRuntimeHandle,
    /// driver timer implementation defines domain expiry.
    DriverTimerDefinesDomainExpiry,
    /// driver compares raw platform time/measurement without normalized unit policy.
    RawPlatformTimeWithoutNormalization,
    /// random generator implementation owns identity semantics.
    RandomGeneratorOwnsIdentitySemantics,
    /// entrypoints silently substitute defaults after required configuration missing.
    EntrypointsSilentlySubstituteRuntimeDefaults,
    /// deterministic test clock/RNG is used as production-readiness evidence.
    DeterministicTestClockRngAsProductionEvidence,
    /// runtime worker state owns domain state.
    RuntimeWorkerOwnsDomainState,
    /// detached task or worker supervision is implicit.
    ImplicitDetachedTaskOrSupervision,
    /// wall-clock timestamp is cross-node causal order without skew trust.
    WallClockAsCrossNodeCausalOrderWithoutTrust,
}

/// runtime task / worker lifecycle concern の owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskOwner {
    /// core owns domain state transition semantics.
    Core,
    /// core owns RuntimePort task contract.
    CoreRuntimePortContract,
    /// driver/entrypoints runtime owns concrete task/join handles.
    DriverEntrypointsRuntime,
    /// driver owns I/O and sink workers.
    Driver,
    /// entrypoints owns process/component supervision observation.
    Entrypoints,
    /// physical execution owner owns queue/mailbox bound.
    PhysicalExecutionOwner,
    /// testing scope is not production semantics.
    TestingScope,
}

/// runtime task / worker lifecycle concern です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskConcern {
    /// domain state transition.
    DomainStateTransition,
    /// RuntimePort task contract.
    RuntimePortTaskContract,
    /// concrete task handle / join handle.
    ConcreteTaskHandle,
    /// driver I/O worker.
    DriverIoWorker,
    /// entrypoints supervision task.
    EntrypointsSupervisionTask,
    /// task queue / mailbox.
    TaskQueueMailbox,
    /// task panic observation.
    TaskPanicObservation,
    /// task cancellation.
    TaskCancellation,
}

impl RuntimeTaskConcern {
    /// concern owner です。
    pub const fn owner(self) -> RuntimeTaskOwner {
        match self {
            Self::DomainStateTransition => RuntimeTaskOwner::Core,
            Self::RuntimePortTaskContract => RuntimeTaskOwner::CoreRuntimePortContract,
            Self::ConcreteTaskHandle => RuntimeTaskOwner::DriverEntrypointsRuntime,
            Self::DriverIoWorker => RuntimeTaskOwner::Driver,
            Self::EntrypointsSupervisionTask => RuntimeTaskOwner::Entrypoints,
            Self::TaskQueueMailbox => RuntimeTaskOwner::PhysicalExecutionOwner,
            Self::TaskPanicObservation | Self::TaskCancellation => {
                RuntimeTaskOwner::DriverEntrypointsRuntime
            }
        }
    }
}

/// v0.2 initial architecture が認める task class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskClass {
    /// target path does not require runtime task.
    NoRuntimeTask,
    /// network/socket/protocol driver worker.
    DriverIoWorker,
    /// packet queue/cache/rewrite/forward execution worker.
    DriverPacketWorker,
    /// audit/metrics/persistence/export sink worker.
    DriverSinkWorker,
    /// entrypoints-level process/component supervisor task.
    EntrypointsSupervisionTask,
    /// timer/deadline callback execution.
    RuntimeTimerTask,
    /// deterministic or fake task execution for tests.
    TestRuntimeTask,
    /// task has no admitted parent/supervision scope.
    DetachedTaskRequested,
}

impl RuntimeTaskClass {
    /// detached task requested は v0.2 initial architecture では拒否されます。
    pub const fn is_rejected_class(self) -> bool {
        match self {
            Self::DetachedTaskRequested => true,
            Self::NoRuntimeTask
            | Self::DriverIoWorker
            | Self::DriverPacketWorker
            | Self::DriverSinkWorker
            | Self::EntrypointsSupervisionTask
            | Self::RuntimeTimerTask
            | Self::TestRuntimeTask => false,
        }
    }
}

/// admitted supervision scope です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupervisionScope {
    /// entrypoint startup/run supervision.
    EntrypointStartupRun,
    /// driver component supervision.
    DriverComponent,
    /// bounded test harness supervision.
    BoundedTestHarness,
    /// explicit RuntimePort schedule/cancel scope.
    RuntimePortScheduleCancelScope,
}

/// runtime task の owning layer です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskOwningLayer {
    /// driver-owned task.
    Driver,
    /// entrypoints-owned task.
    Entrypoints,
    /// testing-owned task.
    Testing,
    /// core observes through RuntimePort but does not own concrete task.
    CoreObservedRuntimePort,
}

/// task input reference type class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskInputReferenceClass {
    /// no input reference.
    None,
    /// opaque schedule reference.
    OpaqueScheduleReference,
    /// opaque task reference.
    OpaqueTaskReference,
    /// command-scoped correlation reference.
    CommandCorrelation,
}

/// task output observation class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskOutputObservation {
    /// no task evidence is claimed.
    NoTaskEvidenceClaim,
    /// spawn accepted observation.
    SpawnObserved,
    /// join/wait observation.
    JoinObserved,
    /// cancellation observation.
    CancellationObserved,
    /// panic observation.
    PanicObserved,
    /// failure observation with cataloged reason.
    FailureObserved,
}

/// cancellation propagation rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CancellationPropagationRule {
    /// no cancellation propagation.
    NotApplicable,
    /// parent scope ending requires bounded join or cancel.
    ParentScopeEndsThenBoundedJoinOrCancel,
    /// command-scoped cancellation preserves prior core decision.
    PreservePriorCoreDecision,
    /// shutdown/drain relation follows shutdown drain Canonical.
    FollowsShutdownDrain,
}

/// runtime task lifecycle policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeTaskLifecyclePolicy {
    task_class: RuntimeTaskClass,
    parent_scope: Option<SupervisionScope>,
    owning_layer: RuntimeTaskOwningLayer,
    input_reference_class: RuntimeTaskInputReferenceClass,
    output_observation: RuntimeTaskOutputObservation,
    cancellation_propagation: CancellationPropagationRule,
    join_wait_bound_required: bool,
    queue_mailbox_bound_required: bool,
    panic_failure_mapping: RuntimeFailureKind,
    audit_event_type: &'static str,
}

/// runtime task lifecycle policy の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeTaskLifecyclePolicyError {
    /// detached task is not admitted.
    DetachedTaskNotAdmitted,
    /// admitted task requires parent/supervision scope.
    SupervisionScopeMissing,
    /// admitted task requires bounded join/wait.
    JoinWaitBoundMissing,
    /// worker queue/mailbox bound is required for this class.
    QueueMailboxBoundMissing,
}

impl RuntimeTaskLifecyclePolicy {
    /// task/worker lifecycle に必要な field を持つ policy を作ります。
    pub fn try_new(
        task_class: RuntimeTaskClass,
        parent_scope: Option<SupervisionScope>,
        owning_layer: RuntimeTaskOwningLayer,
        input_reference_class: RuntimeTaskInputReferenceClass,
        output_observation: RuntimeTaskOutputObservation,
        cancellation_propagation: CancellationPropagationRule,
        join_wait_bound_required: bool,
        queue_mailbox_bound_required: bool,
    ) -> Result<Self, RuntimeTaskLifecyclePolicyError> {
        if matches!(task_class, RuntimeTaskClass::DetachedTaskRequested) {
            return Err(RuntimeTaskLifecyclePolicyError::DetachedTaskNotAdmitted);
        }

        if !matches!(task_class, RuntimeTaskClass::NoRuntimeTask) {
            if parent_scope.is_none() {
                return Err(RuntimeTaskLifecyclePolicyError::SupervisionScopeMissing);
            }
            if !join_wait_bound_required {
                return Err(RuntimeTaskLifecyclePolicyError::JoinWaitBoundMissing);
            }
        }

        if matches!(
            task_class,
            RuntimeTaskClass::DriverIoWorker
                | RuntimeTaskClass::DriverPacketWorker
                | RuntimeTaskClass::DriverSinkWorker
        ) && !queue_mailbox_bound_required
        {
            return Err(RuntimeTaskLifecyclePolicyError::QueueMailboxBoundMissing);
        }

        Ok(Self {
            task_class,
            parent_scope,
            owning_layer,
            input_reference_class,
            output_observation,
            cancellation_propagation,
            join_wait_bound_required,
            queue_mailbox_bound_required,
            panic_failure_mapping: RuntimeFailureKind::RuntimeTaskPanicDetected,
            audit_event_type: "runtime_task_lifecycle_decision",
        })
    }
}

