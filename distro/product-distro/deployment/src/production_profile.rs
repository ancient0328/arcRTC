//! production runtime profile admission の境界です。

use arcrtc_distro_evidence::DistroEnvironmentClass;

use crate::profile::{ProductHostClass, ProductRuntimeProfile};

/// production readiness 入力用の product runtime profile を構築します。
pub const fn build_product_production_profile() -> ProductRuntimeProfile {
    ProductRuntimeProfile {
        profile_name: "product-production-admitted",
        host_class: ProductHostClass::ProductionAdmitted,
        environment_class: DistroEnvironmentClass::ProductionDeferred,
        public_endpoint_claimed: false,
    }
}
