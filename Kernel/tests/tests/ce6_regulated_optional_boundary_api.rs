use arcrtc_core_identity::{
    AuditEventId, CorrelationId, OpaqueReference, ReferenceAuthority, RoomId,
};
use arcrtc_regulated::{
    RegulatedCommunicationReference, RegulatedCoreAuditMutation,
    RegulatedCoreDecisionParticipation, RegulatedDependencyGuard, RegulatedDependencySource,
    RegulatedDependencyTarget, RegulatedDomainPayloadRequirement, RegulatedEnrichmentFailureKind,
    RegulatedEnrichmentGuard, RegulatedEnrichmentInput, RegulatedEnrichmentLifecycleStage,
    RegulatedHashChainRecordPointer, RegulatedNonSensitiveTag, RegulatedOptionalSupportClass,
};
use arcrtc_roadmap_tests::{assert_not_contains, read_impl};

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("reference must be valid")
}

fn valid_input() -> RegulatedEnrichmentInput {
    RegulatedEnrichmentInput::new(
        RegulatedOptionalSupportClass::DomainSpecificEnrichment,
        RegulatedEnrichmentLifecycleStage::MappedOutsideGenericCore,
        RegulatedCommunicationReference::Correlation(CorrelationId::new(reference("corr-ce6"))),
        Some(AuditEventId::new(reference("audit-ce6"))),
        Some(RegulatedNonSensitiveTag::CommunicationEventReference),
        RegulatedCoreDecisionParticipation::NeverParticipates,
        RegulatedCoreAuditMutation::ImmutableReferenceOnly,
        RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
    )
}

#[test]
fn ce6_regulated_enrichment_admits_only_post_core_optional_opaque_references() {
    let record = RegulatedEnrichmentGuard::admit(valid_input())
        .expect("valid regulated enrichment must be admitted as local optional record");
    assert_eq!(
        record.record_class(),
        RegulatedOptionalSupportClass::DomainSpecificEnrichment
    );
    assert!(!record.replaces_core_audit_event());

    let hash_pointer =
        RegulatedHashChainRecordPointer::accept("hash-chain-record-ce6").expect("valid pointer");
    assert_eq!(hash_pointer.as_str(), "hash-chain-record-ce6");
    assert_eq!(
        RegulatedHashChainRecordPointer::accept(""),
        Err(arcrtc_regulated::RegulatedEnrichmentFailure::new(
            RegulatedEnrichmentFailureKind::EmptyHashChainPointer,
        ))
    );

    let room_pointer = RegulatedCommunicationReference::Room(RoomId::new(reference("room-ce6")));
    assert!(room_pointer.is_immutable_pointer_only());
}

#[test]
fn ce6_regulated_boundary_rejects_core_decision_audit_mutation_and_payload_requirement() {
    let requires_core_decision = RegulatedEnrichmentInput::new(
        RegulatedOptionalSupportClass::DomainSpecificEnrichment,
        RegulatedEnrichmentLifecycleStage::MappedOutsideGenericCore,
        RegulatedCommunicationReference::Correlation(CorrelationId::new(reference(
            "corr-ce6-decision",
        ))),
        None,
        None,
        RegulatedCoreDecisionParticipation::RequiresCoreDecision,
        RegulatedCoreAuditMutation::ImmutableReferenceOnly,
        RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
    );
    assert_eq!(
        RegulatedEnrichmentGuard::admit(requires_core_decision)
            .expect_err("regulated must not participate in core decision")
            .kind(),
        RegulatedEnrichmentFailureKind::CoreDecisionParticipationRequested
    );

    let mutates_core_audit = RegulatedEnrichmentInput::new(
        RegulatedOptionalSupportClass::AuditPointerMapping,
        RegulatedEnrichmentLifecycleStage::ImmutableAuditPointerReferenced,
        RegulatedCommunicationReference::AuditEvent(AuditEventId::new(reference(
            "audit-ce6-mutate",
        ))),
        None,
        Some(RegulatedNonSensitiveTag::AuditPointerReference),
        RegulatedCoreDecisionParticipation::NeverParticipates,
        RegulatedCoreAuditMutation::MutatesCoreAuditEvent,
        RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
    );
    assert_eq!(
        RegulatedEnrichmentGuard::admit(mutates_core_audit)
            .expect_err("regulated must not mutate core audit event")
            .kind(),
        RegulatedEnrichmentFailureKind::CoreAuditMutationRequested
    );

    let requires_domain_payload = RegulatedEnrichmentInput::new(
        RegulatedOptionalSupportClass::ExternalComplianceIntegrationHelper,
        RegulatedEnrichmentLifecycleStage::LocalRecordEmitted,
        RegulatedCommunicationReference::Correlation(CorrelationId::new(reference(
            "corr-ce6-domain",
        ))),
        None,
        Some(RegulatedNonSensitiveTag::ComplianceMappingReference),
        RegulatedCoreDecisionParticipation::NeverParticipates,
        RegulatedCoreAuditMutation::ImmutableReferenceOnly,
        RegulatedDomainPayloadRequirement::RequiredByGenericCore,
    );
    assert_eq!(
        RegulatedEnrichmentGuard::admit(requires_domain_payload)
            .expect_err("regulated payload must not become generic core requirement")
            .kind(),
        RegulatedEnrichmentFailureKind::DomainPayloadRequiredByGenericCore
    );
}

#[test]
fn ce6_regulated_dependency_direction_is_single_allowed_edge() {
    assert!(RegulatedDependencyGuard::admits(
        RegulatedDependencySource::Regulated,
        RegulatedDependencyTarget::CoreOpaqueIdentityReferences,
    ));
    for target in [
        RegulatedDependencyTarget::CoreProtocolSemantics,
        RegulatedDependencyTarget::Drivers,
        RegulatedDependencyTarget::Entrypoints,
        RegulatedDependencyTarget::Sdk,
        RegulatedDependencyTarget::Regulated,
    ] {
        assert!(
            !RegulatedDependencyGuard::admits(RegulatedDependencySource::Regulated, target),
            "regulated dependency target {target:?} must remain rejected"
        );
    }

    let manifest = read_impl("regulated/Cargo.toml");
    assert!(manifest.contains("arcrtc-core-identity"));
    assert_not_contains(
        "regulated/Cargo.toml",
        &manifest,
        &[
            "arcrtc-core-signaling",
            "arcrtc-core-sfu",
            "arcrtc-core-turn",
        ],
    );
}
