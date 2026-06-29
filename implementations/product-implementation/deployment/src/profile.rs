//! product runtime profile の境界です。

use arcrtc_implementation_evidence::ImplementationEnvironmentClass;

/// product host classです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductHostClass {
    /// local single-host runtimeです。
    LocalSingleHost,
    /// controlled multi-process runtimeです。
    ControlledMultiProcess,
    /// production provider未採用のdeferred runtimeです。
    ProductionDeferred,
    /// production provider採用済みのruntimeです。
    ProductionAdmitted,
    /// live provider未採用のdeferred runtimeです。
    LiveDeferred,
    /// live endpoint採用済みのruntimeです。
    LiveAdmitted,
}

/// product runtime profileです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRuntimeProfile {
    /// profile nameです。
    pub profile_name: &'static str,
    /// host classです。
    pub host_class: ProductHostClass,
    /// environment classです。
    pub environment_class: ImplementationEnvironmentClass,
    /// public endpoint claimです。
    pub public_endpoint_claimed: bool,
}

/// product runtime profileを構築します。
pub const fn build_product_runtime_profile(host_class: ProductHostClass) -> ProductRuntimeProfile {
    match host_class {
        ProductHostClass::LocalSingleHost => ProductRuntimeProfile {
            profile_name: "product-local-single-host",
            host_class,
            environment_class: ImplementationEnvironmentClass::LocalSingleHost,
            public_endpoint_claimed: false,
        },
        ProductHostClass::ControlledMultiProcess => ProductRuntimeProfile {
            profile_name: "product-controlled-multi-process",
            host_class,
            environment_class: ImplementationEnvironmentClass::ControlledProcess,
            public_endpoint_claimed: false,
        },
        ProductHostClass::ProductionDeferred => ProductRuntimeProfile {
            profile_name: "product-production-deferred",
            host_class,
            environment_class: ImplementationEnvironmentClass::ProductionDeferred,
            public_endpoint_claimed: false,
        },
        ProductHostClass::ProductionAdmitted => ProductRuntimeProfile {
            profile_name: "product-production-admitted",
            host_class,
            environment_class: ImplementationEnvironmentClass::ProductionDeferred,
            public_endpoint_claimed: false,
        },
        ProductHostClass::LiveDeferred => ProductRuntimeProfile {
            profile_name: "product-live-deferred",
            host_class,
            environment_class: ImplementationEnvironmentClass::LiveDeferred,
            public_endpoint_claimed: false,
        },
        ProductHostClass::LiveAdmitted => ProductRuntimeProfile {
            profile_name: "product-live-admitted",
            host_class,
            environment_class: ImplementationEnvironmentClass::LiveDeferred,
            public_endpoint_claimed: true,
        },
    }
}
