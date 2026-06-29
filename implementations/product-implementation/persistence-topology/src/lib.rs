//! product persistence topology の public export 境界です。

pub mod error;
pub mod mapper;
pub mod provider_admission;
pub mod topology;

pub use error::ProductPersistenceTopologyError;
pub use mapper::{map_product_projection, ProductProjectionMapping};
pub use provider_admission::{
    admit_product_persistence_provider, ProductPersistenceProviderAdmission,
    ProductPersistenceProviderClass,
};
pub use topology::{
    build_persistence_topology, ProductPersistenceMode, ProductPersistenceRecordClass,
    ProductPersistenceTopology,
};
