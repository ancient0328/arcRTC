/// rotation policy guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretRotationPolicyError {
    /// generation reference format がありません。
    GenerationReferenceFormatMissing,
    /// overlap window が 0 または未宣言です。
    OverlapWindowMissing,
    /// revocation behavior が未宣言です。
    RevocationBehaviorMissing,
    /// active credential/session relation が未宣言です。
    ActiveCredentialRelationMissing,
    /// stale/revoked/unavailable failure reason が未宣言です。
    FailureReasonMissing,
    /// audit relation が未宣言です。
    AuditRelationMissing,
    /// redaction rule が未宣言です。
    RedactionRuleMissing,
}

impl SecretRotationPolicyGuard {
    /// rotation policy の required fields を検査します。
    pub const fn try_new(
        secret_class: SecretRotationClass,
        generation_reference_format_declared: bool,
        maximum_overlap_window_millis: u64,
        revocation_behavior_declared: bool,
        active_credential_session_relation_declared: bool,
        failure_reason_declared: bool,
        audit_relation_declared: bool,
        redaction_rule_declared: bool,
    ) -> Result<Self, SecretRotationPolicyError> {
        if !generation_reference_format_declared {
            return Err(SecretRotationPolicyError::GenerationReferenceFormatMissing);
        }
        if maximum_overlap_window_millis == 0 {
            return Err(SecretRotationPolicyError::OverlapWindowMissing);
        }
        if !revocation_behavior_declared {
            return Err(SecretRotationPolicyError::RevocationBehaviorMissing);
        }
        if !active_credential_session_relation_declared {
            return Err(SecretRotationPolicyError::ActiveCredentialRelationMissing);
        }
        if !failure_reason_declared {
            return Err(SecretRotationPolicyError::FailureReasonMissing);
        }
        if !audit_relation_declared {
            return Err(SecretRotationPolicyError::AuditRelationMissing);
        }
        if !redaction_rule_declared {
            return Err(SecretRotationPolicyError::RedactionRuleMissing);
        }

        Ok(Self {
            secret_class,
            generation_reference_format_declared,
            maximum_overlap_window_millis,
            revocation_behavior_declared,
            active_credential_session_relation_declared,
            failure_reason_declared,
            audit_relation_declared,
            redaction_rule_declared,
        })
    }
}

/// generation state を decision に使う前の admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecretGenerationStateAdmission {
    state: SecretGenerationState,
    rotation_state_determined: bool,
    accepted_for_new_use: bool,
    accepted_for_bounded_overlap: bool,
    overlap_window_bounded: bool,
    revoked_or_expired_generation_rejected: bool,
}

/// generation state admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretGenerationStateAdmissionError {
    /// rotation state を判断できません。
    RotationStateUnavailable,
    /// current generation が new use に受理されません。
    CurrentGenerationNotAccepted,
    /// previous generation overlap の制約が不正です。
    PreviousOverlapInvalid,
    /// pending generation を decision に使っています。
    PendingGenerationAccepted,
    /// revoked/expired generation が受理されています。
    RevokedOrExpiredGenerationAccepted,
}

impl SecretGenerationStateAdmission {
    /// current/previous/pending/revoked/expired の受理条件を固定します。
    pub const fn try_new(
        state: SecretGenerationState,
        rotation_state_determined: bool,
        accepted_for_new_use: bool,
        accepted_for_bounded_overlap: bool,
        overlap_window_bounded: bool,
        revoked_or_expired_generation_rejected: bool,
    ) -> Result<Self, SecretGenerationStateAdmissionError> {
        if !rotation_state_determined {
            return Err(SecretGenerationStateAdmissionError::RotationStateUnavailable);
        }
        match state {
            SecretGenerationState::CurrentGeneration => {
                if !accepted_for_new_use {
                    return Err(SecretGenerationStateAdmissionError::CurrentGenerationNotAccepted);
                }
            }
            SecretGenerationState::PreviousGenerationOverlap => {
                if accepted_for_new_use || !accepted_for_bounded_overlap || !overlap_window_bounded
                {
                    return Err(SecretGenerationStateAdmissionError::PreviousOverlapInvalid);
                }
            }
            SecretGenerationState::PendingGeneration => {
                if accepted_for_new_use || accepted_for_bounded_overlap {
                    return Err(SecretGenerationStateAdmissionError::PendingGenerationAccepted);
                }
            }
            SecretGenerationState::RevokedGeneration | SecretGenerationState::ExpiredGeneration => {
                if accepted_for_new_use
                    || accepted_for_bounded_overlap
                    || !revoked_or_expired_generation_rejected
                {
                    return Err(
                        SecretGenerationStateAdmissionError::RevokedOrExpiredGenerationAccepted,
                    );
                }
            }
        }

        Ok(Self {
            state,
            rotation_state_determined,
            accepted_for_new_use,
            accepted_for_bounded_overlap,
            overlap_window_bounded,
            revoked_or_expired_generation_rejected,
        })
    }
}

/// secret rotation lifecycle failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretRotationFailureKind {
    /// required rotation has not occurred.
    SecretRotationRequired,
    /// generation is not accepted by policy.
    SecretGenerationNotAccepted,
    /// generation has been revoked.
    SecretKeyRevoked,
    /// overlap window expired.
    SecretOverlapWindowExpired,
    /// rotation state cannot be determined.
    SecretRotationStateUnavailable,
    /// secret source unavailable.
    SecretUnavailable,
    /// runtime security configuration missing.
    RuntimeConfigMissing,
    /// runtime security configuration invalid.
    RuntimeConfigInvalid,
    /// token key unavailable.
    TokenKeyUnavailable,
}

impl SecretRotationFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::SecretRotationRequired => "secret_rotation_required",
            Self::SecretGenerationNotAccepted => "secret_generation_not_accepted",
            Self::SecretKeyRevoked => "secret_key_revoked",
            Self::SecretOverlapWindowExpired => "secret_overlap_window_expired",
            Self::SecretRotationStateUnavailable => "secret_rotation_state_unavailable",
            Self::SecretUnavailable => "secret_unavailable",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::TokenKeyUnavailable => "token_key_unavailable",
        }
    }
}

/// secret rotation failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecretRotationFailure {
    kind: SecretRotationFailureKind,
    reason: CatalogedReasonRef,
}

impl SecretRotationFailure {
    /// rotation failure を cataloged reason に接続します。
    pub fn from_kind(kind: SecretRotationFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("secret rotation reason code must be registered");
        Self { kind, reason }
    }
}

/// rotation execution が policy を silently 変更しないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecretRotationExecutionBoundaryGuard {
    raw_secret_load_driver_owned: bool,
    rotation_execution_driver_or_entrypoints_owned: bool,
    core_policy_not_changed_silently_by_driver_state: bool,
    insecure_fallback_absent: bool,
    token_issuance_absent: bool,
    raw_material_absent_from_audit_log_metric_report: bool,
    raw_material_absent_from_core_state: bool,
    raw_material_absent_from_sdk_public_error: bool,
}

/// rotation execution boundary guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretRotationExecutionBoundaryError {
    /// raw secret load が driver に閉じていません。
    RawSecretLoadEscapesDriver,
    /// rotation execution owner が不明です。
    RotationExecutionOwnerMissing,
    /// driver-local state が core policy を silently 変更しています。
    DriverStateSilentlyChangesCorePolicy,
    /// rotation failure が insecure mode に fallback しています。
    InsecureFallback,
    /// token issuance を arcRTC responsibility として扱っています。
    TokenIssuanceMixed,
    /// raw secret material が audit/observation surface に出ます。
    RawSecretMaterialExposed,
    /// raw secret material が core state または SDK public error に出ます。
    RawSecretMaterialCrossesCoreOrSdk,
}

impl SecretRotationExecutionBoundaryGuard {
    /// rotation 実行と policy ownership の境界を確認します。
    pub const fn try_new(
        raw_secret_load_driver_owned: bool,
        rotation_execution_driver_or_entrypoints_owned: bool,
        core_policy_not_changed_silently_by_driver_state: bool,
        insecure_fallback_absent: bool,
        token_issuance_absent: bool,
        raw_material_absent_from_audit_log_metric_report: bool,
        raw_material_absent_from_core_state: bool,
        raw_material_absent_from_sdk_public_error: bool,
    ) -> Result<Self, SecretRotationExecutionBoundaryError> {
        if !raw_secret_load_driver_owned {
            return Err(SecretRotationExecutionBoundaryError::RawSecretLoadEscapesDriver);
        }
        if !rotation_execution_driver_or_entrypoints_owned {
            return Err(SecretRotationExecutionBoundaryError::RotationExecutionOwnerMissing);
        }
        if !core_policy_not_changed_silently_by_driver_state {
            return Err(
                SecretRotationExecutionBoundaryError::DriverStateSilentlyChangesCorePolicy,
            );
        }
        if !insecure_fallback_absent {
            return Err(SecretRotationExecutionBoundaryError::InsecureFallback);
        }
        if !token_issuance_absent {
            return Err(SecretRotationExecutionBoundaryError::TokenIssuanceMixed);
        }
        if !raw_material_absent_from_audit_log_metric_report {
            return Err(SecretRotationExecutionBoundaryError::RawSecretMaterialExposed);
        }
        if !raw_material_absent_from_core_state || !raw_material_absent_from_sdk_public_error {
            return Err(SecretRotationExecutionBoundaryError::RawSecretMaterialCrossesCoreOrSdk);
        }

        Ok(Self {
            raw_secret_load_driver_owned,
            rotation_execution_driver_or_entrypoints_owned,
            core_policy_not_changed_silently_by_driver_state,
            insecure_fallback_absent,
            token_issuance_absent,
            raw_material_absent_from_audit_log_metric_report,
            raw_material_absent_from_core_state,
            raw_material_absent_from_sdk_public_error,
        })
    }
}

/// secret rotation lifecycle 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedSecretRotationLifecycleBehavior {
    /// stale generation is accepted without overlap policy.
    StaleGenerationAcceptedWithoutOverlapPolicy,
    /// revoked key is accepted due cache convenience.
    RevokedGenerationAcceptedFromCache,
    /// rotation failure falls back to insecure mode.
    RotationFailureInsecureFallback,
    /// raw secret material is written to audit/log/metric/report.
    RawSecretMaterialWrittenToAuditOrObservation,
    /// raw secret material crosses into core state or SDK public error.
    RawSecretMaterialCrossesCoreOrSdk,
    /// driver-local rotation state changes core security policy silently.
    DriverStateChangesCorePolicySilently,
    /// token issuance is treated as arcRTC responsibility.
    TokenIssuanceOwnedByArcRtc,
    /// current/previous/pending semantics differ from shared source policy.
    DriverSpecificGenerationSemantics,
}
