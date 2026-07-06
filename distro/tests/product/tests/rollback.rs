//! product rollback / drain surface 境界を検査します。

use std::{fs, path::PathBuf};

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroPlane};
use arcrtc_product_rollback::{plan_drain, plan_restore, ProductDrainMode, ProductRestoreSource};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn rollback_and_restore_plans_do_not_claim_live_readiness() {
    let drain = plan_drain(
        cid("rollback-drain"),
        vec![DistroPlane::Signaling, DistroPlane::Sfu],
        ProductDrainMode::ReferenceLocal,
    );
    assert_eq!(
        drain.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let empty_drain = plan_drain(
        cid("rollback-empty-drain"),
        Vec::new(),
        ProductDrainMode::ReferenceLocal,
    );
    assert_eq!(
        empty_drain.distro_reason,
        DistroEvidenceReason::RuntimeExecutorError
    );

    let production_drain = plan_drain(
        cid("rollback-production-drain"),
        vec![DistroPlane::Signaling],
        ProductDrainMode::ProductionDeferred,
    );
    assert_eq!(
        production_drain.distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );

    let restore = plan_restore(
        cid("rollback-restore"),
        ProductRestoreSource::InMemoryProjection,
    );
    assert_eq!(
        restore.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let provider_restore = plan_restore(
        cid("rollback-provider-restore"),
        ProductRestoreSource::ProviderDeferred,
    );
    assert_eq!(
        provider_restore.distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );
}

#[test]
fn kpi_product_rollback_executes_operation_plan_without_live_claim() {
    for path in [
        "product-distro/rollback/src/drain.rs",
        "product-distro/rollback/src/restore.rs",
    ] {
        let source =
            fs::read_to_string(root().join(path)).expect("rollback source must be readable");
        for forbidden in [
            "PublicEndpoint",
            "LiveEndpoint",
            "ProviderConnection",
            "DynamoDB",
            "Postgres",
            "Redis",
            "Kubernetes",
            "LoadBalancer",
        ] {
            assert!(
                !source.contains(forbidden),
                "rollback plan must not own provider/live operation: {path} {forbidden}"
            );
        }
    }

    let controlled_drain = plan_drain(
        cid("rollback-controlled-drain"),
        vec![
            DistroPlane::Signaling,
            DistroPlane::Turn,
            DistroPlane::Sfu,
        ],
        ProductDrainMode::ControlledProduct,
    );
    assert_eq!(
        controlled_drain.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let production_deferred = plan_drain(
        cid("rollback-production-deferred"),
        vec![DistroPlane::Signaling],
        ProductDrainMode::ProductionDeferred,
    );
    assert_eq!(
        production_deferred.distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );

    let evidence_restore = plan_restore(
        cid("rollback-evidence-restore"),
        ProductRestoreSource::EvidenceReport,
    );
    assert_eq!(
        evidence_restore.distro_reason,
        DistroEvidenceReason::DistroOk
    );

    let provider_restore = plan_restore(
        cid("rollback-provider-deferred"),
        ProductRestoreSource::ProviderDeferred,
    );
    assert_eq!(
        provider_restore.distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );
}

#[test]
fn kpi_product_drain_restore_reject_live_readiness_claim() {
    let production_drain = plan_drain(
        cid("kpi-product-live-drain-rejected"),
        vec![DistroPlane::Signaling, DistroPlane::Sfu],
        ProductDrainMode::ProductionDeferred,
    );
    assert_eq!(
        production_drain.distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );

    let provider_restore = plan_restore(
        cid("kpi-product-live-restore-rejected"),
        ProductRestoreSource::ProviderDeferred,
    );
    assert_eq!(
        provider_restore.distro_reason,
        DistroEvidenceReason::ReadinessNotAdmitted
    );
}
