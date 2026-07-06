//! SFU server entrypoint は SFU core と drivers を接続する composition root です。
//!
//! routing、forwarding、quality decision は core/sfu と core/quality が所有し、
//! この binary は起動単位と wiring の入口だけを持ちます。

use arcrtc_core_quality::CoreQualitySurface;
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_sfu::CoreSfuSurface;
use arcrtc_core_transport::CoreTransportSurface;
use arcrtc_driver_network::NetworkDriverSurface;
use arcrtc_driver_observability::ObservabilityDriverSurface;
use arcrtc_driver_persistence::PersistenceDriverSurface;
use arcrtc_driver_webrtc_str0m::Str0mDriverSurface;

fn main() {
    let addr = match std::env::args().nth(1) {
        Some(addr) => addr,
        None => "127.0.0.1:0".to_owned(),
    };

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

    println!("listening={local_addr}");
    {
        let mut stdout = std::io::stdout();
        let _ = std::io::Write::flush(&mut stdout);
    }

    // entrypoint は起動と wiring のみを持ち、datagram 解釈は driver へ委譲します。
    let mut engine = arcrtc_driver_webrtc_str0m::Str0mMediaEngine::new(std::time::Instant::now());

    if socket
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .is_err()
    {
        eprintln!("read_failed");
        std::process::exit(4);
    }

    let mut buf = [0u8; 2000];
    let (n, source) = match socket.recv_from(&mut buf) {
        Ok(received) => received,
        Err(_error) => {
            eprintln!("read_failed");
            std::process::exit(4);
        }
    };

    if let Err(failure) =
        engine.ingest_udp_datagram(std::time::Instant::now(), source, local_addr, &buf[..n])
    {
        println!("outcome=rejected reason={}", failure.kind().reason_code());
        let mut stdout = std::io::stdout();
        let _ = std::io::Write::flush(&mut stdout);
        std::process::exit(0);
    }

    let report = match engine.drain_outbound() {
        Ok(report) => report,
        Err(failure) => {
            println!("outcome=rejected reason={}", failure.kind().reason_code());
            let mut stdout = std::io::stdout();
            let _ = std::io::Write::flush(&mut stdout);
            std::process::exit(0);
        }
    };

    let mut transmits = 0usize;
    for outbound in report.outbound() {
        if socket
            .send_to(outbound.payload(), outbound.destination())
            .is_err()
        {
            eprintln!("write_failed");
            std::process::exit(5);
        }
        transmits += 1;
    }

    println!(
        "outcome=ok transmits={transmits} dropped={}",
        report.dropped_over_bound()
    );
    let mut stdout = std::io::stdout();
    let _ = std::io::Write::flush(&mut stdout);
}

/// SFU server entrypoint の composition root marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SfuServerCompositionSurface;

/// SFU server が選択する core/drivers の wiring set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SfuServerWiringSet {
    core_sfu: CoreSfuSurface,
    core_quality: CoreQualitySurface,
    core_transport: CoreTransportSurface,
    webrtc_transport_driver: Str0mDriverSurface,
    network_driver: NetworkDriverSurface,
    persistence_driver: PersistenceDriverSurface,
    observability_driver: ObservabilityDriverSurface,
}

impl SfuServerWiringSet {
    /// SFU routing/quality/transport semantics と selected drivers を束ねます。
    pub const fn new(
        core_sfu: CoreSfuSurface,
        core_quality: CoreQualitySurface,
        core_transport: CoreTransportSurface,
        webrtc_transport_driver: Str0mDriverSurface,
        network_driver: NetworkDriverSurface,
        persistence_driver: PersistenceDriverSurface,
        observability_driver: ObservabilityDriverSurface,
    ) -> Self {
        Self {
            core_sfu,
            core_quality,
            core_transport,
            webrtc_transport_driver,
            network_driver,
            persistence_driver,
            observability_driver,
        }
    }
}

/// SFU server startup/wiring failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuServerStartupFailureKind {
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

impl SfuServerStartupFailureKind {
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

/// SFU server startup/wiring failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SfuServerStartupFailure {
    kind: SfuServerStartupFailureKind,
    reason: CatalogedReasonRef,
}

impl SfuServerStartupFailure {
    /// startup failure を cataloged reason に接続します。
    pub fn from_kind(kind: SfuServerStartupFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("sfu server startup reason code must be registered");
        Self { kind, reason }
    }
}

/// SFU server composition root が domain authority を持たないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SfuServerCompositionGuard {
    typed_runtime_configuration_constructed: bool,
    selected_drivers_declared: bool,
    core_use_case_wired_through_allowed_boundary: bool,
    sfu_route_selection_not_owned_by_app: bool,
    forwarding_decision_not_owned_by_app: bool,
    quality_or_backpressure_policy_not_owned_by_app: bool,
    entrypoint_does_not_define_reason_vocabulary: bool,
    entrypoint_does_not_define_port_trait: bool,
    regulated_not_wired_into_generic_path: bool,
    required_startup_failure_is_fail_closed: bool,
    audit_or_bootstrap_record_path_declared: bool,
}

/// SFU server composition guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuServerCompositionError {
    /// typed runtime configuration がありません。
    RuntimeConfigurationMissing,
    /// selected driver implementation が宣言されていません。
    SelectedDriverMissing,
    /// core use case を許可 boundary 以外で呼んでいます。
    CoreUseCaseBoundaryBypassed,
    /// entrypoints が SFU route selection を所有しています。
    EntrypointOwnsSfuRouteSelection,
    /// entrypoints が forwarding decision を所有しています。
    EntrypointOwnsForwardingDecision,
    /// entrypoints が quality/backpressure policy を所有しています。
    EntrypointOwnsQualityOrBackpressurePolicy,
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

impl SfuServerCompositionGuard {
    /// SFU server composition root の責務境界を検査します。
    pub const fn try_new(
        typed_runtime_configuration_constructed: bool,
        selected_drivers_declared: bool,
        core_use_case_wired_through_allowed_boundary: bool,
        sfu_route_selection_not_owned_by_app: bool,
        forwarding_decision_not_owned_by_app: bool,
        quality_or_backpressure_policy_not_owned_by_app: bool,
        entrypoint_does_not_define_reason_vocabulary: bool,
        entrypoint_does_not_define_port_trait: bool,
        regulated_not_wired_into_generic_path: bool,
        required_startup_failure_is_fail_closed: bool,
        audit_or_bootstrap_record_path_declared: bool,
    ) -> Result<Self, SfuServerCompositionError> {
        if !typed_runtime_configuration_constructed {
            return Err(SfuServerCompositionError::RuntimeConfigurationMissing);
        }
        if !selected_drivers_declared {
            return Err(SfuServerCompositionError::SelectedDriverMissing);
        }
        if !core_use_case_wired_through_allowed_boundary {
            return Err(SfuServerCompositionError::CoreUseCaseBoundaryBypassed);
        }
        if !sfu_route_selection_not_owned_by_app {
            return Err(SfuServerCompositionError::EntrypointOwnsSfuRouteSelection);
        }
        if !forwarding_decision_not_owned_by_app {
            return Err(SfuServerCompositionError::EntrypointOwnsForwardingDecision);
        }
        if !quality_or_backpressure_policy_not_owned_by_app {
            return Err(SfuServerCompositionError::EntrypointOwnsQualityOrBackpressurePolicy);
        }
        if !entrypoint_does_not_define_reason_vocabulary {
            return Err(SfuServerCompositionError::EntrypointDefinesReasonVocabulary);
        }
        if !entrypoint_does_not_define_port_trait {
            return Err(SfuServerCompositionError::EntrypointDefinesPortTrait);
        }
        if !regulated_not_wired_into_generic_path {
            return Err(SfuServerCompositionError::RegulatedPathMixed);
        }
        if !required_startup_failure_is_fail_closed {
            return Err(SfuServerCompositionError::StartupFailureNotFailClosed);
        }
        if !audit_or_bootstrap_record_path_declared {
            return Err(SfuServerCompositionError::StartupFailureRecordPathMissing);
        }

        Ok(Self {
            typed_runtime_configuration_constructed,
            selected_drivers_declared,
            core_use_case_wired_through_allowed_boundary,
            sfu_route_selection_not_owned_by_app,
            forwarding_decision_not_owned_by_app,
            quality_or_backpressure_policy_not_owned_by_app,
            entrypoint_does_not_define_reason_vocabulary,
            entrypoint_does_not_define_port_trait,
            regulated_not_wired_into_generic_path,
            required_startup_failure_is_fail_closed,
            audit_or_bootstrap_record_path_declared,
        })
    }
}

/// SFU server composition root で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedSfuServerCompositionBehavior {
    /// executable entrypoint owns route selection.
    EntrypointOwnsSfuRouteSelection,
    /// executable entrypoint owns forwarding decision.
    EntrypointOwnsForwardingDecision,
    /// entrypoints define quality/backpressure policy.
    EntrypointDefinesQualityOrBackpressurePolicy,
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
    /// listener startup is treated as SFU media readiness.
    ListenerStartupAsSfuMediaReadiness,
}

#[cfg(test)]
mod tests {
    use super::{SfuServerCompositionError, SfuServerCompositionGuard};

    #[test]
    fn composition_guard_rejects_entrypoint_owned_quality_or_forwarding_semantics() {
        // SFU binary は wiring root であり、routing/quality/backpressure policy を所有しません。
        assert_eq!(
            SfuServerCompositionGuard::try_new(
                true, true, true, true, true, false, true, true, true, true, true,
            ),
            Err(SfuServerCompositionError::EntrypointOwnsQualityOrBackpressurePolicy)
        );
    }
}
