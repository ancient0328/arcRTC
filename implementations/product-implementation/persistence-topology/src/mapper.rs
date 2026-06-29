//! product projection mapper の境界です。

use arcrtc_implementation_evidence::{ImplementationEvidenceReason, ImplementationPlane};

use crate::{error::ProductPersistenceTopologyError, topology::ProductPersistenceRecordClass};

/// product projection mappingです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductProjectionMapping {
    /// source planeです。
    pub source_plane: ImplementationPlane,
    /// record classです。
    pub record_class: ProductPersistenceRecordClass,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product projection mappingを作成します。
pub fn map_product_projection(
    source_plane: ImplementationPlane,
    record_class: ProductPersistenceRecordClass,
) -> Result<ProductProjectionMapping, ProductPersistenceTopologyError> {
    let matched = matches!(
        (source_plane, record_class),
        (
            ImplementationPlane::Signaling,
            ProductPersistenceRecordClass::SessionProjection
        ) | (
            ImplementationPlane::Turn,
            ProductPersistenceRecordClass::AllocationProjection
        ) | (
            ImplementationPlane::Sfu,
            ProductPersistenceRecordClass::RouteProjection
        ) | (
            ImplementationPlane::Monitoring,
            ProductPersistenceRecordClass::EvidenceProjection
        )
    );
    if !matched {
        return Err(ProductPersistenceTopologyError::ProjectionMappingViolation);
    }
    Ok(ProductProjectionMapping {
        source_plane,
        record_class,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
    })
}
