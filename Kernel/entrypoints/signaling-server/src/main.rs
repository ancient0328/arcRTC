//! Signaling server entrypoint は core と drivers を接続する composition root です。
//!
//! Signaling の accept/reject 意味論は core/signaling に置き、この binary は
//! 起動単位と wiring の入口だけを所有します。

use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_recovery::{
    CommandRoutingRule, DistributedAuditRelation, DistributedConflictRule,
    DistributedStateAdmission, DistributedStateClass, DistributedStatePolicy, OwnerNodeScope,
    PacketRoutingRule, RecoveryRestoreRelation, RestartReadinessClass, SplitBrainGuard,
};
use arcrtc_core_signaling::CoreSignalingSurface;
use arcrtc_core_state::StateFamily;
use arcrtc_driver_network::NetworkDriverSurface;
use arcrtc_driver_observability::ObservabilityDriverSurface;
use arcrtc_driver_persistence::PersistenceDriverSurface;
use arcrtc_driver_security::{
    SecretRotationExecutionBoundaryGuard, SecurityDriverSurface, TokenVerifierDriverBoundaryGuard,
};

fn startup_owner_reference() -> OpaqueReference {
    OpaqueReference::accept(
        std::process::id().to_string(),
        ReferenceAuthority::CoreValidatedStartupInput,
    )
    .expect("process id string is a non-empty control-free opaque reference")
}

fn admit_node_local_state(
    state_family: StateFamily,
    owner_ref: OpaqueReference,
) -> DistributedStateAdmission {
    let policy = DistributedStatePolicy::try_new(
        state_family,
        DistributedStateClass::NodeLocalState,
        OwnerNodeScope::SingleNode,
        Some(owner_ref),
        None,
        CommandRoutingRule::NotCrossNodeRouted,
        PacketRoutingRule::NotPacketScoped,
        RecoveryRestoreRelation::NoRestoreRelation,
        DistributedConflictRule::RejectConflictingOwner,
        true,
        DistributedAuditRelation::AuditVerificationOnly,
    )
    .expect("node-local distributed state policy arguments are fixed");

    DistributedStateAdmission::admit_initial_v0_2(policy)
        .expect("node-local state is admitted in initial v0.2")
}

fn startup_split_brain_guard() -> SplitBrainGuard {
    SplitBrainGuard::try_new(false, false)
        .expect("a single accept loop per process cannot accept the same owner scope twice")
}

fn startup_readiness_class() -> RestartReadinessClass {
    RestartReadinessClass::ProcessReadinessObservationOnly
}

fn secret_boundary_guards() -> (
    TokenVerifierDriverBoundaryGuard,
    SecretRotationExecutionBoundaryGuard,
) {
    let verifier_guard =
        TokenVerifierDriverBoundaryGuard::try_new(true, true, true, true, true, true, true, true)
            .expect("verifier boundary arguments are fixed");
    let rotation_guard = SecretRotationExecutionBoundaryGuard::try_new(
        true, true, true, true, true, true, true, true,
    )
    .expect("rotation boundary arguments are fixed");

    (verifier_guard, rotation_guard)
}

fn main() {
    // 既存 public marker の使用を維持しつつ、実 I/O はこの entrypoint だけで束ねます。
    let _surface = SignalingServerCompositionSurface;
    // P1 は process 内の node-local 所有境界だけを core/recovery へ接続します。
    let owner_ref = startup_owner_reference();
    let _split_brain_guard = startup_split_brain_guard();
    let _room_admission = admit_node_local_state(StateFamily::SignalingRoom, owner_ref.clone());
    let _participant_admission =
        admit_node_local_state(StateFamily::SignalingParticipant, owner_ref);
    let _readiness_class = startup_readiness_class();
    let _secret_boundary_guards = secret_boundary_guards();

    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:0".to_owned());
    let listener = match std::net::TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(_error) => {
            eprintln!("bind_failed");
            std::process::exit(2);
        }
    };
    let local_addr = match listener.local_addr() {
        Ok(local_addr) => local_addr,
        Err(_error) => {
            eprintln!("bind_failed");
            std::process::exit(2);
        }
    };
    println!("listening={}", local_addr);
    let mut stdout = std::io::stdout();
    let _ = std::io::Write::flush(&mut stdout);

    loop {
        let (mut stream, _peer_addr) = match listener.accept() {
            Ok(accepted) => accepted,
            Err(_error) => {
                eprintln!("accept_failed");
                continue;
            }
        };
        if stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .is_err()
        {
            eprintln!("read_failed");
            continue;
        }

        let mut received = Vec::new();
        let read_result = {
            let mut limited = std::io::Read::take(&mut stream, 65537);
            std::io::Read::read_to_end(&mut limited, &mut received)
        };
        if read_result.is_err() {
            eprintln!("read_failed");
            continue;
        }

        // 常駐化後も、状態判断と wire 変換判断は core / driver への既存委譲に閉じます。
        let response_frame = if received.len() == 65537 {
            arcrtc_driver_network::encode_signaling_rejection_frame(
                arcrtc_driver_network::DriverConversionFailureKind::FrameSizeBoundExceeded,
            )
        } else {
            match arcrtc_driver_network::decode_signaling_command_frame(&received) {
                Err(failure) => {
                    arcrtc_driver_network::encode_signaling_rejection_frame(failure.kind())
                }
                Ok(decoded) => match arcrtc_driver_network::build_signaling_command(decoded) {
                    Err(failure) => {
                        arcrtc_driver_network::encode_signaling_rejection_frame(failure.kind())
                    }
                    Ok(command) => {
                        let command_kind = command.kind();
                        let result = arcrtc_core_signaling::apply_one_shot_signaling_command(
                            command_kind,
                            arcrtc_core_signaling::INITIAL_SIGNALING_ROOM_STATE,
                            arcrtc_core_signaling::INITIAL_SIGNALING_PARTICIPANT_STATE,
                        );
                        let (event, reason) = arcrtc_core_signaling::one_shot_signaling_response(
                            command_kind,
                            &result,
                        );
                        arcrtc_driver_network::encode_signaling_event_frame(event, reason)
                    }
                },
            }
        };

        if std::io::Write::write_all(&mut stream, &response_frame).is_err()
            || std::io::Write::flush(&mut stream).is_err()
        {
            eprintln!("write_failed");
            continue;
        }
        drop(stream);
    }
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
