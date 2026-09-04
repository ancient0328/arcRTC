impl RuntimeReconfigurationRollbackGuard {
    /// rollback の runtime outcome に必要な境界を検査します。
    pub const fn try_new(
        rollback_generation_state: RuntimeConfigurationGenerationState,
        rollback_generation_reference_present: bool,
        rollback_trigger_declared: bool,
        rollback_apply_scope_declared: bool,
        rollback_outcome_observed: bool,
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
        if !rollback_outcome_observed {
            return Err(RuntimeReconfigurationRollbackError::RollbackOutcomeMissing);
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
            rollback_outcome_observed,
            in_flight_operation_handling_declared,
            rollback_failure_reason_declared_when_failed,
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
    /// rollback is required before a new generation can be enabled.
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
    /// feature flag changes protocol/domain semantics without target source policy.
    FeatureFlagChangesRuntimeSemanticsWithoutTargetPolicy,
    /// public endpoint/security/topology/authorization change lacks drain/restart rule.
    SensitiveSurfaceChangeWithoutDrainOrRestartRule,
    /// rollback erases failed apply event.
    RollbackErasesFailedApplyEvent,
    /// accepted decisions are silently reinterpreted under a new generation.
    AcceptedDecisionReinterpretedUnderNewGeneration,
    /// raw config/secret payload is written as generation state.
    RawPayloadWrittenAsGenerationState,
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

    /// test_only_gate は test scope だけに閉じます。
    pub const fn is_test_only(self) -> bool {
        matches!(self, Self::TestOnlyGate)
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
    out_of_scope_feature_has_admission_decision_when_used: bool,
    sdk_capability_matches_server_contract_when_visible: bool,
    test_only_gate_is_test_only: bool,
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
    /// out-of-scope feature が admission decision なしに使われています。
    OutOfScopeAdmissionDecisionMissing,
    /// SDK capability が server contract と一致していません。
    SdkCapabilityServerContractMismatch,
    /// test-only gate が test scope の外で使われています。
    TestOnlyGateUsedOutsideTestScope,
    /// runtime flag change が reconfiguration generation/apply scope を持ちません。
    RuntimeFlagChangeLacksReconfigurationScope,
}
