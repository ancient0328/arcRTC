use arcrtc_core_command::{CommandType, CommandVersion, TargetSurface, UseCaseOutcome};
use arcrtc_core_identity::{
    AllocationId, CorrelationId, OpaqueReference, PermissionId, ReferenceAuthority,
    UntrustedReference,
};
use arcrtc_core_ports::{CorePort, PortFamily};
use arcrtc_core_protocol::{
    ContractVersion, CoreSemanticMessageType, CoreSemanticPayloadClass, CoreSemanticPayloadModel,
    SemanticEnvelopeMessageKind, SignalingSemanticCommandType, WireEncodingVersion,
};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_state::StateClass;
use arcrtc_core_turn::TurnCommandKind;
use arcrtc_driver_browser::{
    BrowserDriverFailure, BrowserDriverFailureKind, BrowserDriverResponsibility,
    BrowserDriverSurface, BrowserLifecycleMappingError, BrowserLifecycleMappingGuard,
    BrowserLifecycleObservation, BrowserOutOfScopeAdmissionGuard, BrowserOutOfScopeFeature,
    BrowserPlatformBoundaryError, BrowserPlatformBoundaryGuard, BrowserTypeConversionError,
    BrowserTypeConversionGuard,
};
use arcrtc_driver_native::{
    NativeConcreteApiType, NativeDriverFailure, NativeDriverFailureKind,
    NativeDriverResponsibility, NativeDriverSurface, NativeLifecycleMappingError,
    NativeLifecycleMappingGuard, NativeLifecycleObservation, NativeOutOfScopeAdmissionGuard,
    NativeOutOfScopeFeature, NativePlatformBoundaryError, NativePlatformBoundaryGuard,
    NativeTypeConversionError, NativeTypeConversionGuard,
};
use arcrtc_driver_network::{
    CoreOwnedIngress, DecodedWireEnvelopeFields, DriverCommandConversionInput,
    DriverConversionAuditEventType, DriverConversionFailure, DriverConversionFailureKind,
    DriverIngressPreconditions, ExternalErrorEmissionFailureKind, ExternalErrorEmissionObservation,
    ExternalIngressKind, ExternalWireEncodingClass, ExternalWireEnvelope, NetworkCoreEntryGuard,
    NetworkCoreEntryGuardError, NetworkCoreEntryPath, NetworkInboundFlowStep, NetworkIoDriverPort,
    NetworkIoFailure, NetworkIoFailureKind, NetworkIoLocalBound, NetworkIoPreconditions,
    ProhibitedNetworkIoBehavior, ProhibitedTurnWireDriverBehavior, SemanticDelegationGuard,
    TransportPeerVerificationRequirement, TransportSecurityBackendClass,
    TransportSecurityConfigurationError, TransportSecurityConfigurationGuard,
    TransportSecurityEvidenceClass, TransportSecurityEvidenceShape,
    TransportSecurityEvidenceShapeError, TransportSecurityFailure, TransportSecurityFailureKind,
    TransportSecurityMode, TransportSecurityPathError, TransportSecurityPathGuard,
    TransportSecurityProfile, TransportSecuritySecretHandlingError,
    TransportSecuritySecretHandlingGuard, TurnWireEncodeError, TurnWireEncodeInput,
    TurnWireFailure, TurnWireFailureKind, TurnWireFailureShapeError, TurnWireMethodClass,
    TurnWireOutputClass, TurnWirePreconditions, WireEnvelopeDecodeInput, WireEnvelopeEncodeError,
    WireEnvelopeEncodeInput,
};
use arcrtc_driver_observability::{
    ObservabilitySignalAuditShape, ObservabilitySignalAuditShapeError,
    ObservabilitySignalEvidenceShape, ObservabilitySignalEvidenceShapeError,
    ObservabilitySignalFailure, ObservabilitySignalFailureKind, PrivacyDataClass,
    PrivacyLabelGuard, PrivacyLabelGuardError, PrivacyRedactionRetentionFailure,
    PrivacyRedactionRetentionFailureKind, PrivacyReportEvidenceGuard,
    PrivacyReportEvidenceGuardError, PrivacyRetentionBoundClass, PrivacyRetentionOwner,
    PrivacyRetentionPolicy, PrivacyRetentionPolicyError, PrivacyRetentionTarget,
};
use arcrtc_driver_persistence::{
    ArtifactIntegrityClass, ArtifactIntegrityGuard, ArtifactIntegrityGuardError,
    ArtifactRedactionClass, ArtifactRestoreImportGuard, ArtifactRestoreImportGuardError,
    ArtifactRetentionClass, ArtifactSensitiveDataError, ArtifactSensitiveDataGuard,
    ExportBackupArtifactAdmission, ExportBackupArtifactAdmissionError,
    ExportBackupArtifactAuditShape, ExportBackupArtifactAuditShapeError, ExportBackupArtifactClass,
    ExportBackupArtifactFailure, ExportBackupArtifactFailureKind, MigrationCompatibilityRule,
    MigrationCompatibilityRuleError, MigrationModeSelectionOwner, PersistenceBackendClass,
    PersistenceDriverAdmissionError, PersistenceDriverAdmissionGuard,
    PersistenceDriverFailureSource, PersistenceDriverPort, PersistenceRetryBoundError,
    PersistenceRetryBoundGuard, PersistenceStorageShapeError, PersistenceStorageShapeGuard,
    RestoreReplayApplicability, SchemaMigrationClass, SchemaMigrationExecutionGuard,
    SchemaMigrationExecutionGuardError, SchemaMigrationFailure, SchemaMigrationFailureKind,
    SchemaMigrationReportShape, SchemaMigrationReportShapeError, UnknownPersistedFieldHandling,
};
use arcrtc_driver_security::{
    KeyCacheRefreshBounds, KeyCacheRefreshBoundsError, KeyLookupRefreshError,
    KeyLookupRefreshGuard, KeyLookupRefreshState, KeySourceConfigurationError,
    KeySourceConfigurationGuard, KeySourceConfigurationOrigin, SecretGenerationState,
    SecretGenerationStateAdmission, SecretGenerationStateAdmissionError, SecretRotationAuditShape,
    SecretRotationAuditShapeError, SecretRotationClass, SecretRotationEvidenceShape,
    SecretRotationEvidenceShapeError, SecretRotationExecutionBoundaryError,
    SecretRotationExecutionBoundaryGuard, SecretRotationFailure, SecretRotationFailureKind,
    SecretRotationPolicyError, SecretRotationPolicyGuard, SecurityDriverSurface,
    SecurityKeySourceType, SecurityTokenVerifierDriverPort, TokenVerificationAuditShape,
    TokenVerificationAuditShapeError, TokenVerifierDriverBoundaryError,
    TokenVerifierDriverBoundaryGuard, VerifierBackendFailureClass, VerifierDriverFailure,
};
use arcrtc_driver_webrtc_str0m::{
    ProhibitedStr0mDriverBehavior, ProhibitedStr0mTypeExposure, Str0mConversionBoundary,
    Str0mConversionBoundaryError, Str0mDriverResourceBound, Str0mDriverResourceBoundError,
    Str0mDriverSurface, Str0mOwnedResourceClass, Str0mTransportDriverPort,
};

fn opaque(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("opaque reference")
}

fn untrusted(value: &str) -> UntrustedReference {
    UntrustedReference::new(value)
}

fn correlation(value: &str) -> CorrelationId {
    CorrelationId::new(opaque(value))
}

fn reason(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("cataloged reason")
}

fn valid_ingress() -> DriverIngressPreconditions {
    DriverIngressPreconditions::new(
        ExternalIngressKind::WebSocketMessage,
        true,
        32,
        1024,
        true,
        true,
        true,
        true,
        true,
        true,
    )
}

fn valid_delegation() -> SemanticDelegationGuard {
    SemanticDelegationGuard::try_new(true, true, false, false).expect("valid delegation")
}

#[test]
fn tertiary_browser_native_boundaries_cover_late_fail_closed_edges() {
    assert_eq!(format!("{BrowserDriverSurface:?}"), "BrowserDriverSurface");
    assert_eq!(format!("{NativeDriverSurface:?}"), "NativeDriverSurface");

    // 前段テストの主経路と重複しない late boolean 分岐を直接確認します。
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
            NativeLifecycleObservation::PlatformShutdown,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(NativeLifecycleMappingError::DriverShutdownReasonMissing)
    );

    for (index, expected) in [
        (3, NativePlatformBoundaryError::CoreDecisionSemanticsChanged),
        (
            4,
            NativePlatformBoundaryError::SdkPublicContractOwnedByDriver,
        ),
        (
            5,
            NativePlatformBoundaryError::RegulatedWorkflowOwnedByDriver,
        ),
        (6, NativePlatformBoundaryError::MediaOrUiWorkflowMixed),
    ] {
        let mut flags = [true; 7];
        flags[index] = false;
        assert_eq!(
            NativePlatformBoundaryGuard::try_new(
                NativeDriverResponsibility::TimerClockBinding,
                Some(PortFamily::Clock),
                flags[0],
                flags[1],
                flags[2],
                flags[3],
                flags[4],
                flags[5],
                flags[6],
            ),
            Err(expected)
        );
    }

    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::PlatformErrorConversion,
            Some(PortFamily::Network),
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
        NativeTypeConversionGuard::try_new(
            NativeConcreteApiType::AndroidSdkOrKotlinType,
            true,
            true,
            false,
            true,
        ),
        Err(NativeTypeConversionError::SemanticCategoryChanged)
    );
    assert_eq!(
        BrowserTypeConversionGuard::try_new(
            arcrtc_driver_browser::BrowserConcreteApiType::WebSocket,
            true,
            true,
            true,
            false,
        ),
        Err(BrowserTypeConversionError::PlatformErrorTextAsReason)
    );

    for feature in [
        BrowserOutOfScopeFeature::ScreenSharing,
        BrowserOutOfScopeFeature::UserAccountOrAuthIssuance,
    ] {
        assert!(BrowserOutOfScopeAdmissionGuard::try_new(feature, true, false).is_ok());
    }
    for feature in [
        NativeOutOfScopeFeature::PushNotificationWorkflow,
        NativeOutOfScopeFeature::UserAccountOrAuthIssuance,
    ] {
        assert!(NativeOutOfScopeAdmissionGuard::try_new(feature, true, false).is_ok());
    }

    for kind in [
        BrowserDriverFailureKind::ExternalEncodeFailed,
        BrowserDriverFailureKind::RuntimeConfigInvalid,
    ] {
        assert!(CatalogedReasonRef::from_code(kind.reason_code()).is_ok());
    }
    for kind in [
        NativeDriverFailureKind::ExternalEncodeFailed,
        NativeDriverFailureKind::RuntimeConfigInvalid,
    ] {
        assert!(CatalogedReasonRef::from_code(kind.reason_code()).is_ok());
    }
    assert!(format!(
        "{:?}{:?}",
        BrowserDriverFailure::from_kind(BrowserDriverFailureKind::DriverShutdown),
        NativeDriverFailure::from_kind(NativeDriverFailureKind::DriverShutdown)
    )
    .contains("driver_shutdown"));
}

#[test]
fn tertiary_network_conversion_io_and_turn_edges_cover_public_failures() {
    let leak_failure = DriverIngressPreconditions::new(
        ExternalIngressKind::TcpFrame,
        true,
        16,
        1024,
        true,
        true,
        true,
        true,
        true,
        false,
    )
    .validate_before_core_entry()
    .unwrap_err();
    assert_eq!(
        leak_failure.kind(),
        DriverConversionFailureKind::ExternalTypeLeakBlocked
    );

    for kind in [
        DriverConversionFailureKind::ExternalDecodeFailed,
        DriverConversionFailureKind::FrameSizeBoundExceeded,
        DriverConversionFailureKind::UnsupportedDriverWireVersion,
        DriverConversionFailureKind::MissingCorrelationId,
        DriverConversionFailureKind::MissingRequiredWireField,
        DriverConversionFailureKind::ExternalEnumUnmapped,
        DriverConversionFailureKind::NetworkReceiveFailed,
        DriverConversionFailureKind::NetworkSendFailed,
        DriverConversionFailureKind::ExternalTypeLeakBlocked,
        DriverConversionFailureKind::ExternalEncodeFailed,
    ] {
        let failure = DriverConversionFailure::from_kind(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(failure.reason().code().as_str(), kind.reason_code());
        assert_eq!(
            failure.audit_event_type(),
            if matches!(kind, DriverConversionFailureKind::FrameSizeBoundExceeded) {
                DriverConversionAuditEventType::DriverResourceBoundDecision
            } else {
                DriverConversionAuditEventType::DriverErrorConverted
            }
        );
        assert!(failure.outcome().requires_reason());
    }

    assert_eq!(
        NetworkCoreEntryGuard::try_new(NetworkCoreEntryPath::DomainAggregateInternal),
        Err(NetworkCoreEntryGuardError::DomainAggregateEntryForbidden)
    );
    assert_eq!(
        <NetworkIoDriverPort as CorePort>::FAMILY,
        PortFamily::Network
    );

    let connection_failure =
        NetworkIoPreconditions::new(false, 8, NetworkIoLocalBound::new(4096, 8))
            .validate_connection_admission()
            .unwrap_err();
    assert_eq!(
        connection_failure.kind(),
        NetworkIoFailureKind::ConnectionConcurrencyExceeded
    );
    assert!(connection_failure.resource_bound_decision().is_some());

    let frame_failure = NetworkIoPreconditions::new(false, 0, NetworkIoLocalBound::new(64, 8))
        .validate_frame_size(65)
        .unwrap_err();
    assert_eq!(
        frame_failure.kind(),
        NetworkIoFailureKind::FrameSizeBoundExceeded
    );
    assert!(
        format!("{:?}", frame_failure.resource_bound_decision().unwrap())
            .contains("InboundFrameSize")
    );

    for kind in [
        NetworkIoFailureKind::ExternalDecodeFailed,
        NetworkIoFailureKind::UnsupportedDriverWireVersion,
        NetworkIoFailureKind::MissingRequiredWireField,
        NetworkIoFailureKind::ExternalEnumUnmapped,
        NetworkIoFailureKind::NetworkReceiveFailed,
        NetworkIoFailureKind::NetworkSendFailed,
        NetworkIoFailureKind::DriverShutdown,
    ] {
        let failure = NetworkIoFailure::from_kind(kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            kind.reason_code()
        );
        assert!(failure.resource_bound_decision().is_none());
    }

    let command_input = DriverCommandConversionInput::new(
        valid_ingress(),
        valid_delegation(),
        true,
        Some(untrusted("corr-tertiary-command")),
        Some(CommandType::new("tertiary.command")),
        Some(CommandVersion::new(1)),
        None,
        vec![opaque("subject-tertiary")],
    );
    assert_eq!(
        command_input
            .into_core_command_envelope()
            .unwrap_err()
            .kind(),
        DriverConversionFailureKind::MissingRequiredWireField
    );

    let command = DriverCommandConversionInput::new(
        valid_ingress(),
        valid_delegation(),
        true,
        Some(untrusted("corr-tertiary-owned-ingress")),
        Some(CommandType::new("tertiary.command")),
        Some(CommandVersion::new(1)),
        Some(TargetSurface::Signaling),
        vec![opaque("subject-tertiary-owned-ingress")],
    )
    .into_core_command_envelope()
    .unwrap();
    let ingress: CoreOwnedIngress<Vec<OpaqueReference>, &'static str> =
        CoreOwnedIngress::Command(command);
    assert!(format!("{ingress:?}").contains("Command"));
    let packet: CoreOwnedIngress<Vec<OpaqueReference>, &'static str> =
        CoreOwnedIngress::PacketView("packet-view");
    assert!(format!("{packet:?}").contains("PacketView"));

    assert_eq!(
        TurnWireMethodClass::UnsupportedMethod
            .to_core_command_kind()
            .unwrap_err()
            .kind(),
        TurnWireFailureKind::UnsupportedTurnMethod
    );
    assert_eq!(
        TurnWireMethodClass::SendIndication
            .to_core_command_kind()
            .unwrap(),
        TurnCommandKind::RelayData
    );
    assert_eq!(
        TurnWirePreconditions::new(128, 64, true, true, true, false)
            .validate_before_core_entry()
            .unwrap_err()
            .kind(),
        TurnWireFailureKind::FrameSizeBoundExceeded
    );
    assert_eq!(
        TurnWireFailure::try_from_kind(TurnWireFailureKind::TurnRelayQueueBoundExceeded),
        Err(TurnWireFailureShapeError::RelayQueueReferencesRequired)
    );
    let relay_failure = TurnWireFailure::relay_queue_bound(
        AllocationId::new(opaque("alloc-tertiary")),
        PermissionId::new(opaque("perm-tertiary")),
    );
    assert_eq!(
        relay_failure.kind(),
        TurnWireFailureKind::TurnRelayQueueBoundExceeded
    );
    assert!(relay_failure.resource_bound_decision().is_some());

    for output in [
        TurnWireOutputClass::AllocationSuccessResponse,
        TurnWireOutputClass::RefreshSuccessResponse,
        TurnWireOutputClass::PermissionSuccessResponse,
        TurnWireOutputClass::ChannelBindSuccessResponse,
        TurnWireOutputClass::RelayDataForwarding,
    ] {
        assert!(output.is_success_output());
    }
    for output in [
        TurnWireOutputClass::ErrorResponse,
        TurnWireOutputClass::DropOrDeny,
    ] {
        assert!(!output.is_success_output());
    }
    assert_eq!(
        TurnWireEncodeInput::try_new(
            UseCaseOutcome::Accepted,
            Some(reason("external_decode_failed")),
            TurnWireOutputClass::AllocationSuccessResponse,
            "ok",
        ),
        Err(TurnWireEncodeError::SuccessReasonMustNotBeInvented)
    );
    assert_eq!(
        TurnWireEncodeInput::try_new(
            UseCaseOutcome::Rejected,
            Some(reason("external_decode_failed")),
            TurnWireOutputClass::AllocationSuccessResponse,
            "bad",
        ),
        Err(TurnWireEncodeError::FailureMappedToSuccessOutput)
    );
}

#[test]
fn tertiary_network_wire_envelope_and_transport_security_edges_are_closed() {
    let external = ExternalWireEnvelope::new(
        ExternalWireEncodingClass::Json,
        WireEncodingVersion::new("wire-tertiary"),
        "metadata",
        64,
    );
    let fields = DecodedWireEnvelopeFields::new(
        None,
        Some(ContractVersion::new(0, 2, 0)),
        Some(untrusted("corr-tertiary-wire")),
        Some(SemanticEnvelopeMessageKind::Command),
        Some(CoreSemanticMessageType::SignalingCommand(
            SignalingSemanticCommandType::JoinRoom,
        )),
        true,
        Some(CoreSemanticPayloadModel::new(
            CoreSemanticPayloadClass::OpaqueCoreReference,
            Some(opaque("payload-tertiary")),
        )),
    );
    let decode_input =
        WireEnvelopeDecodeInput::new(external, valid_ingress(), valid_delegation(), true, fields);
    assert_eq!(
        decode_input
            .into_core_semantic_envelope()
            .unwrap_err()
            .kind(),
        DriverConversionFailureKind::MissingRequiredWireField
    );

    assert_eq!(
        WireEnvelopeEncodeInput::try_new(
            correlation("corr-wire-encode"),
            UseCaseOutcome::Rejected,
            None,
            "response",
        ),
        Err(WireEnvelopeEncodeError::MissingCatalogedReasonForFailure)
    );
    assert_eq!(
        WireEnvelopeEncodeInput::try_new(
            correlation("corr-wire-encode-success"),
            UseCaseOutcome::Accepted,
            Some(reason("external_decode_failed")),
            "response",
        ),
        Err(WireEnvelopeEncodeError::SuccessReasonMustNotBeInvented)
    );
    let emission = ExternalErrorEmissionObservation::from_kind(
        reason("external_decode_failed"),
        ExternalErrorEmissionFailureKind::NetworkSendFailed,
    );
    assert!(format!("{emission:?}").contains("network_send_failed"));

    for (mode, profile, requirement, backend) in [
        (
            TransportSecurityMode::RequiredTls,
            TransportSecurityProfile::Tls,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::TlsBackend,
        ),
        (
            TransportSecurityMode::RequiredMtls,
            TransportSecurityProfile::InternalServiceIdentity,
            TransportPeerVerificationRequirement::RequiredWithInternalServiceIdentity,
            TransportSecurityBackendClass::MtlsBackend,
        ),
        (
            TransportSecurityMode::RequiredDtlsSrtp,
            TransportSecurityProfile::DtlsSrtp,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::SrtpBackend,
        ),
        (
            TransportSecurityMode::EdgeTerminatedWithDownstreamTrust,
            TransportSecurityProfile::EdgeTerminationDownstreamProtection,
            TransportPeerVerificationRequirement::RequiredWithEdgeTrustMapping,
            TransportSecurityBackendClass::EdgeDownstreamProtection,
        ),
    ] {
        assert!(mode.accepts_profile(profile));
        assert!(mode.accepts_peer_verification(requirement));
        assert!(mode.accepts_backend_class(backend));
    }
    assert_eq!(
        TransportSecurityConfigurationGuard::try_new(
            TransportSecurityMode::DevelopmentOnlyNonSensitive,
            TransportSecurityProfile::Tls,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::TlsBackend,
            false,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(TransportSecurityConfigurationError::DevelopmentOnlyModeMisused)
    );
    assert_eq!(
        TransportSecurityPathGuard::try_new(false, false, false, true, true, false, true),
        Err(TransportSecurityPathError::DevelopmentOnlyBoundaryMissing)
    );
    assert_eq!(
        TransportSecuritySecretHandlingGuard::try_new(true, true, true, true, false, true, true),
        Err(TransportSecuritySecretHandlingError::RawSecretInReport)
    );
    assert_eq!(
        TransportSecurityEvidenceShape::try_new(
            TransportSecurityEvidenceClass::RotationGenerationOverlap,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
        ),
        Err(TransportSecurityEvidenceShapeError::RotationEvidenceMissing)
    );

    for kind in [
        TransportSecurityFailureKind::SecretUnavailable,
        TransportSecurityFailureKind::SecretRotationStateUnavailable,
        TransportSecurityFailureKind::SecretKeyRevoked,
        TransportSecurityFailureKind::UnsupportedMediaContractVersion,
    ] {
        assert!(
            format!("{:?}", TransportSecurityFailure::from_kind(kind)).contains(kind.reason_code())
        );
    }
}

#[test]
fn tertiary_observability_privacy_signal_edges_are_closed() {
    for kind in [
        ObservabilitySignalFailureKind::AlertSignalNotAllowed,
        ObservabilitySignalFailureKind::ObservabilityExportNotAllowed,
        ObservabilitySignalFailureKind::MetricsBacklogBoundExceeded,
    ] {
        let failure = ObservabilitySignalFailure::from_kind(kind);
        assert!(format!("{failure:?}").contains(kind.reason_code()));
    }
    assert_eq!(
        ObservabilitySignalEvidenceShape::try_new(true, true, true, true, true, true, false),
        Err(ObservabilitySignalEvidenceShapeError::RequiredSignalEvidenceFieldMissing)
    );
    assert_eq!(
        ObservabilitySignalAuditShape::try_new(true, true, true, false, true),
        Err(ObservabilitySignalAuditShapeError::RequiredSignalAuditFieldMissing)
    );
    assert_eq!(
        PrivacyRetentionTarget::PacketCache.required_owner(),
        PrivacyRetentionOwner::Driver
    );
    assert_eq!(
        PrivacyRetentionPolicy::try_new(
            PrivacyRetentionTarget::PacketCache,
            PrivacyRetentionOwner::Driver,
            PrivacyDataClass::RawPacketPayload,
            PrivacyRetentionBoundClass::CanonicalClosedFields,
            true,
            true,
            true,
            true,
        ),
        Err(PrivacyRetentionPolicyError::DriverCacheNotBoundedLocal)
    );
    assert_eq!(
        PrivacyRetentionPolicy::try_new(
            PrivacyRetentionTarget::ReportEvidence,
            PrivacyRetentionOwner::Reports,
            PrivacyDataClass::RawToken,
            PrivacyRetentionBoundClass::SpecializedRetentionCanonical,
            true,
            true,
            true,
            true,
        ),
        Err(PrivacyRetentionPolicyError::DataClassCannotBeRetained)
    );
    assert_eq!(
        PrivacyReportEvidenceGuard::try_new(true, true, true, true, true, true, true, false),
        Err(PrivacyReportEvidenceGuardError::RedactedEvidenceReferenceMissing)
    );
    assert_eq!(
        PrivacyLabelGuard::try_new(
            PrivacyDataClass::DiagnosticDetail,
            true,
            true,
            true,
            false,
            true,
        ),
        Err(PrivacyLabelGuardError::UnboundedLabelCardinality)
    );
    for kind in [
        PrivacyRedactionRetentionFailureKind::PacketPayloadEvidenceRejected,
        PrivacyRedactionRetentionFailureKind::SensitiveCardinalityRejected,
        PrivacyRedactionRetentionFailureKind::RotationEvidenceRejected,
    ] {
        assert!(
            format!("{:?}", PrivacyRedactionRetentionFailure::from_kind(kind))
                .contains(kind.reason_code())
        );
    }
}

#[test]
fn tertiary_persistence_schema_artifact_edges_are_closed() {
    assert_eq!(
        <PersistenceDriverPort as CorePort>::FAMILY,
        PortFamily::Persistence
    );
    assert_eq!(
        PersistenceStorageShapeGuard::try_new(
            PersistenceBackendClass::ObjectStore,
            true,
            true,
            false,
            true,
        ),
        Err(PersistenceStorageShapeError::MigrationFileAsDomainModel)
    );
    assert_eq!(
        PersistenceDriverAdmissionGuard::try_new(StateClass::DriverLocalState, true, true, false),
        Err(PersistenceDriverAdmissionError::FailedPersistenceOutputUsedAsEvidence)
    );
    assert_eq!(
        PersistenceRetryBoundGuard::try_new(true, true, false, true),
        Err(PersistenceRetryBoundError::UnboundedRetryStore)
    );
    assert_eq!(
        PersistenceDriverFailureSource::AuditBacklogExceeded
            .to_port_failure()
            .kind(),
        arcrtc_core_ports::PersistencePortFailureKind::AuditBacklogBoundExceeded
    );

    assert_eq!(
        MigrationCompatibilityRule::try_new(
            vec!["v1"],
            vec!["v0"],
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
    for kind in [
        SchemaMigrationFailureKind::PersistenceUnavailable,
        SchemaMigrationFailureKind::PersistenceRetryBoundExceeded,
        SchemaMigrationFailureKind::DriverShutdown,
    ] {
        assert!(
            format!("{:?}", SchemaMigrationFailure::from_kind(kind)).contains(kind.reason_code())
        );
    }
    assert_eq!(
        SchemaMigrationReportShape::try_new(
            true, true, true, true, true, false, true, false, true, true,
        ),
        Err(SchemaMigrationReportShapeError::ResultOrReasonMissing)
    );

    assert_eq!(
        ExportBackupArtifactAdmission::try_new(
            ExportBackupArtifactClass::EvidenceBundle,
            ArtifactRedactionClass::Redacted,
            ArtifactRetentionClass::BoundedRetention,
            ArtifactIntegrityClass::Digest,
            RestoreReplayApplicability::NotIntendedForRestoreReplay,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(ExportBackupArtifactAdmissionError::SchemaFormatVersionMissing)
    );
    assert_eq!(
        ArtifactSensitiveDataGuard::try_new(true, true, false, true, true, true),
        Err(ArtifactSensitiveDataError::SensitiveMaterialRequiresRedaction)
    );
    assert_eq!(
        ArtifactIntegrityGuard::try_new(ArtifactIntegrityClass::StorageChecksum, true, false, true),
        Err(ArtifactIntegrityGuardError::IntegrityClassConflated)
    );
    assert_eq!(
        ArtifactRestoreImportGuard::try_new(
            RestoreReplayApplicability::RequiresRestoreCanonicalAdmission,
            true,
            false,
        ),
        Err(ArtifactRestoreImportGuardError::RestoreReplayEvidenceMissing)
    );
    for kind in [
        ExportBackupArtifactFailureKind::ExportSurfaceNotAllowed,
        ExportBackupArtifactFailureKind::ArtifactRestoreNotAllowed,
    ] {
        assert!(
            format!("{:?}", ExportBackupArtifactFailure::from_kind(kind))
                .contains(kind.reason_code())
        );
    }
    assert_eq!(
        ExportBackupArtifactAuditShape::try_new(true, true, true, true, true, true, true, false),
        Err(ExportBackupArtifactAuditShapeError::RequiredAuditFieldMissing)
    );
}

#[test]
fn tertiary_security_token_rotation_edges_are_closed() {
    assert_eq!(
        format!("{SecurityDriverSurface:?}"),
        "SecurityDriverSurface"
    );
    assert_eq!(
        <SecurityTokenVerifierDriverPort as CorePort>::FAMILY,
        PortFamily::TokenVerifier
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
        KeyCacheRefreshBounds::try_new(1, 1, 1, 0, 1),
        Err(KeyCacheRefreshBoundsError::RequiredBoundMissing)
    );
    assert_eq!(
        KeyLookupRefreshGuard::try_new(
            KeyLookupRefreshState::KeySourceUnavailable,
            true,
            false,
            false,
        ),
        Err(KeyLookupRefreshError::KeyUnavailableReasonMissing)
    );

    for backend_failure in [
        VerifierBackendFailureClass::TokenAbsent,
        VerifierBackendFailureClass::RequiredConfigMissing,
        VerifierBackendFailureClass::SecretUnavailable,
        VerifierBackendFailureClass::RotationPolicyRejected,
    ] {
        let failure = VerifierDriverFailure::from_backend_failure(backend_failure);
        assert!(format!("{failure:?}").contains(backend_failure.reason_code()));
    }
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(true, true, true, true, false, true, true, true),
        Err(TokenVerifierDriverBoundaryError::TokenIssuanceMixed)
    );
    assert_eq!(
        TokenVerificationAuditShape::try_new(true, true, false, true),
        Err(TokenVerificationAuditShapeError::PublicWrapperReplacesConcreteReason)
    );

    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::TransportCertificateKey,
            true,
            1000,
            true,
            true,
            true,
            false,
            true,
        ),
        Err(SecretRotationPolicyError::AuditEvidenceRelationMissing)
    );
    assert_eq!(
        SecretGenerationStateAdmission::try_new(
            SecretGenerationState::PreviousGenerationOverlap,
            true,
            false,
            false,
            true,
            true,
        ),
        Err(SecretGenerationStateAdmissionError::PreviousOverlapInvalid)
    );
    for kind in [
        SecretRotationFailureKind::SecretRotationRequired,
        SecretRotationFailureKind::SecretOverlapWindowExpired,
        SecretRotationFailureKind::TokenKeyUnavailable,
    ] {
        assert!(
            format!("{:?}", SecretRotationFailure::from_kind(kind)).contains(kind.reason_code())
        );
    }
    assert_eq!(
        SecretRotationEvidenceShape::try_new(true, true, true, true, true, true, false, true, true),
        Err(SecretRotationEvidenceShapeError::RawSecretMaterialInEvidence)
    );
    assert_eq!(
        SecretRotationAuditShape::try_new(true, true, true, true, true, false),
        Err(SecretRotationAuditShapeError::CorrelationIdMissing)
    );
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            true, true, true, false, true, true, true, true
        ),
        Err(SecretRotationExecutionBoundaryError::InsecureFallback)
    );
}

#[test]
fn tertiary_webrtc_str0m_resource_and_conversion_edges_are_closed() {
    assert_eq!(format!("{Str0mDriverSurface:?}"), "Str0mDriverSurface");
    assert_eq!(
        <Str0mTransportDriverPort as CorePort>::FAMILY,
        PortFamily::WebRtcTransport
    );

    for resource_class in [
        Str0mOwnedResourceClass::BoundedPacketCache,
        Str0mOwnedResourceClass::TransmitQueue,
        Str0mOwnedResourceClass::BufferPool,
        Str0mOwnedResourceClass::BufferLease,
    ] {
        let bound = Str0mDriverResourceBound::try_new(resource_class, 4, true).unwrap();
        let action = bound.required_closed_action().expect("required action");
        assert_eq!(
            action.resource(),
            resource_class.required_resource_bound_kind().unwrap()
        );
        assert_eq!(
            action.reason_code(),
            resource_class.required_resource_reason_code().unwrap()
        );
    }
    for resource_class in [
        Str0mOwnedResourceClass::ExternalLibraryInitialization,
        Str0mOwnedResourceClass::ByteBufferCodec,
        Str0mOwnedResourceClass::RetryTransportDetail,
        Str0mOwnedResourceClass::SerializationFormat,
        Str0mOwnedResourceClass::TlsPlatformTransportSetting,
    ] {
        assert!(resource_class.required_closed_action().is_none());
    }
    assert_eq!(
        Str0mDriverResourceBound::try_new(Str0mOwnedResourceClass::BufferPool, 0, true),
        Err(Str0mDriverResourceBoundError::UnboundedResource)
    );
    assert_eq!(
        Str0mConversionBoundary::try_new(true, false, true),
        Err(Str0mConversionBoundaryError::NonCoreOwnedEventCommand)
    );
    assert!(format!(
        "{:?}{:?}",
        ProhibitedStr0mTypeExposure::PacketBytesOrBufferLeaseMovedToCore,
        ProhibitedStr0mDriverBehavior::DriverResourceMovesToCoreOwnership
    )
    .contains("DriverResourceMovesToCoreOwnership"));
}

#[test]
fn tertiary_regulated_optional_record_and_closed_vocabulary_edges_are_closed() {
    let input = arcrtc_regulated::RegulatedEnrichmentInput::new(
        arcrtc_regulated::RegulatedOptionalSupportClass::ExternalComplianceIntegrationHelper,
        arcrtc_regulated::RegulatedEnrichmentLifecycleStage::ImmutableAuditPointerReferenced,
        arcrtc_regulated::RegulatedCommunicationReference::Correlation(correlation(
            "corr-regulated-tertiary",
        )),
        None,
        None,
        arcrtc_regulated::RegulatedCoreDecisionParticipation::NeverParticipates,
        arcrtc_regulated::RegulatedCoreAuditMutation::ImmutableReferenceOnly,
        arcrtc_regulated::RegulatedDomainPayloadRequirement::NotRequiredByGenericCore,
    );
    assert_eq!(
        input.support_class(),
        arcrtc_regulated::RegulatedOptionalSupportClass::ExternalComplianceIntegrationHelper
    );
    assert_eq!(input.audit_pointer(), None);
    assert_eq!(input.tag(), None);

    let record = arcrtc_regulated::RegulatedEnrichmentGuard::admit(input).unwrap();
    assert_eq!(
        record.input().lifecycle_stage(),
        arcrtc_regulated::RegulatedEnrichmentLifecycleStage::ImmutableAuditPointerReferenced
    );
    assert!(!record.replaces_core_audit_event());
    assert!(format!("{record:?}").contains("ExternalComplianceIntegrationHelper"));

    for source in [
        arcrtc_regulated::RegulatedDependencySource::Core,
        arcrtc_regulated::RegulatedDependencySource::Drivers,
        arcrtc_regulated::RegulatedDependencySource::Entrypoints,
        arcrtc_regulated::RegulatedDependencySource::Sdk,
        arcrtc_regulated::RegulatedDependencySource::Regulated,
    ] {
        for target in [
            arcrtc_regulated::RegulatedDependencyTarget::CoreOpaqueIdentityReferences,
            arcrtc_regulated::RegulatedDependencyTarget::CoreProtocolSemantics,
            arcrtc_regulated::RegulatedDependencyTarget::Drivers,
            arcrtc_regulated::RegulatedDependencyTarget::Entrypoints,
            arcrtc_regulated::RegulatedDependencyTarget::Sdk,
            arcrtc_regulated::RegulatedDependencyTarget::Regulated,
        ] {
            assert_eq!(
                arcrtc_regulated::RegulatedDependencyGuard::admits(source, target),
                matches!(
                    (source, target),
                    (
                        arcrtc_regulated::RegulatedDependencySource::Regulated,
                        arcrtc_regulated::RegulatedDependencyTarget::CoreOpaqueIdentityReferences
                    )
                )
            );
        }
    }

    for kind in [
        arcrtc_regulated::RegulatedEnrichmentFailureKind::SupportClassOwnsCoreDecision,
        arcrtc_regulated::RegulatedEnrichmentFailureKind::LifecycleNotPostCoreOptional,
        arcrtc_regulated::RegulatedEnrichmentFailureKind::ReferenceIsNotImmutablePointer,
        arcrtc_regulated::RegulatedEnrichmentFailureKind::TagRequiredByGenericCore,
        arcrtc_regulated::RegulatedEnrichmentFailureKind::TagDrivesCoreBehavior,
    ] {
        let failure = arcrtc_regulated::RegulatedEnrichmentFailure::new(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(failure.reason_code(), kind.reason_code());
    }
    assert!(format!(
        "{:?}",
        arcrtc_regulated::ProhibitedRegulatedBehavior::DriverBufferOrPacketLeaseReference
    )
    .contains("DriverBufferOrPacketLeaseReference"));
}

#[test]
fn tertiary_network_and_persistence_prohibited_vocabularies_are_closed() {
    assert_eq!(
        format!(
            "{:?}{:?}{:?}",
            NetworkInboundFlowStep::MapCoreResultToExternalOutput,
            ProhibitedNetworkIoBehavior::ResolvedEndpointAsSemanticAuthority,
            ProhibitedTurnWireDriverBehavior::RelayQueueBoundAsRelayDenial
        ),
        "MapCoreResultToExternalOutputResolvedEndpointAsSemanticAuthorityRelayQueueBoundAsRelayDenial"
    );
}
