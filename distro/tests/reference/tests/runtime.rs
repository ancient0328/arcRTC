//! reference runtime lifecycle 境界を検査します。

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_distro_evidence::{
    DistroEvidenceReason, DistroNonClaimScope, DistroPlane,
};
use arcrtc_reference_composition::ReferenceCompositionState;
use arcrtc_reference_ops::{
    build_reference_build_evidence_record, DistroRuntimeState, DistroShutdownMode,
    ReferenceBuildEvidenceInput, ReferenceRuntime, ReferenceRuntimeError,
};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

#[test]
fn kpi_reference_runtime_executes_controlled_lifecycle() {
    let mut runtime = ReferenceRuntime::new(ReferenceCompositionState::default());

    // KPI-T3-5 は runtime lifecycle success と invalid order rejection の closed reason を同時に固定する。
    let started = runtime
        .start(cid("kpi-runtime-start"))
        .expect("runtime start must succeed");
    assert_eq!(started.state, DistroRuntimeState::Running);
    assert_eq!(
        started.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert!(started
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
    assert!(started
        .non_claim_scope
        .contains(&DistroNonClaimScope::LiveReadinessNotClaimed));

    let stopped = runtime
        .shutdown(
            cid("kpi-runtime-shutdown"),
            DistroShutdownMode::DrainThenStop,
        )
        .expect("runtime shutdown must succeed");
    assert_eq!(stopped.state, DistroRuntimeState::Stopped);
    assert_eq!(runtime.state(), DistroRuntimeState::Stopped);

    let mut invalid = ReferenceRuntime::new(ReferenceCompositionState::default());
    let rejected = invalid
        .shutdown(
            cid("kpi-runtime-invalid-shutdown"),
            DistroShutdownMode::Immediate,
        )
        .expect_err("shutdown before start must fail closed");
    assert_eq!(rejected, ReferenceRuntimeError::RuntimeExecutorError);
    assert_eq!(
        rejected.distro_reason(),
        DistroEvidenceReason::RuntimeExecutorError
    );
    assert_eq!(invalid.state(), DistroRuntimeState::Failed);
}

#[test]
fn kpi_reference_runtime_records_single_correlation_for_controlled_flow() {
    let correlation_id = cid("kpi-controlled-runtime-flow");
    let mut runtime = ReferenceRuntime::new(ReferenceCompositionState::default());

    let started = runtime
        .start(correlation_id.clone())
        .expect("runtime start must succeed");
    assert_eq!(started.correlation_id, correlation_id);
    assert_eq!(started.state, DistroRuntimeState::Running);

    let bounded_command = build_reference_build_evidence_record(ReferenceBuildEvidenceInput {
        correlation_id: correlation_id.clone(),
        command: "cargo build --workspace --all-targets".to_owned(),
        target_package: "arcrtc-reference-ops".to_owned(),
        target_scope: "reference-distro/ops".to_owned(),
        toolchain_runtime_version: "rustc-test".to_owned(),
        expected_outcome: "build succeeds".to_owned(),
        actual_outcome: "build succeeds".to_owned(),
        exit_status: 0,
    })
    .expect("bounded reference build command evidence must be accepted");
    assert_eq!(bounded_command.correlation_id, correlation_id.as_str());
    assert_eq!(
        bounded_command.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let stopped = runtime
        .shutdown(
            correlation_id.clone(),
            DistroShutdownMode::DrainThenStop,
        )
        .expect("runtime shutdown must succeed");
    assert_eq!(stopped.correlation_id, correlation_id);
    assert_eq!(stopped.state, DistroRuntimeState::Stopped);

    let mut invalid = ReferenceRuntime::new(ReferenceCompositionState::default());
    assert_eq!(
        invalid.shutdown(correlation_id, DistroShutdownMode::Immediate),
        Err(ReferenceRuntimeError::RuntimeExecutorError)
    );
    assert_eq!(invalid.state(), DistroRuntimeState::Failed);
}

#[test]
fn reference_runtime_start_shutdown_return_runtime_outcome_not_evidence_claim() {
    let mut runtime = ReferenceRuntime::new(ReferenceCompositionState::default());

    let started = runtime
        .start(cid("runtime-start"))
        .expect("start must succeed");
    assert_eq!(started.state, DistroRuntimeState::Running);
    assert_eq!(
        started.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert!(started
        .non_claim_scope
        .contains(&DistroNonClaimScope::BehaviorCorrectnessNotClaimed));
    assert!(started
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
    assert!(started
        .non_claim_scope
        .contains(&DistroNonClaimScope::LiveReadinessNotClaimed));

    let stopped = runtime
        .shutdown(
            cid("runtime-stop"),
            DistroShutdownMode::GracefulLocal,
        )
        .expect("shutdown must succeed");
    assert_eq!(stopped.state, DistroRuntimeState::Stopped);
    assert_eq!(runtime.state(), DistroRuntimeState::Stopped);
}

#[test]
fn reference_runtime_fail_closes_invalid_lifecycle_order() {
    let mut runtime = ReferenceRuntime::new(ReferenceCompositionState::default());

    assert_eq!(
        runtime.shutdown(
            cid("runtime-stop-before-start"),
            DistroShutdownMode::Immediate
        ),
        Err(ReferenceRuntimeError::RuntimeExecutorError)
    );
    assert_eq!(runtime.state(), DistroRuntimeState::Failed);
}

#[test]
fn reference_build_evidence_accepts_only_reference_ops_build_scope() {
    let input = ReferenceBuildEvidenceInput {
        correlation_id: cid("runtime-build-evidence"),
        command: "cargo build --workspace --all-targets".to_owned(),
        target_package: "arcrtc-reference-ops".to_owned(),
        target_scope: "reference-distro/ops".to_owned(),
        toolchain_runtime_version: "rustc-test".to_owned(),
        expected_outcome: "build succeeds".to_owned(),
        actual_outcome: "build succeeds".to_owned(),
        exit_status: 0,
    };
    let record = build_reference_build_evidence_record(input).expect("record must be accepted");
    assert_eq!(record.target_plane, DistroPlane::Ops);
    assert!(record
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
}
