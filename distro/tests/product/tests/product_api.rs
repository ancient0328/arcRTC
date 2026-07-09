//! product API surface と reference output input 境界を検査します。

use std::{fs, path::PathBuf};

use arcrtc_core_identity::{
    CorrelationId, OpaqueReference, ParticipantId, ReferenceAuthority, RoomId,
};
use arcrtc_core_sfu::SfuDecisionKind;
use arcrtc_core_signaling::SignalingEventKind;
use arcrtc_core_turn::TurnDecisionKind;
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroNonClaimScope};
use arcrtc_product_policy::ProductPolicyDecision;
use arcrtc_product_sfu::{apply_product_sfu_policy, ProductSfuPolicyInput};
use arcrtc_product_signaling::{apply_product_signaling_policy, ProductSignalingPolicyInput};
use arcrtc_product_turn::{apply_product_turn_policy, ProductTurnPolicyInput};
use arcrtc_reference_output::{
    ReferenceSfuOutcome, ReferenceSignalingOutcome, ReferenceTurnOutcome,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

fn product_policy_decision() -> ProductPolicyDecision {
    ProductPolicyDecision {
        allowed: true,
        distro_reason: DistroEvidenceReason::DistroOk,
        // product 実行はreadiness claimへ昇格しないことを入力側でも固定します。
        non_claim_scope: vec![
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
    }
}

#[test]
fn product_plane_accepts_only_three_reference_output_inputs() {
    for (path, allowed) in [
        (
            "product-distro/signaling/src/kernel_contract.rs",
            "ReferenceSignalingOutcome",
        ),
        (
            "product-distro/turn/src/kernel_contract.rs",
            "ReferenceTurnOutcome",
        ),
        (
            "product-distro/sfu/src/kernel_contract.rs",
            "ReferenceSfuOutcome",
        ),
    ] {
        let source =
            fs::read_to_string(root().join(path)).expect("product source must be readable");
        assert!(source.contains(allowed), "{path} missing {allowed}");
        assert!(!source.contains("ReferenceCompositionOutcome"));
        assert!(!source.contains("arcrtc_reference_output::*"));
        assert!(!source.contains("pub use arcrtc_reference_output"));
    }
}

#[test]
fn product_sources_do_not_consume_reference_internal_surfaces() {
    let mut scanned = 0usize;
    for path in [
        "product-distro/signaling",
        "product-distro/turn",
        "product-distro/sfu",
        "product-distro/product-policy",
        "product-distro/persistence-topology",
        "product-distro/deployment",
        "product-distro/monitoring",
        "product-distro/rollback",
    ] {
        for source_path in rust_and_manifest_files(root().join(path)) {
            scanned += 1;
            let relative = source_path
                .strip_prefix(root())
                .expect("product source path must be under distro root")
                .to_string_lossy()
                .into_owned();
            let source = fs::read_to_string(&source_path).expect("product source must be readable");
            for forbidden in [
                "arcrtc_reference_signaling",
                "arcrtc_reference_turn",
                "arcrtc_reference_sfu",
                "arcrtc_reference_composition",
                "arcrtc_reference_ops",
                "arcrtc-core-signaling",
                "arcrtc-core-turn",
                "arcrtc-core-sfu",
                "arcrtc-core-command",
                "arcrtc-core-transport",
                "arcrtc-core-security",
                "arcrtc-core-quality",
                "arcrtc-core-state",
                "arcrtc-core-reason",
                "ReferenceSignalingState",
                "ReferenceTurnState",
                "ReferenceSfuState",
                "ReferenceComposition",
                "ReferenceSignalingCommandInput",
                "ReferenceTurnCommandInput",
                "ReferenceSfuAction",
                "ReferenceRuntime",
                "FixtureSessionDescription",
                "FixtureIceCandidate",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{relative} must not consume reference internal surface {forbidden}"
                );
            }
        }
    }
    println!("product reference-internal source scan files={scanned}");
}

#[test]
fn kpi_product_api_executes_allowed_reference_output_inputs() {
    let signaling_correlation = cid("sig-correlation");
    let signaling_result = apply_product_signaling_policy(&ProductSignalingPolicyInput {
        correlation_id: signaling_correlation.clone(),
        reference_outcome: ReferenceSignalingOutcome::new(
            signaling_correlation.clone(),
            SignalingEventKind::Joined,
            RoomId::new(accepted("sig-room")),
            Some(ParticipantId::new(accepted("sig-participant"))),
            DistroEvidenceReason::DistroOk,
        ),
        policy_decision: product_policy_decision(),
    })
    .expect("allowed reference Signaling outcome must execute product policy");
    assert!(signaling_result.allowed);
    assert_eq!(signaling_result.correlation_id, signaling_correlation);
    assert_eq!(
        signaling_result.distro_reason,
        DistroEvidenceReason::DistroOk
    );
    assert!(signaling_result
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
    assert!(signaling_result
        .non_claim_scope
        .contains(&DistroNonClaimScope::LiveReadinessNotClaimed));

    let turn_correlation = cid("product-turn-correlation");
    let turn_result = apply_product_turn_policy(&ProductTurnPolicyInput {
        correlation_id: turn_correlation.clone(),
        reference_outcome: ReferenceTurnOutcome::new(
            TurnDecisionKind::ChannelBind,
            DistroEvidenceReason::DistroOk,
        ),
        policy_decision: product_policy_decision(),
    })
    .expect("allowed reference TURN outcome must execute product policy");
    assert!(turn_result.allowed);
    assert_eq!(turn_result.correlation_id, turn_correlation);
    assert!(turn_result
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
    assert!(turn_result
        .non_claim_scope
        .contains(&DistroNonClaimScope::LiveReadinessNotClaimed));

    let sfu_correlation = cid("product-sfu-correlation");
    let sfu_result = apply_product_sfu_policy(&ProductSfuPolicyInput {
        correlation_id: sfu_correlation.clone(),
        reference_outcome: ReferenceSfuOutcome::new(
            SfuDecisionKind::Forwarding,
            DistroEvidenceReason::DistroOk,
        ),
        policy_decision: product_policy_decision(),
    })
    .expect("allowed reference SFU outcome must execute product policy");
    assert!(sfu_result.allowed);
    assert_eq!(sfu_result.correlation_id, sfu_correlation);
    assert_eq!(sfu_result.distro_reason, DistroEvidenceReason::DistroOk);
    assert!(sfu_result
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
    assert!(sfu_result
        .non_claim_scope
        .contains(&DistroNonClaimScope::LiveReadinessNotClaimed));
}

fn rust_and_manifest_files(path: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(path, &mut files);
    files
}

fn collect_files(path: PathBuf, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "rs")
            || path.file_name().is_some_and(|name| name == "Cargo.toml")
        {
            files.push(path);
        }
        return;
    }
    if path.ends_with("target") {
        return;
    }
    for entry in fs::read_dir(path).expect("directory must be readable") {
        collect_files(
            entry.expect("directory entry must be readable").path(),
            files,
        );
    }
}

#[test]
fn product_plane_public_api_exports_exact_policy_functions() {
    for (path, function_name) in [
        (
            "product-distro/signaling/src/lib.rs",
            "apply_product_signaling_policy",
        ),
        (
            "product-distro/turn/src/lib.rs",
            "apply_product_turn_policy",
        ),
        ("product-distro/sfu/src/lib.rs", "apply_product_sfu_policy"),
    ] {
        let source = fs::read_to_string(root().join(path)).expect("product lib must be readable");
        assert!(
            source.contains(function_name),
            "{path} missing {function_name}"
        );
    }
}
