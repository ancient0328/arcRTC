use arcrtc_regulated::{
    build_regulated_export_pointer, RegulatedAuditPointerIntegrity, RegulatedDependencyGuard,
    RegulatedDependencySource, RegulatedDependencyTarget, RegulatedExportPointerInput,
    RegulatedOpaqueReference, RegulatedRedactionProjectionRef, RegulatedRedactionRequest,
};

#[test]
fn regulated_boundary_uses_only_opaque_reference_and_redaction_projection_refs() {
    let reference = RegulatedOpaqueReference::new("regulated-ref", "regulated-scope");
    let projection = RegulatedRedactionProjectionRef::new("core-security-redaction-projection");
    let request = RegulatedRedactionRequest::new(reference.reference_id, projection);

    // regulated は core/security の redaction decision を opaque projection として参照するだけです。
    assert_eq!(reference.reference_id, "regulated-ref");
    assert_eq!(reference.scope_ref, "regulated-scope");
    assert_eq!(request.reference_id, "regulated-ref");
    assert_eq!(
        request.redaction_boundary_ref.value,
        "core-security-redaction-projection"
    );
}

#[test]
fn regulated_export_pointer_accepts_non_empty_refs_and_preserves_audit_pointer_integrity() {
    let export_pointer = build_regulated_export_pointer(RegulatedExportPointerInput::new(
        "regulated-export-ref",
        "audit-pointer-ref",
    ))
    .expect("regulated export pointer with opaque refs must be accepted");

    assert_eq!(export_pointer.export_ref, "regulated-export-ref");
    assert_eq!(export_pointer.audit_pointer_ref, "audit-pointer-ref");
    assert_eq!(
        export_pointer.audit_pointer_integrity(),
        RegulatedAuditPointerIntegrity::Accepted
    );
}

#[test]
fn regulated_export_pointer_rejects_absent_export_or_audit_pointer_refs() {
    assert_eq!(
        build_regulated_export_pointer(RegulatedExportPointerInput::new("", "audit-pointer-ref")),
        Err(RegulatedAuditPointerIntegrity::Rejected)
    );
    assert_eq!(
        build_regulated_export_pointer(RegulatedExportPointerInput::new(
            "regulated-export-ref",
            ""
        )),
        Err(RegulatedAuditPointerIntegrity::Rejected)
    );
}

#[test]
fn regulated_dependency_guard_admits_only_core_opaque_identity_references() {
    assert!(RegulatedDependencyGuard::admits(
        RegulatedDependencySource::Regulated,
        RegulatedDependencyTarget::CoreOpaqueIdentityReferences,
    ));
    assert!(!RegulatedDependencyGuard::admits(
        RegulatedDependencySource::Regulated,
        RegulatedDependencyTarget::CoreProtocolSemantics,
    ));
    assert!(!RegulatedDependencyGuard::admits(
        RegulatedDependencySource::Regulated,
        RegulatedDependencyTarget::Drivers,
    ));
}
