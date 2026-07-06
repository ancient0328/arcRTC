/// secret が外へ出得る output surface の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretOutputSurfaceClass {
    /// credential reference or credential-derived surface.
    Credential,
    /// key material reference or key-derived surface.
    KeyMaterial,
    /// bearer token surface.
    Token,
    /// media payload surface.
    MediaPayload,
    /// session reference surface.
    SessionRef,
    /// audit payload surface.
    AuditPayload,
    /// log line surface.
    LogLine,
    /// metric label surface.
    MetricLabel,
    /// artifact output surface.
    Artifact,
}

/// redaction 対象の sensitive material class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensitiveMaterialClass {
    /// credential secret.
    CredentialSecret,
    /// crypto key.
    CryptoKey,
    /// bearer token.
    BearerToken,
    /// media bytes.
    MediaBytes,
}

/// audit が消費できる redaction boundary の opaque reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuditRedactionBoundaryRef {
    /// redaction boundary を指す opaque value です。secret 本体ではありません。
    pub value: &'static str,
}

impl AuditRedactionBoundaryRef {
    /// redaction boundary reference を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

/// secret output surface classification input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecretOutputSurfaceInput {
    /// output surface class です。
    pub surface_class: SecretOutputSurfaceClass,
    /// sensitive material class です。raw material は保持しません。
    pub sensitive_material_class: Option<SensitiveMaterialClass>,
    /// audit redaction boundary reference です。
    pub redaction_boundary_ref: Option<AuditRedactionBoundaryRef>,
}

impl SecretOutputSurfaceInput {
    /// output surface と opaque redaction material だけを束ねます。
    pub const fn new(
        surface_class: SecretOutputSurfaceClass,
        sensitive_material_class: Option<SensitiveMaterialClass>,
        redaction_boundary_ref: Option<AuditRedactionBoundaryRef>,
    ) -> Self {
        Self {
            surface_class,
            sensitive_material_class,
            redaction_boundary_ref,
        }
    }
}

/// secret redaction classification の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretRedactionDecision {
    /// redaction boundary が適用されました。
    Redacted,
    /// redaction boundary が不十分なため出力を拒否します。
    Rejected,
}

/// output surface と sensitive material class を redaction decision に写像します。
pub fn classify_secret_output_surface(
    input: SecretOutputSurfaceInput,
) -> SecretRedactionDecision {
    let boundary_is_present = input
        .redaction_boundary_ref
        .map(|boundary_ref| !boundary_ref.value.is_empty())
        .unwrap_or(false);

    let material_matches_surface = match (input.surface_class, input.sensitive_material_class) {
        (SecretOutputSurfaceClass::Credential, Some(SensitiveMaterialClass::CredentialSecret)) => {
            true
        }
        (SecretOutputSurfaceClass::KeyMaterial, Some(SensitiveMaterialClass::CryptoKey)) => true,
        (SecretOutputSurfaceClass::Token, Some(SensitiveMaterialClass::BearerToken)) => true,
        (SecretOutputSurfaceClass::MediaPayload, Some(SensitiveMaterialClass::MediaBytes)) => true,
        (SecretOutputSurfaceClass::SessionRef, None) => true,
        (SecretOutputSurfaceClass::AuditPayload, None) => true,
        (SecretOutputSurfaceClass::LogLine, None) => true,
        (SecretOutputSurfaceClass::MetricLabel, None) => true,
        (SecretOutputSurfaceClass::Artifact, None) => true,
        (SecretOutputSurfaceClass::Credential, None) => false,
        (SecretOutputSurfaceClass::Credential, Some(SensitiveMaterialClass::CryptoKey)) => false,
        (SecretOutputSurfaceClass::Credential, Some(SensitiveMaterialClass::BearerToken)) => false,
        (SecretOutputSurfaceClass::Credential, Some(SensitiveMaterialClass::MediaBytes)) => false,
        (SecretOutputSurfaceClass::KeyMaterial, None) => false,
        (SecretOutputSurfaceClass::KeyMaterial, Some(SensitiveMaterialClass::CredentialSecret)) => {
            false
        }
        (SecretOutputSurfaceClass::KeyMaterial, Some(SensitiveMaterialClass::BearerToken)) => false,
        (SecretOutputSurfaceClass::KeyMaterial, Some(SensitiveMaterialClass::MediaBytes)) => false,
        (SecretOutputSurfaceClass::Token, None) => false,
        (SecretOutputSurfaceClass::Token, Some(SensitiveMaterialClass::CredentialSecret)) => false,
        (SecretOutputSurfaceClass::Token, Some(SensitiveMaterialClass::CryptoKey)) => false,
        (SecretOutputSurfaceClass::Token, Some(SensitiveMaterialClass::MediaBytes)) => false,
        (SecretOutputSurfaceClass::MediaPayload, None) => false,
        (SecretOutputSurfaceClass::MediaPayload, Some(SensitiveMaterialClass::CredentialSecret)) => {
            false
        }
        (SecretOutputSurfaceClass::MediaPayload, Some(SensitiveMaterialClass::CryptoKey)) => false,
        (SecretOutputSurfaceClass::MediaPayload, Some(SensitiveMaterialClass::BearerToken)) => false,
        (SecretOutputSurfaceClass::SessionRef, Some(SensitiveMaterialClass::CredentialSecret)) => {
            false
        }
        (SecretOutputSurfaceClass::SessionRef, Some(SensitiveMaterialClass::CryptoKey)) => false,
        (SecretOutputSurfaceClass::SessionRef, Some(SensitiveMaterialClass::BearerToken)) => false,
        (SecretOutputSurfaceClass::SessionRef, Some(SensitiveMaterialClass::MediaBytes)) => false,
        (SecretOutputSurfaceClass::AuditPayload, Some(SensitiveMaterialClass::CredentialSecret)) => {
            false
        }
        (SecretOutputSurfaceClass::AuditPayload, Some(SensitiveMaterialClass::CryptoKey)) => false,
        (SecretOutputSurfaceClass::AuditPayload, Some(SensitiveMaterialClass::BearerToken)) => false,
        (SecretOutputSurfaceClass::AuditPayload, Some(SensitiveMaterialClass::MediaBytes)) => false,
        (SecretOutputSurfaceClass::LogLine, Some(SensitiveMaterialClass::CredentialSecret)) => false,
        (SecretOutputSurfaceClass::LogLine, Some(SensitiveMaterialClass::CryptoKey)) => false,
        (SecretOutputSurfaceClass::LogLine, Some(SensitiveMaterialClass::BearerToken)) => false,
        (SecretOutputSurfaceClass::LogLine, Some(SensitiveMaterialClass::MediaBytes)) => false,
        (SecretOutputSurfaceClass::MetricLabel, Some(SensitiveMaterialClass::CredentialSecret)) => {
            false
        }
        (SecretOutputSurfaceClass::MetricLabel, Some(SensitiveMaterialClass::CryptoKey)) => false,
        (SecretOutputSurfaceClass::MetricLabel, Some(SensitiveMaterialClass::BearerToken)) => false,
        (SecretOutputSurfaceClass::MetricLabel, Some(SensitiveMaterialClass::MediaBytes)) => false,
        (SecretOutputSurfaceClass::Artifact, Some(SensitiveMaterialClass::CredentialSecret)) => false,
        (SecretOutputSurfaceClass::Artifact, Some(SensitiveMaterialClass::CryptoKey)) => false,
        (SecretOutputSurfaceClass::Artifact, Some(SensitiveMaterialClass::BearerToken)) => false,
        (SecretOutputSurfaceClass::Artifact, Some(SensitiveMaterialClass::MediaBytes)) => false,
    };

    if boundary_is_present && material_matches_surface {
        SecretRedactionDecision::Redacted
    } else {
        SecretRedactionDecision::Rejected
    }
}
