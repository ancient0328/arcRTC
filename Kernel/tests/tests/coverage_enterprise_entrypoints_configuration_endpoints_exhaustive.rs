use arcrtc_core_configuration::{ConfigurationOwner, CoreConfigurationSurface};
use arcrtc_core_features::{CoreFeaturesSurface, FeatureAdmissionFailureKind};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_entrypoint_configuration as configuration;
use arcrtc_entrypoint_endpoints as endpoints;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

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

fn assert_copy_debug_eq<T>(value: T)
where
    T: Copy + Debug + Eq,
{
    let cloned = value;
    assert_eq!(value, cloned);
    assert!(!format!("{:?}", value).is_empty());
}

fn assert_copy_debug_hash<T>(value: T)
where
    T: Copy + Debug + Eq + Hash,
{
    assert_copy_debug_eq(value);
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    assert_ne!(hasher.finish(), 0);
}

fn bundle_guard(
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

fn profile_evidence_guard(
    flags: [bool; 8],
) -> Result<
    configuration::ConfigurationProfileEvidenceGuard,
    configuration::ConfigurationProfileEvidenceError,
> {
    configuration::ConfigurationProfileEvidenceGuard::try_new(
        configuration::ConfigurationProfileClass::ProductionCandidate,
        configuration::ConfigurationProfileEvidenceClaimClass::RuntimeClaim,
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

fn runtime_admission_guard(
    class: configuration::RuntimeReconfigurationClass,
    target: configuration::RuntimeReconfigurationTargetSurface,
    owner: ConfigurationOwner,
    current: configuration::RuntimeConfigurationGenerationState,
    proposed: configuration::RuntimeConfigurationGenerationState,
    flags: [bool; 12],
) -> Result<
    configuration::RuntimeReconfigurationAdmissionGuard,
    configuration::RuntimeReconfigurationAdmissionError,
> {
    configuration::RuntimeReconfigurationAdmissionGuard::try_new(
        class,
        target,
        owner,
        current,
        proposed,
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
        true,
    )
}

fn capability_input(
    surface: configuration::CapabilityDeclarationSurface,
    owner: ConfigurationOwner,
    authority: configuration::FeatureCapabilityAuthorityOwner,
    flags: [bool; 6],
) -> configuration::CapabilityDeclarationGuardInput {
    configuration::CapabilityDeclarationGuardInput {
        surface,
        configuration_wiring_owner: owner,
        authority_owner: authority,
        accepted_contract_version_declared: flags[0],
        optional_behavior_declared: flags[1],
        fallback_when_absent_declared: flags[2],
        required_absent_reason_declared: flags[3],
        sdk_parity_requirement_declared_when_client_visible: flags[4],
        evidence_class_required_before_adoption_declared: flags[5],
    }
}

fn feature_admission_guard(
    flag_class: configuration::FeatureFlagClass,
    flags: [bool; 11],
) -> Result<
    configuration::FeatureCapabilityAdmissionGuard,
    configuration::FeatureCapabilityAdmissionError,
> {
    configuration::FeatureCapabilityAdmissionGuard::try_new(
        flag_class, flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6], flags[7],
        flags[8], flags[9], flags[10],
    )
}

fn experimental_input(
    stage: configuration::ExperimentalLifecycleStage,
    flags: [bool; 7],
) -> configuration::ExperimentalLifecycleGuardInput {
    configuration::ExperimentalLifecycleGuardInput {
        stage,
        scope_and_owner_fixed: flags[0],
        explicit_gate_present_when_scaffold_or_later: flags[1],
        dependency_direction_evidence_present_when_scaffold: flags[2],
        unit_or_contract_evidence_present_when_implemented: flags[3],
        integration_report_present_when_controlled_integration: flags[4],
        adr_or_canonical_update_and_compatibility_rule_present_when_adopted: flags[5],
        compatibility_or_deprecation_lifecycle_satisfied_when_removed: flags[6],
    }
}

fn endpoint_declaration_guard(
    endpoint_class: endpoints::PublicEndpointClass,
    exposure_scope: endpoints::EndpointExposureScope,
    protocol_class: endpoints::PublicEndpointProtocolClass,
    target_contract: endpoints::EndpointTargetContract,
    flags: [bool; 11],
) -> Result<endpoints::PublicEndpointDeclarationGuard, endpoints::PublicEndpointDeclarationError> {
    endpoints::PublicEndpointDeclarationGuard::try_new(
        endpoint_class,
        exposure_scope,
        protocol_class,
        target_contract,
        flags[0],
        flags[1],
        flags[2],
        flags[3],
        flags[4],
        flags[5],
        flags[6],
        flags[7],
        flags[8],
        endpoints::PublicEndpointAuditEventType::PublicEndpointConnectionDecision,
        flags[9],
        flags[10],
    )
}

fn lifecycle_guard(
    state: endpoints::ConnectionLifecycleState,
    flags: [bool; 7],
) -> Result<
    endpoints::ConnectionLifecycleTransitionGuard,
    endpoints::ConnectionLifecycleTransitionError,
> {
    endpoints::ConnectionLifecycleTransitionGuard::try_new(
        state, flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6],
    )
}

fn public_endpoint_evidence_guard(
    flags: [bool; 10],
) -> Result<endpoints::PublicEndpointEvidenceGuard, endpoints::PublicEndpointEvidenceError> {
    endpoints::PublicEndpointEvidenceGuard::try_new(
        flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6], flags[7], flags[8],
        flags[9],
    )
}

fn edge_trust_admission_guard(
    edge: endpoints::EdgeProxyClass,
    metadata: endpoints::TrustedMetadataClass,
    flags: [bool; 13],
) -> Result<endpoints::EdgeProxyTrustAdmissionGuard, endpoints::EdgeProxyTrustAdmissionError> {
    endpoints::EdgeProxyTrustAdmissionGuard::try_new(
        edge, metadata, flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6],
        flags[7], flags[8], flags[9], flags[10], flags[11], flags[12],
    )
}

fn edge_trust_evidence_guard(
    flags: [bool; 13],
) -> Result<endpoints::EdgeProxyTrustEvidenceGuard, endpoints::EdgeProxyTrustEvidenceError> {
    endpoints::EdgeProxyTrustEvidenceGuard::try_new(
        flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6], flags[7], flags[8],
        flags[9], flags[10], flags[11], flags[12],
    )
}

#[test]
fn configuration_closed_vocabularies_and_reason_catalogs_are_exercised() {
    let _surface = configuration::EntrypointConfigurationSurface;
    assert_copy_debug_eq(configuration::EntrypointConfigurationSurface);
    assert_copy_debug_eq(configuration::ConfigurationWiringSet::new(
        CoreConfigurationSurface,
        CoreFeaturesSurface,
    ));

    for (profile, claim, report_present) in [
        (
            configuration::ConfigurationProfileClass::DevelopmentLocal,
            configuration::ConfigurationProfileEvidenceClaimClass::LocalManualEvidence,
            false,
        ),
        (
            configuration::ConfigurationProfileClass::TestDeterministic,
            configuration::ConfigurationProfileEvidenceClaimClass::TestEvidence,
            false,
        ),
        (
            configuration::ConfigurationProfileClass::IntegrationControlled,
            configuration::ConfigurationProfileEvidenceClaimClass::IntegrationEvidence,
            false,
        ),
        (
            configuration::ConfigurationProfileClass::BenchmarkControlled,
            configuration::ConfigurationProfileEvidenceClaimClass::BenchmarkEvidence,
            false,
        ),
        (
            configuration::ConfigurationProfileClass::ProductionCandidate,
            configuration::ConfigurationProfileEvidenceClaimClass::RuntimeClaim,
            true,
        ),
        (
            configuration::ConfigurationProfileClass::ProductionCandidate,
            configuration::ConfigurationProfileEvidenceClaimClass::ProductionEvidence,
            true,
        ),
    ] {
        assert_copy_debug_hash(profile);
        assert_copy_debug_hash(claim);
        let rule = profile.adoption_rule();
        assert_copy_debug_hash(rule);
        assert_eq!(rule.admits_claim(claim, report_present), Ok(()));
    }
    assert_eq!(
        configuration::ConfigurationProfileClass::ProductionCandidate
            .adoption_rule()
            .admits_claim(
                configuration::ConfigurationProfileEvidenceClaimClass::RuntimeClaim,
                false,
            ),
        Err(configuration::ConfigurationProfileEvidenceError::ExplicitEvidenceReportReferenceMissing)
    );
    assert_eq!(
        configuration::ConfigurationProfileClass::ProductionCandidate
            .adoption_rule()
            .admits_claim(
                configuration::ConfigurationProfileEvidenceClaimClass::TestEvidence,
                true,
            ),
        Err(configuration::ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted)
    );

    for bundle in [
        configuration::ConfigurationBundleClass::CorePolicy,
        configuration::ConfigurationBundleClass::DriverRuntime,
        configuration::ConfigurationBundleClass::EntrypointComposition,
        configuration::ConfigurationBundleClass::DeploymentTopology,
        configuration::ConfigurationBundleClass::ServiceDiscovery,
        configuration::ConfigurationBundleClass::InternalServiceTrust,
        configuration::ConfigurationBundleClass::DistributedState,
        configuration::ConfigurationBundleClass::RuntimeTask,
        configuration::ConfigurationBundleClass::SecretRotation,
        configuration::ConfigurationBundleClass::SupplyChain,
        configuration::ConfigurationBundleClass::SdkClient,
        configuration::ConfigurationBundleClass::RegulatedMapping,
        configuration::ConfigurationBundleClass::TestProfile,
    ] {
        assert_copy_debug_hash(bundle);
    }

    for kind in [
        configuration::ConfigurationBundleFailureKind::RuntimeConfigMissing,
        configuration::ConfigurationBundleFailureKind::RuntimeConfigInvalid,
        configuration::ConfigurationBundleFailureKind::CorePolicyConfigInvalid,
        configuration::ConfigurationBundleFailureKind::SecretUnavailable,
        configuration::ConfigurationBundleFailureKind::SecretRotationStateUnavailable,
        configuration::ConfigurationBundleFailureKind::DeploymentTopologyUnsupported,
        configuration::ConfigurationBundleFailureKind::ServiceDiscoveryUnavailable,
        configuration::ConfigurationBundleFailureKind::ServiceDiscoverySourceNotAdmitted,
        configuration::ConfigurationBundleFailureKind::ServiceEndpointScopeConflict,
        configuration::ConfigurationBundleFailureKind::ServiceEndpointFallbackNotAllowed,
        configuration::ConfigurationBundleFailureKind::DistributedStateNotAdmitted,
        configuration::ConfigurationBundleFailureKind::StateReplicationNotAdmitted,
        configuration::ConfigurationBundleFailureKind::ConsensusNotAdmitted,
        configuration::ConfigurationBundleFailureKind::FailoverNotProven,
        configuration::ConfigurationBundleFailureKind::InternalServiceIdentitySourceNotAdmitted,
        configuration::ConfigurationBundleFailureKind::InternalServiceIdentityMissing,
        configuration::ConfigurationBundleFailureKind::InternalServiceIdentityInvalid,
        configuration::ConfigurationBundleFailureKind::InternalServiceIdentityUntrusted,
        configuration::ConfigurationBundleFailureKind::InternalServiceIdentityScopeConflict,
        configuration::ConfigurationBundleFailureKind::InternalServicePeerVerificationFailed,
        configuration::ConfigurationBundleFailureKind::InternalServiceCredentialExpired,
        configuration::ConfigurationBundleFailureKind::InternalServiceTrustPolicyMissing,
        configuration::ConfigurationBundleFailureKind::InternalControlAuthorizationMissing,
        configuration::ConfigurationBundleFailureKind::InternalControlAuthorizationDenied,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskClassNotAdmitted,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskOwnerViolation,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskSupervisionMissing,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskDetachedNotAllowed,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskSpawnFailed,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskJoinFailed,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskCancelFailed,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskPanicDetected,
        configuration::ConfigurationBundleFailureKind::RuntimeTaskQueueBoundExceeded,
        configuration::ConfigurationBundleFailureKind::DriverShutdown,
        configuration::ConfigurationBundleFailureKind::ToolchainVersionMismatch,
        configuration::ConfigurationBundleFailureKind::LockfileDriftDetected,
        configuration::ConfigurationBundleFailureKind::DependencyPolicyViolation,
        configuration::ConfigurationBundleFailureKind::LicensePolicyViolation,
        configuration::ConfigurationBundleFailureKind::VulnerabilityGateFailed,
        configuration::ConfigurationBundleFailureKind::CapabilityNotEnabled,
        configuration::ConfigurationBundleFailureKind::RuntimeReconfigurationNotAllowed,
    ] {
        assert_copy_debug_hash(kind);
        assert_cataloged(kind.reason_code());
        assert_copy_debug_hash(configuration::ConfigurationBundleFailure::from_kind(kind));
    }

    for prohibited in [
        configuration::ProhibitedConfigurationProfileBundleBehavior::EntrypointProfileChangesCoreSemanticsSilently,
        configuration::ProhibitedConfigurationProfileBundleBehavior::DriverRuntimeConfigDerivesCorePolicy,
        configuration::ProhibitedConfigurationProfileBundleBehavior::MissingRequiredBundleFallsBackToDefault,
        configuration::ProhibitedConfigurationProfileBundleBehavior::TestProfileBecomesProductionProfile,
        configuration::ProhibitedConfigurationProfileBundleBehavior::FeatureFlagEnablesExperimentalSurfaceWithoutLifecycle,
        configuration::ProhibitedConfigurationProfileBundleBehavior::ProfileEvidenceOmitsProfileClass,
        configuration::ProhibitedConfigurationProfileBundleBehavior::ImplicitTopologySecretOrSupplyChainBundle,
        configuration::ProhibitedConfigurationProfileBundleBehavior::ImplicitDiscoveryOrDistributedStateBundle,
        configuration::ProhibitedConfigurationProfileBundleBehavior::ImplicitInternalTrustOrRuntimeTaskBundle,
        configuration::ProhibitedConfigurationProfileBundleBehavior::BuildReleaseClaimWithoutSupplyChainEvidence,
        configuration::ProhibitedConfigurationProfileBundleBehavior::StartupValidationAsRuntimeHotSwapPermission,
    ] {
        assert_copy_debug_hash(prohibited);
    }
}

#[test]
fn configuration_guards_cover_success_and_fail_closed_branches() {
    assert_copy_debug_hash(bundle_guard([true; 15]).expect("bundle guard admits complete input"));
    for (index, expected) in [
        (0, configuration::ConfigurationBundleValidationError::EntrypointCompositionBundleMissing),
        (1, configuration::ConfigurationBundleValidationError::DriverRuntimeBundleInvalid),
        (2, configuration::ConfigurationBundleValidationError::CorePolicyBundleInvalid),
        (3, configuration::ConfigurationBundleValidationError::CrossBundleReferenceInvalid),
        (4, configuration::ConfigurationBundleValidationError::FeatureCapabilityNotAllowed),
        (5, configuration::ConfigurationBundleValidationError::DeploymentTopologyMissing),
        (6, configuration::ConfigurationBundleValidationError::ServiceDiscoveryMissing),
        (7, configuration::ConfigurationBundleValidationError::DistributedStateMissing),
        (8, configuration::ConfigurationBundleValidationError::InternalServiceTrustMissing),
        (9, configuration::ConfigurationBundleValidationError::RuntimeTaskMissing),
        (10, configuration::ConfigurationBundleValidationError::SecretRotationPolicyMissing),
        (11, configuration::ConfigurationBundleValidationError::SupplyChainEvidenceMissing),
        (12, configuration::ConfigurationBundleValidationError::ProfileEvidenceClassMissing),
        (13, configuration::ConfigurationBundleValidationError::PartialAcceptanceNotAdmitted),
        (
            14,
            configuration::ConfigurationBundleValidationError::StartupValidationAsRuntimeHotSwapPermission,
        ),
    ] {
        let mut flags = [true; 15];
        flags[index] = false;
        assert_eq!(bundle_guard(flags), Err(expected));
    }

    assert_copy_debug_hash(
        profile_evidence_guard([true; 8]).expect("profile evidence admits complete input"),
    );
    for (index, expected) in [
        (0, configuration::ConfigurationProfileEvidenceError::ProfileClassMissing),
        (1, configuration::ConfigurationProfileEvidenceError::RawSecretMaterialInEvidence),
        (2, configuration::ConfigurationProfileEvidenceError::ExplicitEvidenceReportReferenceMissing),
        (3, configuration::ConfigurationProfileEvidenceError::EvidencePromotedWithoutNewReport),
        (5, configuration::ConfigurationProfileEvidenceError::SupplyChainIdentityMissing),
        (7, configuration::ConfigurationProfileEvidenceError::RuntimeChangeEvidenceMissing),
    ] {
        let mut flags = [true; 8];
        if index == 5 {
            flags[4] = true;
        }
        if index == 7 {
            flags[6] = true;
        }
        flags[index] = false;
        assert_eq!(profile_evidence_guard(flags), Err(expected));
    }
    assert_eq!(
        configuration::ConfigurationProfileEvidenceGuard::try_new(
            configuration::ConfigurationProfileClass::BenchmarkControlled,
            configuration::ConfigurationProfileEvidenceClaimClass::RuntimeClaim,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
        ),
        Err(configuration::ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted)
    );

    assert_eq!(
        configuration::RuntimeReconfigurationAuditEventType::RuntimeReconfigurationDecision
            .event_type(),
        "runtime_reconfiguration_decision"
    );
    assert_copy_debug_hash(
        runtime_admission_guard(
            configuration::RuntimeReconfigurationClass::SecretRotationReload,
            configuration::RuntimeReconfigurationTargetSurface::SecurityMaterial,
            ConfigurationOwner::Drivers,
            configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
            configuration::RuntimeConfigurationGenerationState::PendingGeneration,
            [true; 12],
        )
        .expect("runtime reconfiguration admits complete input"),
    );
    for (target, owner, requires_drain) in [
        (
            configuration::RuntimeReconfigurationTargetSurface::CorePolicy,
            ConfigurationOwner::Core,
            true,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::DriverRuntime,
            ConfigurationOwner::Drivers,
            false,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::EntrypointComposition,
            ConfigurationOwner::Entrypoints,
            true,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::FeatureCapability,
            ConfigurationOwner::Entrypoints,
            true,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::PublicEndpointExposure,
            ConfigurationOwner::Entrypoints,
            true,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::SecurityMaterial,
            ConfigurationOwner::Drivers,
            true,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::DeploymentTopology,
            ConfigurationOwner::Entrypoints,
            true,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::ObservabilityExport,
            ConfigurationOwner::Drivers,
            false,
        ),
        (
            configuration::RuntimeReconfigurationTargetSurface::TestProfile,
            ConfigurationOwner::Entrypoints,
            false,
        ),
    ] {
        assert_copy_debug_hash(target);
        assert!(target.owner_matches(owner));
        assert_eq!(
            target.requires_drain_or_restart_when_runtime_changed(),
            requires_drain
        );
    }
    for state in [
        configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
        configuration::RuntimeConfigurationGenerationState::PendingGeneration,
        configuration::RuntimeConfigurationGenerationState::ValidatingGeneration,
        configuration::RuntimeConfigurationGenerationState::RejectedGeneration,
        configuration::RuntimeConfigurationGenerationState::RollbackGeneration,
        configuration::RuntimeConfigurationGenerationState::RetiredGeneration,
    ] {
        assert_copy_debug_hash(state);
    }
    assert!(configuration::RuntimeConfigurationGenerationState::CurrentGeneration.is_current());
    assert!(
        configuration::RuntimeConfigurationGenerationState::PendingGeneration
            .is_proposed_candidate()
    );
    assert!(
        configuration::RuntimeConfigurationGenerationState::ValidatingGeneration
            .is_proposed_candidate()
    );
    assert!(
        configuration::RuntimeConfigurationGenerationState::RollbackGeneration
            .is_rollback_candidate()
    );

    assert_eq!(
        runtime_admission_guard(
            configuration::RuntimeReconfigurationClass::SecretRotationReload,
            configuration::RuntimeReconfigurationTargetSurface::SecurityMaterial,
            ConfigurationOwner::Entrypoints,
            configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
            configuration::RuntimeConfigurationGenerationState::PendingGeneration,
            [true; 12],
        ),
        Err(configuration::RuntimeReconfigurationAdmissionError::TargetOwnerMismatch)
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
            11,
            configuration::RuntimeReconfigurationAdmissionError::TestProfileSwapEvidenceNotTestOnly,
        ),
    ] {
        let mut flags = [true; 12];
        flags[index] = false;
        let class = if index == 11 {
            configuration::RuntimeReconfigurationClass::TestProfileSwap
        } else {
            configuration::RuntimeReconfigurationClass::SecretRotationReload
        };
        assert_eq!(
            runtime_admission_guard(
                class,
                configuration::RuntimeReconfigurationTargetSurface::SecurityMaterial,
                ConfigurationOwner::Drivers,
                configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
                configuration::RuntimeConfigurationGenerationState::PendingGeneration,
                flags,
            ),
            Err(expected)
        );
    }
    assert_eq!(
        runtime_admission_guard(
            configuration::RuntimeReconfigurationClass::RuntimePolicyHotSwap,
            configuration::RuntimeReconfigurationTargetSurface::CorePolicy,
            ConfigurationOwner::Core,
            configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
            configuration::RuntimeConfigurationGenerationState::PendingGeneration,
            [true; 12],
        ),
        Err(configuration::RuntimeReconfigurationAdmissionError::ReconfigurationClassNotAdmitted)
    );
    assert_eq!(
        runtime_admission_guard(
            configuration::RuntimeReconfigurationClass::SecretRotationReload,
            configuration::RuntimeReconfigurationTargetSurface::SecurityMaterial,
            ConfigurationOwner::Drivers,
            configuration::RuntimeConfigurationGenerationState::PendingGeneration,
            configuration::RuntimeConfigurationGenerationState::PendingGeneration,
            [true; 12],
        ),
        Err(configuration::RuntimeReconfigurationAdmissionError::CurrentGenerationStateInvalid)
    );
    assert_eq!(
        runtime_admission_guard(
            configuration::RuntimeReconfigurationClass::SecretRotationReload,
            configuration::RuntimeReconfigurationTargetSurface::SecurityMaterial,
            ConfigurationOwner::Drivers,
            configuration::RuntimeConfigurationGenerationState::CurrentGeneration,
            configuration::RuntimeConfigurationGenerationState::RejectedGeneration,
            [true; 12],
        ),
        Err(configuration::RuntimeReconfigurationAdmissionError::ProposedGenerationStateInvalid)
    );

    assert_copy_debug_hash(
        configuration::RuntimeReconfigurationApplyGuard::try_new(
            configuration::RuntimeReconfigurationTargetSurface::ObservabilityExport,
            true,
            true,
            false,
            true,
        )
        .expect("non-sensitive target does not require drain rule"),
    );
    assert_copy_debug_hash(
        configuration::RuntimeReconfigurationApplyGuard::try_new(
            configuration::RuntimeReconfigurationTargetSurface::PublicEndpointExposure,
            true,
            true,
            true,
            true,
        )
        .expect("sensitive target admits explicit drain rule"),
    );
    assert_eq!(
        configuration::RuntimeReconfigurationApplyGuard::try_new(
            configuration::RuntimeReconfigurationTargetSurface::ObservabilityExport,
            false,
            true,
            true,
            true,
        ),
        Err(configuration::RuntimeReconfigurationApplyError::AcceptedDecisionGenerationRewritten)
    );
    assert_eq!(
        configuration::RuntimeReconfigurationApplyGuard::try_new(
            configuration::RuntimeReconfigurationTargetSurface::ObservabilityExport,
            true,
            false,
            true,
            true,
        ),
        Err(configuration::RuntimeReconfigurationApplyError::ActiveScopeMigrationRuleMissing)
    );
    assert_eq!(
        configuration::RuntimeReconfigurationApplyGuard::try_new(
            configuration::RuntimeReconfigurationTargetSurface::PublicEndpointExposure,
            true,
            true,
            false,
            true,
        ),
        Err(configuration::RuntimeReconfigurationApplyError::DrainOrRestartRuleMissing)
    );
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

    assert_copy_debug_hash(
        configuration::RuntimeReconfigurationRollbackGuard::try_new(
            configuration::RuntimeConfigurationGenerationState::RollbackGeneration,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .expect("rollback guard admits complete input"),
    );
}

#[test]
fn configuration_feature_capability_guards_are_exhaustive() {
    for class in [
        configuration::RuntimeReconfigurationClass::StartupOnly,
        configuration::RuntimeReconfigurationClass::SecretRotationReload,
        configuration::RuntimeReconfigurationClass::ObservabilityExportReload,
        configuration::RuntimeReconfigurationClass::MaintenanceModeSwitch,
        configuration::RuntimeReconfigurationClass::TestProfileSwap,
        configuration::RuntimeReconfigurationClass::RuntimePolicyHotSwap,
    ] {
        assert_copy_debug_hash(class);
    }
    assert!(!configuration::RuntimeReconfigurationClass::StartupOnly.admits_runtime_apply());
    assert!(
        configuration::RuntimeReconfigurationClass::SecretRotationReload.admits_runtime_apply()
    );
    assert!(configuration::RuntimeReconfigurationClass::TestProfileSwap.is_test_evidence_only());

    for (surface, owner) in [
        (
            configuration::CapabilityDeclarationSurface::CoreProtocol,
            configuration::FeatureCapabilityAuthorityOwner::Core,
        ),
        (
            configuration::CapabilityDeclarationSurface::FeatureFlagValue,
            configuration::FeatureCapabilityAuthorityOwner::EntrypointsConfig,
        ),
        (
            configuration::CapabilityDeclarationSurface::DriverImplementationSelection,
            configuration::FeatureCapabilityAuthorityOwner::EntrypointsComposition,
        ),
        (
            configuration::CapabilityDeclarationSurface::SdkCapabilityExposure,
            configuration::FeatureCapabilityAuthorityOwner::Sdk,
        ),
        (
            configuration::CapabilityDeclarationSurface::ExperimentalLifecycleDecision,
            configuration::FeatureCapabilityAuthorityOwner::AdrCanonical,
        ),
        (
            configuration::CapabilityDeclarationSurface::OutOfScopeFeatureAdmission,
            configuration::FeatureCapabilityAuthorityOwner::AdrCanonical,
        ),
        (
            configuration::CapabilityDeclarationSurface::RuntimeEnablementEvidence,
            configuration::FeatureCapabilityAuthorityOwner::Reports,
        ),
        (
            configuration::CapabilityDeclarationSurface::RuntimeFlagProfileChange,
            configuration::FeatureCapabilityAuthorityOwner::RuntimeReconfigurationCanonical,
        ),
    ] {
        assert_copy_debug_hash(surface);
        assert_copy_debug_hash(owner);
        assert!(surface.authority_owner_matches(owner));
        assert_copy_debug_hash(
            configuration::CapabilityDeclarationGuard::try_new(capability_input(
                surface,
                ConfigurationOwner::Entrypoints,
                owner,
                [true; 6],
            ))
            .expect("capability declaration admits complete input"),
        );
    }
    assert_eq!(
        configuration::CapabilityDeclarationGuard::try_new(capability_input(
            configuration::CapabilityDeclarationSurface::FeatureFlagValue,
            ConfigurationOwner::Core,
            configuration::FeatureCapabilityAuthorityOwner::EntrypointsConfig,
            [true; 6],
        )),
        Err(configuration::CapabilityDeclarationError::ConfigurationWiringOwnerMismatch)
    );
    assert_eq!(
        configuration::CapabilityDeclarationGuard::try_new(capability_input(
            configuration::CapabilityDeclarationSurface::FeatureFlagValue,
            ConfigurationOwner::Entrypoints,
            configuration::FeatureCapabilityAuthorityOwner::Core,
            [true; 6],
        )),
        Err(configuration::CapabilityDeclarationError::SurfaceAuthorityOwnerMismatch)
    );
    for (index, expected) in [
        (
            0,
            configuration::CapabilityDeclarationError::AcceptedContractVersionMissing,
        ),
        (
            1,
            configuration::CapabilityDeclarationError::OptionalBehaviorMissing,
        ),
        (
            2,
            configuration::CapabilityDeclarationError::FallbackWhenAbsentMissing,
        ),
        (
            3,
            configuration::CapabilityDeclarationError::RequiredAbsentReasonMissing,
        ),
        (
            4,
            configuration::CapabilityDeclarationError::SdkParityRequirementMissing,
        ),
        (
            5,
            configuration::CapabilityDeclarationError::EvidenceClassRequirementMissing,
        ),
    ] {
        let mut flags = [true; 6];
        flags[index] = false;
        assert_eq!(
            configuration::CapabilityDeclarationGuard::try_new(capability_input(
                configuration::CapabilityDeclarationSurface::FeatureFlagValue,
                ConfigurationOwner::Entrypoints,
                configuration::FeatureCapabilityAuthorityOwner::EntrypointsConfig,
                flags,
            )),
            Err(expected)
        );
    }

    for flag in [
        configuration::FeatureFlagClass::DriverSelection,
        configuration::FeatureFlagClass::ExporterSelection,
        configuration::FeatureFlagClass::ProfileSelection,
        configuration::FeatureFlagClass::ExperimentalSurfaceGate,
        configuration::FeatureFlagClass::TestOnlyGate,
    ] {
        assert_copy_debug_hash(flag);
        assert!(!flag.may_change_core_decision());
    }
    assert!(configuration::FeatureFlagClass::TestOnlyGate.is_test_only());
    assert_copy_debug_hash(
        feature_admission_guard(configuration::FeatureFlagClass::DriverSelection, [true; 11])
            .expect("feature admission admits complete input"),
    );
    for (index, expected) in [
        (0, configuration::FeatureCapabilityAdmissionError::FlagClassMissing),
        (1, configuration::FeatureCapabilityAdmissionError::AllowedUseMissing),
        (2, configuration::FeatureCapabilityAdmissionError::ProhibitedUseNotBlocked),
        (3, configuration::FeatureCapabilityAdmissionError::FlagUsedAsProtocolVersion),
        (
            4,
            configuration::FeatureCapabilityAdmissionError::CapabilityChangesRequiredStateTransition,
        ),
        (5, configuration::FeatureCapabilityAdmissionError::AbsentCapabilityFallsBackSilently),
        (6, configuration::FeatureCapabilityAdmissionError::ExperimentalSurfaceGateMissing),
        (7, configuration::FeatureCapabilityAdmissionError::OutOfScopeAdmissionCanonicalMissing),
        (8, configuration::FeatureCapabilityAdmissionError::SdkCapabilityServerContractMismatch),
        (9, configuration::FeatureCapabilityAdmissionError::TestOnlyGateUsedOutsideTestEvidence),
        (
            10,
            configuration::FeatureCapabilityAdmissionError::RuntimeFlagChangeLacksReconfigurationScope,
        ),
    ] {
        let mut flags = [true; 11];
        flags[index] = false;
        let flag = if index == 9 {
            configuration::FeatureFlagClass::TestOnlyGate
        } else {
            configuration::FeatureFlagClass::DriverSelection
        };
        assert_eq!(feature_admission_guard(flag, flags), Err(expected));
    }

    for stage in [
        configuration::ExperimentalLifecycleStage::DraftDocumented,
        configuration::ExperimentalLifecycleStage::GatedScaffold,
        configuration::ExperimentalLifecycleStage::GatedImplemented,
        configuration::ExperimentalLifecycleStage::ControlledIntegration,
        configuration::ExperimentalLifecycleStage::AdoptedContract,
        configuration::ExperimentalLifecycleStage::Removed,
    ] {
        assert_copy_debug_hash(stage);
        assert_copy_debug_hash(
            configuration::ExperimentalLifecycleGuard::try_new(experimental_input(
                stage, [true; 7],
            ))
            .expect("experimental lifecycle admits complete input"),
        );
    }
    for (stage, flags, expected) in [
        (
            configuration::ExperimentalLifecycleStage::DraftDocumented,
            [false, true, true, true, true, true, true],
            configuration::ExperimentalLifecycleError::ScopeOrOwnerMissing,
        ),
        (
            configuration::ExperimentalLifecycleStage::GatedScaffold,
            [true, false, true, true, true, true, true],
            configuration::ExperimentalLifecycleError::ExplicitGateMissing,
        ),
        (
            configuration::ExperimentalLifecycleStage::GatedScaffold,
            [true, true, false, true, true, true, true],
            configuration::ExperimentalLifecycleError::DependencyDirectionEvidenceMissing,
        ),
        (
            configuration::ExperimentalLifecycleStage::GatedImplemented,
            [true, true, true, false, true, true, true],
            configuration::ExperimentalLifecycleError::UnitOrContractEvidenceMissing,
        ),
        (
            configuration::ExperimentalLifecycleStage::ControlledIntegration,
            [true, true, true, true, false, true, true],
            configuration::ExperimentalLifecycleError::IntegrationReportMissing,
        ),
        (
            configuration::ExperimentalLifecycleStage::AdoptedContract,
            [true, true, true, true, true, false, true],
            configuration::ExperimentalLifecycleError::AdoptionCanonicalOrCompatibilityRuleMissing,
        ),
        (
            configuration::ExperimentalLifecycleStage::Removed,
            [true, true, true, true, true, true, false],
            configuration::ExperimentalLifecycleError::RemovalCompatibilityLifecycleMissing,
        ),
    ] {
        assert_eq!(
            configuration::ExperimentalLifecycleGuard::try_new(experimental_input(stage, flags)),
            Err(expected)
        );
    }

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
        assert_copy_debug_hash(kind);
        assert_cataloged(kind.reason_code());
        assert_copy_debug_hash(configuration::RuntimeReconfigurationFailure::from_kind(
            kind,
        ));
    }
    assert_copy_debug_hash(
        configuration::RuntimeReconfigurationEvidenceGuard::try_new(
            true, true, true, true, true, true, true, true, true, true, true, true,
        )
        .expect("runtime reconfiguration evidence admits complete input"),
    );
    for (index, expected) in [
        (0, configuration::RuntimeReconfigurationEvidenceError::ReconfigurationClassMissing),
        (1, configuration::RuntimeReconfigurationEvidenceError::TargetSurfaceMissing),
        (2, configuration::RuntimeReconfigurationEvidenceError::CurrentGenerationReferenceMissing),
        (3, configuration::RuntimeReconfigurationEvidenceError::ProposedGenerationReferenceMissing),
        (4, configuration::RuntimeReconfigurationEvidenceError::ValidationProcedureMissing),
        (5, configuration::RuntimeReconfigurationEvidenceError::ApplyScopeMissing),
        (6, configuration::RuntimeReconfigurationEvidenceError::AffectedActiveScopeMissing),
        (7, configuration::RuntimeReconfigurationEvidenceError::DrainOrRestartDecisionMissing),
        (8, configuration::RuntimeReconfigurationEvidenceError::RollbackStatusMissing),
        (9, configuration::RuntimeReconfigurationEvidenceError::AuditEventReferenceMissing),
        (10, configuration::RuntimeReconfigurationEvidenceError::RerunConditionMissing),
        (
            11,
            configuration::RuntimeReconfigurationEvidenceError::StartupEvidenceUsedAsRuntimeReconfigurationEvidence,
        ),
    ] {
        let mut flags = [true; 12];
        flags[index] = false;
        assert_eq!(
            configuration::RuntimeReconfigurationEvidenceGuard::try_new(
                flags[0], flags[1], flags[2], flags[3], flags[4], flags[5], flags[6], flags[7],
                flags[8], flags[9], flags[10], flags[11],
            ),
            Err(expected)
        );
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
        assert_copy_debug_hash(kind);
        assert_cataloged(kind.reason_code());
        assert_copy_debug_hash(configuration::FeatureCapabilityFailure::from_kind(kind));
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
        assert_copy_debug_hash(unsupported);
        assert_cataloged(unsupported.reason_code());
        assert_copy_debug_hash(
            configuration::FeatureCapabilityFailure::from_unsupported_version_reason(unsupported),
        );
    }
}

#[test]
fn endpoints_closed_vocabularies_and_reason_catalogs_are_exercised() {
    assert_copy_debug_eq(endpoints::EntrypointEndpointsSurface);

    for (class, contract, protocol, public, private, test_only) in [
        (
            endpoints::PublicEndpointClass::SignalingPublic,
            endpoints::EndpointTargetContract::Signaling,
            endpoints::PublicEndpointProtocolClass::WebSocket,
            true,
            false,
            false,
        ),
        (
            endpoints::PublicEndpointClass::TurnPublicRelay,
            endpoints::EndpointTargetContract::Turn,
            endpoints::PublicEndpointProtocolClass::Udp,
            true,
            false,
            false,
        ),
        (
            endpoints::PublicEndpointClass::SfuMediaPublic,
            endpoints::EndpointTargetContract::Sfu,
            endpoints::PublicEndpointProtocolClass::SecureMedia,
            true,
            false,
            false,
        ),
        (
            endpoints::PublicEndpointClass::HealthPublicReadonly,
            endpoints::EndpointTargetContract::HealthReadonly,
            endpoints::PublicEndpointProtocolClass::Http,
            true,
            false,
            false,
        ),
        (
            endpoints::PublicEndpointClass::AdminPrivate,
            endpoints::EndpointTargetContract::OperatorAdmin,
            endpoints::PublicEndpointProtocolClass::WebSocket,
            false,
            true,
            false,
        ),
        (
            endpoints::PublicEndpointClass::InternalControlPrivate,
            endpoints::EndpointTargetContract::InternalControl,
            endpoints::PublicEndpointProtocolClass::TlsTcp,
            false,
            true,
            false,
        ),
        (
            endpoints::PublicEndpointClass::TestOnlyEndpoint,
            endpoints::EndpointTargetContract::TestDouble,
            endpoints::PublicEndpointProtocolClass::Tcp,
            false,
            false,
            true,
        ),
    ] {
        assert_copy_debug_hash(class);
        assert!(class.admits_target_contract(contract));
        assert!(class.admits_protocol_class(protocol));
        assert_eq!(class.is_public_surface(), public);
        assert_eq!(class.is_private_control_or_admin(), private);
        assert_eq!(class.is_test_only(), test_only);
    }
    assert!(!endpoints::PublicEndpointClass::SignalingPublic
        .admits_target_contract(endpoints::EndpointTargetContract::Sfu));
    assert!(!endpoints::PublicEndpointClass::SfuMediaPublic
        .admits_protocol_class(endpoints::PublicEndpointProtocolClass::Http));

    for scope in [
        endpoints::EndpointExposureScope::LocalTestOnly,
        endpoints::EndpointExposureScope::PrivateNetwork,
        endpoints::EndpointExposureScope::PublicEdgeTerminated,
        endpoints::EndpointExposureScope::DirectPublic,
    ] {
        assert_copy_debug_hash(scope);
    }
    assert!(endpoints::EndpointExposureScope::LocalTestOnly.is_local_test_only());
    assert_eq!(
        endpoints::PublicEndpointAuditEventType::PublicEndpointConnectionDecision.event_type(),
        "public_endpoint_connection_decision"
    );

    for failure in [
        endpoints::PublicEndpointFailureKind::PublicEndpointNotAllowed,
        endpoints::PublicEndpointFailureKind::PublicEndpointVersionUnsupported,
        endpoints::PublicEndpointFailureKind::PublicEndpointAuthRequired,
        endpoints::PublicEndpointFailureKind::PublicEndpointUpgradeFailed,
        endpoints::PublicEndpointFailureKind::ConnectionLifecycleViolation,
        endpoints::PublicEndpointFailureKind::ConnectionIdleTimeout,
        endpoints::PublicEndpointFailureKind::ConnectionClosePolicyViolation,
    ] {
        assert_copy_debug_hash(failure);
        assert_cataloged(failure.reason_code());
        assert_copy_debug_hash(endpoints::PublicEndpointFailure::from_kind(failure));
    }
    for prohibited in [
        endpoints::ProhibitedPublicEndpointBehavior::ListenerExistenceTreatedAsReadiness,
        endpoints::ProhibitedPublicEndpointBehavior::DriverSocketStateTreatedAsDomainState,
        endpoints::ProhibitedPublicEndpointBehavior::InternalOrAdminRouteExposedPublicByDefault,
        endpoints::ProhibitedPublicEndpointBehavior::WebSocketUpgradeTreatedAsParticipantAdmission,
        endpoints::ProhibitedPublicEndpointBehavior::ListenerBindTreatedAsDomainSuccess,
        endpoints::ProhibitedPublicEndpointBehavior::PublicErrorHidesCatalogedCoreReason,
        endpoints::ProhibitedPublicEndpointBehavior::EndpointClassAddedByNamingConvention,
        endpoints::ProhibitedPublicEndpointBehavior::ProxyMetadataChangesSeparationWithoutTrustAdmission,
        endpoints::ProhibitedPublicEndpointBehavior::DiscoveryChangesSeparationWithoutEndpointAdmission,
    ] {
        assert_copy_debug_hash(prohibited);
    }

    for metadata in [
        endpoints::TrustedMetadataClass::ForwardedFor,
        endpoints::TrustedMetadataClass::ForwardedProto,
        endpoints::TrustedMetadataClass::ForwardedHost,
        endpoints::TrustedMetadataClass::OriginHeader,
        endpoints::TrustedMetadataClass::HostHeader,
        endpoints::TrustedMetadataClass::SniHost,
        endpoints::TrustedMetadataClass::ClientAddressObservation,
        endpoints::TrustedMetadataClass::EdgeRequestId,
    ] {
        assert_copy_debug_hash(metadata);
        assert!(metadata.raw_value_must_not_be_core_identity());
    }
    for (edge, metadata) in [
        (
            endpoints::EdgeProxyClass::DirectPublicListener,
            endpoints::TrustedMetadataClass::ClientAddressObservation,
        ),
        (
            endpoints::EdgeProxyClass::ReverseProxyHttpWs,
            endpoints::TrustedMetadataClass::ForwardedFor,
        ),
        (
            endpoints::EdgeProxyClass::TcpUdpLoadBalancer,
            endpoints::TrustedMetadataClass::ClientAddressObservation,
        ),
        (
            endpoints::EdgeProxyClass::TlsTerminatingEdge,
            endpoints::TrustedMetadataClass::SniHost,
        ),
        (
            endpoints::EdgeProxyClass::ServiceMeshIngress,
            endpoints::TrustedMetadataClass::EdgeRequestId,
        ),
        (
            endpoints::EdgeProxyClass::TestEdgeSimulator,
            endpoints::TrustedMetadataClass::OriginHeader,
        ),
    ] {
        assert_copy_debug_hash(edge);
        assert!(edge.admits_metadata_class(metadata));
    }
    assert!(endpoints::EdgeProxyClass::TestEdgeSimulator.is_test_only());
    assert!(!endpoints::EdgeProxyClass::DirectPublicListener
        .admits_metadata_class(endpoints::TrustedMetadataClass::ForwardedFor));
    assert_eq!(
        endpoints::EdgeProxyAuditEventType::EdgeProxyTrustDecision.event_type(),
        "edge_proxy_trust_decision"
    );

    for failure in [
        endpoints::EdgeProxyTrustFailureKind::EdgeProxyNotAdmitted,
        endpoints::EdgeProxyTrustFailureKind::ForwardedHeaderUntrusted,
        endpoints::EdgeProxyTrustFailureKind::ForwardedHeaderChainInvalid,
        endpoints::EdgeProxyTrustFailureKind::OriginHostNotAllowed,
        endpoints::EdgeProxyTrustFailureKind::ClientAddressUntrusted,
        endpoints::EdgeProxyTrustFailureKind::TlsTerminationBoundaryInvalid,
        endpoints::EdgeProxyTrustFailureKind::PublicInternalRouteConfusion,
    ] {
        assert_copy_debug_hash(failure);
        assert_cataloged(failure.reason_code());
        assert_copy_debug_hash(endpoints::EdgeProxyTrustFailure::from_kind(failure));
    }
    for prohibited in [
        endpoints::ProhibitedEdgeProxyTrustBehavior::ForwardedHeaderTrustedByStandardName,
        endpoints::ProhibitedEdgeProxyTrustBehavior::RawEdgeMetadataBecomesCoreIdentity,
        endpoints::ProhibitedEdgeProxyTrustBehavior::RouteSeparationDependsOnlyOnProxyNaming,
        endpoints::ProhibitedEdgeProxyTrustBehavior::EdgeTlsTerminationTreatedAsBackendOrMediaProof,
        endpoints::ProhibitedEdgeProxyTrustBehavior::RawForwardedHeaderUsedForPolicyWithoutTrust,
        endpoints::ProhibitedEdgeProxyTrustBehavior::ProxyRequestIdReplacesCorrelationId,
        endpoints::ProhibitedEdgeProxyTrustBehavior::MeshIdentityBecomesApplicationAuthorizationByDefault,
    ] {
        assert_copy_debug_hash(prohibited);
    }
    for termination in [
        endpoints::EdgeTlsTerminationClass::NoEdgeTermination,
        endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
        endpoints::EdgeTlsTerminationClass::TerminationNotAdmitted,
    ] {
        assert_copy_debug_hash(termination);
    }
}

#[test]
fn endpoint_declaration_lifecycle_and_evidence_guards_are_exhaustive() {
    assert_copy_debug_hash(
        endpoint_declaration_guard(
            endpoints::PublicEndpointClass::SignalingPublic,
            endpoints::EndpointExposureScope::PublicEdgeTerminated,
            endpoints::PublicEndpointProtocolClass::WebSocket,
            endpoints::EndpointTargetContract::Signaling,
            [true; 11],
        )
        .expect("public endpoint declaration admits complete input"),
    );
    assert_copy_debug_hash(
        endpoint_declaration_guard(
            endpoints::PublicEndpointClass::TestOnlyEndpoint,
            endpoints::EndpointExposureScope::LocalTestOnly,
            endpoints::PublicEndpointProtocolClass::Http,
            endpoints::EndpointTargetContract::TestDouble,
            [
                true, true, true, true, true, true, false, true, true, true, true,
            ],
        )
        .expect("test-only endpoint stays local and skips exposed transport requirement"),
    );
    for (index, expected) in [
        (
            0,
            endpoints::PublicEndpointDeclarationError::EndpointClassMissing,
        ),
        (
            1,
            endpoints::PublicEndpointDeclarationError::ServiceDiscoveryScopeMissing,
        ),
        (
            2,
            endpoints::PublicEndpointDeclarationError::ProtocolOrListenerOwnerMissing,
        ),
        (
            3,
            endpoints::PublicEndpointDeclarationError::EdgeProxyTrustPolicyMissing,
        ),
        (
            4,
            endpoints::PublicEndpointDeclarationError::TargetContractMissing,
        ),
        (
            5,
            endpoints::PublicEndpointDeclarationError::AuthenticationAuthorizationMissing,
        ),
        (
            6,
            endpoints::PublicEndpointDeclarationError::TransportSecurityProfileMissing,
        ),
        (
            7,
            endpoints::PublicEndpointDeclarationError::BoundPolicyMissing,
        ),
        (
            8,
            endpoints::PublicEndpointDeclarationError::CorrelationPropagationMissing,
        ),
        (
            9,
            endpoints::PublicEndpointDeclarationError::PublicErrorMappingMissing,
        ),
        (
            10,
            endpoints::PublicEndpointDeclarationError::EndpointMetadataRedactionMissing,
        ),
    ] {
        let mut flags = [true; 11];
        flags[index] = false;
        assert_eq!(
            endpoint_declaration_guard(
                endpoints::PublicEndpointClass::SignalingPublic,
                endpoints::EndpointExposureScope::DirectPublic,
                endpoints::PublicEndpointProtocolClass::WebSocket,
                endpoints::EndpointTargetContract::Signaling,
                flags,
            ),
            Err(expected)
        );
    }
    assert_eq!(
        endpoint_declaration_guard(
            endpoints::PublicEndpointClass::SignalingPublic,
            endpoints::EndpointExposureScope::DirectPublic,
            endpoints::PublicEndpointProtocolClass::SecureMedia,
            endpoints::EndpointTargetContract::Signaling,
            [true; 11],
        ),
        Err(endpoints::PublicEndpointDeclarationError::EndpointProtocolClassMismatch)
    );
    assert_eq!(
        endpoint_declaration_guard(
            endpoints::PublicEndpointClass::SignalingPublic,
            endpoints::EndpointExposureScope::DirectPublic,
            endpoints::PublicEndpointProtocolClass::WebSocket,
            endpoints::EndpointTargetContract::Sfu,
            [true; 11],
        ),
        Err(endpoints::PublicEndpointDeclarationError::EndpointTargetContractMismatch)
    );
    assert_eq!(
        endpoint_declaration_guard(
            endpoints::PublicEndpointClass::TestOnlyEndpoint,
            endpoints::EndpointExposureScope::PublicEdgeTerminated,
            endpoints::PublicEndpointProtocolClass::Http,
            endpoints::EndpointTargetContract::TestDouble,
            [true; 11],
        ),
        Err(endpoints::PublicEndpointDeclarationError::TestOnlyEndpointExposedOutsideLocalTest)
    );

    for state in [
        endpoints::ConnectionLifecycleState::PreOpen,
        endpoints::ConnectionLifecycleState::ProtocolChecked,
        endpoints::ConnectionLifecycleState::SecurityChecked,
        endpoints::ConnectionLifecycleState::CoreAdmitted,
        endpoints::ConnectionLifecycleState::Active,
        endpoints::ConnectionLifecycleState::Draining,
        endpoints::ConnectionLifecycleState::IdleExpired,
        endpoints::ConnectionLifecycleState::ClosedSuccess,
        endpoints::ConnectionLifecycleState::ClosedByPolicy,
        endpoints::ConnectionLifecycleState::Failed,
    ] {
        assert_copy_debug_hash(state);
    }
    assert!(endpoints::ConnectionLifecycleState::Active.is_active_traffic_state());
    assert!(endpoints::ConnectionLifecycleState::Failed.requires_close_or_failure_reason());
    assert_copy_debug_hash(
        lifecycle_guard(endpoints::ConnectionLifecycleState::Active, [true; 7])
            .expect("active lifecycle requires protocol/security/core admission"),
    );
    assert_copy_debug_hash(
        lifecycle_guard(
            endpoints::ConnectionLifecycleState::ClosedSuccess,
            [true; 7],
        )
        .expect("closed success carries audit or reference material"),
    );
    for (state, flags, expected) in [
        (
            endpoints::ConnectionLifecycleState::PreOpen,
            [false, true, true, true, true, true, true],
            endpoints::ConnectionLifecycleTransitionError::ProtocolCheckMissing,
        ),
        (
            endpoints::ConnectionLifecycleState::Active,
            [true, false, true, true, true, true, true],
            endpoints::ConnectionLifecycleTransitionError::SecurityCheckMissing,
        ),
        (
            endpoints::ConnectionLifecycleState::Active,
            [true, true, false, true, true, true, true],
            endpoints::ConnectionLifecycleTransitionError::CoreAdmissionMissing,
        ),
        (
            endpoints::ConnectionLifecycleState::PreOpen,
            [true, true, true, false, true, true, true],
            endpoints::ConnectionLifecycleTransitionError::PhysicalOpenUsedAsDomainAdmission,
        ),
        (
            endpoints::ConnectionLifecycleState::PreOpen,
            [true, true, true, true, false, true, true],
            endpoints::ConnectionLifecycleTransitionError::CoreRejectionAfterPhysicalOpenNotRepresented,
        ),
        (
            endpoints::ConnectionLifecycleState::Failed,
            [true, true, true, true, true, false, true],
            endpoints::ConnectionLifecycleTransitionError::CloseOrFailureReasonMissing,
        ),
        (
            endpoints::ConnectionLifecycleState::ClosedSuccess,
            [true, true, true, true, true, true, false],
            endpoints::ConnectionLifecycleTransitionError::CloseSuccessAuditReferenceMissing,
        ),
    ] {
        assert_eq!(lifecycle_guard(state, flags), Err(expected));
    }

    assert_copy_debug_hash(
        endpoints::PublicInternalEndpointSeparationGuard::try_new(
            endpoints::PublicEndpointClass::AdminPrivate,
            true,
            true,
            true,
            true,
        )
        .expect("private endpoint stays non-public with private auth canonical"),
    );
    assert_eq!(
        endpoints::PublicInternalEndpointSeparationGuard::try_new(
            endpoints::PublicEndpointClass::AdminPrivate,
            false,
            true,
            true,
            true,
        ),
        Err(endpoints::PublicInternalEndpointSeparationError::PrivateEndpointPublicByDefault)
    );
    assert_eq!(
        endpoints::PublicInternalEndpointSeparationGuard::try_new(
            endpoints::PublicEndpointClass::AdminPrivate,
            true,
            false,
            true,
            true,
        ),
        Err(endpoints::PublicInternalEndpointSeparationError::RouteLevelPrivateClassMissing)
    );
    assert_eq!(
        endpoints::PublicInternalEndpointSeparationGuard::try_new(
            endpoints::PublicEndpointClass::AdminPrivate,
            true,
            true,
            false,
            true,
        ),
        Err(endpoints::PublicInternalEndpointSeparationError::PrivateRouteAuthorizationCanonicalMissing)
    );
    assert_eq!(
        endpoints::PublicInternalEndpointSeparationGuard::try_new(
            endpoints::PublicEndpointClass::SignalingPublic,
            true,
            true,
            true,
            false,
        ),
        Err(endpoints::PublicInternalEndpointSeparationError::PublicEndpointInheritsInternalServiceTrust)
    );

    assert_copy_debug_hash(
        public_endpoint_evidence_guard([true; 10])
            .expect("public endpoint evidence admits complete input"),
    );
    for (index, expected) in [
        (
            0,
            endpoints::PublicEndpointEvidenceError::EndpointClassMissing,
        ),
        (1, endpoints::PublicEndpointEvidenceError::ProtocolMissing),
        (
            2,
            endpoints::PublicEndpointEvidenceError::ListenerOwnerMissing,
        ),
        (
            3,
            endpoints::PublicEndpointEvidenceError::TargetContractMissing,
        ),
        (
            4,
            endpoints::PublicEndpointEvidenceError::AuthSecurityProfileMissing,
        ),
        (
            5,
            endpoints::PublicEndpointEvidenceError::BoundPolicyMissing,
        ),
        (
            6,
            endpoints::PublicEndpointEvidenceError::CorrelationRuleMissing,
        ),
        (
            7,
            endpoints::PublicEndpointEvidenceError::LifecycleTransitionMissing,
        ),
        (
            8,
            endpoints::PublicEndpointEvidenceError::DiscoveryEvidenceMissing,
        ),
        (
            9,
            endpoints::PublicEndpointEvidenceError::ListenerStartupUsedAsReadinessEvidence,
        ),
    ] {
        let mut flags = [true; 10];
        flags[index] = false;
        assert_eq!(public_endpoint_evidence_guard(flags), Err(expected));
    }
}

#[test]
fn endpoint_edge_proxy_guards_are_exhaustive() {
    assert_copy_debug_hash(
        edge_trust_admission_guard(
            endpoints::EdgeProxyClass::ReverseProxyHttpWs,
            endpoints::TrustedMetadataClass::ForwardedFor,
            [true; 13],
        )
        .expect("edge trust admission admits complete input"),
    );
    for (index, expected) in [
        (0, endpoints::EdgeProxyTrustAdmissionError::EdgeClassMissing),
        (1, endpoints::EdgeProxyTrustAdmissionError::DeploymentTopologyClassMissing),
        (2, endpoints::EdgeProxyTrustAdmissionError::TrustedUpstreamScopeMissing),
        (3, endpoints::EdgeProxyTrustAdmissionError::AcceptedMetadataClassMissing),
        (4, endpoints::EdgeProxyTrustAdmissionError::HeaderPrecedenceConflictRuleMissing),
        (5, endpoints::EdgeProxyTrustAdmissionError::MaximumHopCountMissing),
        (6, endpoints::EdgeProxyTrustAdmissionError::TlsTerminationDownstreamSecurityRelationMissing),
        (7, endpoints::EdgeProxyTrustAdmissionError::OriginHostAdmissionRuleMissing),
        (8, endpoints::EdgeProxyTrustAdmissionError::ClientAddressUseLimitMissing),
        (9, endpoints::EdgeProxyTrustAdmissionError::RateQuotaAdmissionRelationMissing),
        (10, endpoints::EdgeProxyTrustAdmissionError::AuditReferenceRuleMissing),
        (11, endpoints::EdgeProxyTrustAdmissionError::RedactionRuleMissing),
    ] {
        let mut flags = [true; 13];
        flags[index] = false;
        assert_eq!(
            edge_trust_admission_guard(
                endpoints::EdgeProxyClass::ReverseProxyHttpWs,
                endpoints::TrustedMetadataClass::ForwardedFor,
                flags,
            ),
            Err(expected)
        );
    }
    assert_eq!(
        edge_trust_admission_guard(
            endpoints::EdgeProxyClass::DirectPublicListener,
            endpoints::TrustedMetadataClass::ForwardedFor,
            [true; 13],
        ),
        Err(endpoints::EdgeProxyTrustAdmissionError::MetadataClassNotAdmittedForEdgeClass)
    );
    assert_eq!(
        edge_trust_admission_guard(
            endpoints::EdgeProxyClass::TestEdgeSimulator,
            endpoints::TrustedMetadataClass::ForwardedFor,
            [true, true, true, true, true, true, true, true, true, true, true, true, false,],
        ),
        Err(endpoints::EdgeProxyTrustAdmissionError::TestEdgeUsedOutsideTestEvidence)
    );

    assert_copy_debug_hash(
        endpoints::EdgeProxyHeaderSourceGuard::try_new(
            endpoints::EdgeProxyClass::ReverseProxyHttpWs,
            endpoints::TrustedMetadataClass::ForwardedHost,
            true,
            true,
            true,
            true,
            true,
        )
        .expect("header source admits complete input"),
    );
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
        endpoints::EdgeProxyHeaderSourceGuard::try_new(
            endpoints::EdgeProxyClass::ReverseProxyHttpWs,
            endpoints::TrustedMetadataClass::ForwardedFor,
            false,
            true,
            true,
            true,
            true,
        ),
        Err(endpoints::EdgeProxyHeaderSourceError::ImmediateUpstreamUntrusted)
    );
    assert_eq!(
        endpoints::EdgeProxyHeaderSourceGuard::try_new(
            endpoints::EdgeProxyClass::ReverseProxyHttpWs,
            endpoints::TrustedMetadataClass::ForwardedFor,
            true,
            false,
            true,
            true,
            true,
        ),
        Err(endpoints::EdgeProxyHeaderSourceError::DeterministicPrecedenceMissing)
    );
    assert_eq!(
        endpoints::EdgeProxyHeaderSourceGuard::try_new(
            endpoints::EdgeProxyClass::ReverseProxyHttpWs,
            endpoints::TrustedMetadataClass::ForwardedFor,
            true,
            true,
            false,
            true,
            true,
        ),
        Err(endpoints::EdgeProxyHeaderSourceError::RawMetadataUsedAsCoreIdentity)
    );
    assert_eq!(
        endpoints::EdgeProxyHeaderSourceGuard::try_new(
            endpoints::EdgeProxyClass::ReverseProxyHttpWs,
            endpoints::TrustedMetadataClass::ForwardedFor,
            true,
            true,
            true,
            false,
            true,
        ),
        Err(endpoints::EdgeProxyHeaderSourceError::TypedPolicyInputMissing)
    );
    assert_eq!(
        endpoints::EdgeProxyHeaderSourceGuard::try_new(
            endpoints::EdgeProxyClass::ReverseProxyHttpWs,
            endpoints::TrustedMetadataClass::EdgeRequestId,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(endpoints::EdgeProxyHeaderSourceError::ProxyRequestIdReplacesCorrelationId)
    );

    assert_copy_debug_hash(
        endpoints::EdgeTlsTerminationGuard::try_new(
            endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .expect("tls termination admits protected backend relation"),
    );
    for (class, flags, expected) in [
        (
            endpoints::EdgeTlsTerminationClass::TerminationNotAdmitted,
            [true, true, true, true, true, true],
            endpoints::EdgeTlsTerminationError::TerminationClassNotAdmitted,
        ),
        (
            endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
            [false, true, true, true, true, true],
            endpoints::EdgeTlsTerminationError::BackendTransportSecurityRequirementMissing,
        ),
        (
            endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
            [true, false, true, true, true, true],
            endpoints::EdgeTlsTerminationError::TrustedEdgeIdentityMissing,
        ),
        (
            endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
            [true, true, false, true, true, true],
            endpoints::EdgeTlsTerminationError::CertificateSecretReferenceClassMissing,
        ),
        (
            endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
            [true, true, true, false, true, true],
            endpoints::EdgeTlsTerminationError::AuditEvidenceRelationMissing,
        ),
        (
            endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
            [true, true, true, true, false, true],
            endpoints::EdgeTlsTerminationError::MissingDownstreamProtectionFailureMappingMissing,
        ),
        (
            endpoints::EdgeTlsTerminationClass::TrustedEdgeTerminatesWithProtectedBackend,
            [true, true, true, true, true, false],
            endpoints::EdgeTlsTerminationError::EdgeTlsUsedAsSecureMediaProof,
        ),
    ] {
        assert_eq!(
            endpoints::EdgeTlsTerminationGuard::try_new(
                class, flags[0], flags[1], flags[2], flags[3], flags[4], flags[5],
            ),
            Err(expected)
        );
    }

    assert_copy_debug_hash(
        edge_trust_evidence_guard([true; 13]).expect("edge trust evidence admits complete input"),
    );
    for (index, expected) in [
        (0, endpoints::EdgeProxyTrustEvidenceError::EdgeClassMissing),
        (
            1,
            endpoints::EdgeProxyTrustEvidenceError::TopologyClassMissing,
        ),
        (
            2,
            endpoints::EdgeProxyTrustEvidenceError::TrustedUpstreamScopeMissing,
        ),
        (
            3,
            endpoints::EdgeProxyTrustEvidenceError::AcceptedMetadataClassesMissing,
        ),
        (
            4,
            endpoints::EdgeProxyTrustEvidenceError::HeaderPrecedenceMissing,
        ),
        (5, endpoints::EdgeProxyTrustEvidenceError::HopCountMissing),
        (
            6,
            endpoints::EdgeProxyTrustEvidenceError::TlsTerminationRelationMissing,
        ),
        (
            7,
            endpoints::EdgeProxyTrustEvidenceError::OriginHostPolicyMissing,
        ),
        (
            8,
            endpoints::EdgeProxyTrustEvidenceError::ClientAddressUseLimitMissing,
        ),
        (
            9,
            endpoints::EdgeProxyTrustEvidenceError::CommandProcedureMissing,
        ),
        (
            10,
            endpoints::EdgeProxyTrustEvidenceError::WorkingDirectoryMissing,
        ),
        (
            11,
            endpoints::EdgeProxyTrustEvidenceError::RerunConditionMissing,
        ),
        (
            12,
            endpoints::EdgeProxyTrustEvidenceError::DiagnosticSnippetUsedAsEvidence,
        ),
    ] {
        let mut flags = [true; 13];
        flags[index] = false;
        assert_eq!(edge_trust_evidence_guard(flags), Err(expected));
    }
}
