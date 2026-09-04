#[allow(dead_code)]
#[path = "../../entrypoints/cli/src/main.rs"]
mod cli_main;
#[allow(dead_code)]
#[path = "../../entrypoints/demo/src/main.rs"]
mod demo_main;

use arcrtc_core_command::CoreCommandSurface;
use arcrtc_core_features::FeatureAdmissionFailureKind;
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_entrypoint_admin as admin;
use arcrtc_entrypoint_composition_root::{
    run_resident_loop, ResidentDriverBindingRef, ResidentServerKind, ResidentServerLoopConfig,
    RuntimeProfileObservationRef, ShutdownDrainObservationRef,
};
use arcrtc_entrypoint_configuration as configuration;
use arcrtc_entrypoint_endpoints as endpoints;
use arcrtc_entrypoint_internal_control as internal_control;
use arcrtc_entrypoint_topology as topology;

fn assert_cataloged(code: &str) {
    assert_eq!(
        CatalogedReasonRef::from_code(code)
            .expect("reason code must be cataloged")
            .definition()
            .code()
            .as_str(),
        code
    );
}

#[test]
fn binary_composition_roots_expose_wiring_without_domain_authority() {
    // entrypoints binary は wiring root に閉じ、domain decision は core surface 側へ接続します。
    let _cli_surface = cli_main::CliCompositionSurface;
    for command_class in [
        cli_main::CliCommandClass::Signaling,
        cli_main::CliCommandClass::Sfu,
        cli_main::CliCommandClass::Turn,
        cli_main::CliCommandClass::AdminMaintenance,
        cli_main::CliCommandClass::DeveloperInspection,
    ] {
        let _ = cli_main::CliCommandWiring::new(CoreCommandSurface, command_class);
    }
    assert!(cli_main::CliCommandClass::AdminMaintenance.requires_operator_authorization());
    assert!(!cli_main::CliCommandClass::DeveloperInspection.requires_operator_authorization());

    let _demo_surface = demo_main::DemoCompositionSurface;
    for scenario_class in [
        demo_main::DemoScenarioClass::SignalingOnly,
        demo_main::DemoScenarioClass::SfuComposition,
        demo_main::DemoScenarioClass::TurnComposition,
        demo_main::DemoScenarioClass::DeveloperInspection,
    ] {
        let _ = demo_main::DemoCommandWiring::new(CoreCommandSurface, scenario_class);
    }

    for (server_kind, drivers, runtime_profile, shutdown_ref, supervision_ref) in [
        (
            ResidentServerKind::Signaling,
            vec![
                ResidentDriverBindingRef::Network,
                ResidentDriverBindingRef::Persistence,
                ResidentDriverBindingRef::Observability,
                ResidentDriverBindingRef::Security,
            ],
            "runtime-profile:signaling",
            "shutdown:signaling",
            "supervision:signaling",
        ),
        (
            ResidentServerKind::Turn,
            vec![
                ResidentDriverBindingRef::Network,
                ResidentDriverBindingRef::TurnRelay,
                ResidentDriverBindingRef::Persistence,
                ResidentDriverBindingRef::Observability,
                ResidentDriverBindingRef::Security,
            ],
            "runtime-profile:turn",
            "shutdown:turn",
            "supervision:turn",
        ),
        (
            ResidentServerKind::Sfu,
            vec![
                ResidentDriverBindingRef::Network,
                ResidentDriverBindingRef::SfuTransport,
                ResidentDriverBindingRef::Persistence,
                ResidentDriverBindingRef::Observability,
            ],
            "runtime-profile:sfu",
            "shutdown:sfu",
            "supervision:sfu",
        ),
    ] {
        let config = ResidentServerLoopConfig::new(
            server_kind,
            RuntimeProfileObservationRef::new(runtime_profile),
            drivers,
            ShutdownDrainObservationRef::new(shutdown_ref),
            ShutdownDrainObservationRef::new(supervision_ref),
        );
        assert_eq!(config.server_kind(), server_kind);
        assert_eq!(
            run_resident_loop(config).unwrap().server_kind(),
            server_kind
        );
    }
}

#[test]
fn admin_failure_vocabularies_are_fully_cataloged() {
    assert_eq!(
        admin::HealthAdminAuditEventType::OperationalProbeObservation.event_type(),
        "operational_probe_observation"
    );
    assert_eq!(
        admin::HealthAdminAuditEventType::AdminMaintenanceDecision.event_type(),
        "admin_maintenance_decision"
    );
    assert_eq!(
        admin::OperatorAdminAuthorizationAuditEventType::OperatorAdminAuthorizationDecision
            .event_type(),
        "operator_admin_authorization_decision"
    );

    for kind in [
        admin::HealthAdminFailureKind::ReadinessNotSatisfied,
        admin::HealthAdminFailureKind::HealthProbeUnavailable,
        admin::HealthAdminFailureKind::MaintenanceModeActive,
        admin::HealthAdminFailureKind::AdminActionNotAllowed,
        admin::HealthAdminFailureKind::OperatorActionDenied,
        admin::HealthAdminFailureKind::OperatorAuthorizationContextMissing,
        admin::HealthAdminFailureKind::RuntimeConfigMissing,
        admin::HealthAdminFailureKind::RuntimeConfigInvalid,
        admin::HealthAdminFailureKind::DriverShutdown,
        admin::HealthAdminFailureKind::ServiceDiscoveryUnavailable,
        admin::HealthAdminFailureKind::ServiceEndpointStale,
        admin::HealthAdminFailureKind::ServiceEndpointFallbackNotAllowed,
        admin::HealthAdminFailureKind::NodeStateUnavailable,
        admin::HealthAdminFailureKind::FailoverNotProven,
        admin::HealthAdminFailureKind::RuntimeTaskClassNotAdmitted,
        admin::HealthAdminFailureKind::RuntimeTaskOwnerViolation,
        admin::HealthAdminFailureKind::RuntimeTaskSupervisionMissing,
        admin::HealthAdminFailureKind::RuntimeTaskDetachedNotAllowed,
        admin::HealthAdminFailureKind::RuntimeTaskSpawnFailed,
        admin::HealthAdminFailureKind::RuntimeTaskJoinFailed,
        admin::HealthAdminFailureKind::RuntimeTaskCancelFailed,
        admin::HealthAdminFailureKind::RuntimeTaskPanicDetected,
        admin::HealthAdminFailureKind::RuntimeTaskQueueBoundExceeded,
        admin::HealthAdminFailureKind::InternalServiceIdentitySourceNotAdmitted,
        admin::HealthAdminFailureKind::InternalServiceIdentityMissing,
        admin::HealthAdminFailureKind::InternalServiceIdentityInvalid,
        admin::HealthAdminFailureKind::InternalServiceIdentityUntrusted,
        admin::HealthAdminFailureKind::InternalServiceIdentityScopeConflict,
        admin::HealthAdminFailureKind::InternalServicePeerVerificationFailed,
        admin::HealthAdminFailureKind::InternalServiceCredentialExpired,
        admin::HealthAdminFailureKind::InternalServiceTrustPolicyMissing,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = admin::HealthAdminFailure::from_kind(kind);
    }

    for kind in [
        admin::OperatorAdminAuthorizationFailureKind::OperatorCredentialMissing,
        admin::OperatorAdminAuthorizationFailureKind::OperatorCredentialInvalid,
        admin::OperatorAdminAuthorizationFailureKind::OperatorAuthorizationContextMissing,
        admin::OperatorAdminAuthorizationFailureKind::OperatorAuthorizationContextExpired,
        admin::OperatorAdminAuthorizationFailureKind::OperatorActionDenied,
        admin::OperatorAdminAuthorizationFailureKind::OperatorScopeNotAllowed,
        admin::OperatorAdminAuthorizationFailureKind::AdminActionNotAllowed,
        admin::OperatorAdminAuthorizationFailureKind::MaintenanceModeActive,
        admin::OperatorAdminAuthorizationFailureKind::RuntimeConfigMissing,
        admin::OperatorAdminAuthorizationFailureKind::RuntimeConfigInvalid,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = admin::OperatorAdminAuthorizationFailure::from_kind(kind);
    }
}

#[test]
fn admin_operator_authorization_guards_cover_late_fail_closed_branches() {
    assert_eq!(
        admin::HealthAdminAuditGuard::try_new(
            admin::HealthAdminAuditEventType::AdminMaintenanceDecision,
            admin::HealthAdminAuditOutcome::Rejected,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(admin::HealthAdminAuditError::CatalogedReasonMissing)
    );

    for (index, expected) in [
        (
            0,
            admin::OperatorAdminAuthorizationError::OperatorAdminClassMissing,
        ),
        (
            1,
            admin::OperatorAdminAuthorizationError::CredentialContextSourceMissing,
        ),
        (
            2,
            admin::OperatorAdminAuthorizationError::AllowedActionClassMissing,
        ),
        (
            3,
            admin::OperatorAdminAuthorizationError::TargetScopeMissing,
        ),
        (
            4,
            admin::OperatorAdminAuthorizationError::LifetimeExpiryMissing,
        ),
        (
            5,
            admin::OperatorAdminAuthorizationError::RedactionRuleMissing,
        ),
        (
            6,
            admin::OperatorAdminAuthorizationError::TargetCommandBoundaryMissing,
        ),
        (
            7,
            admin::OperatorAdminAuthorizationError::AuditEventRelationMissing,
        ),
        (
            8,
            admin::OperatorAdminAuthorizationError::FailureReasonMappingMissing,
        ),
        (
            9,
            admin::OperatorAdminAuthorizationError::RawCredentialBecameCoreIdentity,
        ),
        (
            10,
            admin::OperatorAdminAuthorizationError::UntypedCredentialVerificationResult,
        ),
        (
            11,
            admin::OperatorAdminAuthorizationError::CommunicationAuthorizationReused,
        ),
        (
            12,
            admin::OperatorAdminAuthorizationError::OperatorAuthorizationReusedForDomainAction,
        ),
        (
            13,
            admin::OperatorAdminAuthorizationError::TargetActionBoundaryMissing,
        ),
        (
            14,
            admin::OperatorAdminAuthorizationError::TargetActionEventMissing,
        ),
    ] {
        let mut flags = [true; 16];
        flags[index] = false;
        assert_eq!(operator_admin_authorization_guard(flags), Err(expected));
    }
    let mut developer_flags = [true; 16];
    developer_flags[15] = false;
    assert_eq!(
        admin::OperatorAdminAuthorizationGuard::try_new(
            admin::OperatorAdminClass::DeveloperLocalContext,
            admin::OperatorCredentialContextSourceClass::LocalDeveloperCredentialReference,
            admin::OperatorAdminAllowedActionClass::DiagnosticProbeAction,
            admin::OperatorAdminTargetScopeClass::EntrypointScope,
            developer_flags[0],
            developer_flags[1],
            developer_flags[2],
            developer_flags[3],
            developer_flags[4],
            developer_flags[5],
            developer_flags[6],
            developer_flags[7],
            developer_flags[8],
            developer_flags[9],
            developer_flags[10],
            developer_flags[11],
            developer_flags[12],
            developer_flags[13],
            developer_flags[14],
            developer_flags[15],
        ),
        Err(admin::OperatorAdminAuthorizationError::DeveloperLocalContextUsedAsOperatorAuthorization)
    );

    for (index, expected) in [
        (
            0,
            admin::OperatorAdminAuthorizationAuditError::CorrelationIdMissing,
        ),
        (
            1,
            admin::OperatorAdminAuthorizationAuditError::OperatorAdminClassMissing,
        ),
        (
            2,
            admin::OperatorAdminAuthorizationAuditError::ActionClassMissing,
        ),
        (
            3,
            admin::OperatorAdminAuthorizationAuditError::TargetScopeMissing,
        ),
        (
            4,
            admin::OperatorAdminAuthorizationAuditError::StartupRunIdMissing,
        ),
    ] {
        let mut flags = [true; 6];
        flags[index] = false;
        assert_eq!(
            operator_admin_audit_guard(flags, admin::OperatorAdminAuthorizationOutcome::Accepted),
            Err(expected)
        );
    }
    let mut audit_flags = [true; 6];
    audit_flags[5] = false;
    assert_eq!(
        operator_admin_audit_guard(
            audit_flags,
            admin::OperatorAdminAuthorizationOutcome::Rejected
        ),
        Err(admin::OperatorAdminAuthorizationAuditError::CatalogedReasonMissing)
    );
}

#[test]
fn configuration_failure_vocabularies_are_closed() {
    for kind in [
        configuration::RuntimeReconfigurationFailureKind::RuntimeReconfigurationNotAllowed,
        configuration::RuntimeReconfigurationFailureKind::ConfigurationGenerationMissing,
        configuration::RuntimeReconfigurationFailureKind::ConfigurationGenerationConflict,
        configuration::RuntimeReconfigurationFailureKind::RuntimeReconfigurationValidationFailed,
        configuration::RuntimeReconfigurationFailureKind::RuntimeReconfigurationApplyNotAllowed,
        configuration::RuntimeReconfigurationFailureKind::RuntimeReconfigurationDrainRequired,
        configuration::RuntimeReconfigurationFailureKind::RuntimeReconfigurationRollbackRequired,
        configuration::RuntimeReconfigurationFailureKind::RuntimeReconfigurationRollbackFailed,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = configuration::RuntimeReconfigurationFailure::from_kind(kind);
    }

    for kind in [
        configuration::FeatureCapabilityFailureKind::CapabilityNotEnabled,
        configuration::FeatureCapabilityFailureKind::RuntimeConfigInvalid,
        configuration::FeatureCapabilityFailureKind::CorePolicyConfigInvalid,
        configuration::FeatureCapabilityFailureKind::FeatureAdmissionFailure(
            FeatureAdmissionFailureKind::FeatureOutOfScope,
        ),
        configuration::FeatureCapabilityFailureKind::RuntimeReconfigurationNotAllowed,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = configuration::FeatureCapabilityFailure::from_kind(kind);
    }

    for unsupported in [
        configuration::FeatureCapabilityUnsupportedVersionReason::UnsupportedCommandVersion,
        configuration::FeatureCapabilityUnsupportedVersionReason::UnsupportedMediaContractVersion,
        configuration::FeatureCapabilityUnsupportedVersionReason::UnsupportedTurnContractVersion,
        configuration::FeatureCapabilityUnsupportedVersionReason::UnsupportedDriverWireVersion,
        configuration::FeatureCapabilityUnsupportedVersionReason::PublicEndpointVersionUnsupported,
        configuration::FeatureCapabilityUnsupportedVersionReason::InternalControlVersionUnsupported,
        configuration::FeatureCapabilityUnsupportedVersionReason::TokenUnsupportedAlgorithm,
    ] {
        assert_cataloged(unsupported.reason_code());
        let _ =
            configuration::FeatureCapabilityFailure::from_unsupported_version_reason(unsupported);
    }
}

#[test]
fn endpoints_edge_proxy_and_public_endpoint_paths_are_fail_closed() {
    assert_eq!(
        endpoints::EdgeProxyAuditEventType::EdgeProxyTrustDecision.event_type(),
        "edge_proxy_trust_decision"
    );

    for kind in [
        endpoints::PublicEndpointFailureKind::PublicEndpointNotAllowed,
        endpoints::PublicEndpointFailureKind::PublicEndpointVersionUnsupported,
        endpoints::PublicEndpointFailureKind::PublicEndpointAuthRequired,
        endpoints::PublicEndpointFailureKind::PublicEndpointUpgradeFailed,
        endpoints::PublicEndpointFailureKind::ConnectionLifecycleViolation,
        endpoints::PublicEndpointFailureKind::ConnectionIdleTimeout,
        endpoints::PublicEndpointFailureKind::ConnectionClosePolicyViolation,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = endpoints::PublicEndpointFailure::from_kind(kind);
    }
    for kind in [
        endpoints::EdgeProxyTrustFailureKind::EdgeProxyNotAdmitted,
        endpoints::EdgeProxyTrustFailureKind::ForwardedHeaderUntrusted,
        endpoints::EdgeProxyTrustFailureKind::ForwardedHeaderChainInvalid,
        endpoints::EdgeProxyTrustFailureKind::OriginHostNotAllowed,
        endpoints::EdgeProxyTrustFailureKind::ClientAddressUntrusted,
        endpoints::EdgeProxyTrustFailureKind::TlsTerminationBoundaryInvalid,
        endpoints::EdgeProxyTrustFailureKind::PublicInternalRouteConfusion,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = endpoints::EdgeProxyTrustFailure::from_kind(kind);
    }

    assert_eq!(
        endpoints::EdgeProxyHeaderSourceGuard::try_new(
            endpoints::EdgeProxyClass::DirectPublicListener,
            endpoints::TrustedMetadataClass::ForwardedFor,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(endpoints::EdgeProxyHeaderSourceError::MetadataClassNotAdmittedForEdgeClass)
    );
    for (index, expected) in [
        (
            0,
            endpoints::EdgeProxyHeaderSourceError::ImmediateUpstreamUntrusted,
        ),
        (
            1,
            endpoints::EdgeProxyHeaderSourceError::DeterministicPrecedenceMissing,
        ),
        (
            2,
            endpoints::EdgeProxyHeaderSourceError::RawMetadataUsedAsCoreIdentity,
        ),
        (
            3,
            endpoints::EdgeProxyHeaderSourceError::TypedPolicyInputMissing,
        ),
        (
            4,
            endpoints::EdgeProxyHeaderSourceError::ProxyRequestIdReplacesCorrelationId,
        ),
    ] {
        let mut flags = [true; 5];
        flags[index] = false;
        assert_eq!(edge_proxy_header_source_guard(flags), Err(expected));
    }

    assert_eq!(
        endpoints::EdgeTlsTerminationGuard::try_new(
            endpoints::EdgeTlsTerminationClass::TerminationNotAdmitted,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(endpoints::EdgeTlsTerminationError::TerminationClassNotAdmitted)
    );
    for (index, expected) in [
        (
            0,
            endpoints::EdgeTlsTerminationError::BackendTransportSecurityRequirementMissing,
        ),
        (
            1,
            endpoints::EdgeTlsTerminationError::TrustedEdgeIdentityMissing,
        ),
        (
            2,
            endpoints::EdgeTlsTerminationError::CertificateSecretReferenceClassMissing,
        ),
        (
            3,
            endpoints::EdgeTlsTerminationError::AuditEventRelationMissing,
        ),
        (
            4,
            endpoints::EdgeTlsTerminationError::MissingDownstreamProtectionFailureMappingMissing,
        ),
        (
            5,
            endpoints::EdgeTlsTerminationError::EdgeTlsUsedAsSecureMediaProof,
        ),
    ] {
        let mut flags = [true; 6];
        flags[index] = false;
        assert_eq!(edge_tls_termination_guard(flags), Err(expected));
    }
}

#[test]
fn internal_control_failure_vocabularies_and_audit_are_closed() {
    assert_eq!(
        internal_control::InternalControlPlaneAuditEventType::InternalControlPlaneDecision
            .event_type(),
        "internal_control_plane_decision"
    );

    for kind in [
        internal_control::InternalServiceTrustFailureKind::InternalServiceIdentitySourceNotAdmitted,
        internal_control::InternalServiceTrustFailureKind::InternalServiceIdentityMissing,
        internal_control::InternalServiceTrustFailureKind::InternalServiceIdentityInvalid,
        internal_control::InternalServiceTrustFailureKind::InternalServiceIdentityUntrusted,
        internal_control::InternalServiceTrustFailureKind::InternalServiceIdentityScopeConflict,
        internal_control::InternalServiceTrustFailureKind::InternalServicePeerVerificationFailed,
        internal_control::InternalServiceTrustFailureKind::InternalServiceCredentialExpired,
        internal_control::InternalServiceTrustFailureKind::InternalServiceTrustPolicyMissing,
        internal_control::InternalServiceTrustFailureKind::ServiceEndpointScopeConflict,
        internal_control::InternalServiceTrustFailureKind::InternalControlAuthorizationMissing,
        internal_control::InternalServiceTrustFailureKind::InternalControlAuthorizationDenied,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = internal_control::InternalServiceTrustFailure::from_kind(kind);
    }

    for kind in [
        internal_control::InternalControlPlaneFailureKind::InternalControlMessageInvalid,
        internal_control::InternalControlPlaneFailureKind::InternalControlVersionUnsupported,
        internal_control::InternalControlPlaneFailureKind::InternalControlAuthorizationMissing,
        internal_control::InternalControlPlaneFailureKind::InternalControlAuthorizationDenied,
        internal_control::InternalControlPlaneFailureKind::InternalServiceTrustFailure(
            internal_control::InternalServiceTrustFailureKind::InternalServiceIdentityUntrusted,
        ),
        internal_control::InternalControlPlaneFailureKind::ServiceDiscoveryUnavailable,
        internal_control::InternalControlPlaneFailureKind::NodeAffinityRequired,
        internal_control::InternalControlPlaneFailureKind::NodeStateUnavailable,
        internal_control::InternalControlPlaneFailureKind::CrossNodeRouteNotAllowed,
        internal_control::InternalControlPlaneFailureKind::OperationDeadlineExceeded,
        internal_control::InternalControlPlaneFailureKind::ServiceEndpointStale,
        internal_control::InternalControlPlaneFailureKind::ServiceEndpointFallbackNotAllowed,
        internal_control::InternalControlPlaneFailureKind::StateOwnerConflict,
        internal_control::InternalControlPlaneFailureKind::SplitBrainRiskDetected,
        internal_control::InternalControlPlaneFailureKind::NetworkSendFailed,
        internal_control::InternalControlPlaneFailureKind::NetworkReceiveFailed,
        internal_control::InternalControlPlaneFailureKind::DriverShutdown,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = internal_control::InternalControlPlaneFailure::from_kind(kind);
    }

    for (index, expected) in [
        (
            0,
            internal_control::InternalControlPlaneAuditError::CorrelationIdMissing,
        ),
        (
            1,
            internal_control::InternalControlPlaneAuditError::OutcomeMissing,
        ),
        (
            2,
            internal_control::InternalControlPlaneAuditError::SourceServiceMissing,
        ),
        (
            3,
            internal_control::InternalControlPlaneAuditError::TargetServiceMissing,
        ),
        (
            4,
            internal_control::InternalControlPlaneAuditError::ControlPlaneClassMissing,
        ),
        (
            5,
            internal_control::InternalControlPlaneAuditError::ContractVersionMissing,
        ),
        (
            6,
            internal_control::InternalControlPlaneAuditError::TopologyClassMissing,
        ),
    ] {
        let mut flags = [true; 8];
        flags[index] = false;
        assert_eq!(
            internal_control_audit_guard(
                flags,
                internal_control::InternalControlPlaneOutcome::Success
            ),
            Err(expected)
        );
    }
    let mut audit_flags = [true; 8];
    audit_flags[7] = false;
    assert_eq!(
        internal_control_audit_guard(
            audit_flags,
            internal_control::InternalControlPlaneOutcome::Expired,
        ),
        Err(internal_control::InternalControlPlaneAuditError::CatalogedReasonMissing)
    );
}

#[test]
fn topology_reason_mappings_and_admission_guards_cover_networked_edges() {
    assert_eq!(
        topology::DeploymentTopologyAuditEventType::DeploymentTopologyDecision.event_type(),
        "deployment_topology_decision"
    );
    assert_eq!(
        topology::ServiceDiscoveryAuditEventType::ServiceDiscoveryResolutionDecision.event_type(),
        "service_discovery_resolution_decision"
    );

    for kind in [
        topology::DeploymentTopologyFailureKind::DeploymentTopologyUnsupported,
        topology::DeploymentTopologyFailureKind::ServiceDiscoveryUnavailable,
        topology::DeploymentTopologyFailureKind::ServiceDiscoverySourceNotAdmitted,
        topology::DeploymentTopologyFailureKind::ServiceEndpointScopeConflict,
        topology::DeploymentTopologyFailureKind::InternalServiceIdentitySourceNotAdmitted,
        topology::DeploymentTopologyFailureKind::InternalServiceIdentityMissing,
        topology::DeploymentTopologyFailureKind::InternalServiceIdentityInvalid,
        topology::DeploymentTopologyFailureKind::InternalServiceIdentityUntrusted,
        topology::DeploymentTopologyFailureKind::InternalServiceIdentityScopeConflict,
        topology::DeploymentTopologyFailureKind::InternalServicePeerVerificationFailed,
        topology::DeploymentTopologyFailureKind::InternalServiceTrustPolicyMissing,
        topology::DeploymentTopologyFailureKind::NodeAffinityRequired,
        topology::DeploymentTopologyFailureKind::NodeStateUnavailable,
        topology::DeploymentTopologyFailureKind::CrossNodeRouteNotAllowed,
        topology::DeploymentTopologyFailureKind::RuntimeConfigMissing,
        topology::DeploymentTopologyFailureKind::RuntimeConfigInvalid,
        topology::DeploymentTopologyFailureKind::NetworkSendFailed,
        topology::DeploymentTopologyFailureKind::NetworkReceiveFailed,
        topology::DeploymentTopologyFailureKind::DriverShutdown,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = topology::DeploymentTopologyFailure::from_kind(kind);
    }

    for kind in [
        topology::ServiceDiscoveryResolutionFailureKind::ServiceDiscoverySourceNotAdmitted,
        topology::ServiceDiscoveryResolutionFailureKind::ServiceDiscoveryUnavailable,
        topology::ServiceDiscoveryResolutionFailureKind::ServiceEndpointResolutionFailed,
        topology::ServiceDiscoveryResolutionFailureKind::ServiceEndpointStale,
        topology::ServiceDiscoveryResolutionFailureKind::ServiceEndpointFallbackNotAllowed,
        topology::ServiceDiscoveryResolutionFailureKind::ServiceEndpointScopeConflict,
        topology::ServiceDiscoveryResolutionFailureKind::ServiceEndpointContractMismatch,
        topology::ServiceDiscoveryResolutionFailureKind::PublicInternalRouteConfusion,
        topology::ServiceDiscoveryResolutionFailureKind::InternalServiceIdentityUntrusted,
    ] {
        assert_cataloged(kind.reason_code());
        let _ = topology::ServiceDiscoveryResolutionFailure::from_kind(kind);
    }

    for (topology_class, index, expected) in [
        (
            topology::DeploymentTopologyClass::SplitPlaneNetworked,
            5,
            topology::DeploymentTopologyAdmissionError::ExplicitServiceEndpointWiringMissing,
        ),
        (
            topology::DeploymentTopologyClass::SplitPlaneNetworked,
            6,
            topology::DeploymentTopologyAdmissionError::ServiceEndpointFailureMappingMissing,
        ),
        (
            topology::DeploymentTopologyClass::SplitPlaneNetworked,
            7,
            topology::DeploymentTopologyAdmissionError::InternalServiceTrustRelationMissing,
        ),
        (
            topology::DeploymentTopologyClass::MultiNodeExperimental,
            11,
            topology::DeploymentTopologyAdmissionError::ExperimentalEnablementMissing,
        ),
        (
            topology::DeploymentTopologyClass::SingleProcessLocal,
            12,
            topology::DeploymentTopologyAdmissionError::TopologyPolicyInputNotTyped,
        ),
        (
            topology::DeploymentTopologyClass::SingleProcessLocal,
            13,
            topology::DeploymentTopologyAdmissionError::LocalDevTopologyEnabledForManagedRuntime,
        ),
    ] {
        let mut flags = [true; 14];
        flags[index] = false;
        assert_eq!(
            topology_admission_guard(topology_class, flags),
            Err(expected)
        );
    }

    assert_eq!(
        topology::ServiceDiscoveryResolutionAdmissionGuard::try_new(
            topology::DiscoverySourceClass::LocalProcessRegistry,
            topology::DeploymentTopologyClass::SplitPlaneNetworked,
            topology::ResolutionTargetService::Signaling,
            topology::ResolvedEndpointScope::InternalService,
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
        Err(topology::ServiceDiscoveryResolutionAdmissionError::DiscoverySourceTopologyMismatch)
    );
    for (index, expected) in [
        (
            0,
            topology::ServiceDiscoveryResolutionAdmissionError::DiscoverySourceClassMissing,
        ),
        (1, topology::ServiceDiscoveryResolutionAdmissionError::TopologyClassMissing),
        (2, topology::ServiceDiscoveryResolutionAdmissionError::TargetServiceMissing),
        (3, topology::ServiceDiscoveryResolutionAdmissionError::EndpointScopeMissing),
        (
            4,
            topology::ServiceDiscoveryResolutionAdmissionError::
                PublicInternalEndpointRelationMissing,
        ),
        (
            5,
            topology::ServiceDiscoveryResolutionAdmissionError::EndpointContractReferenceMissing,
        ),
        (
            6,
            topology::ServiceDiscoveryResolutionAdmissionError::TtlCacheStalenessRuleMissing,
        ),
        (7, topology::ServiceDiscoveryResolutionAdmissionError::FallbackBehaviorMissing),
        (
            8,
            topology::ServiceDiscoveryResolutionAdmissionError::AuthorizationContextRelationMissing,
        ),
        (
            9,
            topology::ServiceDiscoveryResolutionAdmissionError::ServiceIdentityTrustRelationMissing,
        ),
        (10, topology::ServiceDiscoveryResolutionAdmissionError::ReadinessRelationMissing),
        (11, topology::ServiceDiscoveryResolutionAdmissionError::AuditShapeMissing),
        (12, topology::ServiceDiscoveryResolutionAdmissionError::RegistryContractMissing),
        (13, topology::ServiceDiscoveryResolutionAdmissionError::MeshPolicyUsedAsAuthorization),
        (
            14,
            topology::ServiceDiscoveryResolutionAdmissionError::TestResolverUsedOutsideTestScope,
        ),
    ] {
        let mut flags = [true; 15];
        flags[index] = false;
        let source = match index {
            12 => topology::DiscoverySourceClass::ServiceRegistryLookup,
            13 => topology::DiscoverySourceClass::ServiceMeshResolution,
            14 => topology::DiscoverySourceClass::TestResolver,
            _ => topology::DiscoverySourceClass::StaticConfigEndpoint,
        };
        let topology_class = if source == topology::DiscoverySourceClass::TestResolver {
            topology::DeploymentTopologyClass::SingleProcessLocal
        } else {
            topology::DeploymentTopologyClass::SplitPlaneNetworked
        };
        assert_eq!(
            service_discovery_admission_guard(source, topology_class, flags),
            Err(expected)
        );
    }
}

fn operator_admin_authorization_guard(
    flags: [bool; 16],
) -> Result<admin::OperatorAdminAuthorizationGuard, admin::OperatorAdminAuthorizationError> {
    admin::OperatorAdminAuthorizationGuard::try_new(
        admin::OperatorAdminClass::OperatorAdminActionContext,
        admin::OperatorCredentialContextSourceClass::EntrypointDriverCredentialReference,
        admin::OperatorAdminAllowedActionClass::AdminAction,
        admin::OperatorAdminTargetScopeClass::EntrypointScope,
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
    )
}

fn operator_admin_audit_guard(
    flags: [bool; 6],
    outcome: admin::OperatorAdminAuthorizationOutcome,
) -> Result<admin::OperatorAdminAuthorizationAuditGuard, admin::OperatorAdminAuthorizationAuditError>
{
    admin::OperatorAdminAuthorizationAuditGuard::try_new(
        admin::OperatorAdminAuthorizationAuditEventType::OperatorAdminAuthorizationDecision,
        outcome,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
        flags[5],
    )
}

fn edge_proxy_header_source_guard(
    flags: [bool; 5],
) -> Result<endpoints::EdgeProxyHeaderSourceGuard, endpoints::EdgeProxyHeaderSourceError> {
    endpoints::EdgeProxyHeaderSourceGuard::try_new(
        endpoints::EdgeProxyClass::ReverseProxyHttpWs,
        endpoints::TrustedMetadataClass::ForwardedFor,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
    )
}

fn edge_tls_termination_guard(
    flags: [bool; 6],
) -> Result<endpoints::EdgeTlsTerminationGuard, endpoints::EdgeTlsTerminationError> {
    endpoints::EdgeTlsTerminationGuard::try_new(
        endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
        flags[5],
    )
}

fn internal_control_audit_guard(
    flags: [bool; 8],
    outcome: internal_control::InternalControlPlaneOutcome,
) -> Result<
    internal_control::InternalControlPlaneAuditGuard,
    internal_control::InternalControlPlaneAuditError,
> {
    internal_control::InternalControlPlaneAuditGuard::try_new(
        internal_control::InternalControlPlaneAuditEventType::InternalControlPlaneDecision,
        outcome,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
        flags[5],
        flags[6],
        flags[7],
    )
}

fn topology_admission_guard(
    topology_class: topology::DeploymentTopologyClass,
    flags: [bool; 14],
) -> Result<topology::DeploymentTopologyAdmissionGuard, topology::DeploymentTopologyAdmissionError>
{
    topology::DeploymentTopologyAdmissionGuard::try_new(
        topology_class,
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
    )
}

fn service_discovery_admission_guard(
    discovery_source_class: topology::DiscoverySourceClass,
    topology_class: topology::DeploymentTopologyClass,
    flags: [bool; 15],
) -> Result<
    topology::ServiceDiscoveryResolutionAdmissionGuard,
    topology::ServiceDiscoveryResolutionAdmissionError,
> {
    topology::ServiceDiscoveryResolutionAdmissionGuard::try_new(
        discovery_source_class,
        topology_class,
        topology::ResolutionTargetService::InternalControl,
        topology::ResolvedEndpointScope::InternalService,
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
    )
}
