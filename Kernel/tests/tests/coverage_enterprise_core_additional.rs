use arcrtc_core_identity::{
    AllocationId, AuditEventId, ConfigurationScopeRef, CorrelationId, EndpointId, OpaqueReference,
    PacketId, ParticipantId, PermissionId, ReferenceAuthority, RoomId, RouteId, SessionId,
    StreamId,
};
use arcrtc_core_operation::{
    AtomicCommitStep, AtomicStepOutcome, AtomicityClass, AtomicityCompensationDecision,
    AtomicityConcern, AtomicityFailureKind, CommitBoundary, CompensationExternalResponseRule,
    CompensationOwner, CompensationRule, CompensationStatus, DrainSequenceStep, OrderingConcern,
    OrderingDecision, OrderingFailureKind, PhysicalLockResource, ProhibitedAtomicityBehavior,
    ProhibitedOrderingLockBehavior, ProhibitedRetryTimeoutCancellationBehavior,
    ProhibitedShutdownDrainBehavior, RetryAttemptShape, RetryClass, RetryPreconditions,
    RetryTimeoutConcern, RetryTimeoutFailureKind, SerializationBoundPolicy, SerializationScope,
    SerializationScopeReference, ShutdownDrainConcern, ShutdownDrainFailureKind, ShutdownDrainMode,
    ShutdownDrainOutcome, TimeoutDeadlineSurface, PLANE_DRAIN_RULES,
};
use arcrtc_core_ports::{
    LoadedCoreStateRef, MetricsExportAcknowledgement, MetricsSinkFailure, MetricsSinkFailureKind,
    MetricsSinkInput, MetricsSinkOutput, NetworkDeliveryObservation, NetworkPortInput,
    NetworkPortOutput, PacketViewClass, PersistenceAcknowledgement,
    PersistenceConsistencyRequirement, PersistenceIntentClass, PersistenceOperationKind,
    PersistencePortFailure, PersistencePortFailureKind, PersistencePortInput,
    PersistencePortIntent, PersistencePortIntentError, PersistencePortOutput, PersistenceRecordRef,
    PersistenceStateClass, PortCallContext, PortCallShape, PortContractShape, PortError,
    PortErrorClass, PortFamily, PortOwnershipRule, RuntimePortOutputClass,
};
use arcrtc_core_protocol::{
    CanonicalDataClass, CanonicalDigest, CanonicalEncodingError, CanonicalEncodingFailureKind,
    CanonicalEncodingRuleSet, CanonicalFormatVersion, CanonicalRuleStatus, Capability,
    CapabilityRule, CompatibilityChange, CompatibilityClassification, CompatibilityFailureKind,
    CompatibilityVersionRange, ContractVersion, DeprecationDecision, DeprecationLifecycleStep,
    UnknownFieldHandling, VersionNegotiationOutcome, VersionOwner, VersionedSurface,
    VersionedSurfaceOwnership, WireEncodingVersion,
};
use arcrtc_core_quality::{
    AdmissionDecisionKind, AdmissionFailureKind, AdmissionScope, AdmissionTarget,
    AuditBacklogOverflowRule, BackpressureDecision, BackpressureDecisionKind,
    BackpressureDecisionShapeError, BackpressureTargetRef, QualityDecisionKind, QualityFailureKind,
    QualityMetric, QualityMetricKind, QualityTargetRef, QualityThreshold, RateWindowPolicy,
    ResourceBoundAuditEventType, ResourceBoundDecision, ResourceBoundDecisionShapeError,
    ResourceBoundOutcome, ResourceBoundReferenceSet, REQUIRED_RESOURCE_BOUNDS,
    REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS,
};
use arcrtc_core_reason::{find_reason_definition, CatalogedReasonRef, Reason};
use arcrtc_core_state::{StateClass, StateFamily};

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("reference must be valid")
}

fn correlation(value: &str) -> CorrelationId {
    CorrelationId::new(reference(value))
}

fn cataloged(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason code must be cataloged")
}

fn assert_reason_is_cataloged(code: &str) {
    assert_eq!(cataloged(code).definition().code().as_str(), code);
}

#[test]
fn coverage_core_operation_closed_vocabularies_are_exercised() {
    for (concern, owner) in [
        (
            ShutdownDrainConcern::ProcessSignalObservation,
            "Entrypoints",
        ),
        (
            ShutdownDrainConcern::SplitServiceShutdownControl,
            "InternalControlPlaneContract",
        ),
        (ShutdownDrainConcern::ShutdownModeSelection, "Entrypoints"),
        (
            ShutdownDrainConcern::RoomDrainCloseSemantics,
            "CoreSignaling",
        ),
        (
            ShutdownDrainConcern::SfuSessionEndpointLifecycleSemantics,
            "CoreSfu",
        ),
        (
            ShutdownDrainConcern::TurnAllocationPermissionRelaySemantics,
            "CoreTurn",
        ),
        (ShutdownDrainConcern::SocketListenerStop, "Driver"),
        (
            ShutdownDrainConcern::RuntimeTaskWorkerStop,
            "DriverEntrypointsRuntime",
        ),
        (ShutdownDrainConcern::PacketBufferRelease, "Driver"),
        (ShutdownDrainConcern::AuditPersistenceMetricsFlush, "Driver"),
        (
            ShutdownDrainConcern::UncleanProcessTerminationObservation,
            "EntrypointsDriverObservation",
        ),
    ] {
        assert_eq!(format!("{:?}", concern.owner()), owner);
    }

    for (step, order) in [
        (DrainSequenceStep::CreateShutdownCorrelationAndRequest, 1),
        (DrainSequenceStep::StopNewExternalAdmission, 2),
        (DrainSequenceStep::BeginSignalingRoomDrain, 3),
        (DrainSequenceStep::DrainSfuSessionsEndpoints, 4),
        (DrainSequenceStep::StopTurnAllocationPermissionPaths, 5),
        (DrainSequenceStep::StopDriverReceiveLoopsAndBufferLeases, 6),
        (DrainSequenceStep::CancelOrJoinRuntimeTasks, 7),
        (DrainSequenceStep::DrainBoundedQueues, 8),
        (DrainSequenceStep::ReleaseBuffersRelayResourcesSockets, 9),
        (DrainSequenceStep::StopRuntimeAfterDrainFinalization, 10),
    ] {
        assert_eq!(step.order(), order);
    }

    for kind in [
        ShutdownDrainFailureKind::RoomDraining,
        ShutdownDrainFailureKind::RoomClosed,
        ShutdownDrainFailureKind::RoomCloseNotAllowed,
        ShutdownDrainFailureKind::SfuSessionNotAccepting,
        ShutdownDrainFailureKind::EndpointClosedByBackpressure,
        ShutdownDrainFailureKind::DriverShutdown,
        ShutdownDrainFailureKind::NetworkReceiveFailed,
        ShutdownDrainFailureKind::NetworkSendFailed,
        ShutdownDrainFailureKind::PersistenceUnavailable,
        ShutdownDrainFailureKind::AuditBacklogBoundExceeded,
        ShutdownDrainFailureKind::MetricsExportFailed,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    assert_eq!(PLANE_DRAIN_RULES.len(), 5);

    let _other_modes = [
        ShutdownDrainMode::GracefulDrain,
        ShutdownDrainMode::AdmissionStopOnly,
    ];
    let _other_outcomes = [
        ShutdownDrainOutcome::Accepted,
        ShutdownDrainOutcome::Rejected,
        ShutdownDrainOutcome::Drained,
        ShutdownDrainOutcome::UncleanTerminationObserved,
    ];
    let _prohibited_drain = [
        ProhibitedShutdownDrainBehavior::EntrypointsDirectDomainStateMutation,
        ProhibitedShutdownDrainBehavior::DriverSocketCloseAsDomainSuccess,
        ProhibitedShutdownDrainBehavior::FailureHiddenBehindProcessExitCode,
        ProhibitedShutdownDrainBehavior::UncleanTerminationAsGracefulDrain,
        ProhibitedShutdownDrainBehavior::UnboundedDrainOrFlush,
        ProhibitedShutdownDrainBehavior::FailedFlushAsSuccessfulShutdown,
        ProhibitedShutdownDrainBehavior::CrossPlaneDrainSuccessInference,
        ProhibitedShutdownDrainBehavior::SplitServiceDrainWithoutControlPlaneAuditRelation,
        ProhibitedShutdownDrainBehavior::ReconfigurationBeforeRequiredDrainRestart,
        ProhibitedShutdownDrainBehavior::DetachedWorkerDuringGracefulDrainClaim,
    ];

    for (concern, owner) in [
        (AtomicityConcern::DomainDecisionAtomicity, "Core"),
        (AtomicityConcern::PortIntentEmission, "Core"),
        (AtomicityConcern::ConcreteDbTransaction, "Driver"),
        (AtomicityConcern::AuditPersistenceExecution, "Driver"),
        (AtomicityConcern::ExternalResponseEmission, "DriverSdk"),
        (
            AtomicityConcern::CompensationDecision,
            "CoreDomainCompensation",
        ),
    ] {
        assert_eq!(format!("{:?}", concern.owner()), owner);
    }

    for (boundary, order) in [
        (CommitBoundary::BeforeCoreEntry, 1),
        (
            CommitBoundary::AfterCoreValidationBeforeAggregateMutation,
            2,
        ),
        (CommitBoundary::AfterAggregateTransition, 3),
        (CommitBoundary::AfterAuditProjection, 4),
        (CommitBoundary::AfterDriverPersistence, 5),
        (CommitBoundary::AfterExternalResponseEmission, 6),
    ] {
        assert_eq!(boundary.order(), order);
    }

    for kind in [
        AtomicityFailureKind::AtomicCommitFailed,
        AtomicityFailureKind::CompensationRequired,
        AtomicityFailureKind::CompensationFailed,
        AtomicityFailureKind::PersistenceUnavailable,
        AtomicityFailureKind::AuditBacklogBoundExceeded,
        AtomicityFailureKind::ExternalEncodeFailed,
        AtomicityFailureKind::NetworkSendFailed,
        AtomicityFailureKind::DriverShutdown,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    let step = AtomicCommitStep::new(
        CommitBoundary::AfterDriverPersistence,
        AtomicStepOutcome::Failed,
        Some(AtomicityFailureKind::PersistenceUnavailable),
    );
    let _other_step_outcomes = [
        AtomicStepOutcome::Accepted,
        AtomicStepOutcome::NotApplicable,
    ];
    let _classes = [
        AtomicityClass::SingleCoreDecision,
        AtomicityClass::CoreDecisionPlusAuditRequired,
        AtomicityClass::CoreDecisionPlusPortIntent,
        AtomicityClass::DriverLocalTransaction,
        AtomicityClass::CompensatingTransitionRequired,
        AtomicityClass::NonCompensableObservation,
    ];
    let _rule = CompensationRule::new(
        AtomicityFailureKind::CompensationRequired,
        "room membership transition",
        CompensationOwner::CoreWhenDomainStateChanges,
        "reject transition after compensation failure",
        "atomicity_compensation_decision",
        CompensationExternalResponseRule::ProjectLaterFailure,
    );
    let _driver_rule = CompensationRule::new(
        AtomicityFailureKind::NetworkSendFailed,
        "external send",
        CompensationOwner::DriverForExternalResourceCleanup,
        "release external resource",
        "driver_cleanup_observation",
        CompensationExternalResponseRule::NoExternalResponse,
    );
    let decision = AtomicityCompensationDecision::new(
        correlation("core-operation-atomicity"),
        AtomicityClass::CompensatingTransitionRequired,
        CommitBoundary::AfterDriverPersistence,
        step,
        CompensationStatus::RequiredAndAvailable,
    );
    assert_eq!(
        decision.audit_event_type(),
        "atomicity_compensation_decision"
    );
    let _compensation_statuses = [
        CompensationStatus::NotRequired,
        CompensationStatus::RequiredButUnavailable,
        CompensationStatus::Executed,
        CompensationStatus::ExecutionFailed,
    ];
    let _compensation_response_rules = [
        CompensationExternalResponseRule::PreserveOriginalAcceptedDecision,
        CompensationExternalResponseRule::ProjectLaterFailure,
        CompensationExternalResponseRule::NoExternalResponse,
    ];
    let _prohibited_atomicity = [
        ProhibitedAtomicityBehavior::DriverTransactionDefinesDomainInvariant,
        ProhibitedAtomicityBehavior::PortIntentAsDriverExecutionSuccess,
        ProhibitedAtomicityBehavior::ExternalResponseAsAuditPersistenceSuccess,
        ProhibitedAtomicityBehavior::CompensationWithoutCoreStateMachineRule,
        ProhibitedAtomicityBehavior::PartialSuccessHiddenBehindFinalSuccess,
        ProhibitedAtomicityBehavior::FailedAuditPersistenceAsSuccessfulCommit,
    ];
}

#[test]
fn coverage_core_operation_ordering_retry_and_timeout_paths_are_exercised() {
    for (concern, owner) in [
        (OrderingConcern::DomainOrderingValidity, "Core"),
        (OrderingConcern::AggregateSerializationScope, "Core"),
        (
            OrderingConcern::PhysicalLockOrMailbox,
            "CoreOrDriverImplementationDetail",
        ),
        (OrderingConcern::RuntimeScheduling, "Driver"),
        (OrderingConcern::RuntimeTaskLifecycle, "Driver"),
        (OrderingConcern::InboundWireOrderingObservation, "Driver"),
        (OrderingConcern::Idempotency, "Core"),
    ] {
        assert_eq!(format!("{:?}", concern.owner()), owner);
    }

    for scope in [
        SerializationScope::RoomScope,
        SerializationScope::ParticipantScope,
        SerializationScope::SfuSessionScope,
        SerializationScope::SfuEndpointScope,
        SerializationScope::PacketLifecycleScope,
        SerializationScope::TurnAllocationScope,
        SerializationScope::TurnPermissionScope,
        SerializationScope::AuditChainScope,
        SerializationScope::ConfigurationScope,
    ] {
        assert!(!scope.protected_semantics().is_empty());
        assert!(!format!("{:?}", scope.owner()).is_empty());
    }

    let references = [
        SerializationScopeReference::Room(RoomId::new(reference("core-order-room"))),
        SerializationScopeReference::Participant(ParticipantId::new(reference(
            "core-order-participant",
        ))),
        SerializationScopeReference::SfuSession(SessionId::new(reference("core-order-session"))),
        SerializationScopeReference::SfuEndpoint(EndpointId::new(reference("core-order-endpoint"))),
        SerializationScopeReference::Packet(PacketId::new(reference("core-order-packet"))),
        SerializationScopeReference::TurnAllocation(AllocationId::new(reference(
            "core-order-allocation",
        ))),
        SerializationScopeReference::TurnPermission(PermissionId::new(reference(
            "core-order-permission",
        ))),
        SerializationScopeReference::AuditChain(AuditEventId::new(reference("core-order-audit"))),
        SerializationScopeReference::Configuration(ConfigurationScopeRef::new(reference(
            "core-order-config",
        ))),
    ];
    assert_eq!(references.len(), 9);

    for kind in [
        OrderingFailureKind::CommandOrderViolation,
        OrderingFailureKind::DuplicateCommand,
        OrderingFailureKind::ConcurrencyConflict,
        OrderingFailureKind::LockContentionBoundExceeded,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    for lock in [
        PhysicalLockResource::AggregateMutationGuard,
        PhysicalLockResource::DriverQueueMutex,
        PhysicalLockResource::EntrypointLevelProcessLock,
        PhysicalLockResource::TestHarnessSynchronization,
    ] {
        assert!(!format!("{:?}", lock.allowed_owner()).is_empty());
    }
    let _policy = SerializationBoundPolicy::new(
        SerializationScope::RoomScope,
        true,
        true,
        SerializationScope::RoomScope.owner(),
        "reject and audit",
    );
    let _ordering = OrderingDecision::new(
        correlation("core-ordering"),
        SerializationScope::RoomScope,
        Some(SerializationScopeReference::Room(RoomId::new(reference(
            "core-ordering-room",
        )))),
        AtomicStepOutcome::Failed,
        Some(OrderingFailureKind::LockContentionBoundExceeded),
    );
    let _prohibited_ordering = [
        ProhibitedOrderingLockBehavior::DriverLockOrderDefinesDomainOrder,
        ProhibitedOrderingLockBehavior::RuntimeSchedulerOrderAsStateMachineAuthority,
        ProhibitedOrderingLockBehavior::UnboundedAggregateCommandQueue,
        ProhibitedOrderingLockBehavior::LockObjectInDomainOrPublicApi,
        ProhibitedOrderingLockBehavior::LastWriterWinsWithoutCoreDecision,
        ProhibitedOrderingLockBehavior::TestSynchronizationAsProductionSemantics,
        ProhibitedOrderingLockBehavior::TaskSchedulingAsSerializationAuthority,
    ];

    for (concern, owner) in [
        (
            RetryTimeoutConcern::RetryabilityMetadata,
            "CoreReasonCatalog",
        ),
        (RetryTimeoutConcern::DomainCommandIdempotency, "Core"),
        (RetryTimeoutConcern::DriverOperationRetryExecution, "Driver"),
        (RetryTimeoutConcern::CommandDeadlinePolicy, "Core"),
        (RetryTimeoutConcern::TimerSchedulerExecution, "Driver"),
        (
            RetryTimeoutConcern::ClientCancellationObservation,
            "DriverSdk",
        ),
        (
            RetryTimeoutConcern::EntrypointShutdownCancellation,
            "Entrypoints",
        ),
    ] {
        assert_eq!(format!("{:?}", concern.owner()), owner);
    }
    for retry_class in [
        RetryClass::NoRetry,
        RetryClass::IdempotentCommandReplay,
        RetryClass::DriverTransportRetry,
        RetryClass::PersistenceRetry,
        RetryClass::AuditSinkRetry,
        RetryClass::TestOnlyRetry,
    ] {
        assert_eq!(
            retry_class.is_runtime_retry_class(),
            !matches!(retry_class, RetryClass::TestOnlyRetry)
        );
        assert!(!format!("{:?}", retry_class.owner()).is_empty());
    }
    for surface in [
        TimeoutDeadlineSurface::CommandDeadline,
        TimeoutDeadlineSurface::DriverSendTimeout,
        TimeoutDeadlineSurface::DriverReceiveTimeout,
        TimeoutDeadlineSurface::PersistenceRetryDuration,
        TimeoutDeadlineSurface::PacketCacheRetentionDuration,
        TimeoutDeadlineSurface::RuntimeShutdownDuringScheduledAction,
    ] {
        assert!(!format!("{:?}", surface.owner()).is_empty());
        assert_reason_is_cataloged(surface.failure_reason().reason_code());
    }
    for kind in [
        RetryTimeoutFailureKind::OperationDeadlineExceeded,
        RetryTimeoutFailureKind::OperationCancelled,
        RetryTimeoutFailureKind::PersistenceRetryBoundExceeded,
        RetryTimeoutFailureKind::PersistenceRetryDurationExceeded,
        RetryTimeoutFailureKind::RetentionDurationExceeded,
        RetryTimeoutFailureKind::NetworkSendFailed,
        RetryTimeoutFailureKind::NetworkReceiveFailed,
        RetryTimeoutFailureKind::AuditBacklogBoundExceeded,
        RetryTimeoutFailureKind::DriverShutdown,
        RetryTimeoutFailureKind::RuntimeTaskCancelFailed,
        RetryTimeoutFailureKind::RuntimeTaskJoinFailed,
        RetryTimeoutFailureKind::RuntimeTaskQueueBoundExceeded,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    let retry_allowed = RetryPreconditions::new(true, true, true, true, true, true, true);
    assert!(retry_allowed.allows_retry());
    let retry_rejected = RetryPreconditions::new(true, true, true, true, false, true, true);
    assert!(!retry_rejected.allows_retry());
    let _attempt: RetryAttemptShape<RoomId, ParticipantId> = RetryAttemptShape::new(
        RetryClass::IdempotentCommandReplay,
        correlation("core-retry-attempt"),
        None,
        2,
        retry_allowed,
    );
    let _prohibited_retry = [
        ProhibitedRetryTimeoutCancellationBehavior::AutomaticRetryOfNonIdempotentDomainCommand,
        ProhibitedRetryTimeoutCancellationBehavior::DriverRetryChangesDomainDecision,
        ProhibitedRetryTimeoutCancellationBehavior::CancellationAsAcceptedDomainTransition,
        ProhibitedRetryTimeoutCancellationBehavior::TimeoutAsGenericSuccessOrFreeTextFailure,
        ProhibitedRetryTimeoutCancellationBehavior::UnboundedRetry,
        ProhibitedRetryTimeoutCancellationBehavior::TestOnlyRetryAsRuntimePolicy,
        ProhibitedRetryTimeoutCancellationBehavior::RetryErasesOriginalCorrelationOrFirstAttemptIdentity,
        ProhibitedRetryTimeoutCancellationBehavior::RuntimeTaskCancellationAsDomainSuccess,
    ];
}

#[test]
fn coverage_core_ports_persistence_and_sink_failures_are_closed() {
    let context = PortCallContext::new(correlation("core-port-context"));
    assert_eq!(context.correlation_id().as_str(), "core-port-context");

    let all_families = [
        PortFamily::Clock,
        PortFamily::Random,
        PortFamily::TokenVerifier,
        PortFamily::Network,
        PortFamily::WebRtcTransport,
        PortFamily::PacketView,
        PortFamily::Persistence,
        PortFamily::AuditSink,
        PortFamily::MetricsSink,
        PortFamily::Runtime,
    ];
    assert_eq!(all_families.len(), 10);

    let delivery = NetworkDeliveryObservation::new(
        correlation("core-port-delivery"),
        Some(reference("core-port-peer")),
        true,
    );
    assert_eq!(delivery.correlation_id().as_str(), "core-port-delivery");
    assert_eq!(
        delivery.peer_ref().expect("peer").as_str(),
        "core-port-peer"
    );
    assert!(delivery.delivered());
    let _network_input = NetworkPortInput::CloseConnection(reference("core-port-connection"));
    let _network_output = NetworkPortOutput::DeliveryObservation(delivery);

    let metric = QualityMetric::new(QualityMetricKind::QueueDepth, 42, "items", "p95");
    let _metrics_input = MetricsSinkInput::SubmitQualityMetric(metric);
    let _metrics_output =
        MetricsSinkOutput::Acknowledgement(MetricsExportAcknowledgement::new(true, false));
    for kind in [
        MetricsSinkFailureKind::MetricsExportFailed,
        MetricsSinkFailureKind::MetricsBacklogBoundExceeded,
        MetricsSinkFailureKind::ObservabilitySignalInvalid,
        MetricsSinkFailureKind::MetricCardinalityExceeded,
        MetricsSinkFailureKind::TelemetrySamplingPolicyMissing,
        MetricsSinkFailureKind::DriverShutdown,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
        let failure = MetricsSinkFailure::from_kind(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            kind.reason_code()
        );
        assert_eq!(
            failure.resource_bound_decision().is_some(),
            matches!(kind, MetricsSinkFailureKind::MetricsBacklogBoundExceeded)
        );
    }

    for call_shape in [
        PortCallShape::Command,
        PortCallShape::Query,
        PortCallShape::SinkSubmit,
        PortCallShape::StreamObservation,
        PortCallShape::Scheduler,
    ] {
        let shape = PortContractShape::new(
            PortFamily::Runtime,
            call_shape,
            vec![
                PortOwnershipRule::CoreOwnedTypesOnly,
                PortOwnershipRule::NoDriverBufferOwnershipTransfer,
                PortOwnershipRule::NoConcreteRuntimeHandleTransfer,
            ],
        );
        assert_eq!(shape.family(), PortFamily::Runtime);
        assert_eq!(shape.call_shape(), call_shape);
        assert_eq!(shape.ownership_rules().len(), 3);
    }

    for class in [
        PortErrorClass::DriverFailure,
        PortErrorClass::BoundOrBackpressure,
        PortErrorClass::ConversionFailure,
        PortErrorClass::RuntimeOrShutdown,
        PortErrorClass::TokenVerification,
        PortErrorClass::Persistence,
    ] {
        let definition = find_reason_definition("driver_shutdown").expect("reason exists");
        let error = PortError::new(class, Reason::new(definition, Some("non-authoritative")));
        assert_eq!(error.class(), class);
        assert_eq!(
            error.reason().definition().code().as_str(),
            "driver_shutdown"
        );
        assert_eq!(error.reason().details(), Some(&"non-authoritative"));
    }

    let checkpoint = PersistencePortIntent::try_new(
        PersistenceIntentClass::StateCheckpoint,
        PersistenceOperationKind::PersistCheckpoint,
        StateFamily::SignalingIdempotency,
        StateClass::CheckpointEligibleState,
        vec![PersistenceConsistencyRequirement::PreserveIdempotency],
        Some(correlation("core-port-checkpoint")),
    )
    .expect("checkpoint state is eligible");
    assert_eq!(
        checkpoint.intent_class(),
        PersistenceIntentClass::StateCheckpoint
    );
    assert_eq!(
        checkpoint.state_class(),
        StateClass::CheckpointEligibleState
    );
    let load_checkpoint = PersistencePortIntent::try_new(
        PersistenceIntentClass::StateCheckpoint,
        PersistenceOperationKind::LoadCheckpoint,
        StateFamily::SignalingIdempotency,
        StateClass::CheckpointEligibleState,
        vec![PersistenceConsistencyRequirement::RetentionPolicy],
        None,
    )
    .expect("load checkpoint uses checkpoint class");
    assert_eq!(
        load_checkpoint.intent_class(),
        PersistenceIntentClass::StateCheckpoint
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::StateCheckpoint,
            PersistenceOperationKind::AppendAuditEvent,
            StateFamily::SignalingIdempotency,
            StateClass::CheckpointEligibleState,
            vec![],
            None,
        ),
        Err(PersistencePortIntentError::OperationClassMismatch)
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::StateCheckpoint,
            PersistenceOperationKind::PersistCheckpoint,
            StateFamily::AuditEvent,
            StateClass::AuditOnlyState,
            vec![],
            None,
        ),
        Err(PersistencePortIntentError::CheckpointRequiresCheckpointEligibleState)
    );

    let audit = PersistencePortIntent::try_new(
        PersistenceIntentClass::AuditPersistence,
        PersistenceOperationKind::AppendAuditEvent,
        StateFamily::AuditEvent,
        StateClass::AuditOnlyState,
        vec![PersistenceConsistencyRequirement::OrderedAppend],
        None,
    )
    .expect("audit event is audit-only");
    assert_eq!(
        audit.intent_class(),
        PersistenceIntentClass::AuditPersistence
    );
    let compensation_audit = PersistencePortIntent::try_new(
        PersistenceIntentClass::AuditPersistence,
        PersistenceOperationKind::AppendAuditEvent,
        StateFamily::AtomicityCompensationEvidence,
        StateClass::AuditOnlyState,
        vec![PersistenceConsistencyRequirement::OrderedAppend],
        None,
    )
    .expect("compensation state is audit persistence");
    assert_eq!(
        compensation_audit.intent_class(),
        PersistenceIntentClass::AuditPersistence
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::AuditPersistence,
            PersistenceOperationKind::AppendAuditEvent,
            StateFamily::SignalingIdempotency,
            StateClass::CheckpointEligibleState,
            vec![],
            None,
        ),
        Err(PersistencePortIntentError::AuditIntentRequiresAuditOnlyState)
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::AuditPersistence,
            PersistenceOperationKind::AppendAuditEvent,
            StateFamily::AuditHashChainRecord,
            StateClass::AuditOnlyState,
            vec![],
            None,
        ),
        Err(PersistencePortIntentError::AuditPersistenceRequiresAuditEventState)
    );

    let hash_chain = PersistencePortIntent::try_new(
        PersistenceIntentClass::HashChainRecordPersistence,
        PersistenceOperationKind::AppendHashChainRecord,
        StateFamily::AuditHashChainRecord,
        StateClass::AuditOnlyState,
        vec![PersistenceConsistencyRequirement::OrderedAppend],
        None,
    )
    .expect("hash-chain record is audit-only");
    assert_eq!(
        hash_chain.intent_class(),
        PersistenceIntentClass::HashChainRecordPersistence
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::HashChainRecordPersistence,
            PersistenceOperationKind::AppendAuditEvent,
            StateFamily::AuditHashChainRecord,
            StateClass::AuditOnlyState,
            vec![],
            None,
        ),
        Err(PersistencePortIntentError::OperationClassMismatch)
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::HashChainRecordPersistence,
            PersistenceOperationKind::AppendHashChainRecord,
            StateFamily::AuditEvent,
            StateClass::AuditOnlyState,
            vec![],
            None,
        ),
        Err(PersistencePortIntentError::HashChainPersistenceRequiresHashChainState)
    );

    for operation in [
        PersistenceOperationKind::EnqueueRetry,
        PersistenceOperationKind::DequeueRetry,
        PersistenceOperationKind::AcknowledgeRetry,
    ] {
        let retry = PersistencePortIntent::try_new(
            PersistenceIntentClass::RetryStore,
            operation,
            StateFamily::DriverRetryStore,
            StateClass::DriverLocalState,
            vec![PersistenceConsistencyRequirement::BoundedRetryStore],
            None,
        )
        .expect("retry store requires bounded retry");
        assert_eq!(retry.intent_class(), PersistenceIntentClass::RetryStore);
    }
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::RetryStore,
            PersistenceOperationKind::EnqueueRetry,
            StateFamily::DriverRetryStore,
            StateClass::DriverLocalState,
            vec![],
            None,
        ),
        Err(PersistencePortIntentError::RetryIntentRequiresBoundedRetry)
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::RetryStore,
            PersistenceOperationKind::EnqueueRetry,
            StateFamily::MetricsBacklog,
            StateClass::DriverLocalState,
            vec![PersistenceConsistencyRequirement::BoundedRetryStore],
            None,
        ),
        Err(PersistencePortIntentError::RetryIntentRequiresDriverRetryStore)
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::RetryStore,
            PersistenceOperationKind::AppendAuditEvent,
            StateFamily::DriverRetryStore,
            StateClass::DriverLocalState,
            vec![PersistenceConsistencyRequirement::BoundedRetryStore],
            None,
        ),
        Err(PersistencePortIntentError::OperationClassMismatch)
    );
    assert_eq!(
        PersistencePortIntent::try_new(
            PersistenceIntentClass::StateCheckpoint,
            PersistenceOperationKind::PersistCheckpoint,
            StateFamily::SignalingRoom,
            StateClass::CheckpointEligibleState,
            vec![],
            None,
        ),
        Err(PersistencePortIntentError::StateFamilyClassPolicyMismatch)
    );

    let record = PersistenceRecordRef::new(reference("core-port-record"));
    assert_eq!(record.as_str(), "core-port-record");
    let ack = PersistenceAcknowledgement::new(
        PersistenceIntentClass::AuditPersistence,
        Some(record.clone()),
        true,
    );
    let loaded = LoadedCoreStateRef::new(
        StateFamily::SignalingIdempotency,
        StateClass::CheckpointEligibleState,
        record,
    );
    let _input = PersistencePortInput::ExecuteIntent(checkpoint);
    let _outputs = [
        PersistencePortOutput::Acknowledgement(ack),
        PersistencePortOutput::LoadedState(loaded),
    ];

    for kind in [
        PersistencePortFailureKind::PersistenceUnavailable,
        PersistencePortFailureKind::PersistenceRetryBoundExceeded,
        PersistencePortFailureKind::PersistenceRetryDurationExceeded,
        PersistencePortFailureKind::AuditBacklogBoundExceeded,
        PersistencePortFailureKind::DriverShutdown,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
        let failure = PersistencePortFailure::from_kind(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            kind.reason_code()
        );
        assert_eq!(
            failure.resource_bound_decision().is_some(),
            matches!(
                kind,
                PersistencePortFailureKind::PersistenceRetryBoundExceeded
                    | PersistencePortFailureKind::PersistenceRetryDurationExceeded
                    | PersistencePortFailureKind::AuditBacklogBoundExceeded
            )
        );
    }

    let _extra_port_classes = [
        PersistenceStateClass::SourceOfTruth,
        PersistenceStateClass::Checkpoint,
        PersistenceStateClass::AuditOnly,
        PersistenceStateClass::DriverRetryData,
    ];
    let _packet_view = PacketViewClass::BorrowedSemanticHeaderView;
    let _runtime_outputs = [
        RuntimePortOutputClass::OpaqueScheduleReference,
        RuntimePortOutputClass::CancellationObservation,
        RuntimePortOutputClass::ExecutionObservation,
    ];
}

#[test]
fn coverage_core_protocol_versioning_and_compatibility_are_closed() {
    let data_classes = [
        CanonicalDataClass::AuditEventHashInput,
        CanonicalDataClass::CoreReason,
        CanonicalDataClass::CoreReference,
        CanonicalDataClass::EvidenceTimestamp,
        CanonicalDataClass::Duration,
        CanonicalDataClass::BinaryPayloadDigest,
    ];
    assert_eq!(data_classes.len(), 6);

    let defined_rules = CanonicalEncodingRuleSet::new(
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        UnknownFieldHandling::Reject,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
    );
    assert!(defined_rules.is_complete_for_canonical_encoding());
    let incomplete_rules = CanonicalEncodingRuleSet::new(
        CanonicalRuleStatus::Unspecified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
        UnknownFieldHandling::IgnoredOnlyWhenCompatibilityAllows,
        CanonicalRuleStatus::Specified,
        CanonicalRuleStatus::Specified,
    );
    assert!(!incomplete_rules.is_complete_for_canonical_encoding());

    let format_version = CanonicalFormatVersion::new("json-c14n", "1");
    assert_eq!(format_version.format(), "json-c14n");
    assert_eq!(format_version.version(), "1");
    let digest = CanonicalDigest::new(format_version, "sha256", vec![1, 2, 3])
        .expect("digest material is present");
    assert_eq!(digest.format_version(), format_version);
    assert_eq!(digest.algorithm(), "sha256");
    assert_eq!(digest.digest(), &[1, 2, 3]);
    assert_eq!(
        CanonicalDigest::new(format_version, "", vec![1]),
        Err(CanonicalEncodingError::EmptyDigestMaterial)
    );
    assert_eq!(
        CanonicalDigest::new(format_version, "sha256", vec![]),
        Err(CanonicalEncodingError::EmptyDigestMaterial)
    );

    for kind in [
        CanonicalEncodingFailureKind::CanonicalSerializationFailed,
        CanonicalEncodingFailureKind::CanonicalSerializationMismatch,
        CanonicalEncodingFailureKind::ExternalDecodeFailed,
        CanonicalEncodingFailureKind::ExternalEncodeFailed,
        CanonicalEncodingFailureKind::MissingRequiredWireField,
        CanonicalEncodingFailureKind::UnsupportedCanonicalVersion,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    for (surface, owner) in [
        (VersionedSurface::SignalingContract, VersionOwner::Core),
        (VersionedSurface::TurnContract, VersionOwner::Core),
        (VersionedSurface::SfuContract, VersionOwner::Core),
        (VersionedSurface::SdkPublicContract, VersionOwner::Sdk),
        (VersionedSurface::AuditSchema, VersionOwner::Core),
        (VersionedSurface::DriverWireEncoding, VersionOwner::Driver),
    ] {
        let ownership = VersionedSurfaceOwnership::new(surface, owner);
        assert_eq!(ownership.surface(), surface);
        assert_eq!(ownership.owner(), owner);
    }
    let requested = ContractVersion::new(2, 0, 0);
    let accepted = ContractVersion::new(2, 1, 0);
    assert_eq!(requested.major(), 2);
    let wire_version = WireEncodingVersion::new("driver-json-v1");
    assert_eq!(wire_version.code(), "driver-json-v1");
    let outcomes = [
        VersionNegotiationOutcome::AcceptedExact(requested),
        VersionNegotiationOutcome::AcceptedCompatible {
            requested,
            accepted,
        },
        VersionNegotiationOutcome::RejectedUnsupported(ContractVersion::new(3, 0, 0)),
    ];
    assert_eq!(outcomes.len(), 3);

    let capability = Capability::new("simulcast");
    assert_eq!(capability.name(), "simulcast");
    let _capability_rules = [
        CapabilityRule::OptionalWithinAcceptedVersion,
        CapabilityRule::MustNotAlterRequiredStateTransition,
    ];
    for (change, classification) in [
        (
            CompatibilityChange::AddOptionalFieldWithDefault,
            CompatibilityClassification::Compatible,
        ),
        (
            CompatibilityChange::AddRequiredField,
            CompatibilityClassification::Breaking,
        ),
        (
            CompatibilityChange::RemoveField,
            CompatibilityClassification::Breaking,
        ),
        (
            CompatibilityChange::ChangeReasonCodeSemantics,
            CompatibilityClassification::Prohibited,
        ),
        (
            CompatibilityChange::AddReasonCode,
            CompatibilityClassification::Conditional,
        ),
        (
            CompatibilityChange::ChangeStateTransition,
            CompatibilityClassification::Breaking,
        ),
        (
            CompatibilityChange::AddDriverEncoding,
            CompatibilityClassification::Compatible,
        ),
    ] {
        assert_eq!(change.classification(), classification);
    }

    let range = CompatibilityVersionRange::new(
        VersionedSurface::SignalingContract,
        ContractVersion::new(1, 0, 0),
        ContractVersion::new(1, 9, 9),
    );
    assert_eq!(range.surface(), VersionedSurface::SignalingContract);
    let lifecycle_steps = vec![
        DeprecationLifecycleStep::IdentifyAffectedSurfaceAndVersion,
        DeprecationLifecycleStep::DefineUnsupportedVersionBehavior,
        DeprecationLifecycleStep::UpdateSdkParityAndDriverMapping,
        DeprecationLifecycleStep::RemoveAfterCompatibilityWindow,
    ];
    let deprecation = DeprecationDecision::new(
        VersionedSurface::SignalingContract,
        ContractVersion::new(1, 0, 0),
        VersionOwner::Core,
        range,
        lifecycle_steps,
    );
    assert_eq!(deprecation.lifecycle_steps().len(), 4);
    for kind in [
        CompatibilityFailureKind::UnsupportedCommandVersion,
        CompatibilityFailureKind::UnsupportedMediaContractVersion,
        CompatibilityFailureKind::UnsupportedTurnContractVersion,
        CompatibilityFailureKind::UnsupportedDriverWireVersion,
        CompatibilityFailureKind::MissingRequiredWireField,
        CompatibilityFailureKind::ExternalEnumUnmapped,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
}

#[test]
fn coverage_core_quality_resource_and_backpressure_rules_are_closed() {
    let _metrics = [
        QualityMetric::new(QualityMetricKind::Rtt, 10, "ms", "p50"),
        QualityMetric::new(QualityMetricKind::PacketLoss, 1, "percent", "p95"),
        QualityMetric::new(QualityMetricKind::Jitter, 4, "ms", "p95"),
        QualityMetric::new(QualityMetricKind::Bitrate, 1200, "kbps", "avg"),
        QualityMetric::new(QualityMetricKind::FrameRate, 30, "fps", "avg"),
        QualityMetric::new(QualityMetricKind::RelayLatency, 20, "ms", "p95"),
        QualityMetric::new(QualityMetricKind::QueueDepth, 3, "items", "max"),
        QualityMetric::new(QualityMetricKind::BackpressureState, 1, "state", "instant"),
        QualityMetric::new(QualityMetricKind::RouteHealth, 99, "score", "instant"),
        QualityMetric::new(QualityMetricKind::MosLikeScore, 45, "score", "p50"),
    ];
    let _threshold = QualityThreshold::new(QualityMetricKind::PacketLoss, 3, "percent");
    let _decisions = [
        QualityDecisionKind::Normal,
        QualityDecisionKind::Degraded,
        QualityDecisionKind::Violation,
        QualityDecisionKind::RouteSuppression,
        QualityDecisionKind::BackpressureAction,
        QualityDecisionKind::RecoveryAllowed,
        QualityDecisionKind::AdmissionRejected,
    ];
    let _targets = [
        QualityTargetRef::Endpoint(EndpointId::new(reference("quality-endpoint"))),
        QualityTargetRef::Route(RouteId::new(reference("quality-route"))),
        QualityTargetRef::Stream(StreamId::new(reference("quality-stream"))),
        QualityTargetRef::Packet(PacketId::new(reference("quality-packet"))),
    ];
    for kind in [
        QualityFailureKind::PacketSuppressedByQuality,
        QualityFailureKind::RouteSuppressedByQuality,
        QualityFailureKind::QualityRecoveryNotAllowed,
        QualityFailureKind::EndpointQualityNotAllowed,
        QualityFailureKind::EndpointDegradedByQuality,
        QualityFailureKind::PublicationQualityNotAllowed,
        QualityFailureKind::SubscriptionQualityNotAllowed,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    let _admission_scopes = [
        AdmissionScope::Global,
        AdmissionScope::Room,
        AdmissionScope::Session,
        AdmissionScope::Allocation,
        AdmissionScope::CredentialRef,
        AdmissionScope::DriverConnectionRef,
    ];
    let _admission_targets = [
        AdmissionTarget::RoomMaterialization,
        AdmissionTarget::ParticipantJoin,
        AdmissionTarget::SignalingCommandQueue,
        AdmissionTarget::SfuEndpointAdmission,
        AdmissionTarget::SfuRouteCandidateConstruction,
        AdmissionTarget::TurnAllocation,
        AdmissionTarget::TurnPermission,
        AdmissionTarget::ConnectionConcurrency,
        AdmissionTarget::InboundFrame,
    ];
    let _rate_window =
        RateWindowPolicy::new(AdmissionScope::Room, Some(10), None, Some(1000), "sliding");
    let _admission_decisions = [
        AdmissionDecisionKind::Accepted,
        AdmissionDecisionKind::Rejected,
        AdmissionDecisionKind::Dropped,
        AdmissionDecisionKind::Shed,
        AdmissionDecisionKind::Expired,
    ];
    for kind in [
        AdmissionFailureKind::RoomCapacityExceeded,
        AdmissionFailureKind::AdmissionCapacityExceeded,
        AdmissionFailureKind::SignalingCommandQueueBoundExceeded,
        AdmissionFailureKind::EndpointCapacityExceeded,
        AdmissionFailureKind::RouteCandidateBoundExceeded,
        AdmissionFailureKind::AllocationCapacityExceeded,
        AdmissionFailureKind::PermissionCapacityExceeded,
        AdmissionFailureKind::ConnectionConcurrencyExceeded,
        AdmissionFailureKind::FrameSizeBoundExceeded,
        AdmissionFailureKind::DriverShutdown,
        AdmissionFailureKind::RuntimeConfigMissing,
        AdmissionFailureKind::RuntimeConfigInvalid,
        AdmissionFailureKind::PersistenceUnavailable,
        AdmissionFailureKind::AuthorizationContextMissing,
        AdmissionFailureKind::AuthorizationScopeNotAllowed,
        AdmissionFailureKind::ClientAddressUntrusted,
        AdmissionFailureKind::ForwardedHeaderUntrusted,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    for bound in REQUIRED_RESOURCE_BOUNDS {
        assert!(!format!("{:?}", bound).is_empty());
    }
    for action in REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS {
        assert!(!action.reason_code().is_empty());
        assert_reason_is_cataloged(action.reason_code());
        assert!(!action.audit_event_type().code().is_empty());
        assert!(!action.outcome().code().is_empty());
        assert!(!format!("{:?}", action.action()).is_empty());
        assert!(!format!("{:?}", action.owner_tuple()).is_empty());
    }
    for audit_event in [
        ResourceBoundAuditEventType::BackpressureDecision,
        ResourceBoundAuditEventType::SfuSubscriptionDecision,
        ResourceBoundAuditEventType::ResourceBoundDecision,
        ResourceBoundAuditEventType::DriverResourceBoundDecision,
        ResourceBoundAuditEventType::TurnAllocationDecision,
        ResourceBoundAuditEventType::TurnRefreshDecision,
        ResourceBoundAuditEventType::TurnPermissionDecision,
        ResourceBoundAuditEventType::TurnChannelBindDecision,
        ResourceBoundAuditEventType::RuntimeTaskLifecycleDecision,
    ] {
        assert!(!audit_event.code().is_empty());
    }
    for outcome in [
        ResourceBoundOutcome::Accepted,
        ResourceBoundOutcome::WithinBoundObserved,
        ResourceBoundOutcome::Rejected,
        ResourceBoundOutcome::Dropped,
        ResourceBoundOutcome::Shed,
        ResourceBoundOutcome::Expired,
    ] {
        assert!(!outcome.code().is_empty());
    }

    let action_by_reason = |reason_code: &str| {
        REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
            .iter()
            .find(|action| action.reason_code() == reason_code)
            .copied()
            .expect("closed action exists")
    };
    assert_eq!(
        ResourceBoundDecision::try_new(
            action_by_reason("turn_relay_queue_bound_exceeded"),
            ResourceBoundReferenceSet::none(),
        ),
        Err(ResourceBoundDecisionShapeError::TurnRelayQueueReferencesMissing)
    );
    assert!(ResourceBoundDecision::try_new(
        action_by_reason("turn_relay_queue_bound_exceeded"),
        ResourceBoundReferenceSet::turn_relay_queue(
            AllocationId::new(reference("quality-allocation")),
            PermissionId::new(reference("quality-permission")),
        ),
    )
    .is_ok());
    assert_eq!(
        ResourceBoundDecision::try_new(
            action_by_reason("packet_cache_bound_exceeded"),
            ResourceBoundReferenceSet::none(),
        ),
        Err(ResourceBoundDecisionShapeError::PacketReferenceMissing)
    );
    assert!(ResourceBoundDecision::try_new(
        action_by_reason("packet_cache_bound_exceeded"),
        ResourceBoundReferenceSet::packet(PacketId::new(reference("quality-cache-packet"))),
    )
    .is_ok());
    assert_eq!(
        ResourceBoundDecision::try_new(
            action_by_reason("frame_size_bound_exceeded"),
            ResourceBoundReferenceSet::none(),
        ),
        Err(ResourceBoundDecisionShapeError::DriverResourceReferenceMissing)
    );
    assert!(ResourceBoundDecision::try_new(
        action_by_reason("frame_size_bound_exceeded"),
        ResourceBoundReferenceSet::driver_resource("websocket-frame"),
    )
    .is_ok());
    assert!(ResourceBoundDecision::try_new(
        action_by_reason("endpoint_capacity_exceeded"),
        ResourceBoundReferenceSet::endpoint(EndpointId::new(reference("quality-cap-endpoint"))),
    )
    .is_ok());
    assert!(ResourceBoundDecision::try_new(
        action_by_reason("route_candidate_bound_exceeded"),
        ResourceBoundReferenceSet::route(RouteId::new(reference("quality-cap-route"))),
    )
    .is_ok());

    for kind in [
        BackpressureDecisionKind::Accept,
        BackpressureDecisionKind::DelayAction,
        BackpressureDecisionKind::SuppressForwarding,
        BackpressureDecisionKind::SuppressSubscription,
        BackpressureDecisionKind::SuppressRouteState,
        BackpressureDecisionKind::DropPacket,
        BackpressureDecisionKind::StopRetainingPacketByPressurePolicy,
        BackpressureDecisionKind::DegradeRoute,
        BackpressureDecisionKind::CloseEndpoint,
        BackpressureDecisionKind::RejectRecoveryFromBackpressureState,
    ] {
        assert!(!kind.audit_event_type().code().is_empty());
        assert!(!kind.outcome().code().is_empty());
        if let Some(code) = kind.reason_code() {
            assert_reason_is_cataloged(code);
        }
    }
    assert!(BackpressureDecision::try_new(
        BackpressureDecisionKind::Accept,
        BackpressureTargetRef::NotApplicable,
    )
    .is_ok());
    assert_eq!(
        BackpressureDecision::try_new(
            BackpressureDecisionKind::Accept,
            BackpressureTargetRef::Packet(PacketId::new(reference("bp-wrong-packet"))),
        ),
        Err(BackpressureDecisionShapeError::AcceptTargetMustBeNotApplicable)
    );
    assert!(BackpressureDecision::try_new(
        BackpressureDecisionKind::DelayAction,
        BackpressureTargetRef::Route(RouteId::new(reference("bp-route"))),
    )
    .is_ok());
    assert_eq!(
        BackpressureDecision::try_new(
            BackpressureDecisionKind::DelayAction,
            BackpressureTargetRef::NotApplicable,
        ),
        Err(BackpressureDecisionShapeError::RouteTargetRequired)
    );
    assert!(BackpressureDecision::try_new(
        BackpressureDecisionKind::SuppressForwarding,
        BackpressureTargetRef::Packet(PacketId::new(reference("bp-packet"))),
    )
    .is_ok());
    assert_eq!(
        BackpressureDecision::try_new(
            BackpressureDecisionKind::DropPacket,
            BackpressureTargetRef::NotApplicable,
        ),
        Err(BackpressureDecisionShapeError::PacketTargetRequired)
    );
    assert!(BackpressureDecision::try_new(
        BackpressureDecisionKind::SuppressSubscription,
        BackpressureTargetRef::Subscription(StreamId::new(reference("bp-stream"))),
    )
    .is_ok());
    assert_eq!(
        BackpressureDecision::try_new(
            BackpressureDecisionKind::SuppressSubscription,
            BackpressureTargetRef::NotApplicable,
        ),
        Err(BackpressureDecisionShapeError::SubscriptionTargetRequired)
    );
    assert!(BackpressureDecision::try_new(
        BackpressureDecisionKind::CloseEndpoint,
        BackpressureTargetRef::Endpoint(EndpointId::new(reference("bp-endpoint"))),
    )
    .is_ok());
    assert_eq!(
        BackpressureDecision::try_new(
            BackpressureDecisionKind::CloseEndpoint,
            BackpressureTargetRef::NotApplicable,
        ),
        Err(BackpressureDecisionShapeError::EndpointTargetRequired)
    );

    let _audit_backlog_rules = [
        AuditBacklogOverflowRule::EmitSingleReservedOverflowRecord,
        AuditBacklogOverflowRule::RejectNewAuditRequiredPath,
        AuditBacklogOverflowRule::RejectFailedAuditOutcomeAsSuccess,
    ];
}
