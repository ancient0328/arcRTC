//! Five-item coverage closure tests for product behavior proof surfaces.

use arcrtc_core_identity::{
    CorrelationId, OpaqueReference, ParticipantId, ReferenceAuthority, RoomId,
};
use arcrtc_core_sfu::SfuDecisionKind;
use arcrtc_core_signaling::SignalingEventKind;
use arcrtc_core_turn::TurnDecisionKind;
use arcrtc_implementation_evidence::{
    ImplementationEnvironmentClass, ImplementationEvidenceReason, ImplementationNonClaimScope,
    ImplementationPlane,
};
use arcrtc_product_deployment::{
    build_product_runtime_profile, runtime::ImplementationRuntimeState, select_product_runtime,
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

fn policy(reason: ImplementationEvidenceReason) -> ProductPolicyDecision {
    ProductPolicyDecision {
        allowed: reason == ImplementationEvidenceReason::ImplementationOk,
        implementation_reason: reason,
        non_claim_scope: vec![
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
    }
}

fn empty_scope_policy(reason: ImplementationEvidenceReason) -> ProductPolicyDecision {
    ProductPolicyDecision {
        allowed: true,
        implementation_reason: reason,
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
            ImplementationEvidenceReason::ImplementationOk,
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
            ImplementationEvidenceReason::ImplementationOk,
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
            ImplementationEvidenceReason::ImplementationOk,
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
    assert_eq!(started.state, ImplementationRuntimeState::Running);

    let drain = plan_drain(
        cid("coverage-runtime-drain"),
        vec![ImplementationPlane::Signaling, ImplementationPlane::Turn],
        ProductDrainMode::ControlledProduct,
    );
    let drained = runtime.drain(drain).expect("running runtime must drain");
    assert_eq!(drained.state, ImplementationRuntimeState::Stopped);
    assert_eq!(
        drained.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );

    let restore = plan_restore(
        cid("coverage-runtime-restore"),
        ProductRestoreSource::EvidenceReport,
    );
    let restored = runtime
        .restore(restore)
        .expect("stopped runtime must restore");
    assert_eq!(restored.state, ImplementationRuntimeState::Running);
    assert!(restored
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed));
}

#[test]
fn product_runtime_lifecycle_fails_closed_for_invalid_state_and_readiness() {
    let mut created_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    let drain_before_start = plan_drain(
        cid("coverage-drain-before-start"),
        vec![ImplementationPlane::Sfu],
        ProductDrainMode::ControlledProduct,
    );
    assert_eq!(
        created_runtime.drain(drain_before_start),
        Err(ProductRuntimeError::RuntimeExecutorError)
    );
    assert_eq!(created_runtime.state(), ImplementationRuntimeState::Failed);

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
    assert_eq!(running_runtime.state(), ImplementationRuntimeState::Failed);

    let mut readiness_runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));
    readiness_runtime
        .start(cid("coverage-readiness-runtime"))
        .expect("local runtime must start");
    let deferred_drain = plan_drain(
        cid("coverage-deferred-drain"),
        vec![ImplementationPlane::Signaling],
        ProductDrainMode::ProductionDeferred,
    );
    assert_eq!(
        readiness_runtime.drain(deferred_drain),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
    assert_eq!(
        readiness_runtime.state(),
        ImplementationRuntimeState::Failed
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
    assert_eq!(failed_runtime.state(), ImplementationRuntimeState::Failed);

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
    assert_eq!(already_running.state(), ImplementationRuntimeState::Failed);
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
        ImplementationRuntimeState::Running
    );

    let mismatched_public = ProductRuntimeProfile {
        profile_name: "coverage-public-production-mismatch",
        host_class: ProductHostClass::ProductionAdmitted,
        environment_class: ImplementationEnvironmentClass::ProductionDeferred,
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
            ImplementationEnvironmentClass::LocalSingleHost,
            false,
            ImplementationEvidenceReason::ImplementationOk,
        ),
        (
            ProductHostClass::ControlledMultiProcess,
            "product-controlled-multi-process",
            ImplementationEnvironmentClass::ControlledProcess,
            false,
            ImplementationEvidenceReason::ImplementationOk,
        ),
        (
            ProductHostClass::ProductionDeferred,
            "product-production-deferred",
            ImplementationEnvironmentClass::ProductionDeferred,
            false,
            ImplementationEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::ProductionAdmitted,
            "product-production-admitted",
            ImplementationEnvironmentClass::ProductionDeferred,
            false,
            ImplementationEvidenceReason::ImplementationOk,
        ),
        (
            ProductHostClass::LiveDeferred,
            "product-live-deferred",
            ImplementationEnvironmentClass::LiveDeferred,
            false,
            ImplementationEvidenceReason::ReadinessNotAdmitted,
        ),
        (
            ProductHostClass::LiveAdmitted,
            "product-live-admitted",
            ImplementationEnvironmentClass::LiveDeferred,
            true,
            ImplementationEvidenceReason::ImplementationOk,
        ),
    ] {
        let profile = build_product_runtime_profile(host_class);
        assert_eq!(profile.profile_name, profile_name);
        assert_eq!(profile.environment_class, environment_class);
        assert_eq!(profile.public_endpoint_claimed, public_endpoint_claimed);
        assert_eq!(
            select_product_runtime(&profile).implementation_reason,
            reason
        );
    }
}

#[test]
fn product_plane_policy_rejects_empty_scope_readiness_and_correlation_mismatch() {
    let correlation = cid("coverage-signaling-policy");
    let mut signaling = signaling_input(
        correlation.clone(),
        policy(ImplementationEvidenceReason::ImplementationOk),
    );
    signaling.reference_outcome = ReferenceSignalingOutcome::new(
        cid("coverage-other-signaling-policy"),
        SignalingEventKind::Joined,
        RoomId::new(accepted("coverage-room")),
        None,
        ImplementationEvidenceReason::ImplementationOk,
    );
    assert_eq!(
        apply_product_signaling_policy(&signaling),
        Err(ProductSignalingError::StateBoundaryViolation)
    );

    assert_eq!(
        apply_product_signaling_policy(&signaling_input(
            cid("coverage-empty-signaling-scope"),
            empty_scope_policy(ImplementationEvidenceReason::ImplementationOk),
        )),
        Err(ProductSignalingError::StateBoundaryViolation)
    );
    assert_eq!(
        apply_product_signaling_policy(&signaling_input(
            cid("coverage-readiness-signaling"),
            policy(ImplementationEvidenceReason::ReadinessNotAdmitted),
        )),
        Err(ProductSignalingError::ReadinessNotAdmitted)
    );

    assert_eq!(
        apply_product_turn_policy(&turn_input(
            cid("coverage-empty-turn-scope"),
            empty_scope_policy(ImplementationEvidenceReason::ImplementationOk),
        )),
        Err(ProductTurnError::StateBoundaryViolation)
    );
    assert_eq!(
        apply_product_turn_policy(&turn_input(
            cid("coverage-readiness-turn"),
            policy(ImplementationEvidenceReason::ReadinessNotAdmitted),
        )),
        Err(ProductTurnError::ReadinessNotAdmitted)
    );

    assert_eq!(
        apply_product_sfu_policy(&sfu_input(
            cid("coverage-empty-sfu-scope"),
            empty_scope_policy(ImplementationEvidenceReason::ImplementationOk),
        )),
        Err(ProductSfuError::StateBoundaryViolation)
    );
    assert_eq!(
        apply_product_sfu_policy(&sfu_input(
            cid("coverage-readiness-sfu"),
            policy(ImplementationEvidenceReason::ReadinessNotAdmitted),
        )),
        Err(ProductSfuError::ReadinessNotAdmitted)
    );
}

#[test]
fn product_plane_runtime_builders_cover_default_and_live_admitted_descriptors() {
    let signaling = build_product_signaling_runtime();
    assert_eq!(signaling.target_plane, ImplementationPlane::Signaling);
    assert!(!signaling.public_endpoint_claimed);
    assert!(signaling.live_endpoint_evidence_ref.is_none());

    let turn = build_product_turn_runtime();
    assert_eq!(turn.target_plane, ImplementationPlane::Turn);
    assert!(!turn.relay_public_endpoint_claimed);
    assert!(turn.live_endpoint_evidence_ref.is_none());

    let sfu = build_product_sfu_runtime();
    assert_eq!(sfu.target_plane, ImplementationPlane::Sfu);
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
