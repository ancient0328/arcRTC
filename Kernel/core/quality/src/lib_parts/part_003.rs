/// v0.2 initial canonical の required bound closed action catalog です。
pub const REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS: &[ResourceBoundClosedAction] = &[
    ResourceBoundClosedAction::new(
        ResourceBoundKind::ActiveRoomSet,
        ResourceClosedActionKind::RejectRoomMaterialization,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Rejected,
        "room_capacity_exceeded",
        CORE_DRIVER_CORE_DECISION_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::RoomLifecycle,
        ResourceClosedActionKind::ExpireOrCloseRoomLifecycle,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Expired,
        "room_lifetime_exceeded",
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::SignalingCommandQueue,
        ResourceClosedActionKind::RejectQueuedCommandAdmission,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Rejected,
        "signaling_command_queue_bound_exceeded",
        CORE_DRIVER_QUEUE_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::RoomParticipantSet,
        ResourceClosedActionKind::RejectParticipantAdmission,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Rejected,
        "admission_capacity_exceeded",
        CORE_DRIVER_CORE_DECISION_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::SfuEndpointAdmission,
        ResourceClosedActionKind::RejectEndpointAdmission,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Rejected,
        "endpoint_capacity_exceeded",
        CORE_DRIVER_CORE_DECISION_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::SfuPacketCache,
        ResourceClosedActionKind::DropOrEvictPacketRetention,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Dropped,
        "packet_cache_bound_exceeded",
        CORE_DRIVER_CACHE_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::SfuPacketCache,
        ResourceClosedActionKind::ExpirePacketRetention,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Expired,
        "retention_duration_exceeded",
        CORE_DRIVER_CACHE_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::SfuTransmitQueue,
        ResourceClosedActionKind::DropPacketEnqueueScheduling,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Dropped,
        "sfu_transmit_queue_bound_exceeded",
        CORE_DRIVER_QUEUE_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::SfuRouteCandidates,
        ResourceClosedActionKind::RejectRouteCandidateConstruction,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Rejected,
        "route_candidate_bound_exceeded",
        CORE_DRIVER_ROUTING_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::TurnAllocationTable,
        ResourceClosedActionKind::RejectAllocation,
        ResourceBoundAuditEventType::TurnAllocationDecision,
        ResourceBoundOutcome::Rejected,
        "allocation_capacity_exceeded",
        CORE_DRIVER_ALLOCATION_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::TurnAllocationLifetime,
        ResourceClosedActionKind::ExpireAllocation,
        ResourceBoundAuditEventType::TurnAllocationDecision,
        ResourceBoundOutcome::Expired,
        "allocation_lifetime_exceeded",
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::TurnRefreshCap,
        ResourceClosedActionKind::ExpireAllocationRefreshPath,
        ResourceBoundAuditEventType::TurnRefreshDecision,
        ResourceBoundOutcome::Expired,
        "refresh_limit_exceeded",
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::TurnPermissionTable,
        ResourceClosedActionKind::RejectPermission,
        ResourceBoundAuditEventType::TurnPermissionDecision,
        ResourceBoundOutcome::Rejected,
        "permission_capacity_exceeded",
        CORE_DRIVER_PERMISSION_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::TurnPermissionLifetime,
        ResourceClosedActionKind::ExpirePermission,
        ResourceBoundAuditEventType::TurnPermissionDecision,
        ResourceBoundOutcome::Expired,
        "permission_lifetime_exceeded",
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::TurnChannelBindLifetime,
        ResourceClosedActionKind::ExpireChannelBind,
        ResourceBoundAuditEventType::TurnChannelBindDecision,
        ResourceBoundOutcome::Expired,
        "channel_bind_lifetime_exceeded",
        CORE_DRIVER_CORE_EXPIRY_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::TurnRelayQueue,
        ResourceClosedActionKind::DropRelayDataScheduling,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Dropped,
        "turn_relay_queue_bound_exceeded",
        CORE_DRIVER_QUEUE_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::AuditSinkBacklog,
        ResourceClosedActionKind::ShedAuditBacklogAndRejectNewAuditRequiredPath,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Shed,
        "audit_backlog_bound_exceeded",
        CORE_DRIVER_SINK_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::AuditSinkBacklog,
        ResourceClosedActionKind::ExpireAuditRetryAndRejectNewAuditRequiredPath,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Expired,
        "retention_duration_exceeded",
        CORE_DRIVER_SINK_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::PersistenceRetryStore,
        ResourceClosedActionKind::ShedRetryEntry,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Shed,
        "persistence_retry_bound_exceeded",
        CORE_DRIVER_RETRY_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::PersistenceRetryStore,
        ResourceClosedActionKind::ExpireRetryEntry,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Expired,
        "persistence_retry_duration_exceeded",
        CORE_DRIVER_RETRY_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::MetricsExportBacklog,
        ResourceClosedActionKind::ShedMetricsExportItem,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Shed,
        "metrics_backlog_bound_exceeded",
        CORE_DRIVER_EXPORT_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::DriverReceiveBufferPool,
        ResourceClosedActionKind::RejectReceiveBufferLease,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Rejected,
        "buffer_pool_bound_exceeded",
        CORE_DRIVER_POOL_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::InboundFrameSize,
        ResourceClosedActionKind::DropInboundFrameBeforeCoreEntry,
        ResourceBoundAuditEventType::DriverResourceBoundDecision,
        ResourceBoundOutcome::Dropped,
        "frame_size_bound_exceeded",
        DRIVER_CONVERSION_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::ConnectionConcurrency,
        ResourceClosedActionKind::RejectConnectionAdmission,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Rejected,
        "connection_concurrency_exceeded",
        CORE_DRIVER_CORE_DECISION_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::MemoryPressure,
        ResourceClosedActionKind::ShedLowerPriorityResourceAction,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Shed,
        "memory_pressure_exceeded",
        CORE_DRIVER_PRESSURE_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::AggregateSerializationQueueLockWait,
        ResourceClosedActionKind::RejectCommandAdmissionToSerializationScope,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundOutcome::Rejected,
        "lock_contention_bound_exceeded",
        SERIALIZATION_QUEUE_OWNER,
    ),
    ResourceBoundClosedAction::new(
        ResourceBoundKind::RuntimeTaskQueueWorkerMailboxJoinWait,
        ResourceClosedActionKind::RejectSpawnScheduleJoinContinuation,
        ResourceBoundAuditEventType::RuntimeTaskLifecycleDecision,
        ResourceBoundOutcome::Rejected,
        "runtime_task_queue_bound_exceeded",
        RUNTIME_TASK_OWNER,
    ),
];

/// resource-bound audit に必要な typed reference 群です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceBoundReferenceSet {
    packet_id: Option<PacketId>,
    allocation_id: Option<AllocationId>,
    permission_id: Option<PermissionId>,
    endpoint_id: Option<EndpointId>,
    route_id: Option<RouteId>,
    driver_resource_ref: Option<&'static str>,
}

impl ResourceBoundReferenceSet {
    /// reference なしの set を作ります。
    pub const fn none() -> Self {
        Self {
            packet_id: None,
            allocation_id: None,
            permission_id: None,
            endpoint_id: None,
            route_id: None,
            driver_resource_ref: None,
        }
    }

    /// TURN relay queue bound 用の必須 reference set を作ります。
    pub const fn turn_relay_queue(
        allocation_id: AllocationId,
        permission_id: PermissionId,
    ) -> Self {
        Self {
            packet_id: None,
            allocation_id: Some(allocation_id),
            permission_id: Some(permission_id),
            endpoint_id: None,
            route_id: None,
            driver_resource_ref: None,
        }
    }

    /// packet-scoped resource bound 用の reference set を作ります。
    pub const fn packet(packet_id: PacketId) -> Self {
        Self {
            packet_id: Some(packet_id),
            allocation_id: None,
            permission_id: None,
            endpoint_id: None,
            route_id: None,
            driver_resource_ref: None,
        }
    }

    /// endpoint-scoped resource bound 用の reference set を作ります。
    pub const fn endpoint(endpoint_id: EndpointId) -> Self {
        Self {
            packet_id: None,
            allocation_id: None,
            permission_id: None,
            endpoint_id: Some(endpoint_id),
            route_id: None,
            driver_resource_ref: None,
        }
    }

    /// route-scoped resource bound 用の reference set を作ります。
    pub const fn route(route_id: RouteId) -> Self {
        Self {
            packet_id: None,
            allocation_id: None,
            permission_id: None,
            endpoint_id: None,
            route_id: Some(route_id),
            driver_resource_ref: None,
        }
    }

    /// driver-local resource bound 用の reference set を作ります。
    pub const fn driver_resource(driver_resource_ref: &'static str) -> Self {
        Self {
            packet_id: None,
            allocation_id: None,
            permission_id: None,
            endpoint_id: None,
            route_id: None,
            driver_resource_ref: Some(driver_resource_ref),
        }
    }
}

/// resource bound decision shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceBoundDecisionShapeError {
    /// TURN relay queue bound requires AllocationId and PermissionId.
    TurnRelayQueueReferencesMissing,
    /// packet-scoped resource bound requires PacketId.
    PacketReferenceMissing,
    /// driver-local resource bound requires driver resource reference.
    DriverResourceReferenceMissing,
}

/// resource bound decision の core 形状です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceBoundDecision {
    closed_action: ResourceBoundClosedAction,
    references: ResourceBoundReferenceSet,
}

impl ResourceBoundDecision {
    /// Canonical mapping 済みの decision を作ります。
    pub fn try_new(
        closed_action: ResourceBoundClosedAction,
        references: ResourceBoundReferenceSet,
    ) -> Result<Self, ResourceBoundDecisionShapeError> {
        match closed_action.resource {
            ResourceBoundKind::TurnRelayQueue
                if references.allocation_id.is_none() || references.permission_id.is_none() =>
            {
                return Err(ResourceBoundDecisionShapeError::TurnRelayQueueReferencesMissing);
            }
            ResourceBoundKind::SfuPacketCache | ResourceBoundKind::SfuTransmitQueue
                if references.packet_id.is_none() =>
            {
                return Err(ResourceBoundDecisionShapeError::PacketReferenceMissing);
            }
            ResourceBoundKind::InboundFrameSize if references.driver_resource_ref.is_none() => {
                return Err(ResourceBoundDecisionShapeError::DriverResourceReferenceMissing);
            }
            _ => {}
        }

        Ok(Self {
            closed_action,
            references,
        })
    }
}

/// backpressure decision outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackpressureOutcome {
    /// accepted.
    Accepted,
    /// delayed.
    Delayed,
    /// suppressed.
    Suppressed,
    /// dropped.
    Dropped,
    /// degraded.
    Degraded,
    /// closed_by_policy.
    ClosedByPolicy,
    /// rejected.
    Rejected,
}

impl BackpressureOutcome {
    /// audit outcome code です。
    pub const fn code(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Delayed => "delayed",
            Self::Suppressed => "suppressed",
            Self::Dropped => "dropped",
            Self::Degraded => "degraded",
            Self::ClosedByPolicy => "closed_by_policy",
            Self::Rejected => "rejected",
        }
    }
}

/// core が所有する backpressure decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackpressureDecisionKind {
    /// accept.
    Accept,
    /// delay action.
    DelayAction,
    /// suppress forwarding.
    SuppressForwarding,
    /// suppress subscription.
    SuppressSubscription,
    /// suppress route state.
    SuppressRouteState,
    /// drop packet.
    DropPacket,
    /// stop retaining packet because of pressure policy.
    StopRetainingPacketByPressurePolicy,
    /// degrade route.
    DegradeRoute,
    /// close endpoint.
    CloseEndpoint,
    /// reject recovery from delayed, degraded, or suppressed route state.
    RejectRecoveryFromBackpressureState,
}

impl BackpressureDecisionKind {
    /// required audit event type です。
    pub const fn audit_event_type(self) -> ResourceBoundAuditEventType {
        match self {
            Self::SuppressSubscription => ResourceBoundAuditEventType::SfuSubscriptionDecision,
            Self::Accept
            | Self::DelayAction
            | Self::SuppressForwarding
            | Self::SuppressRouteState
            | Self::DropPacket
            | Self::StopRetainingPacketByPressurePolicy
            | Self::DegradeRoute
            | Self::CloseEndpoint
            | Self::RejectRecoveryFromBackpressureState => {
                ResourceBoundAuditEventType::BackpressureDecision
            }
        }
    }

    /// required audit outcome です。
    pub const fn outcome(self) -> BackpressureOutcome {
        match self {
            Self::Accept => BackpressureOutcome::Accepted,
            Self::DelayAction => BackpressureOutcome::Delayed,
            Self::SuppressForwarding | Self::SuppressSubscription | Self::SuppressRouteState => {
                BackpressureOutcome::Suppressed
            }
            Self::DropPacket | Self::StopRetainingPacketByPressurePolicy => {
                BackpressureOutcome::Dropped
            }
            Self::DegradeRoute => BackpressureOutcome::Degraded,
            Self::CloseEndpoint => BackpressureOutcome::ClosedByPolicy,
            Self::RejectRecoveryFromBackpressureState => BackpressureOutcome::Rejected,
        }
    }

    /// required reason code です。成功扱いの accept だけ reason を要求しません。
    pub const fn reason_code(self) -> Option<&'static str> {
        match self {
            Self::Accept => None,
            Self::DelayAction => Some("action_delayed_by_backpressure"),
            Self::SuppressForwarding => Some("packet_suppressed_by_backpressure"),
            Self::SuppressSubscription => Some("subscription_backpressure_suppressed"),
            Self::SuppressRouteState => Some("route_suppressed_by_backpressure"),
            Self::DropPacket | Self::StopRetainingPacketByPressurePolicy => {
                Some("packet_dropped_by_backpressure")
            }
            Self::DegradeRoute => Some("route_degraded_by_backpressure"),
            Self::CloseEndpoint => Some("endpoint_closed_by_backpressure"),
            Self::RejectRecoveryFromBackpressureState => Some("backpressure_recovery_not_allowed"),
        }
    }
}

/// backpressure の affected target です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackpressureTargetRef {
    /// target なし、または global/resource scoped。
    NotApplicable,
    /// endpoint-scoped target.
    Endpoint(EndpointId),
    /// route-scoped target.
    Route(RouteId),
    /// packet-scoped target.
    Packet(PacketId),
    /// subscription stream target.
    Subscription(StreamId),
}

/// backpressure decision の core 形状です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackpressureDecision {
    kind: BackpressureDecisionKind,
    target: BackpressureTargetRef,
}

/// backpressure decision shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackpressureDecisionShapeError {
    /// accept does not carry an affected target.
    AcceptTargetMustBeNotApplicable,
    /// action requires route target.
    RouteTargetRequired,
    /// action requires packet target.
    PacketTargetRequired,
    /// action requires subscription stream target.
    SubscriptionTargetRequired,
    /// action requires endpoint target.
    EndpointTargetRequired,
}

