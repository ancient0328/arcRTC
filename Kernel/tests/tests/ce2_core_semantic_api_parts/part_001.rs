use arcrtc_core_audit::{
    find_audit_event_definition, AuditComponent, AuditEvent, AuditEventInput, AuditReason,
    AuditReferenceKind, AuditReferencePresence, AuditReferenceValue, AuditReferences,
    AuditTimestamp, CanonicalEventPayloadDigest, HashAlgorithm, HashChainRecordError,
};
use arcrtc_core_command::{
    AuditProjectionRequirement, CommandEnvelope, CommandIdentity, CommandType, CommandVersion,
    DecisionEvidenceClass, DecisionReason, DecisionShapeError, IdempotencyClass,
    IdempotencyDecision, IdempotencyFailureKind, IdempotencyKey, IdempotencyScope, PortIntent,
    ReplayWindow, SemanticPayloadDigest, StateTransitionSummary, TargetSurface, UseCaseDecision,
    UseCaseDecisionInput, UseCaseOutcome,
};
use arcrtc_core_cross_plane::{
    BindingExpiryBehavior, BindingLifecyclePrecondition, BindingReplayRelation, CrossPlane,
    CrossPlaneBindingClass, CrossPlaneBindingDecision, CrossPlaneBindingFailureKind,
    CrossPlaneBindingOutcome, CrossPlaneBindingPolicy, CrossPlaneReference,
};
use arcrtc_core_identity::{
    AuditEventId, CorrelationId, CredentialRef, EndpointId, OpaqueReference, PacketId,
    ParticipantId, ReferenceAuthority, RoomId, RouteId, SessionId, StartupRunId, StreamId,
};
use arcrtc_core_protocol::{
    CanonicalDigest, CanonicalEncodingError, CanonicalEncodingFailureKind,
    CanonicalEncodingRuleSet, CanonicalFormatVersion, CanonicalRuleStatus, CompatibilityChange,
    CompatibilityClassification, CompatibilityFailureKind, CompatibilityVersionRange,
    ContractVersion, DeprecationDecision, DeprecationLifecycleStep, UnknownFieldHandling,
    VersionOwner, VersionedSurface, VersionedSurfaceOwnership,
};
use arcrtc_core_quality::{
    ResourceBoundDecision, ResourceBoundDecisionShapeError, ResourceBoundKind,
    ResourceBoundReferenceSet, REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS,
};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_recovery::{
    CommandRoutingRule, DistributedAuditRelation, DistributedConflictRule, DistributedStateClass,
    DistributedStatePolicy, DistributedStatePolicyError, OwnerNodeScope, PacketRoutingRule,
    RecoveryRestoreRelation, RestorePreconditionError, RestorePreconditionSet,
};
use arcrtc_core_runtime::{
    CancellationPropagationRule, RuntimeTaskClass, RuntimeTaskInputReferenceClass,
    RuntimeTaskLifecycleDecision, RuntimeTaskLifecycleDecisionError, RuntimeTaskLifecycleOutcome,
    RuntimeTaskLifecyclePolicy, RuntimeTaskLifecyclePolicyError, RuntimeTaskOutputObservation,
    RuntimeTaskOwningLayer, SupervisionScope, TimePolicyTarget,
};
use arcrtc_core_security::{
    AudiencePolicy, IssuerPolicy, RequiredTokenClaim, TokenAlgorithmPolicy, TokenTemporalDecision,
    TokenVerificationFailureKind, TokenVerificationRequest, VerifiedCredential,
};
use arcrtc_core_sfu::{
    CodecProfileRef, CongestionInput, CongestionObservationClass,
    CongestionPacingRetransmissionFailureKind, FeedbackReferenceClass, MediaKind, MediaLayerRef,
    MediaNegotiationClass, MediaNegotiationFailureKind, MediaNegotiationMapping,
    PacingExecutionBoundary, PacketClass, PacketHeaderSemanticView, PacketReleaseReason,
    PacketRewriteTransformClass, PacketRewriteTransformFailureKind, PacketRewriteTransformIntent,
    PacketSemanticMetadata, PayloadTypeRef, RetransmissionBoundary, RewriteCopyAllowanceClass,
    SfuContractError, SfuDecision, SfuDecisionKind, SfuFailureKind, SfuPacketView, SsrcRef,
    TrackRef,
};
use arcrtc_core_signaling::{
    SignalingContractError, SignalingDecision, SignalingFailureKind, SignalingSubject,
};
use arcrtc_core_state::{
    CheckpointIntent, CheckpointIntentError, CheckpointOwnerBoundary, StateClass, StateFamily,
};
use arcrtc_core_time::{
    ClockSkewImpact, ClockSkewPolicy, ClockSkewPolicyError, ObservedSkewClass, PrecisionClass,
    SamplingWindow, TimeNodeScope, TimeSynchronizationDecision, TimeSynchronizationDecisionError,
    TimeSynchronizationFailureKind, TimeSynchronizationOutcome, TimeTrustClass,
    TrustedTimeSourceClass,
};
use arcrtc_core_transport::{IceCandidateRef, SessionDescriptionRef, TransportContractError};
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommand, TurnCommandKind, TurnContractError, TurnDecision,
    TurnDecisionKind, TurnReferenceSet, TurnRequestedLifetimeSeconds, TurnTransactionId,
};

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("reference must be valid")
}

fn correlation() -> CorrelationId {
    CorrelationId::new(reference("corr-ce2"))
}

fn cataloged(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason code must be cataloged")
}

fn decision(
    target_surface: TargetSurface,
    outcome: UseCaseOutcome,
    reason: DecisionReason<CatalogedReasonRef>,
) -> Result<UseCaseDecision<CatalogedReasonRef>, DecisionShapeError> {
    UseCaseDecision::new(UseCaseDecisionInput {
        correlation_id: correlation(),
        command_type: CommandType::new("ce2-command"),
        target_surface,
        outcome,
        reason,
        state_transition: StateTransitionSummary::Changed("ce2-transition"),
        port_intents: vec![PortIntent::new("ce2-port-intent", target_surface)],
        audit_projection: AuditProjectionRequirement::Required,
        evidence_class: DecisionEvidenceClass::SourceDecisionOnly,
    })
}

#[test]
fn ce2_command_decision_event_result_shape_preserves_correlation_and_reason() {
    let envelope = CommandEnvelope::new(
        correlation(),
        CommandType::new("join_room"),
        CommandVersion::new(1),
        TargetSurface::Signaling,
        SignalingSubject::new(RoomId::new(reference("room-ce2")), None),
    );
    assert_eq!(envelope.correlation_id().as_str(), "corr-ce2");
    assert_eq!(envelope.version().value(), 1);
    assert_eq!(envelope.target_surface(), TargetSurface::Signaling);

    let rejected = decision(
        TargetSurface::Signaling,
        UseCaseOutcome::Rejected,
        DecisionReason::Cataloged(cataloged("missing_correlation_id")),
    )
    .expect("non-success with cataloged reason must be valid");
    assert_eq!(rejected.correlation_id().as_str(), "corr-ce2");
    assert!(matches!(rejected.reason(), DecisionReason::Cataloged(_)));
}

#[test]
fn ce2_reason_catalog_is_closed_and_decision_reason_rule_is_fail_closed() {
    assert_eq!(
        CatalogedReasonRef::from_code("not-a-cataloged-reason"),
        Err(arcrtc_core_reason::CatalogedReasonLookupError::NotCataloged)
    );

    let invalid_success = decision(
        TargetSurface::Signaling,
        UseCaseOutcome::Accepted,
        DecisionReason::Cataloged(cataloged("missing_correlation_id")),
    );
    assert_eq!(
        invalid_success,
        Err(DecisionShapeError::SuccessMustNotCarryReason)
    );

    let invalid_reject = decision(
        TargetSurface::Signaling,
        UseCaseOutcome::Rejected,
        DecisionReason::Absent,
    );
    assert_eq!(
        invalid_reject,
        Err(DecisionShapeError::NonSuccessRequiresReason)
    );
}

#[test]
fn ce2_signaling_sfu_turn_decisions_reject_wrong_target_surface() {
    let wrong_for_signaling = decision(
        TargetSurface::Sfu,
        UseCaseOutcome::Rejected,
        DecisionReason::Cataloged(cataloged(
            SignalingFailureKind::MissingCorrelationId.reason_code(),
        )),
    )
    .expect("shape itself is valid");
    assert_eq!(
        SignalingDecision::new(wrong_for_signaling),
        Err(SignalingContractError::WrongTargetSurface)
    );

    let wrong_for_sfu = decision(
        TargetSurface::Turn,
        UseCaseOutcome::Rejected,
        DecisionReason::Cataloged(cataloged(
            SfuFailureKind::ParticipantNotAdmitted.reason_code(),
        )),
    )
    .expect("shape itself is valid");
    assert_eq!(
        SfuDecision::new(SfuDecisionKind::RouteSelection, wrong_for_sfu),
        Err(SfuContractError::WrongTargetSurface)
    );

    let wrong_for_turn = decision(
        TargetSurface::Transport,
        UseCaseOutcome::Rejected,
        DecisionReason::Cataloged(cataloged("allocation_not_found")),
    )
    .expect("shape itself is valid");
    assert_eq!(
        TurnDecision::new(TurnDecisionKind::Allocation, wrong_for_turn),
        Err(TurnContractError::WrongTargetSurface)
    );
}

#[test]
fn ce2_turn_transport_and_cross_plane_fail_closed_inputs_are_public_api_checked() {
    assert_eq!(
        TurnRequestedLifetimeSeconds::try_new(0),
        Err(TurnContractError::InvalidRequestedLifetime)
    );
    assert_eq!(
        CorePeerAddress::new(""),
        Err(TurnContractError::InvalidPeerAddress)
    );

    let transaction = TurnTransactionId::new(reference("turn-tx-ce2"));
    let missing_peer = TurnCommand::try_new(
        TurnCommandKind::CreatePermission,
        transaction,
        TurnReferenceSet::new(None, None, None, None),
        None,
        Some(TurnRequestedLifetimeSeconds::try_new(60).expect("valid lifetime")),
        None,
    );
    assert_eq!(missing_peer, Err(TurnContractError::MissingPeerAddress));

    let session_id = SessionId::new(reference("session-ce2"));
    assert_eq!(
        SessionDescriptionRef::new(session_id.clone(), ""),
        Err(TransportContractError::InvalidSemanticReference)
    );
    assert_eq!(
        IceCandidateRef::new(session_id, "bad\ncandidate"),
        Err(TransportContractError::InvalidSemanticReference)
    );

    assert!(CrossPlaneBindingClass::ImplicitBindingRequested.is_rejected_class());
    let policy = CrossPlaneBindingPolicy::new(
        CrossPlaneBindingClass::ImplicitBindingRequested,
        CrossPlane::Signaling,
        CrossPlane::Sfu,
        CrossPlaneReference::Participant(ParticipantId::new(reference("participant-ce2"))),
        CrossPlaneReference::NotMaterialized,
        None,
        BindingLifecyclePrecondition::ParticipantJoined,
        BindingLifecyclePrecondition::EndpointAdmitted,
        BindingExpiryBehavior::RejectNewTargetPlaneAction,
        BindingReplayRelation::ConflictRejected,
    );
    let cross_plane_decision = CrossPlaneBindingDecision::new(
        correlation(),
        policy,
        CrossPlaneBindingOutcome::Rejected,
        Some(CrossPlaneBindingFailureKind::RequiredBindingAbsent),
    );
    assert_eq!(
        cross_plane_decision.audit_event_type(),
        "cross_plane_binding_decision"
    );
}

#[test]
fn ce2_runtime_state_recovery_quality_and_audit_fail_closed_inputs_are_checked() {
    assert_eq!(
        RuntimeTaskLifecyclePolicy::try_new(
            RuntimeTaskClass::DetachedTaskRequested,
            None,
            RuntimeTaskOwningLayer::Driver,
            RuntimeTaskInputReferenceClass::OpaqueTaskReference,
            RuntimeTaskOutputObservation::SpawnObserved,
            CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
            true,
            true,
        ),
        Err(RuntimeTaskLifecyclePolicyError::DetachedTaskNotAdmitted)
    );

    let worker_without_mailbox_bound = RuntimeTaskLifecyclePolicy::try_new(
        RuntimeTaskClass::DriverIoWorker,
        Some(SupervisionScope::DriverComponent),
        RuntimeTaskOwningLayer::Driver,
        RuntimeTaskInputReferenceClass::OpaqueTaskReference,
        RuntimeTaskOutputObservation::SpawnObserved,
        CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
        true,
        false,
    );
    assert_eq!(
        worker_without_mailbox_bound,
        Err(RuntimeTaskLifecyclePolicyError::QueueMailboxBoundMissing)
    );

    let policy = RuntimeTaskLifecyclePolicy::try_new(
        RuntimeTaskClass::DriverIoWorker,
        Some(SupervisionScope::DriverComponent),
        RuntimeTaskOwningLayer::Driver,
        RuntimeTaskInputReferenceClass::OpaqueTaskReference,
        RuntimeTaskOutputObservation::SpawnObserved,
        CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
        true,
        true,
    )
    .expect("bounded supervised worker policy is valid");
    assert_eq!(
        RuntimeTaskLifecycleDecision::try_new(
            StartupRunId::new(reference("startup-ce2")),
            Some(correlation()),
            policy,
            Some("task-ref-ce2"),
            RuntimeTaskLifecycleOutcome::Failed,
            None,
        ),
        Err(RuntimeTaskLifecycleDecisionError::ReasonRequired)
    );

    assert_eq!(
        CheckpointIntent::try_new(
            StateFamily::SignalingRoom,
            StateClass::EphemeralCoreState,
            CheckpointOwnerBoundary::CoreIntentAndVersion,
            true,
            false,
        ),
        Err(CheckpointIntentError::StateClassNotCheckpointEligible)
    );

    assert_eq!(
        RestorePreconditionSet::try_new(false, true, true, true, true, true, true, true, true),
        Err(RestorePreconditionError::LifecycleObservationNotClassified)
    );

    assert_eq!(
        DistributedStatePolicy::try_new(
            StateFamily::SfuForwardingState,
            DistributedStateClass::AffinityRequiredState,
            OwnerNodeScope::SingleNode,
            Some(reference("owner-ce2")),
            None,
            CommandRoutingRule::OwnerAffinityRequired,
            PacketRoutingRule::OwnerAffinityRequired,
            RecoveryRestoreRelation::NoRestoreRelation,
            DistributedConflictRule::RejectConflictingOwner,
            true,
            DistributedAuditRelation::DistributedStateFailoverDecision,
        ),
        Err(DistributedStatePolicyError::AffinityKeyMissing)
    );

    let packet_cache_action = REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .find(|action| action.resource() == ResourceBoundKind::SfuPacketCache)
        .expect("packet cache resource bound action must be cataloged");
    assert_eq!(
        ResourceBoundDecision::try_new(*packet_cache_action, ResourceBoundReferenceSet::none()),
        Err(ResourceBoundDecisionShapeError::PacketReferenceMissing)
    );

    assert_eq!(
        AuditTimestamp::new(""),
        Err(arcrtc_core_audit::AuditEventShapeError::EmptyTimestamp)
    );
    assert_eq!(
        CanonicalEventPayloadDigest::new(HashAlgorithm::Sha256, vec![]),
        Err(HashChainRecordError::EmptyDigest)
    );

    let event_type = find_audit_event_definition("signaling_join_decision")
        .expect("signaling_join_decision audit type must be cataloged");
    let audit_reason = arcrtc_core_reason::Reason::new(
        cataloged("missing_correlation_id").definition(),
        None::<()>,
    );
    let success_with_reason = AuditEvent::new(AuditEventInput {
        event_id: AuditEventId::new(reference("audit-ce2")),
        correlation: AuditReferencePresence::Present {
            kind: AuditReferenceKind::Correlation,
            value: AuditReferenceValue::CorrelationId(correlation()),
        },
        timestamp: AuditTimestamp::new("2026-06-15T00:00:00Z").expect("timestamp"),
        component: AuditComponent::Core,
        event_type,
        outcome: UseCaseOutcome::Accepted,
        reason: AuditReason::Cataloged(audit_reason),
        references: AuditReferences::default(),
        resource_owner: None,
        tags: vec![],
    });
    assert_eq!(
        success_with_reason,
        Err(arcrtc_core_audit::AuditEventShapeError::SuccessMustNotCarryReason)
    );

    assert_eq!(
        TimePolicyTarget::TurnAllocationExpiry.failure_reason(),
        "allocation_lifetime_exceeded"
    );
}

#[test]
fn ce2_protocol_idempotency_and_canonical_serialization_are_individually_fail_closed() {
    let incomplete_rules = CanonicalEncodingRuleSet::new(
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::RequiresAdrOrCanonical,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        UnknownFieldHandling::Reject,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
    );
    assert!(!incomplete_rules.usable_for_canonical_evidence());
    assert_eq!(
        CanonicalDigest::new(
            CanonicalFormatVersion::new("command", "v1"),
            "sha256",
            vec![]
        ),
        Err(CanonicalEncodingError::EmptyDigestMaterial)
    );
    assert_eq!(
        CanonicalEncodingFailureKind::CanonicalSerializationMismatch.reason_code(),
        "canonical_serialization_mismatch"
    );

    let owner =
        VersionedSurfaceOwnership::new(VersionedSurface::SignalingContract, VersionOwner::Core);
    assert_eq!(owner.owner(), VersionOwner::Core);
    let unsupported = arcrtc_core_protocol::VersionNegotiationOutcome::RejectedUnsupported(
        ContractVersion::new(9, 0, 0),
    );
    assert!(matches!(
        unsupported,
        arcrtc_core_protocol::VersionNegotiationOutcome::RejectedUnsupported(_)
    ));
    assert_eq!(
        CompatibilityChange::RemoveField.classification(),
        CompatibilityClassification::Breaking
    );
    assert_eq!(
        CompatibilityFailureKind::UnsupportedCommandVersion.reason_code(),
        "unsupported_command_version"
    );
    let deprecation = DeprecationDecision::new(
        VersionedSurface::SignalingContract,
        ContractVersion::new(1, 0, 0),
        VersionOwner::Core,
        CompatibilityVersionRange::new(
            VersionedSurface::SignalingContract,
            ContractVersion::new(1, 0, 0),
            ContractVersion::new(1, 9, 9),
        ),
        vec![
            DeprecationLifecycleStep::DefineUnsupportedVersionBehavior,
            DeprecationLifecycleStep::AddCompatibilityNegativePlan,
        ],
    );
    assert!(deprecation
        .lifecycle_steps()
        .contains(&DeprecationLifecycleStep::DefineUnsupportedVersionBehavior));

    assert_eq!(
        IdempotencyScope::new(""),
        Err(arcrtc_core_command::IdempotencyValueError::Empty)
    );
    assert_eq!(
        SemanticPayloadDigest::new("", vec![1]),
        Err(arcrtc_core_command::IdempotencyValueError::Empty)
    );
    let identity: CommandIdentity<OpaqueReference, OpaqueReference> = CommandIdentity::new(
        CommandType::new("join_room"),
        IdempotencyScope::new("room-scope").expect("valid scope"),
        reference("room-target"),
        Some(reference("participant-actor")),
        IdempotencyKey::new("idem-key").expect("valid key"),
        Some(SemanticPayloadDigest::new("sha256", vec![1, 2, 3]).expect("valid digest")),
        ReplayWindow::Bounded("command-window"),
    );
    assert_eq!(identity.scope().as_str(), "room-scope");
    let replay_conflict = IdempotencyDecision::new(
        IdempotencyClass::IdempotentConflict,
        UseCaseOutcome::Rejected,
        DecisionReason::Cataloged(cataloged(
            IdempotencyFailureKind::IdempotencyPayloadMismatch.reason_code(),
        )),
        None,
    )
    .expect("rejected idempotency conflict must carry a cataloged reason");
    assert_eq!(
        replay_conflict.class(),
        IdempotencyClass::IdempotentConflict
    );
}

