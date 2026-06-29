use arcrtc_core_configuration::ConfigurationOwner;
use arcrtc_core_features::FeatureAdmissionFailureKind;
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_entrypoint_admin::{
    AdminMaintenanceActionClass, AdminMaintenanceCommandError, AdminMaintenanceCommandGuard,
    AdminProbeClass, HealthAdminAuditError, HealthAdminAuditEventType, HealthAdminAuditGuard,
    HealthAdminAuditOutcome, HealthAdminEvidenceError, HealthAdminEvidenceGuard,
    HealthAdminFailure, HealthAdminFailureKind, HealthAdminOutcome,
    OperatorAdminAllowedActionClass, OperatorAdminAuthorizationAuditError,
    OperatorAdminAuthorizationAuditEventType, OperatorAdminAuthorizationAuditGuard,
    OperatorAdminAuthorizationError, OperatorAdminAuthorizationEvidenceError,
    OperatorAdminAuthorizationEvidenceGuard, OperatorAdminAuthorizationFailure,
    OperatorAdminAuthorizationFailureKind, OperatorAdminAuthorizationGuard,
    OperatorAdminAuthorizationOutcome, OperatorAdminClass, OperatorAdminTargetScopeClass,
    OperatorCredentialContextSourceClass, ProhibitedHealthAdminBehavior,
    ProhibitedOperatorAdminAuthorizationBehavior, ReadinessCompositionError,
    ReadinessCompositionGuard,
};
use arcrtc_entrypoint_configuration::{
    CapabilityDeclarationError, CapabilityDeclarationGuard, CapabilityDeclarationGuardInput,
    CapabilityDeclarationSurface, ConfigurationBundleFailure, ConfigurationBundleFailureKind,
    ConfigurationBundleValidationError, ConfigurationBundleValidationGuard,
    ConfigurationProfileClass, ConfigurationProfileEvidenceClaimClass,
    ConfigurationProfileEvidenceError, ConfigurationProfileEvidenceGuard,
    ExperimentalLifecycleError, ExperimentalLifecycleGuard, ExperimentalLifecycleGuardInput,
    ExperimentalLifecycleStage, FeatureCapabilityAdmissionError, FeatureCapabilityAdmissionGuard,
    FeatureCapabilityAuthorityOwner, FeatureCapabilityFailure, FeatureCapabilityFailureKind,
    FeatureCapabilityUnsupportedVersionReason, FeatureFlagClass,
    ProhibitedConfigurationProfileBundleBehavior, ProhibitedFeatureCapabilityBehavior,
    ProhibitedRuntimeReconfigurationBehavior, RuntimeConfigurationGenerationState,
    RuntimeReconfigurationAdmissionError, RuntimeReconfigurationAdmissionGuard,
    RuntimeReconfigurationApplyError, RuntimeReconfigurationApplyGuard,
    RuntimeReconfigurationAuditEventType, RuntimeReconfigurationClass,
    RuntimeReconfigurationEvidenceError, RuntimeReconfigurationEvidenceGuard,
    RuntimeReconfigurationFailure, RuntimeReconfigurationFailureKind,
    RuntimeReconfigurationRollbackError, RuntimeReconfigurationRollbackGuard,
    RuntimeReconfigurationTargetSurface,
};
use arcrtc_entrypoint_endpoints::{
    ConnectionLifecycleState, ConnectionLifecycleTransitionError,
    ConnectionLifecycleTransitionGuard, EdgeProxyAuditEventType, EdgeProxyClass,
    EdgeProxyHeaderSourceError, EdgeProxyHeaderSourceGuard, EdgeProxyTrustAdmissionError,
    EdgeProxyTrustAdmissionGuard, EdgeProxyTrustEvidenceError, EdgeProxyTrustEvidenceGuard,
    EdgeProxyTrustFailure, EdgeProxyTrustFailureKind, EdgeTlsTerminationClass,
    EdgeTlsTerminationError, EdgeTlsTerminationGuard, EndpointExposureScope,
    EndpointTargetContract, ProhibitedEdgeProxyTrustBehavior, ProhibitedPublicEndpointBehavior,
    PublicEndpointAuditEventType, PublicEndpointClass, PublicEndpointDeclarationError,
    PublicEndpointDeclarationGuard, PublicEndpointEvidenceError, PublicEndpointEvidenceGuard,
    PublicEndpointFailure, PublicEndpointFailureKind, PublicEndpointProtocolClass,
    PublicInternalEndpointSeparationError, PublicInternalEndpointSeparationGuard,
    TrustedMetadataClass,
};
use arcrtc_entrypoint_internal_control::{
    InternalControlAuthorizationContextClass, InternalControlCommandEventType,
    InternalControlContractVersionState, InternalControlMessageClass,
    InternalControlPlaneAuditError, InternalControlPlaneAuditEventType,
    InternalControlPlaneAuditGuard, InternalControlPlaneClass, InternalControlPlaneContractError,
    InternalControlPlaneContractGuard, InternalControlPlaneEvidenceError,
    InternalControlPlaneEvidenceGuard, InternalControlPlaneFailure,
    InternalControlPlaneFailureKind, InternalControlPlaneOutcome,
    InternalServiceAuthorizationSequenceError, InternalServiceAuthorizationSequenceGuard,
    InternalServiceIdentityMappingError, InternalServiceIdentityMappingGuard, InternalServiceRole,
    InternalServiceTrustAuditError, InternalServiceTrustAuditEventType,
    InternalServiceTrustAuditGuard, InternalServiceTrustClass, InternalServiceTrustDecisionOutcome,
    InternalServiceTrustEvidenceError, InternalServiceTrustEvidenceGuard,
    InternalServiceTrustFailure, InternalServiceTrustFailureKind,
    ProhibitedInternalControlPlaneBehavior, ProhibitedInternalServiceTrustBehavior,
    ServiceIdentityProofReferenceClass,
};
use arcrtc_entrypoint_topology::{
    DeploymentTopologyAdmissionError, DeploymentTopologyAdmissionGuard,
    DeploymentTopologyAuditError, DeploymentTopologyAuditEventType, DeploymentTopologyAuditGuard,
    DeploymentTopologyClass, DeploymentTopologyEvidenceError, DeploymentTopologyEvidenceGuard,
    DeploymentTopologyFailure, DeploymentTopologyFailureKind, DiscoverySourceClass,
    EndpointFallbackPolicyError, EndpointFallbackPolicyGuard, EndpointResolutionState,
    NodeAffinityPolicyError, NodeAffinityPolicyGuard, NodeAffinityPolicyGuardInput,
    NodeLocalStateClass, ProhibitedDeploymentTopologyBehavior,
    ProhibitedServiceDiscoveryResolutionBehavior, ResolutionTargetService, ResolvedEndpointScope,
    ServiceDiscoveryAuditError, ServiceDiscoveryAuditEventType, ServiceDiscoveryAuditGuard,
    ServiceDiscoveryResolutionAdmissionError, ServiceDiscoveryResolutionAdmissionGuard,
    ServiceDiscoveryResolutionEvidenceError, ServiceDiscoveryResolutionEvidenceGuard,
    ServiceDiscoveryResolutionFailure, ServiceDiscoveryResolutionFailureKind,
    TopologyServiceDiscoveryRelationError, TopologyServiceDiscoveryRelationGuard,
};

fn cataloged(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("entrypoint failure reason must be cataloged")
}

#[test]
fn coverage_entrypoints_admin_guards_are_fail_closed_and_cataloged() {
    assert!(!HealthAdminOutcome::Satisfied.requires_reason());
    assert!(HealthAdminOutcome::NotSatisfied.requires_reason());
    assert!(HealthAdminAuditEventType::OperationalProbeObservation
        .admits_outcome(HealthAdminAuditOutcome::WithinBoundObserved));
    assert!(!HealthAdminAuditEventType::OperationalProbeObservation
        .admits_outcome(HealthAdminAuditOutcome::Accepted));
    assert!(OperatorAdminClass::DeveloperLocalContext.is_developer_local());
    assert!(
        OperatorAdminClass::EvidenceVerificationContext.admits_action_scope(
            OperatorAdminAllowedActionClass::EvidenceVerificationAction,
            OperatorAdminTargetScopeClass::EvidenceReportScope,
        )
    );

    let readiness_ok = [true; 19];
    assert!(readiness_guard(readiness_ok, HealthAdminOutcome::Satisfied).is_ok());
    for (index, expected) in [
        (0, ReadinessCompositionError::StartupRunIdMissing),
        (1, ReadinessCompositionError::EntrypointNameMissing),
        (2, ReadinessCompositionError::ProfileClassMissing),
        (3, ReadinessCompositionError::SelectedDriversMissing),
        (4, ReadinessCompositionError::TopologyNodeScopeMissing),
        (10, ReadinessCompositionError::ProbeClassMissing),
        (13, ReadinessCompositionError::OutcomeMissing),
        (15, ReadinessCompositionError::EvidenceClassMissing),
        (
            17,
            ReadinessCompositionError::SingleUnqualifiedBooleanReadiness,
        ),
        (
            18,
            ReadinessCompositionError::UnevaluatedComponentClaimedSatisfied,
        ),
    ] {
        let mut flags = readiness_ok;
        flags[index] = false;
        assert_eq!(
            readiness_guard(flags, HealthAdminOutcome::Satisfied),
            Err(expected)
        );
    }
    let mut missing_reason = readiness_ok;
    missing_reason[14] = false;
    assert_eq!(
        readiness_guard(missing_reason, HealthAdminOutcome::Failed),
        Err(ReadinessCompositionError::CatalogedReasonMissing)
    );

    assert!(AdminMaintenanceCommandGuard::try_new(
        AdminMaintenanceActionClass::RequestDrainShutdown,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        AdminMaintenanceCommandGuard::try_new(
            AdminMaintenanceActionClass::RequestDrainShutdown,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(AdminMaintenanceCommandError::AuthorizationMissing)
    );

    assert!(HealthAdminAuditGuard::try_new(
        HealthAdminAuditEventType::OperationalProbeObservation,
        HealthAdminAuditOutcome::WithinBoundObserved,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        HealthAdminAuditGuard::try_new(
            HealthAdminAuditEventType::AdminMaintenanceDecision,
            HealthAdminAuditOutcome::WithinBoundObserved,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(HealthAdminAuditError::OutcomeMismatch)
    );

    assert!(HealthAdminEvidenceGuard::try_new(
        AdminProbeClass::DriverDependencyReadiness,
        HealthAdminOutcome::NotSatisfied,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        HealthAdminEvidenceGuard::try_new(
            AdminProbeClass::DriverDependencyReadiness,
            HealthAdminOutcome::NotSatisfied,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
        ),
        Err(HealthAdminEvidenceError::CatalogedReasonMissing)
    );

    for kind in [
        HealthAdminFailureKind::ReadinessNotSatisfied,
        HealthAdminFailureKind::HealthProbeUnavailable,
        HealthAdminFailureKind::RuntimeTaskPanicDetected,
        HealthAdminFailureKind::InternalServiceCredentialExpired,
        HealthAdminFailureKind::InternalServiceTrustPolicyMissing,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = HealthAdminFailure::from_kind(kind);
    }

    assert!(OperatorAdminAuthorizationGuard::try_new(
        OperatorAdminClass::OperatorAdminActionContext,
        OperatorCredentialContextSourceClass::ExternalIdentityProviderCredential,
        OperatorAdminAllowedActionClass::AdminAction,
        OperatorAdminTargetScopeClass::ExplicitDomainScope,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        OperatorAdminAuthorizationGuard::try_new(
            OperatorAdminClass::DeveloperLocalContext,
            OperatorCredentialContextSourceClass::LocalDeveloperCredentialReference,
            OperatorAdminAllowedActionClass::DiagnosticProbeAction,
            OperatorAdminTargetScopeClass::EntrypointScope,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(OperatorAdminAuthorizationError::DeveloperLocalContextUsedAsProductionProof)
    );

    assert!(OperatorAdminAuthorizationAuditGuard::try_new(
        OperatorAdminAuthorizationAuditEventType::OperatorAdminAuthorizationDecision,
        OperatorAdminAuthorizationOutcome::Accepted,
        true,
        true,
        true,
        true,
        true,
        false,
    )
    .is_ok());
    assert_eq!(
        OperatorAdminAuthorizationAuditGuard::try_new(
            OperatorAdminAuthorizationAuditEventType::OperatorAdminAuthorizationDecision,
            OperatorAdminAuthorizationOutcome::Rejected,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(OperatorAdminAuthorizationAuditError::CatalogedReasonMissing)
    );

    assert!(OperatorAdminAuthorizationEvidenceGuard::try_new(
        OperatorAdminClass::OperatorProbeContext,
        OperatorAdminAuthorizationOutcome::Rejected,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        OperatorAdminAuthorizationEvidenceGuard::try_new(
            OperatorAdminClass::DeveloperLocalContext,
            OperatorAdminAuthorizationOutcome::Accepted,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        Err(OperatorAdminAuthorizationEvidenceError::CloseNotClaimedScopeMissing)
    );

    for kind in [
        OperatorAdminAuthorizationFailureKind::OperatorCredentialMissing,
        OperatorAdminAuthorizationFailureKind::OperatorAuthorizationContextExpired,
        OperatorAdminAuthorizationFailureKind::OperatorScopeNotAllowed,
        OperatorAdminAuthorizationFailureKind::RuntimeConfigInvalid,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = OperatorAdminAuthorizationFailure::from_kind(kind);
    }

    assert_eq!(
        format!(
            "{:?}",
            ProhibitedHealthAdminBehavior::ProbeResponseHidesFailedDependency
        ),
        "ProbeResponseHidesFailedDependency"
    );
    assert_eq!(
        format!(
            "{:?}",
            ProhibitedOperatorAdminAuthorizationBehavior::OperatorAuthorizationInferredFromEnvironment
        ),
        "OperatorAuthorizationInferredFromEnvironment"
    );
}

#[test]
fn coverage_entrypoints_configuration_guards_are_fail_closed_and_cataloged() {
    assert!(ConfigurationProfileClass::ProductionCandidate
        .adoption_rule()
        .admits_claim(
            ConfigurationProfileEvidenceClaimClass::ProductionEvidence,
            true
        )
        .is_ok());
    assert_eq!(
        ConfigurationProfileClass::ProductionCandidate
            .adoption_rule()
            .admits_claim(
                ConfigurationProfileEvidenceClaimClass::ProductionEvidence,
                false
            ),
        Err(ConfigurationProfileEvidenceError::ExplicitEvidenceReportReferenceMissing)
    );

    assert!(ConfigurationBundleValidationGuard::try_new(
        ConfigurationProfileClass::ProductionCandidate,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        ConfigurationBundleValidationGuard::try_new(
            ConfigurationProfileClass::ProductionCandidate,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(ConfigurationBundleValidationError::DeploymentTopologyMissing)
    );

    assert!(ConfigurationProfileEvidenceGuard::try_new(
        ConfigurationProfileClass::TestDeterministic,
        ConfigurationProfileEvidenceClaimClass::TestEvidence,
        true,
        true,
        false,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        ConfigurationProfileEvidenceGuard::try_new(
            ConfigurationProfileClass::TestDeterministic,
            ConfigurationProfileEvidenceClaimClass::TestEvidence,
            true,
            false,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(ConfigurationProfileEvidenceError::RawSecretMaterialInEvidence)
    );

    assert!(RuntimeReconfigurationClass::SecretRotationReload.admits_runtime_apply());
    assert!(!RuntimeReconfigurationClass::RuntimePolicyHotSwap.admits_runtime_apply());
    assert!(RuntimeReconfigurationTargetSurface::SecurityMaterial
        .owner_matches(ConfigurationOwner::Drivers));
    assert!(RuntimeConfigurationGenerationState::PendingGeneration.is_proposed_candidate());

    assert!(RuntimeReconfigurationAdmissionGuard::try_new(
        RuntimeReconfigurationClass::SecretRotationReload,
        RuntimeReconfigurationTargetSurface::SecurityMaterial,
        ConfigurationOwner::Drivers,
        RuntimeConfigurationGenerationState::CurrentGeneration,
        RuntimeConfigurationGenerationState::PendingGeneration,
        true,
        true,
        true,
        true,
        true,
        true,
        RuntimeReconfigurationAuditEventType::RuntimeReconfigurationDecision,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        RuntimeReconfigurationAdmissionGuard::try_new(
            RuntimeReconfigurationClass::TestProfileSwap,
            RuntimeReconfigurationTargetSurface::TestProfile,
            ConfigurationOwner::Entrypoints,
            RuntimeConfigurationGenerationState::CurrentGeneration,
            RuntimeConfigurationGenerationState::ValidatingGeneration,
            true,
            true,
            true,
            true,
            true,
            true,
            RuntimeReconfigurationAuditEventType::RuntimeReconfigurationDecision,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
        ),
        Err(RuntimeReconfigurationAdmissionError::TestProfileSwapEvidenceNotTestOnly)
    );

    assert_eq!(
        RuntimeReconfigurationApplyGuard::try_new(
            RuntimeReconfigurationTargetSurface::CorePolicy,
            true,
            true,
            false,
            true,
        ),
        Err(RuntimeReconfigurationApplyError::DrainOrRestartRuleMissing)
    );
    assert_eq!(
        RuntimeReconfigurationRollbackGuard::try_new(
            RuntimeConfigurationGenerationState::CurrentGeneration,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(RuntimeReconfigurationRollbackError::RollbackGenerationStateInvalid)
    );
    assert_eq!(
        RuntimeReconfigurationEvidenceGuard::try_new(
            true, true, true, true, true, true, true, true, true, true, true, false,
        ),
        Err(RuntimeReconfigurationEvidenceError::StartupEvidenceUsedAsRuntimeReconfigurationEvidence)
    );

    assert!(
        CapabilityDeclarationGuard::try_new(CapabilityDeclarationGuardInput {
            surface: CapabilityDeclarationSurface::FeatureFlagValue,
            configuration_wiring_owner: ConfigurationOwner::Entrypoints,
            authority_owner: FeatureCapabilityAuthorityOwner::EntrypointsConfig,
            accepted_contract_version_declared: true,
            optional_behavior_declared: true,
            fallback_when_absent_declared: true,
            required_absent_reason_declared: true,
            sdk_parity_requirement_declared_when_client_visible: true,
            evidence_class_required_before_adoption_declared: true,
        })
        .is_ok()
    );
    assert_eq!(
        CapabilityDeclarationGuard::try_new(CapabilityDeclarationGuardInput {
            surface: CapabilityDeclarationSurface::CoreProtocol,
            configuration_wiring_owner: ConfigurationOwner::Entrypoints,
            authority_owner: FeatureCapabilityAuthorityOwner::EntrypointsConfig,
            accepted_contract_version_declared: true,
            optional_behavior_declared: true,
            fallback_when_absent_declared: true,
            required_absent_reason_declared: true,
            sdk_parity_requirement_declared_when_client_visible: true,
            evidence_class_required_before_adoption_declared: true,
        }),
        Err(CapabilityDeclarationError::SurfaceAuthorityOwnerMismatch)
    );

    assert_eq!(
        FeatureCapabilityAdmissionGuard::try_new(
            FeatureFlagClass::TestOnlyGate,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
        ),
        Err(FeatureCapabilityAdmissionError::TestOnlyGateUsedOutsideTestEvidence)
    );
    assert_eq!(
        ExperimentalLifecycleGuard::try_new(ExperimentalLifecycleGuardInput {
            stage: ExperimentalLifecycleStage::AdoptedContract,
            scope_and_owner_fixed: true,
            explicit_gate_present_when_scaffold_or_later: true,
            dependency_direction_evidence_present_when_scaffold: true,
            unit_or_contract_evidence_present_when_implemented: true,
            integration_report_present_when_controlled_integration: true,
            adr_or_canonical_update_and_compatibility_rule_present_when_adopted: false,
            compatibility_or_deprecation_lifecycle_satisfied_when_removed: true,
        }),
        Err(ExperimentalLifecycleError::AdoptionCanonicalOrCompatibilityRuleMissing)
    );

    for kind in [
        ConfigurationBundleFailureKind::RuntimeConfigMissing,
        ConfigurationBundleFailureKind::ServiceDiscoverySourceNotAdmitted,
        ConfigurationBundleFailureKind::RuntimeTaskPanicDetected,
        ConfigurationBundleFailureKind::CapabilityNotEnabled,
        ConfigurationBundleFailureKind::RuntimeReconfigurationNotAllowed,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = ConfigurationBundleFailure::from_kind(kind);
    }
    for kind in [
        RuntimeReconfigurationFailureKind::ConfigurationGenerationMissing,
        RuntimeReconfigurationFailureKind::RuntimeReconfigurationValidationFailed,
        RuntimeReconfigurationFailureKind::RuntimeReconfigurationRollbackFailed,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = RuntimeReconfigurationFailure::from_kind(kind);
    }
    for kind in [
        FeatureCapabilityFailureKind::CapabilityNotEnabled,
        FeatureCapabilityFailureKind::FeatureAdmissionFailure(
            FeatureAdmissionFailureKind::DataChannelNotSupported,
        ),
        FeatureCapabilityFailureKind::RuntimeReconfigurationNotAllowed,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = FeatureCapabilityFailure::from_kind(kind);
    }
    for unsupported in [
        FeatureCapabilityUnsupportedVersionReason::UnsupportedCommandVersion,
        FeatureCapabilityUnsupportedVersionReason::PublicEndpointVersionUnsupported,
        FeatureCapabilityUnsupportedVersionReason::TokenUnsupportedAlgorithm,
    ] {
        assert_eq!(
            cataloged(unsupported.reason_code())
                .definition()
                .code()
                .as_str(),
            unsupported.reason_code()
        );
        let _ = FeatureCapabilityFailure::from_unsupported_version_reason(unsupported);
    }

    assert_eq!(
        format!(
            "{:?}",
            ProhibitedConfigurationProfileBundleBehavior::MissingRequiredBundleFallsBackToDefault
        ),
        "MissingRequiredBundleFallsBackToDefault"
    );
    assert_eq!(
        format!(
            "{:?}",
            ProhibitedRuntimeReconfigurationBehavior::RollbackErasesFailedApplyEvent
        ),
        "RollbackErasesFailedApplyEvent"
    );
    assert_eq!(
        format!(
            "{:?}",
            ProhibitedFeatureCapabilityBehavior::ExperimentalSurfaceEnabledByDefault
        ),
        "ExperimentalSurfaceEnabledByDefault"
    );
}

#[test]
fn coverage_entrypoints_endpoints_guards_are_fail_closed_and_cataloged() {
    assert!(PublicEndpointClass::SignalingPublic.is_public_surface());
    assert!(PublicEndpointClass::TestOnlyEndpoint.is_test_only());
    assert!(PublicEndpointClass::AdminPrivate.is_private_control_or_admin());
    assert!(PublicEndpointClass::TurnPublicRelay
        .admits_protocol_class(PublicEndpointProtocolClass::TlsTcp));
    assert!(
        !PublicEndpointClass::SfuMediaPublic.admits_target_contract(EndpointTargetContract::Turn)
    );
    assert_eq!(
        PublicEndpointAuditEventType::PublicEndpointConnectionDecision.event_type(),
        "public_endpoint_connection_decision"
    );

    assert!(PublicEndpointDeclarationGuard::try_new(
        PublicEndpointClass::SignalingPublic,
        EndpointExposureScope::DirectPublic,
        PublicEndpointProtocolClass::WebSocket,
        EndpointTargetContract::Signaling,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        PublicEndpointAuditEventType::PublicEndpointConnectionDecision,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        PublicEndpointDeclarationGuard::try_new(
            PublicEndpointClass::SignalingPublic,
            EndpointExposureScope::DirectPublic,
            PublicEndpointProtocolClass::WebSocket,
            EndpointTargetContract::Signaling,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            PublicEndpointAuditEventType::PublicEndpointConnectionDecision,
            true,
            true,
        ),
        Err(PublicEndpointDeclarationError::TransportSecurityProfileMissing)
    );

    assert!(ConnectionLifecycleState::Active.is_active_traffic_state());
    assert!(ConnectionLifecycleState::Failed.requires_close_or_failure_reason());
    assert_eq!(
        ConnectionLifecycleTransitionGuard::try_new(
            ConnectionLifecycleState::Active,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(ConnectionLifecycleTransitionError::SecurityCheckMissing)
    );
    assert_eq!(
        ConnectionLifecycleTransitionGuard::try_new(
            ConnectionLifecycleState::ClosedSuccess,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(ConnectionLifecycleTransitionError::CloseSuccessAuditReferenceMissing)
    );
    assert_eq!(
        PublicInternalEndpointSeparationGuard::try_new(
            PublicEndpointClass::AdminPrivate,
            false,
            true,
            true,
            true,
        ),
        Err(PublicInternalEndpointSeparationError::PrivateEndpointPublicByDefault)
    );

    assert_eq!(
        PublicEndpointEvidenceGuard::try_new(
            true, true, true, true, true, true, true, true, true, false,
        ),
        Err(PublicEndpointEvidenceError::ListenerStartupUsedAsReadinessEvidence)
    );
    for kind in [
        PublicEndpointFailureKind::PublicEndpointNotAllowed,
        PublicEndpointFailureKind::PublicEndpointUpgradeFailed,
        PublicEndpointFailureKind::ConnectionClosePolicyViolation,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = PublicEndpointFailure::from_kind(kind);
    }

    assert!(EdgeProxyClass::ReverseProxyHttpWs
        .admits_metadata_class(TrustedMetadataClass::ForwardedFor));
    assert!(TrustedMetadataClass::HostHeader.raw_value_must_not_be_core_identity());
    assert_eq!(
        EdgeProxyAuditEventType::EdgeProxyTrustDecision.event_type(),
        "edge_proxy_trust_decision"
    );
    assert!(EdgeProxyTrustAdmissionGuard::try_new(
        EdgeProxyClass::ReverseProxyHttpWs,
        TrustedMetadataClass::ForwardedFor,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        EdgeProxyTrustAdmissionGuard::try_new(
            EdgeProxyClass::TestEdgeSimulator,
            TrustedMetadataClass::ForwardedFor,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(EdgeProxyTrustAdmissionError::TestEdgeUsedOutsideTestEvidence)
    );
    assert_eq!(
        EdgeProxyHeaderSourceGuard::try_new(
            EdgeProxyClass::DirectPublicListener,
            TrustedMetadataClass::ForwardedFor,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(EdgeProxyHeaderSourceError::MetadataClassNotAdmittedForEdgeClass)
    );
    assert_eq!(
        EdgeTlsTerminationGuard::try_new(
            EdgeTlsTerminationClass::TerminationNotAdmitted,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(EdgeTlsTerminationError::TerminationClassNotAdmitted)
    );
    assert_eq!(
        EdgeProxyTrustEvidenceGuard::try_new(
            true, true, true, true, true, true, true, true, true, true, true, true, false,
        ),
        Err(EdgeProxyTrustEvidenceError::DiagnosticSnippetUsedAsEvidence)
    );
    for kind in [
        EdgeProxyTrustFailureKind::EdgeProxyNotAdmitted,
        EdgeProxyTrustFailureKind::ForwardedHeaderChainInvalid,
        EdgeProxyTrustFailureKind::PublicInternalRouteConfusion,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = EdgeProxyTrustFailure::from_kind(kind);
    }

    assert_eq!(
        format!(
            "{:?}",
            ProhibitedPublicEndpointBehavior::ListenerBindTreatedAsDomainSuccess
        ),
        "ListenerBindTreatedAsDomainSuccess"
    );
    assert_eq!(
        format!(
            "{:?}",
            ProhibitedEdgeProxyTrustBehavior::RawEdgeMetadataBecomesCoreIdentity
        ),
        "RawEdgeMetadataBecomesCoreIdentity"
    );
}

#[test]
fn coverage_entrypoints_internal_control_guards_are_fail_closed_and_cataloged() {
    assert!(InternalServiceTrustClass::MtlsPeerIdentity.is_credential_bearing());
    assert!(InternalServiceTrustClass::TestServiceIdentity.is_test_only());
    assert!(
        InternalServiceTrustClass::UnauthenticatedInternalServiceRequested
            .is_rejected_request_class()
    );
    assert!(InternalServiceTrustClass::SignedServiceTokenIdentity
        .admits_proof_reference(ServiceIdentityProofReferenceClass::SignedServiceTokenReference));
    assert_eq!(
        InternalServiceTrustAuditEventType::InternalServiceTrustDecision.event_type(),
        "internal_service_trust_decision"
    );
    assert!(InternalServiceTrustDecisionOutcome::Expired.requires_reason());

    assert!(InternalServiceIdentityMappingGuard::try_new(
        InternalServiceTrustClass::MtlsPeerIdentity,
        ServiceIdentityProofReferenceClass::PeerCertificateReference,
        InternalServiceRole::Signaling,
        InternalServiceRole::Sfu,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        InternalServiceIdentityMappingGuard::try_new(
            InternalServiceTrustClass::UnauthenticatedInternalServiceRequested,
            ServiceIdentityProofReferenceClass::NoCredentialInProcess,
            InternalServiceRole::Signaling,
            InternalServiceRole::Sfu,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(InternalServiceIdentityMappingError::UnauthenticatedInternalServiceNotAdmitted)
    );

    assert_eq!(
        InternalServiceAuthorizationSequenceGuard::try_new(true, true, true, true, false, true),
        Err(InternalServiceAuthorizationSequenceError::DomainDecisionOwnershipLost)
    );
    assert_eq!(
        InternalServiceTrustAuditGuard::try_new(
            InternalServiceTrustAuditEventType::InternalServiceTrustDecision,
            InternalServiceTrustDecisionOutcome::Failed,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(InternalServiceTrustAuditError::CatalogedReasonMissing)
    );
    assert_eq!(
        InternalServiceTrustEvidenceGuard::try_new(
            true, true, true, true, true, true, true, true, true, true, true, true, true, false,
            true, true, true,
        ),
        Err(InternalServiceTrustEvidenceError::ActualOutcomeMissing)
    );
    for kind in [
        InternalServiceTrustFailureKind::InternalServiceIdentitySourceNotAdmitted,
        InternalServiceTrustFailureKind::InternalServicePeerVerificationFailed,
        InternalServiceTrustFailureKind::InternalControlAuthorizationDenied,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = InternalServiceTrustFailure::from_kind(kind);
    }

    assert!(InternalControlPlaneClass::NetworkedPlaneCall.requires_service_discovery_relation());
    assert!(InternalControlCommandEventType::SfuCommandContractReference
        .admits_message_class(InternalControlMessageClass::Command));
    assert!(InternalControlContractVersionState::SupportedDeclared.is_supported());
    assert!(
        InternalControlAuthorizationContextClass::MissingAuthorizationContextRequested
            .is_missing_requested()
    );
    assert_eq!(
        InternalControlPlaneContractGuard::try_new(
            InternalControlPlaneClass::NetworkedPlaneCall,
            InternalServiceRole::Signaling,
            InternalServiceRole::Sfu,
            InternalControlMessageClass::Command,
            InternalControlCommandEventType::SfuCommandContractReference,
            InternalControlContractVersionState::UnsupportedRequested,
            InternalControlAuthorizationContextClass::ServiceAuthorizationContext,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(InternalControlPlaneContractError::ContractVersionUnsupported)
    );
    assert_eq!(
        InternalControlPlaneAuditEventType::InternalControlPlaneDecision.event_type(),
        "internal_control_plane_decision"
    );
    assert_eq!(
        InternalControlPlaneAuditGuard::try_new(
            InternalControlPlaneAuditEventType::InternalControlPlaneDecision,
            InternalControlPlaneOutcome::Rejected,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(InternalControlPlaneAuditError::CatalogedReasonMissing)
    );
    assert_eq!(
        InternalControlPlaneEvidenceGuard::try_new(
            InternalControlPlaneClass::NodeAffinityPlaneCall,
            InternalControlPlaneOutcome::Success,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            false,
            true,
            true,
        ),
        Err(InternalControlPlaneEvidenceError::DistributedStateOwnerScopeMissing)
    );
    for kind in [
        InternalControlPlaneFailureKind::InternalControlMessageInvalid,
        InternalControlPlaneFailureKind::InternalServiceTrustFailure(
            InternalServiceTrustFailureKind::InternalServiceIdentityMissing,
        ),
        InternalControlPlaneFailureKind::DriverShutdown,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = InternalControlPlaneFailure::from_kind(kind);
    }

    assert_eq!(
        format!(
            "{:?}",
            ProhibitedInternalServiceTrustBehavior::RawServiceCredentialMaterialEscapes
        ),
        "RawServiceCredentialMaterialEscapes"
    );
    assert_eq!(
        format!(
            "{:?}",
            ProhibitedInternalControlPlaneBehavior::InternalRpcStatusBecomesCoreReason
        ),
        "InternalRpcStatusBecomesCoreReason"
    );
}

#[test]
fn coverage_entrypoints_topology_guards_are_fail_closed_and_cataloged() {
    assert!(
        DeploymentTopologyClass::SplitPlaneNetworked.requires_networked_internal_service_relation()
    );
    assert!(DeploymentTopologyClass::MultiNodeExperimental
        .requires_experimental_admission_for_production_claim());
    assert_eq!(
        DeploymentTopologyAuditEventType::DeploymentTopologyDecision.event_type(),
        "deployment_topology_decision"
    );
    assert_eq!(
        DeploymentTopologyAdmissionGuard::try_new(
            DeploymentTopologyClass::SplitPlaneNetworked,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(DeploymentTopologyAdmissionError::ServiceEndpointFailureMappingMissing)
    );
    assert_eq!(
        DeploymentTopologyAuditGuard::try_new(
            DeploymentTopologyAuditEventType::DeploymentTopologyDecision,
            true,
            true,
            true,
            false,
        ),
        Err(DeploymentTopologyAuditError::CorrelationRuleMissing)
    );
    assert_eq!(
        NodeAffinityPolicyGuard::try_new(NodeAffinityPolicyGuardInput {
            state_class: NodeLocalStateClass::SfuEndpointRouteState,
            affinity_key_declared: true,
            owning_node_scope_declared: true,
            failover_behavior_declared: true,
            unavailable_node_reason_declared: true,
            recovery_replay_relation_declared: true,
            evidence_class_declared: true,
            wrong_node_access_fails_closed_without_distributed_state_policy: false,
        }),
        Err(NodeAffinityPolicyError::WrongNodeAccessNotFailClosed)
    );
    assert_eq!(
        TopologyServiceDiscoveryRelationGuard::try_new(true, true, true, true, false, true, true),
        Err(TopologyServiceDiscoveryRelationError::ServiceDiscoveryProvesFailoverRecovery)
    );
    assert_eq!(
        DeploymentTopologyEvidenceGuard::try_new(
            true, true, true, true, true, true, true, true, true, true, true, true, true, false,
        ),
        Err(DeploymentTopologyEvidenceError::SingleNodeEvidenceUsedAsMultiNodeProof)
    );
    for kind in [
        DeploymentTopologyFailureKind::DeploymentTopologyUnsupported,
        DeploymentTopologyFailureKind::InternalServiceTrustPolicyMissing,
        DeploymentTopologyFailureKind::CrossNodeRouteNotAllowed,
        DeploymentTopologyFailureKind::DriverShutdown,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = DeploymentTopologyFailure::from_kind(kind);
    }

    assert!(DiscoverySourceClass::DnsResolution
        .admits_topology(DeploymentTopologyClass::SplitPlaneNetworked));
    assert!(DiscoverySourceClass::ServiceRegistryLookup.requires_registry_contract());
    assert!(DiscoverySourceClass::ServiceMeshResolution.is_mesh_resolution());
    assert!(EndpointResolutionState::ResolutionFailed.requires_reason());
    assert!(ResolvedEndpointScope::PublicEndpoint.requires_public_endpoint_admission());
    assert!(ResolutionTargetService::InternalControl.requires_internal_service_trust_relation());
    assert_eq!(
        ServiceDiscoveryAuditEventType::ServiceDiscoveryResolutionDecision.event_type(),
        "service_discovery_resolution_decision"
    );
    assert_eq!(
        ServiceDiscoveryResolutionAdmissionGuard::try_new(
            DiscoverySourceClass::ServiceRegistryLookup,
            DeploymentTopologyClass::SplitPlaneNetworked,
            ResolutionTargetService::Sfu,
            ResolvedEndpointScope::InternalService,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
        ),
        Err(ServiceDiscoveryResolutionAdmissionError::RegistryContractMissing)
    );
    assert_eq!(
        EndpointFallbackPolicyGuard::try_new(true, true, true, true, true, false, true),
        Err(EndpointFallbackPolicyError::EvidenceLimitationMissing)
    );
    assert_eq!(
        ServiceDiscoveryAuditGuard::try_new(
            ServiceDiscoveryAuditEventType::ServiceDiscoveryResolutionDecision,
            EndpointResolutionState::ResolutionRejected,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(ServiceDiscoveryAuditError::CatalogedReasonMissing)
    );
    assert_eq!(
        ServiceDiscoveryResolutionEvidenceGuard::try_new(
            true, true, true, true, true, true, true, true, true, true, true, true, true, true,
            false,
        ),
        Err(ServiceDiscoveryResolutionEvidenceError::DiagnosticOutputUsedAsEvidence)
    );
    for kind in [
        ServiceDiscoveryResolutionFailureKind::ServiceDiscoverySourceNotAdmitted,
        ServiceDiscoveryResolutionFailureKind::ServiceEndpointContractMismatch,
        ServiceDiscoveryResolutionFailureKind::InternalServiceIdentityUntrusted,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = ServiceDiscoveryResolutionFailure::from_kind(kind);
    }

    assert_eq!(
        format!(
            "{:?}",
            ProhibitedDeploymentTopologyBehavior::NodeLocalStateTreatedAsClusterGlobal
        ),
        "NodeLocalStateTreatedAsClusterGlobal"
    );
    assert_eq!(
        format!(
            "{:?}",
            ProhibitedServiceDiscoveryResolutionBehavior::FallbackEndpointUsedWithoutPolicyAndAuditReason
        ),
        "FallbackEndpointUsedWithoutPolicyAndAuditReason"
    );
}

fn readiness_guard(
    flags: [bool; 19],
    outcome: HealthAdminOutcome,
) -> Result<ReadinessCompositionGuard, ReadinessCompositionError> {
    ReadinessCompositionGuard::try_new(
        AdminProbeClass::CompositionReadiness,
        outcome,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
        flags[5],
        flags[6],
        flags[7],
        flags[8],
        flags[9],
        flags[10],
        flags[11],
        flags[12],
        flags[13],
        flags[14],
        flags[15],
        flags[16],
        flags[17],
        flags[18],
    )
}
