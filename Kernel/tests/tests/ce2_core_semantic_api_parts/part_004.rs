use arcrtc_core_configuration as cfg_core;
use arcrtc_core_domain as domain_core;
use arcrtc_core_features as features_core;
use arcrtc_core_identity::{
    ChannelBindId, ConfigurationScopeRef, CoreIdentitySurface, ExternalExposure,
    ExternalIdentitySource, IdentityKind, IdentityLifetime, OpaqueReferenceError,
    UntrustedReference,
};
use arcrtc_core_security as security_core;
use arcrtc_core_state as state_core;
use arcrtc_core_time as time_core;
use arcrtc_core_transport as transport_core;

#[test]
fn ce2_cov2_configuration_domain_and_features_catalogs_are_closed() {
    let _configuration_surface = cfg_core::CoreConfigurationSurface;
    let policy = cfg_core::CorePolicyConfiguration::new("core-policy-v1");
    assert_eq!(*policy.policy(), "core-policy-v1");

    let non_core = cfg_core::NonCoreConfigurationBoundary::new(
        cfg_core::ConfigurationKind::DriverRuntime,
        cfg_core::ConfigurationOwner::Drivers,
        cfg_core::ConfigurationSourceClass::EnvironmentVariable,
    );
    assert_eq!(non_core.kind(), cfg_core::ConfigurationKind::DriverRuntime);
    assert_eq!(non_core.owner(), cfg_core::ConfigurationOwner::Drivers);

    let configuration_kinds = [
        cfg_core::ConfigurationKind::CorePolicy,
        cfg_core::ConfigurationKind::DriverRuntime,
        cfg_core::ConfigurationKind::EntrypointComposition,
        cfg_core::ConfigurationKind::SdkClient,
        cfg_core::ConfigurationKind::Regulated,
    ];
    assert_eq!(configuration_kinds.len(), 5);

    let configuration_owners = [
        cfg_core::ConfigurationOwner::Core,
        cfg_core::ConfigurationOwner::Drivers,
        cfg_core::ConfigurationOwner::Entrypoints,
        cfg_core::ConfigurationOwner::Sdk,
        cfg_core::ConfigurationOwner::Regulated,
    ];
    assert_eq!(configuration_owners.len(), 5);

    let source_classes = [
        cfg_core::ConfigurationSourceClass::TypedInput,
        cfg_core::ConfigurationSourceClass::EnvironmentVariable,
        cfg_core::ConfigurationSourceClass::File,
        cfg_core::ConfigurationSourceClass::ProcessArgs,
        cfg_core::ConfigurationSourceClass::OsSettings,
        cfg_core::ConfigurationSourceClass::CloudMetadata,
    ];
    assert_eq!(source_classes.len(), 6);

    let allowed_effects = [
        cfg_core::FeatureFlagAllowedEffect::SelectDriverImplementation,
        cfg_core::FeatureFlagAllowedEffect::EnableOptionalExporter,
        cfg_core::FeatureFlagAllowedEffect::ChooseExternalEncoding,
    ];
    assert_eq!(allowed_effects.len(), 3);

    let prohibited_effects = [
        cfg_core::FeatureFlagProhibitedEffect::SignalingStateMachineChange,
        cfg_core::FeatureFlagProhibitedEffect::TurnLifecycleChange,
        cfg_core::FeatureFlagProhibitedEffect::SfuRoutingSemanticsChange,
        cfg_core::FeatureFlagProhibitedEffect::SecurityVerificationBypass,
        cfg_core::FeatureFlagProhibitedEffect::AuditRequirementBypass,
    ];
    assert_eq!(prohibited_effects.len(), 5);

    let configuration_failures = [
        (
            cfg_core::ConfigurationFailureKind::CorePolicyConfigInvalid,
            "core_policy_config_invalid",
        ),
        (
            cfg_core::ConfigurationFailureKind::RuntimeConfigMissing,
            "runtime_config_missing",
        ),
        (
            cfg_core::ConfigurationFailureKind::RuntimeConfigInvalid,
            "runtime_config_invalid",
        ),
        (
            cfg_core::ConfigurationFailureKind::SecretUnavailable,
            "secret_unavailable",
        ),
    ];
    for (kind, code) in configuration_failures {
        assert_eq!(kind.reason_code(), code);
    }

    let secret_boundaries = [
        cfg_core::SecretBoundary::RawSecretOutsideCore,
        cfg_core::SecretBoundary::OpaqueCredentialReferenceOnly,
    ];
    assert_eq!(secret_boundaries.len(), 2);

    let _domain_surface = domain_core::CoreDomainSurface;
    let aggregate_families = [
        domain_core::AggregateFamily::SignalingRoomParticipant,
        domain_core::AggregateFamily::SfuSessionEndpointRoute,
        domain_core::AggregateFamily::TurnAllocationPermissionChannelBind,
        domain_core::AggregateFamily::CrossPlaneBindingScope,
        domain_core::AggregateFamily::AuditChainScope,
        domain_core::AggregateFamily::ConfigurationScope,
    ];
    assert_eq!(aggregate_families.len(), 6);
    assert_eq!(
        domain_core::ENTRYPOINTLICATION_USE_CASE_ORDER,
        &[
            domain_core::UseCaseStep::ReceiveCoreOwnedCommand,
            domain_core::UseCaseStep::VerifyCoreGuards,
            domain_core::UseCaseStep::DelegateDomainDecision,
            domain_core::UseCaseStep::ConnectDecisionReason,
            domain_core::UseCaseStep::EmitCoreEffects,
        ]
    );
    let use_case = domain_core::UseCaseBoundary::<CommandEnvelope<()>, UseCaseDecision<()>>::new(
        "semantic-domain-boundary",
        domain_core::ENTRYPOINTLICATION_USE_CASE_ORDER,
    );
    assert_eq!(use_case.name(), "semantic-domain-boundary");
    assert_eq!(use_case.steps().len(), 5);

    #[derive(Clone, Eq, PartialEq)]
    struct TestValue;
    impl domain_core::ValueObject for TestValue {}

    struct TestAggregate;
    impl domain_core::AggregateRoot for TestAggregate {
        type Id = TestValue;

        fn family(&self) -> domain_core::AggregateFamily {
            domain_core::AggregateFamily::SfuSessionEndpointRoute
        }
    }

    struct TestDomainService;
    impl domain_core::DomainService for TestDomainService {
        fn aggregate_families(&self) -> &'static [domain_core::AggregateFamily] {
            &[
                domain_core::AggregateFamily::SignalingRoomParticipant,
                domain_core::AggregateFamily::SfuSessionEndpointRoute,
            ]
        }
    }

    assert_eq!(
        domain_core::AggregateRoot::family(&TestAggregate),
        domain_core::AggregateFamily::SfuSessionEndpointRoute
    );
    assert_eq!(
        domain_core::DomainService::aggregate_families(&TestDomainService).len(),
        2
    );

    let _features_surface = features_core::CoreFeaturesSurface;
    let excluded_features = [
        features_core::ExcludedFeatureClass::ChatApplicationSemantics,
        features_core::ExcludedFeatureClass::RecordingWorkflow,
        features_core::ExcludedFeatureClass::ScreenShareWorkflow,
        features_core::ExcludedFeatureClass::DataChannelApplicationSemantics,
        features_core::ExcludedFeatureClass::UiEndUserWorkflow,
        features_core::ExcludedFeatureClass::MediaCaptureWorkflow,
        features_core::ExcludedFeatureClass::RegulatedDomainWorkflow,
    ];
    assert_eq!(excluded_features.len(), 7);

    let requested_surfaces = [
        features_core::FeatureRequestedSurface::Core,
        features_core::FeatureRequestedSurface::Signaling,
        features_core::FeatureRequestedSurface::Sfu,
        features_core::FeatureRequestedSurface::Turn,
        features_core::FeatureRequestedSurface::Sdk,
        features_core::FeatureRequestedSurface::Driver,
        features_core::FeatureRequestedSurface::Entrypoints,
        features_core::FeatureRequestedSurface::Regulated,
    ];
    assert_eq!(requested_surfaces.len(), 8);

    let admission_requirements = [
        features_core::FutureAdmissionRequirement::FeatureClass,
        features_core::FutureAdmissionRequirement::OwnerPackageLayer,
        features_core::FutureAdmissionRequirement::GenericCoreRelation,
        features_core::FutureAdmissionRequirement::PublicSdkApiSurface,
        features_core::FutureAdmissionRequirement::DriverRuntimeDependencyBoundary,
        features_core::FutureAdmissionRequirement::SecurityPrivacyRedactionBoundary,
        features_core::FutureAdmissionRequirement::ReasonCatalogAdditions,
        features_core::FutureAdmissionRequirement::AuditEventRelation,
        features_core::FutureAdmissionRequirement::EvidenceClass,
        features_core::FutureAdmissionRequirement::MigrationDeprecationRelation,
    ];
    assert_eq!(admission_requirements.len(), 10);

    let admission = features_core::FeatureAdmissionDecision::new(
        Some(correlation()),
        features_core::ExcludedFeatureClass::DataChannelApplicationSemantics,
        features_core::FeatureRequestedSurface::Sdk,
        features_core::FeatureAdmissionDecisionClass::Rejected,
        Some(features_core::FeatureAdmissionFailureKind::DataChannelNotSupported),
    );
    assert_eq!(
        admission.feature_class(),
        features_core::ExcludedFeatureClass::DataChannelApplicationSemantics
    );
    assert_eq!(
        admission.requested_surface(),
        features_core::FeatureRequestedSurface::Sdk
    );
    assert_eq!(
        admission.decision_class(),
        features_core::FeatureAdmissionDecisionClass::Rejected
    );

    let decision_classes = [
        features_core::FeatureAdmissionDecisionClass::Rejected,
        features_core::FeatureAdmissionDecisionClass::CloseNotClaimed,
        features_core::FeatureAdmissionDecisionClass::AdmittedByCanonical,
    ];
    assert_eq!(decision_classes.len(), 3);

    let feature_failures = [
        (
            features_core::FeatureAdmissionFailureKind::FeatureOutOfScope,
            "feature_out_of_scope",
        ),
        (
            features_core::FeatureAdmissionFailureKind::FeatureAdmissionNotDocumented,
            "feature_admission_not_documented",
        ),
        (
            features_core::FeatureAdmissionFailureKind::ChatNotSupported,
            "chat_not_supported",
        ),
        (
            features_core::FeatureAdmissionFailureKind::RecordingNotSupported,
            "recording_not_supported",
        ),
        (
            features_core::FeatureAdmissionFailureKind::ScreenShareNotSupported,
            "screen_share_not_supported",
        ),
        (
            features_core::FeatureAdmissionFailureKind::DataChannelNotSupported,
            "datachannel_not_supported",
        ),
        (
            features_core::FeatureAdmissionFailureKind::UiWorkflowNotSupported,
            "ui_workflow_not_supported",
        ),
        (
            features_core::FeatureAdmissionFailureKind::MediaCaptureNotSupported,
            "media_capture_not_supported",
        ),
        (
            features_core::FeatureAdmissionFailureKind::RegulatedWorkflowNotSupported,
            "regulated_workflow_not_supported",
        ),
    ];
    for (kind, code) in feature_failures {
        assert_eq!(kind.reason_code(), code);
    }
}

#[test]
fn ce2_cov2_state_identity_security_and_time_boundaries_cover_closed_shapes() {
    let _state_surface = state_core::CoreStateSurface;
    let state_classes = [
        (
            state_core::StateClass::EphemeralCoreState,
            state_core::PersistenceRule::PersistenceNotRequired,
            true,
        ),
        (
            state_core::StateClass::CheckpointEligibleState,
            state_core::PersistenceRule::PersistOnlyViaPersistencePort,
            true,
        ),
        (
            state_core::StateClass::AuditOnlyState,
            state_core::PersistenceRule::AuditSinkOrHashChainOnly,
            true,
        ),
        (
            state_core::StateClass::DriverLocalState,
            state_core::PersistenceRule::DriverOwnedNotSourceOfTruth,
            false,
        ),
        (
            state_core::StateClass::ConfigurationScopeState,
            state_core::PersistenceRule::ConfigurationDecisionStartupReferences,
            true,
        ),
        (
            state_core::StateClass::SdkLocalState,
            state_core::PersistenceRule::SdkOwnedNotServerState,
            false,
        ),
    ];
    for (state_class, persistence_rule, source_of_truth) in state_classes {
        assert_eq!(state_class.persistence_rule(), persistence_rule);
        assert_eq!(
            state_class.can_be_core_source_of_truth(),
            source_of_truth
        );
    }

    let state_families = [
        state_core::StateFamily::SignalingRoom,
        state_core::StateFamily::SignalingParticipant,
        state_core::StateFamily::SignalingIdempotency,
        state_core::StateFamily::SfuForwardingState,
        state_core::StateFamily::TurnRelayAuthorizationState,
        state_core::StateFamily::AuditEvent,
        state_core::StateFamily::AuditHashChainRecord,
        state_core::StateFamily::AtomicityCompensationEvidence,
        state_core::StateFamily::ResourceBoundCounters,
        state_core::StateFamily::DriverRetryStore,
        state_core::StateFamily::MetricsBacklog,
        state_core::StateFamily::ConfigurationDecision,
        state_core::StateFamily::SdkConnectionState,
    ];
    for family in state_families {
        let state_class = family.default_class();
        assert!(!family.implicit_durable_source_of_truth_allowed());
        if matches!(
            family,
            state_core::StateFamily::DriverRetryStore
                | state_core::StateFamily::MetricsBacklog
                | state_core::StateFamily::SdkConnectionState
        ) {
            assert!(!state_class.can_be_core_source_of_truth());
        }
    }

    assert!(state_core::CheckpointIntent::try_new(
        state_core::StateFamily::SignalingIdempotency,
        state_core::StateClass::CheckpointEligibleState,
        state_core::CheckpointOwnerBoundary::CoreIntentAndVersion,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        state_core::CheckpointIntent::try_new(
            state_core::StateFamily::SignalingRoom,
            state_core::StateClass::EphemeralCoreState,
            state_core::CheckpointOwnerBoundary::DriverSchemaAndStorageLayout,
            true,
            false,
        ),
        Err(state_core::CheckpointIntentError::StateClassNotCheckpointEligible)
    );
    assert_eq!(
        state_core::CheckpointIntent::try_new(
            state_core::StateFamily::SignalingIdempotency,
            state_core::StateClass::CheckpointEligibleState,
            state_core::CheckpointOwnerBoundary::CoreIntentAndVersion,
            false,
            false,
        ),
        Err(state_core::CheckpointIntentError::RestorePolicyRequired)
    );

    let source_of_truth_claims = [
        state_core::SourceOfTruthClaim::NoImplicitDurableDomainSourceOfTruth,
        state_core::SourceOfTruthClaim::AuditReplayVerificationOnly,
        state_core::SourceOfTruthClaim::RestorePolicyRequired,
        state_core::SourceOfTruthClaim::DistributedPolicyRequired,
    ];
    assert_eq!(source_of_truth_claims.len(), 4);

    let persistence_failures = [
        (
            state_core::StatePersistenceFailureKind::PersistenceUnavailable,
            "persistence_unavailable",
        ),
        (
            state_core::StatePersistenceFailureKind::PersistenceRetryBoundExceeded,
            "persistence_retry_bound_exceeded",
        ),
        (
            state_core::StatePersistenceFailureKind::PersistenceRetryDurationExceeded,
            "persistence_retry_duration_exceeded",
        ),
        (
            state_core::StatePersistenceFailureKind::AuditBacklogBoundExceeded,
            "audit_backlog_bound_exceeded",
        ),
        (
            state_core::StatePersistenceFailureKind::DriverShutdown,
            "driver_shutdown",
        ),
    ];
    for (kind, code) in persistence_failures {
        assert_eq!(kind.reason_code(), code);
    }

    let prohibited_state_behaviors = [
        state_core::ProhibitedStatePersistenceBehavior::DriverSchemaAsDomainSourceOfTruth,
        state_core::ProhibitedStatePersistenceBehavior::CheckpointRestoreWithoutRestoreCanonical,
        state_core::ProhibitedStatePersistenceBehavior::SfuRouteDurableByDefault,
        state_core::ProhibitedStatePersistenceBehavior::TurnAllocationSilentlyRestored,
        state_core::ProhibitedStatePersistenceBehavior::AuditLogAsMutableStateStore,
        state_core::ProhibitedStatePersistenceBehavior::DriverDbTransactionAsAggregateCommitAuthority,
        state_core::ProhibitedStatePersistenceBehavior::SdkLocalStateAsServerParticipantState,
        state_core::ProhibitedStatePersistenceBehavior::DriverRetryQueueAsDomainState,
        state_core::ProhibitedStatePersistenceBehavior::PersistedStateAsFailoverReadyWithoutPolicy,
    ];
    assert_eq!(prohibited_state_behaviors.len(), 9);

    let _identity_surface = CoreIdentitySurface;
    let identity_kinds = [
        (
            IdentityKind::CorrelationId,
            IdentityLifetime::SingleCommandEventChain,
            ExternalExposure::PublicOpaque,
        ),
        (
            IdentityKind::RoomId,
            IdentityLifetime::RoomLifecycle,
            ExternalExposure::PublicOpaque,
        ),
        (
            IdentityKind::SessionId,
            IdentityLifetime::TransportSessionLifecycle,
            ExternalExposure::PublicOpaque,
        ),
        (
            IdentityKind::ParticipantId,
            IdentityLifetime::RoomMembershipLifecycle,
            ExternalExposure::PublicOpaque,
        ),
        (
            IdentityKind::EndpointId,
            IdentityLifetime::SfuEndpointLifecycle,
            ExternalExposure::ControlledExternal,
        ),
        (
            IdentityKind::StreamId,
            IdentityLifetime::MediaStreamLifecycle,
            ExternalExposure::ControlledExternal,
        ),
        (
            IdentityKind::PacketId,
            IdentityLifetime::PacketLifecycle,
            ExternalExposure::InternalByDefault,
        ),
        (
            IdentityKind::RouteId,
            IdentityLifetime::RouteDecisionLifecycle,
            ExternalExposure::InternalByDefault,
        ),
        (
            IdentityKind::AllocationId,
            IdentityLifetime::TurnAllocationLifecycle,
            ExternalExposure::InternalByDefault,
        ),
        (
            IdentityKind::PermissionId,
            IdentityLifetime::TurnPermissionLifecycle,
            ExternalExposure::InternalByDefault,
        ),
        (
            IdentityKind::ChannelBindId,
            IdentityLifetime::TurnChannelBindLifecycle,
            ExternalExposure::InternalByDefault,
        ),
        (
            IdentityKind::AuditEventId,
            IdentityLifetime::AuditEventLifecycle,
            ExternalExposure::AuditOnly,
        ),
        (
            IdentityKind::CredentialRef,
            IdentityLifetime::CredentialVerificationLifecycle,
            ExternalExposure::NoRawSecretExposure,
        ),
        (
            IdentityKind::StartupRunId,
            IdentityLifetime::SingleStartupWiringAttempt,
            ExternalExposure::AuditOnly,
        ),
        (
            IdentityKind::ConfigurationScopeRef,
            IdentityLifetime::ConfigurationValidationLifecycle,
            ExternalExposure::AuditOnly,
        ),
    ];
    for (kind, lifetime, exposure) in identity_kinds {
        assert_eq!(kind.lifetime(), lifetime);
        assert_eq!(kind.external_exposure(), exposure);
    }

    let untrusted = UntrustedReference::new("untrusted-reference-ce2");
    assert_eq!(untrusted.as_str(), "untrusted-reference-ce2");
    let accepted = OpaqueReference::accept_untrusted(
        untrusted,
        ReferenceAuthority::CoreValidatedUntrustedInput,
    )
    .expect("validated untrusted input can become opaque reference");
    assert_eq!(accepted.as_str(), "untrusted-reference-ce2");
    assert_eq!(
        accepted.authority(),
        ReferenceAuthority::CoreValidatedUntrustedInput
    );
    assert_eq!(
        OpaqueReference::accept("", ReferenceAuthority::CorePolicy),
        Err(OpaqueReferenceError::Empty)
    );
    assert_eq!(
        OpaqueReference::accept("bad\nreference", ReferenceAuthority::CorePolicy),
        Err(OpaqueReferenceError::ControlCharacter)
    );

    let typed_references = [
        (
            CorrelationId::kind(),
            CorrelationId::new(reference("id-correlation"))
                .as_str()
                .to_owned(),
        ),
        (
            RoomId::kind(),
            RoomId::new(reference("id-room")).as_str().to_owned(),
        ),
        (
            SessionId::kind(),
            SessionId::new(reference("id-session"))
                .as_str()
                .to_owned(),
        ),
        (
            ParticipantId::kind(),
            ParticipantId::new(reference("id-participant"))
                .as_str()
                .to_owned(),
        ),
        (
            EndpointId::kind(),
            EndpointId::new(reference("id-endpoint"))
                .as_str()
                .to_owned(),
        ),
        (
            StreamId::kind(),
            StreamId::new(reference("id-stream")).as_str().to_owned(),
        ),
        (
            PacketId::kind(),
            PacketId::new(reference("id-packet")).as_str().to_owned(),
        ),
        (
            RouteId::kind(),
            RouteId::new(reference("id-route")).as_str().to_owned(),
        ),
        (
            AllocationId::kind(),
            AllocationId::new(reference("id-allocation"))
                .as_str()
                .to_owned(),
        ),
        (
            PermissionId::kind(),
            PermissionId::new(reference("id-permission"))
                .as_str()
                .to_owned(),
        ),
        (
            ChannelBindId::kind(),
            ChannelBindId::new(reference("id-channel-bind"))
                .as_str()
                .to_owned(),
        ),
        (
            AuditEventId::kind(),
            AuditEventId::new(reference("id-audit-event"))
                .as_str()
                .to_owned(),
        ),
        (
            CredentialRef::kind(),
            CredentialRef::new(reference("id-credential"))
                .as_str()
                .to_owned(),
        ),
        (
            StartupRunId::kind(),
            StartupRunId::new(reference("id-startup"))
                .as_str()
                .to_owned(),
        ),
        (
            ConfigurationScopeRef::kind(),
            ConfigurationScopeRef::new(reference("id-configuration"))
                .as_str()
                .to_owned(),
        ),
    ];
    assert_eq!(typed_references.len(), 15);
    assert!(typed_references
        .iter()
        .all(|(_, value)| value.starts_with("id-")));

    for source in [
        ExternalIdentitySource::ApplicationUser,
        ExternalIdentitySource::TokenSubject,
        ExternalIdentitySource::RegulatedSubject,
        ExternalIdentitySource::TenantRole,
        ExternalIdentitySource::MedicalRole,
        ExternalIdentitySource::FacilityIdentity,
    ] {
        assert!(!source.issues_core_identity());
    }

    let _security_surface = security_core::CoreSecuritySurface;
    let mapping = security_core::AuthorizationMapping::new(
        security_core::AuthorizationMappingSource::VerifiedCredential,
        security_core::AuthorizationContextClass::ParticipantJoinContext,
        vec![TargetSurface::Signaling, TargetSurface::Sfu],
        security_core::AuthorizationLifetime::Bounded("credential-expiry"),
        security_core::AuthorizationRedactionRule::RawClaimsExcluded,
    );
    assert_eq!(
        mapping.context_class(),
        security_core::AuthorizationContextClass::ParticipantJoinContext
    );
    assert_eq!(
        mapping.lifetime(),
        security_core::AuthorizationLifetime::Bounded("credential-expiry")
    );
    assert_eq!(mapping.allowed_target_surfaces().len(), 2);

    let auth_input = security_core::AuthorizationPolicyInput::new(
        correlation(),
        Some(CredentialRef::new(reference("credential-auth-input"))),
        security_core::AuthorizationContextClass::PublicationPolicyContext,
        TargetSurface::Sfu,
        security_core::CommunicationAction::Publish,
        RouteId::new(reference("auth-scope-route")),
    );
    assert_eq!(auth_input.target_surface(), TargetSurface::Sfu);
    assert_eq!(
        auth_input.action(),
        security_core::CommunicationAction::Publish
    );

    assert_eq!(
        security_core::AuthorizationPolicyDecision::<CatalogedReasonRef>::new(
            UseCaseOutcome::Accepted,
            DecisionReason::Absent,
        )
        .expect("accepted authorization carries no reason")
        .outcome(),
        UseCaseOutcome::Accepted
    );
    assert_eq!(
        security_core::AuthorizationPolicyDecision::new(
            UseCaseOutcome::Accepted,
            DecisionReason::Cataloged(cataloged("authorization_policy_denied")),
        ),
        Err(security_core::AuthorizationPolicyError::SuccessMustNotCarryReason)
    );
    assert_eq!(
        security_core::AuthorizationPolicyDecision::<CatalogedReasonRef>::new(
            UseCaseOutcome::Rejected,
            DecisionReason::Absent,
        ),
        Err(security_core::AuthorizationPolicyError::NonSuccessRequiresReason)
    );

    let authorization_failures = [
        (
            security_core::AuthorizationFailureKind::AuthorizationContextMissing,
            "authorization_context_missing",
        ),
        (
            security_core::AuthorizationFailureKind::AuthorizationContextInvalid,
            "authorization_context_invalid",
        ),
        (
            security_core::AuthorizationFailureKind::AuthorizationContextExpired,
            "authorization_context_expired",
        ),
        (
            security_core::AuthorizationFailureKind::AuthorizationPolicyDenied,
            "authorization_policy_denied",
        ),
        (
            security_core::AuthorizationFailureKind::AuthorizationScopeNotAllowed,
            "authorization_scope_not_allowed",
        ),
        (
            security_core::AuthorizationFailureKind::ForwardedHeaderUntrusted,
            "forwarded_header_untrusted",
        ),
        (
            security_core::AuthorizationFailureKind::ClientAddressUntrusted,
            "client_address_untrusted",
        ),
        (
            security_core::AuthorizationFailureKind::TokenVerificationFailed,
            "token_verification_failed",
        ),
        (
            security_core::AuthorizationFailureKind::RuntimeConfigMissing,
            "runtime_config_missing",
        ),
        (
            security_core::AuthorizationFailureKind::RuntimeConfigInvalid,
            "runtime_config_invalid",
        ),
    ];
    for (kind, code) in authorization_failures {
        assert_eq!(kind.reason_code(), code);
    }

    assert_eq!(
        IssuerPolicy::new(vec!["issuer-a", "issuer-b"]).accepted_issuers(),
        &["issuer-a", "issuer-b"]
    );
    assert_eq!(
        AudiencePolicy::new(vec!["audience-a"]).accepted_audiences(),
        &["audience-a"]
    );
    assert_eq!(
        TokenAlgorithmPolicy::new(vec!["EdDSA", "ES256"]).accepted_algorithms(),
        &["EdDSA", "ES256"]
    );

    let credential_ref = CredentialRef::new(reference("credential-token-result"));
    let verified = VerifiedCredential::new(
        correlation(),
        credential_ref.clone(),
        TokenTemporalDecision::Valid,
    );
    assert_eq!(verified.credential_ref(), &credential_ref);
    assert_eq!(verified.temporal_decision(), TokenTemporalDecision::Valid);
    let accepted_result = security_core::TokenVerificationResult::Accepted(verified.clone());
    assert!(matches!(
        accepted_result,
        security_core::TokenVerificationResult::Accepted(_)
    ));
    let rejected_result = security_core::TokenVerificationResult::Rejected(
        security_core::TokenVerificationFailureKind::TokenUnsupportedAlgorithm,
    );
    assert!(matches!(
        rejected_result,
        security_core::TokenVerificationResult::Rejected(
            security_core::TokenVerificationFailureKind::TokenUnsupportedAlgorithm
        )
    ));

    let required_claims = [
        RequiredTokenClaim::Issuer,
        RequiredTokenClaim::Audience,
        RequiredTokenClaim::Subject,
        RequiredTokenClaim::Expiration,
        RequiredTokenClaim::NotBefore,
        RequiredTokenClaim::IssuedAt,
        RequiredTokenClaim::KeyId,
    ];
    assert_eq!(required_claims.len(), 7);

    for (kind, code) in [
        (
            security_core::TokenVerificationFailureKind::TokenMissing,
            "token_missing",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenMalformed,
            "token_malformed",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenSignatureInvalid,
            "token_signature_invalid",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenKeyUnavailable,
            "token_key_unavailable",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenIssuerMismatch,
            "token_issuer_mismatch",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenAudienceMismatch,
            "token_audience_mismatch",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenExpired,
            "token_expired",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenNotYetValid,
            "token_not_yet_valid",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenRequiredClaimMissing,
            "token_required_claim_missing",
        ),
        (
            security_core::TokenVerificationFailureKind::TokenUnsupportedAlgorithm,
            "token_unsupported_algorithm",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
    }

    let _time_surface = time_core::CoreTimeSurface;
    for (quantity, unit) in [
        (
            time_core::NormalizedQuantity::Duration,
            time_core::NormalizedUnit::MillisecondsInteger,
        ),
        (
            time_core::NormalizedQuantity::Timestamp,
            time_core::NormalizedUnit::UtcEpochMillisecondsEvidenceOnly,
        ),
        (
            time_core::NormalizedQuantity::Bytes,
            time_core::NormalizedUnit::BytesInteger,
        ),
        (
            time_core::NormalizedQuantity::PacketCount,
            time_core::NormalizedUnit::IntegerCount,
        ),
        (
            time_core::NormalizedQuantity::Rate,
            time_core::NormalizedUnit::UnitsPerSecond,
        ),
        (
            time_core::NormalizedQuantity::Ratio,
            time_core::NormalizedUnit::RationalDeclaredPrecision,
        ),
        (
            time_core::NormalizedQuantity::Bitrate,
            time_core::NormalizedUnit::BitsPerSecond,
        ),
        (
            time_core::NormalizedQuantity::JitterRtt,
            time_core::NormalizedUnit::MillisecondsDeclaredPrecision,
        ),
    ] {
        assert_eq!(quantity.default_unit(), unit);
    }
    assert_eq!(
        time_core::NormalizedValue::rational(1, 0),
        Err(time_core::NormalizedValueError::ZeroDenominator)
    );
    let rational = time_core::NormalizedValue::rational(1, 2)
        .expect("non-zero denominator is accepted");
    let measurement = time_core::NormalizedMeasurement::new(
        time_core::NormalizedQuantity::Ratio,
        time_core::NormalizedUnit::RationalDeclaredPrecision,
        rational,
        time_core::PrecisionClass::DeclaredDecimalScale(2),
        time_core::RoundingDirection::Nearest,
        time_core::ComparisonOperator::GreaterThanOrEqual,
        time_core::BoundaryInclusivity::Inclusive,
        time_core::SamplingWindow::PerSecond,
        time_core::RawMeasurementOwner::Driver,
        time_core::PolicyDecisionOwner::Core,
    );
    assert_eq!(measurement, measurement);

    let normalization_failures = [
        (
            time_core::TimeNormalizationFailureKind::MeasurementNormalizationFailed,
            "measurement_normalization_failed",
        ),
        (
            time_core::TimeNormalizationFailureKind::TimeObservationUnavailable,
            "time_observation_unavailable",
        ),
        (
            time_core::TimeNormalizationFailureKind::OperationDeadlineExceeded,
            "operation_deadline_exceeded",
        ),
        (
            time_core::TimeNormalizationFailureKind::RetentionDurationExceeded,
            "retention_duration_exceeded",
        ),
        (
            time_core::TimeNormalizationFailureKind::RuntimeConfigInvalid,
            "runtime_config_invalid",
        ),
        (
            time_core::TimeNormalizationFailureKind::DriverShutdown,
            "driver_shutdown",
        ),
    ];
    for (kind, code) in normalization_failures {
        assert_eq!(kind.reason_code(), code);
    }

    let unit_adoptions = [
        time_core::EvidenceUnitWindowAdoption::RawAndNormalizedRecorded,
        time_core::EvidenceUnitWindowAdoption::NormalizedValueRecorded,
        time_core::EvidenceUnitWindowAdoption::MissingUnitWindowNotAdoptable,
    ];
    assert_eq!(unit_adoptions.len(), 3);

    for (concern, owner) in [
        (
            time_core::TimeSynchronizationConcern::LocalMonotonicDuration,
            time_core::TimeSynchronizationOwner::CorePolicy,
        ),
        (
            time_core::TimeSynchronizationConcern::WallClockTimestamp,
            time_core::TimeSynchronizationOwner::DriverRuntimeObservation,
        ),
        (
            time_core::TimeSynchronizationConcern::CrossNodeSkewPolicy,
            time_core::TimeSynchronizationOwner::CorePolicy,
        ),
        (
            time_core::TimeSynchronizationConcern::ExternalTimeSource,
            time_core::TimeSynchronizationOwner::DriverRuntimeObservation,
        ),
        (
            time_core::TimeSynchronizationConcern::TimestampNormalization,
            time_core::TimeSynchronizationOwner::CorePolicy,
        ),
        (
            time_core::TimeSynchronizationConcern::EvidenceTimestampClaim,
            time_core::TimeSynchronizationOwner::EvidenceCanonical,
        ),
    ] {
        assert_eq!(concern.owner(), owner);
    }

    for (trust, runtime_allowed, cross_node_allowed) in [
        (time_core::TimeTrustClass::SingleProcessMonotonic, true, false),
        (time_core::TimeTrustClass::SingleNodeWallClock, true, false),
        (time_core::TimeTrustClass::MultiNodeBoundedSkew, true, true),
        (time_core::TimeTrustClass::ExternalTrustedTimeSource, true, true),
        (time_core::TimeTrustClass::TestDeterministicClock, false, false),
        (time_core::TimeTrustClass::TimeUntrusted, false, false),
    ] {
        assert_eq!(trust.runtime_evidence_allowed(), runtime_allowed);
        assert_eq!(trust.supports_cross_node_comparison(), cross_node_allowed);
    }

    assert_eq!(
        time_core::ClockSkewPolicy::try_new(
            time_core::TimeNodeScope::MultiNode,
            time_core::TimeTrustClass::MultiNodeBoundedSkew,
            None,
            time_core::PrecisionClass::DeclaredPrecisionLabel("ms"),
            time_core::SamplingWindow::Milliseconds(1000),
            time_core::TrustedTimeSourceClass::ExternalTimeSourceObservation,
            time_core::ClockSkewImpact::Ordering,
        ),
        Err(time_core::ClockSkewPolicyError::MaxSkewRequired)
    );
    assert_eq!(
        time_core::ClockSkewPolicy::try_new(
            time_core::TimeNodeScope::MultiProcess,
            time_core::TimeTrustClass::SingleProcessMonotonic,
            Some(1),
            time_core::PrecisionClass::IntegerExact,
            time_core::SamplingWindow::None,
            time_core::TrustedTimeSourceClass::LocalMonotonicClock,
            time_core::ClockSkewImpact::Expiry,
        ),
        Err(time_core::ClockSkewPolicyError::TrustClassCannotSupportScope)
    );
    assert_eq!(
        time_core::ClockSkewPolicy::try_new(
            time_core::TimeNodeScope::SingleNode,
            time_core::TimeTrustClass::ExternalTrustedTimeSource,
            Some(1),
            time_core::PrecisionClass::IntegerExact,
            time_core::SamplingWindow::None,
            time_core::TrustedTimeSourceClass::LocalWallClock,
            time_core::ClockSkewImpact::Audit,
        ),
        Err(time_core::ClockSkewPolicyError::ExternalSourceClassRequired)
    );
    let policy = time_core::ClockSkewPolicy::try_new(
        time_core::TimeNodeScope::MultiNode,
        time_core::TimeTrustClass::MultiNodeBoundedSkew,
        Some(100),
        time_core::PrecisionClass::DeclaredPrecisionLabel("ms"),
        time_core::SamplingWindow::Milliseconds(5000),
        time_core::TrustedTimeSourceClass::ExternalTimeSourceObservation,
        time_core::ClockSkewImpact::Evidence,
    )
    .expect("bounded skew policy is explicit");
    assert_eq!(
        time_core::TimeSynchronizationDecision::try_new(
            StartupRunId::new(reference("startup-time-accepted")),
            Some(correlation()),
            policy,
            time_core::ObservedSkewClass::WithinPolicy,
            time_core::TimeSynchronizationOutcome::Accepted,
            Some(time_core::TimeSynchronizationFailureKind::ClockSkewExceeded),
        ),
        Err(time_core::TimeSynchronizationDecisionError::ReasonMustBeAbsentForAccepted)
    );
    let decision = time_core::TimeSynchronizationDecision::try_new(
        StartupRunId::new(reference("startup-time-untrusted")),
        None,
        policy,
        time_core::ObservedSkewClass::ObservationUnavailable,
        time_core::TimeSynchronizationOutcome::Untrusted,
        Some(time_core::TimeSynchronizationFailureKind::TimeSourceUntrusted),
    )
    .expect("non-success time sync decision carries reason");
    assert_eq!(decision.audit_event_type(), "time_synchronization_decision");

    for (kind, code) in [
        (
            time_core::TimeSynchronizationFailureKind::ClockSkewExceeded,
            "clock_skew_exceeded",
        ),
        (
            time_core::TimeSynchronizationFailureKind::TimeSourceUntrusted,
            "time_source_untrusted",
        ),
        (
            time_core::TimeSynchronizationFailureKind::TimeSyncUnavailable,
            "time_sync_unavailable",
        ),
        (
            time_core::TimeSynchronizationFailureKind::TimestampOrderUntrusted,
            "timestamp_order_untrusted",
        ),
        (
            time_core::TimeSynchronizationFailureKind::TimeObservationUnavailable,
            "time_observation_unavailable",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
    }
}

#[test]
fn ce2_cov2_transport_boundary_variants_and_failure_codes_are_closed() {
    let _transport_surface = transport_core::CoreTransportSurface;
    let session_id = SessionId::new(reference("transport-session-cov2"));
    let packet_id = PacketId::new(reference("transport-packet-cov2"));

    let command_kinds = [
        transport_core::TransportCommandKind::StartSession,
        transport_core::TransportCommandKind::ApplyLocalDescription,
        transport_core::TransportCommandKind::ApplyRemoteDescription,
        transport_core::TransportCommandKind::AddIceCandidate,
        transport_core::TransportCommandKind::ForwardPacket,
        transport_core::TransportCommandKind::CloseSession,
    ];
    assert_eq!(command_kinds.len(), 6);
    let event_kinds = [
        transport_core::TransportEventKind::SessionObserved,
        transport_core::TransportEventKind::LocalDescriptionAccepted,
        transport_core::TransportEventKind::RemoteDescriptionAccepted,
        transport_core::TransportEventKind::IceCandidateObserved,
        transport_core::TransportEventKind::PacketSemanticViewObserved,
        transport_core::TransportEventKind::TransportClosed,
        transport_core::TransportEventKind::ConversionFailed,
    ];
    assert_eq!(event_kinds.len(), 7);

    let capability = transport_core::TransportCapability::new("webrtc-transport");
    assert_eq!(capability.name(), "webrtc-transport");
    let session_description =
        transport_core::SessionDescriptionRef::new(session_id.clone(), "sdp-ref")
            .expect("semantic SDP reference is valid");
    let ice_candidate = transport_core::IceCandidateRef::new(session_id.clone(), "ice-ref")
        .expect("semantic ICE reference is valid");
    let packet_ref = transport_core::PacketSemanticViewRef::new(packet_id);
    assert_eq!(
        transport_core::SessionDescriptionRef::new(session_id.clone(), ""),
        Err(transport_core::TransportContractError::InvalidSemanticReference)
    );
    assert_eq!(
        transport_core::IceCandidateRef::new(session_id, "bad\ncandidate"),
        Err(transport_core::TransportContractError::InvalidSemanticReference)
    );

    let command = transport_core::TransportCommand::new(
        transport_core::TransportCommandKind::ApplyLocalDescription,
        transport_core::WebRtcTransportPayload::SessionDescription(session_description.clone()),
    );
    assert_eq!(
        command,
        transport_core::TransportCommand::new(
            transport_core::TransportCommandKind::ApplyLocalDescription,
            transport_core::WebRtcTransportPayload::SessionDescription(session_description.clone()),
        )
    );
    let event = transport_core::TransportEvent::new(
        transport_core::TransportEventKind::IceCandidateObserved,
        transport_core::WebRtcTransportObservation::IceCandidate(ice_candidate.clone()),
    );
    assert_eq!(
        event,
        transport_core::TransportEvent::new(
            transport_core::TransportEventKind::IceCandidateObserved,
            transport_core::WebRtcTransportObservation::IceCandidate(ice_candidate.clone()),
        )
    );

    let transport_failures = [
        (
            transport_core::TransportFailureKind::UnsupportedMediaContractVersion,
            "unsupported_media_contract_version",
        ),
        (
            transport_core::TransportFailureKind::ExternalDecodeFailed,
            "external_decode_failed",
        ),
        (
            transport_core::TransportFailureKind::ExternalEncodeFailed,
            "external_encode_failed",
        ),
        (
            transport_core::TransportFailureKind::MediaPayloadMappingInvalid,
            "media_payload_mapping_invalid",
        ),
    ];
    for (kind, code) in transport_failures {
        assert_eq!(kind.reason_code(), code);
    }

    let driver_failure_kinds = [
        transport_core::TransportDriverFailureKind::ExternalDecodeFailed,
        transport_core::TransportDriverFailureKind::ExternalEncodeFailed,
        transport_core::TransportDriverFailureKind::MediaPayloadMappingInvalid,
        transport_core::TransportDriverFailureKind::DriverShutdown,
    ];
    for kind in driver_failure_kinds {
        let failure = transport_core::TransportDriverFailure::from_kind(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            kind.reason_code()
        );
    }

    let payloads = [
        transport_core::WebRtcTransportPayload::Empty,
        transport_core::WebRtcTransportPayload::SessionDescription(session_description),
        transport_core::WebRtcTransportPayload::IceCandidate(ice_candidate.clone()),
        transport_core::WebRtcTransportPayload::PacketView(packet_ref.clone()),
        transport_core::WebRtcTransportPayload::Capability(capability),
    ];
    assert_eq!(payloads.len(), 5);
    let observations = [
        transport_core::WebRtcTransportObservation::Empty,
        transport_core::WebRtcTransportObservation::IceCandidate(ice_candidate),
        transport_core::WebRtcTransportObservation::PacketView(packet_ref),
        transport_core::WebRtcTransportObservation::ConversionFailure(
            transport_core::TransportDriverFailure::from_kind(
                transport_core::TransportDriverFailureKind::ExternalDecodeFailed,
            ),
        ),
    ];
    assert_eq!(observations.len(), 4);

    let negotiation_materials = [
        transport_core::NegotiationMaterialClass::Offer,
        transport_core::NegotiationMaterialClass::Answer,
        transport_core::NegotiationMaterialClass::IceCandidate,
    ];
    assert_eq!(negotiation_materials.len(), 3);
    let negotiation_flow = [
        transport_core::NegotiationFlowStep::ReceiveExternalMaterial,
        transport_core::NegotiationFlowStep::DriverDecodeAndValidate,
        transport_core::NegotiationFlowStep::MapToCoreReference,
        transport_core::NegotiationFlowStep::CoreSignalingEvaluation,
        transport_core::NegotiationFlowStep::CoreRelayDecision,
        transport_core::NegotiationFlowStep::ExternalProjection,
    ];
    assert_eq!(negotiation_flow.len(), 6);
    let negotiation_decisions = [
        transport_core::NegotiationDecisionKind::AcceptedRelay,
        transport_core::NegotiationDecisionKind::RejectedRelay,
        transport_core::NegotiationDecisionKind::ProtocolViolation,
    ];
    assert_eq!(negotiation_decisions.len(), 3);
    let negotiation_versions = [
        transport_core::NegotiationVersionSurface::SignalingCommandEvent,
        transport_core::NegotiationVersionSurface::MediaFacingTransportContract,
        transport_core::NegotiationVersionSurface::DriverWireEncoding,
        transport_core::NegotiationVersionSurface::SdkPublicApi,
    ];
    assert_eq!(negotiation_versions.len(), 4);
    let parity = [
        transport_core::SdkNegotiationParityRequirement::SameCommandEventSet,
        transport_core::SdkNegotiationParityRequirement::SameCorrelationPropagation,
        transport_core::SdkNegotiationParityRequirement::PreserveServerReason,
        transport_core::SdkNegotiationParityRequirement::SameVersionNegotiation,
        transport_core::SdkNegotiationParityRequirement::SignalingOnlyBoundary,
    ];
    assert_eq!(parity.len(), 5);

    for (kind, code) in [
        (
            transport_core::NegotiationFailureKind::ExternalDecodeFailed,
            "external_decode_failed",
        ),
        (
            transport_core::NegotiationFailureKind::IceCandidatePolicyViolation,
            "ice_candidate_policy_violation",
        ),
        (
            transport_core::NegotiationFailureKind::IceConnectivityCheckFailed,
            "ice_connectivity_check_failed",
        ),
        (
            transport_core::NegotiationFailureKind::IceConsentExpired,
            "ice_consent_expired",
        ),
        (
            transport_core::NegotiationFailureKind::MissingRequiredWireField,
            "missing_required_wire_field",
        ),
        (
            transport_core::NegotiationFailureKind::ExternalEnumUnmapped,
            "external_enum_unmapped",
        ),
        (
            transport_core::NegotiationFailureKind::UnsupportedCommandVersion,
            "unsupported_command_version",
        ),
        (
            transport_core::NegotiationFailureKind::UnsupportedMediaContractVersion,
            "unsupported_media_contract_version",
        ),
        (
            transport_core::NegotiationFailureKind::MediaCodecNotSupported,
            "media_codec_not_supported",
        ),
        (
            transport_core::NegotiationFailureKind::MediaPayloadMappingInvalid,
            "media_payload_mapping_invalid",
        ),
        (
            transport_core::NegotiationFailureKind::ParticipantNotJoined,
            "participant_not_joined",
        ),
        (
            transport_core::NegotiationFailureKind::CommandOrderViolation,
            "command_order_violation",
        ),
        (
            transport_core::NegotiationFailureKind::RoomDraining,
            "room_draining",
        ),
        (
            transport_core::NegotiationFailureKind::RoomClosed,
            "room_closed",
        ),
        (
            transport_core::NegotiationFailureKind::NetworkSendFailed,
            "network_send_failed",
        ),
        (
            transport_core::NegotiationFailureKind::NetworkReceiveFailed,
            "network_receive_failed",
        ),
        (
            transport_core::NegotiationFailureKind::DriverShutdown,
            "driver_shutdown",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
    }

    let ice_policy = transport_core::IceCandidatePolicy::new(
        vec![
            transport_core::IceCandidateConnectivityClass::HostCandidateRef,
            transport_core::IceCandidateConnectivityClass::SrflxCandidateRef,
            transport_core::IceCandidateConnectivityClass::RelayCandidateRef,
            transport_core::IceCandidateConnectivityClass::MdnsCandidateRef,
            transport_core::IceCandidateConnectivityClass::TrickleCandidateRef,
            transport_core::IceCandidateConnectivityClass::IceRestartIntent,
            transport_core::IceCandidateConnectivityClass::ConnectivityObservation,
            transport_core::IceCandidateConnectivityClass::ConsentFreshnessObservation,
        ],
        transport_core::IceAddressExposurePolicy::RawAddressRedactionRequired,
        false,
    );
    assert_eq!(ice_policy.accepted_classes().len(), 8);
    let exposure_policies = [
        transport_core::IceAddressExposurePolicy::RelayOnly,
        transport_core::IceAddressExposurePolicy::HostAllowed,
        transport_core::IceAddressExposurePolicy::SrflxAllowed,
        transport_core::IceAddressExposurePolicy::MdnsObfuscationRequired,
        transport_core::IceAddressExposurePolicy::RawAddressRedactionRequired,
    ];
    assert_eq!(exposure_policies.len(), 5);
    let observation_meanings = [
        transport_core::IceObservationMeaning::DiagnosticEvidenceOnly,
        transport_core::IceObservationMeaning::NotSignalingRelaySuccess,
        transport_core::IceObservationMeaning::NotCrossPlaneBinding,
    ];
    assert_eq!(observation_meanings.len(), 3);

    for (kind, code) in [
        (
            transport_core::IceFailureKind::IceCandidatePolicyViolation,
            "ice_candidate_policy_violation",
        ),
        (
            transport_core::IceFailureKind::IceCandidateMappingInvalid,
            "ice_candidate_mapping_invalid",
        ),
        (
            transport_core::IceFailureKind::IceCandidateRedactionRequired,
            "ice_candidate_redaction_required",
        ),
        (
            transport_core::IceFailureKind::IceGatheringFailed,
            "ice_gathering_failed",
        ),
        (
            transport_core::IceFailureKind::IceConnectivityCheckFailed,
            "ice_connectivity_check_failed",
        ),
        (
            transport_core::IceFailureKind::IceConsentExpired,
            "ice_consent_expired",
        ),
        (
            transport_core::IceFailureKind::IceRestartNotAllowed,
            "ice_restart_not_allowed",
        ),
        (
            transport_core::IceFailureKind::ExternalDecodeFailed,
            "external_decode_failed",
        ),
        (
            transport_core::IceFailureKind::CommandOrderViolation,
            "command_order_violation",
        ),
        (
            transport_core::IceFailureKind::NetworkReceiveFailed,
            "network_receive_failed",
        ),
        (
            transport_core::IceFailureKind::NetworkSendFailed,
            "network_send_failed",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
    }

    let secure_media_classes = [
        transport_core::SecureMediaSessionClass::SecureMediaRequired,
        transport_core::SecureMediaSessionClass::DtlsHandshakeObserved,
        transport_core::SecureMediaSessionClass::PeerVerificationObserved,
        transport_core::SecureMediaSessionClass::SrtpProtectionActive,
        transport_core::SecureMediaSessionClass::RekeyRequired,
        transport_core::SecureMediaSessionClass::SessionClosed,
    ];
    assert_eq!(secure_media_classes.len(), 6);
    let secure_policy = transport_core::SecureMediaPolicy::new("media-v1", "dtls-srtp", true, true);
    assert_eq!(secure_policy, secure_policy);
    let secure_evidence = [
        transport_core::SecureMediaEvidenceClass::Handshake,
        transport_core::SecureMediaEvidenceClass::PeerVerification,
        transport_core::SecureMediaEvidenceClass::ProtectionState,
        transport_core::SecureMediaEvidenceClass::PacketForwardingScope,
    ];
    assert_eq!(secure_evidence.len(), 4);

    for (kind, code) in [
        (
            transport_core::SecureMediaFailureKind::SecureMediaProfileNotSupported,
            "secure_media_profile_not_supported",
        ),
        (
            transport_core::SecureMediaFailureKind::SecureMediaHandshakeFailed,
            "secure_media_handshake_failed",
        ),
        (
            transport_core::SecureMediaFailureKind::SecureMediaPeerVerificationFailed,
            "secure_media_peer_verification_failed",
        ),
        (
            transport_core::SecureMediaFailureKind::SecureMediaProtectionNotActive,
            "secure_media_protection_not_active",
        ),
        (
            transport_core::SecureMediaFailureKind::SecureMediaKeyStateInvalid,
            "secure_media_key_state_invalid",
        ),
        (
            transport_core::SecureMediaFailureKind::SecureMediaSessionExpired,
            "secure_media_session_expired",
        ),
        (
            transport_core::SecureMediaFailureKind::SecureMediaRekeyRequired,
            "secure_media_rekey_required",
        ),
        (
            transport_core::SecureMediaFailureKind::SecretUnavailable,
            "secret_unavailable",
        ),
        (
            transport_core::SecureMediaFailureKind::SecretRotationStateUnavailable,
            "secret_rotation_state_unavailable",
        ),
        (
            transport_core::SecureMediaFailureKind::SecretKeyRevoked,
            "secret_key_revoked",
        ),
    ] {
        assert_eq!(kind.reason_code(), code);
    }
}
