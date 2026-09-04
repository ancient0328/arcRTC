/// configuration profile / policy bundle 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedConfigurationProfileBundleBehavior {
    /// entrypoint profile changes core semantics silently.
    EntrypointProfileChangesCoreSemanticsSilently,
    /// driver runtime config derives core policy.
    DriverRuntimeConfigDerivesCorePolicy,
    /// missing required bundle falls back to default.
    MissingRequiredBundleFallsBackToDefault,
    /// test profile becomes production profile.
    TestProfileBecomesProductionProfile,
    /// feature flag enables experimental surface without lifecycle rule.
    FeatureFlagEnablesExperimentalSurfaceWithoutLifecycle,
    /// profile selection omits profile class.
    ProfileSelectionOmitsProfileClass,
    /// topology/secret/supply-chain bundle is implicit default.
    ImplicitTopologySecretOrSupplyChainBundle,
    /// service discovery / distributed state bundle is implicit default.
    ImplicitDiscoveryOrDistributedStateBundle,
    /// internal service trust / runtime task bundle is implicit default.
    ImplicitInternalTrustOrRuntimeTaskBundle,
    /// build/release selection omits supply-chain identity.
    BuildReleaseSelectionOmitsSupplyChainIdentity,
    /// startup validation is treated as runtime hot-swap permission.
    StartupValidationAsRuntimeHotSwapPermission,
}

/// runtime reconfiguration class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeReconfigurationClass {
    /// restart と startup validation が必要な既定 class です。
    StartupOnly,
    /// secret rotation policy 配下の key/secret reload です。
    SecretRotationReload,
    /// observability taxonomy bounds 内の exporter reload です。
    ObservabilityExportReload,
    /// maintenance/drain mode の enter/exit です。
    MaintenanceModeSwitch,
    /// deterministic/test-only profile swap です。
    TestProfileSwap,
    /// restart なしの core policy generation 変更です。v0.2 では許可しません。
    RuntimePolicyHotSwap,
}

impl RuntimeReconfigurationClass {
    /// runtime apply をこの class 自体が許可し得るかを返します。
    pub const fn admits_runtime_apply(self) -> bool {
        matches!(
            self,
            Self::SecretRotationReload
                | Self::ObservabilityExportReload
                | Self::MaintenanceModeSwitch
                | Self::TestProfileSwap
        )
    }

    /// test scope だけに閉じる class かを返します。
    pub const fn is_test_only(self) -> bool {
        matches!(self, Self::TestProfileSwap)
    }
}

/// runtime reconfiguration の target surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeReconfigurationTargetSurface {
    /// core policy generation.
    CorePolicy,
    /// driver runtime configuration.
    DriverRuntime,
    /// entrypoint composition configuration.
    EntrypointComposition,
    /// feature/capability configuration.
    FeatureCapability,
    /// public endpoint exposure.
    PublicEndpointExposure,
    /// security material reference/reload.
    SecurityMaterial,
    /// deployment topology.
    DeploymentTopology,
    /// observability exporter/signal configuration.
    ObservabilityExport,
    /// test profile only.
    TestProfile,
}

impl RuntimeReconfigurationTargetSurface {
    /// target surface と owner の対応を entrypoints 側 wiring で fail-closed 判定します。
    pub const fn owner_matches(self, owner: ConfigurationOwner) -> bool {
        matches!(
            (self, owner),
            (Self::CorePolicy, ConfigurationOwner::Core)
                | (Self::DriverRuntime, ConfigurationOwner::Drivers)
                | (Self::EntrypointComposition, ConfigurationOwner::Entrypoints)
                | (Self::FeatureCapability, ConfigurationOwner::Entrypoints)
                | (Self::PublicEndpointExposure, ConfigurationOwner::Entrypoints)
                | (Self::SecurityMaterial, ConfigurationOwner::Drivers)
                | (Self::DeploymentTopology, ConfigurationOwner::Entrypoints)
                | (Self::ObservabilityExport, ConfigurationOwner::Drivers)
                | (Self::TestProfile, ConfigurationOwner::Entrypoints)
        )
    }

    /// active decision に影響するため drain/restart rule を要する surface です。
    pub const fn requires_drain_or_restart_when_runtime_changed(self) -> bool {
        matches!(
            self,
            Self::CorePolicy
                | Self::EntrypointComposition
                | Self::FeatureCapability
                | Self::PublicEndpointExposure
                | Self::SecurityMaterial
                | Self::DeploymentTopology
        )
    }
}

/// runtime configuration generation state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeConfigurationGenerationState {
    /// active and accepted for declared scope.
    CurrentGeneration,
    /// parsed but not accepted.
    PendingGeneration,
    /// under validation and not active.
    ValidatingGeneration,
    /// failed validation and must not apply.
    RejectedGeneration,
    /// prior accepted generation used for rollback.
    RollbackGeneration,
    /// no longer active and not accepted for new decisions.
    RetiredGeneration,
}

impl RuntimeConfigurationGenerationState {
    /// current generation として扱える state です。
    pub const fn is_current(self) -> bool {
        matches!(self, Self::CurrentGeneration)
    }

    /// proposed generation として validation 対象にできる state です。
    pub const fn is_proposed_candidate(self) -> bool {
        matches!(self, Self::PendingGeneration | Self::ValidatingGeneration)
    }

    /// rollback generation として参照できる state です。
    pub const fn is_rollback_candidate(self) -> bool {
        matches!(self, Self::RollbackGeneration)
    }
}

/// runtime reconfiguration audit event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeReconfigurationAuditEventType {
    /// runtime reconfiguration decision.
    RuntimeReconfigurationDecision,
}

impl RuntimeReconfigurationAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::RuntimeReconfigurationDecision => "runtime_reconfiguration_decision",
        }
    }
}

/// runtime reconfiguration admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeReconfigurationAdmissionGuard {
    reconfiguration_class: RuntimeReconfigurationClass,
    target_surface: RuntimeReconfigurationTargetSurface,
    target_owner: ConfigurationOwner,
    current_generation_state: RuntimeConfigurationGenerationState,
    proposed_generation_state: RuntimeConfigurationGenerationState,
    current_generation_reference_present: bool,
    proposed_generation_reference_present: bool,
    apply_scope_declared: bool,
    affected_active_scope_declared: bool,
    drain_or_restart_requirement_declared: bool,
    rollback_behavior_declared: bool,
    audit_event_type: RuntimeReconfigurationAuditEventType,
    runtime_apply_requested: bool,
    target_policy_admits_class: bool,
    class_specific_rule_satisfied: bool,
    test_profile_swap_is_test_only: bool,
    raw_config_or_secret_payload_absent_from_generation_state: bool,
}

/// runtime reconfiguration admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeReconfigurationAdmissionError {
    /// target owner が surface と一致していません。
    TargetOwnerMismatch,
    /// current generation reference がありません。
    CurrentGenerationReferenceMissing,
    /// proposed generation reference がありません。
    ProposedGenerationReferenceMissing,
    /// current generation state が current ではありません。
    CurrentGenerationStateInvalid,
    /// proposed generation state が pending/validating ではありません。
    ProposedGenerationStateInvalid,
    /// apply scope が記録されていません。
    ApplyScopeMissing,
    /// affected active scope が記録されていません。
    AffectedActiveScopeMissing,
    /// drain/restart requirement が記録されていません。
    DrainOrRestartRequirementMissing,
    /// rollback behavior が記録されていません。
    RollbackBehaviorMissing,
    /// class が runtime apply を許可しません。
    ReconfigurationClassNotAdmitted,
    /// target policy が class を許可していません。
    TargetPolicyDoesNotAdmitClass,
    /// class 固有の source policy 条件を満たしていません。
    ClassSpecificRuleMissing,
    /// test profile swap が test scope に閉じていません。
    TestProfileSwapUsedOutsideTestScope,
    /// generation state に raw config/secret payload が混入しています。
    RawPayloadInGenerationState,
}

impl RuntimeReconfigurationAdmissionGuard {
    /// runtime reconfiguration request の必須 field と class admission を検査します。
    pub const fn try_new(
        reconfiguration_class: RuntimeReconfigurationClass,
        target_surface: RuntimeReconfigurationTargetSurface,
        target_owner: ConfigurationOwner,
        current_generation_state: RuntimeConfigurationGenerationState,
        proposed_generation_state: RuntimeConfigurationGenerationState,
        current_generation_reference_present: bool,
        proposed_generation_reference_present: bool,
        apply_scope_declared: bool,
        affected_active_scope_declared: bool,
        drain_or_restart_requirement_declared: bool,
        rollback_behavior_declared: bool,
        audit_event_type: RuntimeReconfigurationAuditEventType,
        runtime_apply_requested: bool,
        target_policy_admits_class: bool,
        class_specific_rule_satisfied: bool,
        test_profile_swap_is_test_only: bool,
        raw_config_or_secret_payload_absent_from_generation_state: bool,
    ) -> Result<Self, RuntimeReconfigurationAdmissionError> {
        if !target_surface.owner_matches(target_owner) {
            return Err(RuntimeReconfigurationAdmissionError::TargetOwnerMismatch);
        }
        if !current_generation_reference_present {
            return Err(RuntimeReconfigurationAdmissionError::CurrentGenerationReferenceMissing);
        }
        if !proposed_generation_reference_present {
            return Err(RuntimeReconfigurationAdmissionError::ProposedGenerationReferenceMissing);
        }
        if !current_generation_state.is_current() {
            return Err(RuntimeReconfigurationAdmissionError::CurrentGenerationStateInvalid);
        }
        if !proposed_generation_state.is_proposed_candidate() {
            return Err(RuntimeReconfigurationAdmissionError::ProposedGenerationStateInvalid);
        }
        if !apply_scope_declared {
            return Err(RuntimeReconfigurationAdmissionError::ApplyScopeMissing);
        }
        if !affected_active_scope_declared {
            return Err(RuntimeReconfigurationAdmissionError::AffectedActiveScopeMissing);
        }
        if !drain_or_restart_requirement_declared {
            return Err(RuntimeReconfigurationAdmissionError::DrainOrRestartRequirementMissing);
        }
        if !rollback_behavior_declared {
            return Err(RuntimeReconfigurationAdmissionError::RollbackBehaviorMissing);
        }
        if runtime_apply_requested && !reconfiguration_class.admits_runtime_apply() {
            return Err(RuntimeReconfigurationAdmissionError::ReconfigurationClassNotAdmitted);
        }
        if runtime_apply_requested && !target_policy_admits_class {
            return Err(RuntimeReconfigurationAdmissionError::TargetPolicyDoesNotAdmitClass);
        }
        if runtime_apply_requested && !class_specific_rule_satisfied {
            return Err(RuntimeReconfigurationAdmissionError::ClassSpecificRuleMissing);
        }
        if reconfiguration_class.is_test_only() && !test_profile_swap_is_test_only {
            return Err(RuntimeReconfigurationAdmissionError::TestProfileSwapUsedOutsideTestScope);
        }
        if !raw_config_or_secret_payload_absent_from_generation_state {
            return Err(RuntimeReconfigurationAdmissionError::RawPayloadInGenerationState);
        }

        Ok(Self {
            reconfiguration_class,
            target_surface,
            target_owner,
            current_generation_state,
            proposed_generation_state,
            current_generation_reference_present,
            proposed_generation_reference_present,
            apply_scope_declared,
            affected_active_scope_declared,
            drain_or_restart_requirement_declared,
            rollback_behavior_declared,
            audit_event_type,
            runtime_apply_requested,
            target_policy_admits_class,
            class_specific_rule_satisfied,
            test_profile_swap_is_test_only,
            raw_config_or_secret_payload_absent_from_generation_state,
        })
    }
}

/// runtime reconfiguration apply boundary guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeReconfigurationApplyGuard {
    target_surface: RuntimeReconfigurationTargetSurface,
    accepted_domain_decisions_keep_original_generation: bool,
    target_policy_defines_migration_or_reevaluation_for_active_scope: bool,
    drain_or_restart_rule_present_for_sensitive_surface: bool,
    audit_hash_chain_scope_not_rewritten: bool,
}

/// runtime reconfiguration apply の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeReconfigurationApplyError {
    /// 既存の accepted domain decision を新 generation で再解釈しています。
    AcceptedDecisionGenerationRewritten,
    /// active scope migration/re-evaluation rule がありません。
    ActiveScopeMigrationRuleMissing,
    /// sensitive surface の drain/restart rule がありません。
    DrainOrRestartRuleMissing,
    /// audit hash-chain scope を書き換えています。
    AuditHashChainScopeRewritten,
}

impl RuntimeReconfigurationApplyGuard {
    /// apply 時の非遡及性と drain/restart 境界を検査します。
    pub const fn try_new(
        target_surface: RuntimeReconfigurationTargetSurface,
        accepted_domain_decisions_keep_original_generation: bool,
        target_policy_defines_migration_or_reevaluation_for_active_scope: bool,
        drain_or_restart_rule_present_for_sensitive_surface: bool,
        audit_hash_chain_scope_not_rewritten: bool,
    ) -> Result<Self, RuntimeReconfigurationApplyError> {
        if !accepted_domain_decisions_keep_original_generation {
            return Err(RuntimeReconfigurationApplyError::AcceptedDecisionGenerationRewritten);
        }
        if !target_policy_defines_migration_or_reevaluation_for_active_scope {
            return Err(RuntimeReconfigurationApplyError::ActiveScopeMigrationRuleMissing);
        }
        if target_surface.requires_drain_or_restart_when_runtime_changed()
            && !drain_or_restart_rule_present_for_sensitive_surface
        {
            return Err(RuntimeReconfigurationApplyError::DrainOrRestartRuleMissing);
        }
        if !audit_hash_chain_scope_not_rewritten {
            return Err(RuntimeReconfigurationApplyError::AuditHashChainScopeRewritten);
        }

        Ok(Self {
            target_surface,
            accepted_domain_decisions_keep_original_generation,
            target_policy_defines_migration_or_reevaluation_for_active_scope,
            drain_or_restart_rule_present_for_sensitive_surface,
            audit_hash_chain_scope_not_rewritten,
        })
    }
}

/// runtime reconfiguration rollback boundary guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeReconfigurationRollbackGuard {
    rollback_generation_state: RuntimeConfigurationGenerationState,
    rollback_generation_reference_present: bool,
    rollback_trigger_declared: bool,
    rollback_apply_scope_declared: bool,
    rollback_outcome_observed: bool,
    in_flight_operation_handling_declared: bool,
    rollback_failure_reason_declared_when_failed: bool,
}

/// runtime reconfiguration rollback の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeReconfigurationRollbackError {
    /// rollback generation reference がありません。
    RollbackGenerationReferenceMissing,
    /// rollback generation state が不正です。
    RollbackGenerationStateInvalid,
    /// rollback trigger がありません。
    RollbackTriggerMissing,
    /// rollback apply scope がありません。
    RollbackApplyScopeMissing,
    /// rollback outcome が観測されていません。
    RollbackOutcomeMissing,
    /// in-flight operation handling がありません。
    InFlightOperationHandlingMissing,
    /// rollback failure reason がありません。
    RollbackFailureReasonMissing,
}
