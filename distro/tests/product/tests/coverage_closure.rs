//! Five-item coverage closure tests for product behavior proof surfaces.

use arcrtc_core_identity::{
    CorrelationId, OpaqueReference, ParticipantId, ReferenceAuthority, RoomId,
};
use arcrtc_core_sfu::SfuDecisionKind;
use arcrtc_core_signaling::SignalingEventKind;
use arcrtc_core_turn::TurnDecisionKind;
use arcrtc_distro_evidence::{
    DistroEnvironmentClass, DistroEvidenceReason, DistroNonClaimScope,
    DistroPlane,
};
use arcrtc_product_deployment::{
    build_product_runtime_profile, runtime::DistroRuntimeState, select_product_runtime,
    ProductHostClass, ProductRuntime, ProductRuntimeError, ProductRuntimeProfile,
};
use arcrtc_product_policy::ProductPolicyDecision;
use arcrtc_product_rollback::{plan_drain, plan_restore, ProductDrainMode, ProductRestoreSource};
use arcrtc_product_sfu::{
    apply_product_sfu_policy, build_live_product_sfu_runtime, build_product_sfu_runtime,
    ProductSfuError, ProductSfuPolicyInput,
};
use arcrtc_product_signaling::{
    apply_product_signaling_policy, build_live_product_signaling_runtime,
    build_product_signaling_runtime, ProductSignalingError, ProductSignalingPolicyInput,
};
use arcrtc_product_turn::{
    apply_product_turn_policy, build_live_product_turn_runtime, build_product_turn_runtime,
    ProductTurnError, ProductTurnPolicyInput,
};
use arcrtc_reference_output::{
    ReferenceSfuOutcome, ReferenceSignalingOutcome, ReferenceTurnOutcome,
};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

fn policy(reason: DistroEvidenceReason) -> ProductPolicyDecision {
    ProductPolicyDecision {
        allowed: reason == DistroEvidenceReason::DistroOk,
        distro_reason: reason,
        non_claim_scope: vec![
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
    }
}

fn empty_scope_policy(reason: DistroEvidenceReason) -> ProductPolicyDecision {
    ProductPolicyDecision {
        allowed: true,
        distro_reason: reason,
        non_claim_scope: Vec::new(),
    }
}

fn signaling_input(
    correlation_id: CorrelationId,
    policy_decision: ProductPolicyDecision,
) -> ProductSignalingPolicyInput {
    ProductSignalingPolicyInput {
        correlation_id: correlation_id.clone(),
        reference_outcome: ReferenceSignalingOutcome::new(
            correlation_id,
            SignalingEventKind::Joined,
            RoomId::new(accepted("coverage-room")),
            Some(ParticipantId::new(accepted("coverage-participant"))),
            DistroEvidenceReason::DistroOk,
        ),
        policy_decision,
    }
}

fn turn_input(
    correlation_id: CorrelationId,
    policy_decision: ProductPolicyDecision,
) -> ProductTurnPolicyInput {
    ProductTurnPolicyInput {
        correlation_id,
        reference_outcome: ReferenceTurnOutcome::new(
            TurnDecisionKind::ChannelBind,
            DistroEvidenceReason::DistroOk,
        ),
        policy_decision,
    }
}

fn sfu_input(
    correlation_id: CorrelationId,
    policy_decision: ProductPolicyDecision,
) -> ProductSfuPolicyInput {
    ProductSfuPolicyInput {
        correlation_id,
        reference_outcome: ReferenceSfuOutcome::new(
            SfuDecisionKind::Forwarding,
            DistroEvidenceReason::DistroOk,
        ),
        policy_decision,
    }
}

#[test]
fn product_runtime_drain_and_restore_cover_success_lifecycle() {
    let profile = build_product_runtime_profile(ProductHostClass::LocalSingleHost);
    let mut runtime = ProductRuntime::new(profile);

    let started = runtime
        .start(cid("coverage-runtime-start"))
        .expect("local runtime must start");
    assert_eq!(started.state, DistroRuntimeState::Running);

    let drain = plan_drain(
        cid("coverage-runtime-drain"),
        vec![DistroPlane::Signaling, DistroPlane::Turn],
        ProductDrainMode::ControlledProduct,
    );
    let drained = runtime.drain(drain).expect("running runtime must drain");
    assert_eq!(drained.state, DistroRuntimeState::Stopped);
    assert_eq!(
        drained.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let restore = plan_restore(
        cid("coverage-runtime-restore"),
        ProductRestoreSource::EvidenceReport,
    );
    let restored = runtime
        .restore(restore)
        .expect("stopped runtime must restore");
    assert_eq!(restored.state, DistroRuntimeState::Running);
    assert!(restored
        .non_claim_scope
        .contains(&DistroNonClaimScope::BehaviorCorrectnessNotClaimed));
}

#[test]
fn product_runtime_lifecycle_fails_closed_for_invalid_state_and_readiness() {
    let mut created_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    let drain_before_start = plan_drain(
        cid("coverage-drain-before-start"),
        vec![DistroPlane::Sfu],
        ProductDrainMode::ControlledProduct,
    );
    assert_eq!(
        created_runtime.drain(drain_before_start),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );
    assert_eq!(created_runtime.state(), DistroRuntimeState::Failed);

    let mut running_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    running_runtime
        .start(cid("coverage-running-runtime"))
        .expect("local runtime must start");
    assert_eq!(
        running_runtime.restore(plan_restore(
            cid("coverage-restore-while-running"),
            ProductRestoreSource::EvidenceReport,
        )),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );
    assert_eq!(running_runtime.state(), DistroRuntimeState::Failed);

    let mut readiness_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    readiness_runtime
        .start(cid("coverage-readiness-runtime"))
        .expect("local runtime must start");
    let deferred_drain = plan_drain(
        cid("coverage-deferred-drain"),
        vec![DistroPlane::Signaling],
        ProductDrainMode::ProductionDeferred,
    );
    assert_eq!(
        readiness_runtime.drain(deferred_drain),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
    assert_eq!(
        readiness_runtime.state(),
        DistroRuntimeState::Failed
    );

    let mut failed_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    assert_eq!(
        failed_runtime.restore(plan_restore(
            cid("coverage-provider-restore"),
            ProductRestoreSource::ProviderDeferred,
        )),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
    assert_eq!(failed_runtime.state(), DistroRuntimeState::Failed);

    let mut already_running = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    already_running
        .start(cid("coverage-start-once"))
        .expect("first start must succeed");
    assert_eq!(
        already_running.start(cid("coverage-start-twice")),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );
    assert_eq!(already_running.state(), DistroRuntimeState::Failed);
}

#[test]
fn product_runtime_selection_covers_admitted_live_public_endpoint_and_mismatch() {
    let live = build_product_runtime_profile(ProductHostClass::LiveAdmitted);
    assert!(live.public_endpoint_claimed);
    let mut runtime = ProductRuntime::new(live);
    assert_eq!(
        runtime
            .start(cid("coverage-live-admitted-start"))
            .expect("live admitted profile must start")
            .state,
        DistroRuntimeState::Running
    );

    let mismatched_public = ProductRuntimeProfile {
        profile_name: "coverage-public-production-mismatch",
        host_class: ProductHostClass::ProductionAdmitted,
        environment_class: DistroEnvironmentClass::ProductionDeferred,
        public_endpoint_claimed: true,
    };
    let mut runtime = ProductRuntime::new(mismatched_public);
    assert_eq!(
        runtime.start(cid("coverage-public-production-mismatch")),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
}

#[test]
fn product_runtime_profiles_cover_all_host_classes_and_selection_reasons() {
    for (host_class, profile_name, environment_class, public_endpoint_claimed, reason) in [
        (
            ProductHostClass::LocalSingleHost,
            "product-local-single-host",
            DistroEnvironmentClass::LocalSingleHost,
            false,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::ControlledMultiProcess,
            "product-controlled-multi-process",
            DistroEnvironmentClass::ControlledProcess,
            false,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::ProductionDeferred,
            "product-production-deferred",
            DistroEnvironmentClass::ProductionDeferred,
            false,
            DistroEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::ProductionAdmitted,
            "product-production-admitted",
            DistroEnvironmentClass::ProductionDeferred,
            false,
            DistroEvidenceReason::DistroOk,
        ),
        (
            ProductHostClass::LiveDeferred,
            "product-live-deferred",
            DistroEnvironmentClass::LiveDeferred,
            false,
            DistroEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::LiveAdmitted,
            "product-live-admitted",
            DistroEnvironmentClass::LiveDeferred,
            true,
            DistroEvidenceReason::DistroOk,
        ),
    ] {
        let profile = build_product_runtime_profile(host_class);
        assert_eq!(profile.profile_name, profile_name);
        assert_eq!(profile.environment_class, environment_class);
        assert_eq!(profile.public_endpoint_claimed, public_endpoint_claimed);
        assert_eq!(
            select_product_runtime(&profile).distro_reason,
            reason
        );
    }
}

#[test]
fn product_plane_policy_rejects_empty_scope_readiness_and_correlation_mismatch() {
    let correlation = cid("coverage-signaling-policy");
    let mut signaling = signaling_input(
        correlation.clone(),
        policy(DistroEvidenceReason::DistroOk),
    );
    signaling.reference_outcome = ReferenceSignalingOutcome::new(
        cid("coverage-other-signaling-policy"),
        SignalingEventKind::Joined,
        RoomId::new(accepted("coverage-room")),
        None,
        DistroEvidenceReason::DistroOk,
    );
    assert_eq!(
        apply_product_signaling_policy(&signaling),
        Err(ProductSignalingError::StateBoundaryViolation)
    );

    assert_eq!(
        apply_product_signaling_policy(&signaling_input(
            cid("coverage-empty-signaling-scope"),
            empty_scope_policy(DistroEvidenceReason::DistroOk),
        )),
        Err(ProductSignalingError::StateBoundaryViolation)
    );
    assert_eq!(
        apply_product_signaling_policy(&signaling_input(
            cid("coverage-readiness-signaling"),
            policy(DistroEvidenceReason::ReadinessNotAdmitted),
        )),
        Err(ProductSignalingError::ReadinessNotAdmitted)
    );

    assert_eq!(
        apply_product_turn_policy(&turn_input(
            cid("coverage-empty-turn-scope"),
            empty_scope_policy(DistroEvidenceReason::DistroOk),
        )),
        Err(ProductTurnError::StateBoundaryViolation)
    );
    assert_eq!(
        apply_product_turn_policy(&turn_input(
            cid("coverage-readiness-turn"),
            policy(DistroEvidenceReason::ReadinessNotAdmitted),
        )),
        Err(ProductTurnError::ReadinessNotAdmitted)
    );

    assert_eq!(
        apply_product_sfu_policy(&sfu_input(
            cid("coverage-empty-sfu-scope"),
            empty_scope_policy(DistroEvidenceReason::DistroOk),
        )),
        Err(ProductSfuError::StateBoundaryViolation)
    );
    assert_eq!(
        apply_product_sfu_policy(&sfu_input(
            cid("coverage-readiness-sfu"),
            policy(DistroEvidenceReason::ReadinessNotAdmitted),
        )),
        Err(ProductSfuError::ReadinessNotAdmitted)
    );
}

#[test]
fn product_plane_runtime_builders_cover_default_and_live_admitted_descriptors() {
    let signaling = build_product_signaling_runtime();
    assert_eq!(signaling.target_plane, DistroPlane::Signaling);
    assert!(!signaling.public_endpoint_claimed);
    assert!(signaling.live_endpoint_evidence_ref.is_none());

    let turn = build_product_turn_runtime();
    assert_eq!(turn.target_plane, DistroPlane::Turn);
    assert!(!turn.relay_public_endpoint_claimed);
    assert!(turn.live_endpoint_evidence_ref.is_none());

    let sfu = build_product_sfu_runtime();
    assert_eq!(sfu.target_plane, DistroPlane::Sfu);
    assert!(!sfu.media_public_endpoint_claimed);
    assert!(sfu.live_endpoint_evidence_ref.is_none());

    let endpoint_ref = "LIVE_ENDPOINT_EVIDENCE";
    assert_eq!(
        build_live_product_signaling_runtime(endpoint_ref)
            .expect("live signaling runtime")
            .live_endpoint_evidence_ref
            .as_deref(),
        Some(endpoint_ref)
    );
    assert_eq!(
        build_live_product_turn_runtime(endpoint_ref)
            .expect("live turn runtime")
            .live_endpoint_evidence_ref
            .as_deref(),
        Some(endpoint_ref)
    );
    assert_eq!(
        build_live_product_sfu_runtime(endpoint_ref)
            .expect("live sfu runtime")
            .live_endpoint_evidence_ref
            .as_deref(),
        Some(endpoint_ref)
    );

    assert_eq!(
        build_live_product_signaling_runtime(" "),
        Err(ProductSignalingError::ReadinessNotAdmitted)
    );
    assert_eq!(
        build_live_product_turn_runtime(" "),
        Err(ProductTurnError::ReadinessNotAdmitted)
    );
    assert_eq!(
        build_live_product_sfu_runtime(" "),
        Err(ProductSfuError::ReadinessNotAdmitted)
    );
}
