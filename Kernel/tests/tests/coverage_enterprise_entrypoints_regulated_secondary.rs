#[allow(dead_code)]
#[path = "../../entrypoints/cli/src/main.rs"]
mod cli_main;
#[allow(dead_code)]
#[path = "../../entrypoints/demo/src/main.rs"]
mod demo_main;

use arcrtc_core_command::CoreCommandSurface;
use arcrtc_core_configuration::{ConfigurationOwner, CoreConfigurationSurface};
use arcrtc_core_features::{CoreFeaturesSurface, FeatureAdmissionFailureKind};
use arcrtc_core_identity::{
    AuditEventId, CorrelationId, OpaqueReference, ParticipantId, ReferenceAuthority, RoomId,
};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_entrypoint_admin as admin;
use arcrtc_entrypoint_composition_root::{
    run_resident_loop, ResidentDriverBindingRef, ResidentLoopFailureKind, ResidentServerKind,
    ResidentServerLoopConfig, RuntimeProfileObservationRef, ShutdownDrainObservationRef,
};
use arcrtc_entrypoint_configuration as configuration;
use arcrtc_entrypoint_endpoints as endpoints;
use arcrtc_entrypoint_internal_control as internal_control;
use arcrtc_entrypoint_topology as topology;
use arcrtc_regulated as regulated;

fn cataloged(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason code must be cataloged")
}

fn opaque(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("opaque id is valid")
}

#[test]
fn regulated_public_api_preserves_optional_boundary_shape() {
    let _surface = regulated::RegulatedSurface;

    for support_class in [
        regulated::RegulatedOptionalSupportClass::DomainSpecificEnrichment,
        regulated::RegulatedOptionalSupportClass::AuditPointerMapping,
        regulated::RegulatedOptionalSupportClass::NonSensitiveTagClassification,
        regulated::RegulatedOptionalSupportClass::ExternalComplianceIntegrationHelper,
    ] {
        assert!(!support_class.owns_core_decision());
    }

    let hash_pointer =
        regulated::RegulatedHashChainRecordPointer::accept("audit-chain-0001").unwrap();
    assert_eq!(hash_pointer.as_str(), "audit-chain-0001");
    assert_eq!(
        regulated::RegulatedHashChainRecordPointer::accept("")
            .unwrap_err()
            .reason_code(),
        "regulated_empty_hash_chain_pointer"
    );
    assert_eq!(
        regulated::RegulatedHashChainRecordPointer::accept("bad\npointer")
            .unwrap_err()
            .kind(),
        regulated::RegulatedEnrichmentFailureKind::ControlCharacterInHashChainPointer
    );

    for reference in [
        regulated::RegulatedCommunicationReference::Correlation(CorrelationId::new(opaque(
            "corr-regulated-secondary",
        ))),
        regulated::RegulatedCommunicationReference::Room(RoomId::new(opaque(
            "room-regulated-secondary",
        ))),
        regulated::RegulatedCommunicationReference::Participant(ParticipantId::new(opaque(
            "participant-regulated-secondary",
        ))),
        regulated::RegulatedCommunicationReference::AuditEvent(AuditEventId::new(opaque(
            "audit-regulated-secondary",
        ))),
        regulated::RegulatedCommunicationReference::HashChainRecord(hash_pointer.clone()),
    ] {
        assert!(reference.is_immutable_pointer_only());
    }

    for tag in [
        regulated::RegulatedNonSensitiveTag::CommunicationEventReference,
        regulated::RegulatedNonSensitiveTag::AuditPointerReference,
        regulated::RegulatedNonSensitiveTag::QualityMetricReference,
        regulated::RegulatedNonSensitiveTag::ComplianceMappingReference,
    ] {
        assert!(!tag.is_core_required());
        assert!(!tag.drives_core_behavior());
    }

    for stage in [
        regulated::RegulatedEnrichmentLifecycleStage::ReceivedAllowedCorePointer,
        regulated::RegulatedEnrichmentLifecycleStage::MappedOutsideGenericCore,
        regulated::RegulatedEnrichmentLifecycleStage::LocalRecordEmitted,
        regulated::RegulatedEnrichmentLifecycleStage::ImmutableAuditPointerReferenced,
    ] {
        assert!(stage.is_post_core_optional());
    }

    let audit_pointer = AuditEventId::new(opaque("audit-pointer-secondary"));
    let input = regulated::RegulatedEnrichmentInput::new(
        regulated::RegulatedOptionalSupportClass::AuditPointerMapping,
        regulated::RegulatedEnrichmentLifecycleStage::MappedOutsideGenericCore,
        regulated::RegulatedCommunicationReference::HashChainRecord(hash_pointer),
        Some(audit_pointer.clone()),
        Some(regulated::RegulatedNonSensitiveTag::AuditPointerReference),
        regulated::RegulatedCoreDecisionParticipation::NeverParticipates,
        regulated::RegulatedCoreAuditMutation::ImmutableReferenceOnly,
        regulated::RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
    );
    assert_eq!(
        input.support_class(),
        regulated::RegulatedOptionalSupportClass::AuditPointerMapping
    );
    assert_eq!(
        input.lifecycle_stage(),
        regulated::RegulatedEnrichmentLifecycleStage::MappedOutsideGenericCore
    );
    assert!(input.reference().is_immutable_pointer_only());
    assert_eq!(
        input.audit_pointer().unwrap().as_str(),
        audit_pointer.as_str()
    );
    assert_eq!(
        input.tag(),
        Some(regulated::RegulatedNonSensitiveTag::AuditPointerReference)
    );

    let record = regulated::RegulatedEnrichmentGuard::admit(input).unwrap();
    assert_eq!(
        record.record_class(),
        regulated::RegulatedOptionalSupportClass::AuditPointerMapping
    );
    assert_eq!(
        record.input().tag(),
        Some(regulated::RegulatedNonSensitiveTag::AuditPointerReference)
    );
    assert!(!record.replaces_core_audit_event());

    for (input, expected) in [
        (
            regulated_input(
                regulated::RegulatedCoreDecisionParticipation::RequiresCoreDecision,
                regulated::RegulatedCoreAuditMutation::ImmutableReferenceOnly,
                regulated::RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
            ),
            regulated::RegulatedEnrichmentFailureKind::CoreDecisionParticipationRequested,
        ),
        (
            regulated_input(
                regulated::RegulatedCoreDecisionParticipation::NeverParticipates,
                regulated::RegulatedCoreAuditMutation::MutatesCoreAuditEvent,
                regulated::RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
            ),
            regulated::RegulatedEnrichmentFailureKind::CoreAuditMutationRequested,
        ),
        (
            regulated_input(
                regulated::RegulatedCoreDecisionParticipation::NeverParticipates,
                regulated::RegulatedCoreAuditMutation::ImmutableReferenceOnly,
                regulated::RegulatedDomainPayloadRequirement::RequiredByGenericCore,
            ),
            regulated::RegulatedEnrichmentFailureKind::DomainPayloadRequiredByGenericCore,
        ),
    ] {
        let failure = regulated::RegulatedEnrichmentGuard::admit(input).unwrap_err();
        assert_eq!(failure.kind(), expected);
        assert_eq!(failure.reason_code(), expected.reason_code());
    }

    assert!(regulated::RegulatedDependencyGuard::admits(
        regulated::RegulatedDependencySource::Regulated,
        regulated::RegulatedDependencyTarget::CoreOpaqueIdentityReferences,
    ));
    for source in [
        regulated::RegulatedDependencySource::Core,
        regulated::RegulatedDependencySource::Drivers,
        regulated::RegulatedDependencySource::Entrypoints,
        regulated::RegulatedDependencySource::Sdk,
    ] {
        assert!(!regulated::RegulatedDependencyGuard::admits(
            source,
            regulated::RegulatedDependencyTarget::CoreOpaqueIdentityReferences,
        ));
    }
    for target in [
        regulated::RegulatedDependencyTarget::CoreProtocolSemantics,
        regulated::RegulatedDependencyTarget::Drivers,
        regulated::RegulatedDependencyTarget::Entrypoints,
        regulated::RegulatedDependencyTarget::Sdk,
        regulated::RegulatedDependencyTarget::Regulated,
    ] {
        assert!(!regulated::RegulatedDependencyGuard::admits(
            regulated::RegulatedDependencySource::Regulated,
            target,
        ));
    }

    for kind in [
        regulated::RegulatedEnrichmentFailureKind::SupportClassOwnsCoreDecision,
        regulated::RegulatedEnrichmentFailureKind::LifecycleNotPostCoreOptional,
        regulated::RegulatedEnrichmentFailureKind::ReferenceIsNotImmutablePointer,
        regulated::RegulatedEnrichmentFailureKind::TagRequiredByGenericCore,
        regulated::RegulatedEnrichmentFailureKind::TagDrivesCoreBehavior,
        regulated::RegulatedEnrichmentFailureKind::CoreDecisionParticipationRequested,
        regulated::RegulatedEnrichmentFailureKind::CoreAuditMutationRequested,
        regulated::RegulatedEnrichmentFailureKind::DomainPayloadRequiredByGenericCore,
        regulated::RegulatedEnrichmentFailureKind::EmptyHashChainPointer,
        regulated::RegulatedEnrichmentFailureKind::ControlCharacterInHashChainPointer,
    ] {
        assert_eq!(
            regulated::RegulatedEnrichmentFailure::new(kind).reason_code(),
            kind.reason_code()
        );
    }

    assert_eq!(
        format!(
            "{:?}",
            regulated::ProhibitedRegulatedBehavior::RegulatedMutatesCoreAuditEvent
        ),
        "RegulatedMutatesCoreAuditEvent"
    );
}

#[test]
fn binary_entrypoint_composition_guards_are_fail_closed_without_source_changes() {
    assert!(!cli_main::CliCommandClass::Signaling.requires_operator_authorization());
    assert!(cli_main::CliCommandClass::AdminMaintenance.requires_operator_authorization());
    let _cli_wiring = cli_main::CliCommandWiring::new(
        CoreCommandSurface,
        cli_main::CliCommandClass::DeveloperInspection,
    );
    assert!(cli_main::CliCompositionGuard::try_new(
        cli_main::CliCommandClass::Signaling,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        false,
    )
    .is_ok());
    assert_eq!(
        cli_main::CliCompositionGuard::try_new(
            cli_main::CliCommandClass::AdminMaintenance,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(cli_main::CliCompositionError::OperatorAuthorizationMissing)
    );
    for (index, expected) in [
        (0, cli_main::CliCompositionError::TypedCommandMissing),
        (1, cli_main::CliCompositionError::CoreCommandBoundaryMissing),
        (2, cli_main::CliCompositionError::CliOwnsDomainDecision),
        (3, cli_main::CliCompositionError::CliDefinesReasonVocabulary),
        (4, cli_main::CliCompositionError::CliDefinesPortTrait),
        (
            5,
            cli_main::CliCompositionError::DemoDefaultAsProductionPolicy,
        ),
        (6, cli_main::CliCompositionError::OutOfScopeFeatureAdmitted),
    ] {
        let mut flags = [true; 8];
        flags[index] = false;
        assert_eq!(cli_guard(flags), Err(expected));
    }

    let _demo_wiring = demo_main::DemoCommandWiring::new(
        CoreCommandSurface,
        demo_main::DemoScenarioClass::DeveloperInspection,
    );
    for (index, expected) in [
        (0, demo_main::DemoCompositionError::TypedCommandMissing),
        (
            1,
            demo_main::DemoCompositionError::CoreCommandBoundaryMissing,
        ),
        (2, demo_main::DemoCompositionError::DemoOwnsDomainDecision),
        (
            3,
            demo_main::DemoCompositionError::DemoDefinesReasonVocabulary,
        ),
        (4, demo_main::DemoCompositionError::DemoDefinesPortTrait),
        (
            5,
            demo_main::DemoCompositionError::DemoDefaultAsProductionPolicy,
        ),
        (
            6,
            demo_main::DemoCompositionError::ListenerStartupAsReadiness,
        ),
        (
            7,
            demo_main::DemoCompositionError::DemoAsCloseOrReadyEvidence,
        ),
        (
            8,
            demo_main::DemoCompositionError::UiOrEndUserWorkflowOwnedByDemo,
        ),
        (
            9,
            demo_main::DemoCompositionError::OutOfScopeFeatureAdmitted,
        ),
    ] {
        let mut flags = [true; 10];
        flags[index] = false;
        assert_eq!(demo_guard(flags), Err(expected));
    }

    let observation = run_resident_loop(resident_config(
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
    ))
    .unwrap();
    assert_eq!(observation.server_kind(), ResidentServerKind::Signaling);

    for (config, expected) in [
        (
            resident_config(
                ResidentServerKind::Signaling,
                vec![ResidentDriverBindingRef::Network],
                "",
                "shutdown:signaling",
                "supervision:signaling",
            ),
            ResidentLoopFailureKind::ConfigMissing,
        ),
        (
            resident_config(
                ResidentServerKind::Turn,
                Vec::new(),
                "runtime-profile:turn",
                "shutdown:turn",
                "supervision:turn",
            ),
            ResidentLoopFailureKind::DriverBindingMissing,
        ),
        (
            resident_config(
                ResidentServerKind::Sfu,
                vec![ResidentDriverBindingRef::SfuTransport],
                "runtime-profile:sfu",
                "",
                "supervision:sfu",
            ),
            ResidentLoopFailureKind::ShutdownDrainRejected,
        ),
        (
            resident_config(
                ResidentServerKind::Sfu,
                vec![ResidentDriverBindingRef::SfuTransport],
                "runtime-profile:sfu",
                "shutdown:sfu",
                "",
            ),
            ResidentLoopFailureKind::SupervisionObservationFailed,
        ),
    ] {
        assert_eq!(run_resident_loop(config), Err(expected));
    }
}

#[test]
fn binary_entrypoint_failure_reasons_are_cataloged() {
    for kind in [
        cli_main::CliCompositionFailureKind::ExternalDecodeFailed,
        cli_main::CliCompositionFailureKind::RuntimeConfigMissing,
        cli_main::CliCompositionFailureKind::RuntimeConfigInvalid,
        cli_main::CliCompositionFailureKind::AuthorizationPolicyDenied,
        cli_main::CliCompositionFailureKind::FeatureOutOfScope,
        cli_main::CliCompositionFailureKind::FeatureAdmissionNotDocumented,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = cli_main::CliCompositionFailure::from_kind(kind);
    }

    for kind in [
        demo_main::DemoCompositionFailureKind::ExternalDecodeFailed,
        demo_main::DemoCompositionFailureKind::RuntimeConfigMissing,
        demo_main::DemoCompositionFailureKind::RuntimeConfigInvalid,
        demo_main::DemoCompositionFailureKind::FeatureOutOfScope,
        demo_main::DemoCompositionFailureKind::FeatureAdmissionNotDocumented,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = demo_main::DemoCompositionFailure::from_kind(kind);
    }

    for (kind, expected_debug) in [
        (ResidentLoopFailureKind::ConfigMissing, "ConfigMissing"),
        (
            ResidentLoopFailureKind::DriverBindingMissing,
            "DriverBindingMissing",
        ),
        (
            ResidentLoopFailureKind::RuntimeStartRejected,
            "RuntimeStartRejected",
        ),
        (
            ResidentLoopFailureKind::ShutdownDrainRejected,
            "ShutdownDrainRejected",
        ),
        (
            ResidentLoopFailureKind::SupervisionObservationFailed,
            "SupervisionObservationFailed",
        ),
    ] {
        assert_eq!(format!("{kind:?}"), expected_debug);
    }
}

#[test]
fn admin_secondary_guards_cover_late_fail_closed_branches() {
    let readiness_ok = [true; 19];
    assert!(admin_readiness(readiness_ok, admin::HealthAdminOutcome::Satisfied).is_ok());
    for (index, expected) in [
        (
            5,
            admin::ReadinessCompositionError::ServiceDiscoveryResolutionMissing,
        ),
        (
            6,
            admin::ReadinessCompositionError::DistributedStateFailoverMissing,
        ),
        (
            7,
            admin::ReadinessCompositionError::RuntimeTaskSupervisionMissing,
        ),
        (
            8,
            admin::ReadinessCompositionError::InternalServiceTrustMissing,
        ),
        (
            9,
            admin::ReadinessCompositionError::PublicEndpointLifecycleMissing,
        ),
        (
            11,
            admin::ReadinessCompositionError::IncludedDependencyChecksMissing,
        ),
        (12, admin::ReadinessCompositionError::ExcludedChecksMissing),
        (
            16,
            admin::ReadinessCompositionError::CloseNotClaimedScopeMissing,
        ),
    ] {
        let mut flags = readiness_ok;
        flags[index] = false;
        assert_eq!(
            admin_readiness(flags, admin::HealthAdminOutcome::Satisfied),
            Err(expected)
        );
    }

    for (index, expected) in [
        (0, admin::AdminMaintenanceCommandError::ActionClassMissing),
        (
            1,
            admin::AdminMaintenanceCommandError::ActionBoundaryMissing,
        ),
        (
            2,
            admin::AdminMaintenanceCommandError::AuthorizationRuleMissing,
        ),
        (
            4,
            admin::AdminMaintenanceCommandError::SensitiveRawPayloadExposed,
        ),
        (
            5,
            admin::AdminMaintenanceCommandError::StateMutationBypassesCoreUseCase,
        ),
        (
            6,
            admin::AdminMaintenanceCommandError::ProbeSuccessUsedAsDomainAcceptance,
        ),
        (
            7,
            admin::AdminMaintenanceCommandError::CloseoutWithoutEvidenceReport,
        ),
        (
            8,
            admin::AdminMaintenanceCommandError::MaintenanceStatusDriverLocalOnly,
        ),
    ] {
        let mut flags = [true; 9];
        flags[index] = false;
        assert_eq!(admin_maintenance(flags), Err(expected));
    }

    assert_eq!(
        admin::HealthAdminAuditGuard::try_new(
            admin::HealthAdminAuditEventType::OperationalProbeObservation,
            admin::HealthAdminAuditOutcome::WithinBoundObserved,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(admin::HealthAdminAuditError::StartupRunIdMissing)
    );
    assert_eq!(
        admin::HealthAdminAuditGuard::try_new(
            admin::HealthAdminAuditEventType::OperationalProbeObservation,
            admin::HealthAdminAuditOutcome::WithinBoundObserved,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        Err(admin::HealthAdminAuditError::ProbeActionClassMissing)
    );
    assert_eq!(
        admin::HealthAdminAuditGuard::try_new(
            admin::HealthAdminAuditEventType::OperationalProbeObservation,
            admin::HealthAdminAuditOutcome::WithinBoundObserved,
            true,
            true,
            false,
            true,
            true,
            true,
        ),
        Err(admin::HealthAdminAuditError::EntrypointReferenceMissing)
    );
    assert_eq!(
        admin::HealthAdminAuditGuard::try_new(
            admin::HealthAdminAuditEventType::AdminMaintenanceDecision,
            admin::HealthAdminAuditOutcome::Accepted,
            true,
            true,
            true,
            false,
            true,
            true,
        ),
        Err(admin::HealthAdminAuditError::AdminActionReferenceMissing)
    );
    assert_eq!(
        admin::HealthAdminAuditGuard::try_new(
            admin::HealthAdminAuditEventType::AdminMaintenanceDecision,
            admin::HealthAdminAuditOutcome::Accepted,
            true,
            true,
            true,
            true,
            false,
            true,
        ),
        Err(admin::HealthAdminAuditError::CorrelationIdMissing)
    );

    let evidence_ok = [true; 17];
    assert!(admin_health_evidence(evidence_ok, admin::HealthAdminOutcome::Satisfied).is_ok());
    for (index, expected) in [
        (
            0,
            admin::HealthAdminEvidenceError::CommandProbeEndpointMissing,
        ),
        (
            1,
            admin::HealthAdminEvidenceError::WorkingDirectoryTargetEntrypointMissing,
        ),
        (2, admin::HealthAdminEvidenceError::StartupRunIdMissing),
        (3, admin::HealthAdminEvidenceError::CorrelationIdMissing),
        (4, admin::HealthAdminEvidenceError::ProbeClassMissing),
        (
            5,
            admin::HealthAdminEvidenceError::IncludedExcludedChecksMissing,
        ),
        (6, admin::HealthAdminEvidenceError::TopologyNodeScopeMissing),
        (
            7,
            admin::HealthAdminEvidenceError::ServiceDiscoveryResolutionMissing,
        ),
        (
            8,
            admin::HealthAdminEvidenceError::DistributedStateFailoverMissing,
        ),
        (
            9,
            admin::HealthAdminEvidenceError::RuntimeTaskSupervisionMissing,
        ),
        (
            10,
            admin::HealthAdminEvidenceError::InternalServiceTrustMissing,
        ),
        (11, admin::HealthAdminEvidenceError::ExpectedOutcomeMissing),
        (12, admin::HealthAdminEvidenceError::ActualOutcomeMissing),
        (
            14,
            admin::HealthAdminEvidenceError::CloseNotClaimedScopeMissing,
        ),
        (
            15,
            admin::HealthAdminEvidenceError::DiagnosticProbeOutputAdoptedAsEvidence,
        ),
        (
            16,
            admin::HealthAdminEvidenceError::ProbeSuccessUsedAsExternalProof,
        ),
    ] {
        let mut flags = evidence_ok;
        flags[index] = false;
        assert_eq!(
            admin_health_evidence(flags, admin::HealthAdminOutcome::Satisfied),
            Err(expected)
        );
    }
}

#[test]
fn configuration_secondary_guards_cover_mapping_and_fail_closed_branches() {
    let _surface = configuration::EntrypointConfigurationSurface;
    let _wiring =
        configuration::ConfigurationWiringSet::new(CoreConfigurationSurface, CoreFeaturesSurface);

    for (profile, admitted_claim) in [
        (
            configuration::ConfigurationProfileClass::DevelopmentLocal,
            configuration::ConfigurationProfileEvidenceClaimClass::LocalManualEvidence,
        ),
        (
            configuration::ConfigurationProfileClass::TestDeterministic,
            configuration::ConfigurationProfileEvidenceClaimClass::TestEvidence,
        ),
        (
            configuration::ConfigurationProfileClass::IntegrationControlled,
            configuration::ConfigurationProfileEvidenceClaimClass::IntegrationEvidence,
        ),
        (
            configuration::ConfigurationProfileClass::BenchmarkControlled,
            configuration::ConfigurationProfileEvidenceClaimClass::BenchmarkEvidence,
        ),
    ] {
        assert!(profile
            .adoption_rule()
            .admits_claim(admitted_claim, false)
            .is_ok());
        assert_eq!(
            profile.adoption_rule().admits_claim(
                configuration::ConfigurationProfileEvidenceClaimClass::RuntimeClaim,
                true
            ),
            Err(configuration::ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted)
        );
    }

    let bundle_ok = [true; 15];
    for (index, expected) in [
        (
            0,
            configuration::ConfigurationBundleValidationError::EntrypointCompositionBundleMissing,
        ),
        (
            1,
            configuration::ConfigurationBundleValidationError::DriverRuntimeBundleInvalid,
        ),
        (
            2,
            configuration::ConfigurationBundleValidationError::CorePolicyBundleInvalid,
        ),
        (
            3,
            configuration::ConfigurationBundleValidationError::CrossBundleReferenceInvalid,
        ),
        (
            4,
            configuration::ConfigurationBundleValidationError::FeatureCapabilityNotAllowed,
        ),
        (
            6,
            configuration::ConfigurationBundleValidationError::ServiceDiscoveryMissing,
        ),
        (
            7,
            configuration::ConfigurationBundleValidationError::DistributedStateMissing,
        ),
        (
            8,
            configuration::ConfigurationBundleValidationError::InternalServiceTrustMissing,
        ),
        (9, configuration::ConfigurationBundleValidationError::RuntimeTaskMissing),
        (
            10,
            configuration::ConfigurationBundleValidationError::SecretRotationPolicyMissing,
        ),
        (
            11,
            configuration::ConfigurationBundleValidationError::SupplyChainEvidenceMissing,
        ),
        (
            12,
            configuration::ConfigurationBundleValidationError::ProfileEvidenceClassMissing,
        ),
        (
            13,
            configuration::ConfigurationBundleValidationError::PartialAcceptanceNotAdmitted,
        ),
        (
            14,
            configuration::ConfigurationBundleValidationError::StartupValidationAsRuntimeHotSwapPermission,
        ),
    ] {
        let mut flags = bundle_ok;
        flags[index] = false;
        assert_eq!(configuration_bundle(flags), Err(expected));
    }

    assert!(!configuration::RuntimeReconfigurationClass::StartupOnly.admits_runtime_apply());
    assert!(
        configuration::RuntimeReconfigurationClass::ObservabilityExportReload
            .admits_runtime_apply()
    );
    assert!(
        configuration::RuntimeReconfigurationClass::MaintenanceModeSwitch.admits_runtime_apply()
    );
    assert!(configuration::RuntimeReconfigurationClass::TestProfileSwap.is_test_evidence_only());

    for (surface, owner) in [
        (
            configuration::RuntimeReconfigurationTargetSurface::CorePolicy,
            ConfigurationOwner::Core,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::DriverRuntime,
            ConfigurationOwner::Drivers,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::EntrypointComposition,
            ConfigurationOwner::Entrypoints,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::FeatureCapability,
            ConfigurationOwner::Entrypoints,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::PublicEndpointExposure,
            ConfigurationOwner::Entrypoints,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::DeploymentTopology,
            ConfigurationOwner::Entrypoints,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::ObservabilityExport,
            ConfigurationOwner::Drivers,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::TestProfile,
            ConfigurationOwner::Entrypoints,
        ),
    ] {
        assert!(surface.owner_matches(owner));
    }

    assert_eq!(
        configuration_reconfiguration_admission([true; 13]),
        Ok(
            configuration::RuntimeReconfigurationAdmissionGuard::try_new(
                configuration::RuntimeReconfigurationClass::SecretRotationReload,
                configuration::RuntimeReconfigurationTargetSurface::SecurityMaterial,
                ConfigurationOwner::Drivers,
                configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
                configuration::RuntimeConfigurationGenerationState::PendingGeneration,
                true,
                true,
                true,
                true,
                true,
                true,
                configuration::RuntimeReconfigurationAuditEventType::RuntimeReconfigurationDecision,
                true,
                true,
                true,
                true,
                true,
                true,
                true,
            )
            .unwrap()
        )
    );
    for (index, expected) in [
        (
            0,
            configuration::RuntimeReconfigurationAdmissionError::CurrentGenerationReferenceMissing,
        ),
        (
            1,
            configuration::RuntimeReconfigurationAdmissionError::ProposedGenerationReferenceMissing,
        ),
        (
            2,
            configuration::RuntimeReconfigurationAdmissionError::ApplyScopeMissing,
        ),
        (
            3,
            configuration::RuntimeReconfigurationAdmissionError::AffectedActiveScopeMissing,
        ),
        (
            4,
            configuration::RuntimeReconfigurationAdmissionError::DrainOrRestartRequirementMissing,
        ),
        (
            5,
            configuration::RuntimeReconfigurationAdmissionError::RollbackBehaviorMissing,
        ),
        (
            6,
            configuration::RuntimeReconfigurationAdmissionError::EvidenceClassMissing,
        ),
        (
            7,
            configuration::RuntimeReconfigurationAdmissionError::CloseNotClaimedScopeMissing,
        ),
        (
            9,
            configuration::RuntimeReconfigurationAdmissionError::TargetCanonicalDoesNotAdmitClass,
        ),
        (
            10,
            configuration::RuntimeReconfigurationAdmissionError::ClassSpecificRuleMissing,
        ),
        (
            12,
            configuration::RuntimeReconfigurationAdmissionError::RawPayloadInGenerationEvidence,
        ),
    ] {
        let mut flags = [true; 13];
        flags[index] = false;
        assert_eq!(
            configuration_reconfiguration_admission(flags),
            Err(expected)
        );
    }
    let mut startup_only = [true; 13];
    startup_only[8] = true;
    assert_eq!(
        configuration::RuntimeReconfigurationAdmissionGuard::try_new(
            configuration::RuntimeReconfigurationClass::StartupOnly,
            configuration::RuntimeReconfigurationTargetSurface::CorePolicy,
            ConfigurationOwner::Core,
            configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
            configuration::RuntimeConfigurationGenerationState::PendingGeneration,
            true,
            true,
            true,
            true,
            true,
            true,
            configuration::RuntimeReconfigurationAuditEventType::RuntimeReconfigurationDecision,
            true,
            true,
            startup_only[8],
            true,
            true,
            true,
            true,
        ),
        Err(configuration::RuntimeReconfigurationAdmissionError::ReconfigurationClassNotAdmitted)
    );

    assert!(configuration::RuntimeReconfigurationApplyGuard::try_new(
        configuration::RuntimeReconfigurationTargetSurface::ObservabilityExport,
        true,
        true,
        false,
        true,
    )
    .is_ok());
    assert_eq!(
        configuration::RuntimeReconfigurationApplyGuard::try_new(
            configuration::RuntimeReconfigurationTargetSurface::ObservabilityExport,
            true,
            true,
            true,
            false,
        ),
        Err(configuration::RuntimeReconfigurationApplyError::AuditHashChainScopeRewritten)
    );

    for (index, expected) in [
        (
            0,
            configuration::RuntimeReconfigurationRollbackError::RollbackGenerationReferenceMissing,
        ),
        (
            1,
            configuration::RuntimeReconfigurationRollbackError::RollbackTriggerMissing,
        ),
        (
            2,
            configuration::RuntimeReconfigurationRollbackError::RollbackApplyScopeMissing,
        ),
        (
            3,
            configuration::RuntimeReconfigurationRollbackError::RollbackEvidenceMissing,
        ),
        (
            4,
            configuration::RuntimeReconfigurationRollbackError::InFlightOperationHandlingMissing,
        ),
        (
            5,
            configuration::RuntimeReconfigurationRollbackError::RollbackFailureReasonMissing,
        ),
    ] {
        let mut flags = [true; 6];
        flags[index] = false;
        assert_eq!(configuration_rollback(flags), Err(expected));
    }

    for flag in [
        configuration::FeatureFlagClass::DriverSelection,
        configuration::FeatureFlagClass::ExporterSelection,
        configuration::FeatureFlagClass::ProfileSelection,
        configuration::FeatureFlagClass::ExperimentalSurfaceGate,
        configuration::FeatureFlagClass::TestOnlyGate,
    ] {
        assert!(!flag.may_change_core_decision());
    }
    assert!(
        configuration::CapabilityDeclarationSurface::RuntimeFlagProfileChange
            .authority_owner_matches(
                configuration::FeatureCapabilityAuthorityOwner::RuntimeReconfigurationCanonical
            )
    );

    for kind in [
        configuration::FeatureCapabilityFailureKind::RuntimeConfigInvalid,
        configuration::FeatureCapabilityFailureKind::CorePolicyConfigInvalid,
        configuration::FeatureCapabilityFailureKind::FeatureAdmissionFailure(
            FeatureAdmissionFailureKind::RegulatedWorkflowNotSupported,
        ),
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = configuration::FeatureCapabilityFailure::from_kind(kind);
    }
    for unsupported in [
        configuration::FeatureCapabilityUnsupportedVersionReason::UnsupportedMediaContractVersion,
        configuration::FeatureCapabilityUnsupportedVersionReason::UnsupportedTurnContractVersion,
        configuration::FeatureCapabilityUnsupportedVersionReason::UnsupportedDriverWireVersion,
        configuration::FeatureCapabilityUnsupportedVersionReason::InternalControlVersionUnsupported,
    ] {
        assert_eq!(
            cataloged(unsupported.reason_code())
                .definition()
                .code()
                .as_str(),
            unsupported.reason_code()
        );
        let _ =
            configuration::FeatureCapabilityFailure::from_unsupported_version_reason(unsupported);
    }
}

#[test]
fn endpoints_secondary_public_api_covers_closed_mappings() {
    let _surface = endpoints::EntrypointEndpointsSurface;

    assert!(endpoints::PublicEndpointClass::TestOnlyEndpoint.is_test_only());
    assert!(endpoints::EndpointExposureScope::LocalTestOnly.is_local_test_only());
    assert!(endpoints::PublicEndpointClass::TurnPublicRelay
        .admits_target_contract(endpoints::EndpointTargetContract::Turn));
    assert!(!endpoints::PublicEndpointClass::AdminPrivate
        .admits_protocol_class(endpoints::PublicEndpointProtocolClass::Udp));

    assert!(endpoints::ConnectionLifecycleState::Active.is_active_traffic_state());
    assert!(endpoints::ConnectionLifecycleState::IdleExpired.requires_close_or_failure_reason());
    assert!(endpoints::ConnectionLifecycleState::ClosedByPolicy.requires_close_or_failure_reason());
    assert!(endpoints::ConnectionLifecycleState::Failed.requires_close_or_failure_reason());

    assert_eq!(
        endpoints::PublicEndpointDeclarationGuard::try_new(
            endpoints::PublicEndpointClass::TestOnlyEndpoint,
            endpoints::EndpointExposureScope::DirectPublic,
            endpoints::PublicEndpointProtocolClass::Http,
            endpoints::EndpointTargetContract::TestDouble,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            endpoints::PublicEndpointAuditEventType::PublicEndpointConnectionDecision,
            true,
            true,
        ),
        Err(endpoints::PublicEndpointDeclarationError::TestOnlyEndpointExposedOutsideLocalTest)
    );
    assert_eq!(
        endpoints::ConnectionLifecycleTransitionGuard::try_new(
            endpoints::ConnectionLifecycleState::Active,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(endpoints::ConnectionLifecycleTransitionError::SecurityCheckMissing)
    );
    assert_eq!(
        endpoints::PublicInternalEndpointSeparationGuard::try_new(
            endpoints::PublicEndpointClass::AdminPrivate,
            true,
            true,
            false,
            true,
        ),
        Err(
            endpoints::PublicInternalEndpointSeparationError::PrivateRouteAuthorizationCanonicalMissing
        )
    );

    for proxy in [
        endpoints::EdgeProxyClass::DirectPublicListener,
        endpoints::EdgeProxyClass::ReverseProxyHttpWs,
        endpoints::EdgeProxyClass::TcpUdpLoadBalancer,
        endpoints::EdgeProxyClass::TlsTerminatingEdge,
        endpoints::EdgeProxyClass::ServiceMeshIngress,
        endpoints::EdgeProxyClass::TestEdgeSimulator,
    ] {
        assert!(
            proxy.admits_metadata_class(endpoints::TrustedMetadataClass::ClientAddressObservation)
                || proxy.admits_metadata_class(endpoints::TrustedMetadataClass::ForwardedFor)
                || proxy.admits_metadata_class(endpoints::TrustedMetadataClass::EdgeRequestId)
        );
    }
    assert!(endpoints::EdgeProxyClass::TestEdgeSimulator.is_test_only());
    assert!(endpoints::TrustedMetadataClass::ForwardedFor.raw_value_must_not_be_core_identity());
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
    assert_eq!(
        endpoints::EdgeTlsTerminationGuard::try_new(
            endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(endpoints::EdgeTlsTerminationError::BackendTransportSecurityRequirementMissing)
    );

    for kind in [
        endpoints::PublicEndpointFailureKind::PublicEndpointNotAllowed,
        endpoints::PublicEndpointFailureKind::PublicEndpointVersionUnsupported,
        endpoints::PublicEndpointFailureKind::ConnectionLifecycleViolation,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = endpoints::PublicEndpointFailure::from_kind(kind);
    }
    for kind in [
        endpoints::EdgeProxyTrustFailureKind::EdgeProxyNotAdmitted,
        endpoints::EdgeProxyTrustFailureKind::ForwardedHeaderUntrusted,
        endpoints::EdgeProxyTrustFailureKind::TlsTerminationBoundaryInvalid,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = endpoints::EdgeProxyTrustFailure::from_kind(kind);
    }
}

#[test]
fn internal_control_secondary_public_api_covers_closed_mappings() {
    let _surface = internal_control::EntrypointInternalControlSurface;

    assert!(internal_control::InternalServiceTrustClass::MtlsPeerIdentity.is_credential_bearing());
    assert!(internal_control::InternalServiceTrustClass::TestServiceIdentity.is_test_only());
    assert!(
        internal_control::InternalServiceTrustClass::UnauthenticatedInternalServiceRequested
            .is_rejected_request_class()
    );
    assert!(
        internal_control::InternalServiceTrustClass::SignedServiceTokenIdentity
            .admits_proof_reference(
                internal_control::ServiceIdentityProofReferenceClass::SignedServiceTokenReference
            )
    );
    assert!(
        !internal_control::InternalServiceTrustClass::StaticConfiguredServiceIdentity
            .admits_proof_reference(
                internal_control::ServiceIdentityProofReferenceClass::PeerCertificateReference
            )
    );

    for outcome in [
        internal_control::InternalServiceTrustDecisionOutcome::Rejected,
        internal_control::InternalServiceTrustDecisionOutcome::Expired,
        internal_control::InternalServiceTrustDecisionOutcome::Failed,
    ] {
        assert!(outcome.requires_reason());
    }
    assert!(!internal_control::InternalServiceTrustDecisionOutcome::Accepted.requires_reason());

    assert_eq!(
        internal_control::InternalServiceIdentityMappingGuard::try_new(
            internal_control::InternalServiceTrustClass::UnauthenticatedInternalServiceRequested,
            internal_control::ServiceIdentityProofReferenceClass::NoCredentialInProcess,
            internal_control::InternalServiceRole::Signaling,
            internal_control::InternalServiceRole::Sfu,
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
        Err(
            internal_control::InternalServiceIdentityMappingError::UnauthenticatedInternalServiceNotAdmitted
        )
    );
    assert_eq!(
        internal_control::InternalServiceAuthorizationSequenceGuard::try_new(
            true, false, true, true, true, true,
        ),
        Err(internal_control::InternalServiceAuthorizationSequenceError::PeerProofMissing)
    );

    assert!(
        internal_control::InternalControlPlaneClass::SameHostPlaneCall
            .requires_endpoint_and_auth_context()
    );
    assert!(
        internal_control::InternalControlPlaneClass::NetworkedPlaneCall
            .requires_service_discovery_relation()
    );
    assert!(internal_control::InternalControlPlaneClass::AdminPlaneCall
        .requires_admin_authorization_relation());
    assert_eq!(
        internal_control::InternalControlCommandEventType::SfuCommandContractReference
            .message_class(),
        internal_control::InternalControlMessageClass::Command
    );
    assert!(
        internal_control::InternalControlCommandEventType::ServiceDiscoveryResolutionEventReference
            .admits_message_class(internal_control::InternalControlMessageClass::Event)
    );
    assert!(internal_control::InternalControlContractVersionState::SupportedDeclared.is_declared());
    assert!(
        internal_control::InternalControlContractVersionState::SupportedDeclared.is_supported()
    );
    assert!(
        internal_control::InternalControlAuthorizationContextClass::MissingAuthorizationContextRequested
            .is_missing_requested()
    );

    assert_eq!(
        internal_control::InternalControlPlaneContractGuard::try_new(
            internal_control::InternalControlPlaneClass::NetworkedPlaneCall,
            internal_control::InternalServiceRole::Signaling,
            internal_control::InternalServiceRole::Sfu,
            internal_control::InternalControlMessageClass::Event,
            internal_control::InternalControlCommandEventType::SfuCommandContractReference,
            internal_control::InternalControlContractVersionState::SupportedDeclared,
            internal_control::InternalControlAuthorizationContextClass::ServiceAuthorizationContext,
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
            true,
            true,
        ),
        Err(internal_control::InternalControlPlaneContractError::CommandEventClassMismatch)
    );

    for kind in [
        internal_control::InternalServiceTrustFailureKind::InternalServiceIdentityMissing,
        internal_control::InternalServiceTrustFailureKind::InternalServiceCredentialExpired,
        internal_control::InternalServiceTrustFailureKind::InternalServiceTrustPolicyMissing,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = internal_control::InternalServiceTrustFailure::from_kind(kind);
    }
    for kind in [
        internal_control::InternalControlPlaneFailureKind::InternalControlMessageInvalid,
        internal_control::InternalControlPlaneFailureKind::InternalControlAuthorizationMissing,
        internal_control::InternalControlPlaneFailureKind::InternalControlVersionUnsupported,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = internal_control::InternalControlPlaneFailure::from_kind(kind);
    }
}

#[test]
fn topology_secondary_public_api_covers_closed_mappings() {
    let _surface = topology::EntrypointTopologySurface;

    assert!(topology::DeploymentTopologyClass::SplitPlaneSameHost
        .requires_explicit_service_endpoint_wiring());
    assert!(topology::DeploymentTopologyClass::SplitPlaneNetworked
        .requires_networked_internal_service_relation());
    assert!(topology::DeploymentTopologyClass::MultiNodeExperimental
        .requires_experimental_admission_for_production_claim());
    assert!(topology::DeploymentTopologyClass::ExternalManagedDependency
        .requires_external_dependency_contract());
    assert_eq!(
        topology::DeploymentTopologyAuditEventType::DeploymentTopologyDecision.event_type(),
        "deployment_topology_decision"
    );

    assert_eq!(
        topology::DeploymentTopologyAdmissionGuard::try_new(
            topology::DeploymentTopologyClass::ExternalManagedDependency,
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
        Err(topology::DeploymentTopologyAdmissionError::ExternalDependencyContractHealthMissing)
    );

    assert!(topology::DiscoverySourceClass::StaticConfigEndpoint
        .admits_topology(topology::DeploymentTopologyClass::SingleProcessLocal));
    assert!(topology::DiscoverySourceClass::ServiceMeshResolution.requires_ttl_cache_rule());
    assert!(topology::DiscoverySourceClass::ServiceRegistryLookup.requires_registry_contract());
    assert!(topology::DiscoverySourceClass::ServiceMeshResolution.is_mesh_resolution());
    assert!(topology::DiscoverySourceClass::TestResolver.is_test_only());
    assert!(topology::EndpointResolutionState::ResolutionStale.requires_reason());
    assert!(
        topology::ResolvedEndpointScope::InternalService.requires_internal_service_trust_relation()
    );
    assert!(topology::ResolvedEndpointScope::PublicEndpoint.requires_public_endpoint_admission());
    assert!(topology::ResolutionTargetService::InternalControl
        .requires_internal_service_trust_relation());

    assert_eq!(
        topology::NodeAffinityPolicyGuard::try_new(topology::NodeAffinityPolicyGuardInput {
            state_class: topology::NodeLocalStateClass::RoomState,
            affinity_key_declared: true,
            owning_node_scope_declared: true,
            failover_behavior_declared: false,
            unavailable_node_reason_declared: true,
            recovery_replay_relation_declared: true,
            evidence_class_declared: true,
            wrong_node_access_fails_closed_without_distributed_state_policy: true,
        }),
        Err(topology::NodeAffinityPolicyError::FailoverBehaviorMissing)
    );
    assert_eq!(
        topology::EndpointFallbackPolicyGuard::try_new(true, true, false, true, true, true, true,),
        Err(topology::EndpointFallbackPolicyError::StaleEndpointRejectionRuleMissing)
    );

    for kind in [
        topology::DeploymentTopologyFailureKind::DeploymentTopologyUnsupported,
        topology::DeploymentTopologyFailureKind::NodeAffinityRequired,
        topology::DeploymentTopologyFailureKind::ServiceDiscoveryUnavailable,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = topology::DeploymentTopologyFailure::from_kind(kind);
    }
    for kind in [
        topology::ServiceDiscoveryResolutionFailureKind::ServiceDiscoverySourceNotAdmitted,
        topology::ServiceDiscoveryResolutionFailureKind::ServiceEndpointStale,
        topology::ServiceDiscoveryResolutionFailureKind::ServiceEndpointFallbackNotAllowed,
    ] {
        assert_eq!(
            cataloged(kind.reason_code()).definition().code().as_str(),
            kind.reason_code()
        );
        let _ = topology::ServiceDiscoveryResolutionFailure::from_kind(kind);
    }
}

fn regulated_input(
    core_decision_participation: regulated::RegulatedCoreDecisionParticipation,
    core_audit_mutation: regulated::RegulatedCoreAuditMutation,
    domain_payload_requirement: regulated::RegulatedDomainPayloadRequirement,
) -> regulated::RegulatedEnrichmentInput {
    regulated::RegulatedEnrichmentInput::new(
        regulated::RegulatedOptionalSupportClass::DomainSpecificEnrichment,
        regulated::RegulatedEnrichmentLifecycleStage::LocalRecordEmitted,
        regulated::RegulatedCommunicationReference::AuditEvent(AuditEventId::new(opaque(
            "regulated-admit-secondary",
        ))),
        None,
        None,
        core_decision_participation,
        core_audit_mutation,
        domain_payload_requirement,
    )
}

fn cli_guard(
    flags: [bool; 8],
) -> Result<cli_main::CliCompositionGuard, cli_main::CliCompositionError> {
    cli_main::CliCompositionGuard::try_new(
        cli_main::CliCommandClass::Signaling,
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

fn demo_guard(
    flags: [bool; 10],
) -> Result<demo_main::DemoCompositionGuard, demo_main::DemoCompositionError> {
    demo_main::DemoCompositionGuard::try_new(
        demo_main::DemoScenarioClass::SignalingOnly,
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
    )
}

fn resident_config(
    server_kind: ResidentServerKind,
    driver_binding_refs: Vec<ResidentDriverBindingRef>,
    runtime_profile_ref: &'static str,
    shutdown_ref: &'static str,
    supervision_ref: &'static str,
) -> ResidentServerLoopConfig {
    ResidentServerLoopConfig::new(
        server_kind,
        RuntimeProfileObservationRef::new(runtime_profile_ref),
        driver_binding_refs,
        ShutdownDrainObservationRef::new(shutdown_ref),
        ShutdownDrainObservationRef::new(supervision_ref),
    )
}

fn admin_readiness(
    flags: [bool; 19],
    outcome: admin::HealthAdminOutcome,
) -> Result<admin::ReadinessCompositionGuard, admin::ReadinessCompositionError> {
    admin::ReadinessCompositionGuard::try_new(
        admin::AdminProbeClass::CompositionReadiness,
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

fn admin_maintenance(
    flags: [bool; 9],
) -> Result<admin::AdminMaintenanceCommandGuard, admin::AdminMaintenanceCommandError> {
    admin::AdminMaintenanceCommandGuard::try_new(
        admin::AdminMaintenanceActionClass::RequestDrainShutdown,
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

fn admin_health_evidence(
    flags: [bool; 17],
    outcome: admin::HealthAdminOutcome,
) -> Result<admin::HealthAdminEvidenceGuard, admin::HealthAdminEvidenceError> {
    admin::HealthAdminEvidenceGuard::try_new(
        admin::AdminProbeClass::DriverDependencyReadiness,
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
    )
}

fn configuration_bundle(
    flags: [bool; 15],
) -> Result<
    configuration::ConfigurationBundleValidationGuard,
    configuration::ConfigurationBundleValidationError,
> {
    configuration::ConfigurationBundleValidationGuard::try_new(
        configuration::ConfigurationProfileClass::ProductionCandidate,
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

fn configuration_reconfiguration_admission(
    flags: [bool; 13],
) -> Result<
    configuration::RuntimeReconfigurationAdmissionGuard,
    configuration::RuntimeReconfigurationAdmissionError,
> {
    configuration::RuntimeReconfigurationAdmissionGuard::try_new(
        configuration::RuntimeReconfigurationClass::SecretRotationReload,
        configuration::RuntimeReconfigurationTargetSurface::SecurityMaterial,
        ConfigurationOwner::Drivers,
        configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
        configuration::RuntimeConfigurationGenerationState::PendingGeneration,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
        flags[5],
        configuration::RuntimeReconfigurationAuditEventType::RuntimeReconfigurationDecision,
        flags[6],
        flags[7],
        flags[8],
        flags[9],
        flags[10],
        flags[11],
        flags[12],
    )
}

fn configuration_rollback(
    flags: [bool; 6],
) -> Result<
    configuration::RuntimeReconfigurationRollbackGuard,
    configuration::RuntimeReconfigurationRollbackError,
> {
    configuration::RuntimeReconfigurationRollbackGuard::try_new(
        configuration::RuntimeConfigurationGenerationState::RollbackGeneration,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
        flags[5],
    )
}
