use arcrtc_core_command::{
    AuditProjectionRequirement, CommandType, DecisionReason, StateTransitionSummary, TargetSurface,
    UseCaseDecision, UseCaseDecisionInput, UseCaseOutcome,
};
use arcrtc_core_identity::{
    AllocationId, ChannelBindId, ConfigurationScopeRef, CorrelationId, CredentialRef, EndpointId,
    OpaqueReference, PacketId, PermissionId, ReferenceAuthority, RouteId, SessionId, StartupRunId,
    StreamId,
};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_recovery as recovery;
use arcrtc_core_runtime as runtime;
use arcrtc_core_sfu as sfu;
use arcrtc_core_state::{StateClass, StateFamily};
use arcrtc_core_turn as turn;

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("reference is valid")
}

fn correlation(value: &str) -> CorrelationId {
    CorrelationId::new(reference(value))
}

fn startup(value: &str) -> StartupRunId {
    StartupRunId::new(reference(value))
}

fn cataloged(code: &'static str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason code is cataloged")
}

fn assert_reason_is_cataloged(code: &'static str) {
    assert_eq!(cataloged(code).definition().code().as_str(), code);
}

fn full_restore_preconditions() -> recovery::RestorePreconditionSet {
    recovery::RestorePreconditionSet::try_new(true, true, true, true, true, true, true, true, true)
        .expect("all restore preconditions are present")
}

fn full_replay_policy() -> recovery::ReplayPolicyCoverage {
    recovery::ReplayPolicyCoverage::try_new(true, true, true, true, true, true, true)
        .expect("all replay policy coverage fields are present")
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

fn crash_verification(
) -> Result<recovery::CrashRestartVerification, recovery::CrashRestartVerificationError> {
    recovery::CrashRestartVerification::try_new(
        "supervisor",
        recovery::ProcessFailureClass::ProcessCrashObserved,
        recovery::ProcessFailureClass::ProcessCrashObserved,
        Some(startup("crash-before")),
        Some(startup("crash-after")),
        true,
        true,
        true,
        true,
    )
}

fn decision(
    label: &'static str,
    target_surface: TargetSurface,
    outcome: UseCaseOutcome,
    reason: DecisionReason<CatalogedReasonRef>,
) -> UseCaseDecision<CatalogedReasonRef> {
    UseCaseDecision::new(UseCaseDecisionInput {
        correlation_id: correlation(label),
        command_type: CommandType::new(label),
        target_surface,
        outcome,
        reason,
        state_transition: StateTransitionSummary::Changed("coverage-enterprise-core-media"),
        port_intents: Vec::new(),
        audit_projection: AuditProjectionRequirement::Required,
    })
    .expect("decision shape is valid")
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

#[test]
fn recovery_restore_replay_reason_and_prohibited_surface_are_asserted() {
    assert_eq!(
        format!("{:?}", recovery::CoreRecoverySurface),
        "CoreRecoverySurface"
    );

    let default_families = [
        StateFamily::SignalingIdempotency,
        StateFamily::AuditEvent,
        StateFamily::AuditHashChainRecord,
        StateFamily::DriverRetryStore,
        StateFamily::MetricsBacklog,
        StateFamily::SdkConnectionState,
    ];
    for family in default_families {
        let policy = recovery::StateFamilyRecoveryPolicy::default_for(family);
        assert!(!policy.permits_core_restore_candidate());
    }

    let metrics_retry = recovery::StateFamilyRecoveryPolicy::metrics_backlog_retry_configured();
    assert!(!metrics_retry.permits_core_restore_candidate());
    assert!(!metrics_retry.permits_domain_mutation_replay_candidate());

    assert_eq!(
        recovery::RestoreEligibility::try_new(
            recovery::StateFamilyRecoveryPolicy::default_for(StateFamily::SignalingIdempotency),
            StateClass::CheckpointEligibleState,
            full_restore_preconditions(),
        ),
        Err(recovery::RestoreEligibilityError::RecoveryClassDoesNotPermitRestore)
    );
    assert_eq!(
        recovery::RestoreEligibility::try_new(
            recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured(),
            StateClass::AuditOnlyState,
            full_restore_preconditions(),
        ),
        Err(recovery::RestoreEligibilityError::StateClassNotCheckpointEligible)
    );

    let restore_eligibility = recovery::RestoreEligibility::try_new(
        recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured(),
        StateClass::CheckpointEligibleState,
        full_restore_preconditions(),
    )
    .expect("checkpoint restore policy is eligible");

    for (coverage, expected) in [
        (
            recovery::ReplayPolicyCoverage::try_new(true, false, true, true, true, true, true),
            recovery::ReplayPolicyCoverageError::OrderingMissing,
        ),
        (
            recovery::ReplayPolicyCoverage::try_new(true, true, false, true, true, true, true),
            recovery::ReplayPolicyCoverageError::GapHandlingMissing,
        ),
        (
            recovery::ReplayPolicyCoverage::try_new(true, true, true, false, true, true, true),
            recovery::ReplayPolicyCoverageError::DuplicateHandlingMissing,
        ),
        (
            recovery::ReplayPolicyCoverage::try_new(true, true, true, true, false, true, true),
            recovery::ReplayPolicyCoverageError::ConflictResolutionMissing,
        ),
        (
            recovery::ReplayPolicyCoverage::try_new(true, true, true, true, true, false, true),
            recovery::ReplayPolicyCoverageError::BoundRevalidationMissing,
        ),
        (
            recovery::ReplayPolicyCoverage::try_new(true, true, true, true, true, true, false),
            recovery::ReplayPolicyCoverageError::FailureReasonMappingMissing,
        ),
    ] {
        assert_eq!(coverage, Err(expected));
    }

    assert_eq!(
        recovery::ReplayEligibility::domain_mutation_candidate(
            recovery::StateFamilyRecoveryPolicy::default_for(StateFamily::SignalingIdempotency),
            Some(full_replay_policy()),
        ),
        Err(recovery::ReplayEligibilityError::RecoveryClassDoesNotPermitDomainMutationReplay)
    );
    assert!(recovery::ReplayEligibility::domain_mutation_candidate(
        recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured(),
        Some(full_replay_policy()),
    )
    .is_ok());
    assert!(matches!(
        recovery::ReplayEligibility::audit_integrity_verification_only(
            StateFamily::AuditHashChainRecord
        ),
        recovery::ReplayEligibility { .. }
    ));

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
        assert_reason_is_cataloged(failure.reason_code());
    }

    let prohibited = [
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
    ];
    assert_eq!(prohibited.len(), 10);
    assert!(format!("{:?}", restore_eligibility).contains("SignalingIdempotency"));
}

#[test]
fn recovery_distributed_failover_and_process_verification_paths_are_asserted() {
    let owner = reference("owner-node");
    assert_eq!(
        recovery::DistributedStatePolicy::try_new(
            StateFamily::SfuForwardingState,
            recovery::DistributedStateClass::NodeLocalState,
            recovery::OwnerNodeScope::SingleNode,
            Some(owner.clone()),
            None,
            recovery::CommandRoutingRule::NotCrossNodeRouted,
            recovery::PacketRoutingRule::NotPacketScoped,
            recovery::RecoveryRestoreRelation::NoRestoreRelation,
            recovery::DistributedConflictRule::RequiresFutureConsensusAdmission,
            true,
            recovery::DistributedAuditRelation::AuditVerificationOnly,
        ),
        Err(recovery::DistributedStatePolicyError::FutureConsensusConflictRuleNotAdmitted)
    );
    assert_eq!(
        recovery::DistributedStatePolicy::try_new(
            StateFamily::SfuForwardingState,
            recovery::DistributedStateClass::NodeLocalState,
            recovery::OwnerNodeScope::SingleNode,
            Some(owner.clone()),
            None,
            recovery::CommandRoutingRule::NotCrossNodeRouted,
            recovery::PacketRoutingRule::NotPacketScoped,
            recovery::RecoveryRestoreRelation::NoRestoreRelation,
            recovery::DistributedConflictRule::RejectConflictingOwner,
            false,
            recovery::DistributedAuditRelation::AuditVerificationOnly,
        ),
        Err(recovery::DistributedStatePolicyError::FailureReasonMappingMissing)
    );

    for (state_class, expected) in [
        (
            recovery::DistributedStateClass::ReplicatedStateRequested,
            recovery::DistributedStateAdmissionError::StateReplicationNotAdmitted,
        ),
        (
            recovery::DistributedStateClass::ConsensusStateRequested,
            recovery::DistributedStateAdmissionError::ConsensusNotAdmitted,
        ),
        (
            recovery::DistributedStateClass::AutomaticFailoverRequested,
            recovery::DistributedStateAdmissionError::FailoverNotProven,
        ),
    ] {
        let policy = recovery::DistributedStatePolicy::try_new(
            StateFamily::SfuForwardingState,
            state_class,
            recovery::OwnerNodeScope::SingleNode,
            Some(owner.clone()),
            None,
            recovery::CommandRoutingRule::NotCrossNodeRouted,
            recovery::PacketRoutingRule::NotPacketScoped,
            recovery::RecoveryRestoreRelation::NoRestoreRelation,
            recovery::DistributedConflictRule::RejectConflictingOwner,
            true,
            recovery::DistributedAuditRelation::AuditVerificationOnly,
        )
        .expect("policy shape is valid before admission");
        assert_eq!(
            recovery::DistributedStateAdmission::admit_initial_v0_2(policy),
            Err(expected)
        );
    }

    let failover_state = recovery::FailoverVerificationState::try_new(failover_input(Some(
        reference("replacement-node"),
    )))
    .expect("failover verification state is valid");
    for (input, expected) in [
        {
            let mut input = failover_input(Some(reference("replacement-a")));
            input.failed_owner_observed = false;
            (
                input,
                recovery::FailoverVerificationStateError::FailedOwnerObservationMissing,
            )
        },
        {
            let mut input = failover_input(Some(reference("replacement-b")));
            input.affinity_sticky_routing_updated = false;
            (
                input,
                recovery::FailoverVerificationStateError::AffinityRoutingUpdateMissing,
            )
        },
        {
            let mut input = failover_input(Some(reference("replacement-c")));
            input.conflict_and_duplicate_handling_defined = false;
            (
                input,
                recovery::FailoverVerificationStateError::ConflictDuplicateHandlingMissing,
            )
        },
        {
            let mut input = failover_input(Some(reference("replacement-d")));
            input.resource_lifetime_revalidated = false;
            (
                input,
                recovery::FailoverVerificationStateError::ResourceLifetimeRevalidationMissing,
            )
        },
        {
            let mut input = failover_input(Some(reference("replacement-e")));
            input.audit_continuity_recorded = false;
            (
                input,
                recovery::FailoverVerificationStateError::AuditContinuityMissing,
            )
        },
    ] {
        assert_eq!(
            recovery::FailoverVerificationState::try_new(input),
            Err(expected)
        );
    }
    assert_eq!(
        recovery::FailoverAdmission::admit(
            recovery::FailoverClaimClass::ServiceDiscoveryFallback,
            failover_state.clone(),
        ),
        Err(recovery::FailoverAdmissionError::ServiceDiscoveryFallbackIsNotFailoverSuccess)
    );
    assert_eq!(
        recovery::FailoverAdmission::admit(
            recovery::FailoverClaimClass::HealthProbeSuccess,
            failover_state.clone(),
        ),
        Err(recovery::FailoverAdmissionError::HealthProbeIsNotFailoverSuccess)
    );

    let audit_shape = recovery::DistributedStateFailoverAuditShape::new(
        startup("failover-audit-startup"),
        Some(correlation("failover-audit-correlation")),
        recovery::DistributedStateClass::AffinityRequiredState,
        StateFamily::SfuForwardingState,
        recovery::OwnerNodeScope::ServiceInstance,
        owner.clone(),
        Some(reference("affinity-key")),
        recovery::FailoverClaimClass::VerifiedFailoverCandidate,
        Some(reference("replacement-audit")),
        "failover_not_proven",
    );
    assert!(format!("{:?}", audit_shape).contains("failover_not_proven"));

    for failure in [
        recovery::DistributedStateFailureKind::DistributedStateNotAdmitted,
        recovery::DistributedStateFailureKind::StateReplicationNotAdmitted,
        recovery::DistributedStateFailureKind::ConsensusNotAdmitted,
        recovery::DistributedStateFailureKind::FailoverNotProven,
        recovery::DistributedStateFailureKind::NodeAffinityRequired,
        recovery::DistributedStateFailureKind::NodeStateUnavailable,
        recovery::DistributedStateFailureKind::CrossNodeRouteNotAllowed,
        recovery::DistributedStateFailureKind::StateOwnerConflict,
        recovery::DistributedStateFailureKind::SplitBrainRiskDetected,
        recovery::DistributedStateFailureKind::ReplicationLagBoundExceeded,
    ] {
        assert_reason_is_cataloged(failure.reason_code());
    }

    let missing_prior = recovery::ProcessFailureClassification::try_new(
        None,
        None,
        Some(reference("process-missing-prior")),
        recovery::ProcessFailureClass::PanicObserved,
        None,
        Some(recovery::AuditPersistenceStatus::AuditPersistenceCompleted),
        recovery::RestoreReplayPolicyApplication::NotApplied,
        recovery::RestartReadinessClass::NotClaimed,
    );
    assert_eq!(
        missing_prior,
        Err(recovery::ProcessFailureClassificationError::PriorDrainStatusMissing)
    );
    let missing_audit = recovery::ProcessFailureClassification::try_new(
        None,
        None,
        Some(reference("process-missing-audit")),
        recovery::ProcessFailureClass::PanicObserved,
        Some(recovery::PriorDrainStatus::GracefulDrainObserved),
        None,
        recovery::RestoreReplayPolicyApplication::NotApplied,
        recovery::RestartReadinessClass::NotClaimed,
    );
    assert_eq!(
        missing_audit,
        Err(recovery::ProcessFailureClassificationError::AuditPersistenceStatusMissing)
    );

    let audit_unclean = recovery::ProcessFailureClassification::try_new(
        None,
        None,
        Some(reference("process-audit-unclean")),
        recovery::ProcessFailureClass::TaskPanicObserved,
        Some(recovery::PriorDrainStatus::GracefulDrainObserved),
        Some(recovery::AuditPersistenceStatus::AuditPersistenceIncompleteOrAbsent),
        recovery::RestoreReplayPolicyApplication::ReplayVerificationOnly,
        recovery::RestartReadinessClass::ProcessReadinessObservationOnly,
    )
    .expect("classification is valid");
    assert!(audit_unclean.is_unclean_shutdown());

    let clean_task_panic = recovery::ProcessFailureClassification::try_new(
        Some(startup("task-before")),
        Some(startup("task-after")),
        Some(reference("process-task-panic")),
        recovery::ProcessFailureClass::TaskPanicObserved,
        Some(recovery::PriorDrainStatus::GracefulDrainObserved),
        Some(recovery::AuditPersistenceStatus::AuditPersistenceCompleted),
        recovery::RestoreReplayPolicyApplication::RestoreEligibilityApplied,
        recovery::RestartReadinessClass::ReadinessObservedSeparately,
    )
    .expect("task panic classification is valid");
    assert!(!clean_task_panic.is_unclean_shutdown());

    let restore_eligibility = recovery::RestoreEligibility::try_new(
        recovery::StateFamilyRecoveryPolicy::signaling_idempotency_checkpoint_restore_configured(),
        StateClass::CheckpointEligibleState,
        full_restore_preconditions(),
    )
    .expect("restore eligibility is valid");
    assert_eq!(
        recovery::RestartDomainStateClaim::try_new(
            clean_task_panic.clone(),
            Some(restore_eligibility),
            None
        ),
        Err(recovery::RestartDomainStateClaimError::CrashRestartVerificationRequired)
    );
    assert!(recovery::RestartDomainStateClaim::try_new(
        clean_task_panic,
        Some(restore_eligibility),
        Some(crash_verification().expect("crash verification is valid")),
    )
    .is_ok());

    for expected in [
        recovery::CrashRestartVerificationError::SupervisorOrRunnerMissing,
        recovery::CrashRestartVerificationError::StartupRunBeforeMissing,
        recovery::CrashRestartVerificationError::StartupRunAfterMissing,
        recovery::CrashRestartVerificationError::LogsUsedAsAuthoritativeReason,
        recovery::CrashRestartVerificationError::AuditStatusNotDistinct,
        recovery::CrashRestartVerificationError::RestoreStatusNotDistinct,
        recovery::CrashRestartVerificationError::ReadinessStatusNotDistinct,
    ] {
        let result = match expected {
            recovery::CrashRestartVerificationError::SupervisorOrRunnerMissing => {
                recovery::CrashRestartVerification::try_new(
                    "",
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    Some(startup("crash-before-a")),
                    Some(startup("crash-after-a")),
                    true,
                    true,
                    true,
                    true,
                )
            }
            recovery::CrashRestartVerificationError::StartupRunBeforeMissing => {
                recovery::CrashRestartVerification::try_new(
                    "supervisor",
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    None,
                    Some(startup("crash-after-b")),
                    true,
                    true,
                    true,
                    true,
                )
            }
            recovery::CrashRestartVerificationError::StartupRunAfterMissing => {
                recovery::CrashRestartVerification::try_new(
                    "supervisor",
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    Some(startup("crash-before-c")),
                    None,
                    true,
                    true,
                    true,
                    true,
                )
            }
            recovery::CrashRestartVerificationError::LogsUsedAsAuthoritativeReason => {
                recovery::CrashRestartVerification::try_new(
                    "supervisor",
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    Some(startup("crash-before-d")),
                    Some(startup("crash-after-d")),
                    false,
                    true,
                    true,
                    true,
                )
            }
            recovery::CrashRestartVerificationError::AuditStatusNotDistinct => {
                recovery::CrashRestartVerification::try_new(
                    "supervisor",
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    Some(startup("crash-before-e")),
                    Some(startup("crash-after-e")),
                    true,
                    false,
                    true,
                    true,
                )
            }
            recovery::CrashRestartVerificationError::RestoreStatusNotDistinct => {
                recovery::CrashRestartVerification::try_new(
                    "supervisor",
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    Some(startup("crash-before-f")),
                    Some(startup("crash-after-f")),
                    true,
                    true,
                    false,
                    true,
                )
            }
            recovery::CrashRestartVerificationError::ReadinessStatusNotDistinct => {
                recovery::CrashRestartVerification::try_new(
                    "supervisor",
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    recovery::ProcessFailureClass::ProcessCrashObserved,
                    Some(startup("crash-before-g")),
                    Some(startup("crash-after-g")),
                    true,
                    true,
                    true,
                    false,
                )
            }
        };
        assert_eq!(result, Err(expected));
    }

    for failure in [
        recovery::ProcessFailureMappingKind::ProcessPanicDetected,
        recovery::ProcessFailureMappingKind::RuntimeTaskPanicDetected,
        recovery::ProcessFailureMappingKind::ProcessCrashDetected,
        recovery::ProcessFailureMappingKind::UncleanShutdownDetected,
        recovery::ProcessFailureMappingKind::SupervisorRestartObserved,
        recovery::ProcessFailureMappingKind::DriverShutdown,
    ] {
        assert_reason_is_cataloged(failure.reason_code());
    }
    assert!(!recovery::ProcessFailureClass::TaskPanicObserved
        .requires_restore_policy_before_state_claim());
    assert!(!recovery::ProcessFailureClass::TaskPanicObserved.is_process_crash());
    assert_eq!(
        [
            recovery::ProhibitedDistributedStateBehavior::NodeLocalStateAsClusterGlobalDefault,
            recovery::ProhibitedDistributedStateBehavior::ServiceDiscoveryFallbackAsFailoverSuccess,
            recovery::ProhibitedDistributedStateBehavior::HealthSuccessAsStateHandoffSuccess,
            recovery::ProhibitedDistributedStateBehavior::AuditReplayMutatesWithoutRestorePolicy,
            recovery::ProhibitedDistributedStateBehavior::TwoOwnersAcceptedWithoutConflictRule,
            recovery::ProhibitedDistributedStateBehavior::UnboundedReplicationLagOrHandoffWindow,
            recovery::ProhibitedDistributedStateBehavior::TestFakeAsProductionDistributedState,
        ]
        .len(),
        7
    );
}

#[test]
fn runtime_task_lifecycle_time_policy_and_cancellation_paths_are_asserted() {
    assert_eq!(
        format!("{:?}", runtime::CoreRuntimeSurface),
        "CoreRuntimeSurface"
    );

    let runtime_selection = runtime::RuntimeConfigurationSelection::new(
        startup("runtime-startup"),
        ConfigurationScopeRef::new(reference("runtime-config")),
        runtime::RuntimeAbstractionSurface::RuntimePort,
    );
    assert!(format!("{:?}", runtime_selection).contains("RuntimePort"));

    for target in [
        runtime::TimePolicyTarget::TurnAllocationExpiry,
        runtime::TimePolicyTarget::TurnPermissionExpiry,
        runtime::TimePolicyTarget::TurnChannelBindExpiry,
        runtime::TimePolicyTarget::SignalingRoomLifecycle,
        runtime::TimePolicyTarget::ResourceRetention,
        runtime::TimePolicyTarget::ConfigurationStartupTimeout,
        runtime::TimePolicyTarget::CommandDeadline,
    ] {
        assert_reason_is_cataloged(target.failure_reason());
    }

    for use_class in [
        runtime::RandomnessUseClass::Nonce,
        runtime::RandomnessUseClass::OpaqueId,
        runtime::RandomnessUseClass::Challenge,
        runtime::RandomnessUseClass::ReferenceStability,
    ] {
        assert!(
            format!("{:?}", runtime::RandomnessContract::opaque(use_class))
                .contains("core_owns_identity_meaning")
        );
    }

    assert!(runtime::RuntimeTaskClass::DetachedTaskRequested.is_rejected_class());
    for admitted in [
        runtime::RuntimeTaskClass::NoRuntimeTask,
        runtime::RuntimeTaskClass::DriverIoWorker,
        runtime::RuntimeTaskClass::DriverPacketWorker,
        runtime::RuntimeTaskClass::DriverSinkWorker,
        runtime::RuntimeTaskClass::EntrypointsSupervisionTask,
        runtime::RuntimeTaskClass::RuntimeTimerTask,
        runtime::RuntimeTaskClass::TestRuntimeTask,
    ] {
        assert!(!admitted.is_rejected_class());
    }

    assert_eq!(
        runtime::RuntimeTaskLifecyclePolicy::try_new(
            runtime::RuntimeTaskClass::DriverIoWorker,
            None,
            runtime::RuntimeTaskOwningLayer::Driver,
            runtime::RuntimeTaskInputReferenceClass::OpaqueTaskReference,
            runtime::RuntimeTaskOutputObservation::SpawnObserved,
            runtime::CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
            true,
            true,
        ),
        Err(runtime::RuntimeTaskLifecyclePolicyError::SupervisionScopeMissing)
    );
    assert_eq!(
        runtime::RuntimeTaskLifecyclePolicy::try_new(
            runtime::RuntimeTaskClass::DriverIoWorker,
            Some(runtime::SupervisionScope::DriverComponent),
            runtime::RuntimeTaskOwningLayer::Driver,
            runtime::RuntimeTaskInputReferenceClass::OpaqueTaskReference,
            runtime::RuntimeTaskOutputObservation::SpawnObserved,
            runtime::CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
            false,
            true,
        ),
        Err(runtime::RuntimeTaskLifecyclePolicyError::JoinWaitBoundMissing)
    );

    let no_task_policy = runtime::RuntimeTaskLifecyclePolicy::try_new(
        runtime::RuntimeTaskClass::NoRuntimeTask,
        None,
        runtime::RuntimeTaskOwningLayer::CoreObservedRuntimePort,
        runtime::RuntimeTaskInputReferenceClass::None,
        runtime::RuntimeTaskOutputObservation::NoTaskObservation,
        runtime::CancellationPropagationRule::NotApplicable,
        false,
        false,
    )
    .expect("no-runtime-task policy does not require task bounds");
    let worker_policy = runtime::RuntimeTaskLifecyclePolicy::try_new(
        runtime::RuntimeTaskClass::DriverPacketWorker,
        Some(runtime::SupervisionScope::RuntimePortScheduleCancelScope),
        runtime::RuntimeTaskOwningLayer::Driver,
        runtime::RuntimeTaskInputReferenceClass::OpaqueScheduleReference,
        runtime::RuntimeTaskOutputObservation::JoinObserved,
        runtime::CancellationPropagationRule::FollowsShutdownDrain,
        true,
        true,
    )
    .expect("bounded worker policy is valid");

    assert_eq!(
        runtime::RuntimeTaskLifecycleDecision::try_new(
            startup("runtime-decision-success-with-reason"),
            Some(correlation("runtime-decision-correlation")),
            no_task_policy,
            None,
            runtime::RuntimeTaskLifecycleOutcome::Accepted,
            Some(runtime::RuntimeFailureKind::RuntimeTaskSpawnFailed),
        ),
        Err(runtime::RuntimeTaskLifecycleDecisionError::ReasonMustBeAbsentForSuccess)
    );
    assert_eq!(
        runtime::RuntimeTaskLifecycleDecision::try_new(
            startup("runtime-decision-rejected-no-reason"),
            Some(correlation("runtime-decision-rejected")),
            no_task_policy,
            None,
            runtime::RuntimeTaskLifecycleOutcome::Rejected,
            None,
        ),
        Err(runtime::RuntimeTaskLifecycleDecisionError::ReasonRequired)
    );
    let decision = runtime::RuntimeTaskLifecycleDecision::try_new(
        startup("runtime-decision-failed"),
        Some(correlation("runtime-decision-failed-correlation")),
        worker_policy,
        Some("opaque-task-ref"),
        runtime::RuntimeTaskLifecycleOutcome::Failed,
        Some(runtime::RuntimeFailureKind::RuntimeTaskJoinFailed),
    )
    .expect("failure decision carries cataloged reason");
    assert_eq!(
        decision.audit_event_type(),
        "runtime_task_lifecycle_decision"
    );

    for outcome in [
        runtime::RuntimeTaskLifecycleOutcome::Spawned,
        runtime::RuntimeTaskLifecycleOutcome::Joined,
        runtime::RuntimeTaskLifecycleOutcome::Cancelled,
    ] {
        assert!(runtime::RuntimeTaskLifecycleDecision::try_new(
            startup("runtime-success-loop"),
            None,
            worker_policy,
            Some("opaque-task-ref"),
            outcome,
            None,
        )
        .is_ok());
    }
    assert!(runtime::RuntimeTaskLifecycleDecision::try_new(
        startup("runtime-failure-loop"),
        None,
        worker_policy,
        Some("opaque-task-ref"),
        runtime::RuntimeTaskLifecycleOutcome::PanicObserved,
        Some(runtime::RuntimeFailureKind::RuntimeTaskPanicDetected),
    )
    .is_ok());

    for failure in [
        runtime::RuntimeFailureKind::RuntimeConfigMissing,
        runtime::RuntimeFailureKind::RuntimeConfigInvalid,
        runtime::RuntimeFailureKind::RuntimeTaskClassNotAdmitted,
        runtime::RuntimeFailureKind::RuntimeTaskOwnerViolation,
        runtime::RuntimeFailureKind::RuntimeTaskSupervisionMissing,
        runtime::RuntimeFailureKind::RuntimeTaskSpawnFailed,
        runtime::RuntimeFailureKind::RuntimeTaskJoinFailed,
        runtime::RuntimeFailureKind::RuntimeTaskCancelFailed,
        runtime::RuntimeFailureKind::RuntimeTaskPanicDetected,
        runtime::RuntimeFailureKind::DriverShutdown,
        runtime::RuntimeFailureKind::RuntimeTaskQueueBoundExceeded,
        runtime::RuntimeFailureKind::MemoryPressureExceeded,
    ] {
        assert_reason_is_cataloged(failure.reason_code());
    }

    for surface in [
        runtime::TaskCancellationSurface::BeforeDriverCoreConversion,
        runtime::TaskCancellationSurface::AfterCommandEnteredCore,
        runtime::TaskCancellationSurface::DuringShutdownDrain,
        runtime::TaskCancellationSurface::DuringDriverQueueCacheExecution,
        runtime::TaskCancellationSurface::DuringTestHarnessTimeout,
    ] {
        assert!(!surface.required_relation().is_empty());
    }

    assert_eq!(
        [
            runtime::ProhibitedRuntimeClockRandomnessBehavior::CoreImportsConcreteRuntimeHandle,
            runtime::ProhibitedRuntimeClockRandomnessBehavior::DriverTimerDefinesDomainExpiry,
            runtime::ProhibitedRuntimeClockRandomnessBehavior::RawPlatformTimeWithoutNormalization,
            runtime::ProhibitedRuntimeClockRandomnessBehavior::RandomGeneratorOwnsIdentitySemantics,
            runtime::ProhibitedRuntimeClockRandomnessBehavior::EntrypointsSilentlySubstituteRuntimeDefaults,
            runtime::ProhibitedRuntimeClockRandomnessBehavior::DeterministicTestClockRngInProductionRuntime,
            runtime::ProhibitedRuntimeClockRandomnessBehavior::RuntimeWorkerOwnsDomainState,
            runtime::ProhibitedRuntimeClockRandomnessBehavior::ImplicitDetachedTaskOrSupervision,
            runtime::ProhibitedRuntimeClockRandomnessBehavior::WallClockAsCrossNodeCausalOrderWithoutTrust,
        ]
        .len(),
        9
    );
}

#[test]
fn sfu_packet_routing_negotiation_congestion_and_release_paths_are_asserted() {
    assert_eq!(format!("{:?}", sfu::CoreSfuSurface), "CoreSfuSurface");

    let references = sfu::SfuReferenceSet::new(
        session("sfu-session"),
        Some(endpoint("sfu-endpoint")),
        Some(stream("sfu-stream")),
        Some(route("sfu-route")),
        Some(packet("sfu-packet")),
    );
    assert_eq!(references.session_id().as_str(), "sfu-session");

    let contract_item = sfu::SfuContractItem::new(
        sfu::SfuModelKind::ForwardingIntent,
        references,
        "forward-payload",
    );
    assert_eq!(
        contract_item.model_kind(),
        sfu::SfuModelKind::ForwardingIntent
    );

    let sfu_decision = sfu::SfuDecision::new(
        sfu::SfuDecisionKind::Forwarding,
        decision(
            "sfu-forwarding",
            TargetSurface::Sfu,
            UseCaseOutcome::Forwarded,
            DecisionReason::Absent,
        ),
    )
    .expect("SFU decision accepts SFU target surface");
    assert_eq!(sfu_decision.kind(), sfu::SfuDecisionKind::Forwarding);

    let packet_id = packet("borrowed-packet");
    let stream_id = stream("borrowed-stream");
    let source_endpoint = endpoint("borrowed-endpoint");
    let raw_packet = [0_u8, 1, 2, 3, 4];
    let payload = [2_u8, 3, 4];
    let packet_view = sfu::SfuPacketView::new(
        &packet_id,
        &stream_id,
        &source_endpoint,
        sfu::PacketHeaderSemanticView::new(sfu::PacketClass::Rtp, Some(7), Some(11), Some(13)),
        &raw_packet,
        &payload,
    );
    assert_eq!(packet_view.packet_id().as_str(), "borrowed-packet");
    assert_eq!(packet_view.payload_len(), 3);

    let metadata = sfu::PacketSemanticMetadata::new(
        Some(sfu::MediaKind::Video),
        sfu::PacketClass::Rtcp,
        Some(8),
        Some(12),
        Some(sfu::SsrcRef::new(42)),
        Some(sfu::PayloadTypeRef::new(111)),
        Some(true),
        raw_packet.len(),
    );
    assert_eq!(metadata.packet_length(), raw_packet.len());

    let rewrite_intent = sfu::PacketRewriteTransformIntent::new(
        route("rewrite-route"),
        Some(endpoint("rewrite-target")),
        sfu::PacketRewriteTransformClass::SsrcRewrite,
        packet("rewrite-source"),
        Some(packet("rewrite-target-packet")),
        Some("mapping-table"),
        sfu::RewriteCopyAllowanceClass::TargetSpecificHeaderBufferAllowed,
    );
    assert_eq!(
        rewrite_intent.class(),
        sfu::PacketRewriteTransformClass::SsrcRewrite
    );

    let media_mapping = sfu::MediaNegotiationMapping::new(
        "sfu-v0.2",
        sfu::MediaNegotiationClass::TrackSubscriptionRequest,
        Some(sfu::CodecProfileRef::new("opus")),
        Some(sfu::TrackRef::new("audio-main")),
        Some(sfu::MediaLayerRef::new("base")),
        Some(sfu::PayloadTypeRef::new(111)),
        Some(sfu::SsrcRef::new(1001)),
    );
    assert!(format!("{:?}", media_mapping).contains("sfu-v0.2"));

    let congestion =
        sfu::CongestionInput::new(sfu::CongestionObservationClass::PacketLossSignal, 9);
    assert!(format!("{:?}", congestion).contains("PacketLossSignal"));

    for release_reason in [
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
        assert_reason_is_cataloged(
            release_reason
                .reason_code()
                .expect("non-success release reason has reason code"),
        );
    }
    assert_eq!(sfu::PacketReleaseReason::Forwarded.reason_code(), None);

    for failure in [
        sfu::PacketSemanticViewFailureKind::ExternalDecodeFailed,
        sfu::PacketSemanticViewFailureKind::FrameSizeBoundExceeded,
        sfu::PacketSemanticViewFailureKind::UnsupportedMediaContractVersion,
        sfu::PacketSemanticViewFailureKind::BufferReleaseFailed,
    ] {
        assert_reason_is_cataloged(failure.reason_code());
    }
    for failure in [
        sfu::PacketRewriteTransformFailureKind::PacketRewriteClassNotAdmitted,
        sfu::PacketRewriteTransformFailureKind::PacketRewriteIntentInvalid,
        sfu::PacketRewriteTransformFailureKind::PacketRewriteOwnerViolation,
        sfu::PacketRewriteTransformFailureKind::PayloadTransformNotAdmitted,
        sfu::PacketRewriteTransformFailureKind::MediaTranscodeNotSupported,
        sfu::PacketRewriteTransformFailureKind::PayloadTransformFailed,
        sfu::PacketRewriteTransformFailureKind::RewriteCopyBoundExceeded,
    ] {
        assert_reason_is_cataloged(failure.reason_code());
    }
    for failure in [
        sfu::MediaNegotiationFailureKind::MediaCodecNotSupported,
        sfu::MediaNegotiationFailureKind::MediaTrackNotAllowed,
        sfu::MediaNegotiationFailureKind::MediaLayerNotAvailable,
        sfu::MediaNegotiationFailureKind::MediaPayloadMappingInvalid,
        sfu::MediaNegotiationFailureKind::MediaFeedbackNotSupported,
        sfu::MediaNegotiationFailureKind::MediaTranscodeNotSupported,
        sfu::MediaNegotiationFailureKind::UnsupportedMediaContractVersion,
        sfu::MediaNegotiationFailureKind::ExternalDecodeFailed,
    ] {
        assert_reason_is_cataloged(failure.reason_code());
    }
    for failure in [
        sfu::CongestionPacingRetransmissionFailureKind::PacketCacheBoundExceeded,
        sfu::CongestionPacingRetransmissionFailureKind::RetentionDurationExceeded,
        sfu::CongestionPacingRetransmissionFailureKind::SfuTransmitQueueBoundExceeded,
        sfu::CongestionPacingRetransmissionFailureKind::PacketSuppressedByBackpressure,
        sfu::CongestionPacingRetransmissionFailureKind::PacketDroppedByBackpressure,
        sfu::CongestionPacingRetransmissionFailureKind::RouteDegradedByBackpressure,
        sfu::CongestionPacingRetransmissionFailureKind::EndpointClosedByBackpressure,
        sfu::CongestionPacingRetransmissionFailureKind::BackpressureRecoveryNotAllowed,
        sfu::CongestionPacingRetransmissionFailureKind::TargetUnavailable,
        sfu::CongestionPacingRetransmissionFailureKind::NetworkSendFailed,
        sfu::CongestionPacingRetransmissionFailureKind::DriverShutdown,
    ] {
        assert_reason_is_cataloged(failure.reason_code());
    }

    for rule in sfu::SFU_TRANSITION_RULES {
        assert!(!rule.allowed_pre_state().is_empty());
        assert!(format!("{:?}", rule.trigger()).contains(|c: char| c.is_ascii_alphabetic()));
        for reason in rule.reject_reasons() {
            assert_reason_is_cataloged(reason.reason_code());
        }
    }

    assert_eq!(
        [
            sfu::PacketCopyPolicy::NoCopySharedLease,
            sfu::PacketCopyPolicy::HeaderOnlyAllocationPreferred,
            sfu::PacketCopyPolicy::TargetSpecificHeaderBufferAllowed,
            sfu::PacketCopyPolicy::PayloadCopyOnWriteAllowed,
            sfu::PacketCopyPolicy::DriverLocalBackendCopyAllowed,
        ]
        .len(),
        5
    );
    assert_eq!(
        [
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
        ]
        .len(),
        13
    );
}

#[test]
fn turn_command_decision_lifecycle_reason_and_guard_paths_are_asserted() {
    assert_eq!(format!("{:?}", turn::CoreTurnSurface), "CoreTurnSurface");

    let peer = turn::CorePeerAddress::new("203.0.113.10:3478").expect("peer address is valid");
    assert_eq!(peer.as_str(), "203.0.113.10:3478");
    assert_eq!(
        turn::CorePeerAddress::new("bad\npeer"),
        Err(turn::TurnContractError::InvalidPeerAddress)
    );

    let transaction = turn::TurnTransactionId::new(reference("turn-txn-success"));
    assert_eq!(transaction.as_str(), "turn-txn-success");
    let lifetime = turn::TurnRequestedLifetimeSeconds::try_new(600).expect("lifetime is non-zero");
    assert_eq!(lifetime.as_u32(), 600);

    let references = turn::TurnReferenceSet::new(
        Some(allocation("allocation-success")),
        Some(permission("permission-success")),
        Some(channel_bind("channel-success")),
        Some(credential("credential-success")),
    );
    let command = turn::TurnCommand::try_new(
        turn::TurnCommandKind::ChannelBind,
        transaction,
        references.clone(),
        Some(peer.clone()),
        Some(lifetime),
        None,
    )
    .expect("channel bind command has peer and channel reference");
    assert_eq!(command.kind(), turn::TurnCommandKind::ChannelBind);
    assert_eq!(command.transaction_id().as_str(), "turn-txn-success");
    assert_eq!(command.references(), &references);

    assert!(turn::TurnCommand::try_new(
        turn::TurnCommandKind::RelayData,
        turn::TurnTransactionId::new(reference("turn-relay")),
        references.clone(),
        Some(peer.clone()),
        None,
        Some(packet("relay-packet")),
    )
    .is_ok());

    let turn_decision = turn::TurnDecision::new(
        turn::TurnDecisionKind::Relay,
        decision(
            "turn-relay-decision",
            TargetSurface::Turn,
            UseCaseOutcome::Allowed,
            DecisionReason::Absent,
        ),
    )
    .expect("TURN decision accepts TURN target surface");
    assert!(format!("{:?}", turn_decision).contains("Relay"));
    assert_eq!(
        turn::TurnDecision::new(
            turn::TurnDecisionKind::Allocation,
            decision(
                "turn-wrong-surface",
                TargetSurface::Sfu,
                UseCaseOutcome::Accepted,
                DecisionReason::Absent,
            ),
        ),
        Err(turn::TurnContractError::WrongTargetSurface)
    );

    for (command_kind, decision_kind) in [
        (
            turn::TurnCommandKind::Allocate,
            turn::TurnDecisionKind::Allocation,
        ),
        (
            turn::TurnCommandKind::Refresh,
            turn::TurnDecisionKind::Refresh,
        ),
        (
            turn::TurnCommandKind::CreatePermission,
            turn::TurnDecisionKind::Permission,
        ),
        (
            turn::TurnCommandKind::ChannelBind,
            turn::TurnDecisionKind::ChannelBind,
        ),
        (
            turn::TurnCommandKind::RelayData,
            turn::TurnDecisionKind::Relay,
        ),
    ] {
        assert_eq!(command_kind.decision_kind(), decision_kind);
    }

    for failure in [
        turn::TurnFailureKind::MalformedTurnMessage,
        turn::TurnFailureKind::CredentialMissing,
        turn::TurnFailureKind::CredentialInvalid,
        turn::TurnFailureKind::CredentialExpired,
        turn::TurnFailureKind::SecretGenerationNotAccepted,
        turn::TurnFailureKind::SecretKeyRevoked,
        turn::TurnFailureKind::SecretOverlapWindowExpired,
        turn::TurnFailureKind::SecretRotationStateUnavailable,
        turn::TurnFailureKind::AllocationCapacityExceeded,
        turn::TurnFailureKind::PermissionCapacityExceeded,
        turn::TurnFailureKind::PeerNotAllowed,
        turn::TurnFailureKind::PermissionNotFound,
        turn::TurnFailureKind::RelayDenied,
        turn::TurnFailureKind::AllocationNotFound,
        turn::TurnFailureKind::UnsupportedTurnMethod,
        turn::TurnFailureKind::UnsupportedTurnContractVersion,
        turn::TurnFailureKind::TurnLifetimeViolation,
        turn::TurnFailureKind::AllocationLifetimeExceeded,
        turn::TurnFailureKind::PermissionLifetimeExceeded,
        turn::TurnFailureKind::ChannelBindLifetimeExceeded,
        turn::TurnFailureKind::RefreshLimitExceeded,
        turn::TurnFailureKind::TurnRelayQueueBoundExceeded,
    ] {
        assert_reason_is_cataloged(failure.reason_code());
    }

    for rule in turn::TURN_LIFECYCLE_RULES {
        assert!(!rule.event().is_empty());
        assert!(!rule.allowed_pre_state().is_empty());
        assert!(!rule.success_state().is_empty());
        assert!(!rule.failure_state().is_empty());
        for reason in rule.reason_codes() {
            assert_reason_is_cataloged(reason.reason_code());
        }
    }

    let model_kinds = [
        turn::TurnModelKind::TransactionId,
        turn::TurnModelKind::Message(turn::TurnMessageClass::Request),
        turn::TurnModelKind::Message(turn::TurnMessageClass::Response),
        turn::TurnModelKind::Message(turn::TurnMessageClass::Indication),
        turn::TurnModelKind::Message(turn::TurnMessageClass::Error),
        turn::TurnModelKind::Allocation,
        turn::TurnModelKind::Permission,
        turn::TurnModelKind::ChannelBinding,
        turn::TurnModelKind::ChannelBindingReference,
        turn::TurnModelKind::PeerAddress,
        turn::TurnModelKind::RelayDecision,
        turn::TurnModelKind::CredentialVerificationOutcome,
        turn::TurnModelKind::LifetimeExpiry,
        turn::TurnModelKind::ClosedErrorReason,
    ];
    assert_eq!(model_kinds.len(), 14);
    assert_eq!(
        [
            turn::AllocationState::Absent,
            turn::AllocationState::Requested,
            turn::AllocationState::Active,
            turn::AllocationState::Refreshing,
            turn::AllocationState::Expired,
            turn::AllocationState::Released,
            turn::AllocationState::Rejected,
        ]
        .len(),
        7
    );
    assert_eq!(
        [
            turn::PermissionState::Absent,
            turn::PermissionState::Requested,
            turn::PermissionState::Active,
            turn::PermissionState::Expired,
            turn::PermissionState::Revoked,
            turn::PermissionState::Rejected,
        ]
        .len(),
        6
    );
    assert_eq!(
        [
            turn::ChannelBindState::Unbound,
            turn::ChannelBindState::Requested,
            turn::ChannelBindState::Bound,
            turn::ChannelBindState::Expired,
            turn::ChannelBindState::Rejected,
        ]
        .len(),
        5
    );
}
