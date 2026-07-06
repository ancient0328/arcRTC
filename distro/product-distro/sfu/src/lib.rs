//! product SFU の public export 境界です。

pub mod error;
pub mod kernel_contract;

pub use error::ProductSfuError;
pub use kernel_contract::{
    apply_product_sfu_policy, build_live_product_sfu_runtime, build_product_sfu_runtime,
    ProductSfuOutcome, ProductSfuPolicyInput, ProductSfuRuntime,
};
