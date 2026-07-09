//! live endpoint / traversal admission の境界です。

use arcrtc_distro_evidence::DistroEvidenceReason;

use crate::{
    error::ProductRuntimeError,
    profile::{ProductHostClass, ProductRuntimeProfile},
};

/// live endpoint class の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductLiveEndpointClass {
    /// live endpoint 未採用です。
    NotAdmitted,
    /// controlled public endpoint です。
    ControlledPublicEndpoint,
}

/// public traversal class の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductPublicTraversalClass {
    /// public traversal 未採用です。
    NotAdmitted,
    /// controlled public traversal です。
    ControlledPublicTraversal,
}

/// live endpoint admission の証跡入力です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductLiveEndpointAdmission {
    /// endpoint class です。
    pub endpoint_class: ProductLiveEndpointClass,
    /// endpoint boundary です。
    pub endpoint_boundary: &'static str,
    /// public endpoint claim を許可する根拠境界です。
    pub live_endpoint_evidence_boundary: &'static str,
    /// distro reason です。
    pub distro_reason: DistroEvidenceReason,
}

/// public traversal admission の証跡入力です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductPublicTraversalAdmission {
    /// traversal class です。
    pub traversal_class: ProductPublicTraversalClass,
    /// traversal boundary です。
    pub traversal_boundary: &'static str,
    /// redaction boundary です。
    pub redaction_boundary: &'static str,
    /// distro reason です。
    pub distro_reason: DistroEvidenceReason,
}

/// product live endpoint を live readiness 入力として採用します。
pub fn admit_product_live_endpoint(
    endpoint_class: ProductLiveEndpointClass,
) -> Result<ProductLiveEndpointAdmission, ProductRuntimeError> {
    match endpoint_class {
        ProductLiveEndpointClass::ControlledPublicEndpoint => Ok(ProductLiveEndpointAdmission {
            endpoint_class,
            endpoint_boundary: "controlled-live-public-endpoint",
            live_endpoint_evidence_boundary: "live-endpoint-evidence-ref-required",
            distro_reason: DistroEvidenceReason::DistroOk,
        }),
        ProductLiveEndpointClass::NotAdmitted => Err(ProductRuntimeError::ReadinessNotAdmitted),
    }
}

/// product public traversal を live readiness 入力として採用します。
pub fn admit_public_traversal(
    traversal_class: ProductPublicTraversalClass,
) -> Result<ProductPublicTraversalAdmission, ProductRuntimeError> {
    match traversal_class {
        ProductPublicTraversalClass::ControlledPublicTraversal => {
            Ok(ProductPublicTraversalAdmission {
                traversal_class,
                traversal_boundary: "controlled-public-traversal",
                redaction_boundary: "raw-device-and-network-identifiers-not-retained",
                distro_reason: DistroEvidenceReason::DistroOk,
            })
        }
        ProductPublicTraversalClass::NotAdmitted => Err(ProductRuntimeError::ReadinessNotAdmitted),
    }
}

/// admitted live endpoint から product live profile を構築します。
pub fn build_product_live_profile(
    admission: &ProductLiveEndpointAdmission,
) -> Result<ProductRuntimeProfile, ProductRuntimeError> {
    if admission.distro_reason != DistroEvidenceReason::DistroOk {
        return Err(ProductRuntimeError::ReadinessNotAdmitted);
    }
    Ok(ProductRuntimeProfile {
        profile_name: "product-live-admitted",
        host_class: ProductHostClass::LiveAdmitted,
        environment_class: arcrtc_distro_evidence::DistroEnvironmentClass::LiveDeferred,
        public_endpoint_claimed: true,
    })
}
