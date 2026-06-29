//! product policy の public export 境界です。

pub mod auth_policy;
pub mod error;
pub mod provider_admission;
pub mod security_reason;

pub use auth_policy::{
    evaluate_product_auth_policy, ProductAction, ProductPolicyDecision, ProductPolicyInput,
};
pub use error::ProductPolicyError;
pub use provider_admission::{
    admit_product_auth_provider, ProductAuthProviderAdmission, ProductAuthProviderClass,
};
pub use security_reason::map_product_security_reason;
