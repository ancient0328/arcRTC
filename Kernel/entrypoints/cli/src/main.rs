//! CLI entrypoint は operator / developer 向け command entrypoint です。
//!
//! domain decision や port contract は定義せず、後続 task で許可された
//! core use case と driver operation への入口だけを配置します。

use arcrtc_core_command::CoreCommandSurface;
use arcrtc_core_reason::CatalogedReasonRef;

fn main() {
    let Some(token) = std::env::args().nth(1) else {
        eprintln!(
            "{}",
            CliCompositionFailureKind::RuntimeConfigMissing.reason_code()
        );
        std::process::exit(2);
    };

    // token は command class 選択子であり、reason 語彙として扱いません。
    let class = match token.as_str() {
        "signaling" => CliCommandClass::Signaling,
        "sfu" => CliCommandClass::Sfu,
        "turn" => CliCommandClass::Turn,
        "admin-maintenance" => CliCommandClass::AdminMaintenance,
        "developer-inspection" => CliCommandClass::DeveloperInspection,
        _ => {
            eprintln!(
                "{}",
                CliCompositionFailureKind::ExternalDecodeFailed.reason_code()
            );
            std::process::exit(2);
        }
    };

    if CliCompositionGuard::try_new(class, true, true, true, true, true, true, true, false).is_err()
    {
        eprintln!(
            "{}",
            CliCompositionFailureKind::AuthorizationPolicyDenied.reason_code()
        );
        std::process::exit(3);
    }

    let _wiring = CliCommandWiring::new(CoreCommandSurface, class);
    println!("command_class={token}");
}

/// CLI entrypoint の composition root marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CliCompositionSurface;

/// CLI が扱う command class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CliCommandClass {
    /// Signaling core command/use case への入口です。
    Signaling,
    /// SFU core command/use case への入口です。
    Sfu,
    /// TURN core command/use case への入口です。
    Turn,
    /// health/admin authorization policy 配下の privileged command です。
    AdminMaintenance,
    /// developer-only inspection command です。domain authority にはしません。
    DeveloperInspection,
}

impl CliCommandClass {
    /// privileged operator/admin authorization が必要な command です。
    pub const fn requires_operator_authorization(self) -> bool {
        matches!(self, Self::AdminMaintenance)
    }
}

/// CLI command を core command boundary へ写す wiring set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CliCommandWiring {
    core_command: CoreCommandSurface,
    command_class: CliCommandClass,
}

impl CliCommandWiring {
    /// CLI command class と core command surface を束ねます。
    pub const fn new(core_command: CoreCommandSurface, command_class: CliCommandClass) -> Self {
        Self {
            core_command,
            command_class,
        }
    }
}

/// CLI composition failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CliCompositionFailureKind {
    /// CLI input cannot map to typed command.
    ExternalDecodeFailed,
    /// required runtime configuration missing.
    RuntimeConfigMissing,
    /// runtime configuration invalid.
    RuntimeConfigInvalid,
    /// privileged operator/admin authorization denied.
    AuthorizationPolicyDenied,
    /// requested feature is outside v0.2 scope.
    FeatureOutOfScope,
    /// out-of-scope feature admission is missing or rejected.
    FeatureAdmissionMissingOrRejected,
}

impl CliCompositionFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::AuthorizationPolicyDenied => "authorization_policy_denied",
            Self::FeatureOutOfScope => "feature_out_of_scope",
            Self::FeatureAdmissionMissingOrRejected => "feature_not_supported",
        }
    }
}

/// CLI composition failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CliCompositionFailure {
    kind: CliCompositionFailureKind,
    reason: CatalogedReasonRef,
}

impl CliCompositionFailure {
    /// CLI failure を cataloged reason に接続します。
    pub fn from_kind(kind: CliCompositionFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("cli composition reason code must be registered");
        Self { kind, reason }
    }
}

/// CLI が domain authority にならないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CliCompositionGuard {
    command_class: CliCommandClass,
    cli_input_converted_to_typed_command: bool,
    command_maps_to_core_owned_boundary: bool,
    cli_does_not_define_domain_decision: bool,
    cli_does_not_define_reason_vocabulary: bool,
    cli_does_not_define_port_trait: bool,
    demo_defaults_not_used_as_managed_runtime_policy: bool,
    out_of_scope_feature_admission_missing_or_rejected: bool,
    operator_authorization_satisfied_when_required: bool,
}

/// CLI composition guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CliCompositionError {
    /// CLI input が typed command に変換されていません。
    TypedCommandMissing,
    /// core-owned command/use case boundary へ写っていません。
    CoreCommandBoundaryMissing,
    /// CLI が domain decision を所有しています。
    CliOwnsDomainDecision,
    /// CLI が reason vocabulary を定義しています。
    CliDefinesReasonVocabulary,
    /// CLI が port trait を定義しています。
    CliDefinesPortTrait,
    /// demo/default 設定を managed-runtime policy として扱っています。
    DemoDefaultAsManagedRuntimePolicy,
    /// out-of-scope feature の admission が missing/rejected です。
    OutOfScopeFeatureAdmitted,
    /// privileged command に operator/admin authorization がありません。
    OperatorAuthorizationMissing,
}

impl CliCompositionGuard {
    /// CLI command entrypoint の責務境界を検査します。
    pub const fn try_new(
        command_class: CliCommandClass,
        cli_input_converted_to_typed_command: bool,
        command_maps_to_core_owned_boundary: bool,
        cli_does_not_define_domain_decision: bool,
        cli_does_not_define_reason_vocabulary: bool,
        cli_does_not_define_port_trait: bool,
        demo_defaults_not_used_as_managed_runtime_policy: bool,
        out_of_scope_feature_admission_missing_or_rejected: bool,
        operator_authorization_satisfied_when_required: bool,
    ) -> Result<Self, CliCompositionError> {
        if !cli_input_converted_to_typed_command {
            return Err(CliCompositionError::TypedCommandMissing);
        }
        if !command_maps_to_core_owned_boundary {
            return Err(CliCompositionError::CoreCommandBoundaryMissing);
        }
        if !cli_does_not_define_domain_decision {
            return Err(CliCompositionError::CliOwnsDomainDecision);
        }
        if !cli_does_not_define_reason_vocabulary {
            return Err(CliCompositionError::CliDefinesReasonVocabulary);
        }
        if !cli_does_not_define_port_trait {
            return Err(CliCompositionError::CliDefinesPortTrait);
        }
        if !demo_defaults_not_used_as_managed_runtime_policy {
            return Err(CliCompositionError::DemoDefaultAsManagedRuntimePolicy);
        }
        if !out_of_scope_feature_admission_missing_or_rejected {
            return Err(CliCompositionError::OutOfScopeFeatureAdmitted);
        }
        if command_class.requires_operator_authorization()
            && !operator_authorization_satisfied_when_required
        {
            return Err(CliCompositionError::OperatorAuthorizationMissing);
        }

        Ok(Self {
            command_class,
            cli_input_converted_to_typed_command,
            command_maps_to_core_owned_boundary,
            cli_does_not_define_domain_decision,
            cli_does_not_define_reason_vocabulary,
            cli_does_not_define_port_trait,
            demo_defaults_not_used_as_managed_runtime_policy,
            out_of_scope_feature_admission_missing_or_rejected,
            operator_authorization_satisfied_when_required,
        })
    }
}

/// CLI composition で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedCliCompositionBehavior {
    /// CLI path bypasses core use case.
    CliBypassesCoreUseCase,
    /// CLI command owns domain decision.
    CliCommandOwnsDomainDecision,
    /// CLI defines reason vocabulary.
    CliDefinesReasonVocabulary,
    /// CLI defines core port trait.
    CliDefinesCorePortTrait,
    /// demo defaults become managed-runtime policy.
    DemoDefaultsBecomeManagedRuntimePolicy,
    /// privileged command executes without operator/admin authorization.
    PrivilegedCommandWithoutOperatorAuthorization,
    /// out-of-scope feature is admitted into managed-runtime scope.
    OutOfScopeFeatureAdmitted,
}
