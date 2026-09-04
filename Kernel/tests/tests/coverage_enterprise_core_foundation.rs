use arcrtc_core_audit as audit;
use arcrtc_core_command as command;
use arcrtc_core_configuration as configuration;
use arcrtc_core_cross_plane as cross_plane;
use arcrtc_core_domain as domain;
use arcrtc_core_features as features;
use arcrtc_core_identity as identity;
use arcrtc_core_protocol as protocol;
use arcrtc_core_reason::{find_reason_definition, CatalogedReasonRef, Reason};
use arcrtc_core_security as security;
use arcrtc_core_state as state;
use arcrtc_core_time as time;
use arcrtc_core_transport as transport;

fn reference(value: &str) -> identity::OpaqueReference {
    identity::OpaqueReference::accept(value, identity::ReferenceAuthority::CorePolicy)
        .expect("reference must be valid")
}

fn correlation(value: &str) -> identity::CorrelationId {
    identity::CorrelationId::new(reference(value))
}

fn cataloged(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason code must be cataloged")
}

fn assert_reason_is_cataloged(code: &str) {
    assert_eq!(cataloged(code).definition().code().as_str(), code);
}

#[test]
fn coverage_core_foundation_configuration_domain_features_and_identity_are_closed() {
    assert_eq!(
        format!("{:?}", configuration::CoreConfigurationSurface),
        "CoreConfigurationSurface"
    );
    let policy = configuration::CorePolicyConfiguration::new("bounded-room-policy");
    assert_eq!(*policy.policy(), "bounded-room-policy");

    for (kind, owner) in [
        (
            configuration::ConfigurationKind::CorePolicy,
            configuration::ConfigurationOwner::Core,
        ),
        (
            configuration::ConfigurationKind::DriverRuntime,
            configuration::ConfigurationOwner::Drivers,
        ),
        (
            configuration::ConfigurationKind::EntrypointComposition,
            configuration::ConfigurationOwner::Entrypoints,
        ),
        (
            configuration::ConfigurationKind::SdkClient,
            configuration::ConfigurationOwner::Sdk,
        ),
        (
            configuration::ConfigurationKind::Regulated,
            configuration::ConfigurationOwner::Regulated,
        ),
    ] {
        let boundary = configuration::NonCoreConfigurationBoundary::new(
            kind,
            owner,
            configuration::ConfigurationSourceClass::TypedInput,
        );
        assert_eq!(boundary.kind(), kind);
        assert_eq!(boundary.owner(), owner);
    }
    let _sources = [
        configuration::ConfigurationSourceClass::TypedInput,
        configuration::ConfigurationSourceClass::EnvironmentVariable,
        configuration::ConfigurationSourceClass::File,
        configuration::ConfigurationSourceClass::ProcessArgs,
        configuration::ConfigurationSourceClass::OsSettings,
        configuration::ConfigurationSourceClass::CloudMetadata,
    ];
    let _allowed_effects = [
        configuration::FeatureFlagAllowedEffect::SelectDriverImplementation,
        configuration::FeatureFlagAllowedEffect::EnableOptionalExporter,
        configuration::FeatureFlagAllowedEffect::ChooseExternalEncoding,
    ];
    let _prohibited_effects = [
        configuration::FeatureFlagProhibitedEffect::SignalingStateMachineChange,
        configuration::FeatureFlagProhibitedEffect::TurnLifecycleChange,
        configuration::FeatureFlagProhibitedEffect::SfuRoutingSemanticsChange,
        configuration::FeatureFlagProhibitedEffect::SecurityVerificationBypass,
        configuration::FeatureFlagProhibitedEffect::AuditRequirementBypass,
    ];
    for kind in [
        configuration::ConfigurationFailureKind::CorePolicyConfigInvalid,
        configuration::ConfigurationFailureKind::RuntimeConfigMissing,
        configuration::ConfigurationFailureKind::RuntimeConfigInvalid,
        configuration::ConfigurationFailureKind::SecretUnavailable,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    let _secret_boundary = [
        configuration::SecretBoundary::RawSecretOutsideCore,
        configuration::SecretBoundary::OpaqueCredentialReferenceOnly,
    ];

    assert_eq!(
        format!("{:?}", domain::CoreDomainSurface),
        "CoreDomainSurface"
    );
    let boundary = domain::UseCaseBoundary::<(), ()>::new(
        "join-room",
        domain::ENTRYPOINTLICATION_USE_CASE_ORDER,
    );
    assert_eq!(boundary.name(), "join-room");
    assert_eq!(boundary.steps().len(), 5);
    let _families = [
        domain::AggregateFamily::SignalingRoomParticipant,
        domain::AggregateFamily::SfuSessionEndpointRoute,
        domain::AggregateFamily::TurnAllocationPermissionChannelBind,
        domain::AggregateFamily::CrossPlaneBindingScope,
        domain::AggregateFamily::AuditChainScope,
        domain::AggregateFamily::ConfigurationScope,
    ];

    assert_eq!(
        format!("{:?}", features::CoreFeaturesSurface),
        "CoreFeaturesSurface"
    );
    let feature_classes = [
        features::ExcludedFeatureClass::ChatApplicationSemantics,
        features::ExcludedFeatureClass::RecordingWorkflow,
        features::ExcludedFeatureClass::ScreenShareWorkflow,
        features::ExcludedFeatureClass::DataChannelApplicationSemantics,
        features::ExcludedFeatureClass::UiEndUserWorkflow,
        features::ExcludedFeatureClass::MediaCaptureWorkflow,
        features::ExcludedFeatureClass::RegulatedDomainWorkflow,
    ];
    let requested_surfaces = [
        features::FeatureRequestedSurface::Core,
        features::FeatureRequestedSurface::Signaling,
        features::FeatureRequestedSurface::Sfu,
        features::FeatureRequestedSurface::Turn,
        features::FeatureRequestedSurface::Sdk,
        features::FeatureRequestedSurface::Driver,
        features::FeatureRequestedSurface::Entrypoints,
        features::FeatureRequestedSurface::Regulated,
    ];
    assert_eq!(feature_classes.len(), 7);
    assert_eq!(requested_surfaces.len(), 8);
    let decision = features::FeatureAdmissionDecision::new(
        Some(correlation("feature-correlation")),
        features::ExcludedFeatureClass::ChatApplicationSemantics,
        features::FeatureRequestedSurface::Core,
        features::FeatureAdmissionDecisionClass::Rejected,
        Some(features::FeatureAdmissionFailureKind::ChatNotSupported),
    );
    assert_eq!(
        decision.feature_class(),
        features::ExcludedFeatureClass::ChatApplicationSemantics
    );
    assert_eq!(
        decision.requested_surface(),
        features::FeatureRequestedSurface::Core
    );
    assert_eq!(
        decision.decision_class(),
        features::FeatureAdmissionDecisionClass::Rejected
    );
    let _admission_classes = [
        features::FeatureAdmissionDecisionClass::Rejected,
        features::FeatureAdmissionDecisionClass::SupportedBySourceContract,
    ];
    for kind in [
        features::FeatureAdmissionFailureKind::FeatureOutOfScope,
        features::FeatureAdmissionFailureKind::FeatureNotSupported,
        features::FeatureAdmissionFailureKind::ChatNotSupported,
        features::FeatureAdmissionFailureKind::RecordingNotSupported,
        features::FeatureAdmissionFailureKind::ScreenShareNotSupported,
        features::FeatureAdmissionFailureKind::DataChannelNotSupported,
        features::FeatureAdmissionFailureKind::UiWorkflowNotSupported,
        features::FeatureAdmissionFailureKind::MediaCaptureNotSupported,
        features::FeatureAdmissionFailureKind::RegulatedWorkflowNotSupported,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    assert_eq!(
        format!("{:?}", identity::CoreIdentitySurface),
        "CoreIdentitySurface"
    );
    for kind in [
        identity::IdentityKind::CorrelationId,
        identity::IdentityKind::RoomId,
        identity::IdentityKind::SessionId,
        identity::IdentityKind::ParticipantId,
        identity::IdentityKind::EndpointId,
        identity::IdentityKind::StreamId,
        identity::IdentityKind::PacketId,
        identity::IdentityKind::RouteId,
        identity::IdentityKind::AllocationId,
        identity::IdentityKind::PermissionId,
        identity::IdentityKind::ChannelBindId,
        identity::IdentityKind::AuditEventId,
        identity::IdentityKind::CredentialRef,
        identity::IdentityKind::StartupRunId,
        identity::IdentityKind::ConfigurationScopeRef,
    ] {
        assert!(!format!("{:?}", kind.lifetime()).is_empty());
        assert!(!format!("{:?}", kind.external_exposure()).is_empty());
    }
    let untrusted = identity::UntrustedReference::new("external-room");
    assert_eq!(untrusted.as_str(), "external-room");
    let accepted = identity::OpaqueReference::accept_untrusted(
        untrusted,
        identity::ReferenceAuthority::CoreValidatedUntrustedInput,
    )
    .expect("validated untrusted reference is accepted");
    assert_eq!(accepted.as_str(), "external-room");
    assert_eq!(
        accepted.authority(),
        identity::ReferenceAuthority::CoreValidatedUntrustedInput
    );
    assert_eq!(
        identity::OpaqueReference::accept("", identity::ReferenceAuthority::CorePolicy),
        Err(identity::OpaqueReferenceError::Empty)
    );
    assert_eq!(
        identity::OpaqueReference::accept("bad\nref", identity::ReferenceAuthority::CorePolicy),
        Err(identity::OpaqueReferenceError::ControlCharacter)
    );
    for source in [
        identity::ExternalIdentitySource::ApplicationUser,
        identity::ExternalIdentitySource::TokenSubject,
        identity::ExternalIdentitySource::RegulatedSubject,
        identity::ExternalIdentitySource::TenantRole,
        identity::ExternalIdentitySource::MedicalRole,
        identity::ExternalIdentitySource::FacilityIdentity,
    ] {
        assert!(!source.issues_core_identity());
    }
    assert_eq!(identity::RoomId::kind(), identity::IdentityKind::RoomId);
    assert_eq!(
        identity::RoomId::new(reference("room-foundation")).as_str(),
        "room-foundation"
    );
    assert_eq!(
        identity::CredentialRef::new(reference("credential-foundation")).authority(),
        identity::ReferenceAuthority::CorePolicy
    );
}

#[test]
fn coverage_core_foundation_command_and_audit_shapes_are_exercised() {
    assert_eq!(
        format!("{:?}", command::CoreCommandSurface),
        "CoreCommandSurface"
    );
    let command_type = command::CommandType::new("foundation-command");
    assert_eq!(command_type.as_str(), "foundation-command");
    let version = command::CommandVersion::new(2);
    assert_eq!(version.value(), 2);
    for surface in [
        command::TargetSurface::Signaling,
        command::TargetSurface::Sfu,
        command::TargetSurface::Turn,
        command::TargetSurface::Transport,
        command::TargetSurface::Security,
        command::TargetSurface::Audit,
        command::TargetSurface::Quality,
        command::TargetSurface::Ports,
        command::TargetSurface::Configuration,
        command::TargetSurface::Features,
    ] {
        let envelope = command::CommandEnvelope::new(
            correlation("command-envelope"),
            command_type,
            version,
            surface,
            "subject-ref",
        );
        assert_eq!(envelope.command_type().as_str(), "foundation-command");
        assert_eq!(envelope.version().value(), 2);
        assert_eq!(envelope.target_surface(), surface);
        assert_eq!(*envelope.subject_references(), "subject-ref");
    }
    let success_outcomes = [
        command::UseCaseOutcome::Accepted,
        command::UseCaseOutcome::Allowed,
        command::UseCaseOutcome::Forwarded,
        command::UseCaseOutcome::Selected,
        command::UseCaseOutcome::Released,
        command::UseCaseOutcome::ClosedSuccess,
        command::UseCaseOutcome::IdempotentObserved,
        command::UseCaseOutcome::WithinBoundObserved,
    ];
    for outcome in success_outcomes {
        assert!(outcome.is_success());
        assert!(!outcome.requires_reason());
    }
    for outcome in [
        command::UseCaseOutcome::Rejected,
        command::UseCaseOutcome::Denied,
        command::UseCaseOutcome::ProtocolViolation,
        command::UseCaseOutcome::Suppressed,
        command::UseCaseOutcome::Dropped,
        command::UseCaseOutcome::Expired,
        command::UseCaseOutcome::Revoked,
        command::UseCaseOutcome::Shed,
        command::UseCaseOutcome::Delayed,
        command::UseCaseOutcome::Degraded,
        command::UseCaseOutcome::Failed,
        command::UseCaseOutcome::ConvertedFailure,
        command::UseCaseOutcome::ClosedByPolicy,
    ] {
        assert!(!outcome.is_success());
        assert!(outcome.requires_reason());
    }
    let intent = command::PortIntent::new("persist-audit", command::TargetSurface::Audit);
    assert_eq!(intent.intent_type(), "persist-audit");
    assert_eq!(intent.target_surface(), command::TargetSurface::Audit);
    let accepted = command::UseCaseDecision::new(command::UseCaseDecisionInput {
        correlation_id: correlation("command-decision-ok"),
        command_type,
        target_surface: command::TargetSurface::Signaling,
        outcome: command::UseCaseOutcome::Accepted,
        reason: command::DecisionReason::Absent::<CatalogedReasonRef>,
        state_transition: command::StateTransitionSummary::Changed("room joined"),
        port_intents: vec![intent],
        audit_projection: command::AuditProjectionRequirement::Required,
    })
    .expect("accepted decision must not carry reason");
    assert_eq!(accepted.outcome(), command::UseCaseOutcome::Accepted);
    assert_eq!(
        accepted.state_transition(),
        command::StateTransitionSummary::Changed("room joined")
    );
    assert_eq!(accepted.port_intents().len(), 1);
    assert_eq!(
        command::UseCaseDecision::new(command::UseCaseDecisionInput {
            correlation_id: correlation("command-decision-bad"),
            command_type,
            target_surface: command::TargetSurface::Signaling,
            outcome: command::UseCaseOutcome::Accepted,
            reason: command::DecisionReason::Cataloged(cataloged("room_closed")),
            state_transition: command::StateTransitionSummary::NoStateChange,
            port_intents: vec![],
            audit_projection: command::AuditProjectionRequirement::NotRequired,
        }),
        Err(command::DecisionShapeError::SuccessMustNotCarryReason)
    );
    assert_eq!(
        command::DriverObservation::<CatalogedReasonRef>::new(
            correlation("driver-observation"),
            command::UseCaseOutcome::Failed,
            command::DecisionReason::Absent,
        ),
        Err(command::DecisionShapeError::NonSuccessRequiresReason)
    );
    let observation = command::DriverObservation::new(
        correlation("driver-observation-ok"),
        command::UseCaseOutcome::ConvertedFailure,
        command::DecisionReason::Cataloged(cataloged("driver_shutdown")),
    )
    .expect("failure carries cataloged reason");
    assert!(!format!("{:?}", observation).is_empty());
    let event = command::DomainEvent::new(correlation("domain-event"), "room_joined", 7_u8);
    assert_eq!(event.event_type(), "room_joined");
    assert_eq!(*event.payload(), 7);
    let _projection = command::AuditProjection::new(
        correlation("audit-projection"),
        "signaling_join_decision",
        command::UseCaseOutcome::Accepted,
        command::DecisionReason::Absent::<CatalogedReasonRef>,
    );
    let _response = command::ExternalResponseModel::new(
        correlation("external-response"),
        command::UseCaseOutcome::Rejected,
        command::DecisionReason::Cataloged(cataloged("room_closed")),
    );

    let trace = command::CorrelationTrace::new(correlation("trace"));
    assert_eq!(trace.correlation_id().as_str(), "trace");
    let scope = command::IdempotencyScope::new("room:1").expect("scope");
    let key = command::IdempotencyKey::new("join:1").expect("key");
    assert_eq!(scope.as_str(), "room:1");
    assert_eq!(key.as_str(), "join:1");
    assert_eq!(
        command::IdempotencyScope::new(""),
        Err(command::IdempotencyValueError::Empty)
    );
    assert_eq!(
        command::IdempotencyKey::new("bad\nkey"),
        Err(command::IdempotencyValueError::ControlCharacter)
    );
    let digest = command::SemanticPayloadDigest::new("sha256", vec![1, 2, 3]).expect("digest");
    assert_eq!(digest.algorithm(), "sha256");
    assert_eq!(digest.digest(), &[1, 2, 3]);
    assert_eq!(
        command::SemanticPayloadDigest::new("", vec![1]),
        Err(command::IdempotencyValueError::Empty)
    );
    assert_eq!(
        command::SemanticPayloadDigest::new("sha256", vec![]),
        Err(command::IdempotencyValueError::Empty)
    );
    let identity = command::CommandIdentity::new(
        command_type,
        scope,
        identity::RoomId::new(reference("idempotency-room")),
        Some(identity::ParticipantId::new(reference(
            "idempotency-participant",
        ))),
        key,
        Some(digest),
        command::ReplayWindow::Bounded("30s"),
    );
    assert_eq!(identity.command_type().as_str(), "foundation-command");
    assert!(identity.actor_reference().is_some());
    assert!(identity.payload_digest().is_some());
    let prior = command::PriorDecisionReference::new(
        correlation("prior-correlation"),
        Some(identity::AuditEventId::new(reference("prior-audit"))),
    );
    assert_eq!(prior.prior_correlation_id().as_str(), "prior-correlation");
    assert!(prior.prior_audit_event_id().is_some());
    let idempotency = command::IdempotencyDecision::new(
        command::IdempotencyClass::ResponseReplayCandidate,
        command::UseCaseOutcome::IdempotentObserved,
        command::DecisionReason::Absent::<CatalogedReasonRef>,
        Some(prior),
    )
    .expect("idempotent observed is success");
    assert_eq!(
        idempotency.class(),
        command::IdempotencyClass::ResponseReplayCandidate
    );
    assert!(idempotency.prior_decision().is_some());
    for kind in [
        command::IdempotencyFailureKind::MissingCorrelationId,
        command::IdempotencyFailureKind::CorrelationMismatch,
        command::IdempotencyFailureKind::DuplicateCommand,
        command::IdempotencyFailureKind::IdempotencyPayloadMismatch,
        command::IdempotencyFailureKind::IdempotencyWindowExpired,
        command::IdempotencyFailureKind::ReplayNotAllowed,
        command::IdempotencyFailureKind::ResponseReplayNotAvailable,
        command::IdempotencyFailureKind::CanonicalSerializationFailed,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    assert_eq!(format!("{:?}", audit::CoreAuditSurface), "CoreAuditSurface");
    for definition in audit::AUDIT_EVENT_DEFINITIONS {
        assert_eq!(
            audit::find_audit_event_definition(definition.event_type().as_str())
                .expect("event type is cataloged")
                .event_type()
                .as_str(),
            definition.event_type().as_str()
        );
    }
    let mut references = audit::AuditReferences::new();
    references.push(audit::AuditReferencePresence::Present {
        kind: audit::AuditReferenceKind::Correlation,
        value: audit::AuditReferenceValue::CorrelationId(correlation("audit-correlation")),
    });
    references.push(audit::AuditReferencePresence::Present {
        kind: audit::AuditReferenceKind::Room,
        value: audit::AuditReferenceValue::RoomId(identity::RoomId::new(reference("audit-room"))),
    });
    references.push(audit::AuditReferencePresence::AbsentNotApplicable(
        audit::AuditReferenceKind::SecureMediaSession,
    ));
    assert_eq!(references.entries().len(), 3);
    let owner =
        audit::ResourceOwnerTuple::new(audit::AuditComponent::Core, audit::AuditComponent::Driver);
    assert_eq!(owner.resource_policy_owner(), audit::AuditComponent::Core);
    assert_eq!(
        owner.physical_resource_owner(),
        audit::AuditComponent::Driver
    );
    let tag = audit::NonSensitiveTag::new("plane", "signaling");
    assert_eq!(tag.key(), "plane");
    assert_eq!(tag.value(), "signaling");
    assert_eq!(
        audit::AuditTimestamp::new(""),
        Err(audit::AuditEventShapeError::EmptyTimestamp)
    );
    let timestamp = audit::AuditTimestamp::new("2026-06-16T00:00:00Z").expect("timestamp");
    assert_eq!(timestamp.as_str(), "2026-06-16T00:00:00Z");
    let reason = Reason::new(
        find_reason_definition("room_closed").expect("reason definition"),
        Some("non-authoritative detail"),
    );
    let event = audit::AuditEvent::new(audit::AuditEventInput {
        event_id: identity::AuditEventId::new(reference("audit-event")),
        correlation: audit::AuditReferencePresence::Present {
            kind: audit::AuditReferenceKind::Correlation,
            value: audit::AuditReferenceValue::CorrelationId(correlation(
                "audit-event-correlation",
            )),
        },
        timestamp,
        component: audit::AuditComponent::Core,
        event_type: audit::find_audit_event_definition("signaling_join_decision")
            .expect("event type"),
        outcome: command::UseCaseOutcome::Rejected,
        reason: audit::AuditReason::Cataloged(reason),
        references,
        resource_owner: Some(owner),
        tags: vec![tag],
    })
    .expect("rejected audit event carries reason");
    assert_eq!(
        event.event_type().event_type().as_str(),
        "signaling_join_decision"
    );
    assert!(matches!(event.reason(), audit::AuditReason::Cataloged(_)));
    assert_eq!(
        audit::AuditEvent::new(audit::AuditEventInput::<CatalogedReasonRef> {
            event_id: identity::AuditEventId::new(reference("audit-event-bad")),
            correlation: audit::AuditReferencePresence::AbsentNotApplicable(
                audit::AuditReferenceKind::Correlation,
            ),
            timestamp: audit::AuditTimestamp::new("2026-06-16T00:00:01Z").expect("timestamp"),
            component: audit::AuditComponent::Core,
            event_type: audit::find_audit_event_definition("signaling_join_decision")
                .expect("event type"),
            outcome: command::UseCaseOutcome::Accepted,
            reason: audit::AuditReason::Cataloged(Reason::new(
                find_reason_definition("room_closed").expect("reason definition"),
                Some(cataloged("room_closed")),
            )),
            references: audit::AuditReferences::new(),
            resource_owner: None,
            tags: vec![],
        }),
        Err(audit::AuditEventShapeError::SuccessMustNotCarryReason)
    );
    assert_eq!(
        audit::AuditEvent::new(audit::AuditEventInput::<CatalogedReasonRef> {
            event_id: identity::AuditEventId::new(reference("audit-event-bad2")),
            correlation: audit::AuditReferencePresence::AbsentNotApplicable(
                audit::AuditReferenceKind::Correlation,
            ),
            timestamp: audit::AuditTimestamp::new("2026-06-16T00:00:02Z").expect("timestamp"),
            component: audit::AuditComponent::Core,
            event_type: audit::find_audit_event_definition("signaling_join_decision")
                .expect("event type"),
            outcome: command::UseCaseOutcome::Rejected,
            reason: audit::AuditReason::None,
            references: audit::AuditReferences::new(),
            resource_owner: None,
            tags: vec![],
        }),
        Err(audit::AuditEventShapeError::NonSuccessRequiresReason)
    );

    let sequence = audit::HashChainSequence::new(1);
    assert_eq!(sequence.value(), 1);
    assert_eq!(audit::HashAlgorithm::Sha256.as_str(), "sha256");
    let canonical_format = audit::CanonicalRecordFormat::new("json-c14n", "1");
    assert_eq!(canonical_format.format(), "json-c14n");
    assert_eq!(canonical_format.version(), "1");
    assert_eq!(
        audit::CanonicalEventPayloadDigest::new(audit::HashAlgorithm::Sha256, vec![]),
        Err(audit::HashChainRecordError::EmptyDigest)
    );
    let payload_digest =
        audit::CanonicalEventPayloadDigest::new(audit::HashAlgorithm::Sha256, vec![1])
            .expect("payload digest");
    let record_hash =
        audit::RecordHash::new(audit::HashAlgorithm::Sha256, vec![2]).expect("record hash");
    let record = audit::HashChainRecord::new(audit::HashChainRecordInput {
        scope: audit::HashChainScope::Signaling,
        sequence,
        previous_hash: audit::PreviousRecordHash::Genesis,
        event_type: event.event_type().event_type(),
        outcome: command::UseCaseOutcome::Rejected,
        canonical_format,
        payload_digest,
        record_hash,
    });
    assert_eq!(record.scope(), audit::HashChainScope::Signaling);
    assert_eq!(record.sequence().value(), 1);
    assert_eq!(record.event_type().as_str(), "signaling_join_decision");
    assert_eq!(record.outcome(), command::UseCaseOutcome::Rejected);
    let _hash_scopes = [
        audit::HashChainScope::Startup,
        audit::HashChainScope::Signaling,
        audit::HashChainScope::Sfu,
        audit::HashChainScope::Turn,
        audit::HashChainScope::Driver,
    ];
    let _verification_failures = [
        audit::HashChainVerificationFailure::ChainGap,
        audit::HashChainVerificationFailure::SequenceMismatch,
        audit::HashChainVerificationFailure::PreviousHashMismatch,
        audit::HashChainVerificationFailure::UnknownAlgorithm,
        audit::HashChainVerificationFailure::CanonicalSerializationFailed,
        audit::HashChainVerificationFailure::CanonicalizationMismatch,
    ];
}

#[test]
fn coverage_core_foundation_security_time_transport_and_cross_plane_are_exercised() {
    assert_eq!(
        format!("{:?}", security::CoreSecuritySurface),
        "CoreSecuritySurface"
    );
    let mapping = security::AuthorizationMapping::new(
        security::AuthorizationMappingSource::VerifiedCredential,
        security::AuthorizationContextClass::VerifiedCredentialContext,
        vec![
            command::TargetSurface::Signaling,
            command::TargetSurface::Turn,
        ],
        security::AuthorizationLifetime::UntilCredentialExpiry,
        security::AuthorizationRedactionRule::RawClaimsExcluded,
    );
    assert_eq!(
        mapping.context_class(),
        security::AuthorizationContextClass::VerifiedCredentialContext
    );
    assert_eq!(
        mapping.lifetime(),
        security::AuthorizationLifetime::UntilCredentialExpiry
    );
    assert_eq!(mapping.allowed_target_surfaces().len(), 2);
    let input = security::AuthorizationPolicyInput::new(
        correlation("auth-input"),
        Some(identity::CredentialRef::new(reference("auth-credential"))),
        security::AuthorizationContextClass::ParticipantJoinContext,
        command::TargetSurface::Signaling,
        security::CommunicationAction::Join,
        identity::RoomId::new(reference("auth-room")),
    );
    assert_eq!(input.target_surface(), command::TargetSurface::Signaling);
    assert_eq!(input.action(), security::CommunicationAction::Join);
    let accepted = security::AuthorizationPolicyDecision::<CatalogedReasonRef>::new(
        command::UseCaseOutcome::Allowed,
        command::DecisionReason::Absent,
    )
    .expect("success carries no reason");
    assert_eq!(accepted.outcome(), command::UseCaseOutcome::Allowed);
    assert_eq!(
        security::AuthorizationPolicyDecision::new(
            command::UseCaseOutcome::Allowed,
            command::DecisionReason::Cataloged(cataloged("authorization_policy_denied")),
        ),
        Err(security::AuthorizationPolicyError::SuccessMustNotCarryReason)
    );
    assert_eq!(
        security::AuthorizationPolicyDecision::<CatalogedReasonRef>::new(
            command::UseCaseOutcome::Denied,
            command::DecisionReason::Absent,
        ),
        Err(security::AuthorizationPolicyError::NonSuccessRequiresReason)
    );
    for kind in [
        security::AuthorizationFailureKind::AuthorizationContextMissing,
        security::AuthorizationFailureKind::AuthorizationContextInvalid,
        security::AuthorizationFailureKind::AuthorizationContextExpired,
        security::AuthorizationFailureKind::AuthorizationPolicyDenied,
        security::AuthorizationFailureKind::AuthorizationScopeNotAllowed,
        security::AuthorizationFailureKind::ForwardedHeaderUntrusted,
        security::AuthorizationFailureKind::ClientAddressUntrusted,
        security::AuthorizationFailureKind::TokenVerificationFailed,
        security::AuthorizationFailureKind::RuntimeConfigMissing,
        security::AuthorizationFailureKind::RuntimeConfigInvalid,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    let request = security::TokenVerificationRequest::new(
        correlation("token-request"),
        identity::CredentialRef::new(reference("token-credential")),
        vec![
            security::RequiredTokenClaim::Issuer,
            security::RequiredTokenClaim::Audience,
            security::RequiredTokenClaim::Subject,
            security::RequiredTokenClaim::Expiration,
            security::RequiredTokenClaim::NotBefore,
            security::RequiredTokenClaim::IssuedAt,
            security::RequiredTokenClaim::KeyId,
        ],
        security::IssuerPolicy::new(vec!["issuer-a"]),
        security::AudiencePolicy::new(vec!["audience-a"]),
        security::TokenAlgorithmPolicy::new(vec!["EdDSA"]),
    );
    assert_eq!(request.required_claims().len(), 7);
    assert_eq!(request.credential_ref().as_str(), "token-credential");
    let verified = security::VerifiedCredential::new(
        correlation("verified-token"),
        identity::CredentialRef::new(reference("verified-credential")),
        security::TokenTemporalDecision::Valid,
    );
    assert_eq!(verified.credential_ref().as_str(), "verified-credential");
    assert_eq!(
        verified.temporal_decision(),
        security::TokenTemporalDecision::Valid
    );
    let _results = [
        security::TokenVerificationResult::Accepted(verified),
        security::TokenVerificationResult::Rejected(
            security::TokenVerificationFailureKind::TokenMissing,
        ),
    ];
    for kind in [
        security::TokenVerificationFailureKind::TokenMissing,
        security::TokenVerificationFailureKind::TokenMalformed,
        security::TokenVerificationFailureKind::TokenSignatureInvalid,
        security::TokenVerificationFailureKind::TokenKeyUnavailable,
        security::TokenVerificationFailureKind::TokenIssuerMismatch,
        security::TokenVerificationFailureKind::TokenAudienceMismatch,
        security::TokenVerificationFailureKind::TokenExpired,
        security::TokenVerificationFailureKind::TokenNotYetValid,
        security::TokenVerificationFailureKind::TokenRequiredClaimMissing,
        security::TokenVerificationFailureKind::TokenUnsupportedAlgorithm,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    assert_eq!(format!("{:?}", time::CoreTimeSurface), "CoreTimeSurface");
    for quantity in [
        time::NormalizedQuantity::Duration,
        time::NormalizedQuantity::Timestamp,
        time::NormalizedQuantity::Bytes,
        time::NormalizedQuantity::PacketCount,
        time::NormalizedQuantity::Rate,
        time::NormalizedQuantity::Ratio,
        time::NormalizedQuantity::Bitrate,
        time::NormalizedQuantity::JitterRtt,
    ] {
        assert!(!format!("{:?}", quantity.default_unit()).is_empty());
    }
    assert_eq!(
        time::NormalizedValue::rational(1, 0),
        Err(time::NormalizedValueError::ZeroDenominator)
    );
    let _measurement = time::NormalizedMeasurement::new(
        time::NormalizedQuantity::Duration,
        time::NormalizedUnit::MillisecondsInteger,
        time::NormalizedValue::Integer(100),
        time::PrecisionClass::IntegerExact,
        time::RoundingDirection::Exact,
        time::ComparisonOperator::LessThanOrEqual,
        time::BoundaryInclusivity::Inclusive,
        time::SamplingWindow::Milliseconds(1000),
        time::RawMeasurementOwner::Driver,
        time::PolicyDecisionOwner::Core,
    );
    for kind in [
        time::TimeNormalizationFailureKind::MeasurementNormalizationFailed,
        time::TimeNormalizationFailureKind::TimeObservationUnavailable,
        time::TimeNormalizationFailureKind::OperationDeadlineExceeded,
        time::TimeNormalizationFailureKind::RetentionDurationExceeded,
        time::TimeNormalizationFailureKind::RuntimeConfigInvalid,
        time::TimeNormalizationFailureKind::DriverShutdown,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    for concern in [
        time::TimeSynchronizationConcern::LocalMonotonicDuration,
        time::TimeSynchronizationConcern::WallClockTimestamp,
        time::TimeSynchronizationConcern::CrossNodeSkewPolicy,
        time::TimeSynchronizationConcern::ExternalTimeSource,
        time::TimeSynchronizationConcern::TimestampNormalization,
    ] {
        assert!(!format!("{:?}", concern.owner()).is_empty());
    }
    for trust_class in [
        time::TimeTrustClass::SingleProcessMonotonic,
        time::TimeTrustClass::SingleNodeWallClock,
        time::TimeTrustClass::MultiNodeBoundedSkew,
        time::TimeTrustClass::ExternalTrustedTimeSource,
        time::TimeTrustClass::TestDeterministicClock,
        time::TimeTrustClass::TimeUntrusted,
    ] {
        assert_eq!(
            trust_class.supports_runtime_trust_decision(),
            matches!(
                trust_class,
                time::TimeTrustClass::SingleProcessMonotonic
                    | time::TimeTrustClass::SingleNodeWallClock
                    | time::TimeTrustClass::MultiNodeBoundedSkew
                    | time::TimeTrustClass::ExternalTrustedTimeSource
            )
        );
    }
    assert_eq!(
        time::ClockSkewPolicy::try_new(
            time::TimeNodeScope::MultiNode,
            time::TimeTrustClass::MultiNodeBoundedSkew,
            None,
            time::PrecisionClass::IntegerExact,
            time::SamplingWindow::Milliseconds(1000),
            time::TrustedTimeSourceClass::ExternalTimeSourceObservation,
            time::ClockSkewImpact::Ordering,
        ),
        Err(time::ClockSkewPolicyError::MaxSkewRequired)
    );
    let skew_policy = time::ClockSkewPolicy::try_new(
        time::TimeNodeScope::MultiNode,
        time::TimeTrustClass::MultiNodeBoundedSkew,
        Some(250),
        time::PrecisionClass::DeclaredPrecisionLabel("ntp"),
        time::SamplingWindow::PolicyLabel("skew-window"),
        time::TrustedTimeSourceClass::ExternalTimeSourceObservation,
        time::ClockSkewImpact::Verification,
    )
    .expect("bounded skew policy has max skew");
    for kind in [
        time::TimeSynchronizationFailureKind::ClockSkewExceeded,
        time::TimeSynchronizationFailureKind::TimeSourceUntrusted,
        time::TimeSynchronizationFailureKind::TimeSyncUnavailable,
        time::TimeSynchronizationFailureKind::TimestampOrderUntrusted,
        time::TimeSynchronizationFailureKind::TimeObservationUnavailable,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    let time_decision = time::TimeSynchronizationDecision::try_new(
        identity::StartupRunId::new(reference("startup-time-foundation")),
        Some(correlation("time-sync")),
        skew_policy,
        time::ObservedSkewClass::WithinPolicy,
        time::TimeSynchronizationOutcome::Accepted,
        None,
    )
    .expect("accepted time synchronization has no reason");
    assert_eq!(
        time_decision.audit_event_type(),
        "time_synchronization_decision"
    );
    assert_eq!(
        time::TimeSynchronizationDecision::try_new(
            identity::StartupRunId::new(reference("startup-time-bad")),
            None,
            skew_policy,
            time::ObservedSkewClass::ExceedsPolicy,
            time::TimeSynchronizationOutcome::Rejected,
            None,
        ),
        Err(time::TimeSynchronizationDecisionError::ReasonRequired)
    );

    assert_eq!(
        format!("{:?}", transport::CoreTransportSurface),
        "CoreTransportSurface"
    );
    let session = identity::SessionId::new(reference("transport-session"));
    let sdp = transport::SessionDescriptionRef::new(session.clone(), "offer-ref").expect("sdp");
    assert_eq!(
        transport::SessionDescriptionRef::new(session.clone(), ""),
        Err(transport::TransportContractError::InvalidSemanticReference)
    );
    let ice = transport::IceCandidateRef::new(session.clone(), "ice-ref").expect("ice");
    assert_eq!(
        transport::IceCandidateRef::new(session.clone(), "bad\nice"),
        Err(transport::TransportContractError::InvalidSemanticReference)
    );
    let packet_view = transport::PacketSemanticViewRef::new(identity::PacketId::new(reference(
        "transport-packet",
    )));
    let _inputs = [
        transport::WebRtcTransportInput::new(
            transport::TransportCommandKind::StartSession,
            transport::WebRtcTransportPayload::Empty,
        ),
        transport::WebRtcTransportInput::new(
            transport::TransportCommandKind::ApplyLocalDescription,
            transport::WebRtcTransportPayload::SessionDescription(sdp.clone()),
        ),
        transport::WebRtcTransportInput::new(
            transport::TransportCommandKind::AddIceCandidate,
            transport::WebRtcTransportPayload::IceCandidate(ice.clone()),
        ),
        transport::WebRtcTransportInput::new(
            transport::TransportCommandKind::ForwardPacket,
            transport::WebRtcTransportPayload::PacketView(packet_view.clone()),
        ),
        transport::WebRtcTransportInput::new(
            transport::TransportCommandKind::CloseSession,
            transport::WebRtcTransportPayload::Capability(transport::TransportCapability::new(
                "dtls-srtp",
            )),
        ),
    ];
    for kind in [
        transport::TransportFailureKind::UnsupportedMediaContractVersion,
        transport::TransportFailureKind::ExternalDecodeFailed,
        transport::TransportFailureKind::ExternalEncodeFailed,
        transport::TransportFailureKind::MediaPayloadMappingInvalid,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    for kind in [
        transport::TransportDriverFailureKind::ExternalDecodeFailed,
        transport::TransportDriverFailureKind::ExternalEncodeFailed,
        transport::TransportDriverFailureKind::MediaPayloadMappingInvalid,
        transport::TransportDriverFailureKind::DriverShutdown,
    ] {
        let failure = transport::TransportDriverFailure::from_kind(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            kind.reason_code()
        );
    }
    let _observation = transport::WebRtcTransportOutput::new(
        transport::TransportEventKind::ConversionFailed,
        transport::WebRtcTransportObservation::ConversionFailure(
            transport::TransportDriverFailure::from_kind(
                transport::TransportDriverFailureKind::ExternalDecodeFailed,
            ),
        ),
    );
    for kind in [
        transport::NegotiationFailureKind::ExternalDecodeFailed,
        transport::NegotiationFailureKind::IceCandidatePolicyViolation,
        transport::NegotiationFailureKind::IceConnectivityCheckFailed,
        transport::NegotiationFailureKind::IceConsentExpired,
        transport::NegotiationFailureKind::MissingRequiredWireField,
        transport::NegotiationFailureKind::ExternalEnumUnmapped,
        transport::NegotiationFailureKind::UnsupportedCommandVersion,
        transport::NegotiationFailureKind::UnsupportedMediaContractVersion,
        transport::NegotiationFailureKind::MediaCodecNotSupported,
        transport::NegotiationFailureKind::MediaPayloadMappingInvalid,
        transport::NegotiationFailureKind::ParticipantNotJoined,
        transport::NegotiationFailureKind::CommandOrderViolation,
        transport::NegotiationFailureKind::RoomDraining,
        transport::NegotiationFailureKind::RoomClosed,
        transport::NegotiationFailureKind::NetworkSendFailed,
        transport::NegotiationFailureKind::NetworkReceiveFailed,
        transport::NegotiationFailureKind::DriverShutdown,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    let ice_policy = transport::IceCandidatePolicy::new(
        vec![
            transport::IceCandidateConnectivityClass::HostCandidateRef,
            transport::IceCandidateConnectivityClass::RelayCandidateRef,
            transport::IceCandidateConnectivityClass::IceRestartIntent,
        ],
        transport::IceAddressExposurePolicy::RelayOnly,
        true,
    );
    assert_eq!(ice_policy.accepted_classes().len(), 3);
    for kind in [
        transport::IceFailureKind::IceCandidatePolicyViolation,
        transport::IceFailureKind::IceCandidateMappingInvalid,
        transport::IceFailureKind::IceCandidateRedactionRequired,
        transport::IceFailureKind::IceGatheringFailed,
        transport::IceFailureKind::IceConnectivityCheckFailed,
        transport::IceFailureKind::IceConsentExpired,
        transport::IceFailureKind::IceRestartNotAllowed,
        transport::IceFailureKind::ExternalDecodeFailed,
        transport::IceFailureKind::CommandOrderViolation,
        transport::IceFailureKind::NetworkReceiveFailed,
        transport::IceFailureKind::NetworkSendFailed,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    let _secure_policy = transport::SecureMediaPolicy::new("v1", "dtls-srtp", true, true);
    for kind in [
        transport::SecureMediaFailureKind::SecureMediaProfileNotSupported,
        transport::SecureMediaFailureKind::SecureMediaHandshakeFailed,
        transport::SecureMediaFailureKind::SecureMediaPeerVerificationFailed,
        transport::SecureMediaFailureKind::SecureMediaProtectionNotActive,
        transport::SecureMediaFailureKind::SecureMediaKeyStateInvalid,
        transport::SecureMediaFailureKind::SecureMediaSessionExpired,
        transport::SecureMediaFailureKind::SecureMediaRekeyRequired,
        transport::SecureMediaFailureKind::SecretUnavailable,
        transport::SecureMediaFailureKind::SecretRotationStateUnavailable,
        transport::SecureMediaFailureKind::SecretKeyRevoked,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }

    assert_eq!(
        format!("{:?}", cross_plane::CoreCrossPlaneSurface),
        "CoreCrossPlaneSurface"
    );
    for class in [
        cross_plane::CrossPlaneBindingClass::NoCrossPlaneBindingRequired,
        cross_plane::CrossPlaneBindingClass::SignalingParticipantBinding,
        cross_plane::CrossPlaneBindingClass::SfuEndpointBinding,
        cross_plane::CrossPlaneBindingClass::TurnAllocationBinding,
        cross_plane::CrossPlaneBindingClass::TurnPermissionBinding,
        cross_plane::CrossPlaneBindingClass::IceCandidateBinding,
        cross_plane::CrossPlaneBindingClass::SecureMediaSessionBinding,
        cross_plane::CrossPlaneBindingClass::TestCrossPlaneBinding,
        cross_plane::CrossPlaneBindingClass::ImplicitBindingRequested,
    ] {
        assert_eq!(
            class.is_rejected_class(),
            matches!(
                class,
                cross_plane::CrossPlaneBindingClass::ImplicitBindingRequested
            )
        );
    }
    let refs = [
        cross_plane::CrossPlaneReference::NotMaterialized,
        cross_plane::CrossPlaneReference::Room(identity::RoomId::new(reference("cross-room"))),
        cross_plane::CrossPlaneReference::Session(identity::SessionId::new(reference(
            "cross-session",
        ))),
        cross_plane::CrossPlaneReference::Participant(identity::ParticipantId::new(reference(
            "cross-participant",
        ))),
        cross_plane::CrossPlaneReference::Endpoint(identity::EndpointId::new(reference(
            "cross-endpoint",
        ))),
        cross_plane::CrossPlaneReference::Stream(identity::StreamId::new(reference(
            "cross-stream",
        ))),
        cross_plane::CrossPlaneReference::Allocation(identity::AllocationId::new(reference(
            "cross-allocation",
        ))),
        cross_plane::CrossPlaneReference::Permission(identity::PermissionId::new(reference(
            "cross-permission",
        ))),
        cross_plane::CrossPlaneReference::ChannelBind(identity::ChannelBindId::new(reference(
            "cross-channel",
        ))),
        cross_plane::CrossPlaneReference::Credential(identity::CredentialRef::new(reference(
            "cross-credential",
        ))),
    ];
    for item in &refs {
        assert!(!format!("{:?}", item.lifecycle()).is_empty());
    }
    let policy = cross_plane::CrossPlaneBindingPolicy::new(
        cross_plane::CrossPlaneBindingClass::SfuEndpointBinding,
        cross_plane::CrossPlane::Signaling,
        cross_plane::CrossPlane::Sfu,
        refs[1].clone(),
        refs[4].clone(),
        Some(security::AuthorizationContextClass::PublicationPolicyContext),
        cross_plane::BindingLifecyclePrecondition::ParticipantJoined,
        cross_plane::BindingLifecyclePrecondition::EndpointAdmitted,
        cross_plane::BindingExpiryBehavior::RejectNewTargetPlaneAction,
        cross_plane::BindingReplayRelation::PriorAcceptedBindingObservable,
    );
    assert_eq!(
        policy.binding_class(),
        cross_plane::CrossPlaneBindingClass::SfuEndpointBinding
    );
    for outcome in [
        cross_plane::CrossPlaneBindingOutcome::Accepted,
        cross_plane::CrossPlaneBindingOutcome::Rejected,
        cross_plane::CrossPlaneBindingOutcome::Expired,
        cross_plane::CrossPlaneBindingOutcome::Failed,
    ] {
        assert!(!outcome.code().is_empty());
        assert_eq!(
            outcome.requires_reason(),
            !matches!(outcome, cross_plane::CrossPlaneBindingOutcome::Accepted)
        );
    }
    for kind in [
        cross_plane::CrossPlaneBindingFailureKind::BindingClassNotAdmitted,
        cross_plane::CrossPlaneBindingFailureKind::RequiredBindingAbsent,
        cross_plane::CrossPlaneBindingFailureKind::BindingMaterialInvalid,
        cross_plane::CrossPlaneBindingFailureKind::SourceTargetScopeConflict,
        cross_plane::CrossPlaneBindingFailureKind::LifecycleConflict,
        cross_plane::CrossPlaneBindingFailureKind::BindingExpired,
        cross_plane::CrossPlaneBindingFailureKind::BindingReplayDetected,
        cross_plane::CrossPlaneBindingFailureKind::ParticipantNotAdmitted,
        cross_plane::CrossPlaneBindingFailureKind::TargetUnavailable,
        cross_plane::CrossPlaneBindingFailureKind::AllocationNotFound,
        cross_plane::CrossPlaneBindingFailureKind::PermissionNotFound,
        cross_plane::CrossPlaneBindingFailureKind::SecureMediaProtectionNotActive,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    let decision = cross_plane::CrossPlaneBindingDecision::new(
        correlation("cross-plane-decision"),
        policy,
        cross_plane::CrossPlaneBindingOutcome::Rejected,
        Some(cross_plane::CrossPlaneBindingFailureKind::LifecycleConflict),
    );
    assert_eq!(decision.audit_event_type(), "cross_plane_binding_decision");
    let _prohibited = [
        cross_plane::ProhibitedCrossPlaneEquivalence::SignalingJoinAsSfuAdmission,
        cross_plane::ProhibitedCrossPlaneEquivalence::TokenOrAuthorizationAsPlaneBinding,
        cross_plane::ProhibitedCrossPlaneEquivalence::SameCorrelationIdAsBinding,
        cross_plane::ProhibitedCrossPlaneEquivalence::IceRelayAsTurnAllocationOrConnectivityProof,
        cross_plane::ProhibitedCrossPlaneEquivalence::TurnCredentialDeliveryAsActiveTurnState,
        cross_plane::ProhibitedCrossPlaneEquivalence::SecureMediaProtectionAsAuthorization,
        cross_plane::ProhibitedCrossPlaneEquivalence::DriverLocalMapAsBindingAuthority,
        cross_plane::ProhibitedCrossPlaneEquivalence::SilentCrossPlaneLifecycleMutation,
    ];
    let _protocol_surface = protocol::CoreProtocolSurface;
}

#[test]
fn coverage_core_foundation_protocol_envelope_and_state_policy_are_exercised() {
    let signaling_commands = [
        protocol::SignalingSemanticCommandType::JoinRoom,
        protocol::SignalingSemanticCommandType::LeaveRoom,
        protocol::SignalingSemanticCommandType::SendOffer,
        protocol::SignalingSemanticCommandType::SendAnswer,
        protocol::SignalingSemanticCommandType::SendIceCandidate,
        protocol::SignalingSemanticCommandType::RequestTurnCredential,
        protocol::SignalingSemanticCommandType::AcknowledgeForward,
    ];
    let signaling_events = [
        protocol::SignalingSemanticEventType::Joined,
        protocol::SignalingSemanticEventType::Rejected,
        protocol::SignalingSemanticEventType::ParticipantJoined,
        protocol::SignalingSemanticEventType::ParticipantLeft,
        protocol::SignalingSemanticEventType::OfferReceived,
        protocol::SignalingSemanticEventType::AnswerReceived,
        protocol::SignalingSemanticEventType::IceCandidateReceived,
        protocol::SignalingSemanticEventType::TurnCredentialAvailable,
        protocol::SignalingSemanticEventType::ProtocolViolation,
    ];
    let sfu_models = [
        protocol::SfuSemanticModelType::SfuSession,
        protocol::SfuSemanticModelType::ParticipantEndpoint,
        protocol::SfuSemanticModelType::MediaStream,
        protocol::SfuSemanticModelType::MediaNegotiationReference,
        protocol::SfuSemanticModelType::Publication,
        protocol::SfuSemanticModelType::Subscription,
        protocol::SfuSemanticModelType::ForwardingIntent,
        protocol::SfuSemanticModelType::RouteCandidate,
        protocol::SfuSemanticModelType::BorrowedPacketAbstractView,
        protocol::SfuSemanticModelType::QualityObservation,
        protocol::SfuSemanticModelType::BackpressureState,
        protocol::SfuSemanticModelType::AdmissionDecision,
        protocol::SfuSemanticModelType::RejectionReason,
    ];
    let sfu_decisions = [
        protocol::SfuSemanticDecisionType::ParticipantAdmission,
        protocol::SfuSemanticDecisionType::Publication,
        protocol::SfuSemanticDecisionType::Subscription,
        protocol::SfuSemanticDecisionType::RouteSelection,
        protocol::SfuSemanticDecisionType::Forwarding,
        protocol::SfuSemanticDecisionType::DegradationRecovery,
        protocol::SfuSemanticDecisionType::BackpressureAction,
        protocol::SfuSemanticDecisionType::QualityViolation,
    ];
    let turn_models = [
        protocol::TurnSemanticModelType::TransactionId,
        protocol::TurnSemanticModelType::Request,
        protocol::TurnSemanticModelType::Response,
        protocol::TurnSemanticModelType::Indication,
        protocol::TurnSemanticModelType::Error,
        protocol::TurnSemanticModelType::Allocation,
        protocol::TurnSemanticModelType::Permission,
        protocol::TurnSemanticModelType::ChannelBinding,
        protocol::TurnSemanticModelType::ChannelBindingReference,
        protocol::TurnSemanticModelType::PeerAddress,
        protocol::TurnSemanticModelType::RelayDecision,
        protocol::TurnSemanticModelType::CredentialVerificationOutcome,
        protocol::TurnSemanticModelType::LifetimeExpiry,
        protocol::TurnSemanticModelType::ClosedErrorReason,
    ];
    let turn_decisions = [
        protocol::TurnSemanticDecisionType::Allocation,
        protocol::TurnSemanticDecisionType::AllocationLifecycle,
        protocol::TurnSemanticDecisionType::Refresh,
        protocol::TurnSemanticDecisionType::Permission,
        protocol::TurnSemanticDecisionType::ChannelBind,
        protocol::TurnSemanticDecisionType::Relay,
        protocol::TurnSemanticDecisionType::MalformedMessage,
        protocol::TurnSemanticDecisionType::ExpiredCredential,
        protocol::TurnSemanticDecisionType::UnauthorizedRequest,
    ];
    let transport_commands = [
        protocol::TransportSemanticCommandType::StartSession,
        protocol::TransportSemanticCommandType::ApplyLocalDescription,
        protocol::TransportSemanticCommandType::ApplyRemoteDescription,
        protocol::TransportSemanticCommandType::AddIceCandidate,
        protocol::TransportSemanticCommandType::ForwardPacket,
        protocol::TransportSemanticCommandType::CloseSession,
    ];
    let transport_events = [
        protocol::TransportSemanticEventType::SessionObserved,
        protocol::TransportSemanticEventType::LocalDescriptionAccepted,
        protocol::TransportSemanticEventType::RemoteDescriptionAccepted,
        protocol::TransportSemanticEventType::IceCandidateObserved,
        protocol::TransportSemanticEventType::PacketSemanticViewObserved,
        protocol::TransportSemanticEventType::TransportClosed,
        protocol::TransportSemanticEventType::ConversionFailed,
    ];
    assert_eq!(signaling_commands.len(), 7);
    assert_eq!(signaling_events.len(), 9);
    assert_eq!(sfu_models.len(), 13);
    assert_eq!(sfu_decisions.len(), 8);
    assert_eq!(turn_models.len(), 14);
    assert_eq!(turn_decisions.len(), 9);
    assert_eq!(transport_commands.len(), 6);
    assert_eq!(transport_events.len(), 7);

    let payload = protocol::CoreSemanticPayloadModel::new(
        protocol::CoreSemanticPayloadClass::OpaqueCoreReference,
        Some(reference("semantic-payload")),
    );
    let envelope = protocol::CoreSemanticEnvelope::try_new(
        command::TargetSurface::Signaling,
        protocol::ContractVersion::new(1, 0, 0),
        correlation("semantic-envelope"),
        protocol::SemanticEnvelopeMessageKind::Command,
        protocol::CoreSemanticMessageType::SignalingCommand(
            protocol::SignalingSemanticCommandType::JoinRoom,
        ),
        true,
        Some(payload),
        Some(command::UseCaseOutcome::Accepted),
        None,
    )
    .expect("successful envelope carries no reason");
    assert!(!format!("{:?}", envelope).is_empty());
    assert_eq!(
        protocol::CoreSemanticEnvelope::try_new(
            command::TargetSurface::Signaling,
            protocol::ContractVersion::new(1, 0, 0),
            correlation("semantic-envelope-bad"),
            protocol::SemanticEnvelopeMessageKind::Command,
            protocol::CoreSemanticMessageType::SignalingCommand(
                protocol::SignalingSemanticCommandType::JoinRoom,
            ),
            true,
            None,
            Some(command::UseCaseOutcome::Rejected),
            None,
        ),
        Err(protocol::SemanticEnvelopeError::RequiredReasonMissing)
    );
    assert_eq!(
        protocol::CoreSemanticEnvelope::try_new(
            command::TargetSurface::Signaling,
            protocol::ContractVersion::new(1, 0, 0),
            correlation("semantic-envelope-bad2"),
            protocol::SemanticEnvelopeMessageKind::Event,
            protocol::CoreSemanticMessageType::SignalingEvent(
                protocol::SignalingSemanticEventType::Joined,
            ),
            true,
            None,
            Some(command::UseCaseOutcome::Accepted),
            Some(cataloged("room_closed")),
        ),
        Err(protocol::SemanticEnvelopeError::SuccessReasonMustNotBeInvented)
    );
    let _message_types = [
        protocol::CoreSemanticMessageType::SfuModel(protocol::SfuSemanticModelType::SfuSession),
        protocol::CoreSemanticMessageType::SfuDecision(
            protocol::SfuSemanticDecisionType::Forwarding,
        ),
        protocol::CoreSemanticMessageType::TurnModel(protocol::TurnSemanticModelType::Allocation),
        protocol::CoreSemanticMessageType::TurnDecision(protocol::TurnSemanticDecisionType::Relay),
        protocol::CoreSemanticMessageType::TransportCommand(
            protocol::TransportSemanticCommandType::ForwardPacket,
        ),
        protocol::CoreSemanticMessageType::TransportEvent(
            protocol::TransportSemanticEventType::ConversionFailed,
        ),
        protocol::CoreSemanticMessageType::DriverErrorConverted,
    ];
    let _payload_classes = [
        protocol::CoreSemanticPayloadClass::SignalingSubject,
        protocol::CoreSemanticPayloadClass::SfuReferenceSet,
        protocol::CoreSemanticPayloadClass::TurnReferenceSet,
        protocol::CoreSemanticPayloadClass::PacketSemanticView,
        protocol::CoreSemanticPayloadClass::OpaqueCoreReference,
    ];

    assert_eq!(format!("{:?}", state::CoreStateSurface), "CoreStateSurface");
    for class in [
        state::StateClass::EphemeralCoreState,
        state::StateClass::CheckpointEligibleState,
        state::StateClass::AuditOnlyState,
        state::StateClass::DriverLocalState,
        state::StateClass::ConfigurationScopeState,
        state::StateClass::SdkLocalState,
    ] {
        assert!(!format!("{:?}", class.persistence_rule()).is_empty());
        assert_eq!(
            class.can_be_core_source_of_truth(),
            matches!(
                class,
                state::StateClass::EphemeralCoreState
                    | state::StateClass::CheckpointEligibleState
                    | state::StateClass::AuditOnlyState
                    | state::StateClass::ConfigurationScopeState
            )
        );
    }
    for family in [
        state::StateFamily::SignalingRoom,
        state::StateFamily::SignalingParticipant,
        state::StateFamily::SignalingIdempotency,
        state::StateFamily::SfuForwardingState,
        state::StateFamily::TurnRelayAuthorizationState,
        state::StateFamily::AuditEvent,
        state::StateFamily::AuditHashChainRecord,
        state::StateFamily::AtomicityCompensationEvidence,
        state::StateFamily::ResourceBoundCounters,
        state::StateFamily::DriverRetryStore,
        state::StateFamily::MetricsBacklog,
        state::StateFamily::ConfigurationDecision,
        state::StateFamily::SdkConnectionState,
    ] {
        assert!(!format!("{:?}", family.default_class()).is_empty());
        assert!(!family.implicit_durable_source_of_truth_allowed());
    }
    assert_eq!(
        state::CheckpointIntent::try_new(
            state::StateFamily::SignalingRoom,
            state::StateClass::EphemeralCoreState,
            state::CheckpointOwnerBoundary::CoreIntentAndVersion,
            true,
            false,
        ),
        Err(state::CheckpointIntentError::StateClassNotCheckpointEligible)
    );
    assert_eq!(
        state::CheckpointIntent::try_new(
            state::StateFamily::SignalingIdempotency,
            state::StateClass::CheckpointEligibleState,
            state::CheckpointOwnerBoundary::CoreIntentAndVersion,
            false,
            false,
        ),
        Err(state::CheckpointIntentError::RestorePolicyRequired)
    );
    let checkpoint = state::CheckpointIntent::try_new(
        state::StateFamily::SignalingIdempotency,
        state::StateClass::CheckpointEligibleState,
        state::CheckpointOwnerBoundary::DriverSchemaAndStorageLayout,
        true,
        true,
    )
    .expect("checkpoint intent has restore policy");
    assert!(!format!("{:?}", checkpoint).is_empty());
    let _claims = [
        state::SourceOfTruthClaim::NoImplicitDurableDomainSourceOfTruth,
        state::SourceOfTruthClaim::AuditReplayVerificationOnly,
        state::SourceOfTruthClaim::RestorePolicyRequired,
        state::SourceOfTruthClaim::DistributedPolicyRequired,
    ];
    for kind in [
        state::StatePersistenceFailureKind::PersistenceUnavailable,
        state::StatePersistenceFailureKind::PersistenceRetryBoundExceeded,
        state::StatePersistenceFailureKind::PersistenceRetryDurationExceeded,
        state::StatePersistenceFailureKind::AuditBacklogBoundExceeded,
        state::StatePersistenceFailureKind::DriverShutdown,
    ] {
        assert_reason_is_cataloged(kind.reason_code());
    }
    let _prohibited_state = [
        state::ProhibitedStatePersistenceBehavior::DriverSchemaAsDomainSourceOfTruth,
        state::ProhibitedStatePersistenceBehavior::CheckpointRestoreWithoutRestorePolicy,
        state::ProhibitedStatePersistenceBehavior::SfuRouteDurableByDefault,
        state::ProhibitedStatePersistenceBehavior::TurnAllocationSilentlyRestored,
        state::ProhibitedStatePersistenceBehavior::AuditLogAsMutableStateStore,
        state::ProhibitedStatePersistenceBehavior::DriverDbTransactionAsAggregateCommitAuthority,
        state::ProhibitedStatePersistenceBehavior::SdkLocalStateAsServerParticipantState,
        state::ProhibitedStatePersistenceBehavior::DriverRetryQueueAsDomainState,
        state::ProhibitedStatePersistenceBehavior::PersistedStateAsFailoverReadyWithoutPolicy,
    ];
}
