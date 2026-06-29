// core/operation は cross-plane operation semantics の core surface です。
//
// process runtime や concrete I/O は drivers/entrypoints が扱い、
// ここでは shutdown/drain などの順序、owner、reason 接続だけを定義します。

use arcrtc_core_command::CommandIdentity;
use arcrtc_core_identity::{
    AllocationId, AuditEventId, ConfigurationScopeRef, CorrelationId, EndpointId, PacketId,
    ParticipantId, PermissionId, RoomId, SessionId, StartupRunId,
};

/// core operation package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreOperationSurface;

/// shutdown/drain concern の owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownDrainOwner {
    /// entrypoints が process signal や mode selection を扱います。
    Entrypoints,
    /// internal control-plane contract が split-service command/event boundary を扱います。
    InternalControlPlaneContract,
    /// core/signaling が room drain/close semantics を扱います。
    CoreSignaling,
    /// core/sfu が session/endpoint lifecycle semantics を扱います。
    CoreSfu,
    /// core/turn が allocation/permission relay semantics を扱います。
    CoreTurn,
    /// driver が concrete I/O や resource release を扱います。
    Driver,
    /// driver/entrypoints runtime が bounded task/worker stop を扱います。
    DriverEntrypointsRuntime,
    /// entrypoints/driver observation が unclean termination を観測します。
    EntrypointsDriverObservation,
}

/// shutdown/drain boundary concern の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownDrainConcern {
    /// process signal observation.
    ProcessSignalObservation,
    /// split-service shutdown control.
    SplitServiceShutdownControl,
    /// shutdown mode selection.
    ShutdownModeSelection,
    /// room drain/close semantics.
    RoomDrainCloseSemantics,
    /// SFU session/endpoint lifecycle semantics.
    SfuSessionEndpointLifecycleSemantics,
    /// TURN allocation/permission relay semantics.
    TurnAllocationPermissionRelaySemantics,
    /// socket/listener stop.
    SocketListenerStop,
    /// runtime task/worker stop.
    RuntimeTaskWorkerStop,
    /// packet/buffer release.
    PacketBufferRelease,
    /// audit/persistence/metrics flush execution.
    AuditPersistenceMetricsFlush,
    /// unclean process termination observation.
    UncleanProcessTerminationObservation,
}

impl ShutdownDrainConcern {
    /// concern owner です。
    pub const fn owner(self) -> ShutdownDrainOwner {
        match self {
            Self::ProcessSignalObservation | Self::ShutdownModeSelection => {
                ShutdownDrainOwner::Entrypoints
            }
            Self::SplitServiceShutdownControl => ShutdownDrainOwner::InternalControlPlaneContract,
            Self::RoomDrainCloseSemantics => ShutdownDrainOwner::CoreSignaling,
            Self::SfuSessionEndpointLifecycleSemantics => ShutdownDrainOwner::CoreSfu,
            Self::TurnAllocationPermissionRelaySemantics => ShutdownDrainOwner::CoreTurn,
            Self::SocketListenerStop
            | Self::PacketBufferRelease
            | Self::AuditPersistenceMetricsFlush => ShutdownDrainOwner::Driver,
            Self::RuntimeTaskWorkerStop => ShutdownDrainOwner::DriverEntrypointsRuntime,
            Self::UncleanProcessTerminationObservation => {
                ShutdownDrainOwner::EntrypointsDriverObservation
            }
        }
    }
}

/// cross-plane shutdown の drain sequence step です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrainSequenceStep {
    /// entrypoints creates shutdown correlation and typed shutdown request.
    CreateShutdownCorrelationAndRequest,
    /// entrypoints stops new external admission at listener/connection boundary.
    StopNewExternalAdmission,
    /// core/signaling begins room drain or rejects unavailable room commands.
    BeginSignalingRoomDrain,
    /// core/sfu drains sessions/endpoints.
    DrainSfuSessionsEndpoints,
    /// core/turn stops new allocation/permission paths.
    StopTurnAllocationPermissionPaths,
    /// driver stops receive loops and new buffer leases.
    StopDriverReceiveLoopsAndBufferLeases,
    /// driver/entrypoints cancel or join runtime tasks.
    CancelOrJoinRuntimeTasks,
    /// driver drains bounded queues.
    DrainBoundedQueues,
    /// driver releases buffers, relay resources, and sockets.
    ReleaseBuffersRelayResourcesSockets,
    /// entrypoints stops runtime after mandatory evidence path is attempted.
    StopRuntimeAfterEvidencePathAttempt,
}

impl DrainSequenceStep {
    /// Canonical sequence order です。
    pub const fn order(self) -> u8 {
        match self {
            Self::CreateShutdownCorrelationAndRequest => 1,
            Self::StopNewExternalAdmission => 2,
            Self::BeginSignalingRoomDrain => 3,
            Self::DrainSfuSessionsEndpoints => 4,
            Self::StopTurnAllocationPermissionPaths => 5,
            Self::StopDriverReceiveLoopsAndBufferLeases => 6,
            Self::CancelOrJoinRuntimeTasks => 7,
            Self::DrainBoundedQueues => 8,
            Self::ReleaseBuffersRelayResourcesSockets => 9,
            Self::StopRuntimeAfterEvidencePathAttempt => 10,
        }
    }
}

/// shutdown/drain が影響する plane です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownDrainPlane {
    /// Signaling plane.
    Signaling,
    /// SFU plane.
    Sfu,
    /// TURN plane.
    Turn,
    /// network driver plane.
    NetworkDriver,
    /// persistence/audit/metrics driver plane.
    PersistenceAuditMetricsDriver,
    /// runtime task/worker plane.
    RuntimeTaskWorker,
}

/// drain mode の core-facing class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownDrainMode {
    /// stop admission and drain materialized state.
    GracefulDrain,
    /// drain required by runtime reconfiguration.
    ReconfigurationDrain,
    /// stop admission only for pre-core external input.
    AdmissionStopOnly,
}

/// shutdown/drain decision outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownDrainOutcome {
    /// accepted.
    Accepted,
    /// rejected.
    Rejected,
    /// failed.
    Failed,
    /// drained.
    Drained,
    /// observation is unclean termination, not graceful drain.
    UncleanTerminationObserved,
}

/// plane-specific failure relation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownDrainFailureKind {
    /// room is draining.
    RoomDraining,
    /// room is closed.
    RoomClosed,
    /// room close/drain transition invalid.
    RoomCloseNotAllowed,
    /// SFU session is not accepting.
    SfuSessionNotAccepting,
    /// endpoint closed by backpressure.
    EndpointClosedByBackpressure,
    /// driver shutdown.
    DriverShutdown,
    /// network receive failed.
    NetworkReceiveFailed,
    /// network send failed.
    NetworkSendFailed,
    /// persistence unavailable.
    PersistenceUnavailable,
    /// audit backlog bound exceeded.
    AuditBacklogBoundExceeded,
    /// metrics export failed.
    MetricsExportFailed,
}

impl ShutdownDrainFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::RoomDraining => "room_draining",
            Self::RoomClosed => "room_closed",
            Self::RoomCloseNotAllowed => "room_close_not_allowed",
            Self::SfuSessionNotAccepting => "sfu_session_not_accepting",
            Self::EndpointClosedByBackpressure => "endpoint_closed_by_backpressure",
            Self::DriverShutdown => "driver_shutdown",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::AuditBacklogBoundExceeded => "audit_backlog_bound_exceeded",
            Self::MetricsExportFailed => "metrics_export_failed",
        }
    }
}

/// plane-specific drain rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaneDrainRule {
    plane: ShutdownDrainPlane,
    owner: ShutdownDrainOwner,
    required_failure_reasons: &'static [ShutdownDrainFailureKind],
}

impl PlaneDrainRule {
    /// plane drain rule を作ります。
    pub const fn new(
        plane: ShutdownDrainPlane,
        owner: ShutdownDrainOwner,
        required_failure_reasons: &'static [ShutdownDrainFailureKind],
    ) -> Self {
        Self {
            plane,
            owner,
            required_failure_reasons,
        }
    }
}

const SIGNALING_DRAIN_FAILURES: &[ShutdownDrainFailureKind] = &[
    ShutdownDrainFailureKind::RoomDraining,
    ShutdownDrainFailureKind::RoomClosed,
    ShutdownDrainFailureKind::RoomCloseNotAllowed,
];
const SFU_DRAIN_FAILURES: &[ShutdownDrainFailureKind] = &[
    ShutdownDrainFailureKind::SfuSessionNotAccepting,
    ShutdownDrainFailureKind::EndpointClosedByBackpressure,
    ShutdownDrainFailureKind::DriverShutdown,
];
const TURN_DRAIN_FAILURES: &[ShutdownDrainFailureKind] =
    &[ShutdownDrainFailureKind::DriverShutdown];
const NETWORK_DRAIN_FAILURES: &[ShutdownDrainFailureKind] = &[
    ShutdownDrainFailureKind::DriverShutdown,
    ShutdownDrainFailureKind::NetworkReceiveFailed,
    ShutdownDrainFailureKind::NetworkSendFailed,
];
const FLUSH_DRAIN_FAILURES: &[ShutdownDrainFailureKind] = &[
    ShutdownDrainFailureKind::PersistenceUnavailable,
    ShutdownDrainFailureKind::AuditBacklogBoundExceeded,
    ShutdownDrainFailureKind::MetricsExportFailed,
    ShutdownDrainFailureKind::DriverShutdown,
];

/// Canonical plane-specific drain rule catalog です。
pub const PLANE_DRAIN_RULES: &[PlaneDrainRule] = &[
    PlaneDrainRule::new(
        ShutdownDrainPlane::Signaling,
        ShutdownDrainOwner::CoreSignaling,
        SIGNALING_DRAIN_FAILURES,
    ),
    PlaneDrainRule::new(
        ShutdownDrainPlane::Sfu,
        ShutdownDrainOwner::CoreSfu,
        SFU_DRAIN_FAILURES,
    ),
    PlaneDrainRule::new(
        ShutdownDrainPlane::Turn,
        ShutdownDrainOwner::CoreTurn,
        TURN_DRAIN_FAILURES,
    ),
    PlaneDrainRule::new(
        ShutdownDrainPlane::NetworkDriver,
        ShutdownDrainOwner::Driver,
        NETWORK_DRAIN_FAILURES,
    ),
    PlaneDrainRule::new(
        ShutdownDrainPlane::PersistenceAuditMetricsDriver,
        ShutdownDrainOwner::Driver,
        FLUSH_DRAIN_FAILURES,
    ),
];

/// shutdown/drain evidence shape です。これは runtime 成功ではなく、必要 field の境界を示します。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShutdownDrainEvidenceShape {
    shutdown_correlation_id: CorrelationId,
    startup_run_id: Option<StartupRunId>,
    drain_mode: ShutdownDrainMode,
    reconfiguration_generation: Option<ConfigurationScopeRef>,
    affected_plane: ShutdownDrainPlane,
    outcome: ShutdownDrainOutcome,
    reason: Option<ShutdownDrainFailureKind>,
    bounded_flush_or_retry_recorded: bool,
    runtime_task_join_or_cancel_recorded: bool,
    unreleased_resource_count: Option<u64>,
}

impl ShutdownDrainEvidenceShape {
    /// shutdown/drain evidence に必要な field を持つ shape を作ります。
    pub const fn new(
        shutdown_correlation_id: CorrelationId,
        startup_run_id: Option<StartupRunId>,
        drain_mode: ShutdownDrainMode,
        reconfiguration_generation: Option<ConfigurationScopeRef>,
        affected_plane: ShutdownDrainPlane,
        outcome: ShutdownDrainOutcome,
        reason: Option<ShutdownDrainFailureKind>,
        bounded_flush_or_retry_recorded: bool,
        runtime_task_join_or_cancel_recorded: bool,
        unreleased_resource_count: Option<u64>,
    ) -> Self {
        Self {
            shutdown_correlation_id,
            startup_run_id,
            drain_mode,
            reconfiguration_generation,
            affected_plane,
            outcome,
            reason,
            bounded_flush_or_retry_recorded,
            runtime_task_join_or_cancel_recorded,
            unreleased_resource_count,
        }
    }
}

/// shutdown/drain で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedShutdownDrainBehavior {
    /// entrypoints directly mutate room/SFU/TURN state without core decision.
    EntrypointsDirectDomainStateMutation,
    /// driver socket close is treated as successful domain leave/close.
    DriverSocketCloseAsDomainSuccess,
    /// shutdown failure is hidden behind process exit code only.
    FailureHiddenBehindProcessExitCode,
    /// crash/panic/supervisor restart is reported as graceful drain success.
    UncleanTerminationAsGracefulDrain,
    /// unbounded drain wait or flush queue is allowed.
    UnboundedDrainOrFlush,
    /// failed audit/persistence flush is used as closeout evidence.
    FailedFlushAsCloseoutEvidence,
    /// one plane successful drain implies another plane successful drain.
    CrossPlaneDrainSuccessInference,
    /// split-service drain control lacks internal control-plane correlation/audit evidence.
    SplitServiceDrainWithoutControlPlaneEvidence,
    /// runtime reconfiguration applies before required drain/restart.
    ReconfigurationBeforeRequiredDrainRestart,
    /// detached/unjoined worker remains while graceful drain is claimed.
    DetachedWorkerDuringGracefulDrainClaim,
}

/// atomicity / transaction / compensation concern の owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicityOwner {
    /// core が aggregate transition と decision outcome を所有します。
    Core,
    /// driver が concrete transaction や persistence execution を所有します。
    Driver,
    /// driver/sdk が external response projection/send execution を所有します。
    DriverSdk,
    /// domain state change の compensation は core が所有します。
    CoreDomainCompensation,
    /// external resource cleanup の compensation は driver が所有します。
    DriverExternalResourceCompensation,
    /// closeout evidence adoption は reports 側の証跡 class です。
    Reports,
}

/// atomicity boundary concern の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicityConcern {
    /// domain decision atomicity.
    DomainDecisionAtomicity,
    /// port intent emission.
    PortIntentEmission,
    /// concrete DB transaction.
    ConcreteDbTransaction,
    /// audit persistence execution.
    AuditPersistenceExecution,
    /// external response emission.
    ExternalResponseEmission,
    /// compensation decision.
    CompensationDecision,
    /// evidence adoption.
    EvidenceAdoption,
}

impl AtomicityConcern {
    /// concern owner です。
    pub const fn owner(self) -> AtomicityOwner {
        match self {
            Self::DomainDecisionAtomicity | Self::PortIntentEmission => AtomicityOwner::Core,
            Self::ConcreteDbTransaction | Self::AuditPersistenceExecution => {
                AtomicityOwner::Driver
            }
            Self::ExternalResponseEmission => AtomicityOwner::DriverSdk,
            Self::CompensationDecision => AtomicityOwner::CoreDomainCompensation,
            Self::EvidenceAdoption => AtomicityOwner::Reports,
        }
    }
}

/// v0.2 initial architecture が認める atomicity class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicityClass {
    /// one core decision without external side effect claim.
    SingleCoreDecision,
    /// domain decision requires audit evidence.
    CoreDecisionPlusAuditRequired,
    /// driver execution follows accepted decision.
    CoreDecisionPlusPortIntent,
    /// DB/file/network resource transaction.
    DriverLocalTransaction,
    /// accepted state needs follow-up compensation.
    CompensatingTransitionRequired,
    /// observation cannot be undone.
    NonCompensableObservation,
}

/// command path の commit boundary です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitBoundary {
    /// before core entry.
    BeforeCoreEntry,
    /// after core validation but before aggregate mutation.
    AfterCoreValidationBeforeAggregateMutation,
    /// after aggregate transition.
    AfterAggregateTransition,
    /// after audit projection.
    AfterAuditProjection,
    /// after driver persistence.
    AfterDriverPersistence,
    /// after external response emission.
    AfterExternalResponseEmission,
}

impl CommitBoundary {
    /// command path 上の順序です。
    pub const fn order(self) -> u8 {
        match self {
            Self::BeforeCoreEntry => 1,
            Self::AfterCoreValidationBeforeAggregateMutation => 2,
            Self::AfterAggregateTransition => 3,
            Self::AfterAuditProjection => 4,
            Self::AfterDriverPersistence => 5,
            Self::AfterExternalResponseEmission => 6,
        }
    }
}

/// commit step の outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicStepOutcome {
    /// step accepted/succeeded.
    Accepted,
    /// step failed.
    Failed,
    /// step skipped because it is outside the command path.
    NotApplicable,
    /// step outcome is not claimed as close evidence.
    CloseNotClaimed,
}

/// atomicity / compensation failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicityFailureKind {
    /// atomic commit step failed.
    AtomicCommitFailed,
    /// compensation required but not available.
    CompensationRequired,
    /// compensation execution failed.
    CompensationFailed,
    /// persistence unavailable during commit.
    PersistenceUnavailable,
    /// audit backlog prevents required audit.
    AuditBacklogBoundExceeded,
    /// external response encoding failed.
    ExternalEncodeFailed,
    /// external response send failed.
    NetworkSendFailed,
    /// driver shutdown during commit/compensation.
    DriverShutdown,
}

impl AtomicityFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::AtomicCommitFailed => "atomic_commit_failed",
            Self::CompensationRequired => "compensation_required",
            Self::CompensationFailed => "compensation_failed",
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::AuditBacklogBoundExceeded => "audit_backlog_bound_exceeded",
            Self::ExternalEncodeFailed => "external_encode_failed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// command path の per-step outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtomicCommitStep {
    boundary: CommitBoundary,
    outcome: AtomicStepOutcome,
    failure_reason: Option<AtomicityFailureKind>,
}

