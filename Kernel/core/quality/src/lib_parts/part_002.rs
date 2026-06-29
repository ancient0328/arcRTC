const CORE_DRIVER_CORE_EXPIRY_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::CoreExpiryDecisionDriverTimerExecution,
);

const CORE_DRIVER_QUEUE_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::DriverQueueExecution,
);

const CORE_DRIVER_CACHE_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::DriverCacheExecution,
);

const CORE_DRIVER_ROUTING_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::CoreRoutingDecision,
);

const CORE_DRIVER_ALLOCATION_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::CoreAllocationDecisionDriverExecution,
);

const CORE_DRIVER_PERMISSION_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::CorePermissionDecisionDriverExecution,
);

const CORE_DRIVER_SINK_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::DriverSinkExecution,
);

const CORE_DRIVER_RETRY_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::DriverRetryExecution,
);

const CORE_DRIVER_EXPORT_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::DriverExportExecution,
);

const CORE_DRIVER_POOL_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::DriverPoolExecution,
);

const DRIVER_CONVERSION_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Driver,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::DriverConversion,
);

const CORE_DRIVER_PRESSURE_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::Driver,
    ResourceMeasurementOwner::Driver,
    ResourceExecutionOwner::CorePressureDecisionDriverExecution,
);

const SERIALIZATION_QUEUE_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::Core,
    ResourcePhysicalOwner::CoreOrDriverImplementationDetail,
    ResourceMeasurementOwner::PhysicalOwner,
    ResourceExecutionOwner::CoreDecisionPhysicalOwnerExecution,
);

const RUNTIME_TASK_OWNER: ResourceOwnerTuple = ResourceOwnerTuple::new(
    ResourcePolicyOwner::CoreOrDriverByTaskOwner,
    ResourcePhysicalOwner::DriverEntrypointsRuntime,
    ResourceMeasurementOwner::DriverEntrypointsRuntime,
    ResourceExecutionOwner::TaskLifecycleDecisionPhysicalOwnerExecution,
);

/// v0.2 initial canonical の required bound catalog です。
pub const REQUIRED_RESOURCE_BOUNDS: &[RequiredResourceBound] = &[
    RequiredResourceBound::new(
        ResourceBoundKind::ActiveRoomSet,
        "maximum active rooms / maximum materializations per window",
        &["room_capacity_exceeded"],
        CORE_DRIVER_CORE_DECISION_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::RoomLifecycle,
        "maximum room lifetime / maximum idle duration",
        &["room_lifetime_exceeded"],
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::SignalingCommandQueue,
        "maximum commands / maximum wait duration",
        &["signaling_command_queue_bound_exceeded"],
        CORE_DRIVER_QUEUE_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::RoomParticipantSet,
        "maximum participants",
        &["admission_capacity_exceeded"],
        CORE_DRIVER_CORE_DECISION_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::SfuEndpointAdmission,
        "maximum admitted endpoints / maximum endpoint admission window",
        &["endpoint_capacity_exceeded"],
        CORE_DRIVER_CORE_DECISION_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::SfuPacketCache,
        "maximum packets / bytes / retention duration",
        &["packet_cache_bound_exceeded", "retention_duration_exceeded"],
        CORE_DRIVER_CACHE_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::SfuTransmitQueue,
        "maximum packets / bytes / wait duration",
        &["sfu_transmit_queue_bound_exceeded"],
        CORE_DRIVER_QUEUE_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::SfuRouteCandidates,
        "maximum candidates per decision",
        &["route_candidate_bound_exceeded"],
        CORE_DRIVER_ROUTING_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::TurnAllocationTable,
        "maximum allocations",
        &["allocation_capacity_exceeded"],
        CORE_DRIVER_ALLOCATION_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::TurnAllocationLifetime,
        "maximum lifetime",
        &["allocation_lifetime_exceeded"],
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::TurnRefreshCap,
        "maximum refresh count / maximum cumulative refresh duration",
        &["refresh_limit_exceeded"],
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::TurnPermissionTable,
        "maximum permissions per allocation",
        &["permission_capacity_exceeded"],
        CORE_DRIVER_PERMISSION_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::TurnPermissionLifetime,
        "maximum permission lifetime",
        &["permission_lifetime_exceeded"],
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::TurnChannelBindLifetime,
        "maximum channel binding lifetime",
        &["channel_bind_lifetime_exceeded"],
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::TurnRelayQueue,
        "maximum packets / bytes / wait duration",
        &["turn_relay_queue_bound_exceeded"],
        CORE_DRIVER_QUEUE_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::AuditSinkBacklog,
        "maximum events / bytes / retry duration",
        &[
            "audit_backlog_bound_exceeded",
            "retention_duration_exceeded",
        ],
        CORE_DRIVER_SINK_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::PersistenceRetryStore,
        "maximum retry entries / bytes / retry count / retry duration",
        &[
            "persistence_retry_bound_exceeded",
            "persistence_retry_duration_exceeded",
        ],
        CORE_DRIVER_RETRY_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::MetricsExportBacklog,
        "maximum events / bytes",
        &["metrics_backlog_bound_exceeded"],
        CORE_DRIVER_EXPORT_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::DriverReceiveBufferPool,
        "maximum leases / bytes",
        &["buffer_pool_bound_exceeded"],
        CORE_DRIVER_POOL_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::InboundFrameSize,
        "maximum bytes per frame / message",
        &["frame_size_bound_exceeded"],
        DRIVER_CONVERSION_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::ConnectionConcurrency,
        "maximum concurrent connections / admission window",
        &["connection_concurrency_exceeded"],
        CORE_DRIVER_CORE_DECISION_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::MemoryPressure,
        "maximum memory pressure state / pressure duration",
        &["memory_pressure_exceeded"],
        CORE_DRIVER_PRESSURE_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::AggregateSerializationQueueLockWait,
        "maximum pending commands / maximum wait duration per serialization scope",
        &["lock_contention_bound_exceeded"],
        SERIALIZATION_QUEUE_OWNER,
    ),
    RequiredResourceBound::new(
        ResourceBoundKind::RuntimeTaskQueueWorkerMailboxJoinWait,
        "maximum pending tasks / maximum wait duration / maximum cancellation wait",
        &["runtime_task_queue_bound_exceeded"],
        RUNTIME_TASK_OWNER,
    ),
];

/// resource/backpressure に接続する audit event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceBoundAuditEventType {
    /// backpressure_decision.
    BackpressureDecision,
    /// sfu_subscription_decision.
    SfuSubscriptionDecision,
    /// resource_bound_decision.
    ResourceBoundDecision,
    /// driver_resource_bound_decision.
    DriverResourceBoundDecision,
    /// turn_allocation_decision.
    TurnAllocationDecision,
    /// turn_refresh_decision.
    TurnRefreshDecision,
    /// turn_permission_decision.
    TurnPermissionDecision,
    /// turn_channel_bind_decision.
    TurnChannelBindDecision,
    /// runtime_task_lifecycle_decision.
    RuntimeTaskLifecycleDecision,
}

impl ResourceBoundAuditEventType {
    /// audit event catalog 上の code です。
    pub const fn code(self) -> &'static str {
        match self {
            Self::BackpressureDecision => "backpressure_decision",
            Self::SfuSubscriptionDecision => "sfu_subscription_decision",
            Self::ResourceBoundDecision => "resource_bound_decision",
            Self::DriverResourceBoundDecision => "driver_resource_bound_decision",
            Self::TurnAllocationDecision => "turn_allocation_decision",
            Self::TurnRefreshDecision => "turn_refresh_decision",
            Self::TurnPermissionDecision => "turn_permission_decision",
            Self::TurnChannelBindDecision => "turn_channel_bind_decision",
            Self::RuntimeTaskLifecycleDecision => "runtime_task_lifecycle_decision",
        }
    }
}

/// resource bound decision outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceBoundOutcome {
    /// accepted.
    Accepted,
    /// within_bound_observed.
    WithinBoundObserved,
    /// rejected.
    Rejected,
    /// dropped.
    Dropped,
    /// shed.
    Shed,
    /// expired.
    Expired,
}

impl ResourceBoundOutcome {
    /// audit outcome code です。
    pub const fn code(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::WithinBoundObserved => "within_bound_observed",
            Self::Rejected => "rejected",
            Self::Dropped => "dropped",
            Self::Shed => "shed",
            Self::Expired => "expired",
        }
    }
}

/// bound exceeded 時の closed action の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceClosedActionKind {
    /// reject room materialization.
    RejectRoomMaterialization,
    /// expire or close room lifecycle.
    ExpireOrCloseRoomLifecycle,
    /// reject queued command admission.
    RejectQueuedCommandAdmission,
    /// reject participant admission.
    RejectParticipantAdmission,
    /// reject endpoint admission.
    RejectEndpointAdmission,
    /// drop or evict packet retention.
    DropOrEvictPacketRetention,
    /// expire packet retention.
    ExpirePacketRetention,
    /// drop packet enqueue / scheduling.
    DropPacketEnqueueScheduling,
    /// reject route candidate construction.
    RejectRouteCandidateConstruction,
    /// reject allocation.
    RejectAllocation,
    /// expire allocation.
    ExpireAllocation,
    /// expire allocation refresh path.
    ExpireAllocationRefreshPath,
    /// reject permission.
    RejectPermission,
    /// expire permission.
    ExpirePermission,
    /// expire channel bind.
    ExpireChannelBind,
    /// drop relay data scheduling.
    DropRelayDataScheduling,
    /// shed audit backlog and reject new audit-required path.
    ShedAuditBacklogAndRejectNewAuditRequiredPath,
    /// expire audit retry entry and reject new audit-required path.
    ExpireAuditRetryAndRejectNewAuditRequiredPath,
    /// shed retry entry.
    ShedRetryEntry,
    /// expire retry entry.
    ExpireRetryEntry,
    /// shed metrics export item.
    ShedMetricsExportItem,
    /// reject receive buffer lease.
    RejectReceiveBufferLease,
    /// drop inbound frame before core entry.
    DropInboundFrameBeforeCoreEntry,
    /// reject connection admission.
    RejectConnectionAdmission,
    /// shed lower-priority resource action.
    ShedLowerPriorityResourceAction,
    /// reject command admission to serialization scope.
    RejectCommandAdmissionToSerializationScope,
    /// reject spawn/schedule/join continuation.
    RejectSpawnScheduleJoinContinuation,
}

/// required bound closed action の行です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceBoundClosedAction {
    resource: ResourceBoundKind,
    action: ResourceClosedActionKind,
    audit_event_type: ResourceBoundAuditEventType,
    outcome: ResourceBoundOutcome,
    reason_code: &'static str,
    owner_tuple: ResourceOwnerTuple,
}

impl ResourceBoundClosedAction {
    /// closed action mapping を作ります。
    const fn new(
        resource: ResourceBoundKind,
        action: ResourceClosedActionKind,
        audit_event_type: ResourceBoundAuditEventType,
        outcome: ResourceBoundOutcome,
        reason_code: &'static str,
        owner_tuple: ResourceOwnerTuple,
    ) -> Self {
        Self {
            resource,
            action,
            audit_event_type,
            outcome,
            reason_code,
            owner_tuple,
        }
    }

    /// resource kind です。
    pub const fn resource(&self) -> ResourceBoundKind {
        self.resource
    }

    /// closed action kind です。
    pub const fn action(&self) -> ResourceClosedActionKind {
        self.action
    }

    /// audit event type です。
    pub const fn audit_event_type(&self) -> ResourceBoundAuditEventType {
        self.audit_event_type
    }

    /// audit outcome です。
    pub const fn outcome(&self) -> ResourceBoundOutcome {
        self.outcome
    }

    /// reason catalog code です。
    pub const fn reason_code(&self) -> &'static str {
        self.reason_code
    }

    /// owner tuple です。
    pub const fn owner_tuple(&self) -> ResourceOwnerTuple {
        self.owner_tuple
    }
}

