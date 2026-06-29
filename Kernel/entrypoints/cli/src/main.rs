//! CLI entrypoint は operator / developer 向け command entrypoint です。
//!
//! domain decision や port contract は定義せず、後続 task で許可された
//! core use case と driver operation への入口だけを配置します。

use arcrtc_core_command::CoreCommandSurface;
use arcrtc_core_reason::CatalogedReasonRef;

fn main() {
    // CLI は command entrypoint であり、domain decision の所有者にはしません。
    let _surface = CliCompositionSurface;
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
    /// health/admin Canonical 配下の privileged command です。
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
    /// out-of-scope feature lacks ADR/Canonical admission.
    FeatureAdmissionNotDocumented,
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
            Self::FeatureAdmissionNotDocumented => "feature_admission_not_documented",
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
    demo_defaults_not_used_as_production_policy: bool,
    out_of_scope_feature_rejected_or_close_not_claimed: bool,
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
    /// demo/default 設定を production policy として扱っています。
    DemoDefaultAsProductionPolicy,
    /// out-of-scope feature を ADR/Canonical なしに受理しています。
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
        demo_defaults_not_used_as_production_policy: bool,
        out_of_scope_feature_rejected_or_close_not_claimed: bool,
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
        if !demo_defaults_not_used_as_production_policy {
            return Err(CliCompositionError::DemoDefaultAsProductionPolicy);
        }
        if !out_of_scope_feature_rejected_or_close_not_claimed {
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
            demo_defaults_not_used_as_production_policy,
            out_of_scope_feature_rejected_or_close_not_claimed,
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
    /// demo defaults become production policy.
    DemoDefaultsBecomeProductionPolicy,
    /// privileged command executes without operator/admin authorization.
    PrivilegedCommandWithoutOperatorAuthorization,
    /// out-of-scope feature is admitted into production scope.
    OutOfScopeFeatureAdmitted,
}
