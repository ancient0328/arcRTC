// entrypoints/configuration は entrypoint composition 用の configuration / policy wiring surface です。
//
// core configuration semantics は core/configuration が所有し、ここでは
// 起動時に選択される profile や policy bundle の接続面だけを扱います。

use arcrtc_core_configuration::{ConfigurationOwner, CoreConfigurationSurface};
use arcrtc_core_features::{CoreFeaturesSurface, FeatureAdmissionFailureKind};
use arcrtc_core_reason::CatalogedReasonRef;

/// entrypoint configuration package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntrypointConfigurationSurface;

/// v0.2 initial architecture の profile class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationProfileClass {
    /// local manual run profile. production evidence にはしません。
    DevelopmentLocal,
    /// deterministic clock/RNG/fake driver profile. test evidence only.
    TestDeterministic,
    /// controlled integration profile.
    IntegrationControlled,
    /// benchmark profile.
    BenchmarkControlled,
    /// production-like candidate profile.
    ProductionCandidate,
}

impl ConfigurationProfileClass {
    /// profile class ごとの evidence adoption rule を返します。
    pub const fn adoption_rule(self) -> ConfigurationProfileAdoptionRule {
        match self {
            Self::DevelopmentLocal => ConfigurationProfileAdoptionRule::LocalManualOnly,
            Self::TestDeterministic => ConfigurationProfileAdoptionRule::TestEvidenceOnly,
            Self::IntegrationControlled => {
                ConfigurationProfileAdoptionRule::IntegrationEvidenceOnly
            }
            Self::BenchmarkControlled => ConfigurationProfileAdoptionRule::BenchmarkEvidenceOnly,
            Self::ProductionCandidate => {
                ConfigurationProfileAdoptionRule::ProductionCandidateRequiresExplicitReport
            }
        }
    }
}

/// profile evidence / claim class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationProfileEvidenceClaimClass {
    /// local manual run evidence.
    LocalManualEvidence,
    /// deterministic test evidence.
    TestEvidence,
    /// controlled integration evidence.
    IntegrationEvidence,
    /// benchmark evidence.
    BenchmarkEvidence,
    /// runtime claim.
    RuntimeClaim,
    /// production evidence claim.
    ProductionEvidence,
}

/// profile class ごとの採用規則です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationProfileAdoptionRule {
    /// development_local は local manual evidence にだけ採用します。
    LocalManualOnly,
    /// test_deterministic は test evidence にだけ採用します。
    TestEvidenceOnly,
    /// integration_controlled は integration evidence にだけ採用します。
    IntegrationEvidenceOnly,
    /// benchmark_controlled は benchmark evidence にだけ採用します。
    BenchmarkEvidenceOnly,
    /// production_candidate の runtime/prod claim は明示 report 参照を必須にします。
    ProductionCandidateRequiresExplicitReport,
}

impl ConfigurationProfileAdoptionRule {
    /// profile class と evidence/claim class の対応を fail-closed で判定します。
    pub const fn admits_claim(
        self,
        claim_class: ConfigurationProfileEvidenceClaimClass,
        explicit_evidence_report_reference_present: bool,
    ) -> Result<(), ConfigurationProfileEvidenceError> {
        match self {
            Self::LocalManualOnly => {
                if matches!(
                    claim_class,
                    ConfigurationProfileEvidenceClaimClass::LocalManualEvidence
                ) {
                    Ok(())
                } else {
                    Err(ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted)
                }
            }
            Self::TestEvidenceOnly => {
                if matches!(
                    claim_class,
                    ConfigurationProfileEvidenceClaimClass::TestEvidence
                ) {
                    Ok(())
                } else {
                    Err(ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted)
                }
            }
            Self::IntegrationEvidenceOnly => {
                if matches!(
                    claim_class,
                    ConfigurationProfileEvidenceClaimClass::IntegrationEvidence
                ) {
                    Ok(())
                } else {
                    Err(ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted)
                }
            }
            Self::BenchmarkEvidenceOnly => {
                if matches!(
                    claim_class,
                    ConfigurationProfileEvidenceClaimClass::BenchmarkEvidence
                ) {
                    Ok(())
                } else {
                    Err(ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted)
                }
            }
            Self::ProductionCandidateRequiresExplicitReport => {
                if !matches!(
                    claim_class,
                    ConfigurationProfileEvidenceClaimClass::RuntimeClaim
                        | ConfigurationProfileEvidenceClaimClass::ProductionEvidence
                ) {
                    return Err(ConfigurationProfileEvidenceError::ProfileClaimClassNotAdmitted);
                }
                if !explicit_evidence_report_reference_present {
                    return Err(
                        ConfigurationProfileEvidenceError::ExplicitEvidenceReportReferenceMissing,
                    );
                }
                Ok(())
            }
        }
    }
}

/// configuration / policy bundle class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationBundleClass {
    /// core thresholds, accepted versions, bounds, security requirements.
    CorePolicy,
    /// driver socket/DB/exporter/TLS/key source runtime config.
    DriverRuntime,
    /// selected drivers and startup mode.
    EntrypointComposition,
    /// entrypoint/service topology, discovery, node affinity.
    DeploymentTopology,
    /// discovery source, endpoint scope, TTL/cache, fallback.
    ServiceDiscovery,
    /// service-to-service trust class and credential/peer proof.
    InternalServiceTrust,
    /// state class, owner scope, affinity/failover/replication admission.
    DistributedState,
    /// task class, supervision scope, join/cancel bound.
    RuntimeTask,
    /// secret source references, accepted generations, overlap/revocation.
    SecretRotation,
    /// dependency, license, vulnerability, lockfile, toolchain evidence.
    SupplyChain,
    /// SDK client-local profile.
    SdkClient,
    /// regulated optional mapping bundle.
    RegulatedMapping,
    /// deterministic/fake settings for test evidence only.
    TestProfile,
}

/// configuration entrypoint が参照する core surfaces です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigurationWiringSet {
    core_configuration: CoreConfigurationSurface,
    core_features: CoreFeaturesSurface,
}

impl ConfigurationWiringSet {
    /// entrypoint configuration wiring が core configuration/features を参照することを示します。
    pub const fn new(
        core_configuration: CoreConfigurationSurface,
        core_features: CoreFeaturesSurface,
    ) -> Self {
        Self {
            core_configuration,
            core_features,
        }
    }
}

/// startup/wiring 時の bundle validation order を確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigurationBundleValidationGuard {
    profile_class: ConfigurationProfileClass,
    entrypoint_composition_bundle_parseable_and_complete: bool,
    selected_driver_runtime_bundles_present_and_valid: bool,
    core_policy_bundle_present_and_accepted_by_core: bool,
    cross_bundle_references_resolve_without_raw_secret_leakage: bool,
    feature_capability_settings_allowed: bool,
    deployment_topology_class_explicit_and_accepted: bool,
    service_discovery_explicit_when_configured: bool,
    distributed_state_explicit_when_topology_requires: bool,
    internal_service_trust_explicit_when_required: bool,
    runtime_task_supervision_explicit_when_required: bool,
    secret_rotation_policy_present_when_required: bool,
    supply_chain_evidence_class_declared_for_build_claim: bool,
    profile_evidence_class_declared: bool,
    partial_acceptance_admitted_by_canonical: bool,
    startup_validation_not_runtime_hotswap_permission: bool,
}

/// configuration bundle validation guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationBundleValidationError {
    /// entrypoint composition bundle が parseable/complete ではありません。
    EntrypointCompositionBundleMissing,
    /// selected driver runtime bundle が不足または invalid です。
    DriverRuntimeBundleInvalid,
    /// core policy bundle が core に受理されていません。
    CorePolicyBundleInvalid,
    /// cross-bundle reference 解決で raw secret leakage があります。
    CrossBundleReferenceInvalid,
    /// feature/capability setting が許可されていません。
    FeatureCapabilityNotAllowed,
    /// deployment topology class が明示/受理されていません。
    DeploymentTopologyMissing,
    /// service discovery bundle が必要時に明示されていません。
    ServiceDiscoveryMissing,
    /// distributed state bundle が必要時に明示されていません。
    DistributedStateMissing,
    /// internal service trust bundle が必要時に明示されていません。
    InternalServiceTrustMissing,
    /// runtime task bundle が必要時に明示されていません。
    RuntimeTaskMissing,
    /// secret rotation policy が必要時にありません。
    SecretRotationPolicyMissing,
    /// build/release claim に supply-chain evidence class がありません。
    SupplyChainEvidenceMissing,
    /// profile evidence class がありません。
    ProfileEvidenceClassMissing,
    /// partial bundle acceptance が Canonical で認められていません。
    PartialAcceptanceNotAdmitted,
    /// startup validation を runtime hotswap permission として扱っています。
    StartupValidationAsRuntimeHotSwapPermission,
}

impl ConfigurationBundleValidationGuard {
    /// Configuration Profile Policy Bundle Canonical の validation order を検査します。
    pub const fn try_new(
        profile_class: ConfigurationProfileClass,
        entrypoint_composition_bundle_parseable_and_complete: bool,
        selected_driver_runtime_bundles_present_and_valid: bool,
        core_policy_bundle_present_and_accepted_by_core: bool,
        cross_bundle_references_resolve_without_raw_secret_leakage: bool,
        feature_capability_settings_allowed: bool,
        deployment_topology_class_explicit_and_accepted: bool,
        service_discovery_explicit_when_configured: bool,
        distributed_state_explicit_when_topology_requires: bool,
        internal_service_trust_explicit_when_required: bool,
        runtime_task_supervision_explicit_when_required: bool,
        secret_rotation_policy_present_when_required: bool,
        supply_chain_evidence_class_declared_for_build_claim: bool,
        profile_evidence_class_declared: bool,
        partial_acceptance_admitted_by_canonical: bool,
        startup_validation_not_runtime_hotswap_permission: bool,
    ) -> Result<Self, ConfigurationBundleValidationError> {
        if !entrypoint_composition_bundle_parseable_and_complete {
            return Err(ConfigurationBundleValidationError::EntrypointCompositionBundleMissing);
        }
        if !selected_driver_runtime_bundles_present_and_valid {
            return Err(ConfigurationBundleValidationError::DriverRuntimeBundleInvalid);
        }
        if !core_policy_bundle_present_and_accepted_by_core {
            return Err(ConfigurationBundleValidationError::CorePolicyBundleInvalid);
        }
        if !cross_bundle_references_resolve_without_raw_secret_leakage {
            return Err(ConfigurationBundleValidationError::CrossBundleReferenceInvalid);
        }
        if !feature_capability_settings_allowed {
            return Err(ConfigurationBundleValidationError::FeatureCapabilityNotAllowed);
        }
        if !deployment_topology_class_explicit_and_accepted {
            return Err(ConfigurationBundleValidationError::DeploymentTopologyMissing);
        }
        if !service_discovery_explicit_when_configured {
            return Err(ConfigurationBundleValidationError::ServiceDiscoveryMissing);
        }
        if !distributed_state_explicit_when_topology_requires {
            return Err(ConfigurationBundleValidationError::DistributedStateMissing);
        }
        if !internal_service_trust_explicit_when_required {
            return Err(ConfigurationBundleValidationError::InternalServiceTrustMissing);
        }
        if !runtime_task_supervision_explicit_when_required {
            return Err(ConfigurationBundleValidationError::RuntimeTaskMissing);
        }
        if !secret_rotation_policy_present_when_required {
            return Err(ConfigurationBundleValidationError::SecretRotationPolicyMissing);
        }
        if !supply_chain_evidence_class_declared_for_build_claim {
            return Err(ConfigurationBundleValidationError::SupplyChainEvidenceMissing);
        }
        if !profile_evidence_class_declared {
            return Err(ConfigurationBundleValidationError::ProfileEvidenceClassMissing);
        }
        if !partial_acceptance_admitted_by_canonical {
            return Err(ConfigurationBundleValidationError::PartialAcceptanceNotAdmitted);
        }
        if !startup_validation_not_runtime_hotswap_permission {
            return Err(
                ConfigurationBundleValidationError::StartupValidationAsRuntimeHotSwapPermission,
            );
        }

        Ok(Self {
            profile_class,
            entrypoint_composition_bundle_parseable_and_complete,
            selected_driver_runtime_bundles_present_and_valid,
            core_policy_bundle_present_and_accepted_by_core,
            cross_bundle_references_resolve_without_raw_secret_leakage,
            feature_capability_settings_allowed,
            deployment_topology_class_explicit_and_accepted,
            service_discovery_explicit_when_configured,
            distributed_state_explicit_when_topology_requires,
            internal_service_trust_explicit_when_required,
            runtime_task_supervision_explicit_when_required,
            secret_rotation_policy_present_when_required,
            supply_chain_evidence_class_declared_for_build_claim,
            profile_evidence_class_declared,
            partial_acceptance_admitted_by_canonical,
            startup_validation_not_runtime_hotswap_permission,
        })
    }
}

/// configuration profile / policy bundle failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationBundleFailureKind {
    /// required entrypoint/runtime bundle missing.
    RuntimeConfigMissing,
    /// entrypoint/runtime bundle cannot initialize selected driver/entrypoint.
    RuntimeConfigInvalid,
    /// core policy bundle invalid.
    CorePolicyConfigInvalid,
    /// required secret source unavailable.
    SecretUnavailable,
    /// secret rotation policy/state unavailable where required.
    SecretRotationStateUnavailable,
    /// selected deployment topology unsupported.
    DeploymentTopologyUnsupported,
    /// service discovery source unavailable.
    ServiceDiscoveryUnavailable,
    /// service discovery source is not admitted.
    ServiceDiscoverySourceNotAdmitted,
    /// endpoint scope conflict.
    ServiceEndpointScopeConflict,
    /// endpoint fallback not allowed.
    ServiceEndpointFallbackNotAllowed,
    /// distributed state not admitted.
    DistributedStateNotAdmitted,
    /// state replication not admitted.
    StateReplicationNotAdmitted,
    /// consensus not admitted.
    ConsensusNotAdmitted,
    /// failover not proven.
    FailoverNotProven,
    /// internal service trust class is not admitted.
    InternalServiceIdentitySourceNotAdmitted,
    /// required internal service identity is absent.
    InternalServiceIdentityMissing,
    /// internal service identity material cannot be mapped.
    InternalServiceIdentityInvalid,
    /// internal service identity cannot be trusted for target path.
    InternalServiceIdentityUntrusted,
    /// internal service identity scope conflicts with target.
    InternalServiceIdentityScopeConflict,
    /// peer verification failed for internal service trust.
    InternalServicePeerVerificationFailed,
    /// internal service credential or peer proof expired.
    InternalServiceCredentialExpired,
    /// required internal service trust policy is absent.
    InternalServiceTrustPolicyMissing,
    /// internal control authorization context is absent after trust mapping.
    InternalControlAuthorizationMissing,
    /// internal control authorization denied the call.
    InternalControlAuthorizationDenied,
    /// runtime task class is not admitted.
    RuntimeTaskClassNotAdmitted,
    /// runtime task owner/supervision scope is invalid.
    RuntimeTaskOwnerViolation,
    /// runtime task has no admitted supervision scope.
    RuntimeTaskSupervisionMissing,
    /// detached runtime task is requested.
    RuntimeTaskDetachedNotAllowed,
    /// runtime cannot spawn required task.
    RuntimeTaskSpawnFailed,
    /// task join/wait observation failed.
    RuntimeTaskJoinFailed,
    /// task cancellation failed or could not be observed.
    RuntimeTaskCancelFailed,
    /// task panic was observed.
    RuntimeTaskPanicDetected,
    /// task queue/mailbox/join bound exceeded.
    RuntimeTaskQueueBoundExceeded,
    /// driver/runtime is shutting down.
    DriverShutdown,
    /// toolchain version mismatch.
    ToolchainVersionMismatch,
    /// lockfile drift detected.
    LockfileDriftDetected,
    /// dependency policy violation.
    DependencyPolicyViolation,
    /// license policy violation.
    LicensePolicyViolation,
    /// vulnerability gate failed.
    VulnerabilityGateFailed,
    /// selected feature/capability disabled.
    CapabilityNotEnabled,
    /// runtime profile swap not admitted.
    RuntimeReconfigurationNotAllowed,
}

impl ConfigurationBundleFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::CorePolicyConfigInvalid => "core_policy_config_invalid",
            Self::SecretUnavailable => "secret_unavailable",
            Self::SecretRotationStateUnavailable => "secret_rotation_state_unavailable",
            Self::DeploymentTopologyUnsupported => "deployment_topology_unsupported",
            Self::ServiceDiscoveryUnavailable => "service_discovery_unavailable",
            Self::ServiceDiscoverySourceNotAdmitted => "service_discovery_source_not_admitted",
            Self::ServiceEndpointScopeConflict => "service_endpoint_scope_conflict",
            Self::ServiceEndpointFallbackNotAllowed => "service_endpoint_fallback_not_allowed",
            Self::DistributedStateNotAdmitted => "distributed_state_not_admitted",
            Self::StateReplicationNotAdmitted => "state_replication_not_admitted",
            Self::ConsensusNotAdmitted => "consensus_not_admitted",
            Self::FailoverNotProven => "failover_not_proven",
            Self::InternalServiceIdentitySourceNotAdmitted => {
                "internal_service_identity_source_not_admitted"
            }
            Self::InternalServiceIdentityMissing => "internal_service_identity_missing",
            Self::InternalServiceIdentityInvalid => "internal_service_identity_invalid",
            Self::InternalServiceIdentityUntrusted => "internal_service_identity_untrusted",
            Self::InternalServiceIdentityScopeConflict => {
                "internal_service_identity_scope_conflict"
            }
            Self::InternalServicePeerVerificationFailed => {
                "internal_service_peer_verification_failed"
            }
            Self::InternalServiceCredentialExpired => "internal_service_credential_expired",
            Self::InternalServiceTrustPolicyMissing => "internal_service_trust_policy_missing",
            Self::InternalControlAuthorizationMissing => "internal_control_authorization_missing",
            Self::InternalControlAuthorizationDenied => "internal_control_authorization_denied",
            Self::RuntimeTaskClassNotAdmitted => "runtime_task_class_not_admitted",
            Self::RuntimeTaskOwnerViolation => "runtime_task_owner_violation",
            Self::RuntimeTaskSupervisionMissing => "runtime_task_supervision_missing",
            Self::RuntimeTaskDetachedNotAllowed => "runtime_task_detached_not_allowed",
            Self::RuntimeTaskSpawnFailed => "runtime_task_spawn_failed",
            Self::RuntimeTaskJoinFailed => "runtime_task_join_failed",
            Self::RuntimeTaskCancelFailed => "runtime_task_cancel_failed",
            Self::RuntimeTaskPanicDetected => "runtime_task_panic_detected",
            Self::RuntimeTaskQueueBoundExceeded => "runtime_task_queue_bound_exceeded",
            Self::DriverShutdown => "driver_shutdown",
            Self::ToolchainVersionMismatch => "toolchain_version_mismatch",
            Self::LockfileDriftDetected => "lockfile_drift_detected",
            Self::DependencyPolicyViolation => "dependency_policy_violation",
            Self::LicensePolicyViolation => "license_policy_violation",
            Self::VulnerabilityGateFailed => "vulnerability_gate_failed",
            Self::CapabilityNotEnabled => "capability_not_enabled",
            Self::RuntimeReconfigurationNotAllowed => "runtime_reconfiguration_not_allowed",
        }
    }
}

/// configuration bundle failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigurationBundleFailure {
    kind: ConfigurationBundleFailureKind,
    reason: CatalogedReasonRef,
}

impl ConfigurationBundleFailure {
    /// bundle failure を cataloged reason に接続します。
    pub fn from_kind(kind: ConfigurationBundleFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("configuration bundle reason code must be registered");
        Self { kind, reason }
    }
}

/// profile evidence adoption の guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigurationProfileEvidenceGuard {
    profile_class: ConfigurationProfileClass,
    claim_class: ConfigurationProfileEvidenceClaimClass,
    profile_class_declared: bool,
    bundle_sources_declared_without_raw_secret_material: bool,
    explicit_evidence_report_reference_present: bool,
    evidence_not_promoted_to_another_profile_without_new_report: bool,
    build_or_release_claim_depends_on_artifact: bool,
    supply_chain_identity_declared_when_build_claim_depends_on_artifact: bool,
    runtime_profile_change_claimed: bool,
    runtime_change_includes_reconfiguration_class_and_generation: bool,
}

