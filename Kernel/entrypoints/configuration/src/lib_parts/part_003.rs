impl RuntimeReconfigurationRollbackGuard {
    /// rollback を成功扱いする前に必要な evidence 境界を検査します。
    pub const fn try_new(
        rollback_generation_state: RuntimeConfigurationGenerationState,
        rollback_generation_reference_present: bool,
        rollback_trigger_declared: bool,
        rollback_apply_scope_declared: bool,
        rollback_evidence_declared: bool,
        in_flight_operation_handling_declared: bool,
        rollback_failure_reason_declared_when_failed: bool,
    ) -> Result<Self, RuntimeReconfigurationRollbackError> {
        if !rollback_generation_reference_present {
            return Err(RuntimeReconfigurationRollbackError::RollbackGenerationReferenceMissing);
        }
        if !rollback_generation_state.is_rollback_candidate() {
            return Err(RuntimeReconfigurationRollbackError::RollbackGenerationStateInvalid);
        }
        if !rollback_trigger_declared {
            return Err(RuntimeReconfigurationRollbackError::RollbackTriggerMissing);
        }
        if !rollback_apply_scope_declared {
            return Err(RuntimeReconfigurationRollbackError::RollbackApplyScopeMissing);
        }
        if !rollback_evidence_declared {
            return Err(RuntimeReconfigurationRollbackError::RollbackEvidenceMissing);
        }
        if !in_flight_operation_handling_declared {
            return Err(RuntimeReconfigurationRollbackError::InFlightOperationHandlingMissing);
        }
        if !rollback_failure_reason_declared_when_failed {
            return Err(RuntimeReconfigurationRollbackError::RollbackFailureReasonMissing);
        }

        Ok(Self {
            rollback_generation_state,
            rollback_generation_reference_present,
            rollback_trigger_declared,
            rollback_apply_scope_declared,
            rollback_evidence_declared,
            in_flight_operation_handling_declared,
            rollback_failure_reason_declared_when_failed,
        })
    }
}

/// runtime reconfiguration evidence guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeReconfigurationEvidenceGuard {
    reconfiguration_class_declared: bool,
    target_surface_declared: bool,
    current_generation_reference_declared: bool,
    proposed_generation_reference_declared: bool,
    validation_command_or_procedure_declared: bool,
    apply_scope_declared: bool,
    affected_active_scope_declared: bool,
    drain_or_restart_decision_declared: bool,
    rollback_status_declared: bool,
    audit_event_reference_declared: bool,
    rerun_condition_declared: bool,
    startup_configuration_evidence_not_used_as_runtime_reconfiguration_evidence: bool,
}

/// runtime reconfiguration evidence の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeReconfigurationEvidenceError {
    /// reconfiguration class がありません。
    ReconfigurationClassMissing,
    /// target surface がありません。
    TargetSurfaceMissing,
    /// current generation reference がありません。
    CurrentGenerationReferenceMissing,
    /// proposed generation reference がありません。
    ProposedGenerationReferenceMissing,
    /// validation command/procedure がありません。
    ValidationProcedureMissing,
    /// apply scope がありません。
    ApplyScopeMissing,
    /// affected active scope がありません。
    AffectedActiveScopeMissing,
    /// drain/restart decision がありません。
    DrainOrRestartDecisionMissing,
    /// rollback status がありません。
    RollbackStatusMissing,
    /// audit event reference がありません。
    AuditEventReferenceMissing,
    /// rerun condition がありません。
    RerunConditionMissing,
    /// startup configuration evidence を runtime reconfiguration evidence に流用しています。
    StartupEvidenceUsedAsRuntimeReconfigurationEvidence,
}

impl RuntimeReconfigurationEvidenceGuard {
    /// runtime reconfiguration evidence の採用条件を検査します。
    pub const fn try_new(
        reconfiguration_class_declared: bool,
        target_surface_declared: bool,
        current_generation_reference_declared: bool,
        proposed_generation_reference_declared: bool,
        validation_command_or_procedure_declared: bool,
        apply_scope_declared: bool,
        affected_active_scope_declared: bool,
        drain_or_restart_decision_declared: bool,
        rollback_status_declared: bool,
        audit_event_reference_declared: bool,
        rerun_condition_declared: bool,
        startup_configuration_evidence_not_used_as_runtime_reconfiguration_evidence: bool,
    ) -> Result<Self, RuntimeReconfigurationEvidenceError> {
        if !reconfiguration_class_declared {
            return Err(RuntimeReconfigurationEvidenceError::ReconfigurationClassMissing);
        }
        if !target_surface_declared {
            return Err(RuntimeReconfigurationEvidenceError::TargetSurfaceMissing);
        }
        if !current_generation_reference_declared {
            return Err(RuntimeReconfigurationEvidenceError::CurrentGenerationReferenceMissing);
        }
        if !proposed_generation_reference_declared {
            return Err(RuntimeReconfigurationEvidenceError::ProposedGenerationReferenceMissing);
        }
        if !validation_command_or_procedure_declared {
            return Err(RuntimeReconfigurationEvidenceError::ValidationProcedureMissing);
        }
        if !apply_scope_declared {
            return Err(RuntimeReconfigurationEvidenceError::ApplyScopeMissing);
        }
        if !affected_active_scope_declared {
            return Err(RuntimeReconfigurationEvidenceError::AffectedActiveScopeMissing);
        }
        if !drain_or_restart_decision_declared {
            return Err(RuntimeReconfigurationEvidenceError::DrainOrRestartDecisionMissing);
        }
        if !rollback_status_declared {
            return Err(RuntimeReconfigurationEvidenceError::RollbackStatusMissing);
        }
        if !audit_event_reference_declared {
            return Err(RuntimeReconfigurationEvidenceError::AuditEventReferenceMissing);
        }
        if !rerun_condition_declared {
            return Err(RuntimeReconfigurationEvidenceError::RerunConditionMissing);
        }
        if !startup_configuration_evidence_not_used_as_runtime_reconfiguration_evidence {
            return Err(
                RuntimeReconfigurationEvidenceError::StartupEvidenceUsedAsRuntimeReconfigurationEvidence,
            );
        }

        Ok(Self {
            reconfiguration_class_declared,
            target_surface_declared,
            current_generation_reference_declared,
            proposed_generation_reference_declared,
            validation_command_or_procedure_declared,
            apply_scope_declared,
            affected_active_scope_declared,
            drain_or_restart_decision_declared,
            rollback_status_declared,
            audit_event_reference_declared,
            rerun_condition_declared,
            startup_configuration_evidence_not_used_as_runtime_reconfiguration_evidence,
        })
    }
}

/// runtime reconfiguration failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeReconfigurationFailureKind {
    /// runtime reconfiguration class is not admitted.
    RuntimeReconfigurationNotAllowed,
    /// required configuration generation reference is missing.
    ConfigurationGenerationMissing,
    /// proposed generation conflicts with active generation/order.
    ConfigurationGenerationConflict,
    /// proposed generation fails validation.
    RuntimeReconfigurationValidationFailed,
    /// apply is not allowed for target surface or active scope.
    RuntimeReconfigurationApplyNotAllowed,
    /// drain/restart is required before apply.
    RuntimeReconfigurationDrainRequired,
    /// rollback is required before evidence adoption.
    RuntimeReconfigurationRollbackRequired,
    /// rollback execution failed.
    RuntimeReconfigurationRollbackFailed,
}

impl RuntimeReconfigurationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::RuntimeReconfigurationNotAllowed => "runtime_reconfiguration_not_allowed",
            Self::ConfigurationGenerationMissing => "configuration_generation_missing",
            Self::ConfigurationGenerationConflict => "configuration_generation_conflict",
            Self::RuntimeReconfigurationValidationFailed => {
                "runtime_reconfiguration_validation_failed"
            }
            Self::RuntimeReconfigurationApplyNotAllowed => {
                "runtime_reconfiguration_apply_not_allowed"
            }
            Self::RuntimeReconfigurationDrainRequired => "runtime_reconfiguration_drain_required",
            Self::RuntimeReconfigurationRollbackRequired => {
                "runtime_reconfiguration_rollback_required"
            }
            Self::RuntimeReconfigurationRollbackFailed => "runtime_reconfiguration_rollback_failed",
        }
    }
}

/// runtime reconfiguration failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeReconfigurationFailure {
    kind: RuntimeReconfigurationFailureKind,
    reason: CatalogedReasonRef,
}

impl RuntimeReconfigurationFailure {
    /// runtime reconfiguration failure を cataloged reason に接続します。
    pub fn from_kind(kind: RuntimeReconfigurationFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("runtime reconfiguration reason code must be registered");
        Self { kind, reason }
    }
}

/// runtime reconfiguration 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedRuntimeReconfigurationBehavior {
    /// startup config validation is treated as runtime hot-swap permission.
    StartupValidationAsRuntimeHotSwapPermission,
    /// driver reload changes core policy without accepted generation.
    DriverReloadChangesCorePolicyWithoutAcceptedGeneration,
    /// feature flag changes protocol/domain semantics at runtime without target Canonical.
    FeatureFlagChangesRuntimeSemanticsWithoutTargetCanonical,
    /// public endpoint/security/topology/authorization change lacks drain/restart rule.
    SensitiveSurfaceChangeWithoutDrainOrRestartRule,
    /// rollback erases failed apply event.
    RollbackErasesFailedApplyEvent,
    /// accepted decisions are silently reinterpreted under a new generation.
    AcceptedDecisionReinterpretedUnderNewGeneration,
    /// raw config/secret payload is written as generation evidence.
    RawPayloadWrittenAsGenerationEvidence,
}

/// feature flag class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureFlagClass {
    /// concrete driver implementation selection.
    DriverSelection,
    /// metrics/log/audit exporter implementation selection.
    ExporterSelection,
    /// configuration profile/bundle selection.
    ProfileSelection,
    /// explicitly documented experimental path gate.
    ExperimentalSurfaceGate,
    /// fake/deterministic support gate.
    TestOnlyGate,
}

impl FeatureFlagClass {
    /// core state machine を変更できる flag class は存在しません。
    pub const fn may_change_core_decision(self) -> bool {
        match self {
            Self::DriverSelection
            | Self::ExporterSelection
            | Self::ProfileSelection
            | Self::ExperimentalSurfaceGate
            | Self::TestOnlyGate => false,
        }
    }

    /// test_only_gate は test evidence だけに閉じます。
    pub const fn is_test_only(self) -> bool {
        matches!(self, Self::TestOnlyGate)
    }
}

/// capability を宣言する surface の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityDeclarationSurface {
    /// core protocol capability.
    CoreProtocol,
    /// entrypoint/config feature flag value.
    FeatureFlagValue,
    /// driver implementation selection.
    DriverImplementationSelection,
    /// SDK capability exposure.
    SdkCapabilityExposure,
    /// experimental lifecycle decision.
    ExperimentalLifecycleDecision,
    /// out-of-scope feature admission decision.
    OutOfScopeFeatureAdmission,
    /// runtime enablement evidence.
    RuntimeEnablementEvidence,
    /// runtime flag/profile change.
    RuntimeFlagProfileChange,
}

/// feature/capability surface の authority owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureCapabilityAuthorityOwner {
    /// core protocol capability authority.
    Core,
    /// entrypoints/config flag value authority.
    EntrypointsConfig,
    /// entrypoints composition selection authority.
    EntrypointsComposition,
    /// SDK public capability authority.
    Sdk,
    /// ADR/Canonical authority.
    AdrCanonical,
    /// reports authority.
    Reports,
    /// runtime reconfiguration Canonical authority.
    RuntimeReconfigurationCanonical,
}

impl CapabilityDeclarationSurface {
    /// surface と authority owner の対応を固定します。
    pub const fn authority_owner_matches(self, owner: FeatureCapabilityAuthorityOwner) -> bool {
        matches!(
            (self, owner),
            (Self::CoreProtocol, FeatureCapabilityAuthorityOwner::Core)
                | (
                    Self::FeatureFlagValue,
                    FeatureCapabilityAuthorityOwner::EntrypointsConfig
                )
                | (
                    Self::DriverImplementationSelection,
                    FeatureCapabilityAuthorityOwner::EntrypointsComposition
                )
                | (
                    Self::SdkCapabilityExposure,
                    FeatureCapabilityAuthorityOwner::Sdk
                )
                | (
                    Self::ExperimentalLifecycleDecision,
                    FeatureCapabilityAuthorityOwner::AdrCanonical
                )
                | (
                    Self::OutOfScopeFeatureAdmission,
                    FeatureCapabilityAuthorityOwner::AdrCanonical
                )
                | (
                    Self::RuntimeEnablementEvidence,
                    FeatureCapabilityAuthorityOwner::Reports
                )
                | (
                    Self::RuntimeFlagProfileChange,
                    FeatureCapabilityAuthorityOwner::RuntimeReconfigurationCanonical
                )
        )
    }
}

/// experimental surface lifecycle stage の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExperimentalLifecycleStage {
    /// Canonical/ADR draft exists.
    DraftDocumented,
    /// scaffold exists behind explicit gate.
    GatedScaffold,
    /// implementation exists behind explicit gate.
    GatedImplemented,
    /// selected integration evidence exists.
    ControlledIntegration,
    /// promoted into normal contract.
    AdoptedContract,
    /// support removed.
    Removed,
}

/// capability declaration guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilityDeclarationGuard {
    surface: CapabilityDeclarationSurface,
    configuration_wiring_owner: ConfigurationOwner,
    authority_owner: FeatureCapabilityAuthorityOwner,
    accepted_contract_version_declared: bool,
    optional_behavior_declared: bool,
    fallback_when_absent_declared: bool,
    required_absent_reason_declared: bool,
    sdk_parity_requirement_declared_when_client_visible: bool,
    evidence_class_required_before_adoption_declared: bool,
}

/// capability declaration guard 生成時の未検査入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilityDeclarationGuardInput {
    pub surface: CapabilityDeclarationSurface,
    pub configuration_wiring_owner: ConfigurationOwner,
    pub authority_owner: FeatureCapabilityAuthorityOwner,
    pub accepted_contract_version_declared: bool,
    pub optional_behavior_declared: bool,
    pub fallback_when_absent_declared: bool,
    pub required_absent_reason_declared: bool,
    pub sdk_parity_requirement_declared_when_client_visible: bool,
    pub evidence_class_required_before_adoption_declared: bool,
}

/// capability declaration の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityDeclarationError {
    /// entrypoints/configuration wiring owner ではありません。
    ConfigurationWiringOwnerMismatch,
    /// surface authority owner が一致していません。
    SurfaceAuthorityOwnerMismatch,
    /// accepted contract version がありません。
    AcceptedContractVersionMissing,
    /// optional behavior がありません。
    OptionalBehaviorMissing,
    /// capability absent 時の fallback がありません。
    FallbackWhenAbsentMissing,
    /// capability required but absent 時の reason がありません。
    RequiredAbsentReasonMissing,
    /// client-visible capability の SDK parity requirement がありません。
    SdkParityRequirementMissing,
    /// adoption 前に要求する evidence class がありません。
    EvidenceClassRequirementMissing,
}

impl CapabilityDeclarationGuard {
    /// capability declaration の必須 field を検査します。
    pub const fn try_new(
        input: CapabilityDeclarationGuardInput,
    ) -> Result<Self, CapabilityDeclarationError> {
        let CapabilityDeclarationGuardInput {
            surface,
            configuration_wiring_owner,
            authority_owner,
            accepted_contract_version_declared,
            optional_behavior_declared,
            fallback_when_absent_declared,
            required_absent_reason_declared,
            sdk_parity_requirement_declared_when_client_visible,
            evidence_class_required_before_adoption_declared,
        } = input;

        if !matches!(configuration_wiring_owner, ConfigurationOwner::Entrypoints) {
            return Err(CapabilityDeclarationError::ConfigurationWiringOwnerMismatch);
        }
        if !surface.authority_owner_matches(authority_owner) {
            return Err(CapabilityDeclarationError::SurfaceAuthorityOwnerMismatch);
        }
        if !accepted_contract_version_declared {
            return Err(CapabilityDeclarationError::AcceptedContractVersionMissing);
        }
        if !optional_behavior_declared {
            return Err(CapabilityDeclarationError::OptionalBehaviorMissing);
        }
        if !fallback_when_absent_declared {
            return Err(CapabilityDeclarationError::FallbackWhenAbsentMissing);
        }
        if !required_absent_reason_declared {
            return Err(CapabilityDeclarationError::RequiredAbsentReasonMissing);
        }
        if !sdk_parity_requirement_declared_when_client_visible {
            return Err(CapabilityDeclarationError::SdkParityRequirementMissing);
        }
        if !evidence_class_required_before_adoption_declared {
            return Err(CapabilityDeclarationError::EvidenceClassRequirementMissing);
        }

        Ok(Self {
            surface,
            configuration_wiring_owner,
            authority_owner,
            accepted_contract_version_declared,
            optional_behavior_declared,
            fallback_when_absent_declared,
            required_absent_reason_declared,
            sdk_parity_requirement_declared_when_client_visible,
            evidence_class_required_before_adoption_declared,
        })
    }
}

/// feature flag / capability admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureCapabilityAdmissionGuard {
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
}

/// feature flag / capability admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureCapabilityAdmissionError {
    /// flag class がありません。
    FlagClassMissing,
    /// allowed use がありません。
    AllowedUseMissing,
    /// prohibited use が block されていません。
    ProhibitedUseNotBlocked,
    /// protocol version の代替として flag を使っています。
    FlagUsedAsProtocolVersion,
    /// capability が required state transition semantics を変更しています。
    CapabilityChangesRequiredStateTransition,
    /// absent capability が silent fallback しています。
    AbsentCapabilityFallsBackSilently,
    /// experimental surface が explicit gate なしに使われています。
    ExperimentalSurfaceGateMissing,
    /// out-of-scope feature が admission Canonical なしに使われています。
    OutOfScopeAdmissionCanonicalMissing,
    /// SDK capability が server contract と一致していません。
    SdkCapabilityServerContractMismatch,
    /// test-only gate が test evidence の外で使われています。
    TestOnlyGateUsedOutsideTestEvidence,
    /// runtime flag change が reconfiguration generation/apply scope を持ちません。
    RuntimeFlagChangeLacksReconfigurationScope,
}

