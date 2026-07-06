use arcrtc_core_security::AuditRedactionBoundaryRef;

/// audit event が redaction boundary を消費したことを示す source model です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuditRedactionBoundaryConsumption {
    /// core/security owned redaction boundary の opaque reference です。
    pub boundary_ref: AuditRedactionBoundaryRef,
    /// redaction boundary を消費した audit event reference です。
    pub audit_event_ref: AuditEventId,
    /// redaction 後 payload の canonical digest です。
    pub redacted_payload_digest: CanonicalEventPayloadDigest,
}

impl AuditRedactionBoundaryConsumption {
    /// redaction boundary consumption record を作ります。
    pub fn new(
        boundary_ref: AuditRedactionBoundaryRef,
        audit_event_ref: AuditEventId,
        redacted_payload_digest: CanonicalEventPayloadDigest,
    ) -> Self {
        Self {
            boundary_ref,
            audit_event_ref,
            redacted_payload_digest,
        }
    }
}

/// audit redaction boundary consumption の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditRedactionBoundaryConsumptionDecision {
    /// boundary consumption を audit event に接続できます。
    Accepted,
    /// boundary consumption を拒否します。
    Rejected,
}

/// security-owned redaction boundary を opaque ref として audit event へ接続します。
///
/// core/audit は redaction policy semantics や secret classification を所有しません。
pub fn consume_audit_redaction_boundary(
    input: AuditRedactionBoundaryConsumption,
) -> AuditRedactionBoundaryConsumptionDecision {
    let boundary_ref_present = !input.boundary_ref.value.is_empty();
    let audit_event_ref_present = !input.audit_event_ref.as_str().is_empty();
    let redacted_digest_present = !input.redacted_payload_digest.digest().is_empty();

    if boundary_ref_present && audit_event_ref_present && redacted_digest_present {
        AuditRedactionBoundaryConsumptionDecision::Accepted
    } else {
        AuditRedactionBoundaryConsumptionDecision::Rejected
    }
}
