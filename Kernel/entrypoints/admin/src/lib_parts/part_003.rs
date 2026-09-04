/// operator/admin authorization の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorAdminAuthorizationError {
    /// operator/admin class がありません。
    OperatorAdminClassMissing,
    /// credential/context source がありません。
    CredentialContextSourceMissing,
    /// allowed action class がありません。
    AllowedActionClassMissing,
    /// operator/admin class と action class が一致していません。
    ActionClassMismatch,
    /// allowed target scope がありません。
    TargetScopeMissing,
    /// operator/admin class と target scope が一致していません。
    TargetScopeMismatch,
    /// lifetime/expiry がありません。
    LifetimeExpiryMissing,
    /// redaction rule がありません。
    RedactionRuleMissing,
    /// target command boundary がありません。
    TargetCommandBoundaryMissing,
    /// audit event relation がありません。
    AuditEventRelationMissing,
    /// failure reason mapping がありません。
    FailureReasonMappingMissing,
    /// raw credential が core identity として扱われています。
    RawCredentialBecameCoreIdentity,
    /// credential verification result が typed result に閉じていません。
    UntypedCredentialVerificationResult,
    /// communication participant authorization を operator/admin authorization に流用しています。
    CommunicationAuthorizationReused,
    /// operator/admin authorization を participant/domain action に流用しています。
    OperatorAuthorizationReusedForDomainAction,
    /// target action が allowed core/driver boundary を通っていません。
    TargetActionBoundaryMissing,
    /// action execution request の target action event がありません。
    TargetActionEventMissing,
    /// developer-local context を operator authorization に使っています。
    DeveloperLocalContextUsedAsOperatorAuthorization,
}

impl OperatorAdminAuthorizationGuard {
    /// operator/admin authorization context と action/scope の必須境界を検査します。
    pub const fn try_new(
        operator_admin_class: OperatorAdminClass,
        credential_source_class: OperatorCredentialContextSourceClass,
        allowed_action_class: OperatorAdminAllowedActionClass,
        target_scope_class: OperatorAdminTargetScopeClass,
        operator_admin_class_declared: bool,
        credential_context_source_declared: bool,
        allowed_action_class_declared: bool,
        allowed_target_scope_declared: bool,
        lifetime_expiry_declared: bool,
        redaction_rule_declared: bool,
        target_command_boundary_declared: bool,
        audit_event_relation_declared: bool,
        failure_reason_mapping_declared: bool,
        raw_credential_not_core_identity: bool,
        credential_verification_result_typed_only: bool,
        communication_participant_authorization_not_reused: bool,
        operator_admin_authorization_not_used_for_participant_domain_action: bool,
        target_action_keeps_core_or_driver_boundary: bool,
        target_action_event_required_when_execution_requested: bool,
        developer_local_context_not_used_as_operator_authorization: bool,
    ) -> Result<Self, OperatorAdminAuthorizationError> {
        if !operator_admin_class_declared {
            return Err(OperatorAdminAuthorizationError::OperatorAdminClassMissing);
        }
        if !credential_context_source_declared {
            return Err(OperatorAdminAuthorizationError::CredentialContextSourceMissing);
        }
        if !allowed_action_class_declared {
            return Err(OperatorAdminAuthorizationError::AllowedActionClassMissing);
        }
        if !operator_admin_class.admits_action_class(allowed_action_class) {
            return Err(OperatorAdminAuthorizationError::ActionClassMismatch);
        }
        if !allowed_target_scope_declared {
            return Err(OperatorAdminAuthorizationError::TargetScopeMissing);
        }
        if !operator_admin_class.admits_action_scope(allowed_action_class, target_scope_class) {
            return Err(OperatorAdminAuthorizationError::TargetScopeMismatch);
        }
        if !lifetime_expiry_declared {
            return Err(OperatorAdminAuthorizationError::LifetimeExpiryMissing);
        }
        if !redaction_rule_declared {
            return Err(OperatorAdminAuthorizationError::RedactionRuleMissing);
        }
        if !target_command_boundary_declared {
            return Err(OperatorAdminAuthorizationError::TargetCommandBoundaryMissing);
        }
        if !audit_event_relation_declared {
            return Err(OperatorAdminAuthorizationError::AuditEventRelationMissing);
        }
        if !failure_reason_mapping_declared {
            return Err(OperatorAdminAuthorizationError::FailureReasonMappingMissing);
        }
        if !raw_credential_not_core_identity {
            return Err(OperatorAdminAuthorizationError::RawCredentialBecameCoreIdentity);
        }
        if !credential_verification_result_typed_only {
            return Err(OperatorAdminAuthorizationError::UntypedCredentialVerificationResult);
        }
        if !communication_participant_authorization_not_reused {
            return Err(OperatorAdminAuthorizationError::CommunicationAuthorizationReused);
        }
        if !operator_admin_authorization_not_used_for_participant_domain_action {
            return Err(
                OperatorAdminAuthorizationError::OperatorAuthorizationReusedForDomainAction,
            );
        }
        if !target_action_keeps_core_or_driver_boundary {
            return Err(OperatorAdminAuthorizationError::TargetActionBoundaryMissing);
        }
        if !target_action_event_required_when_execution_requested {
            return Err(OperatorAdminAuthorizationError::TargetActionEventMissing);
        }
        if operator_admin_class.is_developer_local()
            && !developer_local_context_not_used_as_operator_authorization
        {
            return Err(
                OperatorAdminAuthorizationError::DeveloperLocalContextUsedAsOperatorAuthorization,
            );
        }

        Ok(Self {
            operator_admin_class,
            credential_source_class,
            allowed_action_class,
            target_scope_class,
            operator_admin_class_declared,
            credential_context_source_declared,
            allowed_action_class_declared,
            allowed_target_scope_declared,
            lifetime_expiry_declared,
            redaction_rule_declared,
            target_command_boundary_declared,
            audit_event_relation_declared,
            failure_reason_mapping_declared,
            raw_credential_not_core_identity,
            credential_verification_result_typed_only,
            communication_participant_authorization_not_reused,
            operator_admin_authorization_not_used_for_participant_domain_action,
            target_action_keeps_core_or_driver_boundary,
            target_action_event_required_when_execution_requested,
            developer_local_context_not_used_as_operator_authorization,
        })
    }
}

/// operator/admin authorization audit event type です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorAdminAuthorizationAuditEventType {
    /// operator/admin authorization decision.
    OperatorAdminAuthorizationDecision,
}

impl OperatorAdminAuthorizationAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::OperatorAdminAuthorizationDecision => "operator_admin_authorization_decision",
        }
    }
}

/// operator/admin authorization audit guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperatorAdminAuthorizationAuditGuard {
    event_type: OperatorAdminAuthorizationAuditEventType,
    outcome: OperatorAdminAuthorizationOutcome,
    correlation_id_declared: bool,
    operator_admin_class_declared: bool,
    action_class_declared: bool,
    target_scope_declared: bool,
    startup_run_id_declared_when_startup_or_entrypoint_scoped: bool,
    cataloged_reason_declared_for_non_success: bool,
}

/// operator/admin authorization audit の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorAdminAuthorizationAuditError {
    /// CorrelationId がありません。
    CorrelationIdMissing,
    /// operator/admin class がありません。
    OperatorAdminClassMissing,
    /// action class がありません。
    ActionClassMissing,
    /// target scope がありません。
    TargetScopeMissing,
    /// startup/entrypoint-scoped decision の StartupRunId がありません。
    StartupRunIdMissing,
    /// non-success outcome の cataloged reason がありません。
    CatalogedReasonMissing,
}

impl OperatorAdminAuthorizationAuditGuard {
    /// operator_admin_authorization_decision audit shape を検査します。
    pub const fn try_new(
        event_type: OperatorAdminAuthorizationAuditEventType,
        outcome: OperatorAdminAuthorizationOutcome,
        correlation_id_declared: bool,
        operator_admin_class_declared: bool,
        action_class_declared: bool,
        target_scope_declared: bool,
        startup_run_id_declared_when_startup_or_entrypoint_scoped: bool,
        cataloged_reason_declared_for_non_success: bool,
    ) -> Result<Self, OperatorAdminAuthorizationAuditError> {
        if !correlation_id_declared {
            return Err(OperatorAdminAuthorizationAuditError::CorrelationIdMissing);
        }
        if !operator_admin_class_declared {
            return Err(OperatorAdminAuthorizationAuditError::OperatorAdminClassMissing);
        }
        if !action_class_declared {
            return Err(OperatorAdminAuthorizationAuditError::ActionClassMissing);
        }
        if !target_scope_declared {
            return Err(OperatorAdminAuthorizationAuditError::TargetScopeMissing);
        }
        if !startup_run_id_declared_when_startup_or_entrypoint_scoped {
            return Err(OperatorAdminAuthorizationAuditError::StartupRunIdMissing);
        }
        if outcome.requires_reason() && !cataloged_reason_declared_for_non_success {
            return Err(OperatorAdminAuthorizationAuditError::CatalogedReasonMissing);
        }

        Ok(Self {
            event_type,
            outcome,
            correlation_id_declared,
            operator_admin_class_declared,
            action_class_declared,
            target_scope_declared,
            startup_run_id_declared_when_startup_or_entrypoint_scoped,
            cataloged_reason_declared_for_non_success,
        })
    }
}

/// operator/admin authorization failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorAdminAuthorizationFailureKind {
    /// operator credential missing.
    OperatorCredentialMissing,
    /// operator credential invalid.
    OperatorCredentialInvalid,
    /// operator authorization context missing.
    OperatorAuthorizationContextMissing,
    /// operator authorization context expired.
    OperatorAuthorizationContextExpired,
    /// operator action denied by policy.
    OperatorActionDenied,
    /// operator target scope not allowed.
    OperatorScopeNotAllowed,
    /// admin action not allowed by boundary/policy.
    AdminActionNotAllowed,
    /// maintenance mode blocks action.
    MaintenanceModeActive,
    /// runtime configuration missing.
    RuntimeConfigMissing,
    /// runtime configuration invalid.
    RuntimeConfigInvalid,
}

impl OperatorAdminAuthorizationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::OperatorCredentialMissing => "operator_credential_missing",
            Self::OperatorCredentialInvalid => "operator_credential_invalid",
            Self::OperatorAuthorizationContextMissing => "operator_authorization_context_missing",
            Self::OperatorAuthorizationContextExpired => "operator_authorization_context_expired",
            Self::OperatorActionDenied => "operator_action_denied",
            Self::OperatorScopeNotAllowed => "operator_scope_not_allowed",
            Self::AdminActionNotAllowed => "admin_action_not_allowed",
            Self::MaintenanceModeActive => "maintenance_mode_active",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
        }
    }
}

/// operator/admin authorization failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperatorAdminAuthorizationFailure {
    kind: OperatorAdminAuthorizationFailureKind,
    reason: CatalogedReasonRef,
}

impl OperatorAdminAuthorizationFailure {
    /// operator/admin authorization failure を cataloged reason に接続します。
    pub fn from_kind(kind: OperatorAdminAuthorizationFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("operator/admin authorization reason code must be registered");
        Self { kind, reason }
    }
}

/// operator/admin authorization 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedOperatorAdminAuthorizationBehavior {
    /// admin command mutates core aggregate state outside core use case/state machine.
    AdminCommandMutatesCoreStateOutsideCoreUseCase,
    /// communication participant token authorizes operator/admin action by default.
    CommunicationParticipantTokenAuthorizesOperatorAction,
    /// operator/admin authorization is inferred from localhost, network reachability, or deployment environment.
    OperatorAuthorizationInferredFromEnvironment,
    /// raw operator credential appears in audit/log/output.
    RawOperatorCredentialAppearsInAuditLogOutput,
    /// admin UI or CLI response becomes domain decision authority.
    AdminUiOrCliResponseBecomesDomainDecisionAuthority,
    /// developer-local context is used as operator authorization.
    DeveloperLocalContextUsedAsOperatorAuthorization,
    /// operator authorization and communication authorization are treated as the same decision.
    OperatorAndCommunicationAuthorizationMerged,
    /// operator/admin denial is recorded only as free-text.
    OperatorAdminDenialRecordedAsFreeTextOnly,
    /// authorization decision replaces target action boundary.
    AuthorizationDecisionReplacesTargetActionBoundary,
}
