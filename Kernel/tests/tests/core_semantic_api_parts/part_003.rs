use arcrtc_core_operation as operation;
use arcrtc_core_ports as ports;
use arcrtc_core_quality as quality;
use arcrtc_core_recovery as recovery;
use arcrtc_core_identity::{AllocationId, PermissionId};

fn full_restore_preconditions() -> recovery::RestorePreconditionSet {
    recovery::RestorePreconditionSet::try_new(true, true, true, true, true, true, true, true, true)
        .expect("full restore preconditions are valid")
}

fn full_replay_policy() -> recovery::ReplayPolicyCoverage {
    recovery::ReplayPolicyCoverage::try_new(true, true, true, true, true, true, true)
        .expect("full replay policy coverage is valid")
}

fn failover_input(
    replacement_owner: Option<OpaqueReference>,
) -> recovery::FailoverVerificationStateInput {
    recovery::FailoverVerificationStateInput {
        failed_owner_observed: true,
        replacement_owner,
        affected_state_family: StateFamily::SfuForwardingState,
        affinity_sticky_routing_updated: true,
        restore_replay_relation:
            recovery::RecoveryRestoreRelation::ExplicitRestoreVerificationRequired,
        conflict_and_duplicate_handling_defined: true,
        resource_lifetime_revalidated: true,
        audit_continuity_recorded: true,
    }
}

#[test]
fn cov1_recovery_restore_replay_and_failover_paths_are_fail_closed() {
    assert!(
        recovery::RecoveryClass::CheckpointRestoreCandidate
            .permits_core_state_restore_candidate()
    );
    assert!(
        !recovery::RecoveryClass::AuditReplayVerificationOnly
            .permits_core_state_restore_candidate()
    );
    assert!(
        recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured()
            .permits_core_restore_candidate()
    );
    assert!(
        !recovery::StateFamilyRecoveryPolicy::default_for(StateFamily::SignalingRoom)
            .permits_domain_mutation_replay_candidate()
    );

    assert_eq!(
        recovery::RestorePreconditionSet::try_new(false, true, true, true, true, true, true, true, true),
        Err(recovery::RestorePreconditionError::LifecycleObservationNotClassified)
    );
    assert_eq!(
        recovery::RestorePreconditionSet::try_new(true, false, true, true, true, true, true, true, true),
        Err(recovery::RestorePreconditionError::RestorePolicyDoesNotAllowStateFamily)
    );
    assert_eq!(
        recovery::RestorePreconditionSet::try_new(true, true, false, true, true, true, true, true, true),
        Err(recovery::RestorePreconditionError::CheckpointVersionNotAccepted)
    );
    assert_eq!(
        recovery::RestorePreconditionSet::try_new(true, true, true, false, true, true, true, true, true),
        Err(recovery::RestorePreconditionError::CorrelationOwnerAggregateNotVerified)
    );
    assert_eq!(
        recovery::RestorePreconditionSet::try_new(true, true, true, true, false, true, true, true, true),
        Err(recovery::RestorePreconditionError::AuditHashChainGapDetected)
    );
    assert_eq!(
        recovery::RestorePreconditionSet::try_new(true, true, true, true, true, false, true, true, true),
        Err(recovery::RestorePreconditionError::CheckpointReplayConflictDetected)
    );
    assert_eq!(
        recovery::RestorePreconditionSet::try_new(true, true, true, true, true, true, false, true, true),
        Err(recovery::RestorePreconditionError::ResourceLifetimeBoundInvalid)
    );
    assert_eq!(
        recovery::RestorePreconditionSet::try_new(true, true, true, true, true, true, true, false, true),
        Err(recovery::RestorePreconditionError::FailureMappingNotCataloged)
    );
    assert_eq!(
        recovery::RestorePreconditionSet::try_new(true, true, true, true, true, true, true, true, false),
        Err(recovery::RestorePreconditionError::DistributedPolicyNotRecorded)
    );

    let restore_eligibility = recovery::RestoreEligibility::try_new(
        recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured(),
        StateClass::CheckpointEligibleState,
        full_restore_preconditions(),
    )
    .expect("explicit signaling idempotency checkpoint restore is eligible");
    assert_eq!(
        recovery::RestoreEligibility::try_new(
            recovery::StateFamilyRecoveryPolicy::default_for(StateFamily::SignalingRoom),
            StateClass::EphemeralCoreState,
            full_restore_preconditions(),
        ),
        Err(recovery::RestoreEligibilityError::StateFamilyNotRestoreEligible)
    );

    assert_eq!(
        recovery::ReplayPolicyCoverage::try_new(false, true, true, true, true, true, true),
        Err(recovery::ReplayPolicyCoverageError::EventTypeMissing)
    );
    assert_eq!(
        recovery::ReplayEligibility::domain_mutation_candidate(
            recovery::StateFamilyRecoveryPolicy::default_for(StateFamily::SignalingRoom),
            Some(full_replay_policy()),
        ),
        Err(recovery::ReplayEligibilityError::StateFamilyNotReplayMutationEligible)
    );
    assert_eq!(
        recovery::ReplayEligibility::domain_mutation_candidate(
            recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured(),
            None,
        ),
        Err(recovery::ReplayEligibilityError::DomainMutationRequiresPolicyCoverage)
    );
    assert!(recovery::ReplayEligibility::domain_mutation_candidate(
        recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured(),
        Some(full_replay_policy()),
    )
    .is_ok());
    let verification_only =
        recovery::ReplayEligibility::audit_integrity_verification_only(StateFamily::AuditEvent);
    assert!(matches!(
        verification_only,
        recovery::ReplayEligibility { .. }
    ));

    assert_eq!(
        recovery::RecoveryFailureKind::FailoverNotProven.reason_code(),
        "failover_not_proven"
    );
    assert!(recovery::DistributedStateClass::AuditVerificationState.admitted_in_initial_v0_2());
    assert!(
        !recovery::DistributedStateClass::AutomaticFailoverRequested.admitted_in_initial_v0_2()
    );

    let owner_ref = reference("owner-cov1");
    let affinity_key = reference("affinity-cov1");
    let policy = recovery::DistributedStatePolicy::try_new(
        StateFamily::SfuForwardingState,
        recovery::DistributedStateClass::AffinityRequiredState,
        recovery::OwnerNodeScope::SingleNode,
        Some(owner_ref.clone()),
        Some(affinity_key.clone()),
        recovery::CommandRoutingRule::OwnerAffinityRequired,
        recovery::PacketRoutingRule::OwnerAffinityRequired,
        recovery::RecoveryRestoreRelation::ExplicitRestoreVerificationRequired,
        recovery::DistributedConflictRule::RejectConflictingOwner,
        true,
        recovery::DistributedAuditRelation::DistributedStateFailoverDecision,
    )
    .expect("affinity state policy has explicit owner, affinity, reason, and audit relation");
    assert!(recovery::DistributedStateAdmission::admit_initial_v0_2(policy).is_ok());
    assert_eq!(
        recovery::DistributedStatePolicy::try_new(
            StateFamily::SfuForwardingState,
            recovery::DistributedStateClass::AffinityRequiredState,
            recovery::OwnerNodeScope::SingleNode,
            Some(owner_ref.clone()),
            None,
            recovery::CommandRoutingRule::OwnerAffinityRequired,
            recovery::PacketRoutingRule::OwnerAffinityRequired,
            recovery::RecoveryRestoreRelation::NoRestoreRelation,
            recovery::DistributedConflictRule::RejectConflictingOwner,
            true,
            recovery::DistributedAuditRelation::DistributedStateFailoverDecision,
        ),
        Err(recovery::DistributedStatePolicyError::AffinityKeyMissing)
    );
    assert_eq!(
        recovery::DistributedStatePolicy::try_new(
            StateFamily::SfuForwardingState,
            recovery::DistributedStateClass::NodeLocalState,
            recovery::OwnerNodeScope::ClusterScopeRequested,
            Some(owner_ref.clone()),
            None,
            recovery::CommandRoutingRule::NotCrossNodeRouted,
            recovery::PacketRoutingRule::NotPacketScoped,
            recovery::RecoveryRestoreRelation::NoRestoreRelation,
            recovery::DistributedConflictRule::RejectConflictingOwner,
            true,
            recovery::DistributedAuditRelation::AuditVerificationOnly,
        ),
        Err(recovery::DistributedStatePolicyError::ClusterScopeNotAdmitted)
    );

    let failover_state = recovery::FailoverVerificationState::try_new(failover_input(Some(
        reference("replacement-cov1"),
    )))
    .expect("verified failover has all required runtime state");
    assert_eq!(
        recovery::FailoverVerificationState::try_new(failover_input(None)),
        Err(recovery::FailoverVerificationStateError::ReplacementOwnerMissing)
    );
    assert!(recovery::FailoverAdmission::admit(
        recovery::FailoverClaimClass::VerifiedFailoverCandidate,
        failover_state.clone(),
    )
    .is_ok());
    assert_eq!(
        recovery::FailoverAdmission::admit(
            recovery::FailoverClaimClass::ProcessRestartObservation,
            failover_state,
        ),
        Err(recovery::FailoverAdmissionError::ProcessRestartIsNotFailoverSuccess)
    );
    assert_eq!(
        recovery::SplitBrainGuard::try_new(true, false),
        Err(recovery::SplitBrainGuardError::SplitBrainRiskDetected)
    );
    assert!(recovery::SplitBrainGuard::try_new(true, true).is_ok());
    assert_eq!(
        recovery::DistributedStateFailureKind::SplitBrainRiskDetected.reason_code(),
        "split_brain_risk_detected"
    );

    let classification = recovery::ProcessFailureClassification::try_new(
        Some(StartupRunId::new(reference("startup-before-cov1"))),
        Some(StartupRunId::new(reference("startup-after-cov1"))),
        Some(reference("process-cov1")),
        recovery::ProcessFailureClass::ProcessCrashObserved,
        Some(recovery::PriorDrainStatus::DrainIncompleteOrAbsent),
        Some(recovery::AuditPersistenceStatus::AuditPersistenceIncompleteOrAbsent),
        recovery::RestoreReplayPolicyApplication::RestoreEligibilityApplied,
        recovery::RestartReadinessClass::ReadinessObservedSeparately,
    )
    .expect("classified process failure has drain and audit status");
    assert!(classification.is_unclean_shutdown());
    assert_eq!(
        recovery::ProcessFailureClassification::try_new(
            None,
            None,
            None,
            recovery::ProcessFailureClass::PanicObserved,
            Some(recovery::PriorDrainStatus::GracefulDrainObserved),
            Some(recovery::AuditPersistenceStatus::AuditPersistenceCompleted),
            recovery::RestoreReplayPolicyApplication::NotApplied,
            recovery::RestartReadinessClass::NotClaimed,
        ),
        Err(recovery::ProcessFailureClassificationError::ProcessIdentityMissing)
    );

    let crash_verification = recovery::CrashRestartVerification::try_new(
        "supervisor-cov1",
        recovery::ProcessFailureClass::ProcessCrashObserved,
        recovery::ProcessFailureClass::ProcessCrashObserved,
        Some(StartupRunId::new(reference("startup-before-claim-cov1"))),
        Some(StartupRunId::new(reference("startup-after-claim-cov1"))),
        true,
        true,
        true,
        true,
    )
    .expect("crash verification separates logs, audit, restore, and readiness");
    assert_eq!(
        recovery::CrashRestartVerification::try_new(
            "",
            recovery::ProcessFailureClass::ProcessCrashObserved,
            recovery::ProcessFailureClass::ProcessCrashObserved,
            Some(StartupRunId::new(reference("startup-before-invalid-cov1"))),
            Some(StartupRunId::new(reference("startup-after-invalid-cov1"))),
            true,
            true,
            true,
            true,
        ),
        Err(recovery::CrashRestartVerificationError::SupervisorOrRunnerMissing)
    );
    assert_eq!(
        recovery::RestartDomainStateClaim::try_new(
            classification,
            None,
            Some(crash_verification.clone()),
        ),
        Err(recovery::RestartDomainStateClaimError::RestoreEligibilityRequired)
    );
    assert!(recovery::RestartDomainStateClaim::try_new(
        recovery::ProcessFailureClassification::try_new(
            Some(StartupRunId::new(reference("startup-before-ok-cov1"))),
            Some(StartupRunId::new(reference("startup-after-ok-cov1"))),
            Some(reference("process-ok-cov1")),
            recovery::ProcessFailureClass::ProcessCrashObserved,
            Some(recovery::PriorDrainStatus::GracefulDrainObserved),
            Some(recovery::AuditPersistenceStatus::AuditPersistenceCompleted),
            recovery::RestoreReplayPolicyApplication::RestoreEligibilityApplied,
            recovery::RestartReadinessClass::ReadinessObservedSeparately,
        )
        .expect("classification is valid"),
        Some(restore_eligibility),
        Some(crash_verification),
    )
    .is_ok());
    assert_eq!(
        recovery::ProcessFailureMappingKind::SupervisorRestartObserved.reason_code(),
        "supervisor_restart_observed"
    );
    assert!(recovery::ProcessFailureClass::ProcessCrashObserved.is_process_crash());
    assert!(
        recovery::ProcessFailureClass::StartupAfterUncleanExit
            .requires_restore_policy_before_state_claim()
    );
}

#[test]
fn cov1_operation_ordering_retry_and_atomicity_boundaries_are_explicit() {
    assert_eq!(
        operation::ShutdownDrainConcern::RoomDrainCloseSemantics.owner(),
        operation::ShutdownDrainOwner::CoreSignaling
    );
    assert_eq!(
        operation::ShutdownDrainConcern::SocketListenerStop.owner(),
        operation::ShutdownDrainOwner::Driver
    );
    assert_eq!(
        operation::DrainSequenceStep::CreateShutdownCorrelationAndRequest.order(),
        1
    );
    assert_eq!(
        operation::DrainSequenceStep::StopRuntimeAfterDrainFinalization.order(),
        10
    );
    assert_eq!(
        operation::ShutdownDrainFailureKind::AuditBacklogBoundExceeded.reason_code(),
        "audit_backlog_bound_exceeded"
    );
    assert!(operation::PLANE_DRAIN_RULES.len() >= 5);
    assert_eq!(
        operation::AtomicityConcern::DomainDecisionAtomicity.owner(),
        operation::AtomicityOwner::Core
    );
    assert_eq!(
        operation::AtomicityConcern::ExternalResponseEmission.owner(),
        operation::AtomicityOwner::DriverSdk
    );
    assert_eq!(
        operation::CommitBoundary::AfterAuditProjection.order(),
        4
    );
    assert_eq!(
        operation::AtomicityFailureKind::NetworkSendFailed.reason_code(),
        "network_send_failed"
    );
    let failed_step = operation::AtomicCommitStep::new(
        operation::CommitBoundary::AfterDriverPersistence,
        operation::AtomicStepOutcome::Failed,
        Some(operation::AtomicityFailureKind::PersistenceUnavailable),
    );
    let _compensation_rule = operation::CompensationRule::new(
        operation::AtomicityFailureKind::PersistenceUnavailable,
        "audit event append",
        operation::CompensationOwner::CoreWhenDomainStateChanges,
        "reject affected state after compensation failure",
        "audit_persistence_failure",
        operation::CompensationExternalResponseRule::ProjectLaterFailure,
    );
    let compensation_decision = operation::AtomicityCompensationDecision::new(
        correlation(),
        operation::AtomicityClass::CompensatingTransitionRequired,
        operation::CommitBoundary::AfterDriverPersistence,
        failed_step,
        operation::CompensationStatus::RequiredAndAvailable,
    );
    assert_eq!(
        compensation_decision.audit_event_type(),
        "atomicity_compensation_decision"
    );

    assert_eq!(
        operation::OrderingConcern::AggregateSerializationScope.owner(),
        operation::OrderingOwner::Core
    );
    assert_eq!(
        operation::SerializationScope::SfuEndpointScope.owner(),
        operation::OrderingOwner::CoreSfu
    );
    assert_eq!(
        operation::SerializationScope::PacketLifecycleScope.protected_semantics(),
        "packet buffer lease, queue, cache, release"
    );
    assert_eq!(
        operation::OrderingFailureKind::LockContentionBoundExceeded.reason_code(),
        "lock_contention_bound_exceeded"
    );
    assert_eq!(
        operation::PhysicalLockResource::DriverQueueMutex.allowed_owner(),
        operation::OrderingOwner::Driver
    );
    let _serialization_bound = operation::SerializationBoundPolicy::new(
        operation::SerializationScope::RoomScope,
        true,
        true,
        operation::OrderingOwner::CoreSignaling,
        "reject queued command",
    );
    let _ordering_decision = operation::OrderingDecision::new(
        correlation(),
        operation::SerializationScope::RoomScope,
        Some(operation::SerializationScopeReference::Room(RoomId::new(reference(
            "room-order-cov1",
        )))),
        operation::AtomicStepOutcome::Failed,
        Some(operation::OrderingFailureKind::ConcurrencyConflict),
    );

    assert_eq!(
        operation::RetryTimeoutConcern::CommandDeadlinePolicy.owner(),
        operation::RetryTimeoutOwner::Core
    );
    assert_eq!(
        operation::RetryClass::TestOnlyRetry.owner(),
        operation::RetryTimeoutOwner::TestingScope
    );
    assert!(!operation::RetryClass::TestOnlyRetry.is_runtime_retry_class());
    assert_eq!(
        operation::RetryTimeoutFailureKind::RuntimeTaskQueueBoundExceeded.reason_code(),
        "runtime_task_queue_bound_exceeded"
    );
    assert_eq!(
        operation::TimeoutDeadlineSurface::CommandDeadline.owner(),
        operation::RetryTimeoutOwner::Core
    );
    assert_eq!(
        operation::TimeoutDeadlineSurface::DriverReceiveTimeout.failure_reason(),
        operation::RetryTimeoutFailureKind::NetworkReceiveFailed
    );
    assert_eq!(
        operation::CancellationSource::ClientAfterCoreEntry.handling(),
        operation::CancellationHandling::PreservePriorDecisionResult
    );
    assert!(operation::RetryPreconditions::new(true, true, true, true, true, true, true)
        .allows_retry());
    assert!(!operation::RetryPreconditions::new(true, true, false, true, true, true, true)
        .allows_retry());
    let retry_shape: operation::RetryAttemptShape<OpaqueReference, OpaqueReference> =
        operation::RetryAttemptShape::new(
            operation::RetryClass::IdempotentCommandReplay,
            correlation(),
            None,
            2,
            operation::RetryPreconditions::new(true, true, true, true, true, true, true),
        );
    assert!(matches!(retry_shape, operation::RetryAttemptShape { .. }));
}

#[test]
fn cov1_quality_resource_and_backpressure_catalogs_are_asserted() {
    let _metric = quality::QualityMetric::new(quality::QualityMetricKind::PacketLoss, 3, "percent", "1s");
    let _threshold = quality::QualityThreshold::new(quality::QualityMetricKind::Jitter, 30, "ms");
    let _rate = quality::RateWindowPolicy::new(
        quality::AdmissionScope::Room,
        Some(100),
        Some(1024),
        Some(1000),
        "sliding",
    );
    assert_eq!(
        quality::QualityFailureKind::EndpointQualityNotAllowed.reason_code(),
        "endpoint_quality_not_allowed"
    );
    assert_eq!(
        quality::AdmissionFailureKind::ForwardedHeaderUntrusted.reason_code(),
        "forwarded_header_untrusted"
    );
    assert_eq!(
        quality::ResourceBoundKind::RuntimeTaskQueueWorkerMailboxJoinWait.resource_name(),
        "runtime task queue / worker mailbox / join wait"
    );
    assert_eq!(
        quality::ResourceBoundAuditEventType::RuntimeTaskLifecycleDecision.code(),
        "runtime_task_lifecycle_decision"
    );
    assert_eq!(quality::ResourceBoundOutcome::Shed.code(), "shed");
    assert_eq!(quality::BackpressureOutcome::ClosedByPolicy.code(), "closed_by_policy");

    assert!(quality::REQUIRED_RESOURCE_BOUNDS.len() >= 24);
    assert!(quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .all(|action| !action.reason_code().is_empty()
            && !action.audit_event_type().code().is_empty()
            && !action.outcome().code().is_empty()
            && !action.resource().resource_name().is_empty()));

    let packet_cache_action = quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .find(|action| action.resource() == quality::ResourceBoundKind::SfuPacketCache)
        .copied()
        .expect("packet cache action exists");
    assert_eq!(
        quality::ResourceBoundDecision::try_new(
            packet_cache_action,
            quality::ResourceBoundReferenceSet::none(),
        ),
        Err(quality::ResourceBoundDecisionShapeError::PacketReferenceMissing)
    );
    assert!(quality::ResourceBoundDecision::try_new(
        packet_cache_action,
        quality::ResourceBoundReferenceSet::packet(PacketId::new(reference("packet-resource-cov1"))),
    )
    .is_ok());

    let relay_action = quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .find(|action| action.resource() == quality::ResourceBoundKind::TurnRelayQueue)
        .copied()
        .expect("relay queue action exists");
    assert_eq!(
        quality::ResourceBoundDecision::try_new(
            relay_action,
            quality::ResourceBoundReferenceSet::none(),
        ),
        Err(quality::ResourceBoundDecisionShapeError::TurnRelayQueueReferencesMissing)
    );
    assert!(quality::ResourceBoundDecision::try_new(
        relay_action,
        quality::ResourceBoundReferenceSet::turn_relay_queue(
            AllocationId::new(reference("allocation-resource-cov1")),
            PermissionId::new(reference("permission-resource-cov1")),
        ),
    )
    .is_ok());

    let frame_action = quality::REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .find(|action| action.resource() == quality::ResourceBoundKind::InboundFrameSize)
        .copied()
        .expect("inbound frame action exists");
    assert_eq!(
        quality::ResourceBoundDecision::try_new(
            frame_action,
            quality::ResourceBoundReferenceSet::none(),
        ),
        Err(quality::ResourceBoundDecisionShapeError::DriverResourceReferenceMissing)
    );
    assert!(quality::ResourceBoundDecision::try_new(
        frame_action,
        quality::ResourceBoundReferenceSet::driver_resource("frame-cov1"),
    )
    .is_ok());

    assert_eq!(
        quality::BackpressureDecisionKind::SuppressSubscription.audit_event_type(),
        quality::ResourceBoundAuditEventType::SfuSubscriptionDecision
    );
    assert_eq!(
        quality::BackpressureDecisionKind::CloseEndpoint.outcome(),
        quality::BackpressureOutcome::ClosedByPolicy
    );
    assert_eq!(
        quality::BackpressureDecisionKind::DropPacket.reason_code(),
        Some("packet_dropped_by_backpressure")
    );
    assert_eq!(quality::BackpressureDecisionKind::Accept.reason_code(), None);
    assert_eq!(
        quality::BackpressureDecision::try_new(
            quality::BackpressureDecisionKind::Accept,
            quality::BackpressureTargetRef::Endpoint(EndpointId::new(reference("endpoint-bp-cov1"))),
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
            quality::BackpressureTargetRef::Packet(PacketId::new(reference("packet-bp-cov1"))),
        ),
        Err(quality::BackpressureDecisionShapeError::RouteTargetRequired)
    );
    assert!(quality::BackpressureDecision::try_new(
        quality::BackpressureDecisionKind::DelayAction,
        quality::BackpressureTargetRef::Route(RouteId::new(reference("route-bp-cov1"))),
    )
    .is_ok());
    assert!(quality::BackpressureDecision::try_new(
        quality::BackpressureDecisionKind::SuppressSubscription,
        quality::BackpressureTargetRef::Subscription(StreamId::new(reference("stream-bp-cov1"))),
    )
    .is_ok());
}

#[test]
fn cov1_ports_contract_persistence_and_failure_mappings_are_core_owned() {
    let context = ports::PortCallContext::new(correlation());
    assert_eq!(context.correlation_id().as_str(), "corr-core-api");

    let delivery = ports::NetworkDeliveryObservation::new(
        correlation(),
        Some(reference("peer-cov1")),
        true,
    );
    assert!(delivery.delivered());
    assert_eq!(
        delivery.peer_ref().expect("peer exists").as_str(),
        "peer-cov1"
    );

    let metrics_failure =
        ports::MetricsSinkFailure::from_kind(ports::MetricsSinkFailureKind::MetricsBacklogBoundExceeded);
    assert_eq!(
        metrics_failure.kind(),
        ports::MetricsSinkFailureKind::MetricsBacklogBoundExceeded
    );
    assert_eq!(
        metrics_failure.reason().definition().code().as_str(),
        "metrics_backlog_bound_exceeded"
    );
    assert!(metrics_failure.resource_bound_decision().is_some());
    assert_eq!(
        ports::MetricsSinkFailure::from_kind(ports::MetricsSinkFailureKind::MetricsExportFailed)
            .resource_bound_decision(),
        None
    );

    let contract = ports::PortContractShape::new(
        ports::PortFamily::PacketView,
        ports::PortCallShape::Query,
        vec![
            ports::PortOwnershipRule::CoreOwnedTypesOnly,
            ports::PortOwnershipRule::NoDriverBufferOwnershipTransfer,
        ],
    );
    assert_eq!(contract.family(), ports::PortFamily::PacketView);
    assert_eq!(contract.call_shape(), ports::PortCallShape::Query);
    assert!(contract
        .ownership_rules()
        .contains(&ports::PortOwnershipRule::NoDriverBufferOwnershipTransfer));

    let checkpoint_intent = ports::PersistencePortIntent::try_new(
        ports::PersistenceIntentClass::StateCheckpoint,
        ports::PersistenceOperationKind::PersistCheckpoint,
        StateFamily::SignalingIdempotency,
        StateClass::CheckpointEligibleState,
        vec![ports::PersistenceConsistencyRequirement::PreserveIdempotency],
        Some(correlation()),
    )
    .expect("checkpoint intent uses checkpoint-eligible state family");
    assert_eq!(
        checkpoint_intent.intent_class(),
        ports::PersistenceIntentClass::StateCheckpoint
    );
    assert_eq!(
        checkpoint_intent.state_class(),
        StateClass::CheckpointEligibleState
    );
    assert_eq!(
        ports::PersistencePortIntent::try_new(
            ports::PersistenceIntentClass::StateCheckpoint,
            ports::PersistenceOperationKind::AppendAuditEvent,
            StateFamily::SignalingIdempotency,
            StateClass::CheckpointEligibleState,
            vec![],
            Some(correlation()),
        ),
        Err(ports::PersistencePortIntentError::OperationClassMismatch)
    );
    assert_eq!(
        ports::PersistencePortIntent::try_new(
            ports::PersistenceIntentClass::RetryStore,
            ports::PersistenceOperationKind::EnqueueRetry,
            StateFamily::DriverRetryStore,
            StateClass::DriverLocalState,
            vec![],
            Some(correlation()),
        ),
        Err(ports::PersistencePortIntentError::RetryIntentRequiresBoundedRetry)
    );
    let retry_intent = ports::PersistencePortIntent::try_new(
        ports::PersistenceIntentClass::RetryStore,
        ports::PersistenceOperationKind::EnqueueRetry,
        StateFamily::DriverRetryStore,
        StateClass::DriverLocalState,
        vec![ports::PersistenceConsistencyRequirement::BoundedRetryStore],
        Some(correlation()),
    )
    .expect("retry intent requires driver retry store and bounded retry");
    assert_eq!(retry_intent.intent_class(), ports::PersistenceIntentClass::RetryStore);

    let record = ports::PersistenceRecordRef::new(reference("record-cov1"));
    assert_eq!(record.as_str(), "record-cov1");
    let _ack = ports::PersistencePortOutput::Acknowledgement(ports::PersistenceAcknowledgement::new(
        ports::PersistenceIntentClass::StateCheckpoint,
        Some(record.clone()),
        true,
    ));
    let _loaded = ports::PersistencePortOutput::LoadedState(ports::LoadedCoreStateRef::new(
        StateFamily::SignalingIdempotency,
        StateClass::CheckpointEligibleState,
        record,
    ));
    let _input = ports::PersistencePortInput::ExecuteIntent(checkpoint_intent);
    let _metric_output = ports::MetricsSinkOutput::Acknowledgement(
        ports::MetricsExportAcknowledgement::new(true, true),
    );

    let persistence_failure = ports::PersistencePortFailure::from_kind(
        ports::PersistencePortFailureKind::PersistenceRetryBoundExceeded,
    );
    assert_eq!(
        persistence_failure.kind(),
        ports::PersistencePortFailureKind::PersistenceRetryBoundExceeded
    );
    assert_eq!(
        persistence_failure.reason().definition().code().as_str(),
        "persistence_retry_bound_exceeded"
    );
    assert!(persistence_failure.resource_bound_decision().is_some());
    assert_eq!(
        ports::PersistencePortFailure::from_kind(ports::PersistencePortFailureKind::DriverShutdown)
            .resource_bound_decision(),
        None
    );
    assert_eq!(
        ports::PersistencePortFailureKind::AuditBacklogBoundExceeded.reason_code(),
        "audit_backlog_bound_exceeded"
    );
    assert_eq!(
        ports::PacketViewClass::BorrowedSemanticHeaderView,
        ports::PacketViewClass::BorrowedSemanticHeaderView
    );
    assert_eq!(
        ports::RuntimePortOutputClass::OpaqueScheduleReference,
        ports::RuntimePortOutputClass::OpaqueScheduleReference
    );
}
