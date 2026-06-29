//! Signaling server entrypoint は core と drivers を接続する composition root です。
//!
//! Signaling の accept/reject 意味論は core/signaling に置き、この binary は
//! 起動単位と wiring の入口だけを所有します。

use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_signaling::CoreSignalingSurface;
use arcrtc_driver_network::NetworkDriverSurface;
use arcrtc_driver_observability::ObservabilityDriverSurface;
use arcrtc_driver_persistence::PersistenceDriverSurface;
use arcrtc_driver_security::SecurityDriverSurface;

fn main() {
    // 実行時の具体起動は後続の runtime wiring で扱い、ここでは composition root を固定します。
    let _surface = SignalingServerCompositionSurface;
}

/// Signaling server entrypoint の composition root marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalingServerCompositionSurface;

/// Signaling server が選択する core/drivers の wiring set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalingServerWiringSet {
    core_signaling: CoreSignalingSurface,
    network_driver: NetworkDriverSurface,
    security_driver: SecurityDriverSurface,
    persistence_driver: PersistenceDriverSurface,
    observability_driver: ObservabilityDriverSurface,
}

impl SignalingServerWiringSet {
    /// core-owned Signaling semantics と selected drivers を束ねます。
    pub const fn new(
        core_signaling: CoreSignalingSurface,
        network_driver: NetworkDriverSurface,
        security_driver: SecurityDriverSurface,
        persistence_driver: PersistenceDriverSurface,
        observability_driver: ObservabilityDriverSurface,
    ) -> Self {
        Self {
            core_signaling,
            network_driver,
            security_driver,
            persistence_driver,
            observability_driver,
        }
    }
}

/// Signaling server startup/wiring failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingServerStartupFailureKind {
    /// required runtime configuration missing.
    RuntimeConfigMissing,
    /// runtime configuration cannot initialize selected driver/entrypoint.
    RuntimeConfigInvalid,
    /// required secret source unavailable.
    SecretUnavailable,
    /// selected driver unavailable after bounded initialization.
    DriverShutdown,
    /// selected public endpoint class is not admitted.
    PublicEndpointNotAllowed,
    /// runtime reconfiguration is not admitted for selected profile.
    RuntimeReconfigurationNotAllowed,
}

impl SignalingServerStartupFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::SecretUnavailable => "secret_unavailable",
            Self::DriverShutdown => "driver_shutdown",
            Self::PublicEndpointNotAllowed => "public_endpoint_not_allowed",
            Self::RuntimeReconfigurationNotAllowed => "runtime_reconfiguration_not_allowed",
        }
    }
}

/// Signaling server startup/wiring failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalingServerStartupFailure {
    kind: SignalingServerStartupFailureKind,
    reason: CatalogedReasonRef,
}

impl SignalingServerStartupFailure {
    /// startup failure を cataloged reason に接続します。
    pub fn from_kind(kind: SignalingServerStartupFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("signaling server startup reason code must be registered");
        Self { kind, reason }
    }
}

/// Signaling server composition root が domain authority を持たないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalingServerCompositionGuard {
    typed_runtime_configuration_constructed: bool,
    selected_drivers_declared: bool,
    core_use_case_wired_through_allowed_boundary: bool,
    signaling_join_acceptance_not_owned_by_app: bool,
    room_state_transition_not_owned_by_app: bool,
    entrypoint_does_not_define_reason_vocabulary: bool,
    entrypoint_does_not_define_port_trait: bool,
    regulated_not_wired_into_generic_path: bool,
    required_startup_failure_is_fail_closed: bool,
    audit_or_bootstrap_record_path_declared: bool,
}

/// Signaling server composition guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingServerCompositionError {
    /// typed runtime configuration がありません。
    RuntimeConfigurationMissing,
    /// selected driver implementation が宣言されていません。
    SelectedDriverMissing,
    /// core use case を許可 boundary 以外で呼んでいます。
    CoreUseCaseBoundaryBypassed,
    /// entrypoints が Signaling join acceptance を所有しています。
    EntrypointOwnsSignalingJoinDecision,
    /// entrypoints が room state transition を所有しています。
    EntrypointOwnsRoomStateTransition,
    /// entrypoints が reason vocabulary を定義しています。
    EntrypointDefinesReasonVocabulary,
    /// entrypoints が port trait を定義しています。
    EntrypointDefinesPortTrait,
    /// regulated support が generic communication path に混入しています。
    RegulatedPathMixed,
    /// startup failure が fail-closed になっていません。
    StartupFailureNotFailClosed,
    /// audit/bootstrap record path がありません。
    StartupFailureRecordPathMissing,
}

impl SignalingServerCompositionGuard {
    /// Signaling server composition root の責務境界を検査します。
    pub const fn try_new(
        typed_runtime_configuration_constructed: bool,
        selected_drivers_declared: bool,
        core_use_case_wired_through_allowed_boundary: bool,
        signaling_join_acceptance_not_owned_by_app: bool,
        room_state_transition_not_owned_by_app: bool,
        entrypoint_does_not_define_reason_vocabulary: bool,
        entrypoint_does_not_define_port_trait: bool,
        regulated_not_wired_into_generic_path: bool,
        required_startup_failure_is_fail_closed: bool,
        audit_or_bootstrap_record_path_declared: bool,
    ) -> Result<Self, SignalingServerCompositionError> {
        if !typed_runtime_configuration_constructed {
            return Err(SignalingServerCompositionError::RuntimeConfigurationMissing);
        }
        if !selected_drivers_declared {
            return Err(SignalingServerCompositionError::SelectedDriverMissing);
        }
        if !core_use_case_wired_through_allowed_boundary {
            return Err(SignalingServerCompositionError::CoreUseCaseBoundaryBypassed);
        }
        if !signaling_join_acceptance_not_owned_by_app {
            return Err(SignalingServerCompositionError::EntrypointOwnsSignalingJoinDecision);
        }
        if !room_state_transition_not_owned_by_app {
            return Err(SignalingServerCompositionError::EntrypointOwnsRoomStateTransition);
        }
        if !entrypoint_does_not_define_reason_vocabulary {
            return Err(SignalingServerCompositionError::EntrypointDefinesReasonVocabulary);
        }
        if !entrypoint_does_not_define_port_trait {
            return Err(SignalingServerCompositionError::EntrypointDefinesPortTrait);
        }
        if !regulated_not_wired_into_generic_path {
            return Err(SignalingServerCompositionError::RegulatedPathMixed);
        }
        if !required_startup_failure_is_fail_closed {
            return Err(SignalingServerCompositionError::StartupFailureNotFailClosed);
        }
        if !audit_or_bootstrap_record_path_declared {
            return Err(SignalingServerCompositionError::StartupFailureRecordPathMissing);
        }

        Ok(Self {
            typed_runtime_configuration_constructed,
            selected_drivers_declared,
            core_use_case_wired_through_allowed_boundary,
            signaling_join_acceptance_not_owned_by_app,
            room_state_transition_not_owned_by_app,
            entrypoint_does_not_define_reason_vocabulary,
            entrypoint_does_not_define_port_trait,
            regulated_not_wired_into_generic_path,
            required_startup_failure_is_fail_closed,
            audit_or_bootstrap_record_path_declared,
        })
    }
}

/// Signaling server composition root で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedSignalingServerCompositionBehavior {
    /// executable entrypoint owns Signaling join acceptance.
    EntrypointOwnsSignalingJoinAcceptance,
    /// entrypoints define a second reason catalog.
    EntrypointDefinesSecondReasonCatalog,
    /// entrypoints define core port traits.
    EntrypointDefinesCorePortTrait,
    /// entrypoints bypass core-owned use case / port boundary.
    EntrypointBypassesCoreUseCaseBoundary,
    /// startup failure is hidden while claiming readiness.
    StartupFailureHidden,
    /// regulated support is wired into generic communication path.
    RegulatedSupportInGenericCommunicationPath,
    /// listener startup is treated as public endpoint readiness.
    ListenerStartupAsEndpointReadiness,
    /// demo/default configuration becomes production policy.
    DemoDefaultAsProductionPolicy,
}

#[cfg(test)]
mod tests {
    use super::{SignalingServerCompositionError, SignalingServerCompositionGuard};

    #[test]
    fn composition_guard_rejects_entrypoint_owned_signaling_semantics() {
        // entrypoint binary は wiring root であり、Signaling join の accept/reject 意味論を所有しません。
        assert_eq!(
            SignalingServerCompositionGuard::try_new(
                true, true, true, false, true, true, true, true, true, true,
            ),
            Err(SignalingServerCompositionError::EntrypointOwnsSignalingJoinDecision)
        );
    }
}
