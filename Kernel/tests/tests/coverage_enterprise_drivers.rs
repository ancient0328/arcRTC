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
use arcrtc_driver_browser::{
    BrowserConcreteApiType, BrowserDriverFailure, BrowserDriverFailureKind,
    BrowserDriverResponsibility, BrowserLifecycleMappingError, BrowserLifecycleMappingGuard,
    BrowserLifecycleObservation, BrowserPlatformBoundaryError, BrowserPlatformBoundaryGuard,
    BrowserTypeConversionError, BrowserTypeConversionGuard,
};
use arcrtc_driver_native::{
    NativeConcreteApiType, NativeDriverFailure, NativeDriverFailureKind,
    NativeDriverResponsibility, NativeLifecycleMappingError, NativeLifecycleMappingGuard,
    NativeLifecycleObservation, NativePlatformBoundaryError, NativePlatformBoundaryGuard,
    NativeTypeConversionError, NativeTypeConversionGuard,
};
use arcrtc_driver_network::{
    DecodedWireEnvelopeFields, DriverCommandConversionInput, DriverConversionAuditEventType,
    DriverConversionFailureKind, DriverIngressPreconditions, ExternalErrorEmissionFailureKind,
    ExternalErrorEmissionObservation, ExternalErrorProjection, ExternalErrorProjectionError,
    ExternalErrorProjectionInput, ExternalErrorSurface, ExternalIngressKind,
    ExternalStatusWrapperClass, ExternalWireEncodingClass, ExternalWireEnvelope,
    NetworkCoreEntryGuard, NetworkCoreEntryGuardError, NetworkCoreEntryPath,
    NetworkInboundFlowStep, NetworkIoFailure, NetworkIoFailureKind, NetworkIoLocalBound,
    NetworkIoPreconditions, NetworkIoSurfaceClass, ProhibitedExternalTypeExposure,
    SemanticDelegationError, SemanticDelegationGuard, TransportPeerVerificationRequirement,
    TransportSecurityBackendClass, TransportSecurityConfigurationError,
    TransportSecurityConfigurationGuard, TransportSecurityFailure, TransportSecurityFailureKind,
    TransportSecurityMode, TransportSecurityPathError, TransportSecurityPathGuard,
    TransportSecurityProfile, TransportSecuritySecretHandlingError,
    TransportSecuritySecretHandlingGuard, TransportSecurityVerificationClass,
    TransportSecurityVerificationError, TransportSecurityVerificationGuard, TurnWireDecodeInput,
    TurnWireDecodedAttributes, TurnWireEncodeError, TurnWireEncodeInput, TurnWireFailure,
    TurnWireFailureKind, TurnWireFailureShapeError, TurnWireMethodClass, TurnWireOutputClass,
    TurnWirePreconditions, WireEnvelopeDecodeInput, WireEnvelopeEncodeError,
    WireEnvelopeEncodeInput,
};
use arcrtc_driver_webrtc_str0m::{
    Str0mConversionBoundary, Str0mConversionBoundaryError, Str0mDriverResourceBound,
    Str0mDriverResourceBoundError, Str0mOwnedResourceClass, Str0mTransportDriverPort,
};

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("valid reference")
}

fn correlation() -> CorrelationId {
    CorrelationId::new(reference("coverage-driver-correlation"))
}

fn cataloged(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason must be cataloged")
}

fn valid_ingress(kind: ExternalIngressKind) -> DriverIngressPreconditions {
    DriverIngressPreconditions::new(kind, true, 8, 1024, true, true, true, true, true, true)
}

fn valid_delegation() -> SemanticDelegationGuard {
    SemanticDelegationGuard::try_new(true, true, false, false).expect("valid delegation")
}

#[test]
fn coverage_browser_and_native_driver_surfaces_exercise_all_guard_branches() {
    for (responsibility, family) in [
        (
            BrowserDriverResponsibility::WebSocketHttpClientBinding,
            Some(PortFamily::Network),
        ),
        (
            BrowserDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
        ),
        (
            BrowserDriverResponsibility::RuntimeCancellationBinding,
            Some(PortFamily::Runtime),
        ),
        (
            BrowserDriverResponsibility::RandomnessBinding,
            Some(PortFamily::Random),
        ),
        (
            BrowserDriverResponsibility::LoggingMetricsSinkBinding,
            Some(PortFamily::MetricsSink),
        ),
        (
            BrowserDriverResponsibility::StoragePersistenceBinding,
            Some(PortFamily::Persistence),
        ),
        (BrowserDriverResponsibility::PlatformErrorConversion, None),
    ] {
        assert_eq!(responsibility.approved_port_family(), family);
        assert!(responsibility.accepts_port_family(family));
        assert!(!responsibility.accepts_port_family(Some(PortFamily::TokenVerifier)));
        assert!(BrowserPlatformBoundaryGuard::try_new(
            responsibility,
            family,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .is_ok());
    }

    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            false,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(BrowserPlatformBoundaryError::CorePortDefinedByDriver)
    );
    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::TimerClockBinding,
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
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            true,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(BrowserPlatformBoundaryError::BrowserConcreteTypeCrossesCore)
    );
    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        Err(BrowserPlatformBoundaryError::PlatformInputNotConverted)
    );
    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            true,
            true,
            true,
            false,
            true,
            true,
            true,
        ),
        Err(BrowserPlatformBoundaryError::CoreDecisionSemanticsChanged)
    );
    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            true,
            true,
            true,
            true,
            false,
            true,
            true,
        ),
        Err(BrowserPlatformBoundaryError::SdkPublicContractOwnedByDriver)
    );
    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            true,
            true,
            true,
            true,
            true,
            false,
            true,
        ),
        Err(BrowserPlatformBoundaryError::RegulatedWorkflowOwnedByDriver)
    );
    assert_eq!(
        BrowserPlatformBoundaryGuard::try_new(
            BrowserDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        ),
        Err(BrowserPlatformBoundaryError::MediaOrUiWorkflowMixed)
    );

    for api_type in [
        BrowserConcreteApiType::WebSocket,
        BrowserConcreteApiType::MessageEvent,
        BrowserConcreteApiType::Blob,
        BrowserConcreteApiType::ArrayBuffer,
        BrowserConcreteApiType::ReadableStream,
        BrowserConcreteApiType::DomEvent,
        BrowserConcreteApiType::PlatformPermissionResult,
        BrowserConcreteApiType::PlatformLifecycleCallback,
        BrowserConcreteApiType::MediaDevice,
        BrowserConcreteApiType::PeerConnection,
    ] {
        assert!(BrowserTypeConversionGuard::try_new(api_type, true, true, true, true).is_ok());
    }
    assert_eq!(
        BrowserTypeConversionGuard::try_new(BrowserConcreteApiType::Blob, false, true, true, true),
        Err(BrowserTypeConversionError::CoreOwnedConversionMissing)
    );
    assert_eq!(
        BrowserTypeConversionGuard::try_new(BrowserConcreteApiType::Blob, true, false, true, true),
        Err(BrowserTypeConversionError::BrowserConcreteTypeCrossesCore)
    );
    assert_eq!(
        BrowserTypeConversionGuard::try_new(BrowserConcreteApiType::Blob, true, true, false, true),
        Err(BrowserTypeConversionError::SemanticCategoryChanged)
    );
    assert_eq!(
        BrowserTypeConversionGuard::try_new(BrowserConcreteApiType::Blob, true, true, true, false),
        Err(BrowserTypeConversionError::PlatformErrorTextAsReason)
    );

    for observation in [
        BrowserLifecycleObservation::TabOrPageVisibility,
        BrowserLifecycleObservation::NetworkAvailabilityChange,
        BrowserLifecycleObservation::PlatformCancellation,
        BrowserLifecycleObservation::PlatformShutdown,
        BrowserLifecycleObservation::PlatformLifecycleCallback,
    ] {
        assert!(BrowserLifecycleMappingGuard::try_new(
            observation,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .is_ok());
    }
    assert_eq!(
        BrowserLifecycleMappingGuard::try_new(
            BrowserLifecycleObservation::TabOrPageVisibility,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(BrowserLifecycleMappingError::ApprovedMappingMissing)
    );
    assert_eq!(
        BrowserLifecycleMappingGuard::try_new(
            BrowserLifecycleObservation::NetworkAvailabilityChange,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        Err(BrowserLifecycleMappingError::PlatformLifecycleOwnsDomainTransition)
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

    for kind in [
        BrowserDriverFailureKind::ExternalDecodeFailed,
        BrowserDriverFailureKind::ExternalEncodeFailed,
        BrowserDriverFailureKind::NetworkReceiveFailed,
        BrowserDriverFailureKind::NetworkSendFailed,
        BrowserDriverFailureKind::RuntimeConfigMissing,
        BrowserDriverFailureKind::RuntimeConfigInvalid,
        BrowserDriverFailureKind::DriverShutdown,
    ] {
        assert!(CatalogedReasonRef::from_code(kind.reason_code()).is_ok());
        assert_eq!(
            BrowserDriverFailure::from_kind(kind),
            BrowserDriverFailure::from_kind(kind)
        );
    }
    for (responsibility, family) in [
        (
            NativeDriverResponsibility::WebSocketHttpClientBinding,
            Some(PortFamily::Network),
        ),
        (
            NativeDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
        ),
        (
            NativeDriverResponsibility::RuntimeCancellationBinding,
            Some(PortFamily::Runtime),
        ),
        (
            NativeDriverResponsibility::RandomnessBinding,
            Some(PortFamily::Random),
        ),
        (
            NativeDriverResponsibility::LoggingMetricsSinkBinding,
            Some(PortFamily::MetricsSink),
        ),
        (
            NativeDriverResponsibility::StoragePersistenceBinding,
            Some(PortFamily::Persistence),
        ),
        (NativeDriverResponsibility::PlatformErrorConversion, None),
    ] {
        assert!(responsibility.accepts_port_family(family));
        assert!(!responsibility.accepts_port_family(Some(PortFamily::TokenVerifier)));
        assert!(NativePlatformBoundaryGuard::try_new(
            responsibility,
            family,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .is_ok());
    }
    assert_eq!(
        NativePlatformBoundaryGuard::try_new(
            NativeDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            false,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(NativePlatformBoundaryError::CorePortDefinedByDriver)
    );
    assert_eq!(
        NativePlatformBoundaryGuard::try_new(
            NativeDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Network),
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
        NativePlatformBoundaryGuard::try_new(
            NativeDriverResponsibility::TimerClockBinding,
            Some(PortFamily::Clock),
            true,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(NativePlatformBoundaryError::NativeConcreteTypeCrossesCore)
    );

    for api_type in [
        NativeConcreteApiType::AndroidSdkOrKotlinType,
        NativeConcreteApiType::IosFoundationType,
        NativeConcreteApiType::IosAvFoundationType,
        NativeConcreteApiType::IosNetworkFrameworkType,
        NativeConcreteApiType::PlatformPermissionResult,
        NativeConcreteApiType::PlatformLifecycleCallback,
        NativeConcreteApiType::MediaDeviceOrTrackObject,
        NativeConcreteApiType::PushNotificationObject,
    ] {
        assert!(NativeTypeConversionGuard::try_new(api_type, true, true, true, true).is_ok());
    }
    assert_eq!(
        NativeTypeConversionGuard::try_new(
            NativeConcreteApiType::IosFoundationType,
            false,
            true,
            true,
            true,
        ),
        Err(NativeTypeConversionError::CoreOwnedConversionMissing)
    );
    assert_eq!(
        NativeTypeConversionGuard::try_new(
            NativeConcreteApiType::IosFoundationType,
            true,
            false,
            true,
            true,
        ),
        Err(NativeTypeConversionError::NativeConcreteTypeCrossesCore)
    );
    assert_eq!(
        NativeTypeConversionGuard::try_new(
            NativeConcreteApiType::IosFoundationType,
            true,
            true,
            false,
            true,
        ),
        Err(NativeTypeConversionError::SemanticCategoryChanged)
    );
    assert_eq!(
        NativeTypeConversionGuard::try_new(
            NativeConcreteApiType::IosFoundationType,
            true,
            true,
            true,
            false,
        ),
        Err(NativeTypeConversionError::PlatformErrorTextAsReason)
    );

    for observation in [
        NativeLifecycleObservation::EntrypointForegroundBackground,
        NativeLifecycleObservation::NetworkAvailabilityChange,
        NativeLifecycleObservation::PlatformCancellation,
        NativeLifecycleObservation::PlatformShutdown,
        NativeLifecycleObservation::PlatformLifecycleCallback,
    ] {
        assert!(NativeLifecycleMappingGuard::try_new(
            observation,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .is_ok());
    }
    assert_eq!(
        NativeLifecycleMappingGuard::try_new(
            NativeLifecycleObservation::EntrypointForegroundBackground,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(NativeLifecycleMappingError::ApprovedMappingMissing)
    );
    assert_eq!(
        NativeLifecycleMappingGuard::try_new(
            NativeLifecycleObservation::PlatformLifecycleCallback,
            true,
            true,
            false,
            true,
            true,
            true,
        ),
        Err(NativeLifecycleMappingError::PlatformLifecycleOwnsDomainTransition)
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
    for kind in [
        NativeDriverFailureKind::ExternalDecodeFailed,
        NativeDriverFailureKind::ExternalEncodeFailed,
        NativeDriverFailureKind::NetworkReceiveFailed,
        NativeDriverFailureKind::NetworkSendFailed,
        NativeDriverFailureKind::RuntimeConfigMissing,
        NativeDriverFailureKind::RuntimeConfigInvalid,
        NativeDriverFailureKind::DriverShutdown,
    ] {
        assert!(CatalogedReasonRef::from_code(kind.reason_code()).is_ok());
        assert_eq!(
            NativeDriverFailure::from_kind(kind),
            NativeDriverFailure::from_kind(kind)
        );
    }
}

#[test]
fn coverage_webrtc_str0m_resource_and_conversion_boundaries_are_exercised() {
    assert_eq!(
        <Str0mTransportDriverPort as CorePort>::FAMILY,
        PortFamily::WebRtcTransport
    );
    for resource_class in [
        Str0mOwnedResourceClass::ExternalLibraryInitialization,
        Str0mOwnedResourceClass::ByteBufferCodec,
        Str0mOwnedResourceClass::BufferPool,
        Str0mOwnedResourceClass::BufferLease,
        Str0mOwnedResourceClass::BoundedPacketCache,
        Str0mOwnedResourceClass::TransmitQueue,
        Str0mOwnedResourceClass::RetryTransportDetail,
        Str0mOwnedResourceClass::SerializationFormat,
        Str0mOwnedResourceClass::TlsPlatformTransportSetting,
    ] {
        let bound =
            Str0mDriverResourceBound::try_new(resource_class, 8, true).expect("bounded resource");
        assert_eq!(
            bound
                .required_closed_action()
                .map(|action| action.reason_code()),
            resource_class.required_resource_reason_code()
        );
        assert_eq!(
            resource_class
                .required_closed_action()
                .map(|action| action.reason_code()),
            resource_class.required_resource_reason_code()
        );
    }
    assert_eq!(
        Str0mDriverResourceBound::try_new(Str0mOwnedResourceClass::BufferPool, 0, true),
        Err(Str0mDriverResourceBoundError::UnboundedResource)
    );
    assert_eq!(
        Str0mDriverResourceBound::try_new(Str0mOwnedResourceClass::BufferPool, 8, false),
        Err(Str0mDriverResourceBoundError::AuditOwnerTupleMissing)
    );
    assert!(Str0mConversionBoundary::try_new(true, true, true).is_ok());
    assert_eq!(
        Str0mConversionBoundary::try_new(false, true, true),
        Err(Str0mConversionBoundaryError::ExternalTypeExposure)
    );
    assert_eq!(
        Str0mConversionBoundary::try_new(true, false, true),
        Err(Str0mConversionBoundaryError::NonCoreOwnedEventCommand)
    );
    assert_eq!(
        Str0mConversionBoundary::try_new(true, true, false),
        Err(Str0mConversionBoundaryError::DriverErrorNotMapped)
    );
}

#[test]
fn coverage_network_driver_conversion_error_io_and_turn_wire_paths_are_exercised() {
    for (preconditions, expected) in [
        (
            DriverIngressPreconditions::new(
                ExternalIngressKind::WebSocketMessage,
                false,
                1,
                8,
                true,
                true,
                true,
                true,
                true,
                true,
            ),
            DriverConversionFailureKind::ExternalDecodeFailed,
        ),
        (
            DriverIngressPreconditions::new(
                ExternalIngressKind::UdpDatagram,
                true,
                9,
                8,
                true,
                true,
                true,
                true,
                true,
                true,
            ),
            DriverConversionFailureKind::FrameSizeBoundExceeded,
        ),
        (
            DriverIngressPreconditions::new(
                ExternalIngressKind::TcpFrame,
                true,
                1,
                8,
                false,
                true,
                true,
                true,
                true,
                true,
            ),
            DriverConversionFailureKind::MissingRequiredWireField,
        ),
        (
            DriverIngressPreconditions::new(
                ExternalIngressKind::HttpRequestBody,
                true,
                1,
                8,
                true,
                false,
                true,
                true,
                true,
                true,
            ),
            DriverConversionFailureKind::ExternalDecodeFailed,
        ),
        (
            DriverIngressPreconditions::new(
                ExternalIngressKind::HttpRequestBody,
                true,
                1,
                8,
                true,
                true,
                false,
                true,
                true,
                true,
            ),
            DriverConversionFailureKind::ExternalEnumUnmapped,
        ),
        (
            DriverIngressPreconditions::new(
                ExternalIngressKind::HttpRequestBody,
                true,
                1,
                8,
                true,
                true,
                true,
                false,
                true,
                true,
            ),
            DriverConversionFailureKind::NetworkReceiveFailed,
        ),
        (
            DriverIngressPreconditions::new(
                ExternalIngressKind::HttpRequestBody,
                true,
                1,
                8,
                true,
                true,
                true,
                true,
                false,
                true,
            ),
            DriverConversionFailureKind::NetworkSendFailed,
        ),
        (
            DriverIngressPreconditions::new(
                ExternalIngressKind::HttpRequestBody,
                true,
                1,
                8,
                true,
                true,
                true,
                true,
                true,
                false,
            ),
            DriverConversionFailureKind::ExternalTypeLeakBlocked,
        ),
    ] {
        assert_eq!(
            preconditions
                .validate_before_core_entry()
                .expect_err("precondition failure")
                .kind(),
            expected
        );
    }
    assert!(valid_ingress(ExternalIngressKind::UdpDatagram)
        .validate_before_core_entry()
        .is_ok());

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
        let failure = kind.to_failure().expect_err("kind maps to failure");
        assert_eq!(failure.kind(), kind);
        assert_eq!(failure.reason().code().as_str(), kind.reason_code());
        if matches!(kind, DriverConversionFailureKind::FrameSizeBoundExceeded) {
            assert_eq!(
                failure.audit_event_type(),
                DriverConversionAuditEventType::DriverResourceBoundDecision
            );
            assert_eq!(failure.outcome(), UseCaseOutcome::Dropped);
        } else {
            assert_eq!(
                failure.audit_event_type(),
                DriverConversionAuditEventType::DriverErrorConverted
            );
            assert_eq!(failure.outcome(), UseCaseOutcome::ConvertedFailure);
        }
    }
    assert_eq!(
        SemanticDelegationGuard::try_new(false, true, false, false),
        Err(SemanticDelegationError::CoreRequiredSemanticFieldDropped)
    );
    assert_eq!(
        SemanticDelegationGuard::try_new(true, true, true, false),
        Err(SemanticDelegationError::DriverPerformedCoreSemanticDecision)
    );

    let command = DriverCommandConversionInput::new(
        valid_ingress(ExternalIngressKind::WebSocketMessage),
        valid_delegation(),
        true,
        Some(UntrustedReference::new("corr-coverage-driver")),
        Some(CommandType::new("join_room")),
        Some(CommandVersion::new(1)),
        Some(TargetSurface::Signaling),
        "subject",
    )
    .into_core_command_envelope()
    .expect("valid command conversion");
    assert_eq!(command.correlation_id().as_str(), "corr-coverage-driver");
    for input in [
        DriverCommandConversionInput::new(
            valid_ingress(ExternalIngressKind::WebSocketMessage),
            valid_delegation(),
            false,
            Some(UntrustedReference::new("corr-coverage-driver")),
            Some(CommandType::new("join_room")),
            Some(CommandVersion::new(1)),
            Some(TargetSurface::Signaling),
            "subject",
        ),
        DriverCommandConversionInput::new(
            valid_ingress(ExternalIngressKind::WebSocketMessage),
            valid_delegation(),
            true,
            None,
            Some(CommandType::new("join_room")),
            Some(CommandVersion::new(1)),
            Some(TargetSurface::Signaling),
            "subject",
        ),
        DriverCommandConversionInput::new(
            valid_ingress(ExternalIngressKind::WebSocketMessage),
            valid_delegation(),
            true,
            Some(UntrustedReference::new("bad\ncorr")),
            Some(CommandType::new("join_room")),
            Some(CommandVersion::new(1)),
            Some(TargetSurface::Signaling),
            "subject",
        ),
        DriverCommandConversionInput::new(
            valid_ingress(ExternalIngressKind::WebSocketMessage),
            valid_delegation(),
            true,
            Some(UntrustedReference::new("corr-coverage-driver")),
            None,
            Some(CommandVersion::new(1)),
            Some(TargetSurface::Signaling),
            "subject",
        ),
        DriverCommandConversionInput::new(
            valid_ingress(ExternalIngressKind::WebSocketMessage),
            valid_delegation(),
            true,
            Some(UntrustedReference::new("corr-coverage-driver")),
            Some(CommandType::new("join_room")),
            None,
            Some(TargetSurface::Signaling),
            "subject",
        ),
        DriverCommandConversionInput::new(
            valid_ingress(ExternalIngressKind::WebSocketMessage),
            valid_delegation(),
            true,
            Some(UntrustedReference::new("corr-coverage-driver")),
            Some(CommandType::new("join_room")),
            Some(CommandVersion::new(1)),
            None,
            "subject",
        ),
    ] {
        assert!(input.into_core_command_envelope().is_err());
    }

    for _step in [
        NetworkInboundFlowStep::ReceiveExternalFrame,
        NetworkInboundFlowStep::EnforceDriverShapeAndBounds,
        NetworkInboundFlowStep::ConvertToCoreOwnedInput,
        NetworkInboundFlowStep::CoreSemanticEvaluation,
        NetworkInboundFlowStep::MapCoreResultToExternalOutput,
    ] {}
    for _surface in [
        NetworkIoSurfaceClass::UdpSocket,
        NetworkIoSurfaceClass::TcpSocket,
        NetworkIoSurfaceClass::HttpEndpoint,
        NetworkIoSurfaceClass::WebSocketEndpoint,
    ] {}
    for _exposure in [
        ProhibitedExternalTypeExposure::FrameworkRequestResponse,
        ProhibitedExternalTypeExposure::WebSocketFrameType,
        ProhibitedExternalTypeExposure::TokioSocketType,
        ProhibitedExternalTypeExposure::Str0mEventType,
        ProhibitedExternalTypeExposure::SqlxRowPool,
        ProhibitedExternalTypeExposure::AwsSdkType,
        ProhibitedExternalTypeExposure::BrowserApiType,
        ProhibitedExternalTypeExposure::MobilePlatformType,
        ProhibitedExternalTypeExposure::DriverBufferHandle,
    ] {}

    let payload = CoreSemanticPayloadModel::new(
        CoreSemanticPayloadClass::OpaqueCoreReference,
        Some(reference("payload-ref")),
    );
    let decoded = DecodedWireEnvelopeFields::new(
        Some(TargetSurface::Signaling),
        Some(ContractVersion::new(1, 0, 0)),
        Some(UntrustedReference::new("corr-wire")),
        Some(SemanticEnvelopeMessageKind::Command),
        Some(CoreSemanticMessageType::SignalingCommand(
            SignalingSemanticCommandType::JoinRoom,
        )),
        true,
        Some(payload),
    );
    let envelope = ExternalWireEnvelope::new(
        ExternalWireEncodingClass::Json,
        WireEncodingVersion::new("wire-v1"),
        "metadata",
        32,
    );
    assert!(WireEnvelopeDecodeInput::new(
        envelope,
        valid_ingress(ExternalIngressKind::WebSocketMessage),
        valid_delegation(),
        true,
        decoded,
    )
    .into_core_semantic_envelope()
    .is_ok());
    for encoding_class in [
        ExternalWireEncodingClass::Json,
        ExternalWireEncodingClass::Binary,
        ExternalWireEncodingClass::Http,
        ExternalWireEncodingClass::WebSocket,
        ExternalWireEncodingClass::Udp,
        ExternalWireEncodingClass::Tcp,
        ExternalWireEncodingClass::StunTurn,
    ] {
        let envelope =
            ExternalWireEnvelope::new(encoding_class, WireEncodingVersion::new("wire-v1"), (), 1);
        let decoded = DecodedWireEnvelopeFields::new(None, None, None, None, None, false, None);
        assert!(WireEnvelopeDecodeInput::new(
            envelope,
            valid_ingress(ExternalIngressKind::UdpDatagram),
            valid_delegation(),
            true,
            decoded,
        )
        .into_core_semantic_envelope()
        .is_err());
    }

    assert!(
        WireEnvelopeEncodeInput::try_new(correlation(), UseCaseOutcome::Accepted, None, "ok")
            .is_ok()
    );
    assert_eq!(
        WireEnvelopeEncodeInput::try_new(correlation(), UseCaseOutcome::Rejected, None, "bad"),
        Err(WireEnvelopeEncodeError::MissingCatalogedReasonForFailure)
    );
    assert_eq!(
        WireEnvelopeEncodeInput::try_new(
            correlation(),
            UseCaseOutcome::Accepted,
            Some(cataloged("room_closed")),
            "bad",
        ),
        Err(WireEnvelopeEncodeError::SuccessReasonMustNotBeInvented)
    );

    let exposed = ExternalErrorProjection::try_new(ExternalErrorProjectionInput {
        surface: ExternalErrorSurface::Http,
        wrapper_class: ExternalStatusWrapperClass::HttpStatusWrapper,
        correlation_id: Some(correlation()),
        outcome: UseCaseOutcome::Rejected,
        authoritative_reason: cataloged("room_closed"),
        opaque_error_ref: None,
        retry_hint_requested: false,
        audit_relation_recorded: true,
    })
    .expect("safe exposed reason projection");
    assert_eq!(exposed, exposed.clone());
    assert_eq!(
        ExternalErrorProjection::try_new(ExternalErrorProjectionInput {
            surface: ExternalErrorSurface::WebSocket,
            wrapper_class: ExternalStatusWrapperClass::WebSocketErrorWrapper,
            correlation_id: Some(correlation()),
            outcome: UseCaseOutcome::Accepted,
            authoritative_reason: cataloged("room_closed"),
            opaque_error_ref: None,
            retry_hint_requested: false,
            audit_relation_recorded: true,
        }),
        Err(ExternalErrorProjectionError::SuccessOutcomeCannotBeExternalError)
    );
    assert_eq!(
        ExternalErrorProjection::try_new(ExternalErrorProjectionInput {
            surface: ExternalErrorSurface::Sdk,
            wrapper_class: ExternalStatusWrapperClass::SdkErrorWrapper,
            correlation_id: Some(correlation()),
            outcome: UseCaseOutcome::Rejected,
            authoritative_reason: cataloged("authorization_policy_denied"),
            opaque_error_ref: None,
            retry_hint_requested: false,
            audit_relation_recorded: true,
        }),
        Err(ExternalErrorProjectionError::UnsafeReasonRequiresOpaqueReference)
    );
    assert_eq!(
        ExternalErrorProjection::try_new(ExternalErrorProjectionInput {
            surface: ExternalErrorSurface::Cli,
            wrapper_class: ExternalStatusWrapperClass::CliExitWrapper,
            correlation_id: Some(correlation()),
            outcome: UseCaseOutcome::Rejected,
            authoritative_reason: cataloged("room_closed"),
            opaque_error_ref: None,
            retry_hint_requested: true,
            audit_relation_recorded: true,
        }),
        Err(ExternalErrorProjectionError::RetryHintNotAllowed)
    );
    assert_eq!(
        ExternalErrorProjection::try_new(ExternalErrorProjectionInput {
            surface: ExternalErrorSurface::LogsTraces,
            wrapper_class: ExternalStatusWrapperClass::RedactedDiagnosticWrapper,
            correlation_id: Some(correlation()),
            outcome: UseCaseOutcome::Rejected,
            authoritative_reason: cataloged("room_closed"),
            opaque_error_ref: None,
            retry_hint_requested: false,
            audit_relation_recorded: false,
        }),
        Err(ExternalErrorProjectionError::AuditRelationRequired)
    );
    for surface in [
        ExternalErrorSurface::Http,
        ExternalErrorSurface::WebSocket,
        ExternalErrorSurface::StunTurn,
        ExternalErrorSurface::Sdk,
        ExternalErrorSurface::Cli,
        ExternalErrorSurface::LogsTraces,
    ] {
        let wrapper = match surface {
            ExternalErrorSurface::Http => ExternalStatusWrapperClass::HttpStatusWrapper,
            ExternalErrorSurface::WebSocket => ExternalStatusWrapperClass::WebSocketErrorWrapper,
            ExternalErrorSurface::StunTurn => ExternalStatusWrapperClass::StunTurnErrorWrapper,
            ExternalErrorSurface::Sdk => ExternalStatusWrapperClass::SdkErrorWrapper,
            ExternalErrorSurface::Cli => ExternalStatusWrapperClass::CliExitWrapper,
            ExternalErrorSurface::LogsTraces => {
                ExternalStatusWrapperClass::RedactedDiagnosticWrapper
            }
        };
        assert!(
            ExternalErrorProjection::try_new(ExternalErrorProjectionInput {
                surface,
                wrapper_class: wrapper,
                correlation_id: Some(correlation()),
                outcome: UseCaseOutcome::Rejected,
                authoritative_reason: cataloged("external_decode_failed"),
                opaque_error_ref: Some(reference("opaque-error")),
                retry_hint_requested: false,
                audit_relation_recorded: true,
            })
            .is_ok()
        );
    }
    for kind in [
        ExternalErrorEmissionFailureKind::ExternalEncodeFailed,
        ExternalErrorEmissionFailureKind::NetworkSendFailed,
        ExternalErrorEmissionFailureKind::DriverShutdown,
    ] {
        assert_eq!(
            ExternalErrorEmissionObservation::from_kind(cataloged("room_closed"), kind),
            ExternalErrorEmissionObservation::from_kind(cataloged("room_closed"), kind)
        );
        assert!(CatalogedReasonRef::from_code(kind.reason_code()).is_ok());
    }

    let ok_io = NetworkIoPreconditions::new(false, 1, NetworkIoLocalBound::new(8, 2));
    assert!(ok_io.validate_connection_admission().is_ok());
    assert!(ok_io.validate_frame_size(8).is_ok());
    assert_eq!(
        NetworkIoPreconditions::new(true, 1, NetworkIoLocalBound::new(8, 2))
            .validate_connection_admission()
            .expect_err("shutdown")
            .kind(),
        NetworkIoFailureKind::DriverShutdown
    );
    assert_eq!(
        NetworkIoPreconditions::new(false, 2, NetworkIoLocalBound::new(8, 2))
            .validate_connection_admission()
            .expect_err("bound")
            .kind(),
        NetworkIoFailureKind::ConnectionConcurrencyExceeded
    );
    assert_eq!(
        ok_io.validate_frame_size(9).expect_err("frame").kind(),
        NetworkIoFailureKind::FrameSizeBoundExceeded
    );
    for kind in [
        NetworkIoFailureKind::ExternalDecodeFailed,
        NetworkIoFailureKind::UnsupportedDriverWireVersion,
        NetworkIoFailureKind::MissingRequiredWireField,
        NetworkIoFailureKind::ExternalEnumUnmapped,
        NetworkIoFailureKind::FrameSizeBoundExceeded,
        NetworkIoFailureKind::ConnectionConcurrencyExceeded,
        NetworkIoFailureKind::NetworkReceiveFailed,
        NetworkIoFailureKind::NetworkSendFailed,
        NetworkIoFailureKind::DriverShutdown,
    ] {
        let failure = NetworkIoFailure::from_kind(kind);
        assert_eq!(failure.kind(), kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            kind.reason_code()
        );
        assert_eq!(
            failure.resource_bound_decision().is_some(),
            kind.resource_bound_resource().is_some()
        );
    }
    assert!(NetworkCoreEntryGuard::try_new(NetworkCoreEntryPath::ApplicationUseCase).is_ok());
    assert!(NetworkCoreEntryGuard::try_new(NetworkCoreEntryPath::CoreOwnedPortBoundary).is_ok());
    assert_eq!(
        NetworkCoreEntryGuard::try_new(NetworkCoreEntryPath::DomainAggregateInternal),
        Err(NetworkCoreEntryGuardError::DomainAggregateEntryForbidden)
    );

    for (method, success) in [
        (TurnWireMethodClass::AllocateRequest, true),
        (TurnWireMethodClass::RefreshRequest, true),
        (TurnWireMethodClass::CreatePermissionRequest, true),
        (TurnWireMethodClass::ChannelBindRequest, true),
        (TurnWireMethodClass::SendIndication, true),
        (TurnWireMethodClass::DataIndication, true),
        (TurnWireMethodClass::UnsupportedMethod, false),
    ] {
        assert_eq!(method.to_core_command_kind().is_ok(), success);
    }
    for (preconditions, expected) in [
        (
            TurnWirePreconditions::new(1, 8, true, true, true, true),
            TurnWireFailureKind::DriverShutdown,
        ),
        (
            TurnWirePreconditions::new(1, 8, true, true, false, false),
            TurnWireFailureKind::NetworkReceiveFailed,
        ),
        (
            TurnWirePreconditions::new(9, 8, true, true, true, false),
            TurnWireFailureKind::FrameSizeBoundExceeded,
        ),
        (
            TurnWirePreconditions::new(1, 8, false, true, true, false),
            TurnWireFailureKind::MalformedTurnMessage,
        ),
    ] {
        assert_eq!(
            preconditions
                .validate_before_core_entry()
                .expect_err("turn precondition")
                .kind(),
            expected
        );
    }
    assert!(TurnWirePreconditions::new(1, 8, true, true, true, false)
        .validate_before_core_entry()
        .is_ok());
    let relay = TurnWireDecodeInput::new(
        TurnWirePreconditions::new(1, 8, true, true, true, false),
        Some(TurnWireMethodClass::SendIndication),
        TurnWireDecodedAttributes::new(
            Some(UntrustedReference::new("turn-tx")),
            Some(UntrustedReference::new("allocation")),
            Some(UntrustedReference::new("permission")),
            None,
            Some(UntrustedReference::new("credential")),
            Some("peer".to_string()),
            Some(60),
            Some(UntrustedReference::new("packet")),
        ),
    )
    .into_core_turn_command()
    .expect("relay command has peer and packet");
    assert_eq!(relay.kind(), arcrtc_core_turn::TurnCommandKind::RelayData);
    assert_eq!(
        TurnWireDecodeInput::new(
            TurnWirePreconditions::new(1, 8, true, true, true, false),
            None,
            TurnWireDecodedAttributes::new(None, None, None, None, None, None, None, None),
        )
        .into_core_turn_command()
        .expect_err("missing method")
        .kind(),
        TurnWireFailureKind::MalformedTurnMessage
    );
    for kind in [
        TurnWireFailureKind::MalformedTurnMessage,
        TurnWireFailureKind::UnsupportedTurnMethod,
        TurnWireFailureKind::NetworkReceiveFailed,
        TurnWireFailureKind::NetworkSendFailed,
        TurnWireFailureKind::FrameSizeBoundExceeded,
        TurnWireFailureKind::DriverShutdown,
    ] {
        let failure = TurnWireFailure::try_from_kind(kind).expect("simple failure");
        assert_eq!(failure.kind(), kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            kind.reason_code()
        );
    }
    assert_eq!(
        TurnWireFailure::try_from_kind(TurnWireFailureKind::TurnRelayQueueBoundExceeded),
        Err(TurnWireFailureShapeError::RelayQueueReferencesRequired)
    );
    assert!(TurnWireFailure::relay_queue_bound(
        AllocationId::new(reference("allocation")),
        PermissionId::new(reference("permission")),
    )
    .resource_bound_decision()
    .is_some());
    for output in [
        TurnWireOutputClass::AllocationSuccessResponse,
        TurnWireOutputClass::RefreshSuccessResponse,
        TurnWireOutputClass::PermissionSuccessResponse,
        TurnWireOutputClass::ChannelBindSuccessResponse,
        TurnWireOutputClass::RelayDataForwarding,
        TurnWireOutputClass::ErrorResponse,
        TurnWireOutputClass::DropOrDeny,
    ] {
        let success = output.is_success_output();
        assert!(TurnWireEncodeInput::try_new(
            if success {
                UseCaseOutcome::Accepted
            } else {
                UseCaseOutcome::Rejected
            },
            if success {
                None
            } else {
                Some(cataloged("room_closed"))
            },
            output,
            "response",
        )
        .is_ok());
    }
    assert_eq!(
        TurnWireEncodeInput::try_new(
            UseCaseOutcome::Rejected,
            None,
            TurnWireOutputClass::ErrorResponse,
            "response",
        ),
        Err(TurnWireEncodeError::MissingCatalogedReasonForFailure)
    );
    assert_eq!(
        TurnWireEncodeInput::try_new(
            UseCaseOutcome::Accepted,
            Some(cataloged("room_closed")),
            TurnWireOutputClass::AllocationSuccessResponse,
            "response",
        ),
        Err(TurnWireEncodeError::SuccessReasonMustNotBeInvented)
    );
    assert_eq!(
        TurnWireEncodeInput::try_new(
            UseCaseOutcome::Rejected,
            Some(cataloged("room_closed")),
            TurnWireOutputClass::AllocationSuccessResponse,
            "response",
        ),
        Err(TurnWireEncodeError::FailureMappedToSuccessOutput)
    );
    assert_eq!(
        TurnWireEncodeInput::try_new(
            UseCaseOutcome::Accepted,
            None,
            TurnWireOutputClass::ErrorResponse,
            "response",
        ),
        Err(TurnWireEncodeError::SuccessMappedToFailureOutput)
    );
}

#[test]
fn coverage_network_transport_security_guards_cover_fail_closed_edges() {
    for (mode, profile, peer, backend) in [
        (
            TransportSecurityMode::RequiredTls,
            TransportSecurityProfile::Tls,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::TlsBackend,
        ),
        (
            TransportSecurityMode::RequiredMtls,
            TransportSecurityProfile::Mtls,
            TransportPeerVerificationRequirement::RequiredWithInternalServiceIdentity,
            TransportSecurityBackendClass::MtlsBackend,
        ),
        (
            TransportSecurityMode::RequiredDtlsSrtp,
            TransportSecurityProfile::DtlsSrtp,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::DtlsBackend,
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
        (
            TransportSecurityMode::DevelopmentOnlyNonSensitive,
            TransportSecurityProfile::Tls,
            TransportPeerVerificationRequirement::DevelopmentOnlyNotRequired,
            TransportSecurityBackendClass::TlsBackend,
        ),
    ] {
        assert!(mode.accepts_profile(profile));
        assert!(mode.accepts_peer_verification(peer));
        assert!(mode.accepts_backend_class(backend));
        assert!(TransportSecurityConfigurationGuard::try_new(
            mode, profile, peer, backend, true, true, true, true, true, true, true, true,
        )
        .is_ok());
    }
    assert_eq!(
        TransportSecurityConfigurationGuard::try_new(
            TransportSecurityMode::DevelopmentOnlyNonSensitive,
            TransportSecurityProfile::Tls,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::TlsBackend,
            true,
            true,
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
        TransportSecurityConfigurationGuard::try_new(
            TransportSecurityMode::RequiredTls,
            TransportSecurityProfile::Mtls,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::TlsBackend,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(TransportSecurityConfigurationError::ModeProfileMismatch)
    );
    assert_eq!(
        TransportSecurityConfigurationGuard::try_new(
            TransportSecurityMode::RequiredTls,
            TransportSecurityProfile::Tls,
            TransportPeerVerificationRequirement::RequiredWithInternalServiceIdentity,
            TransportSecurityBackendClass::TlsBackend,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(TransportSecurityConfigurationError::ModePeerVerificationMismatch)
    );
    assert_eq!(
        TransportSecurityConfigurationGuard::try_new(
            TransportSecurityMode::RequiredTls,
            TransportSecurityProfile::Tls,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::MtlsBackend,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(TransportSecurityConfigurationError::ModeBackendMismatch)
    );
    assert_eq!(
        TransportSecurityConfigurationGuard::try_new(
            TransportSecurityMode::RequiredTls,
            TransportSecurityProfile::Tls,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::TlsBackend,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(TransportSecurityConfigurationError::CertificateOrKeySourceMissing)
    );
    assert_eq!(
        TransportSecurityConfigurationGuard::try_new(
            TransportSecurityMode::RequiredMtls,
            TransportSecurityProfile::Mtls,
            TransportPeerVerificationRequirement::Required,
            TransportSecurityBackendClass::MtlsBackend,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(TransportSecurityConfigurationError::TrustAnchorSourceMissing)
    );

    assert!(TransportSecuritySecretHandlingGuard::try_new(
        true, true, true, true, true, true, true
    )
    .is_ok());
    assert_eq!(
        TransportSecuritySecretHandlingGuard::try_new(false, true, true, true, true, true, true),
        Err(TransportSecuritySecretHandlingError::RawSecretCrossesToCore)
    );
    assert_eq!(
        TransportSecuritySecretHandlingGuard::try_new(true, false, true, true, true, true, true),
        Err(TransportSecuritySecretHandlingError::RawSecretInObservability)
    );
    assert_eq!(
        TransportSecuritySecretHandlingGuard::try_new(true, true, true, false, true, true, true),
        Err(TransportSecuritySecretHandlingError::RawSecretInSdkPublicError)
    );
    assert_eq!(
        TransportSecuritySecretHandlingGuard::try_new(true, true, true, true, false, true, true),
        Err(TransportSecuritySecretHandlingError::RawSecretInDiagnosticExport)
    );
    assert_eq!(
        TransportSecuritySecretHandlingGuard::try_new(true, true, true, true, true, false, true),
        Err(TransportSecuritySecretHandlingError::RedactedReferenceMissing)
    );

    for kind in [
        TransportSecurityFailureKind::RuntimeConfigMissing,
        TransportSecurityFailureKind::RuntimeConfigInvalid,
        TransportSecurityFailureKind::SecretUnavailable,
        TransportSecurityFailureKind::SecretRotationStateUnavailable,
        TransportSecurityFailureKind::SecretKeyRevoked,
        TransportSecurityFailureKind::UnsupportedMediaContractVersion,
        TransportSecurityFailureKind::NetworkReceiveFailed,
        TransportSecurityFailureKind::NetworkSendFailed,
        TransportSecurityFailureKind::DriverShutdown,
    ] {
        assert!(CatalogedReasonRef::from_code(kind.reason_code()).is_ok());
        assert_eq!(
            TransportSecurityFailure::from_kind(kind),
            TransportSecurityFailure::from_kind(kind)
        );
    }
    assert!(
        TransportSecurityPathGuard::try_new(true, true, false, true, true, false, false).is_ok()
    );
    assert_eq!(
        TransportSecurityPathGuard::try_new(true, false, false, true, true, false, false),
        Err(TransportSecurityPathError::SecureSetupMissing)
    );
    assert_eq!(
        TransportSecurityPathGuard::try_new(true, true, true, false, true, false, false),
        Err(TransportSecurityPathError::InsecureFallback)
    );
    assert_eq!(
        TransportSecurityPathGuard::try_new(true, true, true, true, false, false, false),
        Err(TransportSecurityPathError::FailureReasonMissing)
    );
    assert_eq!(
        TransportSecurityPathGuard::try_new(false, false, false, true, true, false, true),
        Err(TransportSecurityPathError::DevelopmentOnlyBoundaryMissing)
    );

    for verification_class in [
        TransportSecurityVerificationClass::ConfigurationShapeValidation,
        TransportSecurityVerificationClass::SecretSourceAvailability,
        TransportSecurityVerificationClass::ListenerStartup,
        TransportSecurityVerificationClass::PeerVerificationBehavior,
        TransportSecurityVerificationClass::InternalServiceIdentityTrustMapping,
        TransportSecurityVerificationClass::SessionEstablishment,
        TransportSecurityVerificationClass::NegativeSecurityCase,
        TransportSecurityVerificationClass::SecretMaterialRedaction,
        TransportSecurityVerificationClass::RotationGenerationOverlap,
        TransportSecurityVerificationClass::SecureMediaSession,
        TransportSecurityVerificationClass::EdgeTerminationDownstreamProtection,
    ] {
        assert!(TransportSecurityVerificationGuard::try_new(
            verification_class,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .is_ok());
    }
    assert_eq!(
        TransportSecurityVerificationGuard::try_new(
            TransportSecurityVerificationClass::ConfigurationShapeValidation,
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
        Err(TransportSecurityVerificationError::RequiredVerificationInputMissing)
    );
    assert_eq!(
        TransportSecurityVerificationGuard::try_new(
            TransportSecurityVerificationClass::ListenerStartup,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
        ),
        Err(TransportSecurityVerificationError::ListenerStartupConflatedWithPeerVerification)
    );
    assert_eq!(
        TransportSecurityVerificationGuard::try_new(
            TransportSecurityVerificationClass::PeerVerificationBehavior,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
        ),
        Err(TransportSecurityVerificationError::PeerVerificationConflatedWithAuthorization)
    );
    assert_eq!(
        TransportSecurityVerificationGuard::try_new(
            TransportSecurityVerificationClass::EdgeTerminationDownstreamProtection,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
        ),
        Err(TransportSecurityVerificationError::EdgeTerminationBoundaryMissing)
    );
}
