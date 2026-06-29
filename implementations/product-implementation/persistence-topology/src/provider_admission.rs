//! production persistence provider admission の境界です。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

use crate::error::ProductPersistenceTopologyError;

/// production persistence provider class の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductPersistenceProviderClass {
    /// provider 未採用です。
    NotAdmitted,
    /// controlled projection store です。
    ControlledProjectionStore,
}

/// production persistence provider admission の証跡入力です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductPersistenceProviderAdmission {
    /// provider class です。
    pub provider_class: ProductPersistenceProviderClass,
    /// record shape の境界です。
    pub record_shape_boundary: &'static str,
    /// storage scope の境界です。
    pub storage_scope: &'static str,
    /// schema owner の境界です。
    pub schema_owner: &'static str,
    /// failure reason closed set の境界です。
    pub failure_reason_closed_set: &'static [&'static str],
    /// implementation reason です。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product persistence provider を production readiness 入力として採用します。
pub fn admit_product_persistence_provider(
    provider_class: ProductPersistenceProviderClass,
) -> Result<ProductPersistenceProviderAdmission, ProductPersistenceTopologyError> {
    match provider_class {
        ProductPersistenceProviderClass::ControlledProjectionStore => {
            Ok(ProductPersistenceProviderAdmission {
                provider_class,
                record_shape_boundary: "session-allocation-route-evidence-projection",
                storage_scope: "controlled-production-projection-store",
                schema_owner: "product-persistence-topology",
                failure_reason_closed_set: &[
                    "IMPLEMENTATION_OK",
                    "STATE_BOUNDARY_VIOLATION",
                    "READINESS_NOT_ADMITTED",
                ],
                implementation_reason: ImplementationEvidenceReason::ImplementationOk,
            })
        }
        ProductPersistenceProviderClass::NotAdmitted => {
            Err(ProductPersistenceTopologyError::ProviderNotAdmitted)
        }
    }
}
