impl FeatureCapabilityAdmissionGuard {
    /// feature flag / capability が core semantics を暗黙変更しないことを検査します。
    pub const fn try_new(
        flag_class: FeatureFlagClass,
        flag_class_declared: bool,
        allowed_use_declared: bool,
        prohibited_use_blocked: bool,
        protocol_version_not_replaced_by_flag: bool,
        capability_not_used_to_change_required_state_transition: bool,
        absent_capability_fails_closed: bool,
        experimental_surface_has_explicit_gate_when_used: bool,
        out_of_scope_feature_has_admission_canonical_when_used: bool,
        sdk_capability_matches_server_contract_when_visible: bool,
        test_only_gate_evidence_is_test_only: bool,
        runtime_flag_change_uses_reconfiguration_generation_and_apply_scope: bool,
    ) -> Result<Self, FeatureCapabilityAdmissionError> {
        if !flag_class_declared {
            return Err(FeatureCapabilityAdmissionError::FlagClassMissing);
        }
        if !allowed_use_declared {
            return Err(FeatureCapabilityAdmissionError::AllowedUseMissing);
        }
        if !prohibited_use_blocked {
            return Err(FeatureCapabilityAdmissionError::ProhibitedUseNotBlocked);
        }
        if flag_class.may_change_core_decision() {
            return Err(FeatureCapabilityAdmissionError::ProhibitedUseNotBlocked);
        }
        if !protocol_version_not_replaced_by_flag {
            return Err(FeatureCapabilityAdmissionError::FlagUsedAsProtocolVersion);
        }
        if !capability_not_used_to_change_required_state_transition {
            return Err(FeatureCapabilityAdmissionError::CapabilityChangesRequiredStateTransition);
        }
        if !absent_capability_fails_closed {
            return Err(FeatureCapabilityAdmissionError::AbsentCapabilityFallsBackSilently);
        }
        if !experimental_surface_has_explicit_gate_when_used {
            return Err(FeatureCapabilityAdmissionError::ExperimentalSurfaceGateMissing);
        }
        if !out_of_scope_feature_has_admission_canonical_when_used {
            return Err(FeatureCapabilityAdmissionError::OutOfScopeAdmissionCanonicalMissing);
        }
        if !sdk_capability_matches_server_contract_when_visible {
            return Err(FeatureCapabilityAdmissionError::SdkCapabilityServerContractMismatch);
        }
        if flag_class.is_test_only() && !test_only_gate_evidence_is_test_only {
            return Err(FeatureCapabilityAdmissionError::TestOnlyGateUsedOutsideTestEvidence);
        }
        if !runtime_flag_change_uses_reconfiguration_generation_and_apply_scope {
            return Err(
                FeatureCapabilityAdmissionError::RuntimeFlagChangeLacksReconfigurationScope,
            );
        }

        Ok(Self {
            flag_class,
            flag_class_declared,
            allowed_use_declared,
            prohibited_use_blocked,
            protocol_version_not_replaced_by_flag,
            capability_not_used_to_change_required_state_transition,
            absent_capability_fails_closed,
            experimental_surface_has_explicit_gate_when_used,
            out_of_scope_feature_has_admission_canonical_when_used,
            sdk_capability_matches_server_contract_when_visible,
            test_only_gate_evidence_is_test_only,
            runtime_flag_change_uses_reconfiguration_generation_and_apply_scope,
        })
    }
}

/// experimental lifecycle stage guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExperimentalLifecycleGuard {
    stage: ExperimentalLifecycleStage,
    scope_and_owner_fixed: bool,
    explicit_gate_present_when_scaffold_or_later: bool,
    dependency_direction_evidence_present_when_scaffold: bool,
    unit_or_contract_evidence_present_when_implemented: bool,
    integration_report_present_when_controlled_integration: bool,
    adr_or_canonical_update_and_compatibility_rule_present_when_adopted: bool,
    compatibility_or_deprecation_lifecycle_satisfied_when_removed: bool,
}

/// experimental lifecycle guard 生成時の未検査入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExperimentalLifecycleGuardInput {
    pub stage: ExperimentalLifecycleStage,
    pub scope_and_owner_fixed: bool,
    pub explicit_gate_present_when_scaffold_or_later: bool,
    pub dependency_direction_evidence_present_when_scaffold: bool,
    pub unit_or_contract_evidence_present_when_implemented: bool,
    pub integration_report_present_when_controlled_integration: bool,
    pub adr_or_canonical_update_and_compatibility_rule_present_when_adopted: bool,
    pub compatibility_or_deprecation_lifecycle_satisfied_when_removed: bool,
}

/// experimental lifecycle の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExperimentalLifecycleError {
    /// scope と owner が固定されていません。
    ScopeOrOwnerMissing,
    /// gated stage に explicit gate がありません。
    ExplicitGateMissing,
    /// gated scaffold に dependency direction evidence がありません。
    DependencyDirectionEvidenceMissing,
    /// gated implementation に unit/contract evidence がありません。
    UnitOrContractEvidenceMissing,
    /// controlled integration に integration report がありません。
    IntegrationReportMissing,
    /// adopted contract に ADR/Canonical update と compatibility rule がありません。
    AdoptionCanonicalOrCompatibilityRuleMissing,
    /// removal に compatibility/deprecation lifecycle がありません。
    RemovalCompatibilityLifecycleMissing,
}

impl ExperimentalLifecycleGuard {
    /// experimental lifecycle stage の promotion condition を検査します。
    pub const fn try_new(
        input: ExperimentalLifecycleGuardInput,
    ) -> Result<Self, ExperimentalLifecycleError> {
        let ExperimentalLifecycleGuardInput {
            stage,
            scope_and_owner_fixed,
            explicit_gate_present_when_scaffold_or_later,
            dependency_direction_evidence_present_when_scaffold,
            unit_or_contract_evidence_present_when_implemented,
            integration_report_present_when_controlled_integration,
            adr_or_canonical_update_and_compatibility_rule_present_when_adopted,
            compatibility_or_deprecation_lifecycle_satisfied_when_removed,
        } = input;

        if !scope_and_owner_fixed {
            return Err(ExperimentalLifecycleError::ScopeOrOwnerMissing);
        }
        if matches!(
            stage,
            ExperimentalLifecycleStage::GatedScaffold
                | ExperimentalLifecycleStage::GatedImplemented
                | ExperimentalLifecycleStage::ControlledIntegration
                | ExperimentalLifecycleStage::AdoptedContract
        ) && !explicit_gate_present_when_scaffold_or_later
        {
            return Err(ExperimentalLifecycleError::ExplicitGateMissing);
        }
        if matches!(stage, ExperimentalLifecycleStage::GatedScaffold)
            && !dependency_direction_evidence_present_when_scaffold
        {
            return Err(ExperimentalLifecycleError::DependencyDirectionEvidenceMissing);
        }
        if matches!(stage, ExperimentalLifecycleStage::GatedImplemented)
            && !unit_or_contract_evidence_present_when_implemented
        {
            return Err(ExperimentalLifecycleError::UnitOrContractEvidenceMissing);
        }
        if matches!(stage, ExperimentalLifecycleStage::ControlledIntegration)
            && !integration_report_present_when_controlled_integration
        {
            return Err(ExperimentalLifecycleError::IntegrationReportMissing);
        }
        if matches!(stage, ExperimentalLifecycleStage::AdoptedContract)
            && !adr_or_canonical_update_and_compatibility_rule_present_when_adopted
        {
            return Err(ExperimentalLifecycleError::AdoptionCanonicalOrCompatibilityRuleMissing);
        }
        if matches!(stage, ExperimentalLifecycleStage::Removed)
            && !compatibility_or_deprecation_lifecycle_satisfied_when_removed
        {
            return Err(ExperimentalLifecycleError::RemovalCompatibilityLifecycleMissing);
        }

        Ok(Self {
            stage,
            scope_and_owner_fixed,
            explicit_gate_present_when_scaffold_or_later,
            dependency_direction_evidence_present_when_scaffold,
            unit_or_contract_evidence_present_when_implemented,
            integration_report_present_when_controlled_integration,
            adr_or_canonical_update_and_compatibility_rule_present_when_adopted,
            compatibility_or_deprecation_lifecycle_satisfied_when_removed,
        })
    }
}

/// feature/capability failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureCapabilityFailureKind {
    /// required capability absent.
    CapabilityNotEnabled,
    /// feature/profile configuration invalid at runtime/entrypoint layer.
    RuntimeConfigInvalid,
    /// feature/profile configuration invalid as core policy.
    CorePolicyConfigInvalid,
    /// out-of-scope feature admission failure.
    FeatureAdmissionFailure(FeatureAdmissionFailureKind),
    /// runtime flag change not admitted.
    RuntimeReconfigurationNotAllowed,
}

impl FeatureCapabilityFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::CapabilityNotEnabled => "capability_not_enabled",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::CorePolicyConfigInvalid => "core_policy_config_invalid",
            Self::FeatureAdmissionFailure(kind) => kind.reason_code(),
            Self::RuntimeReconfigurationNotAllowed => "runtime_reconfiguration_not_allowed",
        }
    }
}

/// feature/capability で許容する unsupported-version reason の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureCapabilityUnsupportedVersionReason {
    /// Signaling command/event version unsupported.
    UnsupportedCommandVersion,
    /// media-facing SFU/transport contract unsupported.
    UnsupportedMediaContractVersion,
    /// TURN contract version unsupported.
    UnsupportedTurnContractVersion,
    /// driver wire encoding unsupported.
    UnsupportedDriverWireVersion,
    /// public endpoint version unsupported.
    PublicEndpointVersionUnsupported,
    /// internal control contract version unsupported.
    InternalControlVersionUnsupported,
    /// token algorithm unsupported.
    TokenUnsupportedAlgorithm,
}

impl FeatureCapabilityUnsupportedVersionReason {
    /// surface-specific unsupported-version reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::UnsupportedCommandVersion => "unsupported_command_version",
            Self::UnsupportedMediaContractVersion => "unsupported_media_contract_version",
            Self::UnsupportedTurnContractVersion => "unsupported_turn_contract_version",
            Self::UnsupportedDriverWireVersion => "unsupported_driver_wire_version",
            Self::PublicEndpointVersionUnsupported => "public_endpoint_version_unsupported",
            Self::InternalControlVersionUnsupported => "internal_control_version_unsupported",
            Self::TokenUnsupportedAlgorithm => "token_unsupported_algorithm",
        }
    }
}

/// feature/capability failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureCapabilityFailure {
    reason: CatalogedReasonRef,
}

impl FeatureCapabilityFailure {
    /// closed failure kind を cataloged reason に接続します。
    pub fn from_kind(kind: FeatureCapabilityFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("feature/capability reason code must be registered");
        Self { reason }
    }

    /// surface-specific unsupported-version reason を cataloged reason に接続します。
    pub fn from_unsupported_version_reason(
        unsupported_version: FeatureCapabilityUnsupportedVersionReason,
    ) -> Self {
        let reason = CatalogedReasonRef::from_code(unsupported_version.reason_code())
            .expect("unsupported-version reason code must be registered");
        Self { reason }
    }
}

/// feature flag / capability lifecycle 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedFeatureCapabilityBehavior {
    /// feature flag changes core state machine without ADR/Canonical.
    FeatureFlagChangesCoreStateMachineWithoutCanonical,
    /// capability alters required behavior inside accepted version.
    CapabilityAltersRequiredBehaviorInsideAcceptedVersion,
    /// experimental surface is enabled by default.
    ExperimentalSurfaceEnabledByDefault,
    /// out-of-scope feature is enabled by flag without ADR/Canonical admission.
    OutOfScopeFeatureEnabledByFlagWithoutAdmission,
    /// SDK exposes capability that server contract does not define.
    SdkExposesUndefinedServerCapability,
    /// test-only gate is used as runtime/prod evidence.
    TestOnlyGateUsedAsRuntimeOrProductionEvidence,
    /// removal bypasses compatibility/deprecation lifecycle.
    RemovalBypassesCompatibilityDeprecationLifecycle,
    /// runtime flag change is treated as startup profile validation.
    RuntimeFlagChangeTreatedAsStartupProfileValidation,
}
