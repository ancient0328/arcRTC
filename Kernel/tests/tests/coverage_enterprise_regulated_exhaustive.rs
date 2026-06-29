use arcrtc_core_identity::{
    AuditEventId, CorrelationId, OpaqueReference, ParticipantId, ReferenceAuthority, RoomId,
};
use arcrtc_regulated as regulated;
use std::collections::HashSet;

fn opaque(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("opaque id is valid")
}

fn regulated_input(
    reference: regulated::RegulatedCommunicationReference,
    tag: Option<regulated::RegulatedNonSensitiveTag>,
    participation: regulated::RegulatedCoreDecisionParticipation,
    audit_mutation: regulated::RegulatedCoreAuditMutation,
    payload_requirement: regulated::RegulatedDomainPayloadRequirement,
) -> regulated::RegulatedEnrichmentInput {
    regulated::RegulatedEnrichmentInput::new(
        regulated::RegulatedOptionalSupportClass::DomainSpecificEnrichment,
        regulated::RegulatedEnrichmentLifecycleStage::LocalRecordEmitted,
        reference,
        Some(AuditEventId::new(opaque("regulated-audit-pointer"))),
        tag,
        participation,
        audit_mutation,
        payload_requirement,
    )
}

#[test]
fn regulated_closed_vocabularies_are_executed_without_core_authority() {
    let _surface = regulated::RegulatedSurface;

    for support_class in [
        regulated::RegulatedOptionalSupportClass::DomainSpecificEnrichment,
        regulated::RegulatedOptionalSupportClass::AuditPointerMapping,
        regulated::RegulatedOptionalSupportClass::NonSensitiveTagClassification,
        regulated::RegulatedOptionalSupportClass::ExternalComplianceIntegrationHelper,
    ] {
        assert!(!support_class.owns_core_decision());
    }

    for tag in [
        regulated::RegulatedNonSensitiveTag::CommunicationEventReference,
        regulated::RegulatedNonSensitiveTag::AuditPointerReference,
        regulated::RegulatedNonSensitiveTag::QualityMetricReference,
        regulated::RegulatedNonSensitiveTag::ComplianceMappingReference,
    ] {
        assert!(!tag.is_core_required());
        assert!(!tag.drives_core_behavior());
    }

    for stage in [
        regulated::RegulatedEnrichmentLifecycleStage::ReceivedAllowedCorePointer,
        regulated::RegulatedEnrichmentLifecycleStage::MappedOutsideGenericCore,
        regulated::RegulatedEnrichmentLifecycleStage::LocalRecordEmitted,
        regulated::RegulatedEnrichmentLifecycleStage::ImmutableAuditPointerReferenced,
    ] {
        assert!(stage.is_post_core_optional());
    }
}

#[test]
fn regulated_reference_and_enrichment_paths_execute_success_and_fail_closed_edges() {
    let hash_pointer =
        regulated::RegulatedHashChainRecordPointer::accept("hash-chain-exhaustive").unwrap();

    for reference in [
        regulated::RegulatedCommunicationReference::Correlation(CorrelationId::new(opaque(
            "regulated-correlation",
        ))),
        regulated::RegulatedCommunicationReference::Room(RoomId::new(opaque("regulated-room"))),
        regulated::RegulatedCommunicationReference::Participant(ParticipantId::new(opaque(
            "regulated-participant",
        ))),
        regulated::RegulatedCommunicationReference::AuditEvent(AuditEventId::new(opaque(
            "regulated-audit-event",
        ))),
        regulated::RegulatedCommunicationReference::HashChainRecord(hash_pointer.clone()),
    ] {
        assert!(reference.is_immutable_pointer_only());
        let input = regulated_input(
            reference,
            Some(regulated::RegulatedNonSensitiveTag::AuditPointerReference),
            regulated::RegulatedCoreDecisionParticipation::NeverParticipates,
            regulated::RegulatedCoreAuditMutation::ImmutableReferenceOnly,
            regulated::RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
        );
        let record = regulated::RegulatedEnrichmentGuard::admit(input).unwrap();
        assert!(!record.replaces_core_audit_event());
        assert_eq!(
            record.record_class(),
            regulated::RegulatedOptionalSupportClass::DomainSpecificEnrichment
        );
        assert_eq!(
            record.input().tag(),
            Some(regulated::RegulatedNonSensitiveTag::AuditPointerReference)
        );
        assert!(record.input().audit_pointer().is_some());
    }

    for (input, expected) in [
        (
            regulated_input(
                regulated::RegulatedCommunicationReference::HashChainRecord(hash_pointer.clone()),
                None,
                regulated::RegulatedCoreDecisionParticipation::RequiresCoreDecision,
                regulated::RegulatedCoreAuditMutation::ImmutableReferenceOnly,
                regulated::RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
            ),
            regulated::RegulatedEnrichmentFailureKind::CoreDecisionParticipationRequested,
        ),
        (
            regulated_input(
                regulated::RegulatedCommunicationReference::HashChainRecord(hash_pointer.clone()),
                None,
                regulated::RegulatedCoreDecisionParticipation::NeverParticipates,
                regulated::RegulatedCoreAuditMutation::MutatesCoreAuditEvent,
                regulated::RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
            ),
            regulated::RegulatedEnrichmentFailureKind::CoreAuditMutationRequested,
        ),
        (
            regulated_input(
                regulated::RegulatedCommunicationReference::HashChainRecord(hash_pointer),
                None,
                regulated::RegulatedCoreDecisionParticipation::NeverParticipates,
                regulated::RegulatedCoreAuditMutation::ImmutableReferenceOnly,
                regulated::RegulatedDomainPayloadRequirement::RequiredByGenericCore,
            ),
            regulated::RegulatedEnrichmentFailureKind::DomainPayloadRequiredByGenericCore,
        ),
    ] {
        let failure = regulated::RegulatedEnrichmentGuard::admit(input).unwrap_err();
        assert_eq!(failure.kind(), expected);
        assert_eq!(failure.reason_code(), expected.reason_code());
    }

    assert_eq!(
        regulated::RegulatedHashChainRecordPointer::accept("")
            .unwrap_err()
            .kind(),
        regulated::RegulatedEnrichmentFailureKind::EmptyHashChainPointer
    );
    assert_eq!(
        regulated::RegulatedHashChainRecordPointer::accept("hash\nchain")
            .unwrap_err()
            .reason_code(),
        "regulated_control_character_in_hash_chain_pointer"
    );
}

#[test]
fn regulated_dependency_matrix_and_failure_codes_are_closed() {
    for source in [
        regulated::RegulatedDependencySource::Core,
        regulated::RegulatedDependencySource::Drivers,
        regulated::RegulatedDependencySource::Entrypoints,
        regulated::RegulatedDependencySource::Sdk,
        regulated::RegulatedDependencySource::Regulated,
    ] {
        for target in [
            regulated::RegulatedDependencyTarget::CoreOpaqueIdentityReferences,
            regulated::RegulatedDependencyTarget::CoreProtocolSemantics,
            regulated::RegulatedDependencyTarget::Drivers,
            regulated::RegulatedDependencyTarget::Entrypoints,
            regulated::RegulatedDependencyTarget::Sdk,
            regulated::RegulatedDependencyTarget::Regulated,
        ] {
            let admitted = regulated::RegulatedDependencyGuard::admits(source, target);
            assert_eq!(
                admitted,
                matches!(
                    (source, target),
                    (
                        regulated::RegulatedDependencySource::Regulated,
                        regulated::RegulatedDependencyTarget::CoreOpaqueIdentityReferences
                    )
                )
            );
        }
    }

    let mut reason_codes = HashSet::new();
    for kind in [
        regulated::RegulatedEnrichmentFailureKind::SupportClassOwnsCoreDecision,
        regulated::RegulatedEnrichmentFailureKind::LifecycleNotPostCoreOptional,
        regulated::RegulatedEnrichmentFailureKind::ReferenceIsNotImmutablePointer,
        regulated::RegulatedEnrichmentFailureKind::TagRequiredByGenericCore,
        regulated::RegulatedEnrichmentFailureKind::TagDrivesCoreBehavior,
        regulated::RegulatedEnrichmentFailureKind::CoreDecisionParticipationRequested,
        regulated::RegulatedEnrichmentFailureKind::CoreAuditMutationRequested,
        regulated::RegulatedEnrichmentFailureKind::DomainPayloadRequiredByGenericCore,
        regulated::RegulatedEnrichmentFailureKind::EmptyHashChainPointer,
        regulated::RegulatedEnrichmentFailureKind::ControlCharacterInHashChainPointer,
    ] {
        let failure = regulated::RegulatedEnrichmentFailure::new(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(failure.reason_code(), kind.reason_code());
        assert!(reason_codes.insert(kind.reason_code()));
    }
}

#[test]
fn prohibited_regulated_behaviors_remain_visible_closed_vocabulary() {
    for behavior in [
        regulated::ProhibitedRegulatedBehavior::CoreImportsRegulatedType,
        regulated::ProhibitedRegulatedBehavior::SdkDependsOnRegulated,
        regulated::ProhibitedRegulatedBehavior::RegulatedDependsOnSdk,
        regulated::ProhibitedRegulatedBehavior::RegulatedDependsOnDrivers,
        regulated::ProhibitedRegulatedBehavior::RegulatedDependsOnEntrypoints,
        regulated::ProhibitedRegulatedBehavior::RegulatedOwnsCommunicationProtocolDecision,
        regulated::ProhibitedRegulatedBehavior::SensitiveDomainPayloadRequiredByCoreEvent,
        regulated::ProhibitedRegulatedBehavior::GenericCoreRequiresRegulatedEnrichment,
        regulated::ProhibitedRegulatedBehavior::RegulatedMutatesCoreAuditEvent,
        regulated::ProhibitedRegulatedBehavior::RegulatedTagBecomesHiddenCoreAuthorizationPolicy,
        regulated::ProhibitedRegulatedBehavior::DriverBufferOrPacketLeaseReference,
        regulated::ProhibitedRegulatedBehavior::RawCredentialReference,
        regulated::ProhibitedRegulatedBehavior::RawMediaPayloadReference,
    ] {
        assert!(format!("{behavior:?}").contains(|c: char| c.is_ascii_alphanumeric()));
    }
}
