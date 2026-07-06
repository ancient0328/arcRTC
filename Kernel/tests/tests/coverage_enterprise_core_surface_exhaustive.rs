use arcrtc_core_command as command;
use arcrtc_core_configuration as configuration;
use arcrtc_core_cross_plane as cross_plane;
use arcrtc_core_domain as domain;
use arcrtc_core_features as features;
use arcrtc_core_identity::{
    AllocationId, ChannelBindId, ConfigurationScopeRef, CorrelationId, CredentialRef, EndpointId,
    OpaqueReference, PacketId, ParticipantId, PermissionId, ReferenceAuthority, RoomId, RouteId,
    SessionId, StartupRunId, StreamId,
};
use arcrtc_core_protocol as protocol;
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_runtime as runtime;
use arcrtc_core_security as security;
use arcrtc_core_signaling as signaling;
use arcrtc_core_time as time;
use arcrtc_core_transport as transport;

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("reference is valid")
}

fn correlation(value: &str) -> CorrelationId {
    CorrelationId::new(reference(value))
}

fn cataloged(code: &'static str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason code is cataloged")
}

fn startup(value: &str) -> StartupRunId {
    StartupRunId::new(reference(value))
}

fn room(value: &str) -> RoomId {
    RoomId::new(reference(value))
}

fn participant(value: &str) -> ParticipantId {
    ParticipantId::new(reference(value))
}

fn session(value: &str) -> SessionId {
    SessionId::new(reference(value))
}

fn endpoint(value: &str) -> EndpointId {
    EndpointId::new(reference(value))
}

fn stream(value: &str) -> StreamId {
    StreamId::new(reference(value))
}

fn allocation(value: &str) -> AllocationId {
    AllocationId::new(reference(value))
}

fn permission(value: &str) -> PermissionId {
    PermissionId::new(reference(value))
}

fn channel_bind(value: &str) -> ChannelBindId {
    ChannelBindId::new(reference(value))
}

fn credential(value: &str) -> CredentialRef {
    CredentialRef::new(reference(value))
}

fn packet(value: &str) -> PacketId {
    PacketId::new(reference(value))
}

fn config_scope(value: &str) -> ConfigurationScopeRef {
    ConfigurationScopeRef::new(reference(value))
}

fn command_decision(
    target_surface: command::TargetSurface,
    outcome: command::UseCaseOutcome,
    reason: command::DecisionReason<CatalogedReasonRef>,
) -> command::UseCaseDecision<CatalogedReasonRef> {
    command::UseCaseDecision::new(command::UseCaseDecisionInput {
        correlation_id: correlation("core-surface-decision"),
        command_type: command::CommandType::new("core-surface-command"),
        target_surface,
        outcome,
        reason,
        state_transition: command::StateTransitionSummary::NoStateChange,
        port_intents: Vec::new(),
        audit_projection: command::AuditProjectionRequirement::Required,
        evidence_class: command::DecisionEvidenceClass::SourceDecisionOnly,
    })
    .expect("decision shape is valid")
}

#[test]
fn configuration_domain_and_features_surfaces_execute_closed_vocabularies() {
    // private field を外から直接読まず、公開 constructor/accessor と Debug surface で境界を検証します。
    let boundary = configuration::NonCoreConfigurationBoundary::new(
        configuration::ConfigurationKind::DriverRuntime,
        configuration::ConfigurationOwner::Drivers,
        configuration::ConfigurationSourceClass::EnvironmentVariable,
    );
    assert_eq!(
        format!("{boundary:?}"),
        "NonCoreConfigurationBoundary { kind: DriverRuntime, owner: Drivers, source_class: EnvironmentVariable }"
    );
    assert_eq!(
        boundary.kind(),
        configuration::ConfigurationKind::DriverRuntime
    );
    assert_eq!(boundary.owner(), configuration::ConfigurationOwner::Drivers);

    for source in [
        configuration::ConfigurationSourceClass::TypedInput,
        configuration::ConfigurationSourceClass::EnvironmentVariable,
        configuration::ConfigurationSourceClass::File,
        configuration::ConfigurationSourceClass::ProcessArgs,
        configuration::ConfigurationSourceClass::OsSettings,
        configuration::ConfigurationSourceClass::CloudMetadata,
    ] {
        assert!(format!("{source:?}").len() > 2);
    }

    for allowed in [
        configuration::FeatureFlagAllowedEffect::SelectDriverImplementation,
        configuration::FeatureFlagAllowedEffect::EnableOptionalExporter,
        configuration::FeatureFlagAllowedEffect::ChooseExternalEncoding,
    ] {
        assert!(format!("{allowed:?}").contains(|c: char| c.is_ascii_alphabetic()));
    }

    for prohibited in [
        configuration::FeatureFlagProhibitedEffect::SignalingStateMachineChange,
        configuration::FeatureFlagProhibitedEffect::TurnLifecycleChange,
        configuration::FeatureFlagProhibitedEffect::SfuRoutingSemanticsChange,
        configuration::FeatureFlagProhibitedEffect::SecurityVerificationBypass,
        configuration::FeatureFlagProhibitedEffect::AuditRequirementBypass,
    ] {
        assert!(
            format!("{prohibited:?}").contains("Change")
                || format!("{prohibited:?}").contains("Bypass")
        );
    }

    for secret_boundary in [
        configuration::SecretBoundary::RawSecretOutsideCore,
        configuration::SecretBoundary::OpaqueCredentialReferenceOnly,
    ] {
        assert!(
            format!("{secret_boundary:?}").contains("Secret")
                || format!("{secret_boundary:?}").contains("Credential")
        );
    }

    for step in domain::ENTRYPOINTLICATION_USE_CASE_ORDER {
        assert!(matches!(
            step,
            domain::UseCaseStep::ReceiveCoreOwnedCommand
                | domain::UseCaseStep::VerifyCoreGuards
                | domain::UseCaseStep::DelegateDomainDecision
                | domain::UseCaseStep::ConnectDecisionReason
                | domain::UseCaseStep::EmitCoreEffects
        ));
    }

    let rejected = features::FeatureAdmissionDecision::new(
        None,
        features::ExcludedFeatureClass::RecordingWorkflow,
        features::FeatureRequestedSurface::Entrypoints,
        features::FeatureAdmissionDecisionClass::CloseNotClaimed,
        Some(features::FeatureAdmissionFailureKind::RecordingNotSupported),
    );
    assert_eq!(
        rejected.feature_class(),
        features::ExcludedFeatureClass::RecordingWorkflow
    );
    assert_eq!(
        rejected.requested_surface(),
        features::FeatureRequestedSurface::Entrypoints
    );
    assert_eq!(
        rejected.decision_class(),
        features::FeatureAdmissionDecisionClass::CloseNotClaimed
    );

    for requirement in [
        features::FutureAdmissionRequirement::FeatureClass,
        features::FutureAdmissionRequirement::OwnerPackageLayer,
        features::FutureAdmissionRequirement::GenericCoreRelation,
        features::FutureAdmissionRequirement::PublicSdkApiSurface,
        features::FutureAdmissionRequirement::DriverRuntimeDependencyBoundary,
        features::FutureAdmissionRequirement::SecurityPrivacyRedactionBoundary,
        features::FutureAdmissionRequirement::ReasonCatalogAdditions,
        features::FutureAdmissionRequirement::AuditEventRelation,
        features::FutureAdmissionRequirement::EvidenceClass,
        features::FutureAdmissionRequirement::MigrationDeprecationRelation,
    ] {
        assert!(format!("{requirement:?}").len() > 3);
    }
}

#[test]
fn signaling_transition_table_and_contract_errors_are_executed() {
    let subject = signaling::SignalingSubject::new(
        room("signaling-room"),
        Some(participant("signaling-participant")),
    );
    assert_eq!(subject.room_id().as_str(), "signaling-room");
    assert_eq!(
        subject
            .participant_id()
            .expect("participant exists")
            .as_str(),
        "signaling-participant"
    );

    let envelope = command::CommandEnvelope::new(
        correlation("signaling-command"),
        command::CommandType::new("join"),
        command::CommandVersion::new(1),
        command::TargetSurface::Signaling,
        subject.clone(),
    );
    let command = signaling::SignalingCommand::new(
        envelope,
        signaling::SignalingCommandKind::JoinRoom,
        "payload-ref",
    );
    assert_eq!(command.kind(), signaling::SignalingCommandKind::JoinRoom);
    assert_eq!(
        command.envelope().target_surface(),
        command::TargetSurface::Signaling
    );

    let event = signaling::SignalingEvent::new(
        correlation("signaling-event"),
        signaling::SignalingEventKind::ProtocolViolation,
        subject,
        "event-payload-ref",
    );
    assert_eq!(
        event.kind(),
        signaling::SignalingEventKind::ProtocolViolation
    );

    for kind in [
        signaling::SignalingCommandKind::JoinRoom,
        signaling::SignalingCommandKind::LeaveRoom,
        signaling::SignalingCommandKind::SendOffer,
        signaling::SignalingCommandKind::SendAnswer,
        signaling::SignalingCommandKind::SendIceCandidate,
        signaling::SignalingCommandKind::RequestTurnCredential,
        signaling::SignalingCommandKind::AcknowledgeForward,
    ] {
        assert!(format!("{kind:?}").len() > 3);
    }

    for kind in [
        signaling::SignalingEventKind::Joined,
        signaling::SignalingEventKind::Rejected,
        signaling::SignalingEventKind::ParticipantJoined,
        signaling::SignalingEventKind::ParticipantLeft,
        signaling::SignalingEventKind::OfferReceived,
        signaling::SignalingEventKind::AnswerReceived,
        signaling::SignalingEventKind::IceCandidateReceived,
        signaling::SignalingEventKind::TurnCredentialAvailable,
        signaling::SignalingEventKind::ProtocolViolation,
    ] {
        assert!(format!("{kind:?}").len() > 3);
    }

    let signaling_decision = signaling::SignalingDecision::new(command_decision(
        command::TargetSurface::Signaling,
        command::UseCaseOutcome::Accepted,
        command::DecisionReason::Absent,
    ))
    .expect("signaling target is accepted");
    assert_eq!(
        signaling_decision.decision().target_surface(),
        command::TargetSurface::Signaling
    );
    assert_eq!(
        signaling::SignalingDecision::new(command_decision(
            command::TargetSurface::Sfu,
            command::UseCaseOutcome::Accepted,
            command::DecisionReason::Absent,
        )),
        Err(signaling::SignalingContractError::WrongTargetSurface)
    );

    let absent_reason: command::DecisionReason<CatalogedReasonRef> =
        command::DecisionReason::Absent;
    let cataloged_reason = command::DecisionReason::Cataloged(cataloged("room_closed"));
    assert_eq!(
        signaling::SignalingContractError::from_decision_reason_error(&absent_reason),
        None
    );
    assert_eq!(
        signaling::SignalingContractError::from_decision_reason_error(&cataloged_reason),
        None
    );

    for failure in [
        signaling::SignalingFailureKind::MissingCorrelationId,
        signaling::SignalingFailureKind::MalformedCommand,
        signaling::SignalingFailureKind::TokenVerificationFailed,
        signaling::SignalingFailureKind::RoomNotAcceptingJoin,
        signaling::SignalingFailureKind::RoomCapacityExceeded,
        signaling::SignalingFailureKind::AdmissionCapacityExceeded,
        signaling::SignalingFailureKind::RoomLifetimeExceeded,
        signaling::SignalingFailureKind::RoomDraining,
        signaling::SignalingFailureKind::RoomClosed,
        signaling::SignalingFailureKind::RoomCloseNotAllowed,
        signaling::SignalingFailureKind::ParticipantNotJoined,
        signaling::SignalingFailureKind::ParticipantRejected,
        signaling::SignalingFailureKind::DuplicateCommand,
        signaling::SignalingFailureKind::IdempotencyPayloadMismatch,
        signaling::SignalingFailureKind::IdempotencyWindowExpired,
        signaling::SignalingFailureKind::AuthorizationContextMissing,
        signaling::SignalingFailureKind::AuthorizationContextInvalid,
        signaling::SignalingFailureKind::AuthorizationPolicyDenied,
        signaling::SignalingFailureKind::CommandOrderViolation,
        signaling::SignalingFailureKind::SignalingCommandQueueBoundExceeded,
        signaling::SignalingFailureKind::UnsupportedCommandVersion,
    ] {
        assert_eq!(
            cataloged(failure.reason_code())
                .definition()
                .code()
                .as_str(),
            failure.reason_code()
        );
    }

    for prohibited in [
        signaling::ProhibitedSignalingSemantic::MedicalRole,
        signaling::ProhibitedSignalingSemantic::ApplicationWorkflowState,
        signaling::ProhibitedSignalingSemantic::ChatSemantics,
        signaling::ProhibitedSignalingSemantic::RecordingSemantics,
        signaling::ProhibitedSignalingSemantic::ScreenShareSemantics,
        signaling::ProhibitedSignalingSemantic::DataChannelApplicationSemantics,
        signaling::ProhibitedSignalingSemantic::UiEndUserWorkflow,
        signaling::ProhibitedSignalingSemantic::UserAuthenticationIssuance,
        signaling::ProhibitedSignalingSemantic::RegulatedPayload,
    ] {
        assert!(format!("{prohibited:?}").len() > 3);
    }

    for rule in signaling::SIGNALING_TRANSITION_RULES {
        assert!(format!("{:?}", rule.trigger()).len() > 3);
        assert!(rule.allowed_room_states().iter().all(|state| matches!(
            state,
            signaling::RoomState::RoomAbsent
                | signaling::RoomState::RoomOpen
                | signaling::RoomState::RoomDraining
                | signaling::RoomState::RoomClosed
        )));
        assert!(rule
            .allowed_participant_states()
            .iter()
            .all(|state| matches!(
                state,
                signaling::ParticipantState::ParticipantNew
                    | signaling::ParticipantState::ParticipantVerifying
                    | signaling::ParticipantState::ParticipantJoined
                    | signaling::ParticipantState::ParticipantLeaving
                    | signaling::ParticipantState::ParticipantLeft
                    | signaling::ParticipantState::ParticipantRejected
            )));
        let _ = rule.success_room_state();
        let _ = rule.success_participant_state();
        assert!(!rule.reject_reasons().is_empty());
    }
}

#[test]
fn protocol_versioning_digest_and_semantic_envelope_fail_closed_paths_execute() {
    let fully_defined = protocol::CanonicalEncodingRuleSet::new(
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::UnknownFieldHandling::Reject,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
    );
    assert!(fully_defined.usable_for_canonical_evidence());
    let incomplete = protocol::CanonicalEncodingRuleSet::new(
        protocol::CanonicalRuleStatus::RequiresAdrOrCanonical,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::UnknownFieldHandling::IgnoredOnlyWhenCompatibilityAllows,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
    );
    assert!(!incomplete.usable_for_canonical_evidence());

    for data_class in [
        protocol::CanonicalDataClass::AuditEventHashInput,
        protocol::CanonicalDataClass::CoreReason,
        protocol::CanonicalDataClass::CoreReference,
        protocol::CanonicalDataClass::EvidenceTimestamp,
        protocol::CanonicalDataClass::Duration,
        protocol::CanonicalDataClass::BinaryPayloadDigest,
    ] {
        assert!(format!("{data_class:?}").len() > 3);
    }

    let format_version = protocol::CanonicalFormatVersion::new("json-canon", "1");
    let digest =
        protocol::CanonicalDigest::new(format_version, "sha256", vec![1, 2, 3]).expect("digest");
    assert_eq!(digest.format_version().format(), "json-canon");
    assert_eq!(digest.format_version().version(), "1");
    assert_eq!(digest.algorithm(), "sha256");
    assert_eq!(digest.digest(), &[1, 2, 3]);
    assert_eq!(
        protocol::CanonicalDigest::new(format_version, "", vec![1]),
        Err(protocol::CanonicalEncodingError::EmptyDigestMaterial)
    );
    assert_eq!(
        protocol::CanonicalDigest::new(format_version, "sha256", Vec::new()),
        Err(protocol::CanonicalEncodingError::EmptyDigestMaterial)
    );

    for failure in [
        protocol::CanonicalEncodingFailureKind::CanonicalSerializationFailed,
        protocol::CanonicalEncodingFailureKind::CanonicalSerializationMismatch,
        protocol::CanonicalEncodingFailureKind::ExternalDecodeFailed,
        protocol::CanonicalEncodingFailureKind::ExternalEncodeFailed,
        protocol::CanonicalEncodingFailureKind::MissingRequiredWireField,
        protocol::CanonicalEncodingFailureKind::UnsupportedCanonicalVersion,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for failure in [
        protocol::CompatibilityFailureKind::UnsupportedCommandVersion,
        protocol::CompatibilityFailureKind::UnsupportedMediaContractVersion,
        protocol::CompatibilityFailureKind::UnsupportedTurnContractVersion,
        protocol::CompatibilityFailureKind::UnsupportedDriverWireVersion,
        protocol::CompatibilityFailureKind::MissingRequiredWireField,
        protocol::CompatibilityFailureKind::ExternalEnumUnmapped,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    let v1 = protocol::ContractVersion::new(1, 0, 0);
    let v1_1 = protocol::ContractVersion::new(1, 1, 0);
    assert_eq!(v1.major(), 1);
    for outcome in [
        protocol::VersionNegotiationOutcome::AcceptedExact(v1),
        protocol::VersionNegotiationOutcome::AcceptedCompatible {
            requested: v1,
            accepted: v1_1,
        },
        protocol::VersionNegotiationOutcome::RejectedUnsupported(v1_1),
    ] {
        assert!(
            format!("{outcome:?}").contains("Accepted")
                || format!("{outcome:?}").contains("Rejected")
        );
    }

    for ownership in [
        protocol::VersionedSurfaceOwnership::new(
            protocol::VersionedSurface::SignalingContract,
            protocol::VersionOwner::Core,
        ),
        protocol::VersionedSurfaceOwnership::new(
            protocol::VersionedSurface::DriverWireEncoding,
            protocol::VersionOwner::Driver,
        ),
        protocol::VersionedSurfaceOwnership::new(
            protocol::VersionedSurface::SdkPublicContract,
            protocol::VersionOwner::Sdk,
        ),
    ] {
        assert!(format!("{:?}", ownership.surface()).len() > 3);
        assert!(!format!("{:?}", ownership.owner()).is_empty());
    }

    for change in [
        protocol::CompatibilityChange::AddOptionalFieldWithDefault,
        protocol::CompatibilityChange::AddRequiredField,
        protocol::CompatibilityChange::RemoveField,
        protocol::CompatibilityChange::ChangeReasonCodeSemantics,
        protocol::CompatibilityChange::AddReasonCode,
        protocol::CompatibilityChange::ChangeStateTransition,
        protocol::CompatibilityChange::AddDriverEncoding,
    ] {
        assert!(matches!(
            change.classification(),
            protocol::CompatibilityClassification::Compatible
                | protocol::CompatibilityClassification::Breaking
                | protocol::CompatibilityClassification::Conditional
                | protocol::CompatibilityClassification::Prohibited
        ));
    }

    let range =
        protocol::CompatibilityVersionRange::new(protocol::VersionedSurface::SfuContract, v1, v1_1);
    assert_eq!(range.surface(), protocol::VersionedSurface::SfuContract);
    let deprecation = protocol::DeprecationDecision::new(
        protocol::VersionedSurface::SfuContract,
        v1,
        protocol::VersionOwner::Core,
        range,
        vec![
            protocol::DeprecationLifecycleStep::IdentifyAffectedSurfaceAndVersion,
            protocol::DeprecationLifecycleStep::RecordAdrOrCanonicalUpdate,
            protocol::DeprecationLifecycleStep::DefineUnsupportedVersionBehavior,
            protocol::DeprecationLifecycleStep::UpdateSdkParityAndDriverMapping,
            protocol::DeprecationLifecycleStep::AddCompatibilityNegativePlan,
            protocol::DeprecationLifecycleStep::RecordExecutionEvidence,
            protocol::DeprecationLifecycleStep::RemoveAfterDocumentedCondition,
        ],
    );
    assert_eq!(deprecation.lifecycle_steps().len(), 7);

    let payload = protocol::CoreSemanticPayloadModel::new(
        protocol::CoreSemanticPayloadClass::OpaqueCoreReference,
        Some(reference("semantic-reference")),
    );
    let payload_debug = format!("{payload:?}");
    assert!(payload_debug.contains("value_len"));
    assert!(!payload_debug.contains("semantic-reference"));
    let reason = cataloged("room_closed");
    assert_eq!(
        protocol::CoreSemanticEnvelope::try_new(
            command::TargetSurface::Signaling,
            v1,
            correlation("semantic-envelope-missing"),
            protocol::SemanticEnvelopeMessageKind::Command,
            protocol::CoreSemanticMessageType::SignalingCommand(
                protocol::SignalingSemanticCommandType::JoinRoom
            ),
            true,
            Some(payload.clone()),
            Some(command::UseCaseOutcome::Rejected),
            None,
        ),
        Err(protocol::SemanticEnvelopeError::RequiredReasonMissing)
    );
    assert_eq!(
        protocol::CoreSemanticEnvelope::try_new(
            command::TargetSurface::Signaling,
            v1,
            correlation("semantic-envelope-fake"),
            protocol::SemanticEnvelopeMessageKind::Event,
            protocol::CoreSemanticMessageType::SignalingEvent(
                protocol::SignalingSemanticEventType::Joined
            ),
            true,
            Some(payload.clone()),
            Some(command::UseCaseOutcome::Accepted),
            Some(reason),
        ),
        Err(protocol::SemanticEnvelopeError::SuccessReasonMustNotBeInvented)
    );
    assert!(protocol::CoreSemanticEnvelope::try_new(
        command::TargetSurface::Signaling,
        v1,
        correlation("semantic-envelope-ok"),
        protocol::SemanticEnvelopeMessageKind::Event,
        protocol::CoreSemanticMessageType::SignalingEvent(
            protocol::SignalingSemanticEventType::Rejected
        ),
        true,
        Some(payload),
        Some(command::UseCaseOutcome::Rejected),
        Some(reason),
    )
    .is_ok());

    for message_type in [
        protocol::CoreSemanticMessageType::SignalingCommand(
            protocol::SignalingSemanticCommandType::LeaveRoom,
        ),
        protocol::CoreSemanticMessageType::SignalingEvent(
            protocol::SignalingSemanticEventType::ProtocolViolation,
        ),
        protocol::CoreSemanticMessageType::SfuModel(protocol::SfuSemanticModelType::SfuSession),
        protocol::CoreSemanticMessageType::SfuDecision(
            protocol::SfuSemanticDecisionType::Forwarding,
        ),
        protocol::CoreSemanticMessageType::TurnModel(protocol::TurnSemanticModelType::Allocation),
        protocol::CoreSemanticMessageType::TurnDecision(protocol::TurnSemanticDecisionType::Relay),
        protocol::CoreSemanticMessageType::TransportCommand(
            protocol::TransportSemanticCommandType::ForwardPacket,
        ),
        protocol::CoreSemanticMessageType::TransportEvent(
            protocol::TransportSemanticEventType::ConversionFailed,
        ),
        protocol::CoreSemanticMessageType::DriverErrorConverted,
    ] {
        assert!(format!("{message_type:?}").len() > 3);
    }
}

#[test]
fn transport_negotiation_ice_and_secure_media_surfaces_execute() {
    assert_eq!(
        format!("{:?}", transport::CoreTransportSurface),
        "CoreTransportSurface"
    );
    let capability = transport::TransportCapability::new("ice-lite");
    assert_eq!(capability.name(), "ice-lite");
    let sdp = transport::SessionDescriptionRef::new(session("transport-session"), "sdp-ref")
        .expect("semantic reference");
    assert_eq!(
        transport::SessionDescriptionRef::new(session("transport-session"), ""),
        Err(transport::TransportContractError::InvalidSemanticReference)
    );
    let ice = transport::IceCandidateRef::new(session("transport-session"), "ice-ref")
        .expect("semantic reference");
    assert_eq!(
        transport::IceCandidateRef::new(session("transport-session"), "bad\ncandidate"),
        Err(transport::TransportContractError::InvalidSemanticReference)
    );
    let packet_view = transport::PacketSemanticViewRef::new(packet("transport-packet"));

    for payload in [
        transport::WebRtcTransportPayload::Empty,
        transport::WebRtcTransportPayload::SessionDescription(sdp.clone()),
        transport::WebRtcTransportPayload::IceCandidate(ice.clone()),
        transport::WebRtcTransportPayload::PacketView(packet_view.clone()),
        transport::WebRtcTransportPayload::Capability(capability),
    ] {
        let command = transport::WebRtcTransportInput::new(
            transport::TransportCommandKind::ForwardPacket,
            payload,
        );
        assert!(format!("{command:?}").contains("ForwardPacket"));
    }

    for failure_kind in [
        transport::TransportDriverFailureKind::ExternalDecodeFailed,
        transport::TransportDriverFailureKind::ExternalEncodeFailed,
        transport::TransportDriverFailureKind::MediaPayloadMappingInvalid,
        transport::TransportDriverFailureKind::DriverShutdown,
    ] {
        let failure = transport::TransportDriverFailure::from_kind(failure_kind);
        assert_eq!(failure.kind(), failure_kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            failure_kind.reason_code()
        );
    }

    for observation in [
        transport::WebRtcTransportObservation::Empty,
        transport::WebRtcTransportObservation::SessionDescription(sdp),
        transport::WebRtcTransportObservation::IceCandidate(ice),
        transport::WebRtcTransportObservation::PacketView(packet_view),
        transport::WebRtcTransportObservation::ConversionFailure(
            transport::TransportDriverFailure::from_kind(
                transport::TransportDriverFailureKind::ExternalDecodeFailed,
            ),
        ),
    ] {
        let event = transport::WebRtcTransportOutput::new(
            transport::TransportEventKind::ConversionFailed,
            observation,
        );
        assert!(format!("{event:?}").contains("ConversionFailed"));
    }

    for kind in [
        transport::TransportCommandKind::StartSession,
        transport::TransportCommandKind::ApplyLocalDescription,
        transport::TransportCommandKind::ApplyRemoteDescription,
        transport::TransportCommandKind::AddIceCandidate,
        transport::TransportCommandKind::ForwardPacket,
        transport::TransportCommandKind::CloseSession,
    ] {
        assert!(format!("{kind:?}").len() > 3);
    }
    for kind in [
        transport::TransportEventKind::SessionObserved,
        transport::TransportEventKind::LocalDescriptionAccepted,
        transport::TransportEventKind::RemoteDescriptionAccepted,
        transport::TransportEventKind::IceCandidateObserved,
        transport::TransportEventKind::PacketSemanticViewObserved,
        transport::TransportEventKind::TransportClosed,
        transport::TransportEventKind::ConversionFailed,
    ] {
        assert!(format!("{kind:?}").len() > 3);
    }

    for failure in [
        transport::TransportFailureKind::UnsupportedMediaContractVersion,
        transport::TransportFailureKind::ExternalDecodeFailed,
        transport::TransportFailureKind::ExternalEncodeFailed,
        transport::TransportFailureKind::MediaPayloadMappingInvalid,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for failure in [
        transport::NegotiationFailureKind::ExternalDecodeFailed,
        transport::NegotiationFailureKind::IceCandidatePolicyViolation,
        transport::NegotiationFailureKind::IceConnectivityCheckFailed,
        transport::NegotiationFailureKind::IceConsentExpired,
        transport::NegotiationFailureKind::MissingRequiredWireField,
        transport::NegotiationFailureKind::ExternalEnumUnmapped,
        transport::NegotiationFailureKind::UnsupportedCommandVersion,
        transport::NegotiationFailureKind::UnsupportedMediaContractVersion,
        transport::NegotiationFailureKind::MediaCodecNotSupported,
        transport::NegotiationFailureKind::MediaPayloadMappingInvalid,
        transport::NegotiationFailureKind::ParticipantNotJoined,
        transport::NegotiationFailureKind::CommandOrderViolation,
        transport::NegotiationFailureKind::RoomDraining,
        transport::NegotiationFailureKind::RoomClosed,
        transport::NegotiationFailureKind::NetworkSendFailed,
        transport::NegotiationFailureKind::NetworkReceiveFailed,
        transport::NegotiationFailureKind::DriverShutdown,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    let ice_policy = transport::IceCandidatePolicy::new(
        vec![
            transport::IceCandidateConnectivityClass::HostCandidateRef,
            transport::IceCandidateConnectivityClass::SrflxCandidateRef,
            transport::IceCandidateConnectivityClass::RelayCandidateRef,
            transport::IceCandidateConnectivityClass::MdnsCandidateRef,
            transport::IceCandidateConnectivityClass::TrickleCandidateRef,
            transport::IceCandidateConnectivityClass::IceRestartIntent,
            transport::IceCandidateConnectivityClass::ConnectivityObservation,
            transport::IceCandidateConnectivityClass::ConsentFreshnessObservation,
        ],
        transport::IceAddressExposurePolicy::RelayOnly,
        true,
    );
    assert_eq!(ice_policy.accepted_classes().len(), 8);
    for policy in [
        transport::IceAddressExposurePolicy::RelayOnly,
        transport::IceAddressExposurePolicy::HostAllowed,
        transport::IceAddressExposurePolicy::SrflxAllowed,
        transport::IceAddressExposurePolicy::MdnsObfuscationRequired,
        transport::IceAddressExposurePolicy::RawAddressRedactionRequired,
    ] {
        assert!(format!("{policy:?}").len() > 3);
    }
    for meaning in [
        transport::IceObservationMeaning::DiagnosticEvidenceOnly,
        transport::IceObservationMeaning::NotSignalingRelaySuccess,
        transport::IceObservationMeaning::NotCrossPlaneBinding,
    ] {
        assert!(format!("{meaning:?}").len() > 3);
    }
    for failure in [
        transport::IceFailureKind::IceCandidatePolicyViolation,
        transport::IceFailureKind::IceCandidateMappingInvalid,
        transport::IceFailureKind::IceCandidateRedactionRequired,
        transport::IceFailureKind::IceGatheringFailed,
        transport::IceFailureKind::IceConnectivityCheckFailed,
        transport::IceFailureKind::IceConsentExpired,
        transport::IceFailureKind::IceRestartNotAllowed,
        transport::IceFailureKind::ExternalDecodeFailed,
        transport::IceFailureKind::CommandOrderViolation,
        transport::IceFailureKind::NetworkReceiveFailed,
        transport::IceFailureKind::NetworkSendFailed,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    let secure_policy = transport::SecureMediaPolicy::new("v1", "dtls-srtp", true, true);
    assert!(format!("{secure_policy:?}").contains("dtls-srtp"));
    for class in [
        transport::SecureMediaSessionClass::SecureMediaRequired,
        transport::SecureMediaSessionClass::DtlsHandshakeObserved,
        transport::SecureMediaSessionClass::PeerVerificationObserved,
        transport::SecureMediaSessionClass::SrtpProtectionActive,
        transport::SecureMediaSessionClass::RekeyRequired,
        transport::SecureMediaSessionClass::SessionClosed,
    ] {
        assert!(format!("{class:?}").len() > 3);
    }
    for evidence in [
        transport::SecureMediaEvidenceClass::Handshake,
        transport::SecureMediaEvidenceClass::PeerVerification,
        transport::SecureMediaEvidenceClass::ProtectionState,
        transport::SecureMediaEvidenceClass::PacketForwardingScope,
    ] {
        assert!(format!("{evidence:?}").len() > 3);
    }
    for failure in [
        transport::SecureMediaFailureKind::SecureMediaProfileNotSupported,
        transport::SecureMediaFailureKind::SecureMediaHandshakeFailed,
        transport::SecureMediaFailureKind::SecureMediaPeerVerificationFailed,
        transport::SecureMediaFailureKind::SecureMediaProtectionNotActive,
        transport::SecureMediaFailureKind::SecureMediaKeyStateInvalid,
        transport::SecureMediaFailureKind::SecureMediaSessionExpired,
        transport::SecureMediaFailureKind::SecureMediaRekeyRequired,
        transport::SecureMediaFailureKind::SecretUnavailable,
        transport::SecureMediaFailureKind::SecretRotationStateUnavailable,
        transport::SecureMediaFailureKind::SecretKeyRevoked,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
}

#[test]
fn runtime_task_lifecycle_and_time_policy_fail_closed_paths_execute() {
    for surface in [
        runtime::RuntimeAbstractionSurface::ClockPort,
        runtime::RuntimeAbstractionSurface::RandomPort,
        runtime::RuntimeAbstractionSurface::RuntimePort,
    ] {
        assert!(format!("{:?}", surface.ownership()).contains("owns"));
    }

    for target in [
        runtime::TimePolicyTarget::TurnAllocationExpiry,
        runtime::TimePolicyTarget::TurnPermissionExpiry,
        runtime::TimePolicyTarget::TurnChannelBindExpiry,
        runtime::TimePolicyTarget::SignalingRoomLifecycle,
        runtime::TimePolicyTarget::ResourceRetention,
        runtime::TimePolicyTarget::ConfigurationStartupTimeout,
        runtime::TimePolicyTarget::CommandDeadline,
    ] {
        assert!(CatalogedReasonRef::from_code(target.failure_reason()).is_ok());
    }

    for use_class in [
        runtime::RandomnessUseClass::Nonce,
        runtime::RandomnessUseClass::OpaqueId,
        runtime::RandomnessUseClass::Challenge,
        runtime::RandomnessUseClass::ReferenceStability,
    ] {
        assert!(format!("{:?}", runtime::RandomnessContract::opaque(use_class)).contains("true"));
    }

    let selection = runtime::RuntimeConfigurationSelection::new(
        startup("runtime-startup"),
        config_scope("runtime-config"),
        runtime::RuntimeAbstractionSurface::RuntimePort,
    );
    assert!(format!("{selection:?}").contains("RuntimePort"));

    for failure in [
        runtime::RuntimeFailureKind::RuntimeConfigMissing,
        runtime::RuntimeFailureKind::RuntimeConfigInvalid,
        runtime::RuntimeFailureKind::RuntimeTaskClassNotAdmitted,
        runtime::RuntimeFailureKind::RuntimeTaskOwnerViolation,
        runtime::RuntimeFailureKind::RuntimeTaskSupervisionMissing,
        runtime::RuntimeFailureKind::RuntimeTaskSpawnFailed,
        runtime::RuntimeFailureKind::RuntimeTaskJoinFailed,
        runtime::RuntimeFailureKind::RuntimeTaskCancelFailed,
        runtime::RuntimeFailureKind::RuntimeTaskPanicDetected,
        runtime::RuntimeFailureKind::DriverShutdown,
        runtime::RuntimeFailureKind::RuntimeTaskQueueBoundExceeded,
        runtime::RuntimeFailureKind::MemoryPressureExceeded,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    for concern in [
        runtime::RuntimeTaskConcern::PriorDomainDecisionReference,
        runtime::RuntimeTaskConcern::RuntimePortTaskContract,
        runtime::RuntimeTaskConcern::ConcreteTaskHandle,
        runtime::RuntimeTaskConcern::DriverIoWorker,
        runtime::RuntimeTaskConcern::EntrypointsSupervisionTask,
        runtime::RuntimeTaskConcern::TaskQueueMailbox,
        runtime::RuntimeTaskConcern::TaskPanicObservation,
        runtime::RuntimeTaskConcern::TaskCancellation,
    ] {
        assert!(format!("{:?}", concern.owner()).len() > 3);
    }

    for task_class in [
        runtime::RuntimeTaskClass::NoRuntimeTask,
        runtime::RuntimeTaskClass::DriverIoWorker,
        runtime::RuntimeTaskClass::DriverPacketWorker,
        runtime::RuntimeTaskClass::DriverSinkWorker,
        runtime::RuntimeTaskClass::EntrypointsSupervisionTask,
        runtime::RuntimeTaskClass::RuntimeTimerTask,
        runtime::RuntimeTaskClass::TestRuntimeTask,
    ] {
        assert!(!task_class.is_rejected_class());
    }
    assert!(runtime::RuntimeTaskClass::DetachedTaskRequested.is_rejected_class());

    assert_eq!(
        runtime::RuntimeTaskLifecyclePolicy::try_new(
            runtime::RuntimeTaskClass::DetachedTaskRequested,
            None,
            runtime::RuntimeTaskOwningLayer::Entrypoints,
            runtime::RuntimeTaskInputReferenceClass::None,
            runtime::RuntimeTaskOutputObservation::NoTaskEvidenceClaim,
            runtime::CancellationPropagationRule::NotApplicable,
            false,
            false,
        ),
        Err(runtime::RuntimeTaskLifecyclePolicyError::DetachedTaskNotAdmitted)
    );
    assert_eq!(
        runtime::RuntimeTaskLifecyclePolicy::try_new(
            runtime::RuntimeTaskClass::DriverIoWorker,
            None,
            runtime::RuntimeTaskOwningLayer::Driver,
            runtime::RuntimeTaskInputReferenceClass::OpaqueTaskReference,
            runtime::RuntimeTaskOutputObservation::SpawnObserved,
            runtime::CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
            true,
            true,
        ),
        Err(runtime::RuntimeTaskLifecyclePolicyError::SupervisionScopeMissing)
    );
    assert_eq!(
        runtime::RuntimeTaskLifecyclePolicy::try_new(
            runtime::RuntimeTaskClass::DriverIoWorker,
            Some(runtime::SupervisionScope::DriverComponent),
            runtime::RuntimeTaskOwningLayer::Driver,
            runtime::RuntimeTaskInputReferenceClass::OpaqueTaskReference,
            runtime::RuntimeTaskOutputObservation::SpawnObserved,
            runtime::CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
            false,
            true,
        ),
        Err(runtime::RuntimeTaskLifecyclePolicyError::JoinWaitBoundMissing)
    );
    assert_eq!(
        runtime::RuntimeTaskLifecyclePolicy::try_new(
            runtime::RuntimeTaskClass::DriverPacketWorker,
            Some(runtime::SupervisionScope::DriverComponent),
            runtime::RuntimeTaskOwningLayer::Driver,
            runtime::RuntimeTaskInputReferenceClass::CommandCorrelation,
            runtime::RuntimeTaskOutputObservation::FailureObserved,
            runtime::CancellationPropagationRule::FollowsShutdownDrain,
            true,
            false,
        ),
        Err(runtime::RuntimeTaskLifecyclePolicyError::QueueMailboxBoundMissing)
    );
    let policy = runtime::RuntimeTaskLifecyclePolicy::try_new(
        runtime::RuntimeTaskClass::DriverSinkWorker,
        Some(runtime::SupervisionScope::DriverComponent),
        runtime::RuntimeTaskOwningLayer::Driver,
        runtime::RuntimeTaskInputReferenceClass::OpaqueTaskReference,
        runtime::RuntimeTaskOutputObservation::JoinObserved,
        runtime::CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
        true,
        true,
    )
    .expect("bounded driver worker policy");
    assert!(format!("{policy:?}").contains("runtime_task_lifecycle_decision"));

    assert_eq!(
        runtime::RuntimeTaskLifecycleDecision::try_new(
            startup("runtime-decision"),
            Some(correlation("runtime-decision-correlation")),
            policy,
            Some("task-ref"),
            runtime::RuntimeTaskLifecycleOutcome::Accepted,
            Some(runtime::RuntimeFailureKind::RuntimeTaskSpawnFailed),
        ),
        Err(runtime::RuntimeTaskLifecycleDecisionError::ReasonMustBeAbsentForSuccess)
    );
    assert_eq!(
        runtime::RuntimeTaskLifecycleDecision::try_new(
            startup("runtime-decision"),
            Some(correlation("runtime-decision-correlation")),
            policy,
            Some("task-ref"),
            runtime::RuntimeTaskLifecycleOutcome::Failed,
            None,
        ),
        Err(runtime::RuntimeTaskLifecycleDecisionError::ReasonRequired)
    );
    let decision = runtime::RuntimeTaskLifecycleDecision::try_new(
        startup("runtime-decision"),
        Some(correlation("runtime-decision-correlation")),
        policy,
        Some("task-ref"),
        runtime::RuntimeTaskLifecycleOutcome::Failed,
        Some(runtime::RuntimeFailureKind::RuntimeTaskJoinFailed),
    )
    .expect("failed lifecycle decision carries reason");
    assert_eq!(
        decision.audit_event_type(),
        "runtime_task_lifecycle_decision"
    );

    for surface in [
        runtime::TaskCancellationSurface::BeforeDriverCoreConversion,
        runtime::TaskCancellationSurface::AfterCommandEnteredCore,
        runtime::TaskCancellationSurface::DuringShutdownDrain,
        runtime::TaskCancellationSurface::DuringDriverQueueCacheExecution,
        runtime::TaskCancellationSurface::DuringTestHarnessTimeout,
    ] {
        assert!(surface.required_relation().len() > 10);
    }

    for quantity in [
        time::NormalizedQuantity::Duration,
        time::NormalizedQuantity::Timestamp,
        time::NormalizedQuantity::Bytes,
        time::NormalizedQuantity::PacketCount,
        time::NormalizedQuantity::Rate,
        time::NormalizedQuantity::Ratio,
        time::NormalizedQuantity::Bitrate,
        time::NormalizedQuantity::JitterRtt,
    ] {
        assert!(format!("{:?}", quantity.default_unit()).len() > 3);
    }
    assert_eq!(
        time::NormalizedValue::rational(1, 0),
        Err(time::NormalizedValueError::ZeroDenominator)
    );
    assert!(time::NormalizedValue::rational(1, 3).is_ok());
    let measurement = time::NormalizedMeasurement::new(
        time::NormalizedQuantity::Rate,
        time::NormalizedUnit::UnitsPerSecond,
        time::NormalizedValue::Integer(42),
        time::PrecisionClass::IntegerExact,
        time::RoundingDirection::Exact,
        time::ComparisonOperator::LessThanOrEqual,
        time::BoundaryInclusivity::Inclusive,
        time::SamplingWindow::PerSecond,
        time::RawMeasurementOwner::Driver,
        time::PolicyDecisionOwner::Core,
    );
    assert!(format!("{measurement:?}").contains("UnitsPerSecond"));

    for failure in [
        time::TimeNormalizationFailureKind::MeasurementNormalizationFailed,
        time::TimeNormalizationFailureKind::TimeObservationUnavailable,
        time::TimeNormalizationFailureKind::OperationDeadlineExceeded,
        time::TimeNormalizationFailureKind::RetentionDurationExceeded,
        time::TimeNormalizationFailureKind::RuntimeConfigInvalid,
        time::TimeNormalizationFailureKind::DriverShutdown,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for failure in [
        time::TimeSynchronizationFailureKind::ClockSkewExceeded,
        time::TimeSynchronizationFailureKind::TimeSourceUntrusted,
        time::TimeSynchronizationFailureKind::TimeSyncUnavailable,
        time::TimeSynchronizationFailureKind::TimestampOrderUntrusted,
        time::TimeSynchronizationFailureKind::TimeObservationUnavailable,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    assert_eq!(
        time::ClockSkewPolicy::try_new(
            time::TimeNodeScope::MultiNode,
            time::TimeTrustClass::MultiNodeBoundedSkew,
            None,
            time::PrecisionClass::IntegerExact,
            time::SamplingWindow::Milliseconds(100),
            time::TrustedTimeSourceClass::ExternalTimeSourceObservation,
            time::ClockSkewImpact::Ordering,
        ),
        Err(time::ClockSkewPolicyError::MaxSkewRequired)
    );
    assert_eq!(
        time::ClockSkewPolicy::try_new(
            time::TimeNodeScope::MultiProcess,
            time::TimeTrustClass::SingleProcessMonotonic,
            Some(5),
            time::PrecisionClass::IntegerExact,
            time::SamplingWindow::Milliseconds(100),
            time::TrustedTimeSourceClass::LocalMonotonicClock,
            time::ClockSkewImpact::Expiry,
        ),
        Err(time::ClockSkewPolicyError::TrustClassCannotSupportScope)
    );
    assert_eq!(
        time::ClockSkewPolicy::try_new(
            time::TimeNodeScope::SingleNode,
            time::TimeTrustClass::ExternalTrustedTimeSource,
            Some(5),
            time::PrecisionClass::DeclaredPrecisionLabel("ms"),
            time::SamplingWindow::PolicyLabel("startup-window"),
            time::TrustedTimeSourceClass::LocalWallClock,
            time::ClockSkewImpact::Evidence,
        ),
        Err(time::ClockSkewPolicyError::ExternalSourceClassRequired)
    );
    let skew_policy = time::ClockSkewPolicy::try_new(
        time::TimeNodeScope::MultiNode,
        time::TimeTrustClass::MultiNodeBoundedSkew,
        Some(5),
        time::PrecisionClass::IntegerExact,
        time::SamplingWindow::Milliseconds(100),
        time::TrustedTimeSourceClass::ExternalTimeSourceObservation,
        time::ClockSkewImpact::Audit,
    )
    .expect("bounded skew policy");
    assert!(time::TimeTrustClass::ExternalTrustedTimeSource.runtime_evidence_allowed());
    assert!(!time::TimeTrustClass::TimeUntrusted.runtime_evidence_allowed());
    assert!(time::TimeTrustClass::MultiNodeBoundedSkew.supports_cross_node_comparison());
    assert!(!time::TimeTrustClass::SingleNodeWallClock.supports_cross_node_comparison());

    assert_eq!(
        time::TimeSynchronizationDecision::try_new(
            startup("time-sync"),
            Some(correlation("time-sync-correlation")),
            skew_policy,
            time::ObservedSkewClass::WithinPolicy,
            time::TimeSynchronizationOutcome::Accepted,
            Some(time::TimeSynchronizationFailureKind::ClockSkewExceeded),
        ),
        Err(time::TimeSynchronizationDecisionError::ReasonMustBeAbsentForAccepted)
    );
    assert_eq!(
        time::TimeSynchronizationDecision::try_new(
            startup("time-sync"),
            Some(correlation("time-sync-correlation")),
            skew_policy,
            time::ObservedSkewClass::ObservationUnavailable,
            time::TimeSynchronizationOutcome::Rejected,
            None,
        ),
        Err(time::TimeSynchronizationDecisionError::ReasonRequired)
    );
    let time_decision = time::TimeSynchronizationDecision::try_new(
        startup("time-sync"),
        Some(correlation("time-sync-correlation")),
        skew_policy,
        time::ObservedSkewClass::WithinPolicy,
        time::TimeSynchronizationOutcome::Accepted,
        None,
    )
    .expect("accepted sync decision has no reason");
    assert_eq!(
        time_decision.audit_event_type(),
        "time_synchronization_decision"
    );
}

#[test]
fn cross_plane_binding_references_and_outcomes_are_executed() {
    for binding_class in [
        cross_plane::CrossPlaneBindingClass::NoCrossPlaneBindingRequired,
        cross_plane::CrossPlaneBindingClass::SignalingParticipantBinding,
        cross_plane::CrossPlaneBindingClass::SfuEndpointBinding,
        cross_plane::CrossPlaneBindingClass::TurnAllocationBinding,
        cross_plane::CrossPlaneBindingClass::TurnPermissionBinding,
        cross_plane::CrossPlaneBindingClass::IceCandidateBinding,
        cross_plane::CrossPlaneBindingClass::SecureMediaSessionBinding,
        cross_plane::CrossPlaneBindingClass::TestCrossPlaneBinding,
    ] {
        assert!(!binding_class.is_rejected_class());
    }
    assert!(cross_plane::CrossPlaneBindingClass::ImplicitBindingRequested.is_rejected_class());

    let references = [
        cross_plane::CrossPlaneReference::NotMaterialized,
        cross_plane::CrossPlaneReference::Room(room("cross-room")),
        cross_plane::CrossPlaneReference::Session(session("cross-session")),
        cross_plane::CrossPlaneReference::Participant(participant("cross-participant")),
        cross_plane::CrossPlaneReference::Endpoint(endpoint("cross-endpoint")),
        cross_plane::CrossPlaneReference::Stream(stream("cross-stream")),
        cross_plane::CrossPlaneReference::Allocation(allocation("cross-allocation")),
        cross_plane::CrossPlaneReference::Permission(permission("cross-permission")),
        cross_plane::CrossPlaneReference::ChannelBind(channel_bind("cross-channel-bind")),
        cross_plane::CrossPlaneReference::Credential(credential("cross-credential")),
    ];
    let lifecycles: Vec<_> = references
        .iter()
        .map(|reference| reference.lifecycle())
        .collect();
    assert!(lifecycles.contains(&cross_plane::BindingReferenceLifecycle::Room));
    assert!(lifecycles.contains(&cross_plane::BindingReferenceLifecycle::CredentialVerification));

    let policy = cross_plane::CrossPlaneBindingPolicy::new(
        cross_plane::CrossPlaneBindingClass::SfuEndpointBinding,
        cross_plane::CrossPlane::Signaling,
        cross_plane::CrossPlane::Sfu,
        cross_plane::CrossPlaneReference::Participant(participant("binding-participant")),
        cross_plane::CrossPlaneReference::Endpoint(endpoint("binding-endpoint")),
        Some(security::AuthorizationContextClass::VerifiedCredentialContext),
        cross_plane::BindingLifecyclePrecondition::ParticipantJoined,
        cross_plane::BindingLifecyclePrecondition::EndpointAdmitted,
        cross_plane::BindingExpiryBehavior::RejectNewTargetPlaneAction,
        cross_plane::BindingReplayRelation::PriorAcceptedBindingObservable,
    );
    assert_eq!(
        policy.binding_class(),
        cross_plane::CrossPlaneBindingClass::SfuEndpointBinding
    );

    for outcome in [
        cross_plane::CrossPlaneBindingOutcome::Accepted,
        cross_plane::CrossPlaneBindingOutcome::Rejected,
        cross_plane::CrossPlaneBindingOutcome::Expired,
        cross_plane::CrossPlaneBindingOutcome::Failed,
        cross_plane::CrossPlaneBindingOutcome::CloseNotClaimed,
    ] {
        assert!(!outcome.code().is_empty());
        assert_eq!(
            outcome.requires_reason(),
            !matches!(outcome, cross_plane::CrossPlaneBindingOutcome::Accepted)
        );
    }

    for failure in [
        cross_plane::CrossPlaneBindingFailureKind::BindingClassNotAdmitted,
        cross_plane::CrossPlaneBindingFailureKind::RequiredBindingAbsent,
        cross_plane::CrossPlaneBindingFailureKind::BindingMaterialInvalid,
        cross_plane::CrossPlaneBindingFailureKind::SourceTargetScopeConflict,
        cross_plane::CrossPlaneBindingFailureKind::LifecycleConflict,
        cross_plane::CrossPlaneBindingFailureKind::BindingExpired,
        cross_plane::CrossPlaneBindingFailureKind::BindingReplayDetected,
        cross_plane::CrossPlaneBindingFailureKind::ParticipantNotAdmitted,
        cross_plane::CrossPlaneBindingFailureKind::TargetUnavailable,
        cross_plane::CrossPlaneBindingFailureKind::AllocationNotFound,
        cross_plane::CrossPlaneBindingFailureKind::PermissionNotFound,
        cross_plane::CrossPlaneBindingFailureKind::SecureMediaProtectionNotActive,
    ] {
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    let decision = cross_plane::CrossPlaneBindingDecision::new(
        correlation("cross-plane-decision"),
        policy,
        cross_plane::CrossPlaneBindingOutcome::Rejected,
        Some(cross_plane::CrossPlaneBindingFailureKind::LifecycleConflict),
    );
    assert_eq!(decision.audit_event_type(), "cross_plane_binding_decision");

    for adoption in [
        cross_plane::CrossPlaneEvidenceAdoption::SinglePlaneOnly,
        cross_plane::CrossPlaneEvidenceAdoption::CrossPlaneBindingEvidence,
        cross_plane::CrossPlaneEvidenceAdoption::CloseNotClaimed,
    ] {
        assert!(format!("{adoption:?}").len() > 3);
    }
    for prohibited in [
        cross_plane::ProhibitedCrossPlaneEquivalence::SignalingJoinAsSfuAdmission,
        cross_plane::ProhibitedCrossPlaneEquivalence::TokenOrAuthorizationAsPlaneBinding,
        cross_plane::ProhibitedCrossPlaneEquivalence::SameCorrelationIdAsBinding,
        cross_plane::ProhibitedCrossPlaneEquivalence::IceRelayAsTurnAllocationOrConnectivityProof,
        cross_plane::ProhibitedCrossPlaneEquivalence::TurnCredentialDeliveryAsActiveTurnState,
        cross_plane::ProhibitedCrossPlaneEquivalence::SecureMediaProtectionAsAuthorization,
        cross_plane::ProhibitedCrossPlaneEquivalence::DriverLocalMapAsBindingAuthority,
        cross_plane::ProhibitedCrossPlaneEquivalence::SilentCrossPlaneLifecycleMutation,
    ] {
        assert!(format!("{prohibited:?}").len() > 3);
    }
}

#[test]
fn media_identity_symbols_used_by_surface_test_remain_semantically_typed() {
    // この test は import 群が raw string 化しないことを確認し、型つき参照の接続を維持します。
    let _typed_media_refs = (
        arcrtc_core_sfu::SfuReferenceSet::new(
            session("sfu-session"),
            Some(endpoint("sfu-endpoint")),
            Some(stream("sfu-stream")),
            Some(RouteId::new(reference("route-id"))),
            Some(packet("sfu-packet")),
        ),
        RouteId::new(reference("route-id")),
        arcrtc_core_turn::TurnReferenceSet::new(
            Some(allocation("turn-allocation")),
            Some(permission("turn-permission")),
            Some(channel_bind("turn-channel-bind")),
            Some(credential("turn-credential")),
        ),
    );
    let debug_output = format!("{:?}", _typed_media_refs);
    assert!(debug_output.contains("value_len"));
    assert!(!debug_output.contains("route-id"));
}
