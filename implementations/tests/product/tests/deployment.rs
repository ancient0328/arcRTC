//! product deployment runtime profile と lifecycle 境界を直接検査します。

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_implementation_evidence::{
    ImplementationEnvironmentClass, ImplementationEvidenceReason, ImplementationNonClaimScope,
};
use arcrtc_product_deployment::{
    build_product_runtime_profile, select_product_runtime, ProductHostClass, ProductRuntime,
    ProductRuntimeError, ProductRuntimeProfile,
};

fn correlation_id(value: &str) -> CorrelationId {
    CorrelationId::new(
        OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
            .expect("correlation id must be accepted"),
    )
}

#[test]
fn deployment_runtime_starts_local_profile_without_readiness_claim() {
    let profile = build_product_runtime_profile(ProductHostClass::LocalSingleHost);
    let selection = select_product_runtime(&profile);
    assert_eq!(
        selection.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );

    let mut runtime = ProductRuntime::new(profile);
    let outcome = runtime
        .start(correlation_id("product-runtime-start"))
        .expect("local runtime must start");
    assert_eq!(
        outcome.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );
    assert!(outcome
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::ProductionReadinessNotClaimed));
    assert!(outcome
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::LiveReadinessNotClaimed));
}

#[test]
fn deployment_runtime_rejects_public_endpoint_claim_without_readiness_admission() {
    let profile = ProductRuntimeProfile {
        profile_name: "product-local-single-host-public-claim",
        host_class: ProductHostClass::LocalSingleHost,
        environment_class: ImplementationEnvironmentClass::LocalSingleHost,
        public_endpoint_claimed: true,
    };
    let selection = select_product_runtime(&profile);
    assert_eq!(
        selection.implementation_reason,
        ImplementationEvidenceReason::ReadinessNotAdmitted
    );

    let mut runtime = ProductRuntime::new(profile);
    assert_eq!(
        runtime.start(correlation_id("product-runtime-public-claim")),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
}

#[test]
fn kpi_product_runtime_rejects_public_endpoint_readiness_claim() {
    let profile = ProductRuntimeProfile {
        profile_name: "kpi-product-public-endpoint-rejected",
        host_class: ProductHostClass::ControlledMultiProcess,
        environment_class: ImplementationEnvironmentClass::ControlledProcess,
        public_endpoint_claimed: true,
    };
    let selection = select_product_runtime(&profile);
    assert_eq!(
        selection.implementation_reason,
        ImplementationEvidenceReason::ReadinessNotAdmitted
    );

    let mut runtime = ProductRuntime::new(profile);
    assert_eq!(
        runtime.start(correlation_id("kpi-product-runtime-public-endpoint")),
        Err(ProductRuntimeError::ReadinessNotAdmitted)
    );
}

#[test]
fn kpi_product_deployment_executes_local_controlled_profile() {
    for host_class in [
        ProductHostClass::LocalSingleHost,
        ProductHostClass::ControlledMultiProcess,
    ] {
        let profile = build_product_runtime_profile(host_class);
        let selection = select_product_runtime(&profile);
        assert!(!selection.profile.public_endpoint_claimed);
        assert_eq!(
            selection.implementation_reason,
            ImplementationEvidenceReason::ImplementationOk
        );
    }

    for host_class in [
        ProductHostClass::ProductionDeferred,
        ProductHostClass::LiveDeferred,
    ] {
        let profile = build_product_runtime_profile(host_class);
        let selection = select_product_runtime(&profile);
        assert!(!selection.profile.public_endpoint_claimed);
        assert_eq!(
            selection.implementation_reason,
            ImplementationEvidenceReason::ReadinessNotAdmitted
        );
        let mut runtime = ProductRuntime::new(profile);
        assert_eq!(
            runtime.start(correlation_id("product-runtime-deferred")),
            Err(ProductRuntimeError::ReadinessNotAdmitted)
        );
    }

    let mismatched_profile = ProductRuntimeProfile {
        profile_name: "product-controlled-with-production-env",
        host_class: ProductHostClass::ControlledMultiProcess,
        environment_class: ImplementationEnvironmentClass::ProductionDeferred,
        public_endpoint_claimed: false,
    };
    assert_eq!(
        select_product_runtime(&mismatched_profile).implementation_reason,
        ImplementationEvidenceReason::ReadinessNotAdmitted
    );
}
