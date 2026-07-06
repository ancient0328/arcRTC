//! production auth provider admission の境界です。

use arcrtc_distro_evidence::DistroEvidenceReason;

use crate::error::ProductPolicyError;

/// production auth provider class の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductAuthProviderClass {
    /// provider 未採用です。
    NotAdmitted,
    /// controlled production identity provider です。
    ControlledProductionIdentity,
}

/// production auth provider admission の証跡入力です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductAuthProviderAdmission {
    /// provider class です。
    pub provider_class: ProductAuthProviderClass,
    /// credential source の境界です。
    pub credential_source: &'static str,
    /// validation rule の境界です。
    pub token_validation_rule: &'static str,
    /// secret storage の境界です。
    pub secret_storage_boundary: &'static str,
    /// failure reason closed set の境界です。
    pub failure_reason_closed_set: &'static [&'static str],
    /// audit evidence の境界です。
    pub audit_evidence_boundary: &'static str,
    /// distro reason です。
    pub distro_reason: DistroEvidenceReason,
}

/// product auth provider を production readiness 入力として採用します。
pub fn admit_product_auth_provider(
    provider_class: ProductAuthProviderClass,
) -> Result<ProductAuthProviderAdmission, ProductPolicyError> {
    match provider_class {
        ProductAuthProviderClass::ControlledProductionIdentity => {
            Ok(ProductAuthProviderAdmission {
                provider_class,
                credential_source: "controlled-production-identity-source",
                token_validation_rule: "closed-validation-rule",
                secret_storage_boundary: "no-raw-secret-material-in-repository",
                failure_reason_closed_set: &[
                    "DISTRO_OK",
                    "FIXTURE_IDENTITY_INVALID",
                    "READINESS_NOT_ADMITTED",
                ],
                audit_evidence_boundary: "production-auth-admission-evidence-only",
                distro_reason: DistroEvidenceReason::DistroOk,
            })
        }
        ProductAuthProviderClass::NotAdmitted => Err(ProductPolicyError::ReadinessNotAdmitted),
    }
}
