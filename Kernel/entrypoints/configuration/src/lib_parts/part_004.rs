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
        out_of_scope_feature_has_admission_decision_when_used: bool,
        sdk_capability_matches_server_contract_when_visible: bool,
        test_only_gate_is_test_only: bool,
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
        if !out_of_scope_feature_has_admission_decision_when_used {
            return Err(FeatureCapabilityAdmissionError::OutOfScopeAdmissionDecisionMissing);
        }
        if !sdk_capability_matches_server_contract_when_visible {
            return Err(FeatureCapabilityAdmissionError::SdkCapabilityServerContractMismatch);
        }
        if flag_class.is_test_only() && !test_only_gate_is_test_only {
            return Err(FeatureCapabilityAdmissionError::TestOnlyGateUsedOutsideTestScope);
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
            out_of_scope_feature_has_admission_decision_when_used,
            sdk_capability_matches_server_contract_when_visible,
            test_only_gate_is_test_only,
            runtime_flag_change_uses_reconfiguration_generation_and_apply_scope,
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
    /// feature flag changes core state machine without source policy.
    FeatureFlagChangesCoreStateMachineWithoutSourcePolicy,
    /// capability alters required behavior inside accepted version.
    CapabilityAltersRequiredBehaviorInsideAcceptedVersion,
    /// experimental surface is enabled by default.
    ExperimentalSurfaceEnabledByDefault,
    /// out-of-scope feature is enabled by flag without an admission decision.
    OutOfScopeFeatureEnabledByFlagWithoutAdmission,
    /// SDK exposes capability that server contract does not define.
    SdkExposesUndefinedServerCapability,
    /// test-only gate is used outside test scope.
    TestOnlyGateUsedOutsideTestScope,
    /// removal bypasses compatibility/deprecation lifecycle.
    RemovalBypassesCompatibilityDeprecationLifecycle,
    /// runtime flag change is treated as startup profile validation.
    RuntimeFlagChangeTreatedAsStartupProfileValidation,
}
