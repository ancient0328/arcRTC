use arcrtc_core_command::{CommandType, CommandVersion, TargetSurface, UseCaseOutcome};
use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority, UntrustedReference};
use arcrtc_core_ports::PortFamily;
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_state::StateClass;
use arcrtc_driver_browser::{
    BrowserConcreteApiType, BrowserDriverResponsibility, BrowserLifecycleMappingError,
    BrowserLifecycleMappingGuard, BrowserLifecycleObservation, BrowserPlatformBoundaryError,
    BrowserPlatformBoundaryGuard, BrowserTypeConversionError, BrowserTypeConversionGuard,
};
use arcrtc_driver_native::{
    NativeConcreteApiType, NativeDriverResponsibility, NativeLifecycleMappingError,
    NativeLifecycleMappingGuard, NativeLifecycleObservation, NativePlatformBoundaryError,
    NativePlatformBoundaryGuard, NativeTypeConversionError, NativeTypeConversionGuard,
};
use arcrtc_driver_network::{
    DriverCommandConversionInput, DriverConversionAuditEventType, DriverConversionFailureKind,
    DriverIngressPreconditions, ExternalErrorProjection, ExternalErrorProjectionError,
    ExternalErrorProjectionInput, ExternalErrorSurface, ExternalIngressKind,
    ExternalStatusWrapperClass, SemanticDelegationError, SemanticDelegationGuard,
    TurnWireDecodeInput, TurnWireDecodedAttributes, TurnWireFailureKind, TurnWireMethodClass,
    TurnWirePreconditions, WireEnvelopeEncodeError, WireEnvelopeEncodeInput,
};
use arcrtc_driver_observability::{
    MetricsExportBacklogBound, MetricsExportBacklogBoundError, ObservabilityProjectionClass,
    ObservabilityProjectionError, ObservabilityProjectionGuard, ObservabilitySensitiveDataError,
    ObservabilitySensitiveDataGuard,
};
use arcrtc_driver_persistence::{
    ArtifactIntegrityClass, ArtifactIntegrityGuard, ArtifactIntegrityGuardError,
    ArtifactRedactionClass, ArtifactRetentionClass, ExportBackupArtifactAdmission,
    ExportBackupArtifactAdmissionError, ExportBackupArtifactClass, MigrationCompatibilityRule,
    MigrationCompatibilityRuleError, MigrationModeSelectionOwner, PersistenceBackendClass,
    PersistenceDriverAdmissionError, PersistenceDriverAdmissionGuard, PersistenceStorageShapeError,
    PersistenceStorageShapeGuard, RestoreReplayApplicability, SchemaMigrationClass,
    SchemaMigrationExecutionGuard, SchemaMigrationExecutionGuardError,
    UnknownPersistedFieldHandling,
};
use arcrtc_driver_security::{
    KeyCacheRefreshBounds, KeyCacheRefreshBoundsError, KeySourceConfigurationError,
    KeySourceConfigurationGuard, KeySourceConfigurationOrigin, SecretGenerationState,
    SecretGenerationStateAdmission, SecretGenerationStateAdmissionError, SecretRotationClass,
    SecretRotationExecutionBoundaryError, SecretRotationExecutionBoundaryGuard,
    SecretRotationPolicyError, SecretRotationPolicyGuard, SecurityKeySourceType,
    TokenVerifierDriverBoundaryError, TokenVerifierDriverBoundaryGuard,
};
use arcrtc_driver_webrtc_str0m::{
    Str0mConversionBoundary, Str0mConversionBoundaryError, Str0mDriverResourceBound,
    Str0mDriverResourceBoundError, Str0mOwnedResourceClass,
};
use arcrtc_roadmap_tests::{
    assert_impl_rust_source_set_contains, assert_not_contains, read_impl_rust_source_set,
};

fn valid_preconditions() -> DriverIngressPreconditions {
    DriverIngressPreconditions::new(
        ExternalIngressKind::WebSocketMessage,
        true,
        8,
        1024,
        true,
        true,
        true,
        true,
        true,
        true,
    )
}

#[test]
fn ce3_schema_migration_and_export_backup_artifacts_are_explicitly_guarded() {
    assert_eq!(
        MigrationCompatibilityRule::try_new(
            vec!["v2"],
            vec!["v1"],
            None,
            true,
            true,
            UnknownPersistedFieldHandling::RejectWithCatalogedReason,
            true,
            true,
            true,
        ),
        Err(MigrationCompatibilityRuleError::CanonicalFormatVersionMissing)
    );
    assert_eq!(
        SchemaMigrationExecutionGuard::try_new(
            SchemaMigrationClass::DriverSchemaForward,
            MigrationModeSelectionOwner::DriverSelfSelection,
            true,
            true,
            true,
        ),
        Err(SchemaMigrationExecutionGuardError::MigrationModeNotSelectedByEntrypoints)
    );

    assert_eq!(
        ExportBackupArtifactAdmission::try_new(
            ExportBackupArtifactClass::EvidenceBundle,
            ArtifactRedactionClass::Redacted,
            ArtifactRetentionClass::BoundedRetention,
            ArtifactIntegrityClass::Digest,
            RestoreReplayApplicability::NotIntendedForRestoreReplay,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(ExportBackupArtifactAdmissionError::IntegrityEvidenceMissing)
    );
    assert_eq!(
        ArtifactIntegrityGuard::try_new(ArtifactIntegrityClass::Digest, false, true, true),
        Err(ArtifactIntegrityGuardError::IntegrityClassConflated)
    );
}

#[test]
fn ce3_secret_rotation_and_platform_lifecycle_mapping_are_fail_closed() {
    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::TurnSharedSecret,
            true,
            0,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(SecretRotationPolicyError::OverlapWindowMissing)
    );
    assert_eq!(
        SecretGenerationStateAdmission::try_new(
            SecretGenerationState::PendingGeneration,
            true,
            true,
            false,
            true,
            true,
        ),
        Err(SecretGenerationStateAdmissionError::PendingGenerationAccepted)
    );
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            false, true, true, true, true, true, true, true,
        ),
        Err(SecretRotationExecutionBoundaryError::RawSecretLoadEscapesDriver)
    );

    assert_eq!(
        BrowserLifecycleMappingGuard::try_new(
            BrowserLifecycleObservation::PlatformShutdown,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(BrowserLifecycleMappingError::DriverShutdownReasonMissing)
    );
    assert_eq!(
        NativeLifecycleMappingGuard::try_new(
            NativeLifecycleObservation::PlatformLifecycleCallback,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        Err(NativeLifecycleMappingError::PlatformLifecycleOwnsDomainTransition)
    );
}

#[test]
fn ce3_turn_wire_decode_projects_only_core_turn_command_fields() {
    assert_eq!(
        TurnWireMethodClass::UnsupportedMethod
            .to_core_command_kind()
            .expect_err("unsupported TURN method must be rejected")
            .kind(),
        TurnWireFailureKind::UnsupportedTurnMethod
    );

    let malformed = TurnWireDecodeInput::new(
        TurnWirePreconditions::new(64, 128, false, true, true, false),
        Some(TurnWireMethodClass::AllocateRequest),
        TurnWireDecodedAttributes::new(
            Some(UntrustedReference::new("turn-tx-ce3")),
            None,
            None,
            None,
            None,
            None,
            Some(60),
            None,
        ),
    );
    assert_eq!(
        malformed
            .into_core_turn_command()
            .expect_err("malformed wire input must fail before core TURN lifecycle")
            .kind(),
        TurnWireFailureKind::MalformedTurnMessage
    );

    let allocate = TurnWireDecodeInput::new(
        TurnWirePreconditions::new(64, 128, true, true, true, false),
        Some(TurnWireMethodClass::AllocateRequest),
        TurnWireDecodedAttributes::new(
            Some(UntrustedReference::new("turn-tx-ce3-ok")),
            None,
            None,
            None,
            None,
            None,
            Some(60),
            None,
        ),
    )
    .into_core_turn_command()
    .expect("valid TURN wire projection must become a core-owned TURN command");
    assert_eq!(allocate.kind(), arcrtc_core_turn::TurnCommandKind::Allocate);
}

fn valid_delegation() -> SemanticDelegationGuard {
    SemanticDelegationGuard::try_new(true, true, false, false)
        .expect("valid driver semantic delegation")
}

#[test]
fn ce3_driver_conversion_returns_core_owned_command_or_closed_failure() {
    let input = DriverCommandConversionInput::new(
        valid_preconditions(),
        valid_delegation(),
        true,
        Some(UntrustedReference::new("corr-ce3")),
        Some(CommandType::new("join_room")),
        Some(CommandVersion::new(1)),
        Some(TargetSurface::Signaling),
        "subject-reference",
    );
    let envelope = input
        .into_core_command_envelope()
        .expect("valid driver input converts to core envelope");
    assert_eq!(envelope.correlation_id().as_str(), "corr-ce3");
    assert_eq!(envelope.target_surface(), TargetSurface::Signaling);

    let missing_correlation = DriverCommandConversionInput::new(
        valid_preconditions(),
        valid_delegation(),
        true,
        None,
        Some(CommandType::new("join_room")),
        Some(CommandVersion::new(1)),
        Some(TargetSurface::Signaling),
        "subject-reference",
    );
    let failure = missing_correlation
        .into_core_command_envelope()
        .expect_err("missing correlation must fail before core entry");
    assert_eq!(
        failure.kind(),
        DriverConversionFailureKind::MissingCorrelationId
    );
    assert_eq!(
        failure.audit_event_type(),
        DriverConversionAuditEventType::DriverErrorConverted
    );
}

#[test]
fn ce3_driver_rejects_semantic_decision_ownership_and_wire_encode_fail_open() {
    assert_eq!(
        SemanticDelegationGuard::try_new(false, true, false, false),
        Err(SemanticDelegationError::CoreRequiredSemanticFieldDropped)
    );
    assert_eq!(
        SemanticDelegationGuard::try_new(true, true, true, false),
        Err(SemanticDelegationError::DriverPerformedCoreSemanticDecision)
    );

    let correlation = arcrtc_core_identity::CorrelationId::new(
        OpaqueReference::accept("corr-ce3-encode", ReferenceAuthority::CorePolicy)
            .expect("valid correlation"),
    );
    assert_eq!(
        WireEnvelopeEncodeInput::try_new(correlation.clone(), UseCaseOutcome::Rejected, None, ()),
        Err(WireEnvelopeEncodeError::MissingCatalogedReasonForFailure)
    );
    assert_eq!(
        WireEnvelopeEncodeInput::try_new(
            correlation,
            UseCaseOutcome::Accepted,
            Some(CatalogedReasonRef::from_code("missing_correlation_id").expect("cataloged")),
            (),
        ),
        Err(WireEnvelopeEncodeError::SuccessReasonMustNotBeInvented)
    );
}

#[test]
fn ce3_external_error_projection_preserves_core_reason_metadata() {
    let unsafe_reason =
        CatalogedReasonRef::from_code("token_verification_failed").expect("cataloged reason");
    assert_eq!(
        ExternalErrorProjection::try_new(ExternalErrorProjectionInput {
            surface: ExternalErrorSurface::Http,
            wrapper_class: ExternalStatusWrapperClass::HttpStatusWrapper,
            correlation_id: None,
            outcome: UseCaseOutcome::Denied,
            authoritative_reason: unsafe_reason,
            opaque_error_ref: None,
            retry_hint_requested: false,
            audit_relation_recorded: true,
        }),
        Err(ExternalErrorProjectionError::UnsafeReasonRequiresOpaqueReference)
    );

    let opaque_error_ref =
        OpaqueReference::accept("opaque-error-ce3", ReferenceAuthority::CorePolicy).unwrap();
    ExternalErrorProjection::try_new(ExternalErrorProjectionInput {
        surface: ExternalErrorSurface::Http,
        wrapper_class: ExternalStatusWrapperClass::HttpStatusWrapper,
        correlation_id: None,
        outcome: UseCaseOutcome::Denied,
        authoritative_reason: unsafe_reason,
        opaque_error_ref: Some(opaque_error_ref),
        retry_hint_requested: false,
        audit_relation_recorded: true,
    })
    .expect("unsafe reason can be projected only with opaque error reference and audit relation");
}

#[test]
fn ce3_platform_drivers_do_not_claim_domain_semantic_ownership() {
    for path in [
        "drivers/webrtc-str0m/src",
        "drivers/persistence/src",
        "drivers/observability/src",
        "drivers/security/src",
        "drivers/browser/src",
        "drivers/native/src",
    ] {
        let source = read_impl_rust_source_set(path);
        assert_not_contains(
            path,
            &source,
            &[
                "SignalingJoinDecision",
                "TurnAllocationDecision",
                "SfuRouteDecision",
                "GenericCommunicationAcceptance",
            ],
        );
    }

    assert_impl_rust_source_set_contains(
        "drivers/webrtc-str0m/src",
        &["Str0mTransportDriverPort", "TransportPort"],
    );
    assert_impl_rust_source_set_contains(
        "drivers/persistence/src",
        &["PersistenceDriverPort", "PersistencePort"],
    );
}

#[test]
fn ce3_concrete_driver_guards_fail_closed_across_platform_storage_security_and_observability() {
    assert_eq!(
        PersistenceStorageShapeGuard::try_new(
            PersistenceBackendClass::SqlDatabase,
            false,
            true,
            true,
            true,
        ),
        Err(PersistenceStorageShapeError::StorageLayoutExposedAsCoreApi)
    );
    assert_eq!(
        PersistenceDriverAdmissionGuard::try_new(
            StateClass::CheckpointEligibleState,
            false,
            true,
            true,
        ),
        Err(PersistenceDriverAdmissionError::DriverOwnsDomainSemantics)
    );

    assert_eq!(
        KeySourceConfigurationGuard::try_new(
            SecurityKeySourceType::JwksEndpoint,
            KeySourceConfigurationOrigin::DriverEnvironmentRead,
            true,
            true,
            true,
            true,
        ),
        Err(KeySourceConfigurationError::ConfigurationOriginInvalid)
    );
    assert_eq!(
        KeyCacheRefreshBounds::try_new(1, 0, 1, 1, 1),
        Err(KeyCacheRefreshBoundsError::RequiredBoundMissing)
    );
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(true, true, false, true, true, true, true, true,),
        Err(TokenVerifierDriverBoundaryError::CorePolicyTakenByDriver)
    );

    assert_eq!(
        ObservabilityProjectionGuard::try_new(
            ObservabilityProjectionClass::Trace,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
        ),
        Err(ObservabilityProjectionError::FreeTextAsAuthority)
    );
    assert_eq!(
        ObservabilitySensitiveDataGuard::try_new(true, false, true, true, true, true),
        Err(ObservabilitySensitiveDataError::SensitiveMaterialInSignal)
    );
    assert_eq!(
        MetricsExportBacklogBound::try_new(0, 1024),
        Err(MetricsExportBacklogBoundError::UnboundedMetricsBacklog)
    );

    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::WebSocketHttpClientBinding,
            Some(PortFamily::Persistence),
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(BrowserPlatformBoundaryError::PortFamilyMismatch)
    );
    assert_eq!(
        BrowserTypeConversionGuard::try_new(
            BrowserConcreteApiType::WebSocket,
            false,
            true,
            true,
            true
        ),
        Err(BrowserTypeConversionError::CoreOwnedConversionMissing)
    );

    assert_eq!(
        NativePlatformBoundaryGuard::try_new(
            NativeDriverResponsibility::WebSocketHttpClientBinding,
            Some(PortFamily::Persistence),
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(NativePlatformBoundaryError::PortFamilyMismatch)
    );
    assert_eq!(
        NativeTypeConversionGuard::try_new(
            NativeConcreteApiType::AndroidSdkOrKotlinType,
            false,
            true,
            true,
            true,
        ),
        Err(NativeTypeConversionError::CoreOwnedConversionMissing)
    );

    assert_eq!(
        Str0mDriverResourceBound::try_new(Str0mOwnedResourceClass::BoundedPacketCache, 0, true),
        Err(Str0mDriverResourceBoundError::UnboundedResource)
    );
    assert_eq!(
        Str0mConversionBoundary::try_new(true, false, true),
        Err(Str0mConversionBoundaryError::NonCoreOwnedEventCommand)
    );
}
