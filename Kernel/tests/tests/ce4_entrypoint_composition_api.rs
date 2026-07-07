use arcrtc_core_configuration::ConfigurationOwner;
use arcrtc_entrypoint_admin::{
    AdminProbeClass, HealthAdminOutcome, OperatorAdminAllowedActionClass,
    OperatorAdminAuthorizationError, OperatorAdminAuthorizationGuard, OperatorAdminClass,
    OperatorAdminTargetScopeClass, OperatorCredentialContextSourceClass, ReadinessCompositionError,
    ReadinessCompositionGuard,
};
use arcrtc_entrypoint_configuration::{
    ConfigurationBundleValidationError, ConfigurationBundleValidationGuard,
    ConfigurationProfileClass, ConfigurationProfileEvidenceClaimClass,
    ConfigurationProfileEvidenceError, RuntimeConfigurationGenerationState,
    RuntimeReconfigurationAdmissionError, RuntimeReconfigurationAdmissionGuard,
    RuntimeReconfigurationAuditEventType, RuntimeReconfigurationClass,
    RuntimeReconfigurationTargetSurface,
};
use arcrtc_entrypoint_endpoints::{
    EdgeProxyClass, EdgeProxyHeaderSourceError, EdgeProxyHeaderSourceGuard,
    EdgeProxyTrustAdmissionError, EdgeProxyTrustAdmissionGuard, EndpointExposureScope,
    EndpointTargetContract, PublicEndpointAuditEventType, PublicEndpointClass,
    PublicEndpointDeclarationError, PublicEndpointDeclarationGuard, PublicEndpointProtocolClass,
    TrustedMetadataClass,
};
use arcrtc_entrypoint_internal_control::{
    InternalControlAuthorizationContextClass, InternalControlCommandEventType,
    InternalControlContractVersionState, InternalControlMessageClass, InternalControlPlaneClass,
    InternalControlPlaneContractError, InternalControlPlaneContractGuard,
    InternalServiceAuthorizationSequenceError, InternalServiceAuthorizationSequenceGuard,
    InternalServiceRole, InternalServiceTrustClass, InternalServiceTrustDecisionOutcome,
    InternalServiceTrustEvidenceError, InternalServiceTrustEvidenceGuard,
    ServiceIdentityProofReferenceClass,
};
use arcrtc_entrypoint_topology::{
    DeploymentTopologyClass, DiscoverySourceClass, EndpointFallbackPolicyError,
    EndpointFallbackPolicyGuard, ResolutionTargetService, ResolvedEndpointScope,
    ServiceDiscoveryResolutionAdmissionError, ServiceDiscoveryResolutionAdmissionGuard,
};
use arcrtc_roadmap_tests::{assert_not_contains, read_impl};

#[test]
fn ce4_configuration_bundle_validation_is_fail_closed_and_profile_scoped() {
    assert_eq!(
        ConfigurationBundleValidationGuard::try_new(
            ConfigurationProfileClass::ProductionCandidate,
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
        Err(ConfigurationBundleValidationError::EntrypointCompositionBundleMissing)
    );

    assert_eq!(
        ConfigurationProfileClass::TestDeterministic
            .adoption_rule()
            .admits_claim(
                ConfigurationProfileEvidenceClaimClass::ProductionEvidence,
                true
            ),
        Err(ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted)
    );
    assert_eq!(
        ConfigurationProfileClass::ProductionCandidate
            .adoption_rule()
            .admits_claim(ConfigurationProfileEvidenceClaimClass::RuntimeClaim, false),
        Err(ConfigurationProfileEvidenceError::ExplicitEvidenceReportReferenceMissing)
    );
}

#[test]
fn ce4_public_endpoint_declaration_rejects_protocol_target_and_scope_drift() {
    assert_eq!(
        PublicEndpointDeclarationGuard::try_new(
            PublicEndpointClass::SignalingPublic,
            EndpointExposureScope::PublicEdgeTerminated,
            PublicEndpointProtocolClass::Udp,
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
        ),
        Err(PublicEndpointDeclarationError::EndpointProtocolClassMismatch)
    );

    assert_eq!(
        PublicEndpointDeclarationGuard::try_new(
            PublicEndpointClass::TestOnlyEndpoint,
            EndpointExposureScope::DirectPublic,
            PublicEndpointProtocolClass::Http,
            EndpointTargetContract::TestDouble,
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
        ),
        Err(PublicEndpointDeclarationError::TestOnlyEndpointExposedOutsideLocalTest)
    );
}

#[test]
fn ce4_readiness_and_internal_control_do_not_collapse_to_boolean_success() {
    assert_eq!(
        ReadinessCompositionGuard::try_new(
            AdminProbeClass::CompositionReadiness,
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
            false,
            true,
            true,
            true,
            true,
        ),
        Err(ReadinessCompositionError::CatalogedReasonMissing)
    );
    assert_eq!(
        ReadinessCompositionGuard::try_new(
            AdminProbeClass::CompositionReadiness,
            HealthAdminOutcome::Satisfied,
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
            false,
            true,
        ),
        Err(ReadinessCompositionError::SingleUnqualifiedBooleanReadiness)
    );

    assert_eq!(
        InternalControlPlaneContractGuard::try_new(
            InternalControlPlaneClass::NetworkedPlaneCall,
            InternalServiceRole::Signaling,
            InternalServiceRole::Sfu,
            InternalControlMessageClass::Event,
            InternalControlCommandEventType::SignalingCommandContractReference,
            InternalControlContractVersionState::SupportedDeclared,
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
        Err(InternalControlPlaneContractError::CommandEventClassMismatch)
    );
}

#[test]
fn ce4_server_composition_sources_remain_core_driver_wiring_only() {
    for path in [
        "entrypoints/signaling-server/src/main.rs",
        "entrypoints/sfu-server/src/main.rs",
        "entrypoints/turn-server/src/main.rs",
    ] {
        let source = read_impl(path);
        assert!(
            source.contains("ResidentServerLoopConfig::new")
                && source.contains("serve_resident_loop(config)"),
            "{path} must construct resident loop config and delegate to composition root"
        );
        assert_not_contains(
            path,
            &source,
            &[
                "CompositionSurface",
                "WiringSet",
                "StartupFailureKind",
                "CompositionGuard",
                "arcrtc_core_",
                "arcrtc_driver_",
                "arcrtc_regulated",
                "arcrtc_sdk",
                "SdkSurface",
                "RegulatedSurface",
                "ReasonDefinition",
                "find_reason_definition",
            ],
        );
    }
}

#[test]
fn ce4_runtime_reconfiguration_requires_generation_scope_and_admitted_class() {
    assert_eq!(
        RuntimeReconfigurationAdmissionGuard::try_new(
            RuntimeReconfigurationClass::RuntimePolicyHotSwap,
            RuntimeReconfigurationTargetSurface::CorePolicy,
            ConfigurationOwner::Core,
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
        ),
        Err(RuntimeReconfigurationAdmissionError::ReconfigurationClassNotAdmitted)
    );
    assert_eq!(
        RuntimeReconfigurationAdmissionGuard::try_new(
            RuntimeReconfigurationClass::SecretRotationReload,
            RuntimeReconfigurationTargetSurface::SecurityMaterial,
            ConfigurationOwner::Core,
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
        ),
        Err(RuntimeReconfigurationAdmissionError::TargetOwnerMismatch)
    );
}

#[test]
fn ce4_edge_proxy_service_discovery_and_internal_trust_do_not_fail_open() {
    assert_eq!(
        EdgeProxyTrustAdmissionGuard::try_new(
            EdgeProxyClass::DirectPublicListener,
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
        ),
        Err(EdgeProxyTrustAdmissionError::MetadataClassNotAdmittedForEdgeClass)
    );
    assert_eq!(
        EdgeProxyHeaderSourceGuard::try_new(
            EdgeProxyClass::ReverseProxyHttpWs,
            TrustedMetadataClass::ForwardedFor,
            false,
            true,
            true,
            true,
            true,
        ),
        Err(EdgeProxyHeaderSourceError::ImmediateUpstreamUntrusted)
    );

    assert_eq!(
        ServiceDiscoveryResolutionAdmissionGuard::try_new(
            DiscoverySourceClass::ServiceRegistryLookup,
            DeploymentTopologyClass::SplitPlaneNetworked,
            ResolutionTargetService::InternalControl,
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
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(ServiceDiscoveryResolutionAdmissionError::ServiceIdentityTrustRelationMissing)
    );
    assert_eq!(
        EndpointFallbackPolicyGuard::try_new(true, true, false, true, true, true, true),
        Err(EndpointFallbackPolicyError::StaleEndpointRejectionRuleMissing)
    );

    assert_eq!(
        InternalServiceAuthorizationSequenceGuard::try_new(true, true, true, false, true, true),
        Err(InternalServiceAuthorizationSequenceError::InternalControlContractValidationMissing)
    );
    assert_eq!(
        InternalServiceTrustEvidenceGuard::try_new(
            true, true, true, true, false, true, true, true, true, true, true, true, true, true,
            true, true, true,
        ),
        Err(InternalServiceTrustEvidenceError::PeerVerificationClassMissing)
    );
    assert!(InternalServiceTrustClass::MtlsPeerIdentity
        .admits_proof_reference(ServiceIdentityProofReferenceClass::PeerCertificateReference));
    assert!(InternalServiceTrustDecisionOutcome::Rejected.requires_reason());
}

#[test]
fn ce4_operator_admin_authorization_rejects_missing_scope_and_domain_reuse() {
    assert_eq!(
        OperatorAdminAuthorizationGuard::try_new(
            OperatorAdminClass::OperatorAdminActionContext,
            OperatorCredentialContextSourceClass::ExternalIdentityProviderCredential,
            OperatorAdminAllowedActionClass::MaintenanceAction,
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
        ),
        Err(OperatorAdminAuthorizationError::ActionClassMismatch)
    );
    assert_eq!(
        OperatorAdminAuthorizationGuard::try_new(
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
            false,
            true,
            true,
            true,
            true,
        ),
        Err(OperatorAdminAuthorizationError::CommunicationAuthorizationReused)
    );
}
