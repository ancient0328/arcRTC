//! reference runtime executorです。

use std::net::SocketAddr;

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{
    validate_evidence_record, DistroCommandClass, DistroEnvironmentClass, DistroEvidenceReason,
    DistroEvidenceRecord, DistroLayer, DistroNonClaimScope, DistroPlane, DISTRO_COMMAND_ROOT,
};
use arcrtc_reference_composition::ReferenceCompositionState;
use tokio::net::TcpListener;

use crate::error::ReferenceRuntimeError;

/// runtimeが扱うdistro planeです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistroRuntimePlane {
    /// Signaling planeです。
    Signaling,
    /// TURN planeです。
    Turn,
    /// SFU planeです。
    Sfu,
    /// Composition planeです。
    Composition,
}

/// runtime stateです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistroRuntimeState {
    /// created stateです。
    Created,
    /// starting stateです。
    Starting,
    /// running stateです。
    Running,
    /// draining stateです。
    Draining,
    /// stopped stateです。
    Stopped,
    /// failed stateです。
    Failed,
}

/// shutdown modeです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistroShutdownMode {
    /// immediate shutdownです。
    Immediate,
    /// local graceful shutdownです。
    GracefulLocal,
    /// drain then stop shutdownです。
    DrainThenStop,
}

/// reference build command evidence の入力です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceBuildEvidenceInput {
    /// command / report / span を接続する相関IDです。
    pub correlation_id: CorrelationId,
    /// 実行した build command です。
    pub command: String,
    /// build target packageです。
    pub target_package: String,
    /// build target scopeです。
    pub target_scope: String,
    /// toolchain / runtime version表記です。
    pub toolchain_runtime_version: String,
    /// 期待結果です。
    pub expected_outcome: String,
    /// 実結果です。
    pub actual_outcome: String,
    /// process exit statusです。
    pub exit_status: i32,
}

/// reference runtime lifecycle outcomeです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceRuntimeOutcome {
    /// command / report / span を接続する相関IDです。
    pub correlation_id: CorrelationId,
    /// runtime action 後のstateです。
    pub state: DistroRuntimeState,
    /// runtime action の実装側reasonです。
    pub distro_reason: DistroEvidenceReason,
    /// runtime action が主張しない範囲です。
    pub non_claim_scope: Vec<DistroNonClaimScope>,
}

/// reference runtime executorです。
#[derive(Debug)]
pub struct ReferenceRuntime {
    /// runtime stateです。
    state: DistroRuntimeState,
    /// composition stateです。
    composition: ReferenceCompositionState,
    /// socket probe の listener です。外部公開 endpoint ではなく、Running の実 I/O 生存性だけを保持します。
    socket_probe_listener: Option<TcpListener>,
}

/// reference runtime socket probe の観測結果です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceRuntimeSocketProbe {
    /// 実際にbindされたlocal addressです。
    local_addr: SocketAddr,
}

impl ReferenceRuntimeSocketProbe {
    /// 実際にbindされたlocal addressを返します。
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl ReferenceRuntime {
    /// reference runtimeを作ります。
    pub const fn new(composition: ReferenceCompositionState) -> Self {
        Self {
            state: DistroRuntimeState::Created,
            composition,
            socket_probe_listener: None,
        }
    }

    /// runtimeを開始します。
    pub fn start(
        &mut self,
        correlation_id: CorrelationId,
    ) -> Result<ReferenceRuntimeOutcome, ReferenceRuntimeError> {
        if self.state != DistroRuntimeState::Created {
            self.state = DistroRuntimeState::Failed;
            return Err(ReferenceRuntimeError::RuntimeExecutorError);
        }
        self.state = DistroRuntimeState::Starting;
        self.state = DistroRuntimeState::Running;
        Ok(reference_runtime_outcome(correlation_id, self.state))
    }

    /// 実socket bindを伴ってruntimeを開始します。
    pub async fn start_with_socket_probe(
        &mut self,
        correlation_id: CorrelationId,
        bind_addr: SocketAddr,
    ) -> Result<(ReferenceRuntimeOutcome, ReferenceRuntimeSocketProbe), ReferenceRuntimeError> {
        if self.state != DistroRuntimeState::Created {
            self.state = DistroRuntimeState::Failed;
            return Err(ReferenceRuntimeError::RuntimeExecutorError);
        }
        self.state = DistroRuntimeState::Starting;
        let listener = TcpListener::bind(bind_addr).await.map_err(|_| {
            self.state = DistroRuntimeState::Failed;
            ReferenceRuntimeError::RuntimeExecutorError
        })?;
        let local_addr = listener.local_addr().map_err(|_| {
            self.state = DistroRuntimeState::Failed;
            ReferenceRuntimeError::RuntimeExecutorError
        })?;
        // listener を runtime に保持することで、Running が単なる enum 代入でないことを示します。
        self.socket_probe_listener = Some(listener);
        self.state = DistroRuntimeState::Running;
        Ok((
            reference_runtime_outcome(correlation_id, self.state),
            ReferenceRuntimeSocketProbe { local_addr },
        ))
    }

    /// runtimeをshutdownします。
    pub fn shutdown(
        &mut self,
        correlation_id: CorrelationId,
        mode: DistroShutdownMode,
    ) -> Result<ReferenceRuntimeOutcome, ReferenceRuntimeError> {
        if self.state != DistroRuntimeState::Running {
            self.state = DistroRuntimeState::Failed;
            return Err(ReferenceRuntimeError::RuntimeExecutorError);
        }
        match mode {
            DistroShutdownMode::Immediate => {
                self.socket_probe_listener = None;
                self.state = DistroRuntimeState::Stopped;
            }
            DistroShutdownMode::GracefulLocal | DistroShutdownMode::DrainThenStop => {
                self.state = DistroRuntimeState::Draining;
                self.socket_probe_listener = None;
                self.state = DistroRuntimeState::Stopped;
            }
        }
        Ok(reference_runtime_outcome(correlation_id, self.state))
    }

    /// runtime stateを返します。
    pub const fn state(&self) -> DistroRuntimeState {
        self.state
    }

    /// composition stateをborrowします。
    pub const fn composition(&self) -> &ReferenceCompositionState {
        &self.composition
    }
}

fn reference_runtime_outcome(
    correlation_id: CorrelationId,
    state: DistroRuntimeState,
) -> ReferenceRuntimeOutcome {
    ReferenceRuntimeOutcome {
        correlation_id,
        state,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: vec![
            DistroNonClaimScope::BehaviorCorrectnessNotClaimed,
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
    }
}

/// reference build command evidence recordを作ります。
pub fn build_reference_build_evidence_record(
    input: ReferenceBuildEvidenceInput,
) -> Result<DistroEvidenceRecord, ReferenceRuntimeError> {
    let record = DistroEvidenceRecord {
        correlation_id: input.correlation_id.as_str().to_owned(),
        command: input.command,
        working_directory: DISTRO_COMMAND_ROOT.to_owned(),
        target_package: Some(input.target_package),
        target_scope: input.target_scope,
        command_class: DistroCommandClass::Build,
        distro_layer: DistroLayer::Reference,
        target_plane: DistroPlane::Ops,
        environment_class: DistroEnvironmentClass::ControlledProcess,
        toolchain_runtime_version: input.toolchain_runtime_version,
        input_fixture_or_workload: None,
        expected_outcome: input.expected_outcome,
        actual_outcome: input.actual_outcome,
        exit_status: Some(input.exit_status),
        kernel_reason: None,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: vec![
            DistroNonClaimScope::BehaviorCorrectnessNotClaimed,
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "reference ops source or build command change".to_owned(),
    };
    validate_evidence_record(&record)
        .map_err(|_| ReferenceRuntimeError::EvidenceFieldsIncomplete)?;
    validate_reference_build_evidence_ownership(&record)?;
    Ok(record)
}

fn validate_reference_build_evidence_ownership(
    record: &DistroEvidenceRecord,
) -> Result<(), ReferenceRuntimeError> {
    if record.distro_layer == DistroLayer::Reference
        && record.target_plane == DistroPlane::Ops
        && record.target_package.as_deref() == Some("arcrtc-reference-ops")
        && record.target_scope == "reference-distro/ops"
        && record.command_class == DistroCommandClass::Build
        && is_cargo_build_command(&record.command)
    {
        return Ok(());
    }
    Err(ReferenceRuntimeError::CommandScopeMismatch)
}

fn is_cargo_build_command(command: &str) -> bool {
    command.trim() == "cargo build --workspace --all-targets"
}
