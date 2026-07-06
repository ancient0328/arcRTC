//! product persistence topology model の境界です。

use crate::error::ProductPersistenceTopologyError;

/// product persistence modeです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductPersistenceMode {
    /// persistence未採用です。
    NotAdmitted,
    /// in-memory projectionのみです。
    InMemoryProjectionOnly,
    /// provider採用が別ADR待ちであることを表すmodeです。
    ProviderDeferred,
}

/// product persistence record classです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductPersistenceRecordClass {
    /// session projectionです。
    SessionProjection,
    /// allocation projectionです。
    AllocationProjection,
    /// route projectionです。
    RouteProjection,
    /// evidence projectionです。
    EvidenceProjection,
}

/// product persistence topologyです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductPersistenceTopology {
    /// persistence modeです。
    pub mode: ProductPersistenceMode,
    /// topologyが扱うrecord classです。
    pub record_classes: Vec<ProductPersistenceRecordClass>,
}

/// product persistence topologyを構築します。
pub fn build_persistence_topology(
    mode: ProductPersistenceMode,
) -> Result<ProductPersistenceTopology, ProductPersistenceTopologyError> {
    match mode {
        ProductPersistenceMode::InMemoryProjectionOnly => Ok(ProductPersistenceTopology {
            mode,
            record_classes: vec![
                ProductPersistenceRecordClass::SessionProjection,
                ProductPersistenceRecordClass::AllocationProjection,
                ProductPersistenceRecordClass::RouteProjection,
                ProductPersistenceRecordClass::EvidenceProjection,
            ],
        }),
        ProductPersistenceMode::NotAdmitted | ProductPersistenceMode::ProviderDeferred => {
            Err(ProductPersistenceTopologyError::ProviderNotAdmitted)
        }
    }
}
