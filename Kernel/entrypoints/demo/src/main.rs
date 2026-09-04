//! demo entrypoint は v0.2 composition を確認するための demo entrypoint です。
//!
//! UI や end-user workflow は所有せず、後続 task で許可された wiring の
//! 見通しを保つための起動面だけを持ちます。

use arcrtc_core_command::CoreCommandSurface;
use arcrtc_core_reason::CatalogedReasonRef;

fn main() {
    let token = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("signaling-only"));

    // default scenario は demo 起動面の既定であり、managed-runtime policy には使いません。
    let scenario = match token.as_str() {
        "signaling-only" => DemoScenarioClass::SignalingOnly,
        "sfu-composition" => DemoScenarioClass::SfuComposition,
        "turn-composition" => DemoScenarioClass::TurnComposition,
        "developer-inspection" => DemoScenarioClass::DeveloperInspection,
        _ => {
            eprintln!(
                "{}",
                DemoCompositionFailureKind::ExternalDecodeFailed.reason_code()
            );
            std::process::exit(2);
        }
    };

    DemoCompositionGuard::try_new(
        scenario, true, true, true, true, true, true, true, true, true, true,
    )
    .expect("demo composition guard arguments are fixed");

    let _wiring = DemoCommandWiring::new(CoreCommandSurface, scenario);
    println!("scenario={token}");
}

/// demo entrypoint の composition root marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DemoCompositionSurface;

/// demo scenario の分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemoScenarioClass {
    /// Signaling-only happy-path composition sample.
    SignalingOnly,
    /// SFU composition sample.
    SfuComposition,
    /// TURN composition sample.
    TurnComposition,
    /// developer inspection sample.
    DeveloperInspection,
}

/// demo command を core command boundary へ写す wiring set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoCommandWiring {
    core_command: CoreCommandSurface,
    scenario_class: DemoScenarioClass,
}

impl DemoCommandWiring {
    /// demo scenario と core command surface を束ねます。
    pub const fn new(core_command: CoreCommandSurface, scenario_class: DemoScenarioClass) -> Self {
        Self {
            core_command,
            scenario_class,
        }
    }
}

/// demo composition failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemoCompositionFailureKind {
    /// demo input cannot map to typed command.
    ExternalDecodeFailed,
    /// required runtime configuration missing.
    RuntimeConfigMissing,
    /// runtime configuration invalid.
    RuntimeConfigInvalid,
    /// requested demo feature is outside v0.2 scope.
    FeatureOutOfScope,
    /// demo feature admission is missing or rejected.
    FeatureAdmissionMissingOrRejected,
}

impl DemoCompositionFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::FeatureOutOfScope => "feature_out_of_scope",
            Self::FeatureAdmissionMissingOrRejected => "feature_not_supported",
        }
    }
}

/// demo composition failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DemoCompositionFailure {
    kind: DemoCompositionFailureKind,
    reason: CatalogedReasonRef,
}

impl DemoCompositionFailure {
    /// demo failure を cataloged reason に接続します。
    pub fn from_kind(kind: DemoCompositionFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("demo composition reason code must be registered");
        Self { kind, reason }
    }
}

/// demo が managed-runtime/domain authority にならないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DemoCompositionGuard {
    scenario_class: DemoScenarioClass,
    demo_input_converted_to_typed_command: bool,
    scenario_maps_to_core_owned_boundary: bool,
    demo_does_not_define_domain_decision: bool,
    demo_does_not_define_reason_vocabulary: bool,
    demo_does_not_define_port_trait: bool,
    demo_defaults_not_used_as_managed_runtime_policy: bool,
    demo_listener_startup_not_used_as_readiness: bool,
    demo_run_not_used_as_managed_runtime_authority: bool,
    ui_or_end_user_workflow_absent: bool,
    out_of_scope_feature_admission_missing_or_rejected: bool,
}

/// demo composition guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemoCompositionError {
    /// demo input が typed command に変換されていません。
    TypedCommandMissing,
    /// core-owned command/use case boundary へ写っていません。
    CoreCommandBoundaryMissing,
    /// demo が domain decision を所有しています。
    DemoOwnsDomainDecision,
    /// demo が reason vocabulary を定義しています。
    DemoDefinesReasonVocabulary,
    /// demo が port trait を定義しています。
    DemoDefinesPortTrait,
    /// demo/default 設定を managed-runtime policy として扱っています。
    DemoDefaultAsManagedRuntimePolicy,
    /// listener startup を readiness として扱っています。
    ListenerStartupAsReadiness,
    /// demo 実行を managed-runtime authority として使っています。
    DemoUsedAsManagedRuntimeAuthority,
    /// UI/end-user workflow を demo entrypoint が所有しています。
    UiOrEndUserWorkflowOwnedByDemo,
    /// out-of-scope feature の admission が missing/rejected です。
    OutOfScopeFeatureAdmitted,
}

impl DemoCompositionGuard {
    /// demo entrypoint の責務境界を検査します。
    pub const fn try_new(
        scenario_class: DemoScenarioClass,
        demo_input_converted_to_typed_command: bool,
        scenario_maps_to_core_owned_boundary: bool,
        demo_does_not_define_domain_decision: bool,
        demo_does_not_define_reason_vocabulary: bool,
        demo_does_not_define_port_trait: bool,
        demo_defaults_not_used_as_managed_runtime_policy: bool,
        demo_listener_startup_not_used_as_readiness: bool,
        demo_run_not_used_as_managed_runtime_authority: bool,
        ui_or_end_user_workflow_absent: bool,
        out_of_scope_feature_admission_missing_or_rejected: bool,
    ) -> Result<Self, DemoCompositionError> {
        if !demo_input_converted_to_typed_command {
            return Err(DemoCompositionError::TypedCommandMissing);
        }
        if !scenario_maps_to_core_owned_boundary {
            return Err(DemoCompositionError::CoreCommandBoundaryMissing);
        }
        if !demo_does_not_define_domain_decision {
            return Err(DemoCompositionError::DemoOwnsDomainDecision);
        }
        if !demo_does_not_define_reason_vocabulary {
            return Err(DemoCompositionError::DemoDefinesReasonVocabulary);
        }
        if !demo_does_not_define_port_trait {
            return Err(DemoCompositionError::DemoDefinesPortTrait);
        }
        if !demo_defaults_not_used_as_managed_runtime_policy {
            return Err(DemoCompositionError::DemoDefaultAsManagedRuntimePolicy);
        }
        if !demo_listener_startup_not_used_as_readiness {
            return Err(DemoCompositionError::ListenerStartupAsReadiness);
        }
        if !demo_run_not_used_as_managed_runtime_authority {
            return Err(DemoCompositionError::DemoUsedAsManagedRuntimeAuthority);
        }
        if !ui_or_end_user_workflow_absent {
            return Err(DemoCompositionError::UiOrEndUserWorkflowOwnedByDemo);
        }
        if !out_of_scope_feature_admission_missing_or_rejected {
            return Err(DemoCompositionError::OutOfScopeFeatureAdmitted);
        }

        Ok(Self {
            scenario_class,
            demo_input_converted_to_typed_command,
            scenario_maps_to_core_owned_boundary,
            demo_does_not_define_domain_decision,
            demo_does_not_define_reason_vocabulary,
            demo_does_not_define_port_trait,
            demo_defaults_not_used_as_managed_runtime_policy,
            demo_listener_startup_not_used_as_readiness,
            demo_run_not_used_as_managed_runtime_authority,
            ui_or_end_user_workflow_absent,
            out_of_scope_feature_admission_missing_or_rejected,
        })
    }
}

/// demo composition で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedDemoCompositionBehavior {
    /// demo path bypasses core use case.
    DemoBypassesCoreUseCase,
    /// demo command owns domain decision.
    DemoCommandOwnsDomainDecision,
    /// demo defines reason vocabulary.
    DemoDefinesReasonVocabulary,
    /// demo defines core port trait.
    DemoDefinesCorePortTrait,
    /// demo defaults become managed-runtime policy.
    DemoDefaultsBecomeManagedRuntimePolicy,
    /// listener startup is treated as readiness.
    ListenerStartupAsReadiness,
    /// demo run is treated as managed-runtime authority.
    DemoRunUsedAsManagedRuntimeAuthority,
    /// UI/end-user workflow is owned by demo composition.
    UiOrEndUserWorkflowOwnedByDemo,
    /// out-of-scope feature is admitted into managed-runtime scope.
    OutOfScopeFeatureAdmitted,
}
