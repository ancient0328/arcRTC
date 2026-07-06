//! TURN server entrypoint は TURN core と network driver を接続する composition root です。
//!
//! allocation、permission、channel-bind の意味論は core/turn が所有し、
//! この binary は起動単位と wiring の入口だけを持ちます。

use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_turn::CoreTurnSurface;
use arcrtc_driver_network::NetworkDriverSurface;
use arcrtc_driver_observability::ObservabilityDriverSurface;
use arcrtc_driver_persistence::PersistenceDriverSurface;
use arcrtc_driver_security::SecurityDriverSurface;

fn main() {
    use std::io::Write;

    // 既存 public marker の使用を維持しつつ、UDP I/O はこの entrypoint だけで束ねます。
    let _surface = TurnServerCompositionSurface;
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:0".to_owned());
    let socket = match std::net::UdpSocket::bind(addr) {
        Ok(socket) => socket,
        Err(_error) => {
            eprintln!("bind_failed");
            std::process::exit(2);
        }
    };
    let local_addr = match socket.local_addr() {
        Ok(local_addr) => local_addr,
        Err(_error) => {
            eprintln!("bind_failed");
            std::process::exit(2);
        }
    };
    let mut stdout = std::io::stdout();
    if writeln!(&mut stdout, "listening={}", local_addr)
        .and_then(|_| stdout.flush())
        .is_err()
    {
        eprintln!("write_failed");
        std::process::exit(5);
    }

    if socket
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .is_err()
    {
        eprintln!("read_failed");
        std::process::exit(4);
    }
    loop {
        let mut buf = [0u8; 2049];
        let (len, peer) = match socket.recv_from(&mut buf) {
            Ok(received) => received,
            Err(_error) => {
                eprintln!("read_failed");
                continue;
            }
        };

        // 常駐化後も、TURN 判断は driver / core への既存委譲に閉じます。
        let decode_input = arcrtc_driver_network::decode_turn_datagram(&buf[..len]);
        let reason_code = match decode_input.into_core_turn_command() {
            Err(failure) => failure.kind().reason_code(),
            Ok(command) => arcrtc_core_turn::apply_initial_turn_command(&command).reason_code(),
        };
        let response = arcrtc_driver_network::encode_turn_error_datagram(reason_code);
        if socket.send_to(&response, peer).is_err() {
            eprintln!("write_failed");
            continue;
        }
    }
}

/// TURN server entrypoint の composition root marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnServerCompositionSurface;

/// TURN server が選択する core/drivers の wiring set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnServerWiringSet {
    core_turn: CoreTurnSurface,
    network_driver: NetworkDriverSurface,
    security_driver: SecurityDriverSurface,
    persistence_driver: PersistenceDriverSurface,
    observability_driver: ObservabilityDriverSurface,
}

impl TurnServerWiringSet {
    /// TURN lifecycle semantics と selected drivers を束ねます。
    pub const fn new(
        core_turn: CoreTurnSurface,
        network_driver: NetworkDriverSurface,
        security_driver: SecurityDriverSurface,
        persistence_driver: PersistenceDriverSurface,
        observability_driver: ObservabilityDriverSurface,
    ) -> Self {
        Self {
            core_turn,
            network_driver,
            security_driver,
            persistence_driver,
            observability_driver,
        }
    }
}

/// TURN server startup/wiring failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnServerStartupFailureKind {
    /// required runtime configuration missing.
    RuntimeConfigMissing,
    /// runtime configuration cannot initialize selected driver/entrypoint.
    RuntimeConfigInvalid,
    /// required secret source unavailable.
    SecretUnavailable,
    /// selected driver unavailable after bounded initialization.
    DriverShutdown,
    /// selected deployment topology unsupported.
    DeploymentTopologyUnsupported,
    /// selected public endpoint class is not admitted.
    PublicEndpointNotAllowed,
    /// runtime reconfiguration is not admitted for selected profile.
    RuntimeReconfigurationNotAllowed,
}

impl TurnServerStartupFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::SecretUnavailable => "secret_unavailable",
            Self::DriverShutdown => "driver_shutdown",
            Self::DeploymentTopologyUnsupported => "deployment_topology_unsupported",
            Self::PublicEndpointNotAllowed => "public_endpoint_not_allowed",
            Self::RuntimeReconfigurationNotAllowed => "runtime_reconfiguration_not_allowed",
        }
    }
}

/// TURN server startup/wiring failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnServerStartupFailure {
    kind: TurnServerStartupFailureKind,
    reason: CatalogedReasonRef,
}

impl TurnServerStartupFailure {
    /// startup failure を cataloged reason に接続します。
    pub fn from_kind(kind: TurnServerStartupFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("turn server startup reason code must be registered");
        Self { kind, reason }
    }
}

/// TURN server composition root が domain authority を持たないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnServerCompositionGuard {
    typed_runtime_configuration_constructed: bool,
    selected_drivers_declared: bool,
    core_use_case_wired_through_allowed_boundary: bool,
    allocation_lifecycle_not_owned_by_app: bool,
    permission_decision_not_owned_by_app: bool,
    channel_bind_decision_not_owned_by_app: bool,
    relay_authorization_not_owned_by_app: bool,
    turn_credential_issuance_not_owned_by_app: bool,
    entrypoint_does_not_define_reason_vocabulary: bool,
    entrypoint_does_not_define_port_trait: bool,
    regulated_not_wired_into_generic_path: bool,
    required_startup_failure_is_fail_closed: bool,
    audit_or_bootstrap_record_path_declared: bool,
}

/// TURN server composition guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnServerCompositionError {
    /// typed runtime configuration がありません。
    RuntimeConfigurationMissing,
    /// selected driver implementation が宣言されていません。
    SelectedDriverMissing,
    /// core use case を許可 boundary 以外で呼んでいます。
    CoreUseCaseBoundaryBypassed,
    /// entrypoints が TURN allocation lifecycle を所有しています。
    EntrypointOwnsAllocationLifecycle,
    /// entrypoints が TURN permission decision を所有しています。
    EntrypointOwnsPermissionDecision,
    /// entrypoints が TURN channel-bind decision を所有しています。
    EntrypointOwnsChannelBindDecision,
    /// entrypoints が relay authorization を所有しています。
    EntrypointOwnsRelayAuthorization,
    /// entrypoints が TURN credential issuance を所有しています。
    EntrypointOwnsTurnCredentialIssuance,
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

impl TurnServerCompositionGuard {
    /// TURN server composition root の責務境界を検査します。
    pub const fn try_new(
        typed_runtime_configuration_constructed: bool,
        selected_drivers_declared: bool,
        core_use_case_wired_through_allowed_boundary: bool,
        allocation_lifecycle_not_owned_by_app: bool,
        permission_decision_not_owned_by_app: bool,
        channel_bind_decision_not_owned_by_app: bool,
        relay_authorization_not_owned_by_app: bool,
        turn_credential_issuance_not_owned_by_app: bool,
        entrypoint_does_not_define_reason_vocabulary: bool,
        entrypoint_does_not_define_port_trait: bool,
        regulated_not_wired_into_generic_path: bool,
        required_startup_failure_is_fail_closed: bool,
        audit_or_bootstrap_record_path_declared: bool,
    ) -> Result<Self, TurnServerCompositionError> {
        if !typed_runtime_configuration_constructed {
            return Err(TurnServerCompositionError::RuntimeConfigurationMissing);
        }
        if !selected_drivers_declared {
            return Err(TurnServerCompositionError::SelectedDriverMissing);
        }
        if !core_use_case_wired_through_allowed_boundary {
            return Err(TurnServerCompositionError::CoreUseCaseBoundaryBypassed);
        }
        if !allocation_lifecycle_not_owned_by_app {
            return Err(TurnServerCompositionError::EntrypointOwnsAllocationLifecycle);
        }
        if !permission_decision_not_owned_by_app {
            return Err(TurnServerCompositionError::EntrypointOwnsPermissionDecision);
        }
        if !channel_bind_decision_not_owned_by_app {
            return Err(TurnServerCompositionError::EntrypointOwnsChannelBindDecision);
        }
        if !relay_authorization_not_owned_by_app {
            return Err(TurnServerCompositionError::EntrypointOwnsRelayAuthorization);
        }
        if !turn_credential_issuance_not_owned_by_app {
            return Err(TurnServerCompositionError::EntrypointOwnsTurnCredentialIssuance);
        }
        if !entrypoint_does_not_define_reason_vocabulary {
            return Err(TurnServerCompositionError::EntrypointDefinesReasonVocabulary);
        }
        if !entrypoint_does_not_define_port_trait {
            return Err(TurnServerCompositionError::EntrypointDefinesPortTrait);
        }
        if !regulated_not_wired_into_generic_path {
            return Err(TurnServerCompositionError::RegulatedPathMixed);
        }
        if !required_startup_failure_is_fail_closed {
            return Err(TurnServerCompositionError::StartupFailureNotFailClosed);
        }
        if !audit_or_bootstrap_record_path_declared {
            return Err(TurnServerCompositionError::StartupFailureRecordPathMissing);
        }

        Ok(Self {
            typed_runtime_configuration_constructed,
            selected_drivers_declared,
            core_use_case_wired_through_allowed_boundary,
            allocation_lifecycle_not_owned_by_app,
            permission_decision_not_owned_by_app,
            channel_bind_decision_not_owned_by_app,
            relay_authorization_not_owned_by_app,
            turn_credential_issuance_not_owned_by_app,
            entrypoint_does_not_define_reason_vocabulary,
            entrypoint_does_not_define_port_trait,
            regulated_not_wired_into_generic_path,
            required_startup_failure_is_fail_closed,
            audit_or_bootstrap_record_path_declared,
        })
    }
}

/// TURN server composition root で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedTurnServerCompositionBehavior {
    /// executable entrypoint owns TURN allocation lifecycle.
    EntrypointOwnsAllocationLifecycle,
    /// executable entrypoint owns TURN permission decision.
    EntrypointOwnsPermissionDecision,
    /// executable entrypoint owns channel-bind decision.
    EntrypointOwnsChannelBindDecision,
    /// executable entrypoint owns relay authorization.
    EntrypointOwnsRelayAuthorization,
    /// entrypoints issue TURN credentials.
    EntrypointIssuesTurnCredentials,
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
}

#[cfg(test)]
mod tests {
    use super::{TurnServerCompositionError, TurnServerCompositionGuard};

    #[test]
    fn composition_guard_rejects_entrypoint_owned_turn_relay_or_credential_semantics() {
        // TURN binary は wiring root であり、relay authorization や credential issuance を所有しません。
        assert_eq!(
            TurnServerCompositionGuard::try_new(
                true, true, true, true, true, true, false, true, true, true, true, true, true,
            ),
            Err(TurnServerCompositionError::EntrypointOwnsRelayAuthorization)
        );
    }
}
