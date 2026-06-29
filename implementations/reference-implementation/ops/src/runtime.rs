//! reference runtime executorです。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{
    validate_evidence_record, ImplementationCommandClass, ImplementationEnvironmentClass,
    ImplementationEvidenceReason, ImplementationEvidenceRecord, ImplementationLayer,
    ImplementationNonClaimScope, ImplementationPlane, IMPLEMENTATIONS_COMMAND_ROOT,
};
use arcrtc_reference_composition::ReferenceCompositionState;

use crate::error::ReferenceRuntimeError;

/// runtimeが扱うimplementation planeです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImplementationRuntimePlane {
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
pub enum ImplementationRuntimeState {
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
pub enum ImplementationShutdownMode {
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
    pub state: ImplementationRuntimeState,
    /// runtime action の実装側reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
    /// runtime action が主張しない範囲です。
    pub non_claim_scope: Vec<ImplementationNonClaimScope>,
}

/// reference runtime executorです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceRuntime {
    /// runtime stateです。
    state: ImplementationRuntimeState,
    /// composition stateです。
    composition: ReferenceCompositionState,
}

impl ReferenceRuntime {
    /// reference runtimeを作ります。
    pub const fn new(composition: ReferenceCompositionState) -> Self {
        Self {
            state: ImplementationRuntimeState::Created,
            composition,
        }
    }

    /// runtimeを開始します。
    pub fn start(
        &mut self,
        correlation_id: CorrelationId,
    ) -> Result<ReferenceRuntimeOutcome, ReferenceRuntimeError> {
        if self.state != ImplementationRuntimeState::Created {
            self.state = ImplementationRuntimeState::Failed;
            return Err(ReferenceRuntimeError::RuntimeExecutorError);
        }
        self.state = ImplementationRuntimeState::Starting;
        self.state = ImplementationRuntimeState::Running;
        Ok(reference_runtime_outcome(correlation_id, self.state))
    }

    /// runtimeをshutdownします。
    pub fn shutdown(
        &mut self,
        correlation_id: CorrelationId,
        mode: ImplementationShutdownMode,
    ) -> Result<ReferenceRuntimeOutcome, ReferenceRuntimeError> {
        if self.state != ImplementationRuntimeState::Running {
            self.state = ImplementationRuntimeState::Failed;
            return Err(ReferenceRuntimeError::RuntimeExecutorError);
        }
        match mode {
            ImplementationShutdownMode::Immediate => {
                self.state = ImplementationRuntimeState::Stopped;
            }
            ImplementationShutdownMode::GracefulLocal
            | ImplementationShutdownMode::DrainThenStop => {
                self.state = ImplementationRuntimeState::Draining;
                self.state = ImplementationRuntimeState::Stopped;
            }
        }
        Ok(reference_runtime_outcome(correlation_id, self.state))
    }

    /// runtime stateを返します。
    pub const fn state(&self) -> ImplementationRuntimeState {
        self.state
    }

    /// composition stateをborrowします。
    pub const fn composition(&self) -> &ReferenceCompositionState {
        &self.composition
    }
}

fn reference_runtime_outcome(
    correlation_id: CorrelationId,
    state: ImplementationRuntimeState,
) -> ReferenceRuntimeOutcome {
    ReferenceRuntimeOutcome {
        correlation_id,
        state,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
    }
}

/// reference build command evidence recordを作ります。
pub fn build_reference_build_evidence_record(
    input: ReferenceBuildEvidenceInput,
) -> Result<ImplementationEvidenceRecord, ReferenceRuntimeError> {
    let record = ImplementationEvidenceRecord {
        correlation_id: input.correlation_id.as_str().to_owned(),
        command: input.command,
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some(input.target_package),
        target_scope: input.target_scope,
        command_class: ImplementationCommandClass::Build,
        implementation_layer: ImplementationLayer::Reference,
        target_plane: ImplementationPlane::Ops,
        environment_class: ImplementationEnvironmentClass::ControlledProcess,
        toolchain_runtime_version: input.toolchain_runtime_version,
        input_fixture_or_workload: None,
        expected_outcome: input.expected_outcome,
        actual_outcome: input.actual_outcome,
        exit_status: Some(input.exit_status),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "reference ops source or build command change".to_owned(),
    };
    validate_evidence_record(&record)
        .map_err(|_| ReferenceRuntimeError::EvidenceFieldsIncomplete)?;
    validate_reference_build_evidence_ownership(&record)?;
    Ok(record)
}

fn validate_reference_build_evidence_ownership(
    record: &ImplementationEvidenceRecord,
) -> Result<(), ReferenceRuntimeError> {
    if record.implementation_layer == ImplementationLayer::Reference
        && record.target_plane == ImplementationPlane::Ops
        && record.target_package.as_deref() == Some("arcrtc-reference-ops")
        && record.target_scope == "reference-implementation/ops"
        && record.command_class == ImplementationCommandClass::Build
        && is_cargo_build_command(&record.command)
    {
        return Ok(());
    }
    Err(ReferenceRuntimeError::CommandScopeMismatch)
}

fn is_cargo_build_command(command: &str) -> bool {
    command.trim() == "cargo build --workspace --all-targets"
}
