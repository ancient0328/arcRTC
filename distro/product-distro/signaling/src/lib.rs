//! product Signaling の public export 境界です。

pub mod error;
pub mod kernel_contract;

pub use error::ProductSignalingError;
pub use kernel_contract::{
    apply_product_signaling_policy, build_live_product_signaling_runtime,
    build_product_signaling_runtime, ProductSignalingOutcome, ProductSignalingPolicyInput,
    ProductSignalingRuntime,
};
