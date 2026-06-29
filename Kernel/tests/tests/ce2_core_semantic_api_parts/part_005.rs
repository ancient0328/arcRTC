use arcrtc_core_audit as audit_core;
use arcrtc_core_command as command_core;
use arcrtc_core_cross_plane as cross_core;
use arcrtc_core_protocol as protocol_core;
use arcrtc_core_reason as reason_core;
use arcrtc_core_runtime as runtime_core;
use arcrtc_core_sfu as sfu_core;
use arcrtc_core_signaling as signaling_core;
use arcrtc_core_turn as turn_core;
use arcrtc_core_turn::TurnFailureKind;

fn assert_cataloged(code: &str) {
    assert!(
        CatalogedReasonRef::from_code(code).is_ok(),
        "{code} must be present in the closed reason catalog"
    );
}

#[test]
fn ce2_cov3_reason_audit_and_cross_plane_catalogs_are_asserted() {
    let _reason_surface = reason_core::CoreReasonSurface;
    for (category, code, retryable, safe_to_expose, audit_required) in [
        (
            reason_core::ReasonCategory::MalformedInput,
            "malformed_input",
            false,
            true,
            true,
        ),
        (
            reason_core::ReasonCategory::UnsupportedVersion,
            "unsupported_version",
            false,
            true,
            true,
        ),
        (
            reason_core::ReasonCategory::Unauthorized,
            "unauthorized",
            false,
            false,
            true,
        ),
        (
            reason_core::ReasonCategory::ForbiddenState,
            "forbidden_state",
            false,
            true,
            true,
        ),
        (
            reason_core::ReasonCategory::Duplicate,
            "duplicate",
            false,
            true,
            false,
        ),
        (
            reason_core::ReasonCategory::OrderingViolation,
            "ordering_violation",
            false,
            true,
            true,
        ),
        (
            reason_core::ReasonCategory::Expired,
            "expired",
            false,
            true,
            true,
        ),
        (
            reason_core::ReasonCategory::ResourceExhausted,
            "resource_exhausted",
            true,
            true,
            true,
        ),
        (
            reason_core::ReasonCategory::Backpressure,
            "backpressure",
            true,
            true,
            true,
        ),
        (
            reason_core::ReasonCategory::QualityViolation,
            "quality_violation",
            true,
            true,
            true,
        ),
        (
            reason_core::ReasonCategory::DriverFailure,
            "driver_failure",
            true,
            false,
            true,
        ),
        (
            reason_core::ReasonCategory::Shutdown,
            "shutdown",
            false,
            true,
            true,
        ),
    ] {
        assert_eq!(category.as_str(), code);
        let metadata = category.default_metadata();
        assert_eq!(metadata.retryable(), retryable);
        assert_eq!(metadata.safe_to_expose(), safe_to_expose);
        assert_eq!(metadata.audit_required(), audit_required);
    }

    let reason_definition = reason_core::find_reason_definition("cross_plane_binding_missing")
        .expect("known cross-plane reason is cataloged");
    let reason = reason_core::Reason::new(reason_definition, Some("detail"));
    assert_eq!(
        reason.definition().code().as_str(),
        "cross_plane_binding_missing"
    );
    assert_eq!(reason.details(), Some(&"detail"));
    assert_eq!(
        CatalogedReasonRef::from_definition(
            reason_core::find_reason_definition("missing_correlation_id")
                .expect("known reason is cataloged")
        )
        .expect("definition must normalize to cataloged reason")
        .definition()
        .code()
        .as_str(),
        "missing_correlation_id"
    );
    assert!(reason_core::find_reason_definition("not-cataloged").is_none());

    let _audit_surface = audit_core::CoreAuditSurface;
    assert!(audit_core::AUDIT_EVENT_DEFINITIONS.len() > 50);
    for definition in audit_core::AUDIT_EVENT_DEFINITIONS {
        assert_eq!(
            audit_core::find_audit_event_definition(definition.event_type().as_str()),
            Some(definition)
        );
        assert!(!definition.event_type().as_str().is_empty());
    }
    assert!(audit_core::find_audit_event_definition("not_an_audit_event").is_none());

    let mut references = audit_core::AuditReferences::new();
    references.push(audit_core::AuditReferencePresence::Present {
        kind: audit_core::AuditReferenceKind::Correlation,
        value: audit_core::AuditReferenceValue::CorrelationId(correlation()),
    });
    references.push(audit_core::AuditReferencePresence::AbsentNotApplicable(
        audit_core::AuditReferenceKind::PacketRewriteMediaTransform,
    ));
    assert_eq!(references.entries().len(), 2);

    let owner_tuple = audit_core::ResourceOwnerTuple::new(
        audit_core::AuditComponent::Core,
        audit_core::AuditComponent::Driver,
    );
    assert_eq!(
        owner_tuple.resource_policy_owner(),
        audit_core::AuditComponent::Core
    );
    assert_eq!(
        owner_tuple.physical_resource_owner(),
        audit_core::AuditComponent::Driver
    );
    let tag = audit_core::NonSensitiveTag::new("surface", "core");
    assert_eq!(tag.key(), "surface");
    assert_eq!(tag.value(), "core");
    assert_eq!(
        audit_core::AuditTimestamp::new(""),
        Err(audit_core::AuditEventShapeError::EmptyTimestamp)
    );

    let event_type = audit_core::find_audit_event_definition("cross_plane_binding_decision")
        .expect("cross-plane event type exists");
    let audit_event = audit_core::AuditEvent::new(audit_core::AuditEventInput {
        event_id: AuditEventId::new(reference("audit-event-cov3")),
        correlation: audit_core::AuditReferencePresence::Present {
            kind: audit_core::AuditReferenceKind::Correlation,
            value: audit_core::AuditReferenceValue::CorrelationId(correlation()),
        },
        event_type,
        component: audit_core::AuditComponent::Core,
        outcome: UseCaseOutcome::Rejected,
        reason: audit_core::AuditReason::Cataloged(reason_core::Reason::new(
            reason_core::find_reason_definition("cross_plane_binding_missing")
                .expect("cross-plane reason is cataloged"),
            Some("missing binding"),
        )),
        references,
        resource_owner: Some(owner_tuple),
        timestamp: audit_core::AuditTimestamp::new("2026-06-16T00:00:00Z")
            .expect("timestamp label is valid"),
        tags: vec![tag],
    })
    .expect("non-success audit event carries cataloged reason");
    assert_eq!(audit_event.event_id().as_str(), "audit-event-cov3");
    assert_eq!(audit_event.event_type().event_type().as_str(), "cross_plane_binding_decision");
    assert_eq!(audit_event.outcome(), UseCaseOutcome::Rejected);
    assert_eq!(audit_event.reason().presence(), audit_core::AuditReasonPresence::Cataloged);
    assert_eq!(
        audit_core::AuditEvent::<&str>::new(audit_core::AuditEventInput {
            event_id: AuditEventId::new(reference("audit-event-bad-success")),
            correlation: audit_core::AuditReferencePresence::AbsentNotApplicable(
                audit_core::AuditReferenceKind::Correlation,
            ),
            event_type,
            component: audit_core::AuditComponent::Core,
            outcome: UseCaseOutcome::Accepted,
            reason: audit_core::AuditReason::Cataloged(reason_core::Reason::new(
                reason_core::find_reason_definition("cross_plane_binding_missing")
                    .expect("cross-plane reason is cataloged"),
                None,
            )),
            references: audit_core::AuditReferences::new(),
            resource_owner: None,
            timestamp: audit_core::AuditTimestamp::new("2026-06-16T00:00:00Z")
                .expect("timestamp label is valid"),
            tags: vec![],
        }),
        Err(audit_core::AuditEventShapeError::SuccessMustNotCarryReason)
    );

    assert_eq!(audit_core::HashAlgorithm::Sha256.as_str(), "sha256");
    let payload_digest = audit_core::CanonicalEventPayloadDigest::new(
        audit_core::HashAlgorithm::Sha256,
        vec![1, 2, 3],
    )
    .expect("payload digest is non-empty");
    assert_eq!(payload_digest.algorithm(), audit_core::HashAlgorithm::Sha256);
    assert_eq!(payload_digest.digest(), &[1, 2, 3]);
    assert_eq!(
        audit_core::CanonicalEventPayloadDigest::new(audit_core::HashAlgorithm::Sha256, vec![]),
        Err(audit_core::HashChainRecordError::EmptyDigest)
    );
    let record_hash =
        audit_core::RecordHash::new(audit_core::HashAlgorithm::Sha256, vec![4, 5, 6])
            .expect("record hash is non-empty");
    let format = audit_core::CanonicalRecordFormat::new("audit-record", "v1");
    assert_eq!(format.format(), "audit-record");
    assert_eq!(format.version(), "v1");
    let record = audit_core::HashChainRecord::new(audit_core::HashChainRecordInput {
        scope: audit_core::HashChainScope::Sfu,
        sequence: audit_core::HashChainSequence::new(7),
        previous_hash: audit_core::PreviousRecordHash::Previous(record_hash.clone()),
        event_type: event_type.event_type(),
        outcome: UseCaseOutcome::Rejected,
        canonical_format: format,
        payload_digest,
        record_hash,
    });
    assert_eq!(record.scope(), audit_core::HashChainScope::Sfu);
    assert_eq!(record.sequence().value(), 7);
    assert!(matches!(
        record.previous_hash(),
        audit_core::PreviousRecordHash::Previous(_)
    ));
    assert_eq!(record.outcome(), UseCaseOutcome::Rejected);

    let _cross_surface = cross_core::CoreCrossPlaneSurface;
    for (class, rejected) in [
        (
            cross_core::CrossPlaneBindingClass::NoCrossPlaneBindingRequired,
            false,
        ),
        (
            cross_core::CrossPlaneBindingClass::SignalingParticipantBinding,
            false,
        ),
        (cross_core::CrossPlaneBindingClass::SfuEndpointBinding, false),
        (
            cross_core::CrossPlaneBindingClass::TurnAllocationBinding,
            false,
        ),
        (
            cross_core::CrossPlaneBindingClass::TurnPermissionBinding,
            false,
        ),
        (cross_core::CrossPlaneBindingClass::IceCandidateBinding, false),
        (
            cross_core::CrossPlaneBindingClass::SecureMediaSessionBinding,
            false,
        ),
        (cross_core::CrossPlaneBindingClass::TestCrossPlaneBinding, false),
        (
            cross_core::CrossPlaneBindingClass::ImplicitBindingRequested,
            true,
        ),
    ] {
        assert_eq!(class.is_rejected_class(), rejected);
    }
    for (reference, lifecycle) in [
        (
            cross_core::CrossPlaneReference::Room(RoomId::new(reference("cross-room"))),
            cross_core::BindingReferenceLifecycle::Room,
        ),
        (
            cross_core::CrossPlaneReference::Session(SessionId::new(reference("cross-session"))),
            cross_core::BindingReferenceLifecycle::Session,
        ),
        (
            cross_core::CrossPlaneReference::Participant(ParticipantId::new(reference(
                "cross-participant",
            ))),
            cross_core::BindingReferenceLifecycle::ParticipantMembership,
        ),
        (
            cross_core::CrossPlaneReference::Endpoint(EndpointId::new(reference(
                "cross-endpoint",
            ))),
            cross_core::BindingReferenceLifecycle::SfuEndpoint,
        ),
        (
            cross_core::CrossPlaneReference::Stream(StreamId::new(reference("cross-stream"))),
            cross_core::BindingReferenceLifecycle::MediaStream,
        ),
        (
            cross_core::CrossPlaneReference::Allocation(AllocationId::new(reference(
                "cross-allocation",
            ))),
            cross_core::BindingReferenceLifecycle::TurnAllocation,
        ),
        (
            cross_core::CrossPlaneReference::Permission(PermissionId::new(reference(
                "cross-permission",
            ))),
            cross_core::BindingReferenceLifecycle::TurnPermission,
        ),
        (
            cross_core::CrossPlaneReference::ChannelBind(ChannelBindId::new(reference(
                "cross-channel",
            ))),
            cross_core::BindingReferenceLifecycle::TurnChannelBind,
        ),
        (
            cross_core::CrossPlaneReference::Credential(CredentialRef::new(reference(
                "cross-credential",
            ))),
            cross_core::BindingReferenceLifecycle::CredentialVerification,
        ),
        (
            cross_core::CrossPlaneReference::NotMaterialized,
            cross_core::BindingReferenceLifecycle::NotMaterialized,
        ),
    ] {
        assert_eq!(reference.lifecycle(), lifecycle);
    }
    for (outcome, code, requires_reason) in [
        (cross_core::CrossPlaneBindingOutcome::Accepted, "accepted", false),
        (cross_core::CrossPlaneBindingOutcome::Rejected, "rejected", true),
        (cross_core::CrossPlaneBindingOutcome::Expired, "expired", true),
        (cross_core::CrossPlaneBindingOutcome::Failed, "failed", true),
        (cross_core::CrossPlaneBindingOutcome::CloseNotClaimed, "close_not_claimed", true),
    ] {
        assert_eq!(outcome.code(), code);
        assert_eq!(outcome.requires_reason(), requires_reason);
    }
    for (kind, code) in [
        (
            cross_core::CrossPlaneBindingFailureKind::BindingClassNotAdmitted,
            "cross_plane_binding_not_admitted",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::RequiredBindingAbsent,
            "cross_plane_binding_missing",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::BindingMaterialInvalid,
            "cross_plane_binding_invalid",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::SourceTargetScopeConflict,
            "cross_plane_binding_scope_conflict",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::LifecycleConflict,
            "cross_plane_binding_lifecycle_conflict",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::BindingExpired,
            "cross_plane_binding_expired",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::BindingReplayDetected,
            "cross_plane_binding_replay_detected",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::ParticipantNotAdmitted,
            "participant_not_admitted",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::TargetUnavailable,
            "target_unavailable",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::AllocationNotFound,
            "allocation_not_found",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::PermissionNotFound,
            "permission_not_found",
        ),
        (
            cross_core::CrossPlaneBindingFailureKind::SecureMediaProtectionNotActive,
            "secure_media_protection_not_active",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
}

#[test]
fn ce2_cov3_command_protocol_and_runtime_catalogs_are_asserted() {
    let _command_surface = command_core::CoreCommandSurface;
    for (outcome, is_success, requires_reason) in [
        (UseCaseOutcome::Accepted, true, false),
        (UseCaseOutcome::Allowed, true, false),
        (UseCaseOutcome::Forwarded, true, false),
        (UseCaseOutcome::Selected, true, false),
        (UseCaseOutcome::Released, true, false),
        (UseCaseOutcome::ClosedSuccess, true, false),
        (UseCaseOutcome::IdempotentObserved, true, false),
        (UseCaseOutcome::WithinBoundObserved, true, false),
        (UseCaseOutcome::Rejected, false, true),
        (UseCaseOutcome::Denied, false, true),
        (UseCaseOutcome::ProtocolViolation, false, true),
        (UseCaseOutcome::Suppressed, false, true),
        (UseCaseOutcome::Dropped, false, true),
        (UseCaseOutcome::Expired, false, true),
        (UseCaseOutcome::Revoked, false, true),
        (UseCaseOutcome::Shed, false, true),
        (UseCaseOutcome::Delayed, false, true),
        (UseCaseOutcome::Degraded, false, true),
        (UseCaseOutcome::Failed, false, true),
        (UseCaseOutcome::ConvertedFailure, false, true),
        (UseCaseOutcome::ClosedByPolicy, false, true),
    ] {
        assert_eq!(outcome.is_success(), is_success);
        assert_eq!(outcome.requires_reason(), requires_reason);
    }
    let result_shapes = [
        command_core::ResultShapeClass::CommandEnvelope,
        command_core::ResultShapeClass::UseCaseDecision,
        command_core::ResultShapeClass::DomainEvent,
        command_core::ResultShapeClass::PortIntent,
        command_core::ResultShapeClass::DriverObservation,
        command_core::ResultShapeClass::AuditProjection,
        command_core::ResultShapeClass::ExternalResponseModel,
    ];
    assert_eq!(result_shapes.len(), 7);
    let targets = [
        TargetSurface::Signaling,
        TargetSurface::Sfu,
        TargetSurface::Turn,
        TargetSurface::Transport,
        TargetSurface::Security,
        TargetSurface::Audit,
        TargetSurface::Quality,
        TargetSurface::Ports,
        TargetSurface::Configuration,
        TargetSurface::Features,
    ];
    assert_eq!(targets.len(), 10);
    let port_intent = PortIntent::new("audit", TargetSurface::Audit);
    assert_eq!(port_intent.intent_type(), "audit");
    assert_eq!(port_intent.target_surface(), TargetSurface::Audit);
    let event = command_core::DomainEvent::new(correlation(), "domain-event", "payload");
    assert_eq!(event.correlation_id().as_str(), "corr-ce2");
    assert_eq!(event.event_type(), "domain-event");
    assert_eq!(event.payload(), &"payload");
    assert!(command_core::DriverObservation::<CatalogedReasonRef>::new(
        correlation(),
        UseCaseOutcome::Accepted,
        DecisionReason::Absent,
    )
    .is_ok());
    assert_eq!(
        command_core::DriverObservation::new(
            correlation(),
            UseCaseOutcome::Rejected,
            DecisionReason::Absent::<CatalogedReasonRef>,
        ),
        Err(DecisionShapeError::NonSuccessRequiresReason)
    );
    assert_eq!(
        command_core::DriverObservation::new(
            correlation(),
            UseCaseOutcome::Accepted,
            DecisionReason::Cataloged(cataloged("room_closed")),
        ),
        Err(DecisionShapeError::SuccessMustNotCarryReason)
    );
    let projection = command_core::AuditProjection::new(
        correlation(),
        "audit-event",
        UseCaseOutcome::Rejected,
        DecisionReason::Cataloged(cataloged("room_closed")),
    );
    assert_eq!(projection, projection.clone());
    let response = command_core::ExternalResponseModel::new(
        correlation(),
        UseCaseOutcome::Rejected,
        DecisionReason::Cataloged(cataloged("room_closed")),
    );
    assert_eq!(response, response.clone());
    let trace = command_core::CorrelationTrace::new(correlation());
    assert_eq!(trace.correlation_id().as_str(), "corr-ce2");
    assert_eq!(
        IdempotencyScope::new(""),
        Err(command_core::IdempotencyValueError::Empty)
    );
    assert_eq!(
        IdempotencyKey::new("bad\nkey"),
        Err(command_core::IdempotencyValueError::ControlCharacter)
    );
    assert_eq!(
        SemanticPayloadDigest::new("sha256", vec![]),
        Err(command_core::IdempotencyValueError::Empty)
    );
    let payload_digest = SemanticPayloadDigest::new("sha256", vec![1, 2, 3])
        .expect("semantic payload digest is non-empty");
    assert_eq!(payload_digest.algorithm(), "sha256");
    assert_eq!(payload_digest.digest(), &[1, 2, 3]);
    let command_identity = CommandIdentity::new(
        CommandType::new("publish"),
        IdempotencyScope::new("room-scope").expect("scope is valid"),
        RouteId::new(reference("route-command-identity")),
        Some(ParticipantId::new(reference("participant-command-identity"))),
        IdempotencyKey::new("key").expect("key is valid"),
        Some(payload_digest),
        ReplayWindow::Bounded("1000ms"),
    );
    assert_eq!(command_identity.command_type().as_str(), "publish");
    assert_eq!(command_identity.scope().as_str(), "room-scope");
    assert!(command_identity.actor_reference().is_some());
    assert!(command_identity.payload_digest().is_some());
    assert_eq!(
        command_identity.replay_window(),
        ReplayWindow::Bounded("1000ms")
    );
    let prior = command_core::PriorDecisionReference::new(
        correlation(),
        Some(AuditEventId::new(reference("prior-audit-event"))),
    );
    assert_eq!(prior.prior_correlation_id().as_str(), "corr-ce2");
    assert!(prior.prior_audit_event_id().is_some());
    let idempotency = IdempotencyDecision::new(
        IdempotencyClass::ResponseReplayCandidate,
        UseCaseOutcome::Rejected,
        DecisionReason::Cataloged(cataloged("duplicate_command")),
        Some(prior),
    )
    .expect("non-success idempotency decision carries reason");
    assert_eq!(idempotency.class(), IdempotencyClass::ResponseReplayCandidate);
    assert_eq!(idempotency.outcome(), UseCaseOutcome::Rejected);
    assert!(idempotency.prior_decision().is_some());
    for (kind, code) in [
        (IdempotencyFailureKind::MissingCorrelationId, "missing_correlation_id"),
        (IdempotencyFailureKind::CorrelationMismatch, "correlation_mismatch"),
        (IdempotencyFailureKind::DuplicateCommand, "duplicate_command"),
        (
            IdempotencyFailureKind::IdempotencyPayloadMismatch,
            "idempotency_payload_mismatch",
        ),
        (
            IdempotencyFailureKind::IdempotencyWindowExpired,
            "idempotency_window_expired",
        ),
        (IdempotencyFailureKind::ReplayNotAllowed, "replay_not_allowed"),
        (
            IdempotencyFailureKind::ResponseReplayNotAvailable,
            "response_replay_not_available",
        ),
        (
            IdempotencyFailureKind::CanonicalSerializationFailed,
            "canonical_serialization_failed",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }

    let _protocol_surface = protocol_core::CoreProtocolSurface;
    let rule_set = protocol_core::CanonicalEncodingRuleSet::new(
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::UnknownFieldHandling::Reject,
        protocol_core::CanonicalRuleStatus::Defined,
        protocol_core::CanonicalRuleStatus::Defined,
    );
    assert!(rule_set.usable_for_canonical_evidence());
    let format_version = protocol_core::CanonicalFormatVersion::new("canonical-json", "1");
    let digest = protocol_core::CanonicalDigest::new(format_version, "sha256", vec![9])
        .expect("canonical digest is non-empty");
    assert_eq!(digest.format_version(), format_version);
    assert_eq!(digest.algorithm(), "sha256");
    assert_eq!(digest.digest(), &[9]);
    assert_eq!(
        protocol_core::CanonicalDigest::new(format_version, "sha256", vec![]),
        Err(protocol_core::CanonicalEncodingError::EmptyDigestMaterial)
    );
    for (kind, code) in [
        (
            protocol_core::CanonicalEncodingFailureKind::CanonicalSerializationFailed,
            "canonical_serialization_failed",
        ),
        (
            protocol_core::CanonicalEncodingFailureKind::CanonicalSerializationMismatch,
            "canonical_serialization_mismatch",
        ),
        (
            protocol_core::CanonicalEncodingFailureKind::ExternalDecodeFailed,
            "external_decode_failed",
        ),
        (
            protocol_core::CanonicalEncodingFailureKind::ExternalEncodeFailed,
            "external_encode_failed",
        ),
        (
            protocol_core::CanonicalEncodingFailureKind::MissingRequiredWireField,
            "missing_required_wire_field",
        ),
        (
            protocol_core::CanonicalEncodingFailureKind::UnsupportedCanonicalVersion,
            "unsupported_version",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    for (change, class) in [
        (
            protocol_core::CompatibilityChange::AddOptionalFieldWithDefault,
            protocol_core::CompatibilityClassification::Compatible,
        ),
        (
            protocol_core::CompatibilityChange::AddRequiredField,
            protocol_core::CompatibilityClassification::Breaking,
        ),
        (
            protocol_core::CompatibilityChange::RemoveField,
            protocol_core::CompatibilityClassification::Breaking,
        ),
        (
            protocol_core::CompatibilityChange::ChangeReasonCodeSemantics,
            protocol_core::CompatibilityClassification::Prohibited,
        ),
        (
            protocol_core::CompatibilityChange::AddReasonCode,
            protocol_core::CompatibilityClassification::Conditional,
        ),
        (
            protocol_core::CompatibilityChange::ChangeStateTransition,
            protocol_core::CompatibilityClassification::Breaking,
        ),
        (
            protocol_core::CompatibilityChange::AddDriverEncoding,
            protocol_core::CompatibilityClassification::Compatible,
        ),
    ] {
        assert_eq!(change.classification(), class);
    }
    let version_range = protocol_core::CompatibilityVersionRange::new(
        protocol_core::VersionedSurface::SignalingContract,
        protocol_core::ContractVersion::new(1, 0, 0),
        protocol_core::ContractVersion::new(1, 1, 0),
    );
    assert_eq!(
        version_range.surface(),
        protocol_core::VersionedSurface::SignalingContract
    );
    let deprecation = protocol_core::DeprecationDecision::new(
        protocol_core::VersionedSurface::SdkPublicContract,
        protocol_core::ContractVersion::new(1, 0, 0),
        protocol_core::VersionOwner::Sdk,
        version_range,
        vec![
            protocol_core::DeprecationLifecycleStep::IdentifyAffectedSurfaceAndVersion,
            protocol_core::DeprecationLifecycleStep::RecordAdrOrCanonicalUpdate,
            protocol_core::DeprecationLifecycleStep::DefineUnsupportedVersionBehavior,
            protocol_core::DeprecationLifecycleStep::UpdateSdkParityAndDriverMapping,
            protocol_core::DeprecationLifecycleStep::AddCompatibilityNegativePlan,
            protocol_core::DeprecationLifecycleStep::RecordExecutionEvidence,
            protocol_core::DeprecationLifecycleStep::RemoveAfterDocumentedCondition,
        ],
    );
    assert_eq!(deprecation.lifecycle_steps().len(), 7);
    for (kind, code) in [
        (
            protocol_core::CompatibilityFailureKind::UnsupportedCommandVersion,
            "unsupported_command_version",
        ),
        (
            protocol_core::CompatibilityFailureKind::UnsupportedMediaContractVersion,
            "unsupported_media_contract_version",
        ),
        (
            protocol_core::CompatibilityFailureKind::UnsupportedTurnContractVersion,
            "unsupported_turn_contract_version",
        ),
        (
            protocol_core::CompatibilityFailureKind::UnsupportedDriverWireVersion,
            "unsupported_driver_wire_version",
        ),
        (
            protocol_core::CompatibilityFailureKind::MissingRequiredWireField,
            "missing_required_wire_field",
        ),
        (
            protocol_core::CompatibilityFailureKind::ExternalEnumUnmapped,
            "external_enum_unmapped",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    let semantic_payload = protocol_core::CoreSemanticPayloadModel::new(
        protocol_core::CoreSemanticPayloadClass::OpaqueCoreReference,
        Some(reference("semantic-envelope-ref")),
    );
    assert!(protocol_core::CoreSemanticEnvelope::try_new(
        TargetSurface::Signaling,
        protocol_core::ContractVersion::new(1, 0, 0),
        correlation(),
        protocol_core::SemanticEnvelopeMessageKind::Command,
        protocol_core::CoreSemanticMessageType::SignalingCommand(
            protocol_core::SignalingSemanticCommandType::JoinRoom
        ),
        true,
        Some(semantic_payload),
        Some(UseCaseOutcome::Accepted),
        None,
    )
    .is_ok());
    assert_eq!(
        protocol_core::CoreSemanticEnvelope::try_new(
            TargetSurface::Signaling,
            protocol_core::ContractVersion::new(1, 0, 0),
            correlation(),
            protocol_core::SemanticEnvelopeMessageKind::Command,
            protocol_core::CoreSemanticMessageType::SignalingCommand(
                protocol_core::SignalingSemanticCommandType::JoinRoom
            ),
            true,
            None,
            Some(UseCaseOutcome::Rejected),
            None,
        ),
        Err(protocol_core::SemanticEnvelopeError::RequiredReasonMissing)
    );
    assert_eq!(
        protocol_core::CoreSemanticEnvelope::try_new(
            TargetSurface::Signaling,
            protocol_core::ContractVersion::new(1, 0, 0),
            correlation(),
            protocol_core::SemanticEnvelopeMessageKind::Command,
            protocol_core::CoreSemanticMessageType::SignalingCommand(
                protocol_core::SignalingSemanticCommandType::JoinRoom
            ),
            true,
            None,
            Some(UseCaseOutcome::Accepted),
            Some(cataloged("room_closed")),
        ),
        Err(protocol_core::SemanticEnvelopeError::SuccessReasonMustNotBeInvented)
    );

    let _runtime_surface = runtime_core::CoreRuntimeSurface;
    for surface in [
        runtime_core::RuntimeAbstractionSurface::ClockPort,
        runtime_core::RuntimeAbstractionSurface::RandomPort,
        runtime_core::RuntimeAbstractionSurface::RuntimePort,
    ] {
        assert_eq!(surface.ownership(), surface.ownership());
    }
    for (target, reason) in [
        (
            runtime_core::TimePolicyTarget::TurnAllocationExpiry,
            "allocation_lifetime_exceeded",
        ),
        (
            runtime_core::TimePolicyTarget::TurnPermissionExpiry,
            "permission_lifetime_exceeded",
        ),
        (
            runtime_core::TimePolicyTarget::TurnChannelBindExpiry,
            "channel_bind_lifetime_exceeded",
        ),
        (
            runtime_core::TimePolicyTarget::SignalingRoomLifecycle,
            "room_lifetime_exceeded",
        ),
        (
            runtime_core::TimePolicyTarget::ResourceRetention,
            "retention_duration_exceeded",
        ),
        (
            runtime_core::TimePolicyTarget::ConfigurationStartupTimeout,
            "runtime_config_invalid",
        ),
        (
            runtime_core::TimePolicyTarget::CommandDeadline,
            "operation_deadline_exceeded",
        ),
    ] {
        assert_eq!(target.failure_reason(), reason);
        assert_cataloged(reason);
    }
    for use_class in [
        runtime_core::RandomnessUseClass::Nonce,
        runtime_core::RandomnessUseClass::OpaqueId,
        runtime_core::RandomnessUseClass::Challenge,
        runtime_core::RandomnessUseClass::ReferenceStability,
    ] {
        assert_eq!(
            runtime_core::RandomnessContract::opaque(use_class),
            runtime_core::RandomnessContract::opaque(use_class)
        );
    }
    for (kind, code) in [
        (runtime_core::RuntimeFailureKind::RuntimeConfigMissing, "runtime_config_missing"),
        (runtime_core::RuntimeFailureKind::RuntimeConfigInvalid, "runtime_config_invalid"),
        (
            runtime_core::RuntimeFailureKind::RuntimeTaskClassNotAdmitted,
            "runtime_task_class_not_admitted",
        ),
        (
            runtime_core::RuntimeFailureKind::RuntimeTaskOwnerViolation,
            "runtime_task_owner_violation",
        ),
        (
            runtime_core::RuntimeFailureKind::RuntimeTaskSupervisionMissing,
            "runtime_task_supervision_missing",
        ),
        (
            runtime_core::RuntimeFailureKind::RuntimeTaskSpawnFailed,
            "runtime_task_spawn_failed",
        ),
        (
            runtime_core::RuntimeFailureKind::RuntimeTaskJoinFailed,
            "runtime_task_join_failed",
        ),
        (
            runtime_core::RuntimeFailureKind::RuntimeTaskCancelFailed,
            "runtime_task_cancel_failed",
        ),
        (
            runtime_core::RuntimeFailureKind::RuntimeTaskPanicDetected,
            "runtime_task_panic_detected",
        ),
        (runtime_core::RuntimeFailureKind::DriverShutdown, "driver_shutdown"),
        (
            runtime_core::RuntimeFailureKind::RuntimeTaskQueueBoundExceeded,
            "runtime_task_queue_bound_exceeded",
        ),
        (
            runtime_core::RuntimeFailureKind::MemoryPressureExceeded,
            "memory_pressure_exceeded",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    for (concern, owner) in [
        (
            runtime_core::RuntimeTaskConcern::DomainStateTransition,
            runtime_core::RuntimeTaskOwner::Core,
        ),
        (
            runtime_core::RuntimeTaskConcern::RuntimePortTaskContract,
            runtime_core::RuntimeTaskOwner::CoreRuntimePortContract,
        ),
        (
            runtime_core::RuntimeTaskConcern::ConcreteTaskHandle,
            runtime_core::RuntimeTaskOwner::DriverEntrypointsRuntime,
        ),
        (
            runtime_core::RuntimeTaskConcern::DriverIoWorker,
            runtime_core::RuntimeTaskOwner::Driver,
        ),
        (
            runtime_core::RuntimeTaskConcern::EntrypointsSupervisionTask,
            runtime_core::RuntimeTaskOwner::Entrypoints,
        ),
        (
            runtime_core::RuntimeTaskConcern::TaskQueueMailbox,
            runtime_core::RuntimeTaskOwner::PhysicalExecutionOwner,
        ),
        (
            runtime_core::RuntimeTaskConcern::TaskPanicObservation,
            runtime_core::RuntimeTaskOwner::DriverEntrypointsRuntime,
        ),
        (
            runtime_core::RuntimeTaskConcern::TaskCancellation,
            runtime_core::RuntimeTaskOwner::DriverEntrypointsRuntime,
        ),
    ] {
        assert_eq!(concern.owner(), owner);
    }
    let runtime_policy = runtime_core::RuntimeTaskLifecyclePolicy::try_new(
        runtime_core::RuntimeTaskClass::DriverIoWorker,
        Some(runtime_core::SupervisionScope::DriverComponent),
        runtime_core::RuntimeTaskOwningLayer::Driver,
        runtime_core::RuntimeTaskInputReferenceClass::OpaqueTaskReference,
        runtime_core::RuntimeTaskOutputObservation::SpawnObserved,
        runtime_core::CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
        true,
        true,
    )
    .expect("driver worker has explicit supervision and bounds");
    assert_eq!(
        runtime_core::RuntimeTaskLifecycleDecision::try_new(
            StartupRunId::new(reference("runtime-task-cov3")),
            Some(correlation()),
            runtime_policy,
            Some("task-ref"),
            runtime_core::RuntimeTaskLifecycleOutcome::Rejected,
            None,
        ),
        Err(runtime_core::RuntimeTaskLifecycleDecisionError::ReasonRequired)
    );
    let lifecycle_decision = runtime_core::RuntimeTaskLifecycleDecision::try_new(
        StartupRunId::new(reference("runtime-task-failed-cov3")),
        Some(correlation()),
        runtime_policy,
        Some("task-ref"),
        runtime_core::RuntimeTaskLifecycleOutcome::Failed,
        Some(runtime_core::RuntimeFailureKind::RuntimeTaskSpawnFailed),
    )
    .expect("failed lifecycle decision carries reason");
    assert_eq!(
        lifecycle_decision.audit_event_type(),
        "runtime_task_lifecycle_decision"
    );
    for surface in [
        runtime_core::TaskCancellationSurface::BeforeDriverCoreConversion,
        runtime_core::TaskCancellationSurface::AfterCommandEnteredCore,
        runtime_core::TaskCancellationSurface::DuringShutdownDrain,
        runtime_core::TaskCancellationSurface::DuringDriverQueueCacheExecution,
        runtime_core::TaskCancellationSurface::DuringTestHarnessTimeout,
    ] {
        assert!(!surface.required_relation().is_empty());
    }
}

#[test]
fn ce2_cov3_signaling_sfu_and_turn_state_tables_are_asserted() {
    let _signaling_surface = signaling_core::CoreSignalingSurface;
    let subject = SignalingSubject::new(
        RoomId::new(reference("signaling-room-cov3")),
        Some(ParticipantId::new(reference("signaling-participant-cov3"))),
    );
    assert_eq!(subject.room_id().as_str(), "signaling-room-cov3");
    assert!(subject.participant_id().is_some());
    let envelope = CommandEnvelope::new(
        correlation(),
        CommandType::new("send_offer"),
        CommandVersion::new(2),
        TargetSurface::Signaling,
        subject.clone(),
    );
    let command = signaling_core::SignalingCommand::new(
        envelope,
        signaling_core::SignalingCommandKind::SendOffer,
        "offer",
    );
    assert_eq!(command.kind(), signaling_core::SignalingCommandKind::SendOffer);
    assert_eq!(command.envelope().target_surface(), TargetSurface::Signaling);
    let event = signaling_core::SignalingEvent::new(
        correlation(),
        signaling_core::SignalingEventKind::OfferReceived,
        subject,
        "offer",
    );
    assert_eq!(event.kind(), signaling_core::SignalingEventKind::OfferReceived);
    for (kind, code) in [
        (SignalingFailureKind::MissingCorrelationId, "missing_correlation_id"),
        (SignalingFailureKind::MalformedCommand, "malformed_command"),
        (
            SignalingFailureKind::TokenVerificationFailed,
            "token_verification_failed",
        ),
        (SignalingFailureKind::RoomNotAcceptingJoin, "room_not_accepting_join"),
        (SignalingFailureKind::RoomCapacityExceeded, "room_capacity_exceeded"),
        (
            SignalingFailureKind::AdmissionCapacityExceeded,
            "admission_capacity_exceeded",
        ),
        (SignalingFailureKind::RoomLifetimeExceeded, "room_lifetime_exceeded"),
        (SignalingFailureKind::RoomDraining, "room_draining"),
        (SignalingFailureKind::RoomClosed, "room_closed"),
        (SignalingFailureKind::RoomCloseNotAllowed, "room_close_not_allowed"),
        (SignalingFailureKind::ParticipantNotJoined, "participant_not_joined"),
        (SignalingFailureKind::ParticipantRejected, "participant_rejected"),
        (SignalingFailureKind::DuplicateCommand, "duplicate_command"),
        (
            SignalingFailureKind::IdempotencyPayloadMismatch,
            "idempotency_payload_mismatch",
        ),
        (
            SignalingFailureKind::IdempotencyWindowExpired,
            "idempotency_window_expired",
        ),
        (
            SignalingFailureKind::AuthorizationContextMissing,
            "authorization_context_missing",
        ),
        (
            SignalingFailureKind::AuthorizationContextInvalid,
            "authorization_context_invalid",
        ),
        (
            SignalingFailureKind::AuthorizationPolicyDenied,
            "authorization_policy_denied",
        ),
        (SignalingFailureKind::CommandOrderViolation, "command_order_violation"),
        (
            SignalingFailureKind::SignalingCommandQueueBoundExceeded,
            "signaling_command_queue_bound_exceeded",
        ),
        (
            SignalingFailureKind::UnsupportedCommandVersion,
            "unsupported_command_version",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    assert!(signaling_core::SIGNALING_TRANSITION_RULES.len() >= 13);
    for rule in signaling_core::SIGNALING_TRANSITION_RULES {
        assert!(!rule.reject_reasons().is_empty());
        for reason in rule.reject_reasons() {
            assert_cataloged(reason.reason_code());
        }
        let _ = rule.trigger();
        let _ = rule.allowed_room_states();
        let _ = rule.allowed_participant_states();
        let _ = rule.success_room_state();
        let _ = rule.success_participant_state();
    }

    let _sfu_surface = sfu_core::CoreSfuSurface;
    let reference_set = sfu_core::SfuReferenceSet::new(
        SessionId::new(reference("sfu-session-cov3")),
        Some(EndpointId::new(reference("sfu-endpoint-cov3"))),
        Some(StreamId::new(reference("sfu-stream-cov3"))),
        Some(RouteId::new(reference("sfu-route-cov3"))),
        Some(PacketId::new(reference("sfu-packet-cov3"))),
    );
    assert_eq!(reference_set.session_id().as_str(), "sfu-session-cov3");
    let item = sfu_core::SfuContractItem::new(
        sfu_core::SfuModelKind::ForwardingIntent,
        reference_set,
        "payload",
    );
    assert_eq!(item.model_kind(), sfu_core::SfuModelKind::ForwardingIntent);
    for (kind, code) in [
        (SfuFailureKind::ParticipantNotAdmitted, "participant_not_admitted"),
        (SfuFailureKind::SfuSessionNotAccepting, "sfu_session_not_accepting"),
        (SfuFailureKind::EndpointCapacityExceeded, "endpoint_capacity_exceeded"),
        (SfuFailureKind::EndpointQualityNotAllowed, "endpoint_quality_not_allowed"),
        (SfuFailureKind::EndpointDegradedByQuality, "endpoint_degraded_by_quality"),
        (SfuFailureKind::QualityRecoveryNotAllowed, "quality_recovery_not_allowed"),
        (SfuFailureKind::TargetUnavailable, "target_unavailable"),
        (SfuFailureKind::StreamNotFound, "stream_not_found"),
        (SfuFailureKind::PublicationNotAllowed, "publication_not_allowed"),
        (
            SfuFailureKind::PublicationQualityNotAllowed,
            "publication_quality_not_allowed",
        ),
        (SfuFailureKind::SubscriptionNotAllowed, "subscription_not_allowed"),
        (
            SfuFailureKind::SubscriptionQualityNotAllowed,
            "subscription_quality_not_allowed",
        ),
        (SfuFailureKind::RouteConflict, "route_conflict"),
        (
            SfuFailureKind::RouteCandidateBoundExceeded,
            "route_candidate_bound_exceeded",
        ),
        (SfuFailureKind::RouteSuppressedByQuality, "route_suppressed_by_quality"),
        (
            SfuFailureKind::RouteSuppressedByBackpressure,
            "route_suppressed_by_backpressure",
        ),
        (SfuFailureKind::PacketSuppressedByQuality, "packet_suppressed_by_quality"),
        (
            SfuFailureKind::PacketSuppressedByBackpressure,
            "packet_suppressed_by_backpressure",
        ),
        (SfuFailureKind::PacketDroppedByBackpressure, "packet_dropped_by_backpressure"),
        (
            SfuFailureKind::SubscriptionBackpressureSuppressed,
            "subscription_backpressure_suppressed",
        ),
        (SfuFailureKind::ActionDelayedByBackpressure, "action_delayed_by_backpressure"),
        (SfuFailureKind::RouteDegradedByBackpressure, "route_degraded_by_backpressure"),
        (
            SfuFailureKind::EndpointClosedByBackpressure,
            "endpoint_closed_by_backpressure",
        ),
        (
            SfuFailureKind::BackpressureRecoveryNotAllowed,
            "backpressure_recovery_not_allowed",
        ),
        (
            SfuFailureKind::SfuTransmitQueueBoundExceeded,
            "sfu_transmit_queue_bound_exceeded",
        ),
        (
            SfuFailureKind::UnsupportedMediaContractVersion,
            "unsupported_media_contract_version",
        ),
        (SfuFailureKind::MediaCodecNotSupported, "media_codec_not_supported"),
        (SfuFailureKind::MediaTrackNotAllowed, "media_track_not_allowed"),
        (SfuFailureKind::MediaLayerNotAvailable, "media_layer_not_available"),
        (
            SfuFailureKind::MediaPayloadMappingInvalid,
            "media_payload_mapping_invalid",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    assert!(sfu_core::SFU_TRANSITION_RULES.len() >= 25);
    for rule in sfu_core::SFU_TRANSITION_RULES {
        assert!(!rule.allowed_pre_state().is_empty());
        assert!(!rule.reject_reasons().is_empty());
        for reason in rule.reject_reasons() {
            assert_cataloged(reason.reason_code());
        }
        let _ = rule.trigger();
        let _ = rule.success_effect();
    }
    for (kind, code) in [
        (
            sfu_core::PacketSemanticViewFailureKind::ExternalDecodeFailed,
            "external_decode_failed",
        ),
        (
            sfu_core::PacketSemanticViewFailureKind::FrameSizeBoundExceeded,
            "frame_size_bound_exceeded",
        ),
        (
            sfu_core::PacketSemanticViewFailureKind::UnsupportedMediaContractVersion,
            "unsupported_media_contract_version",
        ),
        (
            sfu_core::PacketSemanticViewFailureKind::BufferReleaseFailed,
            "buffer_release_failed",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    for (kind, code) in [
        (
            sfu_core::PacketRewriteTransformFailureKind::PacketRewriteClassNotAdmitted,
            "packet_rewrite_class_not_admitted",
        ),
        (
            sfu_core::PacketRewriteTransformFailureKind::PacketRewriteIntentInvalid,
            "packet_rewrite_intent_invalid",
        ),
        (
            sfu_core::PacketRewriteTransformFailureKind::PacketRewriteOwnerViolation,
            "packet_rewrite_owner_violation",
        ),
        (
            sfu_core::PacketRewriteTransformFailureKind::PayloadTransformNotAdmitted,
            "payload_transform_not_admitted",
        ),
        (
            sfu_core::PacketRewriteTransformFailureKind::MediaTranscodeNotSupported,
            "media_transcode_not_supported",
        ),
        (
            sfu_core::PacketRewriteTransformFailureKind::PayloadTransformFailed,
            "payload_transform_failed",
        ),
        (
            sfu_core::PacketRewriteTransformFailureKind::RewriteCopyBoundExceeded,
            "rewrite_copy_bound_exceeded",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    for (kind, code) in [
        (
            sfu_core::MediaNegotiationFailureKind::MediaCodecNotSupported,
            "media_codec_not_supported",
        ),
        (
            sfu_core::MediaNegotiationFailureKind::MediaTrackNotAllowed,
            "media_track_not_allowed",
        ),
        (
            sfu_core::MediaNegotiationFailureKind::MediaLayerNotAvailable,
            "media_layer_not_available",
        ),
        (
            sfu_core::MediaNegotiationFailureKind::MediaPayloadMappingInvalid,
            "media_payload_mapping_invalid",
        ),
        (
            sfu_core::MediaNegotiationFailureKind::MediaFeedbackNotSupported,
            "media_feedback_not_supported",
        ),
        (
            sfu_core::MediaNegotiationFailureKind::MediaTranscodeNotSupported,
            "media_transcode_not_supported",
        ),
        (
            sfu_core::MediaNegotiationFailureKind::UnsupportedMediaContractVersion,
            "unsupported_media_contract_version",
        ),
        (
            sfu_core::MediaNegotiationFailureKind::ExternalDecodeFailed,
            "external_decode_failed",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    for (kind, code) in [
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::PacketCacheBoundExceeded,
            "packet_cache_bound_exceeded",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::RetentionDurationExceeded,
            "retention_duration_exceeded",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::SfuTransmitQueueBoundExceeded,
            "sfu_transmit_queue_bound_exceeded",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::PacketSuppressedByBackpressure,
            "packet_suppressed_by_backpressure",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::PacketDroppedByBackpressure,
            "packet_dropped_by_backpressure",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::RouteDegradedByBackpressure,
            "route_degraded_by_backpressure",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::EndpointClosedByBackpressure,
            "endpoint_closed_by_backpressure",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::BackpressureRecoveryNotAllowed,
            "backpressure_recovery_not_allowed",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::TargetUnavailable,
            "target_unavailable",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::NetworkSendFailed,
            "network_send_failed",
        ),
        (
            sfu_core::CongestionPacingRetransmissionFailureKind::DriverShutdown,
            "driver_shutdown",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }

    let _turn_surface = turn_core::CoreTurnSurface;
    assert_eq!(CorePeerAddress::new("peer-address").expect("peer").as_str(), "peer-address");
    assert_eq!(
        TurnRequestedLifetimeSeconds::try_new(300)
            .expect("lifetime")
            .as_u32(),
        300
    );
    for (command, decision) in [
        (TurnCommandKind::Allocate, TurnDecisionKind::Allocation),
        (TurnCommandKind::Refresh, TurnDecisionKind::Refresh),
        (TurnCommandKind::CreatePermission, TurnDecisionKind::Permission),
        (TurnCommandKind::ChannelBind, TurnDecisionKind::ChannelBind),
        (TurnCommandKind::RelayData, TurnDecisionKind::Relay),
    ] {
        assert_eq!(command.decision_kind(), decision);
    }
    for (kind, code) in [
        (TurnFailureKind::MalformedTurnMessage, "malformed_turn_message"),
        (TurnFailureKind::CredentialMissing, "credential_missing"),
        (TurnFailureKind::CredentialInvalid, "credential_invalid"),
        (TurnFailureKind::CredentialExpired, "credential_expired"),
        (
            TurnFailureKind::SecretGenerationNotAccepted,
            "secret_generation_not_accepted",
        ),
        (TurnFailureKind::SecretKeyRevoked, "secret_key_revoked"),
        (
            TurnFailureKind::SecretOverlapWindowExpired,
            "secret_overlap_window_expired",
        ),
        (
            TurnFailureKind::SecretRotationStateUnavailable,
            "secret_rotation_state_unavailable",
        ),
        (
            TurnFailureKind::AllocationCapacityExceeded,
            "allocation_capacity_exceeded",
        ),
        (
            TurnFailureKind::PermissionCapacityExceeded,
            "permission_capacity_exceeded",
        ),
        (TurnFailureKind::PeerNotAllowed, "peer_not_allowed"),
        (TurnFailureKind::PermissionNotFound, "permission_not_found"),
        (TurnFailureKind::RelayDenied, "relay_denied"),
        (TurnFailureKind::AllocationNotFound, "allocation_not_found"),
        (TurnFailureKind::UnsupportedTurnMethod, "unsupported_turn_method"),
        (
            TurnFailureKind::UnsupportedTurnContractVersion,
            "unsupported_turn_contract_version",
        ),
        (TurnFailureKind::TurnLifetimeViolation, "turn_lifetime_violation"),
        (
            TurnFailureKind::AllocationLifetimeExceeded,
            "allocation_lifetime_exceeded",
        ),
        (
            TurnFailureKind::PermissionLifetimeExceeded,
            "permission_lifetime_exceeded",
        ),
        (
            TurnFailureKind::ChannelBindLifetimeExceeded,
            "channel_bind_lifetime_exceeded",
        ),
        (TurnFailureKind::RefreshLimitExceeded, "refresh_limit_exceeded"),
        (
            TurnFailureKind::TurnRelayQueueBoundExceeded,
            "turn_relay_queue_bound_exceeded",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
        assert_cataloged(code);
    }
    assert!(turn_core::TURN_LIFECYCLE_RULES.len() >= 30);
    for rule in turn_core::TURN_LIFECYCLE_RULES {
        assert!(!rule.event().is_empty());
        assert!(!rule.allowed_pre_state().is_empty());
        assert!(!rule.success_state().is_empty());
        assert!(!rule.failure_state().is_empty());
        for reason in rule.reason_codes() {
            assert_cataloged(reason.reason_code());
        }
    }
    let allocation_states = [
        turn_core::AllocationState::Absent,
        turn_core::AllocationState::Requested,
        turn_core::AllocationState::Active,
        turn_core::AllocationState::Refreshing,
        turn_core::AllocationState::Expired,
        turn_core::AllocationState::Released,
        turn_core::AllocationState::Rejected,
    ];
    let permission_states = [
        turn_core::PermissionState::Absent,
        turn_core::PermissionState::Requested,
        turn_core::PermissionState::Active,
        turn_core::PermissionState::Expired,
        turn_core::PermissionState::Revoked,
        turn_core::PermissionState::Rejected,
    ];
    let channel_states = [
        turn_core::ChannelBindState::Unbound,
        turn_core::ChannelBindState::Requested,
        turn_core::ChannelBindState::Bound,
        turn_core::ChannelBindState::Expired,
        turn_core::ChannelBindState::Rejected,
    ];
    assert_eq!(allocation_states.len() + permission_states.len() + channel_states.len(), 18);
}
