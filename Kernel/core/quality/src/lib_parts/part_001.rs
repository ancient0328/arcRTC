// core/quality は quality metrics、admission、resource/backpressure の core 語彙を所有する surface です。
//
// metrics exporter や monitoring runtime は drivers/observability に置き、
// ここでは通信意味論に必要な品質判断の境界だけを扱います。

use arcrtc_core_identity::{AllocationId, EndpointId, PacketId, PermissionId, RouteId, StreamId};

/// core quality package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreQualitySurface;

/// core が扱う metric kind の基本集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityMetricKind {
    /// round-trip time.
    Rtt,
    /// packet loss.
    PacketLoss,
    /// jitter.
    Jitter,
    /// bitrate.
    Bitrate,
    /// frame rate when required.
    FrameRate,
    /// relay latency.
    RelayLatency,
    /// queue depth.
    QueueDepth,
    /// backpressure state.
    BackpressureState,
    /// route health.
    RouteHealth,
    /// MOS-like score if explicitly defined.
    MosLikeScore,
}

/// normalized quality measurement です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QualityMetric {
    kind: QualityMetricKind,
    normalized_value: i64,
    unit: &'static str,
    window: &'static str,
}

impl QualityMetric {
    /// normalized unit/window を持つ metric を作ります。
    pub const fn new(
        kind: QualityMetricKind,
        normalized_value: i64,
        unit: &'static str,
        window: &'static str,
    ) -> Self {
        Self {
            kind,
            normalized_value,
            unit,
            window,
        }
    }
}

/// quality threshold です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QualityThreshold {
    metric_kind: QualityMetricKind,
    threshold_value: i64,
    unit: &'static str,
}

impl QualityThreshold {
    /// threshold を作ります。
    pub const fn new(
        metric_kind: QualityMetricKind,
        threshold_value: i64,
        unit: &'static str,
    ) -> Self {
        Self {
            metric_kind,
            threshold_value,
            unit,
        }
    }
}

/// core quality decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityDecisionKind {
    /// quality normal.
    Normal,
    /// quality degraded.
    Degraded,
    /// quality violation.
    Violation,
    /// route suppression.
    RouteSuppression,
    /// backpressure action.
    BackpressureAction,
    /// recovery allowed.
    RecoveryAllowed,
    /// admission rejected by quality policy.
    AdmissionRejected,
}

/// quality target reference の閉集合です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QualityTargetRef {
    /// endpoint target.
    Endpoint(EndpointId),
    /// route target.
    Route(RouteId),
    /// stream target.
    Stream(StreamId),
    /// packet target.
    Packet(PacketId),
}

/// quality decision failure/reason mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityFailureKind {
    /// packet forwarding suppressed by quality.
    PacketSuppressedByQuality,
    /// route suppressed by quality policy.
    RouteSuppressedByQuality,
    /// route/endpoint recovery rejected by quality policy.
    QualityRecoveryNotAllowed,
    /// endpoint admission rejected by quality policy.
    EndpointQualityNotAllowed,
    /// admitted endpoint degraded by quality policy.
    EndpointDegradedByQuality,
    /// publication admission or suppression by quality policy.
    PublicationQualityNotAllowed,
    /// subscription admission or suppression by quality policy.
    SubscriptionQualityNotAllowed,
}

impl QualityFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PacketSuppressedByQuality => "packet_suppressed_by_quality",
            Self::RouteSuppressedByQuality => "route_suppressed_by_quality",
            Self::QualityRecoveryNotAllowed => "quality_recovery_not_allowed",
            Self::EndpointQualityNotAllowed => "endpoint_quality_not_allowed",
            Self::EndpointDegradedByQuality => "endpoint_degraded_by_quality",
            Self::PublicationQualityNotAllowed => "publication_quality_not_allowed",
            Self::SubscriptionQualityNotAllowed => "subscription_quality_not_allowed",
        }
    }
}

/// admission scope の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdmissionScope {
    /// process/runtime wide policy scope.
    Global,
    /// room-scoped policy.
    Room,
    /// SFU session-scoped policy.
    Session,
    /// TURN allocation-scoped policy.
    Allocation,
    /// opaque credential/key verification result scope.
    CredentialRef,
    /// pre-core connection scope.
    DriverConnectionRef,
}

/// required admission target です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdmissionTarget {
    /// room materialization / active room.
    RoomMaterialization,
    /// participant join.
    ParticipantJoin,
    /// signaling command queue admission.
    SignalingCommandQueue,
    /// SFU endpoint admission.
    SfuEndpointAdmission,
    /// SFU route candidate construction.
    SfuRouteCandidateConstruction,
    /// TURN allocation.
    TurnAllocation,
    /// TURN permission.
    TurnPermission,
    /// connection concurrency.
    ConnectionConcurrency,
    /// inbound frame.
    InboundFrame,
}

/// rate limit window policy shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RateWindowPolicy {
    scope: AdmissionScope,
    maximum_count: Option<u64>,
    maximum_bytes: Option<u64>,
    maximum_window_ms: Option<u64>,
    reset_semantics: &'static str,
}

impl RateWindowPolicy {
    /// bounded rate window policy を作ります。
    pub const fn new(
        scope: AdmissionScope,
        maximum_count: Option<u64>,
        maximum_bytes: Option<u64>,
        maximum_window_ms: Option<u64>,
        reset_semantics: &'static str,
    ) -> Self {
        Self {
            scope,
            maximum_count,
            maximum_bytes,
            maximum_window_ms,
            reset_semantics,
        }
    }
}

/// admission decision class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdmissionDecisionKind {
    /// accepted.
    Accepted,
    /// rejected.
    Rejected,
    /// dropped.
    Dropped,
    /// shed.
    Shed,
    /// expired.
    Expired,
}

/// admission / quota / rate failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdmissionFailureKind {
    /// room capacity exceeded.
    RoomCapacityExceeded,
    /// admission capacity exceeded.
    AdmissionCapacityExceeded,
    /// signaling command queue bound exceeded.
    SignalingCommandQueueBoundExceeded,
    /// endpoint capacity exceeded.
    EndpointCapacityExceeded,
    /// route candidate bound exceeded.
    RouteCandidateBoundExceeded,
    /// allocation capacity exceeded.
    AllocationCapacityExceeded,
    /// permission capacity exceeded.
    PermissionCapacityExceeded,
    /// connection concurrency exceeded.
    ConnectionConcurrencyExceeded,
    /// frame size bound exceeded.
    FrameSizeBoundExceeded,
    /// driver shutdown.
    DriverShutdown,
    /// runtime config missing.
    RuntimeConfigMissing,
    /// runtime config invalid.
    RuntimeConfigInvalid,
    /// persistence unavailable.
    PersistenceUnavailable,
    /// authorization context missing.
    AuthorizationContextMissing,
    /// authorization scope not allowed.
    AuthorizationScopeNotAllowed,
    /// client address untrusted.
    ClientAddressUntrusted,
    /// forwarded header untrusted.
    ForwardedHeaderUntrusted,
}

impl AdmissionFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::RoomCapacityExceeded => "room_capacity_exceeded",
            Self::AdmissionCapacityExceeded => "admission_capacity_exceeded",
            Self::SignalingCommandQueueBoundExceeded => "signaling_command_queue_bound_exceeded",
            Self::EndpointCapacityExceeded => "endpoint_capacity_exceeded",
            Self::RouteCandidateBoundExceeded => "route_candidate_bound_exceeded",
            Self::AllocationCapacityExceeded => "allocation_capacity_exceeded",
            Self::PermissionCapacityExceeded => "permission_capacity_exceeded",
            Self::ConnectionConcurrencyExceeded => "connection_concurrency_exceeded",
            Self::FrameSizeBoundExceeded => "frame_size_bound_exceeded",
            Self::DriverShutdown => "driver_shutdown",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::AuthorizationContextMissing => "authorization_context_missing",
            Self::AuthorizationScopeNotAllowed => "authorization_scope_not_allowed",
            Self::ClientAddressUntrusted => "client_address_untrusted",
            Self::ForwardedHeaderUntrusted => "forwarded_header_untrusted",
        }
    }
}

/// bounded resource の policy owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourcePolicyOwner {
    /// core が policy を所有します。
    Core,
    /// driver が policy を所有します。
    Driver,
    /// task owner に応じて core または driver が policy を所有します。
    CoreOrDriverByTaskOwner,
}

/// bounded resource の physical owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourcePhysicalOwner {
    /// driver が物理 resource を所有します。
    Driver,
    /// 実装上の queue/lock 所有者が物理 resource を所有します。
    CoreOrDriverImplementationDetail,
    /// driver/entrypoints runtime が物理 resource を所有します。
    DriverEntrypointsRuntime,
}

/// bounded resource の measurement owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceMeasurementOwner {
    /// driver が計測値を所有します。
    Driver,
    /// 物理 queue/lock の owner が計測値を所有します。
    PhysicalOwner,
    /// driver/entrypoints runtime が task lifecycle の計測値を所有します。
    DriverEntrypointsRuntime,
}

/// bounded resource の execution owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceExecutionOwner {
    /// core decision を driver が実行します。
    CoreDecisionDriverExecution,
    /// core expiry decision を driver timer が実行します。
    CoreExpiryDecisionDriverTimerExecution,
    /// driver queue が実行します。
    DriverQueueExecution,
    /// driver cache が実行します。
    DriverCacheExecution,
    /// core routing decision として実行します。
    CoreRoutingDecision,
    /// core allocation decision を driver が実行します。
    CoreAllocationDecisionDriverExecution,
    /// core permission decision を driver が実行します。
    CorePermissionDecisionDriverExecution,
    /// driver sink が実行します。
    DriverSinkExecution,
    /// driver retry が実行します。
    DriverRetryExecution,
    /// driver export が実行します。
    DriverExportExecution,
    /// driver pool が実行します。
    DriverPoolExecution,
    /// driver conversion が実行します。
    DriverConversion,
    /// core pressure decision を driver が実行します。
    CorePressureDecisionDriverExecution,
    /// core decision を物理 owner が実行します。
    CoreDecisionPhysicalOwnerExecution,
    /// task lifecycle decision を物理 owner が実行します。
    TaskLifecycleDecisionPhysicalOwnerExecution,
}

/// resource owner tuple です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceOwnerTuple {
    policy_owner: ResourcePolicyOwner,
    physical_owner: ResourcePhysicalOwner,
    measurement_owner: ResourceMeasurementOwner,
    execution_owner: ResourceExecutionOwner,
}

impl ResourceOwnerTuple {
    /// Canonical 上の owner tuple を作ります。
    const fn new(
        policy_owner: ResourcePolicyOwner,
        physical_owner: ResourcePhysicalOwner,
        measurement_owner: ResourceMeasurementOwner,
        execution_owner: ResourceExecutionOwner,
    ) -> Self {
        Self {
            policy_owner,
            physical_owner,
            measurement_owner,
            execution_owner,
        }
    }
}

/// v0.2 initial canonical が要求する bounded resource の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceBoundKind {
    /// active room set.
    ActiveRoomSet,
    /// room lifecycle.
    RoomLifecycle,
    /// signaling command queue.
    SignalingCommandQueue,
    /// room participant set.
    RoomParticipantSet,
    /// SFU endpoint admission.
    SfuEndpointAdmission,
    /// SFU packet cache.
    SfuPacketCache,
    /// SFU transmit queue.
    SfuTransmitQueue,
    /// SFU route candidates.
    SfuRouteCandidates,
    /// TURN allocation table.
    TurnAllocationTable,
    /// TURN allocation lifetime.
    TurnAllocationLifetime,
    /// TURN refresh cap.
    TurnRefreshCap,
    /// TURN permission table.
    TurnPermissionTable,
    /// TURN permission lifetime.
    TurnPermissionLifetime,
    /// TURN channel bind lifetime.
    TurnChannelBindLifetime,
    /// TURN relay queue.
    TurnRelayQueue,
    /// audit sink backlog.
    AuditSinkBacklog,
    /// persistence retry store.
    PersistenceRetryStore,
    /// metrics export backlog.
    MetricsExportBacklog,
    /// driver receive buffer pool.
    DriverReceiveBufferPool,
    /// inbound frame size.
    InboundFrameSize,
    /// connection concurrency.
    ConnectionConcurrency,
    /// memory pressure.
    MemoryPressure,
    /// aggregate serialization queue / lock wait.
    AggregateSerializationQueueLockWait,
    /// runtime task queue / worker mailbox / join wait.
    RuntimeTaskQueueWorkerMailboxJoinWait,
}

impl ResourceBoundKind {
    /// Canonical の resource name です。
    pub const fn resource_name(self) -> &'static str {
        match self {
            Self::ActiveRoomSet => "active room set",
            Self::RoomLifecycle => "room lifecycle",
            Self::SignalingCommandQueue => "signaling command queue",
            Self::RoomParticipantSet => "room participant set",
            Self::SfuEndpointAdmission => "SFU endpoint admission",
            Self::SfuPacketCache => "SFU packet cache",
            Self::SfuTransmitQueue => "SFU transmit queue",
            Self::SfuRouteCandidates => "SFU route candidates",
            Self::TurnAllocationTable => "TURN allocation table",
            Self::TurnAllocationLifetime => "TURN allocation lifetime",
            Self::TurnRefreshCap => "TURN refresh cap",
            Self::TurnPermissionTable => "TURN permission table",
            Self::TurnPermissionLifetime => "TURN permission lifetime",
            Self::TurnChannelBindLifetime => "TURN channel bind lifetime",
            Self::TurnRelayQueue => "TURN relay queue",
            Self::AuditSinkBacklog => "audit sink backlog",
            Self::PersistenceRetryStore => "persistence retry store",
            Self::MetricsExportBacklog => "metrics export backlog",
            Self::DriverReceiveBufferPool => "driver receive buffer pool",
            Self::InboundFrameSize => "inbound frame size",
            Self::ConnectionConcurrency => "connection concurrency",
            Self::MemoryPressure => "memory pressure",
            Self::AggregateSerializationQueueLockWait => {
                "aggregate serialization queue / lock wait"
            }
            Self::RuntimeTaskQueueWorkerMailboxJoinWait => {
                "runtime task queue / worker mailbox / join wait"
            }
        }
    }
}

/// required bound の shape です。数値は driver/config surface が所有し、この型は必須 dimension だけを固定します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequiredResourceBound {
    resource: ResourceBoundKind,
    required_bound: &'static str,
    exceeded_reason_codes: &'static [&'static str],
    owner_tuple: ResourceOwnerTuple,
}

impl RequiredResourceBound {
    /// required bound row を作ります。
    const fn new(
        resource: ResourceBoundKind,
        required_bound: &'static str,
        exceeded_reason_codes: &'static [&'static str],
        owner_tuple: ResourceOwnerTuple,
    ) -> Self {
        Self {
            resource,
            required_bound,
            exceeded_reason_codes,
            owner_tuple,
        }
    }
}

const CORE_DRIVER_CORE_DECISION_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::CoreDecisionDriverExecution,
);

