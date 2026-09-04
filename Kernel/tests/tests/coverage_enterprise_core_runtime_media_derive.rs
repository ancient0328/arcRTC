use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use arcrtc_core_audit as audit;
use arcrtc_core_command as command;
use arcrtc_core_cross_plane as cross_plane;
use arcrtc_core_identity::{
    AllocationId, AuditEventId, ChannelBindId, ConfigurationScopeRef, CorrelationId, CredentialRef,
    EndpointId, OpaqueReference, PacketId, ParticipantId, PermissionId, ReferenceAuthority, RoomId,
    RouteId, SessionId, StartupRunId, StreamId,
};
use arcrtc_core_ports as ports;
use arcrtc_core_protocol as protocol;
use arcrtc_core_quality as quality;
use arcrtc_core_reason::{find_reason_definition, CatalogedReasonRef, Reason};
use arcrtc_core_recovery as recovery;
use arcrtc_core_runtime as runtime;
use arcrtc_core_security as security;
use arcrtc_core_sfu as sfu;
use arcrtc_core_signaling as signaling;
use arcrtc_core_state as state;

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

fn channel_bind(value: &str) -> ChannelBindId {
    ChannelBindId::new(reference(value))
}

fn credential(value: &str) -> CredentialRef {
    CredentialRef::new(reference(value))
}

fn startup(value: &str) -> StartupRunId {
    StartupRunId::new(reference(value))
}

fn config_scope(value: &str) -> ConfigurationScopeRef {
    ConfigurationScopeRef::new(reference(value))
}

fn cataloged(code: &'static str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason code is cataloged")
}

fn reason(code: &'static str) -> Reason<CatalogedReasonRef> {
    Reason::new(
        find_reason_definition(code).expect("reason definition is cataloged"),
        Some(cataloged(code)),
    )
}

fn touch_eq<T>(value: T)
where
    T: Clone + Debug + Eq,
{
    let cloned = value.clone();
    assert_eq!(cloned, value);
    assert!(!format!("{value:?}").is_empty());
}

fn touch_hash<T>(value: T)
where
    T: Clone + Debug + Eq + Hash,
{
    touch_eq(value.clone());
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    assert_ne!(hasher.finish(), 0);
}

fn command_decision(
    target_surface: command::TargetSurface,
    outcome: command::UseCaseOutcome,
    reason: command::DecisionReason<CatalogedReasonRef>,
) -> command::UseCaseDecision<CatalogedReasonRef> {
    command::UseCaseDecision::new(command::UseCaseDecisionInput {
        correlation_id: correlation("runtime-media-command-decision"),
        command_type: command::CommandType::new("runtime_media_command"),
        target_surface,
        outcome,
        reason,
        state_transition: command::StateTransitionSummary::Changed("runtime_media_derive"),
        port_intents: vec![command::PortIntent::new(
            "runtime_media_port_intent",
            target_surface,
        )],
        audit_projection: command::AuditProjectionRequirement::Required,
    })
    .expect("decision shape is valid")
}

fn accepted_runtime_task_policy() -> runtime::RuntimeTaskLifecyclePolicy {
    runtime::RuntimeTaskLifecyclePolicy::try_new(
        runtime::RuntimeTaskClass::DriverPacketWorker,
        Some(runtime::SupervisionScope::DriverComponent),
        runtime::RuntimeTaskOwningLayer::Driver,
        runtime::RuntimeTaskInputReferenceClass::OpaqueTaskReference,
        runtime::RuntimeTaskOutputObservation::SpawnObserved,
        runtime::CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
        true,
        true,
    )
    .expect("driver packet worker policy is bounded")
}

fn restore_preconditions() -> recovery::RestorePreconditionSet {
    recovery::RestorePreconditionSet::try_new(true, true, true, true, true, true, true, true, true)
        .expect("all restore preconditions are present")
}

fn restore_eligibility() -> recovery::RestoreEligibility {
    recovery::RestoreEligibility::try_new(
        recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured(),
        state::StateClass::CheckpointEligibleState,
        restore_preconditions(),
    )
    .expect("signaling idempotency checkpoint restore is eligible")
}

fn crash_restart_verification() -> recovery::CrashRestartVerification {
    recovery::CrashRestartVerification::try_new(
        "supervisor",
        recovery::ProcessFailureClass::SupervisorRestartObserved,
        recovery::ProcessFailureClass::SupervisorRestartObserved,
        Some(startup("crash-before")),
        Some(startup("crash-after")),
        true,
        true,
        true,
        true,
    )
    .expect("crash restart verification has distinct runtime states")
}

#[test]
fn audit_and_command_surface_derives_accessors_and_fail_closed_edges_execute() {
    touch_eq(audit::CoreAuditSurface);
    touch_eq(command::CoreCommandSurface);

    for component in [
        audit::AuditComponent::Core,
        audit::AuditComponent::Driver,
        audit::AuditComponent::Entrypoints,
        audit::AuditComponent::Sdk,
        audit::AuditComponent::Regulated,
    ] {
        touch_hash(component);
    }

    for kind in [
        audit::AuditReferenceKind::Event,
        audit::AuditReferenceKind::Correlation,
        audit::AuditReferenceKind::SubjectTransport,
        audit::AuditReferenceKind::Room,
        audit::AuditReferenceKind::Session,
        audit::AuditReferenceKind::Participant,
        audit::AuditReferenceKind::Endpoint,
        audit::AuditReferenceKind::Stream,
        audit::AuditReferenceKind::Packet,
        audit::AuditReferenceKind::Route,
        audit::AuditReferenceKind::Allocation,
        audit::AuditReferenceKind::Permission,
        audit::AuditReferenceKind::ChannelBind,
        audit::AuditReferenceKind::Credential,
        audit::AuditReferenceKind::StartupRun,
        audit::AuditReferenceKind::ConfigurationScope,
        audit::AuditReferenceKind::AuthorizationContext,
        audit::AuditReferenceKind::MediaNegotiation,
        audit::AuditReferenceKind::ServiceTopology,
        audit::AuditReferenceKind::ObservabilitySignal,
        audit::AuditReferenceKind::DependencyToolchain,
        audit::AuditReferenceKind::SdkPlatformProjection,
        audit::AuditReferenceKind::InternalControlPlane,
        audit::AuditReferenceKind::IceCandidateConnectivity,
        audit::AuditReferenceKind::SecureMediaSession,
        audit::AuditReferenceKind::OperatorAdminAuthorization,
        audit::AuditReferenceKind::OutOfScopeFeature,
        audit::AuditReferenceKind::PublicEndpointConnection,
        audit::AuditReferenceKind::ExportBackupArtifact,
        audit::AuditReferenceKind::ReleaseArtifactDistribution,
        audit::AuditReferenceKind::TimeSynchronizationClockSkew,
        audit::AuditReferenceKind::EdgeProxyTrust,
        audit::AuditReferenceKind::RuntimeReconfiguration,
        audit::AuditReferenceKind::PacketRewriteMediaTransform,
        audit::AuditReferenceKind::ServiceDiscoveryEndpointResolution,
        audit::AuditReferenceKind::DistributedStateFailover,
        audit::AuditReferenceKind::RuntimeTaskWorker,
        audit::AuditReferenceKind::InternalServiceIdentityTrust,
        audit::AuditReferenceKind::CrossPlaneBinding,
    ] {
        touch_hash(kind);
    }

    for value in [
        audit::AuditReferenceValue::CorrelationId(correlation("audit-ref-correlation")),
        audit::AuditReferenceValue::RoomId(room("audit-ref-room")),
        audit::AuditReferenceValue::SessionId(session("audit-ref-session")),
        audit::AuditReferenceValue::ParticipantId(participant("audit-ref-participant")),
        audit::AuditReferenceValue::EndpointId(endpoint("audit-ref-endpoint")),
        audit::AuditReferenceValue::StreamId(stream("audit-ref-stream")),
        audit::AuditReferenceValue::PacketId(packet("audit-ref-packet")),
        audit::AuditReferenceValue::RouteId(route("audit-ref-route")),
        audit::AuditReferenceValue::AllocationId(allocation("audit-ref-allocation")),
        audit::AuditReferenceValue::PermissionId(permission("audit-ref-permission")),
        audit::AuditReferenceValue::ChannelBindId(channel_bind("audit-ref-channel-bind")),
        audit::AuditReferenceValue::AuditEventId(AuditEventId::new(reference("audit-ref-event"))),
        audit::AuditReferenceValue::CredentialRef(credential("audit-ref-credential")),
        audit::AuditReferenceValue::StartupRunId(startup("audit-ref-startup")),
        audit::AuditReferenceValue::ConfigurationScopeRef(config_scope("audit-ref-config")),
        audit::AuditReferenceValue::Generic("non_sensitive_reference"),
    ] {
        touch_hash(value);
    }

    let mut references = audit::AuditReferences::default();
    references.push(audit::AuditReferencePresence::Present {
        kind: audit::AuditReferenceKind::Correlation,
        value: audit::AuditReferenceValue::CorrelationId(correlation("audit-accessor")),
    });
    let cloned_references = references.clone();
    assert_eq!(cloned_references, references);

    let timestamp = audit::AuditTimestamp::new("2026-06-16T09:30:00Z").expect("timestamp");
    let event = audit::AuditEvent::new(audit::AuditEventInput {
        event_id: AuditEventId::new(reference("audit-accessor-event")),
        correlation: audit::AuditReferencePresence::Present {
            kind: audit::AuditReferenceKind::Correlation,
            value: audit::AuditReferenceValue::CorrelationId(correlation("audit-accessor")),
        },
        timestamp,
        component: audit::AuditComponent::Core,
        event_type: audit::find_audit_event_definition("runtime_task_lifecycle_decision")
            .expect("event type is registered"),
        outcome: command::UseCaseOutcome::Failed,
        reason: audit::AuditReason::Cataloged(reason("runtime_task_spawn_failed")),
        references,
        resource_owner: Some(audit::ResourceOwnerTuple::new(
            audit::AuditComponent::Core,
            audit::AuditComponent::Driver,
        )),
        tags: vec![audit::NonSensitiveTag::new("surface", "runtime")],
    })
    .expect("failed audit event carries a cataloged reason");
    assert!(matches!(
        event.correlation(),
        audit::AuditReferencePresence::Present { .. }
    ));
    assert_eq!(event.event_id().as_str(), "audit-accessor-event");
    assert_eq!(event.outcome(), command::UseCaseOutcome::Failed);
    touch_eq(event.clone());
    touch_hash(audit::HashChainSequence::new(42));
    let record_hash =
        audit::RecordHash::new(audit::HashAlgorithm::Sha256, vec![9]).expect("record hash");
    assert_eq!(record_hash.algorithm(), audit::HashAlgorithm::Sha256);
    assert_eq!(record_hash.digest(), &[9]);
    touch_hash(audit::PreviousRecordHash::Previous(record_hash));
    touch_hash(audit::HashChainRecordError::EmptyDigest);

    for shape in [
        command::ResultShapeClass::CommandEnvelope,
        command::ResultShapeClass::UseCaseDecision,
        command::ResultShapeClass::DomainEvent,
        command::ResultShapeClass::PortIntent,
        command::ResultShapeClass::DriverObservation,
        command::ResultShapeClass::AuditProjection,
        command::ResultShapeClass::ExternalResponseModel,
    ] {
        touch_hash(shape);
    }

    let envelope = command::CommandEnvelope::new(
        correlation("command-envelope"),
        command::CommandType::new("runtime_media_subject"),
        command::CommandVersion::new(2),
        command::TargetSurface::Sfu,
        route("command-envelope-route"),
    );
    touch_eq(envelope.clone());
    assert_eq!(envelope.command_type().as_str(), "runtime_media_subject");
    assert_eq!(envelope.version().value(), 2);
    assert_eq!(
        envelope.subject_references().as_str(),
        "command-envelope-route"
    );

    let decision = command_decision(
        command::TargetSurface::Sfu,
        command::UseCaseOutcome::Rejected,
        command::DecisionReason::Cataloged(cataloged("route_conflict")),
    );
    assert_eq!(decision.command_type().as_str(), "runtime_media_command");
    assert_eq!(
        decision.audit_projection(),
        command::AuditProjectionRequirement::Required
    );
    touch_eq(decision.clone());

    assert!(command::DriverObservation::new(
        correlation("driver-observation"),
        command::UseCaseOutcome::Accepted,
        command::DecisionReason::Cataloged(cataloged("room_closed")),
    )
    .is_err());
    assert!(command::DriverObservation::<CatalogedReasonRef>::new(
        correlation("driver-observation-missing"),
        command::UseCaseOutcome::Failed,
        command::DecisionReason::Absent,
    )
    .is_err());
    touch_eq(
        command::DriverObservation::new(
            correlation("driver-observation-ok"),
            command::UseCaseOutcome::Failed,
            command::DecisionReason::Cataloged(cataloged("driver_shutdown")),
        )
        .expect("failed observation carries reason"),
    );
    touch_eq(command::AuditProjection::new(
        correlation("audit-projection"),
        "runtime_task_lifecycle_decision",
        command::UseCaseOutcome::Failed,
        command::DecisionReason::Cataloged(cataloged("runtime_task_join_failed")),
    ));
    touch_eq(command::ExternalResponseModel::new(
        correlation("external-response"),
        command::UseCaseOutcome::Rejected,
        command::DecisionReason::Cataloged(cataloged("room_closed")),
    ));
}

#[test]
fn ports_and_cross_plane_public_shapes_execute_remaining_derive_paths() {
    touch_eq(ports::CorePortsSurface);
    touch_eq(cross_plane::CoreCrossPlaneSurface);

    for family in [
        ports::PortFamily::Clock,
        ports::PortFamily::Random,
        ports::PortFamily::TokenVerifier,
        ports::PortFamily::Network,
        ports::PortFamily::WebRtcTransport,
        ports::PortFamily::PacketView,
        ports::PortFamily::Persistence,
        ports::PortFamily::AuditSink,
        ports::PortFamily::MetricsSink,
        ports::PortFamily::Runtime,
    ] {
        touch_hash(family);
    }

    let context = ports::PortCallContext::new(correlation("port-context"));
    touch_eq(context.clone());
    assert_eq!(context.correlation_id().as_str(), "port-context");

    let semantic_envelope = protocol::CoreSemanticEnvelope::try_new(
        command::TargetSurface::Sfu,
        protocol::ContractVersion::new(2, 0, 0),
        correlation("network-envelope"),
        protocol::SemanticEnvelopeMessageKind::Event,
        protocol::CoreSemanticMessageType::SfuModel(
            protocol::SfuSemanticModelType::ForwardingIntent,
        ),
        true,
        Some(protocol::CoreSemanticPayloadModel::new(
            protocol::CoreSemanticPayloadClass::SfuReferenceSet,
            Some(reference("network-payload-reference")),
        )),
        Some(command::UseCaseOutcome::Forwarded),
        None,
    )
    .expect("success semantic envelope has no reason");
    touch_eq(semantic_envelope.clone());
    touch_eq(ports::NetworkPortInput::SendSemanticEnvelope(
        semantic_envelope.clone(),
    ));
    touch_eq(ports::NetworkPortInput::ObserveInbound(
        semantic_envelope.clone(),
    ));
    touch_eq(ports::NetworkPortInput::CloseConnection(reference(
        "network-peer",
    )));
    let delivery =
        ports::NetworkDeliveryObservation::new(correlation("delivery-observation"), None, false);
    touch_hash(delivery.clone());
    assert!(delivery.peer_ref().is_none());
    assert!(!delivery.delivered());
    touch_eq(ports::NetworkPortOutput::DeliveryObservation(delivery));
    touch_eq(ports::NetworkPortOutput::ConvertedInbound(
        semantic_envelope,
    ));

    let metric = quality::QualityMetric::new(
        quality::QualityMetricKind::QueueDepth,
        7,
        "items",
        "instant",
    );
    touch_eq(ports::MetricsSinkInput::SubmitQualityMetric(metric));
    let acknowledgement = ports::MetricsExportAcknowledgement::new(true, false);
    touch_hash(acknowledgement);
    touch_hash(ports::MetricsSinkOutput::Acknowledgement(acknowledgement));

    for shape in [
        ports::PortCallShape::Command,
        ports::PortCallShape::Query,
        ports::PortCallShape::SinkSubmit,
        ports::PortCallShape::StreamObservation,
        ports::PortCallShape::Scheduler,
    ] {
        touch_hash(shape);
    }
    for rule in [
        ports::PortOwnershipRule::CoreOwnedTypesOnly,
        ports::PortOwnershipRule::NoDriverBufferOwnershipTransfer,
        ports::PortOwnershipRule::NoConcreteRuntimeHandleTransfer,
    ] {
        touch_hash(rule);
    }
    for class in [
        ports::PortErrorClass::DriverFailure,
        ports::PortErrorClass::BoundOrBackpressure,
        ports::PortErrorClass::ConversionFailure,
        ports::PortErrorClass::RuntimeOrShutdown,
        ports::PortErrorClass::TokenVerification,
        ports::PortErrorClass::Persistence,
    ] {
        touch_hash(class);
        touch_eq(ports::PortError::new(
            class,
            Reason::new(
                find_reason_definition("driver_shutdown").expect("reason definition"),
                Some("driver_shutdown"),
            ),
        ));
    }

    assert_eq!(
        ports::PersistencePortIntent::try_new(
            ports::PersistenceIntentClass::AuditPersistence,
            ports::PersistenceOperationKind::LoadCheckpoint,
            state::StateFamily::AuditEvent,
            state::StateClass::AuditOnlyState,
            vec![ports::PersistenceConsistencyRequirement::OrderedAppend],
            Some(correlation("persistence-audit-mismatch")),
        ),
        Err(ports::PersistencePortIntentError::OperationClassMismatch)
    );
    assert_eq!(
        ports::PersistencePortIntent::try_new(
            ports::PersistenceIntentClass::RetryStore,
            ports::PersistenceOperationKind::EnqueueRetry,
            state::StateFamily::MetricsBacklog,
            state::StateClass::DriverLocalState,
            vec![ports::PersistenceConsistencyRequirement::BoundedRetryStore],
            Some(correlation("persistence-retry-family")),
        ),
        Err(ports::PersistencePortIntentError::RetryIntentRequiresDriverRetryStore)
    );
    let retry_intent = ports::PersistencePortIntent::try_new(
        ports::PersistenceIntentClass::RetryStore,
        ports::PersistenceOperationKind::AcknowledgeRetry,
        state::StateFamily::DriverRetryStore,
        state::StateClass::DriverLocalState,
        vec![ports::PersistenceConsistencyRequirement::BoundedRetryStore],
        Some(correlation("persistence-retry-ok")),
    )
    .expect("bounded retry store intent is accepted");
    touch_eq(ports::PersistencePortInput::ExecuteIntent(retry_intent));
    touch_eq(ports::PersistencePortOutput::Acknowledgement(
        ports::PersistenceAcknowledgement::new(
            ports::PersistenceIntentClass::RetryStore,
            Some(ports::PersistenceRecordRef::new(reference(
                "persistence-ack-record",
            ))),
            true,
        ),
    ));
    touch_eq(ports::PersistencePortOutput::LoadedState(
        ports::LoadedCoreStateRef::new(
            state::StateFamily::SignalingIdempotency,
            state::StateClass::CheckpointEligibleState,
            ports::PersistenceRecordRef::new(reference("loaded-state")),
        ),
    ));
    for kind in [
        ports::PersistencePortFailureKind::PersistenceUnavailable,
        ports::PersistencePortFailureKind::PersistenceRetryBoundExceeded,
        ports::PersistencePortFailureKind::PersistenceRetryDurationExceeded,
        ports::PersistencePortFailureKind::AuditBacklogBoundExceeded,
        ports::PersistencePortFailureKind::DriverShutdown,
    ] {
        touch_hash(kind);
        let failure = ports::PersistencePortFailure::from_kind(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            kind.reason_code()
        );
        if matches!(
            kind,
            ports::PersistencePortFailureKind::PersistenceRetryBoundExceeded
                | ports::PersistencePortFailureKind::PersistenceRetryDurationExceeded
                | ports::PersistencePortFailureKind::AuditBacklogBoundExceeded
        ) {
            assert!(failure.resource_bound_decision().is_some());
        }
        touch_hash(failure);
    }

    for class in [
        ports::PersistenceStateClass::SourceOfTruth,
        ports::PersistenceStateClass::Checkpoint,
        ports::PersistenceStateClass::AuditOnly,
        ports::PersistenceStateClass::DriverRetryData,
    ] {
        touch_hash(class);
    }
    let class = ports::PacketViewClass::BorrowedSemanticHeaderView;
    touch_hash(class);
    for class in [
        ports::RuntimePortOutputClass::OpaqueScheduleReference,
        ports::RuntimePortOutputClass::CancellationObservation,
        ports::RuntimePortOutputClass::ExecutionObservation,
    ] {
        touch_hash(class);
    }

    for plane in [
        cross_plane::CrossPlane::Signaling,
        cross_plane::CrossPlane::Sfu,
        cross_plane::CrossPlane::Turn,
        cross_plane::CrossPlane::IceTransport,
        cross_plane::CrossPlane::SecureMedia,
    ] {
        touch_hash(plane);
    }
    for precondition in [
        cross_plane::BindingLifecyclePrecondition::NotApplicable,
        cross_plane::BindingLifecyclePrecondition::ParticipantJoined,
        cross_plane::BindingLifecyclePrecondition::EndpointAdmitted,
        cross_plane::BindingLifecyclePrecondition::AllocationActive,
        cross_plane::BindingLifecyclePrecondition::PermissionActive,
        cross_plane::BindingLifecyclePrecondition::ChannelBindActive,
        cross_plane::BindingLifecyclePrecondition::CandidatePolicyAccepted,
        cross_plane::BindingLifecyclePrecondition::SecureMediaProtectionActive,
    ] {
        touch_hash(precondition);
    }
    for behavior in [
        cross_plane::BindingExpiryBehavior::NotApplicable,
        cross_plane::BindingExpiryBehavior::RejectNewTargetPlaneAction,
        cross_plane::BindingExpiryBehavior::TargetPlaneClosesThroughOwnStateMachine,
        cross_plane::BindingExpiryBehavior::RecordExpiredRelation,
    ] {
        touch_hash(behavior);
    }
    for relation in [
        cross_plane::BindingReplayRelation::NotCommandScoped,
        cross_plane::BindingReplayRelation::PriorAcceptedBindingObservable,
        cross_plane::BindingReplayRelation::ConflictRejected,
    ] {
        touch_hash(relation);
    }
    let policy = cross_plane::CrossPlaneBindingPolicy::new(
        cross_plane::CrossPlaneBindingClass::SecureMediaSessionBinding,
        cross_plane::CrossPlane::Signaling,
        cross_plane::CrossPlane::SecureMedia,
        cross_plane::CrossPlaneReference::Participant(participant("cross-plane-participant")),
        cross_plane::CrossPlaneReference::Credential(credential("cross-plane-credential")),
        Some(security::AuthorizationContextClass::VerifiedCredentialContext),
        cross_plane::BindingLifecyclePrecondition::ParticipantJoined,
        cross_plane::BindingLifecyclePrecondition::SecureMediaProtectionActive,
        cross_plane::BindingExpiryBehavior::RecordExpiredRelation,
        cross_plane::BindingReplayRelation::ConflictRejected,
    );
    touch_hash(policy.clone());
    let decision = cross_plane::CrossPlaneBindingDecision::new(
        correlation("cross-plane-decision"),
        policy,
        cross_plane::CrossPlaneBindingOutcome::Rejected,
        Some(cross_plane::CrossPlaneBindingFailureKind::SecureMediaProtectionNotActive),
    );
    touch_hash(decision);
}

#[test]
fn recovery_runtime_and_sfu_closed_vocabularies_execute_derive_paths() {
    touch_eq(recovery::CoreRecoverySurface);
    touch_eq(runtime::CoreRuntimeSurface);
    touch_eq(sfu::CoreSfuSurface);

    for recovery_class in [
        recovery::RecoveryClass::NoRestore,
        recovery::RecoveryClass::CheckpointRestoreCandidate,
        recovery::RecoveryClass::AuditReplayVerificationOnly,
        recovery::RecoveryClass::DriverRetryRecovery,
    ] {
        touch_hash(recovery_class);
        let permitted = recovery_class.permits_core_state_restore_candidate();
        assert_eq!(
            permitted,
            matches!(
                recovery_class,
                recovery::RecoveryClass::CheckpointRestoreCandidate
            )
        );
    }

    let default_metrics =
        recovery::StateFamilyRecoveryPolicy::default_for(state::StateFamily::MetricsBacklog);
    assert!(!default_metrics.permits_core_restore_candidate());
    let explicit_metrics = recovery::StateFamilyRecoveryPolicy::metrics_backlog_retry_configured();
    assert!(!explicit_metrics.permits_domain_mutation_replay_candidate());
    touch_hash(default_metrics);
    touch_hash(explicit_metrics);

    assert_eq!(
        recovery::RestoreEligibility::try_new(
            recovery::StateFamilyRecoveryPolicy::default_for(state::StateFamily::AuditEvent),
            state::StateClass::AuditOnlyState,
            restore_preconditions(),
        ),
        Err(recovery::RestoreEligibilityError::StateFamilyNotRestoreEligible)
    );
    touch_hash(restore_preconditions());
    touch_hash(restore_eligibility());

    for mode in [
        recovery::ReplayMode::AuditIntegrityVerificationOnly,
        recovery::ReplayMode::DomainMutationCandidate,
    ] {
        touch_hash(mode);
    }
    touch_hash(
        recovery::ReplayPolicyCoverage::try_new(true, true, true, true, true, true, true)
            .expect("complete replay policy coverage"),
    );
    for failure in [
        recovery::RecoveryFailureKind::PersistenceUnavailable,
        recovery::RecoveryFailureKind::ExternalDecodeFailed,
        recovery::RecoveryFailureKind::UnsupportedDriverWireVersion,
        recovery::RecoveryFailureKind::MissingRequiredWireField,
        recovery::RecoveryFailureKind::ExternalEnumUnmapped,
        recovery::RecoveryFailureKind::CommandOrderViolation,
        recovery::RecoveryFailureKind::PersistenceRetryBoundExceeded,
        recovery::RecoveryFailureKind::PersistenceRetryDurationExceeded,
        recovery::RecoveryFailureKind::DriverShutdown,
        recovery::RecoveryFailureKind::UncleanShutdownDetected,
        recovery::RecoveryFailureKind::FailoverNotProven,
        recovery::RecoveryFailureKind::StateOwnerConflict,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for prohibited in [
        recovery::ProhibitedRecoveryBehavior::DriverSchemaAsDomainSourceOfTruth,
        recovery::ProhibitedRecoveryBehavior::AuditReplayMutatesWithoutExplicitRestorePolicy,
        recovery::ProhibitedRecoveryBehavior::SfuRouteStateSilentlyRestored,
        recovery::ProhibitedRecoveryBehavior::TurnRelayStateSilentlyRestored,
        recovery::ProhibitedRecoveryBehavior::SdkReconnectStateAsServerParticipantState,
        recovery::ProhibitedRecoveryBehavior::DriverLocalConflictResolution,
        recovery::ProhibitedRecoveryBehavior::RestoreSuccessWithoutVerification,
        recovery::ProhibitedRecoveryBehavior::CrashObservationAsRestoreSuccess,
        recovery::ProhibitedRecoveryBehavior::ReplacementProcessAsFailoverSuccess,
        recovery::ProhibitedRecoveryBehavior::ReplicationInferredFromRestore,
    ] {
        touch_hash(prohibited);
    }

    for class in [
        recovery::DistributedStateClass::NodeLocalState,
        recovery::DistributedStateClass::AffinityRequiredState,
        recovery::DistributedStateClass::CheckpointCandidateState,
        recovery::DistributedStateClass::AuditVerificationState,
        recovery::DistributedStateClass::ReplicatedStateRequested,
        recovery::DistributedStateClass::ConsensusStateRequested,
        recovery::DistributedStateClass::AutomaticFailoverRequested,
    ] {
        touch_hash(class);
        let _ = class.admitted_in_initial_v0_2();
    }

    assert_eq!(
        recovery::ProcessFailureClassification::try_new(
            None,
            Some(startup("after")),
            None,
            recovery::ProcessFailureClass::PanicObserved,
            Some(recovery::PriorDrainStatus::GracefulDrainObserved),
            Some(recovery::AuditPersistenceStatus::AuditPersistenceCompleted),
            recovery::RestoreReplayPolicyApplication::RestoreEligibilityApplied,
            recovery::RestartReadinessClass::NotClaimed,
        ),
        Err(recovery::ProcessFailureClassificationError::ProcessIdentityMissing)
    );
    let classification = recovery::ProcessFailureClassification::try_new(
        Some(startup("before")),
        Some(startup("after")),
        Some(reference("process-identity")),
        recovery::ProcessFailureClass::PanicObserved,
        Some(recovery::PriorDrainStatus::DrainIncompleteOrAbsent),
        Some(recovery::AuditPersistenceStatus::AuditPersistenceIncompleteOrAbsent),
        recovery::RestoreReplayPolicyApplication::RestoreEligibilityApplied,
        recovery::RestartReadinessClass::NotClaimed,
    )
    .expect("process failure classification is complete");
    assert!(classification.is_unclean_shutdown());
    touch_hash(classification.clone());
    assert_eq!(
        recovery::RestartDomainStateClaim::try_new(
            classification.clone(),
            Some(restore_eligibility()),
            None,
        ),
        Err(recovery::RestartDomainStateClaimError::CrashRestartVerificationRequired)
    );
    touch_hash(
        recovery::RestartDomainStateClaim::try_new(
            classification,
            Some(restore_eligibility()),
            Some(crash_restart_verification()),
        )
        .expect("crash restart claim includes restore eligibility and verification"),
    );
    touch_hash(crash_restart_verification());

    for mapping in [
        recovery::ProcessFailureMappingKind::ProcessPanicDetected,
        recovery::ProcessFailureMappingKind::RuntimeTaskPanicDetected,
        recovery::ProcessFailureMappingKind::ProcessCrashDetected,
        recovery::ProcessFailureMappingKind::UncleanShutdownDetected,
        recovery::ProcessFailureMappingKind::SupervisorRestartObserved,
        recovery::ProcessFailureMappingKind::DriverShutdown,
    ] {
        touch_hash(mapping);
        assert!(CatalogedReasonRef::from_code(mapping.reason_code()).is_ok());
    }
    for prohibited in [
        recovery::ProhibitedProcessFailureBehavior::UncleanCrashAsGracefulShutdown,
        recovery::ProhibitedProcessFailureBehavior::SupervisorRestartAsReadiness,
        recovery::ProhibitedProcessFailureBehavior::CrashRecoveryInferredWithoutRestoreVerification,
        recovery::ProhibitedProcessFailureBehavior::PanicLogTextAsAuthoritativeReason,
        recovery::ProhibitedProcessFailureBehavior::RestartReusesDomainStateWithoutRestorePolicy,
        recovery::ProhibitedProcessFailureBehavior::CrashVerificationOmitsAuditOrDrainStatus,
        recovery::ProhibitedProcessFailureBehavior::TaskPanicCollapsedIntoReadinessOrDriverFailure,
    ] {
        touch_hash(prohibited);
    }

    for surface in [
        runtime::RuntimeAbstractionSurface::ClockPort,
        runtime::RuntimeAbstractionSurface::RandomPort,
        runtime::RuntimeAbstractionSurface::RuntimePort,
    ] {
        touch_hash(surface);
        touch_hash(surface.ownership());
    }
    for class in [
        runtime::ClockObservationClass::CurrentTime,
        runtime::ClockObservationClass::MonotonicComparisonInput,
        runtime::ClockObservationClass::DeadlineComparisonInput,
        runtime::ClockObservationClass::ExpiryComparisonInput,
    ] {
        touch_hash(class);
    }
    for class in [
        runtime::RuntimeOperationClass::Timer,
        runtime::RuntimeOperationClass::Spawn,
        runtime::RuntimeOperationClass::Cancellation,
        runtime::RuntimeOperationClass::ShutdownObservation,
    ] {
        touch_hash(class);
    }
    for class in [
        runtime::RuntimeOutputClass::OpaqueScheduleReference,
        runtime::RuntimeOutputClass::OpaqueTaskReference,
        runtime::RuntimeOutputClass::CancellationObservation,
        runtime::RuntimeOutputClass::ShutdownObservation,
    ] {
        touch_hash(class);
    }
    touch_hash(runtime::RuntimeConfigurationSelection::new(
        startup("runtime-startup"),
        config_scope("runtime-config"),
        runtime::RuntimeAbstractionSurface::RuntimePort,
    ));
    for prohibited in [
        runtime::ProhibitedRuntimeClockRandomnessBehavior::CoreImportsConcreteRuntimeHandle,
        runtime::ProhibitedRuntimeClockRandomnessBehavior::DriverTimerDefinesDomainExpiry,
        runtime::ProhibitedRuntimeClockRandomnessBehavior::RawPlatformTimeWithoutNormalization,
        runtime::ProhibitedRuntimeClockRandomnessBehavior::RandomGeneratorOwnsIdentitySemantics,
        runtime::ProhibitedRuntimeClockRandomnessBehavior::EntrypointsSilentlySubstituteRuntimeDefaults,
        runtime::ProhibitedRuntimeClockRandomnessBehavior::DeterministicTestClockRngInProductionRuntime,
        runtime::ProhibitedRuntimeClockRandomnessBehavior::RuntimeWorkerOwnsDomainState,
        runtime::ProhibitedRuntimeClockRandomnessBehavior::ImplicitDetachedTaskOrSupervision,
        runtime::ProhibitedRuntimeClockRandomnessBehavior::WallClockAsCrossNodeCausalOrderWithoutTrust,
    ] {
        touch_hash(prohibited);
    }
    for concern in [
        runtime::RuntimeTaskConcern::PriorDomainDecisionReference,
        runtime::RuntimeTaskConcern::RuntimePortTaskContract,
        runtime::RuntimeTaskConcern::ConcreteTaskHandle,
        runtime::RuntimeTaskConcern::DriverIoWorker,
        runtime::RuntimeTaskConcern::EntrypointsSupervisionTask,
        runtime::RuntimeTaskConcern::TaskQueueMailbox,
        runtime::RuntimeTaskConcern::TaskPanicObservation,
        runtime::RuntimeTaskConcern::TaskCancellation,
    ] {
        touch_hash(concern);
        touch_hash(concern.owner());
    }
    let policy = accepted_runtime_task_policy();
    touch_hash(policy);
    for outcome in [
        runtime::RuntimeTaskLifecycleOutcome::Accepted,
        runtime::RuntimeTaskLifecycleOutcome::Rejected,
        runtime::RuntimeTaskLifecycleOutcome::Spawned,
        runtime::RuntimeTaskLifecycleOutcome::Joined,
        runtime::RuntimeTaskLifecycleOutcome::Cancelled,
        runtime::RuntimeTaskLifecycleOutcome::Failed,
        runtime::RuntimeTaskLifecycleOutcome::PanicObserved,
    ] {
        touch_hash(outcome);
    }
    touch_hash(
        runtime::RuntimeTaskLifecycleDecision::try_new(
            startup("runtime-lifecycle"),
            Some(correlation("runtime-lifecycle")),
            accepted_runtime_task_policy(),
            Some("opaque-task-ref"),
            runtime::RuntimeTaskLifecycleOutcome::PanicObserved,
            Some(runtime::RuntimeFailureKind::RuntimeTaskPanicDetected),
        )
        .expect("panic observation carries a reason"),
    );
    for cancellation in [
        runtime::TaskCancellationSurface::BeforeDriverCoreConversion,
        runtime::TaskCancellationSurface::AfterCommandEnteredCore,
        runtime::TaskCancellationSurface::DuringShutdownDrain,
        runtime::TaskCancellationSurface::DuringDriverQueueCacheExecution,
        runtime::TaskCancellationSurface::DuringTestHarnessTimeout,
    ] {
        touch_hash(cancellation);
        assert!(!cancellation.required_relation().is_empty());
    }
    for prohibited in [
        runtime::ProhibitedRuntimeTaskWorkerBehavior::ConcreteRuntimeTaskHandleInCoreApi,
        runtime::ProhibitedRuntimeTaskWorkerBehavior::DetachedTaskWithoutAdmittedSupervision,
        runtime::ProhibitedRuntimeTaskWorkerBehavior::WorkerCompletionAsDomainDecision,
        runtime::ProhibitedRuntimeTaskWorkerBehavior::TaskCancellationRewritesPriorDecision,
        runtime::ProhibitedRuntimeTaskWorkerBehavior::TaskPanicAsGracefulShutdownOrRecovery,
        runtime::ProhibitedRuntimeTaskWorkerBehavior::UnboundedTaskQueueMailboxJoinOrRestart,
        runtime::ProhibitedRuntimeTaskWorkerBehavior::SupervisorRestartAsReadinessOrRestoreSuccess,
        runtime::ProhibitedRuntimeTaskWorkerBehavior::DriverWorkerOwnsDomainSemantics,
    ] {
        touch_hash(prohibited);
    }

    for kind in [
        sfu::SfuModelKind::SfuSession,
        sfu::SfuModelKind::ParticipantEndpoint,
        sfu::SfuModelKind::MediaStream,
        sfu::SfuModelKind::MediaNegotiationReference,
        sfu::SfuModelKind::Publication,
        sfu::SfuModelKind::Subscription,
        sfu::SfuModelKind::ForwardingIntent,
        sfu::SfuModelKind::RouteCandidate,
        sfu::SfuModelKind::BorrowedPacketAbstractView,
        sfu::SfuModelKind::QualityObservation,
        sfu::SfuModelKind::BackpressureState,
        sfu::SfuModelKind::AdmissionDecision,
        sfu::SfuModelKind::RejectionReason,
    ] {
        touch_hash(kind);
    }
    for state in [
        sfu::SfuSessionState::Open,
        sfu::SfuSessionState::Draining,
        sfu::SfuSessionState::Closed,
    ] {
        touch_hash(state);
    }
    for state in [
        sfu::SfuEndpointState::Observed,
        sfu::SfuEndpointState::AdmissionPending,
        sfu::SfuEndpointState::Admitted,
        sfu::SfuEndpointState::Degraded,
        sfu::SfuEndpointState::Draining,
        sfu::SfuEndpointState::Removed,
        sfu::SfuEndpointState::Rejected,
    ] {
        touch_hash(state);
    }
    for state in [
        sfu::SfuPublicationState::Absent,
        sfu::SfuPublicationState::Requested,
        sfu::SfuPublicationState::Active,
        sfu::SfuPublicationState::Suppressed,
        sfu::SfuPublicationState::Closed,
        sfu::SfuPublicationState::Rejected,
    ] {
        touch_hash(state);
    }
    for state in [
        sfu::SfuSubscriptionState::Absent,
        sfu::SfuSubscriptionState::Requested,
        sfu::SfuSubscriptionState::Active,
        sfu::SfuSubscriptionState::Suppressed,
        sfu::SfuSubscriptionState::Closed,
        sfu::SfuSubscriptionState::Rejected,
    ] {
        touch_hash(state);
    }
    for state in [
        sfu::SfuRouteState::Candidate,
        sfu::SfuRouteState::Selected,
        sfu::SfuRouteState::Delayed,
        sfu::SfuRouteState::SuppressedByBackpressure,
        sfu::SfuRouteState::SuppressedByQuality,
        sfu::SfuRouteState::Degraded,
        sfu::SfuRouteState::Dropped,
        sfu::SfuRouteState::Closed,
    ] {
        touch_hash(state);
    }
    let refs = sfu::SfuReferenceSet::new(
        session("sfu-session"),
        Some(endpoint("sfu-endpoint")),
        Some(stream("sfu-stream")),
        Some(route("sfu-route")),
        Some(packet("sfu-packet")),
    );
    touch_eq(refs.clone());
    touch_eq(sfu::SfuContractItem::new(
        sfu::SfuModelKind::MediaNegotiationReference,
        refs,
        "payload-model",
    ));
    touch_eq(
        sfu::SfuDecision::new(
            sfu::SfuDecisionKind::RouteSelection,
            command_decision(
                command::TargetSurface::Sfu,
                command::UseCaseOutcome::Rejected,
                command::DecisionReason::Cataloged(cataloged("route_conflict")),
            ),
        )
        .expect("sfu decision accepts sfu target surface"),
    );
}

#[test]
fn sfu_media_and_signaling_runtime_related_catalogs_execute_public_edges() {
    touch_eq(signaling::CoreSignalingSurface);

    for command_kind in [
        signaling::SignalingCommandKind::JoinRoom,
        signaling::SignalingCommandKind::LeaveRoom,
        signaling::SignalingCommandKind::SendOffer,
        signaling::SignalingCommandKind::SendAnswer,
        signaling::SignalingCommandKind::SendIceCandidate,
        signaling::SignalingCommandKind::RequestTurnCredential,
        signaling::SignalingCommandKind::AcknowledgeForward,
    ] {
        touch_hash(command_kind);
    }
    for event_kind in [
        signaling::SignalingEventKind::Joined,
        signaling::SignalingEventKind::Rejected,
        signaling::SignalingEventKind::ParticipantJoined,
        signaling::SignalingEventKind::ParticipantLeft,
        signaling::SignalingEventKind::OfferReceived,
        signaling::SignalingEventKind::AnswerReceived,
        signaling::SignalingEventKind::IceCandidateReceived,
        signaling::SignalingEventKind::TurnCredentialAvailable,
        signaling::SignalingEventKind::ProtocolViolation,
    ] {
        touch_hash(event_kind);
    }

    let subject = signaling::SignalingSubject::new(
        room("signaling-room"),
        Some(participant("signaling-participant")),
    );
    touch_eq(subject.clone());
    let envelope = command::CommandEnvelope::new(
        correlation("signaling-envelope"),
        command::CommandType::new("join_room"),
        command::CommandVersion::new(1),
        command::TargetSurface::Signaling,
        subject.clone(),
    );
    let command = signaling::SignalingCommand::new(
        envelope,
        signaling::SignalingCommandKind::JoinRoom,
        "payload",
    );
    touch_eq(command.clone());
    assert_eq!(
        command.envelope().target_surface(),
        command::TargetSurface::Signaling
    );
    let event = signaling::SignalingEvent::new(
        correlation("signaling-event"),
        signaling::SignalingEventKind::ParticipantJoined,
        subject,
        "event-payload",
    );
    touch_eq(event.clone());
    assert_eq!(
        event.kind(),
        signaling::SignalingEventKind::ParticipantJoined
    );
    touch_eq(
        signaling::SignalingDecision::new(command_decision(
            command::TargetSurface::Signaling,
            command::UseCaseOutcome::Accepted,
            command::DecisionReason::Absent,
        ))
        .expect("signaling decision accepts signaling target surface"),
    );
    let scope = signaling::SignalingSuccessScope::SignalingRoomMembershipOnly;
    touch_hash(scope);
    for state in [
        signaling::RoomState::RoomAbsent,
        signaling::RoomState::RoomOpen,
        signaling::RoomState::RoomDraining,
        signaling::RoomState::RoomClosed,
    ] {
        touch_hash(state);
    }
    for state in [
        signaling::ParticipantState::ParticipantNew,
        signaling::ParticipantState::ParticipantVerifying,
        signaling::ParticipantState::ParticipantJoined,
        signaling::ParticipantState::ParticipantLeaving,
        signaling::ParticipantState::ParticipantLeft,
        signaling::ParticipantState::ParticipantRejected,
    ] {
        touch_hash(state);
    }
    for error in [
        signaling::SignalingContractError::WrongTargetSurface,
        signaling::SignalingContractError::SuccessMustNotCarryReason,
        signaling::SignalingContractError::NonSuccessRequiresReason,
    ] {
        touch_hash(error);
    }

    for prohibited in [
        sfu::ProhibitedSfuSemantic::MedicalWorkflowPriority,
        sfu::ProhibitedSfuSemantic::ApplicationSpecificRoomPolicy,
        sfu::ProhibitedSfuSemantic::UiState,
        sfu::ProhibitedSfuSemantic::RecordingPolicy,
        sfu::ProhibitedSfuSemantic::ChatSemantics,
        sfu::ProhibitedSfuSemantic::ScreenShareWorkflow,
        sfu::ProhibitedSfuSemantic::DataChannelApplicationSemantics,
        sfu::ProhibitedSfuSemantic::RegulatedDataClassification,
        sfu::ProhibitedSfuSemantic::ConcreteWorkerThreadStrategy,
        sfu::ProhibitedSfuSemantic::PacketBytesOwnership,
        sfu::ProhibitedSfuSemantic::PacketCache,
        sfu::ProhibitedSfuSemantic::TransmitQueue,
        sfu::ProhibitedSfuSemantic::CodecImplementationTranscodingBackend,
    ] {
        touch_hash(prohibited);
    }

    for packet_class in [sfu::PacketClass::Rtp, sfu::PacketClass::Rtcp] {
        touch_hash(packet_class);
    }
    let packet_header =
        sfu::PacketHeaderSemanticView::new(sfu::PacketClass::Rtp, Some(100), Some(1234), Some(42));
    touch_hash(packet_header);
    let packet_id = packet("sfu-borrowed-packet");
    let stream_id = stream("sfu-borrowed-stream");
    let endpoint_id = endpoint("sfu-source-endpoint");
    let packet_view = sfu::SfuPacketView::new(
        &packet_id,
        &stream_id,
        &endpoint_id,
        packet_header,
        &[0x80, 0x60, 0x00, 0x01],
        &[0x11, 0x22],
    );
    touch_eq(packet_view);
    assert_eq!(packet_view.packet_id().as_str(), "sfu-borrowed-packet");
    assert_eq!(packet_view.payload_len(), 2);
    for reason in [
        sfu::PacketReleaseReason::Forwarded,
        sfu::PacketReleaseReason::SuppressedByRoutingDecision,
        sfu::PacketReleaseReason::SuppressedByQualityDecision,
        sfu::PacketReleaseReason::SuppressedByBackpressure,
        sfu::PacketReleaseReason::DroppedByBackpressure,
        sfu::PacketReleaseReason::DroppedByTransmitQueueBound,
        sfu::PacketReleaseReason::TargetUnavailable,
        sfu::PacketReleaseReason::SendFailed,
        sfu::PacketReleaseReason::ExpiredFromPacketCache,
        sfu::PacketReleaseReason::EvictedByCacheBound,
        sfu::PacketReleaseReason::TransformFailed,
        sfu::PacketReleaseReason::ReleaseFailed,
        sfu::PacketReleaseReason::DriverShutdown,
    ] {
        touch_hash(reason);
        if let Some(code) = reason.reason_code() {
            assert!(CatalogedReasonRef::from_code(code).is_ok());
        }
    }
    for field in [
        sfu::PacketSemanticField::PacketId,
        sfu::PacketSemanticField::CorrelationId,
        sfu::PacketSemanticField::SourceEndpointId,
        sfu::PacketSemanticField::StreamId,
        sfu::PacketSemanticField::MediaKind,
        sfu::PacketSemanticField::PacketKind,
        sfu::PacketSemanticField::SequenceNumber,
        sfu::PacketSemanticField::Timestamp,
        sfu::PacketSemanticField::SsrcRef,
        sfu::PacketSemanticField::PayloadTypeRef,
        sfu::PacketSemanticField::Marker,
        sfu::PacketSemanticField::PacketLength,
        sfu::PacketSemanticField::ArrivalTime,
    ] {
        touch_hash(field);
    }
    touch_hash(sfu::PacketSemanticMetadata::new(
        Some(sfu::MediaKind::Video),
        sfu::PacketClass::Rtp,
        Some(101),
        Some(5678),
        Some(sfu::SsrcRef::new(2)),
        Some(sfu::PayloadTypeRef::new(97)),
        Some(true),
        1200,
    ));
    for failure in [
        sfu::PacketSemanticViewFailureKind::ExternalDecodeFailed,
        sfu::PacketSemanticViewFailureKind::FrameSizeBoundExceeded,
        sfu::PacketSemanticViewFailureKind::UnsupportedMediaContractVersion,
        sfu::PacketSemanticViewFailureKind::BufferReleaseFailed,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    let rewrite_intent = sfu::PacketRewriteTransformIntent::new(
        route("rewrite-route"),
        Some(endpoint("rewrite-target-endpoint")),
        sfu::PacketRewriteTransformClass::HeaderRewriteOnly,
        packet("rewrite-source-packet"),
        Some(packet("rewrite-target-packet")),
        Some("rewrite-map"),
        sfu::RewriteCopyAllowanceClass::HeaderOnlyAllocationPreferred,
    );
    touch_eq(rewrite_intent.clone());
    assert_eq!(
        rewrite_intent.class(),
        sfu::PacketRewriteTransformClass::HeaderRewriteOnly
    );
    for failure in [
        sfu::PacketRewriteTransformFailureKind::PacketRewriteClassNotAdmitted,
        sfu::PacketRewriteTransformFailureKind::PacketRewriteIntentInvalid,
        sfu::PacketRewriteTransformFailureKind::PacketRewriteOwnerViolation,
        sfu::PacketRewriteTransformFailureKind::PayloadTransformNotAdmitted,
        sfu::PacketRewriteTransformFailureKind::MediaTranscodeNotSupported,
        sfu::PacketRewriteTransformFailureKind::PayloadTransformFailed,
        sfu::PacketRewriteTransformFailureKind::RewriteCopyBoundExceeded,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    touch_eq(sfu::MediaNegotiationMapping::new(
        "media-v1",
        sfu::MediaNegotiationClass::PayloadTypeMapping,
        Some(sfu::CodecProfileRef::new("opus")),
        Some(sfu::TrackRef::new("audio-main")),
        Some(sfu::MediaLayerRef::new("base")),
        Some(sfu::PayloadTypeRef::new(111)),
        Some(sfu::SsrcRef::new(4)),
    ));
    for action in [
        sfu::CongestionPolicyAction::Delay,
        sfu::CongestionPolicyAction::Degrade,
        sfu::CongestionPolicyAction::Suppress,
        sfu::CongestionPolicyAction::Drop,
        sfu::CongestionPolicyAction::CloseByPolicy,
        sfu::CongestionPolicyAction::RejectRecovery,
    ] {
        touch_hash(action);
    }
    touch_hash(sfu::CongestionInput::new(
        sfu::CongestionObservationClass::PacketLossSignal,
        13,
    ));
}
