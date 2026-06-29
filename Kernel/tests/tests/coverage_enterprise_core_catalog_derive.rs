use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use arcrtc_core_identity::{
    AllocationId, AuditEventId, ConfigurationScopeRef, CorrelationId, EndpointId, OpaqueReference,
    PacketId, ParticipantId, PermissionId, ReferenceAuthority, RoomId, RouteId, SessionId,
    StartupRunId, StreamId,
};
use arcrtc_core_operation as operation;
use arcrtc_core_quality as quality;
use arcrtc_core_reason::CatalogedReasonRef;

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("reference is valid")
}

fn correlation(value: &str) -> CorrelationId {
    CorrelationId::new(reference(value))
}

fn room(value: &str) -> RoomId {
    RoomId::new(reference(value))
}

fn participant(value: &str) -> ParticipantId {
    ParticipantId::new(reference(value))
}

fn session(value: &str) -> SessionId {
    SessionId::new(reference(value))
}

fn endpoint(value: &str) -> EndpointId {
    EndpointId::new(reference(value))
}

fn stream(value: &str) -> StreamId {
    StreamId::new(reference(value))
}

fn route(value: &str) -> RouteId {
    RouteId::new(reference(value))
}

fn packet(value: &str) -> PacketId {
    PacketId::new(reference(value))
}

fn allocation(value: &str) -> AllocationId {
    AllocationId::new(reference(value))
}

fn permission(value: &str) -> PermissionId {
    PermissionId::new(reference(value))
}

fn audit_event(value: &str) -> AuditEventId {
    AuditEventId::new(reference(value))
}

fn config_scope(value: &str) -> ConfigurationScopeRef {
    ConfigurationScopeRef::new(reference(value))
}

fn startup(value: &str) -> StartupRunId {
    StartupRunId::new(reference(value))
}

fn touch_hash<T>(value: T)
where
    T: Clone + Debug + Eq + Hash,
{
    let cloned = value.clone();
    assert_eq!(cloned, value);
    assert!(!format!("{value:?}").is_empty());
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let _ = hasher.finish();
}

fn touch_eq<T>(value: T)
where
    T: Clone + Debug + Eq,
{
    let cloned = value.clone();
    assert_eq!(cloned, value);
    assert!(!format!("{value:?}").is_empty());
}

#[test]
fn operation_catalog_derive_and_table_rows_are_executed() {
    // closed vocabulary は Debug だけでなく Clone/Eq/Hash 経路も通し、derive 行の未到達を潰します。
    touch_eq(operation::CoreOperationSurface);

    for owner in [
        operation::ShutdownDrainOwner::Entrypoints,
        operation::ShutdownDrainOwner::InternalControlPlaneContract,
        operation::ShutdownDrainOwner::CoreSignaling,
        operation::ShutdownDrainOwner::CoreSfu,
        operation::ShutdownDrainOwner::CoreTurn,
        operation::ShutdownDrainOwner::Driver,
        operation::ShutdownDrainOwner::DriverEntrypointsRuntime,
        operation::ShutdownDrainOwner::EntrypointsDriverObservation,
    ] {
        touch_hash(owner);
    }

    for concern in [
        operation::ShutdownDrainConcern::ProcessSignalObservation,
        operation::ShutdownDrainConcern::SplitServiceShutdownControl,
        operation::ShutdownDrainConcern::ShutdownModeSelection,
        operation::ShutdownDrainConcern::RoomDrainCloseSemantics,
        operation::ShutdownDrainConcern::SfuSessionEndpointLifecycleSemantics,
        operation::ShutdownDrainConcern::TurnAllocationPermissionRelaySemantics,
        operation::ShutdownDrainConcern::SocketListenerStop,
        operation::ShutdownDrainConcern::RuntimeTaskWorkerStop,
        operation::ShutdownDrainConcern::PacketBufferRelease,
        operation::ShutdownDrainConcern::AuditPersistenceMetricsFlush,
        operation::ShutdownDrainConcern::UncleanProcessTerminationObservation,
    ] {
        touch_hash(concern);
        touch_hash(concern.owner());
    }

    for step in [
        operation::DrainSequenceStep::CreateShutdownCorrelationAndRequest,
        operation::DrainSequenceStep::StopNewExternalAdmission,
        operation::DrainSequenceStep::BeginSignalingRoomDrain,
        operation::DrainSequenceStep::DrainSfuSessionsEndpoints,
        operation::DrainSequenceStep::StopTurnAllocationPermissionPaths,
        operation::DrainSequenceStep::StopDriverReceiveLoopsAndBufferLeases,
        operation::DrainSequenceStep::CancelOrJoinRuntimeTasks,
        operation::DrainSequenceStep::DrainBoundedQueues,
        operation::DrainSequenceStep::ReleaseBuffersRelayResourcesSockets,
        operation::DrainSequenceStep::StopRuntimeAfterEvidencePathAttempt,
    ] {
        touch_hash(step);
        assert!((1..=10).contains(&step.order()));
    }

    for plane in [
        operation::ShutdownDrainPlane::Signaling,
        operation::ShutdownDrainPlane::Sfu,
        operation::ShutdownDrainPlane::Turn,
        operation::ShutdownDrainPlane::NetworkDriver,
        operation::ShutdownDrainPlane::PersistenceAuditMetricsDriver,
        operation::ShutdownDrainPlane::RuntimeTaskWorker,
    ] {
        touch_hash(plane);
    }

    for mode in [
        operation::ShutdownDrainMode::GracefulDrain,
        operation::ShutdownDrainMode::ReconfigurationDrain,
        operation::ShutdownDrainMode::AdmissionStopOnly,
    ] {
        touch_hash(mode);
    }

    for outcome in [
        operation::ShutdownDrainOutcome::Accepted,
        operation::ShutdownDrainOutcome::Rejected,
        operation::ShutdownDrainOutcome::Failed,
        operation::ShutdownDrainOutcome::Drained,
        operation::ShutdownDrainOutcome::UncleanTerminationObserved,
    ] {
        touch_hash(outcome);
    }

    for failure in [
        operation::ShutdownDrainFailureKind::RoomDraining,
        operation::ShutdownDrainFailureKind::RoomClosed,
        operation::ShutdownDrainFailureKind::RoomCloseNotAllowed,
        operation::ShutdownDrainFailureKind::SfuSessionNotAccepting,
        operation::ShutdownDrainFailureKind::EndpointClosedByBackpressure,
        operation::ShutdownDrainFailureKind::DriverShutdown,
        operation::ShutdownDrainFailureKind::NetworkReceiveFailed,
        operation::ShutdownDrainFailureKind::NetworkSendFailed,
        operation::ShutdownDrainFailureKind::PersistenceUnavailable,
        operation::ShutdownDrainFailureKind::AuditBacklogBoundExceeded,
        operation::ShutdownDrainFailureKind::MetricsExportFailed,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    for rule in operation::PLANE_DRAIN_RULES {
        touch_hash(*rule);
        assert!(format!("{rule:?}").contains("required_failure_reasons"));
    }

    touch_hash(operation::ShutdownDrainEvidenceShape::new(
        correlation("operation-drain-derived"),
        Some(startup("operation-startup-derived")),
        operation::ShutdownDrainMode::GracefulDrain,
        Some(config_scope("operation-config-derived")),
        operation::ShutdownDrainPlane::NetworkDriver,
        operation::ShutdownDrainOutcome::Failed,
        Some(operation::ShutdownDrainFailureKind::NetworkSendFailed),
        true,
        true,
        Some(1),
    ));

    for prohibited in [
        operation::ProhibitedShutdownDrainBehavior::EntrypointsDirectDomainStateMutation,
        operation::ProhibitedShutdownDrainBehavior::DriverSocketCloseAsDomainSuccess,
        operation::ProhibitedShutdownDrainBehavior::FailureHiddenBehindProcessExitCode,
        operation::ProhibitedShutdownDrainBehavior::UncleanTerminationAsGracefulDrain,
        operation::ProhibitedShutdownDrainBehavior::UnboundedDrainOrFlush,
        operation::ProhibitedShutdownDrainBehavior::FailedFlushAsCloseoutEvidence,
        operation::ProhibitedShutdownDrainBehavior::CrossPlaneDrainSuccessInference,
        operation::ProhibitedShutdownDrainBehavior::SplitServiceDrainWithoutControlPlaneEvidence,
        operation::ProhibitedShutdownDrainBehavior::ReconfigurationBeforeRequiredDrainRestart,
        operation::ProhibitedShutdownDrainBehavior::DetachedWorkerDuringGracefulDrainClaim,
    ] {
        touch_hash(prohibited);
    }
}

#[test]
fn operation_atomicity_ordering_retry_derive_paths_are_executed() {
    for owner in [
        operation::AtomicityOwner::Core,
        operation::AtomicityOwner::Driver,
        operation::AtomicityOwner::DriverSdk,
        operation::AtomicityOwner::CoreDomainCompensation,
        operation::AtomicityOwner::DriverExternalResourceCompensation,
        operation::AtomicityOwner::Reports,
    ] {
        touch_hash(owner);
    }

    for concern in [
        operation::AtomicityConcern::DomainDecisionAtomicity,
        operation::AtomicityConcern::PortIntentEmission,
        operation::AtomicityConcern::ConcreteDbTransaction,
        operation::AtomicityConcern::AuditPersistenceExecution,
        operation::AtomicityConcern::ExternalResponseEmission,
        operation::AtomicityConcern::CompensationDecision,
        operation::AtomicityConcern::EvidenceAdoption,
    ] {
        touch_hash(concern);
        touch_hash(concern.owner());
    }

    for class in [
        operation::AtomicityClass::SingleCoreDecision,
        operation::AtomicityClass::CoreDecisionPlusAuditRequired,
        operation::AtomicityClass::CoreDecisionPlusPortIntent,
        operation::AtomicityClass::DriverLocalTransaction,
        operation::AtomicityClass::CompensatingTransitionRequired,
        operation::AtomicityClass::NonCompensableObservation,
    ] {
        touch_hash(class);
    }

    for boundary in [
        operation::CommitBoundary::BeforeCoreEntry,
        operation::CommitBoundary::AfterCoreValidationBeforeAggregateMutation,
        operation::CommitBoundary::AfterAggregateTransition,
        operation::CommitBoundary::AfterAuditProjection,
        operation::CommitBoundary::AfterDriverPersistence,
        operation::CommitBoundary::AfterExternalResponseEmission,
    ] {
        touch_hash(boundary);
        assert!((1..=6).contains(&boundary.order()));
    }

    for outcome in [
        operation::AtomicStepOutcome::Accepted,
        operation::AtomicStepOutcome::Failed,
        operation::AtomicStepOutcome::NotApplicable,
        operation::AtomicStepOutcome::CloseNotClaimed,
    ] {
        touch_hash(outcome);
    }

    for failure in [
        operation::AtomicityFailureKind::AtomicCommitFailed,
        operation::AtomicityFailureKind::CompensationRequired,
        operation::AtomicityFailureKind::CompensationFailed,
        operation::AtomicityFailureKind::PersistenceUnavailable,
        operation::AtomicityFailureKind::AuditBacklogBoundExceeded,
        operation::AtomicityFailureKind::ExternalEncodeFailed,
        operation::AtomicityFailureKind::NetworkSendFailed,
        operation::AtomicityFailureKind::DriverShutdown,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    let commit_step = operation::AtomicCommitStep::new(
        operation::CommitBoundary::AfterDriverPersistence,
        operation::AtomicStepOutcome::Failed,
        Some(operation::AtomicityFailureKind::PersistenceUnavailable),
    );
    touch_hash(commit_step);

    for owner in [
        operation::CompensationOwner::CoreWhenDomainStateChanges,
        operation::CompensationOwner::DriverForExternalResourceCleanup,
    ] {
        touch_hash(owner);
    }
    for rule in [
        operation::CompensationExternalResponseRule::PreserveOriginalAcceptedDecision,
        operation::CompensationExternalResponseRule::ProjectLaterFailure,
        operation::CompensationExternalResponseRule::NoExternalResponse,
    ] {
        touch_hash(rule);
    }
    for rule in [
        operation::CompensationEvidenceAdoptionRule::RequireCompensationResultBeforeCloseEvidence,
        operation::CompensationEvidenceAdoptionRule::DriverObservationOrEvidenceLimitation,
        operation::CompensationEvidenceAdoptionRule::CloseNotClaimed,
    ] {
        touch_hash(rule);
    }
    for status in [
        operation::CompensationStatus::NotRequired,
        operation::CompensationStatus::RequiredAndAvailable,
        operation::CompensationStatus::RequiredButUnavailable,
        operation::CompensationStatus::Executed,
        operation::CompensationStatus::ExecutionFailed,
    ] {
        touch_hash(status);
    }

    touch_hash(operation::CompensationRule::new(
        operation::AtomicityFailureKind::CompensationRequired,
        "route transition",
        operation::CompensationOwner::CoreWhenDomainStateChanges,
        "mark route close-not-claimed",
        "atomicity_compensation_decision",
        operation::CompensationExternalResponseRule::ProjectLaterFailure,
        operation::CompensationEvidenceAdoptionRule::RequireCompensationResultBeforeCloseEvidence,
    ));
    let compensation = operation::AtomicityCompensationDecision::new(
        correlation("operation-atomic-derived"),
        operation::AtomicityClass::CompensatingTransitionRequired,
        operation::CommitBoundary::AfterDriverPersistence,
        commit_step,
        operation::CompensationStatus::RequiredAndAvailable,
    );
    touch_hash(compensation.clone());
    assert_eq!(
        compensation.audit_event_type(),
        "atomicity_compensation_decision"
    );

    for class in [
        operation::PartialSuccessEvidenceClass::ClassifiedPerStep,
        operation::PartialSuccessEvidenceClass::MissingClassificationNotAdoptable,
        operation::PartialSuccessEvidenceClass::CloseNotClaimedForFailedStep,
    ] {
        touch_hash(class);
    }
    for prohibited in [
        operation::ProhibitedAtomicityBehavior::DriverTransactionDefinesDomainInvariant,
        operation::ProhibitedAtomicityBehavior::PortIntentAsDriverExecutionSuccess,
        operation::ProhibitedAtomicityBehavior::ExternalResponseAsAuditPersistenceSuccess,
        operation::ProhibitedAtomicityBehavior::CompensationWithoutCoreStateMachineRule,
        operation::ProhibitedAtomicityBehavior::PartialSuccessHiddenBehindFinalSuccess,
        operation::ProhibitedAtomicityBehavior::FailedAuditPersistenceAsCloseEvidence,
    ] {
        touch_hash(prohibited);
    }
}

#[test]
fn operation_ordering_and_retry_catalogs_are_hash_executed() {
    for concern in [
        operation::OrderingConcern::DomainOrderingValidity,
        operation::OrderingConcern::AggregateSerializationScope,
        operation::OrderingConcern::PhysicalLockOrMailbox,
        operation::OrderingConcern::RuntimeScheduling,
        operation::OrderingConcern::RuntimeTaskLifecycle,
        operation::OrderingConcern::InboundWireOrderingObservation,
        operation::OrderingConcern::Idempotency,
    ] {
        touch_hash(concern);
        touch_hash(concern.owner());
    }

    for scope in [
        operation::SerializationScope::RoomScope,
        operation::SerializationScope::ParticipantScope,
        operation::SerializationScope::SfuSessionScope,
        operation::SerializationScope::SfuEndpointScope,
        operation::SerializationScope::PacketLifecycleScope,
        operation::SerializationScope::TurnAllocationScope,
        operation::SerializationScope::TurnPermissionScope,
        operation::SerializationScope::AuditChainScope,
        operation::SerializationScope::ConfigurationScope,
    ] {
        touch_hash(scope);
        touch_hash(scope.owner());
        assert!(!scope.protected_semantics().is_empty());
    }

    for reference in [
        operation::SerializationScopeReference::Room(room("ordering-room")),
        operation::SerializationScopeReference::Participant(participant("ordering-participant")),
        operation::SerializationScopeReference::SfuSession(session("ordering-session")),
        operation::SerializationScopeReference::SfuEndpoint(endpoint("ordering-endpoint")),
        operation::SerializationScopeReference::Packet(packet("ordering-packet")),
        operation::SerializationScopeReference::TurnAllocation(allocation("ordering-allocation")),
        operation::SerializationScopeReference::TurnPermission(permission("ordering-permission")),
        operation::SerializationScopeReference::AuditChain(audit_event("ordering-audit")),
        operation::SerializationScopeReference::Configuration(config_scope("ordering-config")),
    ] {
        touch_hash(reference);
    }

    for failure in [
        operation::OrderingFailureKind::CommandOrderViolation,
        operation::OrderingFailureKind::DuplicateCommand,
        operation::OrderingFailureKind::ConcurrencyConflict,
        operation::OrderingFailureKind::LockContentionBoundExceeded,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    for lock in [
        operation::PhysicalLockResource::AggregateMutationGuard,
        operation::PhysicalLockResource::DriverQueueMutex,
        operation::PhysicalLockResource::EntrypointLevelProcessLock,
        operation::PhysicalLockResource::TestHarnessSynchronization,
    ] {
        touch_hash(lock);
        touch_hash(lock.allowed_owner());
    }

    touch_hash(operation::SerializationBoundPolicy::new(
        operation::SerializationScope::RoomScope,
        true,
        true,
        operation::OrderingOwner::CoreSignaling,
        "cancel with prior decision preserved",
    ));
    touch_hash(operation::OrderingDecision::new(
        correlation("ordering-derived"),
        operation::SerializationScope::RoomScope,
        Some(operation::SerializationScopeReference::Room(room(
            "ordering-decision-room",
        ))),
        operation::AtomicStepOutcome::Failed,
        Some(operation::OrderingFailureKind::LockContentionBoundExceeded),
    ));

    for prohibited in [
        operation::ProhibitedOrderingLockBehavior::DriverLockOrderDefinesDomainOrder,
        operation::ProhibitedOrderingLockBehavior::RuntimeSchedulerOrderAsStateMachineAuthority,
        operation::ProhibitedOrderingLockBehavior::UnboundedAggregateCommandQueue,
        operation::ProhibitedOrderingLockBehavior::LockObjectInDomainOrPublicApi,
        operation::ProhibitedOrderingLockBehavior::LastWriterWinsWithoutCoreDecision,
        operation::ProhibitedOrderingLockBehavior::TestSynchronizationAsProductionSemantics,
        operation::ProhibitedOrderingLockBehavior::TaskSchedulingAsSerializationAuthority,
    ] {
        touch_hash(prohibited);
    }

    for concern in [
        operation::RetryTimeoutConcern::RetryabilityMetadata,
        operation::RetryTimeoutConcern::DomainCommandIdempotency,
        operation::RetryTimeoutConcern::DriverOperationRetryExecution,
        operation::RetryTimeoutConcern::CommandDeadlinePolicy,
        operation::RetryTimeoutConcern::TimerSchedulerExecution,
        operation::RetryTimeoutConcern::ClientCancellationObservation,
        operation::RetryTimeoutConcern::EntrypointShutdownCancellation,
    ] {
        touch_hash(concern);
        touch_hash(concern.owner());
    }

    for retry_class in [
        operation::RetryClass::NoRetry,
        operation::RetryClass::IdempotentCommandReplay,
        operation::RetryClass::DriverTransportRetry,
        operation::RetryClass::PersistenceRetry,
        operation::RetryClass::AuditSinkRetry,
        operation::RetryClass::TestOnlyRetry,
    ] {
        touch_hash(retry_class);
        touch_hash(retry_class.owner());
        assert_eq!(
            retry_class.runtime_evidence_allowed(),
            !matches!(retry_class, operation::RetryClass::TestOnlyRetry)
        );
    }

    for surface in [
        operation::TimeoutDeadlineSurface::CommandDeadline,
        operation::TimeoutDeadlineSurface::DriverSendTimeout,
        operation::TimeoutDeadlineSurface::DriverReceiveTimeout,
        operation::TimeoutDeadlineSurface::PersistenceRetryDuration,
        operation::TimeoutDeadlineSurface::PacketCacheRetentionDuration,
        operation::TimeoutDeadlineSurface::RuntimeShutdownDuringScheduledAction,
    ] {
        touch_hash(surface);
        touch_hash(surface.owner());
        assert!(CatalogedReasonRef::from_code(surface.failure_reason().reason_code()).is_ok());
    }

    for source in [
        operation::CancellationSource::ClientBeforeDriverCoreBoundary,
        operation::CancellationSource::ClientAfterCoreEntry,
        operation::CancellationSource::EntrypointShutdown,
        operation::CancellationSource::RuntimeTaskCancelled,
        operation::CancellationSource::TestHarnessCancelled,
    ] {
        touch_hash(source);
        touch_hash(source.handling());
    }

    let retry_preconditions =
        operation::RetryPreconditions::new(true, true, true, true, true, true, true);
    touch_hash(retry_preconditions);
    assert!(retry_preconditions.allows_retry());
    let retry_attempt = operation::RetryAttemptShape::<RoomId, ParticipantId>::new(
        operation::RetryClass::IdempotentCommandReplay,
        correlation("retry-attempt-derived"),
        None,
        1,
        retry_preconditions,
    );
    touch_eq(retry_attempt);

    for prohibited in [
        operation::ProhibitedRetryTimeoutCancellationBehavior::AutomaticRetryOfNonIdempotentDomainCommand,
        operation::ProhibitedRetryTimeoutCancellationBehavior::DriverRetryChangesDomainDecision,
        operation::ProhibitedRetryTimeoutCancellationBehavior::CancellationAsAcceptedDomainTransition,
        operation::ProhibitedRetryTimeoutCancellationBehavior::TimeoutAsGenericSuccessOrFreeTextFailure,
        operation::ProhibitedRetryTimeoutCancellationBehavior::UnboundedRetry,
        operation::ProhibitedRetryTimeoutCancellationBehavior::TestOnlyRetryAsRuntimeEvidence,
        operation::ProhibitedRetryTimeoutCancellationBehavior::RetryErasesOriginalCorrelationOrFirstAttemptEvidence,
        operation::ProhibitedRetryTimeoutCancellationBehavior::RuntimeTaskCancellationAsDomainSuccess,
    ] {
        touch_hash(prohibited);
    }
}

#[test]
fn quality_catalogs_resource_tables_and_backpressure_are_executed() {
    touch_eq(quality::CoreQualitySurface);

    for kind in [
        quality::QualityMetricKind::Rtt,
        quality::QualityMetricKind::PacketLoss,
        quality::QualityMetricKind::Jitter,
        quality::QualityMetricKind::Bitrate,
        quality::QualityMetricKind::FrameRate,
        quality::QualityMetricKind::RelayLatency,
        quality::QualityMetricKind::QueueDepth,
        quality::QualityMetricKind::BackpressureState,
        quality::QualityMetricKind::RouteHealth,
        quality::QualityMetricKind::MosLikeScore,
    ] {
        touch_hash(kind);
        touch_hash(quality::QualityMetric::new(kind, 10, "unit", "window"));
        touch_hash(quality::QualityThreshold::new(kind, 20, "unit"));
    }

    for decision in [
        quality::QualityDecisionKind::Normal,
        quality::QualityDecisionKind::Degraded,
        quality::QualityDecisionKind::Violation,
        quality::QualityDecisionKind::RouteSuppression,
        quality::QualityDecisionKind::BackpressureAction,
        quality::QualityDecisionKind::RecoveryAllowed,
        quality::QualityDecisionKind::AdmissionRejected,
    ] {
        touch_hash(decision);
    }

    for target in [
        quality::QualityTargetRef::Endpoint(endpoint("quality-endpoint")),
        quality::QualityTargetRef::Route(route("quality-route")),
        quality::QualityTargetRef::Stream(stream("quality-stream")),
        quality::QualityTargetRef::Packet(packet("quality-packet")),
    ] {
        touch_hash(target);
    }

    for failure in [
        quality::QualityFailureKind::PacketSuppressedByQuality,
        quality::QualityFailureKind::RouteSuppressedByQuality,
        quality::QualityFailureKind::QualityRecoveryNotAllowed,
        quality::QualityFailureKind::EndpointQualityNotAllowed,
        quality::QualityFailureKind::EndpointDegradedByQuality,
        quality::QualityFailureKind::PublicationQualityNotAllowed,
        quality::QualityFailureKind::SubscriptionQualityNotAllowed,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    for scope in [
        quality::AdmissionScope::Global,
        quality::AdmissionScope::Room,
        quality::AdmissionScope::Session,
        quality::AdmissionScope::Allocation,
        quality::AdmissionScope::CredentialRef,
        quality::AdmissionScope::DriverConnectionRef,
    ] {
        touch_hash(scope);
        touch_hash(quality::RateWindowPolicy::new(
            scope,
            Some(1),
            Some(2),
            Some(3),
            "reset",
        ));
    }

    for target in [
        quality::AdmissionTarget::RoomMaterialization,
        quality::AdmissionTarget::ParticipantJoin,
        quality::AdmissionTarget::SignalingCommandQueue,
        quality::AdmissionTarget::SfuEndpointAdmission,
        quality::AdmissionTarget::SfuRouteCandidateConstruction,
        quality::AdmissionTarget::TurnAllocation,
        quality::AdmissionTarget::TurnPermission,
        quality::AdmissionTarget::ConnectionConcurrency,
        quality::AdmissionTarget::InboundFrame,
    ] {
        touch_hash(target);
    }

    for decision in [
        quality::AdmissionDecisionKind::Accepted,
        quality::AdmissionDecisionKind::Rejected,
        quality::AdmissionDecisionKind::Dropped,
        quality::AdmissionDecisionKind::Shed,
        quality::AdmissionDecisionKind::Expired,
    ] {
        touch_hash(decision);
    }

    for failure in [
        quality::AdmissionFailureKind::RoomCapacityExceeded,
        quality::AdmissionFailureKind::AdmissionCapacityExceeded,
        quality::AdmissionFailureKind::SignalingCommandQueueBoundExceeded,
        quality::AdmissionFailureKind::EndpointCapacityExceeded,
        quality::AdmissionFailureKind::RouteCandidateBoundExceeded,
        quality::AdmissionFailureKind::AllocationCapacityExceeded,
        quality::AdmissionFailureKind::PermissionCapacityExceeded,
        quality::AdmissionFailureKind::ConnectionConcurrencyExceeded,
        quality::AdmissionFailureKind::FrameSizeBoundExceeded,
        quality::AdmissionFailureKind::DriverShutdown,
        quality::AdmissionFailureKind::RuntimeConfigMissing,
        quality::AdmissionFailureKind::RuntimeConfigInvalid,
        quality::AdmissionFailureKind::PersistenceUnavailable,
        quality::AdmissionFailureKind::AuthorizationContextMissing,
        quality::AdmissionFailureKind::AuthorizationScopeNotAllowed,
        quality::AdmissionFailureKind::ClientAddressUntrusted,
        quality::AdmissionFailureKind::ForwardedHeaderUntrusted,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
}

#[test]
fn quality_resource_bound_tables_are_accessor_verified() {
    assert!(quality::REQUIRED_RESOURCE_BOUNDS.len() >= 24);
    for bound in quality::REQUIRED_RESOURCE_BOUNDS {
        touch_hash(*bound);
        assert!(format!("{bound:?}").contains("required_bound"));
    }

    for resource in [
        quality::ResourceBoundKind::ActiveRoomSet,
        quality::ResourceBoundKind::RoomLifecycle,
        quality::ResourceBoundKind::SignalingCommandQueue,
        quality::ResourceBoundKind::RoomParticipantSet,
        quality::ResourceBoundKind::SfuEndpointAdmission,
        quality::ResourceBoundKind::SfuPacketCache,
        quality::ResourceBoundKind::SfuTransmitQueue,
        quality::ResourceBoundKind::SfuRouteCandidates,
        quality::ResourceBoundKind::TurnAllocationTable,
        quality::ResourceBoundKind::TurnAllocationLifetime,
        quality::ResourceBoundKind::TurnRefreshCap,
        quality::ResourceBoundKind::TurnPermissionTable,
        quality::ResourceBoundKind::TurnPermissionLifetime,
        quality::ResourceBoundKind::TurnChannelBindLifetime,
        quality::ResourceBoundKind::TurnRelayQueue,
        quality::ResourceBoundKind::AuditSinkBacklog,
        quality::ResourceBoundKind::PersistenceRetryStore,
        quality::ResourceBoundKind::MetricsExportBacklog,
        quality::ResourceBoundKind::DriverReceiveBufferPool,
        quality::ResourceBoundKind::InboundFrameSize,
        quality::ResourceBoundKind::ConnectionConcurrency,
        quality::ResourceBoundKind::MemoryPressure,
        quality::ResourceBoundKind::AggregateSerializationQueueLockWait,
        quality::ResourceBoundKind::RuntimeTaskQueueWorkerMailboxJoinWait,
    ] {
        touch_hash(resource);
        assert!(!resource.resource_name().is_empty());
    }

    assert!(quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS.len() >= 25);
    for action in quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS {
        touch_hash(*action);
        touch_hash(action.resource());
        touch_hash(action.action());
        touch_hash(action.audit_event_type());
        touch_hash(action.outcome());
        touch_hash(action.owner_tuple());
        assert!(CatalogedReasonRef::from_code(action.reason_code()).is_ok());
    }

    for audit_event in [
        quality::ResourceBoundAuditEventType::BackpressureDecision,
        quality::ResourceBoundAuditEventType::SfuSubscriptionDecision,
        quality::ResourceBoundAuditEventType::ResourceBoundDecision,
        quality::ResourceBoundAuditEventType::DriverResourceBoundDecision,
        quality::ResourceBoundAuditEventType::TurnAllocationDecision,
        quality::ResourceBoundAuditEventType::TurnRefreshDecision,
        quality::ResourceBoundAuditEventType::TurnPermissionDecision,
        quality::ResourceBoundAuditEventType::TurnChannelBindDecision,
        quality::ResourceBoundAuditEventType::RuntimeTaskLifecycleDecision,
    ] {
        touch_hash(audit_event);
        assert!(!audit_event.code().is_empty());
    }

    for outcome in [
        quality::ResourceBoundOutcome::Accepted,
        quality::ResourceBoundOutcome::WithinBoundObserved,
        quality::ResourceBoundOutcome::Rejected,
        quality::ResourceBoundOutcome::Dropped,
        quality::ResourceBoundOutcome::Shed,
        quality::ResourceBoundOutcome::Expired,
    ] {
        touch_hash(outcome);
        assert!(!outcome.code().is_empty());
    }
}

#[test]
fn quality_resource_decision_and_backpressure_fail_closed_edges_execute() {
    let turn_relay_action = quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .copied()
        .find(|action| action.resource() == quality::ResourceBoundKind::TurnRelayQueue)
        .expect("turn relay closed action exists");
    assert_eq!(
        quality::ResourceBoundDecision::try_new(
            turn_relay_action,
            quality::ResourceBoundReferenceSet::none(),
        ),
        Err(quality::ResourceBoundDecisionShapeError::TurnRelayQueueReferencesMissing)
    );
    assert!(quality::ResourceBoundDecision::try_new(
        turn_relay_action,
        quality::ResourceBoundReferenceSet::turn_relay_queue(
            allocation("quality-allocation"),
            permission("quality-permission"),
        ),
    )
    .is_ok());

    let packet_action = quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .copied()
        .find(|action| action.resource() == quality::ResourceBoundKind::SfuPacketCache)
        .expect("packet cache closed action exists");
    assert_eq!(
        quality::ResourceBoundDecision::try_new(
            packet_action,
            quality::ResourceBoundReferenceSet::route(route("quality-route-missing-packet")),
        ),
        Err(quality::ResourceBoundDecisionShapeError::PacketReferenceMissing)
    );
    assert!(quality::ResourceBoundDecision::try_new(
        packet_action,
        quality::ResourceBoundReferenceSet::packet(packet("quality-packet-ok")),
    )
    .is_ok());

    let inbound_frame_action = quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .copied()
        .find(|action| action.resource() == quality::ResourceBoundKind::InboundFrameSize)
        .expect("inbound frame closed action exists");
    assert_eq!(
        quality::ResourceBoundDecision::try_new(
            inbound_frame_action,
            quality::ResourceBoundReferenceSet::endpoint(endpoint("quality-endpoint-missing")),
        ),
        Err(quality::ResourceBoundDecisionShapeError::DriverResourceReferenceMissing)
    );
    assert!(quality::ResourceBoundDecision::try_new(
        inbound_frame_action,
        quality::ResourceBoundReferenceSet::driver_resource("inbound-frame-buffer"),
    )
    .is_ok());

    for outcome in [
        quality::BackpressureOutcome::Accepted,
        quality::BackpressureOutcome::Delayed,
        quality::BackpressureOutcome::Suppressed,
        quality::BackpressureOutcome::Dropped,
        quality::BackpressureOutcome::Degraded,
        quality::BackpressureOutcome::ClosedByPolicy,
        quality::BackpressureOutcome::Rejected,
    ] {
        touch_hash(outcome);
        assert!(!outcome.code().is_empty());
    }

    for kind in [
        quality::BackpressureDecisionKind::Accept,
        quality::BackpressureDecisionKind::DelayAction,
        quality::BackpressureDecisionKind::SuppressForwarding,
        quality::BackpressureDecisionKind::SuppressSubscription,
        quality::BackpressureDecisionKind::SuppressRouteState,
        quality::BackpressureDecisionKind::DropPacket,
        quality::BackpressureDecisionKind::StopRetainingPacketByPressurePolicy,
        quality::BackpressureDecisionKind::DegradeRoute,
        quality::BackpressureDecisionKind::CloseEndpoint,
        quality::BackpressureDecisionKind::RejectRecoveryFromBackpressureState,
    ] {
        touch_hash(kind);
        touch_hash(kind.audit_event_type());
        touch_hash(kind.outcome());
        if let Some(reason) = kind.reason_code() {
            assert!(CatalogedReasonRef::from_code(reason).is_ok());
        }
    }

    assert_eq!(
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::Accept,
            quality::BackpressureTargetRef::Packet(packet("bad-accept-target")),
        ),
        Err(quality::BackpressureDecisionShapeError::AcceptTargetMustBeNotApplicable)
    );
    assert!(quality::BackpressureDecision::try_new(
        quality::BackpressureDecisionKind::Accept,
        quality::BackpressureTargetRef::NotApplicable,
    )
    .is_ok());
    assert_eq!(
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::DelayAction,
            quality::BackpressureTargetRef::Packet(packet("bad-route-target")),
        ),
        Err(quality::BackpressureDecisionShapeError::RouteTargetRequired)
    );
    assert_eq!(
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::SuppressForwarding,
            quality::BackpressureTargetRef::Route(route("bad-packet-target")),
        ),
        Err(quality::BackpressureDecisionShapeError::PacketTargetRequired)
    );
    assert_eq!(
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::SuppressSubscription,
            quality::BackpressureTargetRef::Route(route("bad-subscription-target")),
        ),
        Err(quality::BackpressureDecisionShapeError::SubscriptionTargetRequired)
    );
    assert_eq!(
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::CloseEndpoint,
            quality::BackpressureTargetRef::Route(route("bad-endpoint-target")),
        ),
        Err(quality::BackpressureDecisionShapeError::EndpointTargetRequired)
    );

    for decision in [
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::DelayAction,
            quality::BackpressureTargetRef::Route(route("delay-route")),
        ),
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::SuppressForwarding,
            quality::BackpressureTargetRef::Packet(packet("suppress-packet")),
        ),
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::SuppressSubscription,
            quality::BackpressureTargetRef::Subscription(stream("suppress-stream")),
        ),
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::CloseEndpoint,
            quality::BackpressureTargetRef::Endpoint(endpoint("close-endpoint")),
        ),
    ] {
        touch_hash(decision.expect("valid backpressure target"));
    }

    for rule in [
        quality::AuditBacklogOverflowRule::EmitSingleReservedOverflowRecord,
        quality::AuditBacklogOverflowRule::RejectNewAuditRequiredPath,
        quality::AuditBacklogOverflowRule::ProhibitCloseoutEvidence,
    ] {
        touch_hash(rule);
    }
    for prohibited in [
        quality::ProhibitedResourceBoundaryBehavior::UnboundedQueue,
        quality::ProhibitedResourceBoundaryBehavior::UnboundedCache,
        quality::ProhibitedResourceBoundaryBehavior::UnboundedRetry,
        quality::ProhibitedResourceBoundaryBehavior::UnboundedPacketRetention,
        quality::ProhibitedResourceBoundaryBehavior::BestEffortFallbackWithoutReason,
        quality::ProhibitedResourceBoundaryBehavior::DriverLocalDropWithoutCoreReasonMapping,
        quality::ProhibitedResourceBoundaryBehavior::EntrypointsOverrideCoreBoundPolicy,
        quality::ProhibitedResourceBoundaryBehavior::HiddenRuntimeTaskWorkerBound,
    ] {
        touch_hash(prohibited);
    }
}
