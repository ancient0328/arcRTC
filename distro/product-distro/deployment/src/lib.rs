//! product deployment の public export 境界です。

pub mod error;
pub mod live_endpoint;
#[rustfmt::skip]
pub mod profile;
#[rustfmt::skip]
pub mod production_profile;
pub mod runtime;

pub use error::ProductRuntimeError;
pub use live_endpoint::{
    admit_product_live_endpoint, admit_public_traversal, build_product_live_profile,
    ProductLiveEndpointAdmission, ProductLiveEndpointClass, ProductPublicTraversalAdmission,
    ProductPublicTraversalClass,
};
#[rustfmt::skip]
pub use profile::{build_product_runtime_profile, ProductHostClass, ProductRuntimeProfile};
#[rustfmt::skip]
pub use production_profile::build_product_production_profile;
pub use runtime::{
    select_product_runtime, ProductRuntime, ProductRuntimeOutcome, ProductRuntimeSelection,
    ProductRuntimeSocketProbe,
};
