use arcrtc_core_reason::CatalogedReasonRef;
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
fn internal_control_identity_mapping_closes_all_guard_branches() {
    let _surface = internal_control::EntrypointInternalControlSurface;
    assert_eq!(
        format!("{:?}", internal_control::EntrypointInternalControlSurface),
        "EntrypointInternalControlSurface"
    );

    for (trust_class, proof_class, credential_bearing, test_only) in [
        (
            internal_control::InternalServiceTrustClass::ServiceIdentityNotRequired,
            internal_control::ServiceIdentityProofReferenceClass::NoCredentialInProcess,
            false,
            false,
        ),
        (
            internal_control::InternalServiceTrustClass::StaticConfiguredServiceIdentity,
            internal_control::ServiceIdentityProofReferenceClass::StaticConfigIdentityReference,
            false,
            false,
        ),
        (
            internal_control::InternalServiceTrustClass::MtlsPeerIdentity,
            internal_control::ServiceIdentityProofReferenceClass::PeerCertificateReference,
            true,
            false,
        ),
        (
            internal_control::InternalServiceTrustClass::SignedServiceTokenIdentity,
            internal_control::ServiceIdentityProofReferenceClass::SignedServiceTokenReference,
            true,
            false,
        ),
        (
            internal_control::InternalServiceTrustClass::MeshAssertedServiceIdentity,
            internal_control::ServiceIdentityProofReferenceClass::MeshAssertionReference,
            true,
            false,
        ),
        (
            internal_control::InternalServiceTrustClass::TestServiceIdentity,
            internal_control::ServiceIdentityProofReferenceClass::TestIdentityReference,
            false,
            true,
        ),
    ] {
        assert_eq!(trust_class.is_credential_bearing(), credential_bearing);
        assert_eq!(trust_class.is_test_only(), test_only);
        assert!(!trust_class.is_rejected_request_class());
        assert!(trust_class.admits_proof_reference(proof_class));
    }
    assert!(
        internal_control::InternalServiceTrustClass::UnauthenticatedInternalServiceRequested
            .is_rejected_request_class()
    );
    assert!(
        !internal_control::InternalServiceTrustClass::StaticConfiguredServiceIdentity
            .admits_proof_reference(
                internal_control::ServiceIdentityProofReferenceClass::PeerCertificateReference
            )
    );

    assert!(identity_mapping_guard(
        internal_control::InternalServiceTrustClass::StaticConfiguredServiceIdentity,
        internal_control::ServiceIdentityProofReferenceClass::StaticConfigIdentityReference,
        [true; 18],
    )
    .is_ok());
    assert!(identity_mapping_guard(
        internal_control::InternalServiceTrustClass::TestServiceIdentity,
        internal_control::ServiceIdentityProofReferenceClass::TestIdentityReference,
        [true; 18],
    )
    .is_ok());
    assert!(identity_mapping_guard(
        internal_control::InternalServiceTrustClass::ServiceIdentityNotRequired,
        internal_control::ServiceIdentityProofReferenceClass::NoCredentialInProcess,
        [true; 18],
    )
    .is_ok());

    for (index, expected) in [
        (
            0,
            internal_control::InternalServiceIdentityMappingError::TrustClassMissing,
        ),
        (
            1,
            internal_control::InternalServiceIdentityMappingError::SourceServiceMissing,
        ),
        (
            2,
            internal_control::InternalServiceIdentityMappingError::TargetServiceMissing,
        ),
        (
            3,
            internal_control::InternalServiceIdentityMappingError::TopologyClassMissing,
        ),
        (
            4,
            internal_control::InternalServiceIdentityMappingError::ProofReferenceClassMissing,
        ),
        (
            5,
            internal_control::InternalServiceIdentityMappingError::TrustAnchorVerifierReferenceMissing,
        ),
        (
            6,
            internal_control::InternalServiceIdentityMappingError::AcceptedScopeMissing,
        ),
        (
            7,
            internal_control::InternalServiceIdentityMappingError::ContractVersionRelationMissing,
        ),
        (
            8,
            internal_control::InternalServiceIdentityMappingError::LifetimeExpiryRuleMissing,
        ),
        (
            10,
            internal_control::InternalServiceIdentityMappingError::
                ServiceDiscoveryEndpointScopeRelationMissing,
        ),
        (
            11,
            internal_control::InternalServiceIdentityMappingError::
                InternalControlAuthorizationContextRelationMissing,
        ),
        (
            12,
            internal_control::InternalServiceIdentityMappingError::AuditShapeMissing,
        ),
        (
            13,
            internal_control::InternalServiceIdentityMappingError::RawCredentialMaterialEscaped,
        ),
        (
            14,
            internal_control::InternalServiceIdentityMappingError::RawCredentialMaterialEscaped,
        ),
        (
            15,
            internal_control::InternalServiceIdentityMappingError::NonOpaqueOrUnredactedEvidenceMaterial,
        ),
    ] {
        let mut flags = [true; 18];
        flags[index] = false;
        assert_eq!(
            identity_mapping_guard(
                internal_control::InternalServiceTrustClass::StaticConfiguredServiceIdentity,
                internal_control::ServiceIdentityProofReferenceClass::StaticConfigIdentityReference,
                flags,
            ),
            Err(expected)
        );
    }

    assert_eq!(
        identity_mapping_guard(
            internal_control::InternalServiceTrustClass::StaticConfiguredServiceIdentity,
            internal_control::ServiceIdentityProofReferenceClass::PeerCertificateReference,
            [true; 18],
        ),
        Err(internal_control::InternalServiceIdentityMappingError::ProofReferenceClassMismatch)
    );
    assert_eq!(
        identity_mapping_guard(
            internal_control::InternalServiceTrustClass::UnauthenticatedInternalServiceRequested,
            internal_control::ServiceIdentityProofReferenceClass::NoCredentialInProcess,
            [true; 18],
        ),
        Err(
            internal_control::InternalServiceIdentityMappingError::
                UnauthenticatedInternalServiceNotAdmitted
        )
    );

    let mut credential_flags = [true; 18];
    credential_flags[9] = false;
    assert_eq!(
        identity_mapping_guard(
            internal_control::InternalServiceTrustClass::MtlsPeerIdentity,
            internal_control::ServiceIdentityProofReferenceClass::PeerCertificateReference,
            credential_flags,
        ),
        Err(internal_control::InternalServiceIdentityMappingError::ReplayFreshnessRuleMissing)
    );

    let mut test_flags = [true; 18];
    test_flags[16] = false;
    assert_eq!(
        identity_mapping_guard(
            internal_control::InternalServiceTrustClass::TestServiceIdentity,
            internal_control::ServiceIdentityProofReferenceClass::TestIdentityReference,
            test_flags,
        ),
        Err(internal_control::InternalServiceIdentityMappingError::TestIdentityUsedOutsideTestEvidence)
    );

    let mut in_process_flags = [true; 18];
    in_process_flags[17] = false;
    assert_eq!(
        identity_mapping_guard(
            internal_control::InternalServiceTrustClass::ServiceIdentityNotRequired,
            internal_control::ServiceIdentityProofReferenceClass::NoCredentialInProcess,
            in_process_flags,
        ),
        Err(
            internal_control::InternalServiceIdentityMappingError::
                IdentityNotRequiredUsedOutsideInProcessEvidence
        )
    );
}

#[test]
fn internal_control_contract_audit_and_evidence_success_paths_are_exercised() {
    for control_class in [
        internal_control::InternalControlPlaneClass::InProcessPlaneCall,
        internal_control::InternalControlPlaneClass::SameHostPlaneCall,
        internal_control::InternalControlPlaneClass::NetworkedPlaneCall,
        internal_control::InternalControlPlaneClass::NodeAffinityPlaneCall,
        internal_control::InternalControlPlaneClass::AdminPlaneCall,
    ] {
        let requires_remote = matches!(
            control_class,
            internal_control::InternalControlPlaneClass::SameHostPlaneCall
                | internal_control::InternalControlPlaneClass::NetworkedPlaneCall
                | internal_control::InternalControlPlaneClass::NodeAffinityPlaneCall
        );
        assert_eq!(
            control_class.requires_endpoint_and_auth_context(),
            requires_remote
        );
        assert_eq!(
            control_class.requires_internal_service_trust_context(),
            requires_remote
        );
    }
    assert!(internal_control::InternalControlPlaneClass::AdminPlaneCall
        .requires_admin_authorization_relation());
    assert!(
        internal_control::InternalControlPlaneClass::NodeAffinityPlaneCall
            .requires_node_affinity_rule()
    );

    for (event_type, class) in [
        (
            internal_control::InternalControlCommandEventType::SignalingCommandContractReference,
            internal_control::InternalControlMessageClass::Command,
        ),
        (
            internal_control::InternalControlCommandEventType::SfuCommandContractReference,
            internal_control::InternalControlMessageClass::Command,
        ),
        (
            internal_control::InternalControlCommandEventType::TurnCommandContractReference,
            internal_control::InternalControlMessageClass::Command,
        ),
        (
            internal_control::InternalControlCommandEventType::CrossPlaneShutdownDrainCommandReference,
            internal_control::InternalControlMessageClass::Command,
        ),
        (
            internal_control::InternalControlCommandEventType::TopologyNodeAffinityCommandReference,
            internal_control::InternalControlMessageClass::Command,
        ),
        (
            internal_control::InternalControlCommandEventType::AdminMaintenanceCommandReference,
            internal_control::InternalControlMessageClass::Command,
        ),
        (
            internal_control::InternalControlCommandEventType::InternalControlPlaneDecisionEventReference,
            internal_control::InternalControlMessageClass::Event,
        ),
        (
            internal_control::InternalControlCommandEventType::ServiceDiscoveryResolutionEventReference,
            internal_control::InternalControlMessageClass::Event,
        ),
        (
            internal_control::InternalControlCommandEventType::DistributedStateOwnershipEventReference,
            internal_control::InternalControlMessageClass::Event,
        ),
    ] {
        assert_eq!(event_type.message_class(), class);
        assert!(event_type.admits_message_class(class));
        assert!(!event_type.admits_message_class(match class {
            internal_control::InternalControlMessageClass::Command => {
                internal_control::InternalControlMessageClass::Event
            }
            internal_control::InternalControlMessageClass::Event => {
                internal_control::InternalControlMessageClass::Command
            }
        }));
    }

    assert!(internal_control::InternalControlContractVersionState::SupportedDeclared.is_declared());
    assert!(
        !internal_control::InternalControlContractVersionState::UnsupportedRequested.is_supported()
    );
    assert!(!internal_control::InternalControlContractVersionState::Missing.is_declared());
    assert!(
        internal_control::InternalControlAuthorizationContextClass::
            MissingAuthorizationContextRequested
            .is_missing_requested()
    );

    assert!(contract_guard(
        internal_control::InternalControlPlaneClass::NodeAffinityPlaneCall,
        internal_control::InternalControlMessageClass::Command,
        internal_control::InternalControlCommandEventType::TopologyNodeAffinityCommandReference,
        internal_control::InternalControlContractVersionState::SupportedDeclared,
        internal_control::InternalControlAuthorizationContextClass::ServiceAuthorizationContext,
        [true; 20],
    )
    .is_ok());
    assert!(contract_guard(
        internal_control::InternalControlPlaneClass::InProcessPlaneCall,
        internal_control::InternalControlMessageClass::Event,
        internal_control::InternalControlCommandEventType::InternalControlPlaneDecisionEventReference,
        internal_control::InternalControlContractVersionState::SupportedDeclared,
        internal_control::InternalControlAuthorizationContextClass::ApplicationAuthorizationContext,
        [true; 20],
    )
    .is_ok());

    for outcome in [
        internal_control::InternalControlPlaneOutcome::Success,
        internal_control::InternalControlPlaneOutcome::Rejected,
        internal_control::InternalControlPlaneOutcome::Expired,
        internal_control::InternalControlPlaneOutcome::Failed,
    ] {
        assert_eq!(
            outcome.requires_reason(),
            !matches!(
                outcome,
                internal_control::InternalControlPlaneOutcome::Success
            )
        );
    }
    assert!(internal_control_audit_guard(
        [true; 8],
        internal_control::InternalControlPlaneOutcome::Success,
    )
    .is_ok());
    assert!(internal_control_evidence_guard(
        [true; 18],
        internal_control::InternalControlPlaneClass::NetworkedPlaneCall,
        internal_control::InternalControlPlaneOutcome::Success,
    )
    .is_ok());
}

#[test]
fn internal_control_trust_audit_evidence_and_sequence_are_fail_closed() {
    assert_eq!(
        internal_control::InternalServiceTrustAuditEventType::InternalServiceTrustDecision
            .event_type(),
        "internal_service_trust_decision"
    );

    for outcome in [
        internal_control::InternalServiceTrustDecisionOutcome::Accepted,
        internal_control::InternalServiceTrustDecisionOutcome::Rejected,
        internal_control::InternalServiceTrustDecisionOutcome::Expired,
        internal_control::InternalServiceTrustDecisionOutcome::Failed,
    ] {
        assert_eq!(
            outcome.requires_reason(),
            !matches!(
                outcome,
                internal_control::InternalServiceTrustDecisionOutcome::Accepted
            )
        );
    }

    assert!(authorization_sequence_guard([true; 6]).is_ok());
    for (index, expected) in [
        (
            0,
            internal_control::InternalServiceAuthorizationSequenceError::EndpointScopeNotResolved,
        ),
        (
            1,
            internal_control::InternalServiceAuthorizationSequenceError::PeerProofMissing,
        ),
        (
            2,
            internal_control::InternalServiceAuthorizationSequenceError::OpaqueIdentityContextMissing,
        ),
        (
            3,
            internal_control::InternalServiceAuthorizationSequenceError::
                InternalControlContractValidationMissing,
        ),
        (
            4,
            internal_control::InternalServiceAuthorizationSequenceError::DomainDecisionOwnershipLost,
        ),
        (
            5,
            internal_control::InternalServiceAuthorizationSequenceError::
                EarlierFailureReclassifiedAsSuccess,
        ),
    ] {
        let mut flags = [true; 6];
        flags[index] = false;
        assert_eq!(authorization_sequence_guard(flags), Err(expected));
    }

    assert!(trust_audit_guard(
        [true; 11],
        internal_control::InternalServiceTrustDecisionOutcome::Accepted,
    )
    .is_ok());
    for (index, expected) in [
        (
            0,
            internal_control::InternalServiceTrustAuditError::StartupRunIdMissing,
        ),
        (
            1,
            internal_control::InternalServiceTrustAuditError::OutcomeMissing,
        ),
        (
            2,
            internal_control::InternalServiceTrustAuditError::TrustClassMissing,
        ),
        (
            3,
            internal_control::InternalServiceTrustAuditError::SourceServiceMissing,
        ),
        (
            4,
            internal_control::InternalServiceTrustAuditError::TargetServiceMissing,
        ),
        (
            5,
            internal_control::InternalServiceTrustAuditError::TopologyClassMissing,
        ),
        (
            6,
            internal_control::InternalServiceTrustAuditError::ProofReferenceClassMissing,
        ),
        (
            7,
            internal_control::InternalServiceTrustAuditError::TrustPolicyReferenceMissing,
        ),
        (
            8,
            internal_control::InternalServiceTrustAuditError::EndpointScopeMissing,
        ),
        (
            9,
            internal_control::InternalServiceTrustAuditError::CorrelationRuleMissing,
        ),
    ] {
        let mut flags = [true; 11];
        flags[index] = false;
        assert_eq!(
            trust_audit_guard(
                flags,
                internal_control::InternalServiceTrustDecisionOutcome::Accepted
            ),
            Err(expected)
        );
    }
    let mut audit_flags = [true; 11];
    audit_flags[10] = false;
    assert_eq!(
        trust_audit_guard(
            audit_flags,
            internal_control::InternalServiceTrustDecisionOutcome::Failed,
        ),
        Err(internal_control::InternalServiceTrustAuditError::CatalogedReasonMissing)
    );

    assert!(trust_evidence_guard([true; 17]).is_ok());
    for (index, expected) in [
        (0, internal_control::InternalServiceTrustEvidenceError::TrustClassMissing),
        (
            1,
            internal_control::InternalServiceTrustEvidenceError::SourceTargetServiceMissing,
        ),
        (2, internal_control::InternalServiceTrustEvidenceError::TopologyClassMissing),
        (
            3,
            internal_control::InternalServiceTrustEvidenceError::EndpointResolutionStateMissing,
        ),
        (
            4,
            internal_control::InternalServiceTrustEvidenceError::PeerVerificationClassMissing,
        ),
        (5, internal_control::InternalServiceTrustEvidenceError::ProofReferenceClassMissing),
        (
            6,
            internal_control::InternalServiceTrustEvidenceError::TrustPolicyReferenceMissing,
        ),
        (
            7,
            internal_control::InternalServiceTrustEvidenceError::ScopeContractRelationMissing,
        ),
        (
            8,
            internal_control::InternalServiceTrustEvidenceError::LifetimeExpiryFreshnessRuleMissing,
        ),
        (
            9,
            internal_control::InternalServiceTrustEvidenceError::
                InternalControlAuthorizationRelationMissing,
        ),
        (10, internal_control::InternalServiceTrustEvidenceError::CommandProcedureMissing),
        (11, internal_control::InternalServiceTrustEvidenceError::WorkingDirectoryMissing),
        (12, internal_control::InternalServiceTrustEvidenceError::ExpectedOutcomeMissing),
        (13, internal_control::InternalServiceTrustEvidenceError::ActualOutcomeMissing),
        (14, internal_control::InternalServiceTrustEvidenceError::CatalogedReasonMissing),
        (15, internal_control::InternalServiceTrustEvidenceError::CloseNotClaimedScopeMissing),
        (
            16,
            internal_control::InternalServiceTrustEvidenceError::DiagnosticLogsUsedAsEvidence,
        ),
    ] {
        let mut flags = [true; 17];
        flags[index] = false;
        assert_eq!(trust_evidence_guard(flags), Err(expected));
    }
}

#[test]
fn internal_control_reason_catalog_and_prohibited_vocabulary_are_closed() {
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
            internal_control::InternalServiceTrustFailureKind::InternalServiceCredentialExpired,
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

    for behavior in [
        internal_control::ProhibitedInternalServiceTrustBehavior::
            EndpointResolutionSuccessTreatedAsTrustedServiceIdentity,
        internal_control::ProhibitedInternalServiceTrustBehavior::TlsListenerStartupTreatedAsPeerTrustSuccess,
        internal_control::ProhibitedInternalServiceTrustBehavior::
            PeerVerificationTreatedAsAuthorizationSuccess,
        internal_control::ProhibitedInternalServiceTrustBehavior::MeshPolicyNameBecomesCoreServiceIdentity,
        internal_control::ProhibitedInternalServiceTrustBehavior::
            PublicEndpointCredentialReusedAsInternalServiceIdentity,
        internal_control::ProhibitedInternalServiceTrustBehavior::RawServiceCredentialMaterialEscapes,
        internal_control::ProhibitedInternalServiceTrustBehavior::
            InProcessIdentityEvidenceReusedAsNetworkedTrustEvidence,
        internal_control::ProhibitedInternalServiceTrustBehavior::TrustFailureRecordedAsFreeTextOnly,
    ] {
        assert!(format!("{:?}", behavior).len() > 10);
    }
    for behavior in [
        internal_control::ProhibitedInternalControlPlaneBehavior::InternalRpcStatusBecomesCoreReason,
        internal_control::ProhibitedInternalControlPlaneBehavior::ServiceDiscoveryOwnsDomainDecision,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            EndpointResolutionSuccessTreatedAsInternalControlSuccess,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            ExternalClientCommandBypassesPublicSignalingContract,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            AuthorizationInferredFromNetworkReachability,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            ServiceIdentityInferredFromEndpointTlsListenerOrMeshRoute,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            TopologyChangeAltersDomainStateSemantics,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            DriverToDriverInternalCallBecomesDomainAuthority,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            InternalTransportEncodingChangesDomainDecisionSemantics,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            ServiceToServiceFailureRecordedAsFreeTextOnly,
        internal_control::ProhibitedInternalControlPlaneBehavior::
            InProcessEvidenceUsedAsRemoteControlEvidence,
    ] {
        assert!(format!("{:?}", behavior).len() > 10);
    }
}

#[test]
fn topology_closed_vocabulary_const_fns_and_reason_catalog_are_exhaustive() {
    let _surface = topology::EntrypointTopologySurface;
    assert_eq!(
        format!("{:?}", topology::EntrypointTopologySurface),
        "EntrypointTopologySurface"
    );

    for (topology_class, endpoint, networked, experimental, external) in [
        (
            topology::DeploymentTopologyClass::SingleProcessLocal,
            false,
            false,
            false,
            false,
        ),
        (
            topology::DeploymentTopologyClass::SplitPlaneSameHost,
            true,
            false,
            false,
            false,
        ),
        (
            topology::DeploymentTopologyClass::SplitPlaneNetworked,
            true,
            true,
            false,
            false,
        ),
        (
            topology::DeploymentTopologyClass::MultiNodeExperimental,
            false,
            false,
            true,
            false,
        ),
        (
            topology::DeploymentTopologyClass::ExternalManagedDependency,
            false,
            false,
            false,
            true,
        ),
    ] {
        assert_eq!(
            topology_class.requires_explicit_service_endpoint_wiring(),
            endpoint
        );
        assert_eq!(
            topology_class.requires_networked_internal_service_relation(),
            networked
        );
        assert_eq!(
            topology_class.requires_experimental_admission_for_production_claim(),
            experimental
        );
        assert_eq!(
            topology_class.requires_external_dependency_contract(),
            external
        );
    }

    for state in [
        topology::NodeLocalStateClass::RoomState,
        topology::NodeLocalStateClass::SfuEndpointRouteState,
        topology::NodeLocalStateClass::TurnAllocationPermissionState,
        topology::NodeLocalStateClass::PacketCache,
        topology::NodeLocalStateClass::RuntimeQueue,
        topology::NodeLocalStateClass::SdkLocalState,
    ] {
        assert!(format!("{:?}", state).len() > 4);
    }

    for (source, topology_class, admitted, ttl, registry, mesh, test_only) in [
        (
            topology::DiscoverySourceClass::StaticConfigEndpoint,
            topology::DeploymentTopologyClass::ExternalManagedDependency,
            true,
            false,
            false,
            false,
            false,
        ),
        (
            topology::DiscoverySourceClass::LocalProcessRegistry,
            topology::DeploymentTopologyClass::SplitPlaneSameHost,
            true,
            false,
            false,
            false,
            false,
        ),
        (
            topology::DiscoverySourceClass::DnsResolution,
            topology::DeploymentTopologyClass::MultiNodeExperimental,
            true,
            true,
            false,
            false,
            false,
        ),
        (
            topology::DiscoverySourceClass::ServiceRegistryLookup,
            topology::DeploymentTopologyClass::SplitPlaneNetworked,
            true,
            true,
            true,
            false,
            false,
        ),
        (
            topology::DiscoverySourceClass::ServiceMeshResolution,
            topology::DeploymentTopologyClass::SplitPlaneNetworked,
            true,
            true,
            false,
            true,
            false,
        ),
        (
            topology::DiscoverySourceClass::TestResolver,
            topology::DeploymentTopologyClass::SingleProcessLocal,
            true,
            false,
            false,
            false,
            true,
        ),
        (
            topology::DiscoverySourceClass::LocalProcessRegistry,
            topology::DeploymentTopologyClass::SplitPlaneNetworked,
            false,
            false,
            false,
            false,
            false,
        ),
    ] {
        assert_eq!(source.admits_topology(topology_class), admitted);
        assert_eq!(source.requires_ttl_cache_rule(), ttl);
        assert_eq!(source.requires_registry_contract(), registry);
        assert_eq!(source.is_mesh_resolution(), mesh);
        assert_eq!(source.is_test_only(), test_only);
    }

    for state in [
        topology::EndpointResolutionState::ResolutionNotRequired,
        topology::EndpointResolutionState::ResolutionPending,
        topology::EndpointResolutionState::ResolutionAccepted,
        topology::EndpointResolutionState::ResolutionRejected,
        topology::EndpointResolutionState::ResolutionStale,
        topology::EndpointResolutionState::ResolutionFailed,
    ] {
        assert_eq!(
            state.requires_reason(),
            matches!(
                state,
                topology::EndpointResolutionState::ResolutionRejected
                    | topology::EndpointResolutionState::ResolutionStale
                    | topology::EndpointResolutionState::ResolutionFailed
            )
        );
    }

    assert!(
        topology::ResolvedEndpointScope::InternalService.requires_internal_service_trust_relation()
    );
    assert!(topology::ResolvedEndpointScope::PublicEndpoint.requires_public_endpoint_admission());
    assert!(topology::ResolutionTargetService::InternalControl
        .requires_internal_service_trust_relation());
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
}

#[test]
fn topology_deployment_guards_cover_success_and_fail_closed_branches() {
    assert!(topology_admission_guard(
        topology::DeploymentTopologyClass::SingleProcessLocal,
        [true; 14],
    )
    .is_ok());
    assert!(topology_admission_guard(
        topology::DeploymentTopologyClass::SplitPlaneNetworked,
        [true; 14],
    )
    .is_ok());
    assert!(topology_admission_guard(
        topology::DeploymentTopologyClass::ExternalManagedDependency,
        [true; 14],
    )
    .is_ok());
    for (index, expected) in [
        (
            0,
            topology::DeploymentTopologyAdmissionError::TopologyClassMissing,
        ),
        (
            1,
            topology::DeploymentTopologyAdmissionError::AuditShapeMissing,
        ),
        (
            2,
            topology::DeploymentTopologyAdmissionError::ServiceCompositionMissing,
        ),
        (
            3,
            topology::DeploymentTopologyAdmissionError::TopologyChangesCoreSemantics,
        ),
        (
            4,
            topology::DeploymentTopologyAdmissionError::SelectedDriverWiringMissing,
        ),
        (
            8,
            topology::DeploymentTopologyAdmissionError::EdgeProxyPolicyMissing,
        ),
        (
            9,
            topology::DeploymentTopologyAdmissionError::DistributedStatePolicyMissing,
        ),
        (
            10,
            topology::DeploymentTopologyAdmissionError::ExternalDependencyContractHealthMissing,
        ),
    ] {
        let mut flags = [true; 14];
        flags[index] = false;
        let topology_class = if index == 10 {
            topology::DeploymentTopologyClass::ExternalManagedDependency
        } else {
            topology::DeploymentTopologyClass::SingleProcessLocal
        };
        assert_eq!(
            topology_admission_guard(topology_class, flags),
            Err(expected)
        );
    }

    assert!(deployment_audit_guard([true; 4]).is_ok());
    for (index, expected) in [
        (
            0,
            topology::DeploymentTopologyAuditError::TopologyClassMissing,
        ),
        (
            1,
            topology::DeploymentTopologyAuditError::EntrypointServiceReferenceMissing,
        ),
        (
            2,
            topology::DeploymentTopologyAuditError::StartupRunIdMissing,
        ),
        (
            3,
            topology::DeploymentTopologyAuditError::CorrelationRuleMissing,
        ),
    ] {
        let mut flags = [true; 4];
        flags[index] = false;
        assert_eq!(deployment_audit_guard(flags), Err(expected));
    }

    assert!(node_affinity_guard([true; 7]).is_ok());
    for (index, expected) in [
        (0, topology::NodeAffinityPolicyError::AffinityKeyMissing),
        (1, topology::NodeAffinityPolicyError::OwningNodeScopeMissing),
        (
            2,
            topology::NodeAffinityPolicyError::FailoverBehaviorMissing,
        ),
        (
            3,
            topology::NodeAffinityPolicyError::UnavailableNodeReasonMissing,
        ),
        (
            4,
            topology::NodeAffinityPolicyError::RecoveryReplayRelationMissing,
        ),
        (5, topology::NodeAffinityPolicyError::EvidenceClassMissing),
        (
            6,
            topology::NodeAffinityPolicyError::WrongNodeAccessNotFailClosed,
        ),
    ] {
        let mut flags = [true; 7];
        flags[index] = false;
        assert_eq!(node_affinity_guard(flags), Err(expected));
    }

    assert!(topology_relation_guard([true; 7]).is_ok());
    for (index, expected) in [
        (
            0,
            topology::TopologyServiceDiscoveryRelationError::ServiceDiscoveryOwnsDomainDecision,
        ),
        (
            1,
            topology::TopologyServiceDiscoveryRelationError::
                ServiceDiscoveryOwnsProtocolVersionSemantics,
        ),
        (
            2,
            topology::TopologyServiceDiscoveryRelationError::ServiceDiscoveryOwnsAuthorizationPolicy,
        ),
        (
            3,
            topology::TopologyServiceDiscoveryRelationError::ServiceDiscoveryProvesOtherPlaneReadiness,
        ),
        (
            4,
            topology::TopologyServiceDiscoveryRelationError::ServiceDiscoveryProvesFailoverRecovery,
        ),
        (
            5,
            topology::TopologyServiceDiscoveryRelationError::DiscoveryFailureHiddenByFallback,
        ),
        (
            6,
            topology::TopologyServiceDiscoveryRelationError::NetworkedEndpointTrustEvidenceMissing,
        ),
    ] {
        let mut flags = [true; 7];
        flags[index] = false;
        assert_eq!(topology_relation_guard(flags), Err(expected));
    }

    assert!(deployment_evidence_guard([true; 14]).is_ok());
    for (index, expected) in [
        (
            0,
            topology::DeploymentTopologyEvidenceError::TopologyClassMissing,
        ),
        (
            1,
            topology::DeploymentTopologyEvidenceError::AuditShapeMissing,
        ),
        (
            2,
            topology::DeploymentTopologyEvidenceError::EdgeProxyClassMissing,
        ),
        (
            3,
            topology::DeploymentTopologyEvidenceError::ProcessEntrypointSetMissing,
        ),
        (
            4,
            topology::DeploymentTopologyEvidenceError::NodeScopeMissing,
        ),
        (
            5,
            topology::DeploymentTopologyEvidenceError::SelectedEndpointReferenceMissing,
        ),
        (
            6,
            topology::DeploymentTopologyEvidenceError::NodeAffinityRuleMissing,
        ),
        (
            7,
            topology::DeploymentTopologyEvidenceError::DiscoveryFailureBehaviorMissing,
        ),
        (
            8,
            topology::DeploymentTopologyEvidenceError::DiscoverySourceCacheFallbackMissing,
        ),
        (
            9,
            topology::DeploymentTopologyEvidenceError::InternalServiceTrustClassMissing,
        ),
        (
            10,
            topology::DeploymentTopologyEvidenceError::DistributedStateFailoverAdmissionMissing,
        ),
        (
            11,
            topology::DeploymentTopologyEvidenceError::HealthReadinessRelationMissing,
        ),
        (
            12,
            topology::DeploymentTopologyEvidenceError::CloseNotClaimedScopeMissing,
        ),
        (
            13,
            topology::DeploymentTopologyEvidenceError::SingleNodeEvidenceUsedAsMultiNodeProof,
        ),
    ] {
        let mut flags = [true; 14];
        flags[index] = false;
        assert_eq!(deployment_evidence_guard(flags), Err(expected));
    }
}

#[test]
fn topology_service_discovery_guards_cover_success_and_fail_closed_branches() {
    assert!(service_discovery_admission_guard(
        topology::DiscoverySourceClass::StaticConfigEndpoint,
        topology::DeploymentTopologyClass::SplitPlaneNetworked,
        topology::ResolutionTargetService::InternalControl,
        topology::ResolvedEndpointScope::InternalService,
        [true; 15],
    )
    .is_ok());
    assert!(service_discovery_admission_guard(
        topology::DiscoverySourceClass::ServiceMeshResolution,
        topology::DeploymentTopologyClass::SplitPlaneNetworked,
        topology::ResolutionTargetService::Signaling,
        topology::ResolvedEndpointScope::SameHostService,
        [true; 15],
    )
    .is_ok());
    assert!(service_discovery_admission_guard(
        topology::DiscoverySourceClass::TestResolver,
        topology::DeploymentTopologyClass::SingleProcessLocal,
        topology::ResolutionTargetService::TestOnly,
        topology::ResolvedEndpointScope::TestOnly,
        [true; 15],
    )
    .is_ok());

    assert!(endpoint_fallback_guard([true; 7]).is_ok());
    for (index, expected) in [
        (
            0,
            topology::EndpointFallbackPolicyError::FallbackSourceClassMissing,
        ),
        (
            1,
            topology::EndpointFallbackPolicyError::AcceptedTargetScopeMissing,
        ),
        (
            2,
            topology::EndpointFallbackPolicyError::StaleEndpointRejectionRuleMissing,
        ),
        (
            3,
            topology::EndpointFallbackPolicyError::PublicInternalSeparationRuleMissing,
        ),
        (
            4,
            topology::EndpointFallbackPolicyError::PrimaryFailureAuditReasonMissing,
        ),
        (
            5,
            topology::EndpointFallbackPolicyError::EvidenceLimitationMissing,
        ),
        (
            6,
            topology::EndpointFallbackPolicyError::UnverifiedEndpointNotFailClosed,
        ),
    ] {
        let mut flags = [true; 7];
        flags[index] = false;
        assert_eq!(endpoint_fallback_guard(flags), Err(expected));
    }

    assert!(service_discovery_audit_guard(
        [true; 9],
        topology::EndpointResolutionState::ResolutionAccepted,
    )
    .is_ok());
    for (index, expected) in [
        (0, topology::ServiceDiscoveryAuditError::StartupRunIdMissing),
        (
            1,
            topology::ServiceDiscoveryAuditError::CorrelationRuleMissing,
        ),
        (
            2,
            topology::ServiceDiscoveryAuditError::DiscoverySourceClassMissing,
        ),
        (
            3,
            topology::ServiceDiscoveryAuditError::TopologyClassMissing,
        ),
        (
            4,
            topology::ServiceDiscoveryAuditError::TargetServiceMissing,
        ),
        (
            5,
            topology::ServiceDiscoveryAuditError::EndpointScopeMissing,
        ),
        (
            6,
            topology::ServiceDiscoveryAuditError::ResolutionStateMissing,
        ),
        (
            7,
            topology::ServiceDiscoveryAuditError::FallbackClassMissing,
        ),
    ] {
        let mut flags = [true; 9];
        flags[index] = false;
        assert_eq!(
            service_discovery_audit_guard(
                flags,
                topology::EndpointResolutionState::ResolutionAccepted
            ),
            Err(expected)
        );
    }
    let mut audit_flags = [true; 9];
    audit_flags[8] = false;
    assert_eq!(
        service_discovery_audit_guard(
            audit_flags,
            topology::EndpointResolutionState::ResolutionRejected,
        ),
        Err(topology::ServiceDiscoveryAuditError::CatalogedReasonMissing)
    );

    assert!(service_discovery_evidence_guard([true; 15]).is_ok());
    for (index, expected) in [
        (
            0,
            topology::ServiceDiscoveryResolutionEvidenceError::DiscoverySourceClassMissing,
        ),
        (
            1,
            topology::ServiceDiscoveryResolutionEvidenceError::AuditShapeMissing,
        ),
        (
            2,
            topology::ServiceDiscoveryResolutionEvidenceError::TopologyClassMissing,
        ),
        (
            3,
            topology::ServiceDiscoveryResolutionEvidenceError::TargetServiceMissing,
        ),
        (
            4,
            topology::ServiceDiscoveryResolutionEvidenceError::EndpointScopeMissing,
        ),
        (
            5,
            topology::ServiceDiscoveryResolutionEvidenceError::EndpointReferenceMissing,
        ),
        (
            6,
            topology::ServiceDiscoveryResolutionEvidenceError::TtlCacheRuleMissing,
        ),
        (
            7,
            topology::ServiceDiscoveryResolutionEvidenceError::StalenessStateMissing,
        ),
        (
            8,
            topology::ServiceDiscoveryResolutionEvidenceError::FallbackBehaviorMissing,
        ),
        (
            9,
            topology::ServiceDiscoveryResolutionEvidenceError::ContractVersionReferenceMissing,
        ),
        (
            10,
            topology::ServiceDiscoveryResolutionEvidenceError::CommandProcedureMissing,
        ),
        (
            11,
            topology::ServiceDiscoveryResolutionEvidenceError::WorkingDirectoryMissing,
        ),
        (
            12,
            topology::ServiceDiscoveryResolutionEvidenceError::RerunConditionMissing,
        ),
        (
            13,
            topology::ServiceDiscoveryResolutionEvidenceError::ServiceIdentityTrustRelationMissing,
        ),
        (
            14,
            topology::ServiceDiscoveryResolutionEvidenceError::DiagnosticOutputUsedAsEvidence,
        ),
    ] {
        let mut flags = [true; 15];
        flags[index] = false;
        assert_eq!(service_discovery_evidence_guard(flags), Err(expected));
    }
}

#[test]
fn topology_prohibited_vocabulary_is_exercised_without_source_changes() {
    for behavior in [
        topology::ProhibitedDeploymentTopologyBehavior::TopologySelectionChangesCoreSemantics,
        topology::ProhibitedDeploymentTopologyBehavior::ServiceDiscoveryOwnsDomainDecision,
        topology::ProhibitedDeploymentTopologyBehavior::DiscoveryOrTlsStartupTreatedAsInternalTrust,
        topology::ProhibitedDeploymentTopologyBehavior::NodeLocalStateTreatedAsClusterGlobal,
        topology::ProhibitedDeploymentTopologyBehavior::FailoverClaimedWithoutRecoveryReplayEvidence,
        topology::ProhibitedDeploymentTopologyBehavior::FailoverClaimedFromDiscoveryFallbackAlone,
        topology::ProhibitedDeploymentTopologyBehavior::ReplicationConsensusImpliedByMultiNodeTopology,
        topology::ProhibitedDeploymentTopologyBehavior::StickyRoutingRequirementHidden,
        topology::ProhibitedDeploymentTopologyBehavior::LocalDevTopologyTreatedAsProduction,
        topology::ProhibitedDeploymentTopologyBehavior::ProxyMetadataTrustedWithoutEdgePolicy,
    ] {
        assert!(format!("{:?}", behavior).len() > 8);
    }
    for behavior in [
        topology::ProhibitedServiceDiscoveryResolutionBehavior::ResolutionSuccessTreatedAsReadiness,
        topology::ProhibitedServiceDiscoveryResolutionBehavior::ServiceDiscoveryOwnsDomainDecision,
        topology::ProhibitedServiceDiscoveryResolutionBehavior::
            FallbackEndpointUsedWithoutPolicyAndAuditReason,
        topology::ProhibitedServiceDiscoveryResolutionBehavior::StaleCachedEndpointUsedAsFreshEvidence,
        topology::ProhibitedServiceDiscoveryResolutionBehavior::
            ResolvedInternalEndpointExposedPublicByNaming,
        topology::ProhibitedServiceDiscoveryResolutionBehavior::MeshPolicyBecomesApplicationAuthorization,
        topology::ProhibitedServiceDiscoveryResolutionBehavior::
            ResolvedEndpointTreatedAsTrustedServiceIdentity,
        topology::ProhibitedServiceDiscoveryResolutionBehavior::
            InProcessEvidenceReusedAsNetworkedDiscoveryEvidence,
    ] {
        assert!(format!("{:?}", behavior).len() > 8);
    }
}

fn identity_mapping_guard(
    trust_class: internal_control::InternalServiceTrustClass,
    proof_reference_class: internal_control::ServiceIdentityProofReferenceClass,
    flags: [bool; 18],
) -> Result<
    internal_control::InternalServiceIdentityMappingGuard,
    internal_control::InternalServiceIdentityMappingError,
> {
    internal_control::InternalServiceIdentityMappingGuard::try_new(
        trust_class,
        proof_reference_class,
        internal_control::InternalServiceRole::Signaling,
        internal_control::InternalServiceRole::Sfu,
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
    )
}

fn authorization_sequence_guard(
    flags: [bool; 6],
) -> Result<
    internal_control::InternalServiceAuthorizationSequenceGuard,
    internal_control::InternalServiceAuthorizationSequenceError,
> {
    internal_control::InternalServiceAuthorizationSequenceGuard::try_new(
        flags[0], flags[1], flags[2], flags[3], flags[4], flags[5],
    )
}

fn trust_audit_guard(
    flags: [bool; 11],
    outcome: internal_control::InternalServiceTrustDecisionOutcome,
) -> Result<
    internal_control::InternalServiceTrustAuditGuard,
    internal_control::InternalServiceTrustAuditError,
> {
    internal_control::InternalServiceTrustAuditGuard::try_new(
        internal_control::InternalServiceTrustAuditEventType::InternalServiceTrustDecision,
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
    )
}

fn trust_evidence_guard(
    flags: [bool; 17],
) -> Result<
    internal_control::InternalServiceTrustEvidenceGuard,
    internal_control::InternalServiceTrustEvidenceError,
> {
    internal_control::InternalServiceTrustEvidenceGuard::try_new(
        flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6], flags[7], flags[8],
        flags[9], flags[10], flags[11], flags[12], flags[13], flags[14], flags[15], flags[16],
    )
}

fn contract_guard(
    control_plane_class: internal_control::InternalControlPlaneClass,
    message_class: internal_control::InternalControlMessageClass,
    command_event_type: internal_control::InternalControlCommandEventType,
    contract_version_state: internal_control::InternalControlContractVersionState,
    authorization_context_class: internal_control::InternalControlAuthorizationContextClass,
    flags: [bool; 20],
) -> Result<
    internal_control::InternalControlPlaneContractGuard,
    internal_control::InternalControlPlaneContractError,
> {
    internal_control::InternalControlPlaneContractGuard::try_new(
        control_plane_class,
        internal_control::InternalServiceRole::InternalControl,
        internal_control::InternalServiceRole::TopologyConfiguration,
        message_class,
        command_event_type,
        contract_version_state,
        authorization_context_class,
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
        flags[19],
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

fn internal_control_evidence_guard(
    flags: [bool; 18],
    control_plane_class: internal_control::InternalControlPlaneClass,
    outcome: internal_control::InternalControlPlaneOutcome,
) -> Result<
    internal_control::InternalControlPlaneEvidenceGuard,
    internal_control::InternalControlPlaneEvidenceError,
> {
    internal_control::InternalControlPlaneEvidenceGuard::try_new(
        control_plane_class,
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

fn deployment_audit_guard(
    flags: [bool; 4],
) -> Result<topology::DeploymentTopologyAuditGuard, topology::DeploymentTopologyAuditError> {
    topology::DeploymentTopologyAuditGuard::try_new(
        topology::DeploymentTopologyAuditEventType::DeploymentTopologyDecision,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
    )
}

fn node_affinity_guard(
    flags: [bool; 7],
) -> Result<topology::NodeAffinityPolicyGuard, topology::NodeAffinityPolicyError> {
    topology::NodeAffinityPolicyGuard::try_new(topology::NodeAffinityPolicyGuardInput {
        state_class: topology::NodeLocalStateClass::RoomState,
        affinity_key_declared: flags[0],
        owning_node_scope_declared: flags[1],
        failover_behavior_declared: flags[2],
        unavailable_node_reason_declared: flags[3],
        recovery_replay_relation_declared: flags[4],
        evidence_class_declared: flags[5],
        wrong_node_access_fails_closed_without_distributed_state_policy: flags[6],
    })
}

fn topology_relation_guard(
    flags: [bool; 7],
) -> Result<
    topology::TopologyServiceDiscoveryRelationGuard,
    topology::TopologyServiceDiscoveryRelationError,
> {
    topology::TopologyServiceDiscoveryRelationGuard::try_new(
        flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6],
    )
}

fn deployment_evidence_guard(
    flags: [bool; 14],
) -> Result<topology::DeploymentTopologyEvidenceGuard, topology::DeploymentTopologyEvidenceError> {
    topology::DeploymentTopologyEvidenceGuard::try_new(
        flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6], flags[7], flags[8],
        flags[9], flags[10], flags[11], flags[12], flags[13],
    )
}

fn service_discovery_admission_guard(
    discovery_source_class: topology::DiscoverySourceClass,
    topology_class: topology::DeploymentTopologyClass,
    target_service: topology::ResolutionTargetService,
    endpoint_scope: topology::ResolvedEndpointScope,
    flags: [bool; 15],
) -> Result<
    topology::ServiceDiscoveryResolutionAdmissionGuard,
    topology::ServiceDiscoveryResolutionAdmissionError,
> {
    topology::ServiceDiscoveryResolutionAdmissionGuard::try_new(
        discovery_source_class,
        topology_class,
        target_service,
        endpoint_scope,
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

fn endpoint_fallback_guard(
    flags: [bool; 7],
) -> Result<topology::EndpointFallbackPolicyGuard, topology::EndpointFallbackPolicyError> {
    topology::EndpointFallbackPolicyGuard::try_new(
        flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6],
    )
}

fn service_discovery_audit_guard(
    flags: [bool; 9],
    resolution_state: topology::EndpointResolutionState,
) -> Result<topology::ServiceDiscoveryAuditGuard, topology::ServiceDiscoveryAuditError> {
    topology::ServiceDiscoveryAuditGuard::try_new(
        topology::ServiceDiscoveryAuditEventType::ServiceDiscoveryResolutionDecision,
        resolution_state,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
        flags[5],
        flags[6],
        flags[7],
        flags[8],
    )
}

fn service_discovery_evidence_guard(
    flags: [bool; 15],
) -> Result<
    topology::ServiceDiscoveryResolutionEvidenceGuard,
    topology::ServiceDiscoveryResolutionEvidenceError,
> {
    topology::ServiceDiscoveryResolutionEvidenceGuard::try_new(
        flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6], flags[7], flags[8],
        flags[9], flags[10], flags[11], flags[12], flags[13], flags[14],
    )
}
