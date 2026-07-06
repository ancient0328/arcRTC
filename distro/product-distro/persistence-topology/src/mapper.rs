//! product projection mapper の境界です。

use arcrtc_distro_evidence::{DistroEvidenceReason, DistroPlane};

use crate::{error::ProductPersistenceTopologyError, topology::ProductPersistenceRecordClass};

/// product projection mappingです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductProjectionMapping {
    /// source planeです。
    pub source_plane: DistroPlane,
    /// record classです。
    pub record_class: ProductPersistenceRecordClass,
    /// distro reasonです。
    pub distro_reason: DistroEvidenceReason,
}

/// product projection mappingを作成します。
pub fn map_product_projection(
    source_plane: DistroPlane,
    record_class: ProductPersistenceRecordClass,
) -> Result<ProductProjectionMapping, ProductPersistenceTopologyError> {
    let matched = matches!(
        (source_plane, record_class),
        (
            DistroPlane::Signaling,
            ProductPersistenceRecordClass::SessionProjection
        ) | (
            DistroPlane::Turn,
            ProductPersistenceRecordClass::AllocationProjection
        ) | (
            DistroPlane::Sfu,
            ProductPersistenceRecordClass::RouteProjection
        ) | (
            DistroPlane::Monitoring,
            ProductPersistenceRecordClass::EvidenceProjection
        )
    );
    if !matched {
        return Err(ProductPersistenceTopologyError::ProjectionMappingViolation);
    }
    Ok(ProductProjectionMapping {
        source_plane,
        record_class,
        distro_reason: DistroEvidenceReason::DistroOk,
    })
}
