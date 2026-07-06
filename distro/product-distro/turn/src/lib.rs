//! product TURN の public export 境界です。

pub mod error;
pub mod kernel_contract;

pub use error::ProductTurnError;
pub use kernel_contract::{
    apply_product_turn_policy, build_live_product_turn_runtime, build_product_turn_runtime,
    ProductTurnOutcome, ProductTurnPolicyInput, ProductTurnRuntime,
};
