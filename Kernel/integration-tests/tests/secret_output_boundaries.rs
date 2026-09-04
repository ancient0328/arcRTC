use arcrtc_core_audit::{
    consume_audit_redaction_boundary, AuditRedactionBoundaryConsumption,
    AuditRedactionBoundaryConsumptionDecision, CanonicalEventPayloadDigest, HashAlgorithm,
};
use arcrtc_core_identity::{AuditEventId, OpaqueReference, ReferenceAuthority};
use arcrtc_core_security::{
    classify_secret_output_surface, AuditRedactionBoundaryRef, SecretOutputSurfaceClass,
    SecretOutputSurfaceInput, SecretRedactionDecision, SensitiveMaterialClass,
};

fn accepted_ref(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy)
        .expect("secret-output boundary test references must be accepted")
}

fn boundary_ref() -> AuditRedactionBoundaryRef {
    AuditRedactionBoundaryRef::new("redaction-boundary-ref")
}

#[test]
fn all_secret_output_surfaces_are_redacted_through_closed_boundary() {
    let surfaces = [
        (
            SecretOutputSurfaceClass::Credential,
            Some(SensitiveMaterialClass::CredentialSecret),
        ),
        (
            SecretOutputSurfaceClass::KeyMaterial,
            Some(SensitiveMaterialClass::CryptoKey),
        ),
        (
            SecretOutputSurfaceClass::Token,
            Some(SensitiveMaterialClass::BearerToken),
        ),
        (
            SecretOutputSurfaceClass::MediaPayload,
            Some(SensitiveMaterialClass::MediaBytes),
        ),
        (SecretOutputSurfaceClass::SessionRef, None),
        (SecretOutputSurfaceClass::AuditPayload, None),
        (SecretOutputSurfaceClass::LogLine, None),
        (SecretOutputSurfaceClass::MetricLabel, None),
        (SecretOutputSurfaceClass::Artifact, None),
    ];

    for (surface_class, material_class) in surfaces {
        // secret 本体ではなく、surface class と opaque redaction boundary だけを検証します。
        let decision = classify_secret_output_surface(SecretOutputSurfaceInput::new(
            surface_class,
            material_class,
            Some(boundary_ref()),
        ));
        assert_eq!(decision, SecretRedactionDecision::Redacted);
    }
}

#[test]
fn secret_output_surfaces_reject_missing_or_mismatched_redaction_boundary() {
    let rejected_cases = [
        SecretOutputSurfaceInput::new(
            SecretOutputSurfaceClass::Credential,
            Some(SensitiveMaterialClass::CredentialSecret),
            None,
        ),
        SecretOutputSurfaceInput::new(
            SecretOutputSurfaceClass::Credential,
            Some(SensitiveMaterialClass::CryptoKey),
            Some(boundary_ref()),
        ),
        SecretOutputSurfaceInput::new(
            SecretOutputSurfaceClass::KeyMaterial,
            Some(SensitiveMaterialClass::CredentialSecret),
            Some(boundary_ref()),
        ),
        SecretOutputSurfaceInput::new(
            SecretOutputSurfaceClass::Token,
            Some(SensitiveMaterialClass::MediaBytes),
            Some(boundary_ref()),
        ),
        SecretOutputSurfaceInput::new(
            SecretOutputSurfaceClass::LogLine,
            Some(SensitiveMaterialClass::BearerToken),
            Some(boundary_ref()),
        ),
        SecretOutputSurfaceInput::new(
            SecretOutputSurfaceClass::Artifact,
            Some(SensitiveMaterialClass::CryptoKey),
            Some(boundary_ref()),
        ),
    ];

    for input in rejected_cases {
        assert_eq!(
            classify_secret_output_surface(input),
            SecretRedactionDecision::Rejected
        );
    }
}

#[test]
fn audit_consumes_redaction_boundary_without_owning_secret_classification() {
    let audit_event_ref = AuditEventId::new(accepted_ref("audit-event-ref"));
    let redacted_payload_digest =
        CanonicalEventPayloadDigest::new(HashAlgorithm::Sha256, vec![0xA7, 0xD1])
            .expect("redacted payload digest must be non-empty");
    let consumption = AuditRedactionBoundaryConsumption::new(
        boundary_ref(),
        audit_event_ref,
        redacted_payload_digest,
    );

    assert_eq!(
        consume_audit_redaction_boundary(consumption),
        AuditRedactionBoundaryConsumptionDecision::Accepted
    );
}

#[test]
fn audit_rejects_absent_redaction_boundary_reference() {
    let audit_event_ref = AuditEventId::new(accepted_ref("audit-event-ref"));
    let redacted_payload_digest =
        CanonicalEventPayloadDigest::new(HashAlgorithm::Sha256, vec![0xA7, 0xD1])
            .expect("redacted payload digest must be non-empty");
    let consumption = AuditRedactionBoundaryConsumption::new(
        AuditRedactionBoundaryRef::new(""),
        audit_event_ref,
        redacted_payload_digest,
    );

    assert_eq!(
        consume_audit_redaction_boundary(consumption),
        AuditRedactionBoundaryConsumptionDecision::Rejected
    );
}
