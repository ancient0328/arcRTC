use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use arcrtc_core_command as command;
use arcrtc_core_configuration as configuration;
use arcrtc_core_domain as domain;
use arcrtc_core_domain::{AggregateRoot, DomainService};
use arcrtc_core_features as features;
use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CorrelationId, CredentialRef, OpaqueReference, PacketId,
    PermissionId, ReferenceAuthority, SessionId, StartupRunId,
};
use arcrtc_core_protocol as protocol;
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_security as security;
use arcrtc_core_state as state;
use arcrtc_core_time as time;
use arcrtc_core_transport as transport;
use arcrtc_core_turn as turn;

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("reference is valid")
}

fn correlation(value: &str) -> CorrelationId {
    CorrelationId::new(reference(value))
}

fn session(value: &str) -> SessionId {
    SessionId::new(reference(value))
}

fn packet(value: &str) -> PacketId {
    PacketId::new(reference(value))
}

fn startup(value: &str) -> StartupRunId {
    StartupRunId::new(reference(value))
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

fn reason(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("reason code is cataloged")
}

fn touch_hash<T>(value: T)
where
    T: Clone + Debug + Eq + Hash,
{
    let cloned = value.clone();
    assert_eq!(cloned, value);
    assert!(!format!("{value:?}").is_empty());
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let _ = hasher.finish();
}

fn touch_eq<T>(value: T)
where
    T: Clone + Debug + Eq,
{
    let cloned = value.clone();
    assert_eq!(cloned, value);
    assert!(!format!("{value:?}").is_empty());
}

fn decision(
    target_surface: command::TargetSurface,
    outcome: command::UseCaseOutcome,
    reason: command::DecisionReason<turn::TurnFailureKind>,
) -> command::UseCaseDecision<turn::TurnFailureKind> {
    command::UseCaseDecision::new(command::UseCaseDecisionInput {
        correlation_id: correlation("turn-decision"),
        command_type: command::CommandType::new("turn-command"),
        target_surface,
        outcome,
        reason,
        state_transition: command::StateTransitionSummary::Changed("turn-state"),
        port_intents: vec![command::PortIntent::new(
            "turn-port-intent",
            command::TargetSurface::Turn,
        )],
        audit_projection: command::AuditProjectionRequirement::Required,
        evidence_class: command::DecisionEvidenceClass::SourceDecisionOnly,
    })
    .expect("decision shape is valid")
}

#[test]
fn protocol_canonical_version_and_semantic_envelope_paths_are_exercised() {
    // protocol 境界の closed vocabulary と canonical/versioning の accessor 経路を広く通します。
    touch_eq(protocol::CoreProtocolSurface);

    for class in [
        protocol::CanonicalDataClass::AuditEventHashInput,
        protocol::CanonicalDataClass::CoreReason,
        protocol::CanonicalDataClass::CoreReference,
        protocol::CanonicalDataClass::EvidenceTimestamp,
        protocol::CanonicalDataClass::Duration,
        protocol::CanonicalDataClass::BinaryPayloadDigest,
    ] {
        touch_hash(class);
    }

    for status in [
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::RequiresAdrOrCanonical,
    ] {
        touch_hash(status);
    }
    for handling in [
        protocol::UnknownFieldHandling::IgnoredOnlyWhenCompatibilityAllows,
        protocol::UnknownFieldHandling::Reject,
    ] {
        touch_hash(handling);
    }

    let defined_rules = protocol::CanonicalEncodingRuleSet::new(
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
    assert!(defined_rules.usable_for_canonical_evidence());
    touch_hash(defined_rules);

    let incomplete_digest_rule = protocol::CanonicalEncodingRuleSet::new(
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::RequiresAdrOrCanonical,
        protocol::CanonicalRuleStatus::Defined,
        protocol::UnknownFieldHandling::IgnoredOnlyWhenCompatibilityAllows,
        protocol::CanonicalRuleStatus::Defined,
        protocol::CanonicalRuleStatus::Defined,
    );
    assert!(!incomplete_digest_rule.usable_for_canonical_evidence());

    let format_version = protocol::CanonicalFormatVersion::new("arcrtc-canonical", "0.2.0");
    assert_eq!(format_version.format(), "arcrtc-canonical");
    assert_eq!(format_version.version(), "0.2.0");
    touch_hash(format_version);
    assert_eq!(
        protocol::CanonicalDigest::new(format_version, "", vec![1]),
        Err(protocol::CanonicalEncodingError::EmptyDigestMaterial)
    );
    assert_eq!(
        protocol::CanonicalDigest::new(format_version, "sha256", vec![]),
        Err(protocol::CanonicalEncodingError::EmptyDigestMaterial)
    );
    let digest = protocol::CanonicalDigest::new(format_version, "sha256", vec![1, 2, 3])
        .expect("digest material is present");
    assert_eq!(digest.format_version(), format_version);
    assert_eq!(digest.algorithm(), "sha256");
    assert_eq!(digest.digest(), &[1, 2, 3]);
    touch_hash(digest);

    for failure in [
        protocol::CanonicalEncodingFailureKind::CanonicalSerializationFailed,
        protocol::CanonicalEncodingFailureKind::CanonicalSerializationMismatch,
        protocol::CanonicalEncodingFailureKind::ExternalDecodeFailed,
        protocol::CanonicalEncodingFailureKind::ExternalEncodeFailed,
        protocol::CanonicalEncodingFailureKind::MissingRequiredWireField,
        protocol::CanonicalEncodingFailureKind::UnsupportedCanonicalVersion,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    let v020 = protocol::ContractVersion::new(0, 2, 0);
    let v021 = protocol::ContractVersion::new(0, 2, 1);
    assert_eq!(v020.major(), 0);
    assert!(v020 < v021);
    touch_hash(v020);
    let wire = protocol::WireEncodingVersion::new("driver-json-v1");
    assert_eq!(wire.code(), "driver-json-v1");
    touch_hash(wire);

    for surface in [
        protocol::VersionedSurface::SignalingContract,
        protocol::VersionedSurface::TurnContract,
        protocol::VersionedSurface::SfuContract,
        protocol::VersionedSurface::SdkPublicContract,
        protocol::VersionedSurface::AuditSchema,
        protocol::VersionedSurface::DriverWireEncoding,
    ] {
        touch_hash(surface);
    }
    for owner in [
        protocol::VersionOwner::Core,
        protocol::VersionOwner::Driver,
        protocol::VersionOwner::Sdk,
    ] {
        touch_hash(owner);
    }
    for outcome in [
        protocol::VersionNegotiationOutcome::AcceptedExact(v020),
        protocol::VersionNegotiationOutcome::AcceptedCompatible {
            requested: v020,
            accepted: v021,
        },
        protocol::VersionNegotiationOutcome::RejectedUnsupported(v021),
    ] {
        touch_hash(outcome);
    }

    let capability = protocol::Capability::new("simulcast");
    assert_eq!(capability.name(), "simulcast");
    touch_hash(capability);
    for rule in [
        protocol::CapabilityRule::OptionalWithinAcceptedVersion,
        protocol::CapabilityRule::MustNotAlterRequiredStateTransition,
    ] {
        touch_hash(rule);
    }

    let ownership = protocol::VersionedSurfaceOwnership::new(
        protocol::VersionedSurface::TurnContract,
        protocol::VersionOwner::Core,
    );
    assert_eq!(
        ownership.surface(),
        protocol::VersionedSurface::TurnContract
    );
    assert_eq!(ownership.owner(), protocol::VersionOwner::Core);
    touch_hash(ownership);

    for (change, classification) in [
        (
            protocol::CompatibilityChange::AddOptionalFieldWithDefault,
            protocol::CompatibilityClassification::Compatible,
        ),
        (
            protocol::CompatibilityChange::AddRequiredField,
            protocol::CompatibilityClassification::Breaking,
        ),
        (
            protocol::CompatibilityChange::RemoveField,
            protocol::CompatibilityClassification::Breaking,
        ),
        (
            protocol::CompatibilityChange::ChangeReasonCodeSemantics,
            protocol::CompatibilityClassification::Prohibited,
        ),
        (
            protocol::CompatibilityChange::AddReasonCode,
            protocol::CompatibilityClassification::Conditional,
        ),
        (
            protocol::CompatibilityChange::ChangeStateTransition,
            protocol::CompatibilityClassification::Breaking,
        ),
        (
            protocol::CompatibilityChange::AddDriverEncoding,
            protocol::CompatibilityClassification::Compatible,
        ),
    ] {
        touch_hash(change);
        touch_hash(classification);
        assert_eq!(change.classification(), classification);
    }

    let range = protocol::CompatibilityVersionRange::new(
        protocol::VersionedSurface::SfuContract,
        v020,
        v021,
    );
    assert_eq!(range.surface(), protocol::VersionedSurface::SfuContract);
    touch_hash(range);
    let deprecation = protocol::DeprecationDecision::new(
        protocol::VersionedSurface::SdkPublicContract,
        v020,
        protocol::VersionOwner::Sdk,
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
    touch_eq(deprecation);

    for failure in [
        protocol::CompatibilityFailureKind::UnsupportedCommandVersion,
        protocol::CompatibilityFailureKind::UnsupportedMediaContractVersion,
        protocol::CompatibilityFailureKind::UnsupportedTurnContractVersion,
        protocol::CompatibilityFailureKind::UnsupportedDriverWireVersion,
        protocol::CompatibilityFailureKind::MissingRequiredWireField,
        protocol::CompatibilityFailureKind::ExternalEnumUnmapped,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    for kind in [
        protocol::SemanticEnvelopeMessageKind::Command,
        protocol::SemanticEnvelopeMessageKind::Event,
    ] {
        touch_hash(kind);
    }
    for command_type in [
        protocol::SignalingSemanticCommandType::JoinRoom,
        protocol::SignalingSemanticCommandType::LeaveRoom,
        protocol::SignalingSemanticCommandType::SendOffer,
        protocol::SignalingSemanticCommandType::SendAnswer,
        protocol::SignalingSemanticCommandType::SendIceCandidate,
        protocol::SignalingSemanticCommandType::RequestTurnCredential,
        protocol::SignalingSemanticCommandType::AcknowledgeForward,
    ] {
        touch_hash(command_type);
    }
    for event_type in [
        protocol::SignalingSemanticEventType::Joined,
        protocol::SignalingSemanticEventType::Rejected,
        protocol::SignalingSemanticEventType::ParticipantJoined,
        protocol::SignalingSemanticEventType::ParticipantLeft,
        protocol::SignalingSemanticEventType::OfferReceived,
        protocol::SignalingSemanticEventType::AnswerReceived,
        protocol::SignalingSemanticEventType::IceCandidateReceived,
        protocol::SignalingSemanticEventType::TurnCredentialAvailable,
        protocol::SignalingSemanticEventType::ProtocolViolation,
    ] {
        touch_hash(event_type);
    }
    for model_type in [
        protocol::SfuSemanticModelType::SfuSession,
        protocol::SfuSemanticModelType::ParticipantEndpoint,
        protocol::SfuSemanticModelType::MediaStream,
        protocol::SfuSemanticModelType::MediaNegotiationReference,
        protocol::SfuSemanticModelType::Publication,
        protocol::SfuSemanticModelType::Subscription,
        protocol::SfuSemanticModelType::ForwardingIntent,
        protocol::SfuSemanticModelType::RouteCandidate,
        protocol::SfuSemanticModelType::BorrowedPacketAbstractView,
        protocol::SfuSemanticModelType::QualityObservation,
        protocol::SfuSemanticModelType::BackpressureState,
        protocol::SfuSemanticModelType::AdmissionDecision,
        protocol::SfuSemanticModelType::RejectionReason,
    ] {
        touch_hash(model_type);
    }
    for decision_type in [
        protocol::SfuSemanticDecisionType::ParticipantAdmission,
        protocol::SfuSemanticDecisionType::Publication,
        protocol::SfuSemanticDecisionType::Subscription,
        protocol::SfuSemanticDecisionType::RouteSelection,
        protocol::SfuSemanticDecisionType::Forwarding,
        protocol::SfuSemanticDecisionType::DegradationRecovery,
        protocol::SfuSemanticDecisionType::BackpressureAction,
        protocol::SfuSemanticDecisionType::QualityViolation,
    ] {
        touch_hash(decision_type);
    }
    for model_type in [
        protocol::TurnSemanticModelType::TransactionId,
        protocol::TurnSemanticModelType::Request,
        protocol::TurnSemanticModelType::Response,
        protocol::TurnSemanticModelType::Indication,
        protocol::TurnSemanticModelType::Error,
        protocol::TurnSemanticModelType::Allocation,
        protocol::TurnSemanticModelType::Permission,
        protocol::TurnSemanticModelType::ChannelBinding,
        protocol::TurnSemanticModelType::ChannelBindingReference,
        protocol::TurnSemanticModelType::PeerAddress,
        protocol::TurnSemanticModelType::RelayDecision,
        protocol::TurnSemanticModelType::CredentialVerificationOutcome,
        protocol::TurnSemanticModelType::LifetimeExpiry,
        protocol::TurnSemanticModelType::ClosedErrorReason,
    ] {
        touch_hash(model_type);
    }
    for decision_type in [
        protocol::TurnSemanticDecisionType::Allocation,
        protocol::TurnSemanticDecisionType::AllocationLifecycle,
        protocol::TurnSemanticDecisionType::Refresh,
        protocol::TurnSemanticDecisionType::Permission,
        protocol::TurnSemanticDecisionType::ChannelBind,
        protocol::TurnSemanticDecisionType::Relay,
        protocol::TurnSemanticDecisionType::MalformedMessage,
        protocol::TurnSemanticDecisionType::ExpiredCredential,
        protocol::TurnSemanticDecisionType::UnauthorizedRequest,
    ] {
        touch_hash(decision_type);
    }
    for command_type in [
        protocol::TransportSemanticCommandType::StartSession,
        protocol::TransportSemanticCommandType::ApplyLocalDescription,
        protocol::TransportSemanticCommandType::ApplyRemoteDescription,
        protocol::TransportSemanticCommandType::AddIceCandidate,
        protocol::TransportSemanticCommandType::ForwardPacket,
        protocol::TransportSemanticCommandType::CloseSession,
    ] {
        touch_hash(command_type);
    }
    for event_type in [
        protocol::TransportSemanticEventType::SessionObserved,
        protocol::TransportSemanticEventType::LocalDescriptionAccepted,
        protocol::TransportSemanticEventType::RemoteDescriptionAccepted,
        protocol::TransportSemanticEventType::IceCandidateObserved,
        protocol::TransportSemanticEventType::PacketSemanticViewObserved,
        protocol::TransportSemanticEventType::TransportClosed,
        protocol::TransportSemanticEventType::ConversionFailed,
    ] {
        touch_hash(event_type);
    }

    let payload = protocol::CoreSemanticPayloadModel::new(
        protocol::CoreSemanticPayloadClass::OpaqueCoreReference,
        Some(reference("semantic-payload")),
    );
    touch_hash(payload.clone());
    for payload_class in [
        protocol::CoreSemanticPayloadClass::SignalingSubject,
        protocol::CoreSemanticPayloadClass::SfuReferenceSet,
        protocol::CoreSemanticPayloadClass::TurnReferenceSet,
        protocol::CoreSemanticPayloadClass::PacketSemanticView,
        protocol::CoreSemanticPayloadClass::OpaqueCoreReference,
    ] {
        touch_hash(payload_class);
    }

    let semantic_message = protocol::CoreSemanticMessageType::TransportCommand(
        protocol::TransportSemanticCommandType::ForwardPacket,
    );
    touch_hash(semantic_message);
    assert_eq!(
        protocol::CoreSemanticEnvelope::try_new(
            command::TargetSurface::Transport,
            v020,
            correlation("semantic-envelope"),
            protocol::SemanticEnvelopeMessageKind::Command,
            semantic_message,
            true,
            Some(payload.clone()),
            Some(command::UseCaseOutcome::Rejected),
            None,
        ),
        Err(protocol::SemanticEnvelopeError::RequiredReasonMissing)
    );
    assert_eq!(
        protocol::CoreSemanticEnvelope::try_new(
            command::TargetSurface::Transport,
            v020,
            correlation("semantic-envelope"),
            protocol::SemanticEnvelopeMessageKind::Event,
            protocol::CoreSemanticMessageType::DriverErrorConverted,
            true,
            Some(payload.clone()),
            Some(command::UseCaseOutcome::Accepted),
            Some(reason("external_decode_failed")),
        ),
        Err(protocol::SemanticEnvelopeError::SuccessReasonMustNotBeInvented)
    );
    let envelope = protocol::CoreSemanticEnvelope::try_new(
        command::TargetSurface::Transport,
        v020,
        correlation("semantic-envelope-ok"),
        protocol::SemanticEnvelopeMessageKind::Event,
        protocol::CoreSemanticMessageType::DriverErrorConverted,
        true,
        Some(payload),
        Some(command::UseCaseOutcome::ConvertedFailure),
        Some(reason("external_decode_failed")),
    )
    .expect("non-success envelope carries cataloged reason");
    touch_eq(envelope);
}

#[test]
fn time_normalization_clock_skew_and_sync_decision_paths_are_exercised() {
    // time 境界では unit/window/clock trust を public vocabulary として実行します。
    touch_eq(time::CoreTimeSurface);

    for (quantity, unit) in [
        (
            time::NormalizedQuantity::Duration,
            time::NormalizedUnit::MillisecondsInteger,
        ),
        (
            time::NormalizedQuantity::Timestamp,
            time::NormalizedUnit::UtcEpochMillisecondsEvidenceOnly,
        ),
        (
            time::NormalizedQuantity::Bytes,
            time::NormalizedUnit::BytesInteger,
        ),
        (
            time::NormalizedQuantity::PacketCount,
            time::NormalizedUnit::IntegerCount,
        ),
        (
            time::NormalizedQuantity::Rate,
            time::NormalizedUnit::UnitsPerSecond,
        ),
        (
            time::NormalizedQuantity::Ratio,
            time::NormalizedUnit::RationalDeclaredPrecision,
        ),
        (
            time::NormalizedQuantity::Bitrate,
            time::NormalizedUnit::BitsPerSecond,
        ),
        (
            time::NormalizedQuantity::JitterRtt,
            time::NormalizedUnit::MillisecondsDeclaredPrecision,
        ),
    ] {
        touch_hash(quantity);
        touch_hash(unit);
        assert_eq!(quantity.default_unit(), unit);
    }
    touch_hash(time::NormalizedUnit::FixedDecimalDeclaredPrecision);

    for value in [
        time::NormalizedValue::Integer(42),
        time::NormalizedValue::rational(1, 3).expect("non-zero denominator"),
        time::NormalizedValue::FixedDecimal {
            value: 12345,
            scale: 2,
        },
    ] {
        touch_hash(value);
    }
    assert_eq!(
        time::NormalizedValue::rational(1, 0),
        Err(time::NormalizedValueError::ZeroDenominator)
    );

    for precision in [
        time::PrecisionClass::IntegerExact,
        time::PrecisionClass::DeclaredDecimalScale(3),
        time::PrecisionClass::DeclaredPrecisionLabel("millisecond"),
    ] {
        touch_hash(precision);
    }
    for rounding in [
        time::RoundingDirection::Exact,
        time::RoundingDirection::Floor,
        time::RoundingDirection::Ceiling,
        time::RoundingDirection::Nearest,
        time::RoundingDirection::TowardZero,
    ] {
        touch_hash(rounding);
    }
    for operator in [
        time::ComparisonOperator::LessThan,
        time::ComparisonOperator::LessThanOrEqual,
        time::ComparisonOperator::Equal,
        time::ComparisonOperator::GreaterThanOrEqual,
        time::ComparisonOperator::GreaterThan,
    ] {
        touch_hash(operator);
    }
    for boundary in [
        time::BoundaryInclusivity::Inclusive,
        time::BoundaryInclusivity::Exclusive,
    ] {
        touch_hash(boundary);
    }
    for window in [
        time::SamplingWindow::None,
        time::SamplingWindow::Milliseconds(250),
        time::SamplingWindow::PerSecond,
        time::SamplingWindow::PolicyLabel("startup-window"),
    ] {
        touch_hash(window);
    }
    for owner in [
        time::RawMeasurementOwner::Driver,
        time::RawMeasurementOwner::Entrypoints,
        time::RawMeasurementOwner::BenchmarkDocsReports,
    ] {
        touch_hash(owner);
    }
    for owner in [
        time::PolicyDecisionOwner::Core,
        time::PolicyDecisionOwner::Reports,
    ] {
        touch_hash(owner);
    }
    for source in [
        time::TimeSourceClass::MonotonicClockPort,
        time::TimeSourceClass::WallClockEvidenceOnly,
    ] {
        touch_hash(source);
    }
    touch_hash(time::NormalizedMeasurement::new(
        time::NormalizedQuantity::Bitrate,
        time::NormalizedUnit::BitsPerSecond,
        time::NormalizedValue::Integer(128_000),
        time::PrecisionClass::IntegerExact,
        time::RoundingDirection::Exact,
        time::ComparisonOperator::LessThanOrEqual,
        time::BoundaryInclusivity::Inclusive,
        time::SamplingWindow::PerSecond,
        time::RawMeasurementOwner::Driver,
        time::PolicyDecisionOwner::Core,
    ));

    for failure in [
        time::TimeNormalizationFailureKind::MeasurementNormalizationFailed,
        time::TimeNormalizationFailureKind::TimeObservationUnavailable,
        time::TimeNormalizationFailureKind::OperationDeadlineExceeded,
        time::TimeNormalizationFailureKind::RetentionDurationExceeded,
        time::TimeNormalizationFailureKind::RuntimeConfigInvalid,
        time::TimeNormalizationFailureKind::DriverShutdown,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for adoption in [
        time::EvidenceUnitWindowAdoption::RawAndNormalizedRecorded,
        time::EvidenceUnitWindowAdoption::NormalizedValueRecorded,
        time::EvidenceUnitWindowAdoption::MissingUnitWindowNotAdoptable,
    ] {
        touch_hash(adoption);
    }
    for prohibited in [
        time::ProhibitedUnitMeasurementBehavior::DriverMetricLabelAsPolicyUnit,
        time::ProhibitedUnitMeasurementBehavior::WallClockOrderAsMonotonicPolicy,
        time::ProhibitedUnitMeasurementBehavior::HiddenFloatPrecisionComparison,
        time::ProhibitedUnitMeasurementBehavior::BitrateByteRateConflated,
        time::ProhibitedUnitMeasurementBehavior::BenchmarkWithoutUnitWindowAggregation,
        time::ProhibitedUnitMeasurementBehavior::RawPlatformStatsInCorePolicy,
        time::ProhibitedUnitMeasurementBehavior::TimezoneConversionAsSynchronizationEvidence,
    ] {
        touch_hash(prohibited);
    }

    for (concern, owner) in [
        (
            time::TimeSynchronizationConcern::LocalMonotonicDuration,
            time::TimeSynchronizationOwner::CorePolicy,
        ),
        (
            time::TimeSynchronizationConcern::WallClockTimestamp,
            time::TimeSynchronizationOwner::DriverRuntimeObservation,
        ),
        (
            time::TimeSynchronizationConcern::CrossNodeSkewPolicy,
            time::TimeSynchronizationOwner::CorePolicy,
        ),
        (
            time::TimeSynchronizationConcern::ExternalTimeSource,
            time::TimeSynchronizationOwner::DriverRuntimeObservation,
        ),
        (
            time::TimeSynchronizationConcern::TimestampNormalization,
            time::TimeSynchronizationOwner::CorePolicy,
        ),
        (
            time::TimeSynchronizationConcern::EvidenceTimestampClaim,
            time::TimeSynchronizationOwner::EvidenceCanonical,
        ),
    ] {
        touch_hash(concern);
        touch_hash(owner);
        assert_eq!(concern.owner(), owner);
    }

    for (trust, runtime_allowed, cross_node_allowed) in [
        (time::TimeTrustClass::SingleProcessMonotonic, true, false),
        (time::TimeTrustClass::SingleNodeWallClock, true, false),
        (time::TimeTrustClass::MultiNodeBoundedSkew, true, true),
        (time::TimeTrustClass::ExternalTrustedTimeSource, true, true),
        (time::TimeTrustClass::TestDeterministicClock, false, false),
        (time::TimeTrustClass::TimeUntrusted, false, false),
    ] {
        touch_hash(trust);
        assert_eq!(trust.runtime_evidence_allowed(), runtime_allowed);
        assert_eq!(trust.supports_cross_node_comparison(), cross_node_allowed);
    }
    for scope in [
        time::TimeNodeScope::SingleProcess,
        time::TimeNodeScope::SingleNode,
        time::TimeNodeScope::MultiNode,
        time::TimeNodeScope::MultiProcess,
    ] {
        touch_hash(scope);
    }
    for source in [
        time::TrustedTimeSourceClass::LocalMonotonicClock,
        time::TrustedTimeSourceClass::LocalWallClock,
        time::TrustedTimeSourceClass::ExternalTimeSourceObservation,
        time::TrustedTimeSourceClass::DeterministicTestClock,
        time::TrustedTimeSourceClass::UntrustedOrUnavailable,
    ] {
        touch_hash(source);
    }
    for skew in [
        time::ObservedSkewClass::NotRequired,
        time::ObservedSkewClass::WithinPolicy,
        time::ObservedSkewClass::ExceedsPolicy,
        time::ObservedSkewClass::ObservationUnavailable,
    ] {
        touch_hash(skew);
    }
    for impact in [
        time::ClockSkewImpact::Expiry,
        time::ClockSkewImpact::Ordering,
        time::ClockSkewImpact::Audit,
        time::ClockSkewImpact::Evidence,
    ] {
        touch_hash(impact);
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
            Some(50),
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
            Some(50),
            time::PrecisionClass::DeclaredPrecisionLabel("external-ms"),
            time::SamplingWindow::PolicyLabel("ntp-sample"),
            time::TrustedTimeSourceClass::LocalWallClock,
            time::ClockSkewImpact::Evidence,
        ),
        Err(time::ClockSkewPolicyError::ExternalSourceClassRequired)
    );
    let policy = time::ClockSkewPolicy::try_new(
        time::TimeNodeScope::MultiNode,
        time::TimeTrustClass::MultiNodeBoundedSkew,
        Some(100),
        time::PrecisionClass::IntegerExact,
        time::SamplingWindow::Milliseconds(100),
        time::TrustedTimeSourceClass::ExternalTimeSourceObservation,
        time::ClockSkewImpact::Audit,
    )
    .expect("bounded skew policy is complete");
    touch_hash(policy);

    for basis in [
        time::TimestampOrderingBasis::Correlation,
        time::TimestampOrderingBasis::Sequence,
        time::TimestampOrderingBasis::Idempotency,
        time::TimestampOrderingBasis::AggregateVersion,
        time::TimestampOrderingBasis::ProtocolState,
        time::TimestampOrderingBasis::BoundedSkewPolicy,
        time::TimestampOrderingBasis::AuditHashChainSequence,
        time::TimestampOrderingBasis::EventSequence,
        time::TimestampOrderingBasis::PresentationOnlyTimestamp,
    ] {
        touch_hash(basis);
    }
    for basis in [
        time::ExpiryDeadlineTimeBasis::MonotonicDuration,
        time::ExpiryDeadlineTimeBasis::WallClockTimestamp,
        time::ExpiryDeadlineTimeBasis::BoundedSkew,
    ] {
        touch_hash(basis);
    }
    for failure in [
        time::TimeSynchronizationFailureKind::ClockSkewExceeded,
        time::TimeSynchronizationFailureKind::TimeSourceUntrusted,
        time::TimeSynchronizationFailureKind::TimeSyncUnavailable,
        time::TimeSynchronizationFailureKind::TimestampOrderUntrusted,
        time::TimeSynchronizationFailureKind::TimeObservationUnavailable,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for outcome in [
        time::TimeSynchronizationOutcome::Accepted,
        time::TimeSynchronizationOutcome::Rejected,
        time::TimeSynchronizationOutcome::Unavailable,
        time::TimeSynchronizationOutcome::Untrusted,
    ] {
        touch_hash(outcome);
    }

    assert_eq!(
        time::TimeSynchronizationDecision::try_new(
            startup("time-sync-reason-on-success"),
            Some(correlation("time-sync-reason-on-success")),
            policy,
            time::ObservedSkewClass::WithinPolicy,
            time::TimeSynchronizationOutcome::Accepted,
            Some(time::TimeSynchronizationFailureKind::ClockSkewExceeded),
        ),
        Err(time::TimeSynchronizationDecisionError::ReasonMustBeAbsentForAccepted)
    );
    assert_eq!(
        time::TimeSynchronizationDecision::try_new(
            startup("time-sync-missing-reason"),
            Some(correlation("time-sync-missing-reason")),
            policy,
            time::ObservedSkewClass::ObservationUnavailable,
            time::TimeSynchronizationOutcome::Unavailable,
            None,
        ),
        Err(time::TimeSynchronizationDecisionError::ReasonRequired)
    );
    let sync_decision = time::TimeSynchronizationDecision::try_new(
        startup("time-sync-ok"),
        Some(correlation("time-sync-ok")),
        policy,
        time::ObservedSkewClass::WithinPolicy,
        time::TimeSynchronizationOutcome::Accepted,
        None,
    )
    .expect("accepted sync decision carries no reason");
    assert_eq!(
        sync_decision.audit_event_type(),
        "time_synchronization_decision"
    );
    touch_hash(sync_decision);

    for prohibited in [
        time::ProhibitedTimeSynchronizationBehavior::WallClockAsCrossNodeCausalOrder,
        time::ProhibitedTimeSynchronizationBehavior::ReportCreationTimeAsRuntimeObservation,
        time::ProhibitedTimeSynchronizationBehavior::DeterministicTestClockAsProductionSyncEvidence,
        time::ProhibitedTimeSynchronizationBehavior::DriverNtpStatusRedefinesCorePolicy,
        time::ProhibitedTimeSynchronizationBehavior::TimezoneConversionAsSynchronizationProof,
        time::ProhibitedTimeSynchronizationBehavior::MissingSkewObservationAccepted,
    ] {
        touch_hash(prohibited);
    }
}

#[test]
fn transport_and_turn_contracts_exercise_derive_reason_and_fail_closed_edges() {
    // transport/TURN は driver wire object を持たず、core-owned reference だけを実行します。
    touch_eq(transport::CoreTransportSurface);
    touch_eq(turn::CoreTurnSurface);

    for kind in [
        transport::TransportCommandKind::StartSession,
        transport::TransportCommandKind::ApplyLocalDescription,
        transport::TransportCommandKind::ApplyRemoteDescription,
        transport::TransportCommandKind::AddIceCandidate,
        transport::TransportCommandKind::ForwardPacket,
        transport::TransportCommandKind::CloseSession,
    ] {
        touch_hash(kind);
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
        touch_hash(kind);
    }
    let capability = transport::TransportCapability::new("ice-trickle");
    assert_eq!(capability.name(), "ice-trickle");
    touch_hash(capability);

    let session_id = session("transport-session");
    assert_eq!(
        transport::SessionDescriptionRef::new(session_id.clone(), ""),
        Err(transport::TransportContractError::InvalidSemanticReference)
    );
    assert_eq!(
        transport::IceCandidateRef::new(session_id.clone(), "bad\ncandidate"),
        Err(transport::TransportContractError::InvalidSemanticReference)
    );
    let description = transport::SessionDescriptionRef::new(session_id.clone(), "sdp-ref")
        .expect("description reference is valid");
    let candidate = transport::IceCandidateRef::new(session_id, "candidate-ref")
        .expect("candidate reference is valid");
    let packet_view = transport::PacketSemanticViewRef::new(packet("transport-packet"));
    touch_hash(description.clone());
    touch_hash(candidate.clone());
    touch_hash(packet_view.clone());

    for payload in [
        transport::WebRtcTransportPayload::Empty,
        transport::WebRtcTransportPayload::SessionDescription(description.clone()),
        transport::WebRtcTransportPayload::IceCandidate(candidate.clone()),
        transport::WebRtcTransportPayload::PacketView(packet_view.clone()),
        transport::WebRtcTransportPayload::Capability(capability),
    ] {
        touch_eq(transport::TransportCommand::new(
            transport::TransportCommandKind::ForwardPacket,
            payload.clone(),
        ));
        touch_eq(payload);
    }
    for failure in [
        transport::TransportFailureKind::UnsupportedMediaContractVersion,
        transport::TransportFailureKind::ExternalDecodeFailed,
        transport::TransportFailureKind::ExternalEncodeFailed,
        transport::TransportFailureKind::MediaPayloadMappingInvalid,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for failure in [
        transport::TransportDriverFailureKind::ExternalDecodeFailed,
        transport::TransportDriverFailureKind::ExternalEncodeFailed,
        transport::TransportDriverFailureKind::MediaPayloadMappingInvalid,
        transport::TransportDriverFailureKind::DriverShutdown,
    ] {
        touch_hash(failure);
        let mapped = transport::TransportDriverFailure::from_kind(failure);
        assert_eq!(mapped.kind(), failure);
        assert_eq!(mapped.reason(), reason(failure.reason_code()));
        touch_hash(mapped);
        touch_eq(transport::WebRtcTransportObservation::ConversionFailure(
            mapped,
        ));
    }
    for observation in [
        transport::WebRtcTransportObservation::Empty,
        transport::WebRtcTransportObservation::SessionDescription(description),
        transport::WebRtcTransportObservation::IceCandidate(candidate),
        transport::WebRtcTransportObservation::PacketView(packet_view),
    ] {
        touch_eq(transport::TransportEvent::new(
            transport::TransportEventKind::PacketSemanticViewObserved,
            observation.clone(),
        ));
        touch_eq(observation);
    }

    for class in [
        transport::NegotiationMaterialClass::Offer,
        transport::NegotiationMaterialClass::Answer,
        transport::NegotiationMaterialClass::IceCandidate,
    ] {
        touch_hash(class);
    }
    for step in [
        transport::NegotiationFlowStep::ReceiveExternalMaterial,
        transport::NegotiationFlowStep::DriverDecodeAndValidate,
        transport::NegotiationFlowStep::MapToCoreReference,
        transport::NegotiationFlowStep::CoreSignalingEvaluation,
        transport::NegotiationFlowStep::CoreRelayDecision,
        transport::NegotiationFlowStep::ExternalProjection,
    ] {
        touch_hash(step);
    }
    for kind in [
        transport::NegotiationDecisionKind::AcceptedRelay,
        transport::NegotiationDecisionKind::RejectedRelay,
        transport::NegotiationDecisionKind::ProtocolViolation,
    ] {
        touch_hash(kind);
    }
    for surface in [
        transport::NegotiationVersionSurface::SignalingCommandEvent,
        transport::NegotiationVersionSurface::MediaFacingTransportContract,
        transport::NegotiationVersionSurface::DriverWireEncoding,
        transport::NegotiationVersionSurface::SdkPublicApi,
    ] {
        touch_hash(surface);
    }
    for requirement in [
        transport::SdkNegotiationParityRequirement::SameCommandEventSet,
        transport::SdkNegotiationParityRequirement::SameCorrelationPropagation,
        transport::SdkNegotiationParityRequirement::PreserveServerReason,
        transport::SdkNegotiationParityRequirement::SameVersionNegotiation,
        transport::SdkNegotiationParityRequirement::SignalingOnlyBoundary,
    ] {
        touch_hash(requirement);
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
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    for class in [
        transport::IceCandidateConnectivityClass::HostCandidateRef,
        transport::IceCandidateConnectivityClass::SrflxCandidateRef,
        transport::IceCandidateConnectivityClass::RelayCandidateRef,
        transport::IceCandidateConnectivityClass::MdnsCandidateRef,
        transport::IceCandidateConnectivityClass::TrickleCandidateRef,
        transport::IceCandidateConnectivityClass::IceRestartIntent,
        transport::IceCandidateConnectivityClass::ConnectivityObservation,
        transport::IceCandidateConnectivityClass::ConsentFreshnessObservation,
    ] {
        touch_hash(class);
    }
    for policy in [
        transport::IceAddressExposurePolicy::RelayOnly,
        transport::IceAddressExposurePolicy::HostAllowed,
        transport::IceAddressExposurePolicy::SrflxAllowed,
        transport::IceAddressExposurePolicy::MdnsObfuscationRequired,
        transport::IceAddressExposurePolicy::RawAddressRedactionRequired,
    ] {
        touch_hash(policy);
    }
    let ice_policy = transport::IceCandidatePolicy::new(
        vec![
            transport::IceCandidateConnectivityClass::RelayCandidateRef,
            transport::IceCandidateConnectivityClass::TrickleCandidateRef,
        ],
        transport::IceAddressExposurePolicy::RelayOnly,
        false,
    );
    assert_eq!(ice_policy.accepted_classes().len(), 2);
    touch_eq(ice_policy);
    for meaning in [
        transport::IceObservationMeaning::DiagnosticEvidenceOnly,
        transport::IceObservationMeaning::NotSignalingRelaySuccess,
        transport::IceObservationMeaning::NotCrossPlaneBinding,
    ] {
        touch_hash(meaning);
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
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for class in [
        transport::SecureMediaSessionClass::SecureMediaRequired,
        transport::SecureMediaSessionClass::DtlsHandshakeObserved,
        transport::SecureMediaSessionClass::PeerVerificationObserved,
        transport::SecureMediaSessionClass::SrtpProtectionActive,
        transport::SecureMediaSessionClass::RekeyRequired,
        transport::SecureMediaSessionClass::SessionClosed,
    ] {
        touch_hash(class);
    }
    touch_hash(transport::SecureMediaPolicy::new(
        "secure-media-v1",
        "dtls-srtp",
        true,
        true,
    ));
    for class in [
        transport::SecureMediaEvidenceClass::Handshake,
        transport::SecureMediaEvidenceClass::PeerVerification,
        transport::SecureMediaEvidenceClass::ProtectionState,
        transport::SecureMediaEvidenceClass::PacketForwardingScope,
    ] {
        touch_hash(class);
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
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    for message_class in [
        turn::TurnMessageClass::Request,
        turn::TurnMessageClass::Response,
        turn::TurnMessageClass::Indication,
        turn::TurnMessageClass::Error,
    ] {
        touch_hash(message_class);
        touch_hash(turn::TurnModelKind::Message(message_class));
    }
    for model in [
        turn::TurnModelKind::TransactionId,
        turn::TurnModelKind::Allocation,
        turn::TurnModelKind::Permission,
        turn::TurnModelKind::ChannelBinding,
        turn::TurnModelKind::ChannelBindingReference,
        turn::TurnModelKind::PeerAddress,
        turn::TurnModelKind::RelayDecision,
        turn::TurnModelKind::CredentialVerificationOutcome,
        turn::TurnModelKind::LifetimeExpiry,
        turn::TurnModelKind::ClosedErrorReason,
    ] {
        touch_hash(model);
    }
    for decision_kind in [
        turn::TurnDecisionKind::Allocation,
        turn::TurnDecisionKind::AllocationLifecycle,
        turn::TurnDecisionKind::Refresh,
        turn::TurnDecisionKind::Permission,
        turn::TurnDecisionKind::ChannelBind,
        turn::TurnDecisionKind::Relay,
        turn::TurnDecisionKind::MalformedMessage,
        turn::TurnDecisionKind::ExpiredCredential,
        turn::TurnDecisionKind::UnauthorizedRequest,
    ] {
        touch_hash(decision_kind);
    }

    assert_eq!(
        turn::CorePeerAddress::new(""),
        Err(turn::TurnContractError::InvalidPeerAddress)
    );
    assert_eq!(
        turn::CorePeerAddress::new("bad\npeer"),
        Err(turn::TurnContractError::InvalidPeerAddress)
    );
    let peer = turn::CorePeerAddress::new("peer-addr").expect("peer address is valid");
    assert_eq!(peer.as_str(), "peer-addr");
    touch_hash(peer.clone());
    let transaction = turn::TurnTransactionId::new(reference("turn-transaction"));
    assert_eq!(transaction.as_str(), "turn-transaction");
    touch_hash(transaction.clone());
    assert_eq!(
        turn::TurnRequestedLifetimeSeconds::try_new(0),
        Err(turn::TurnContractError::InvalidRequestedLifetime)
    );
    let lifetime =
        turn::TurnRequestedLifetimeSeconds::try_new(600).expect("non-zero lifetime is valid");
    assert_eq!(lifetime.as_u32(), 600);
    touch_hash(lifetime);

    let refs = turn::TurnReferenceSet::new(
        Some(allocation("turn-allocation")),
        Some(permission("turn-permission")),
        Some(channel_bind("turn-channel")),
        Some(credential("turn-credential")),
    );
    touch_eq(refs.clone());
    for (kind, expected) in [
        (
            turn::TurnCommandKind::Allocate,
            turn::TurnDecisionKind::Allocation,
        ),
        (
            turn::TurnCommandKind::Refresh,
            turn::TurnDecisionKind::Refresh,
        ),
        (
            turn::TurnCommandKind::CreatePermission,
            turn::TurnDecisionKind::Permission,
        ),
        (
            turn::TurnCommandKind::ChannelBind,
            turn::TurnDecisionKind::ChannelBind,
        ),
        (
            turn::TurnCommandKind::RelayData,
            turn::TurnDecisionKind::Relay,
        ),
    ] {
        touch_hash(kind);
        assert_eq!(kind.decision_kind(), expected);
    }
    assert_eq!(
        turn::TurnCommand::try_new(
            turn::TurnCommandKind::CreatePermission,
            transaction.clone(),
            refs.clone(),
            None,
            None,
            None,
        ),
        Err(turn::TurnContractError::MissingPeerAddress)
    );
    assert_eq!(
        turn::TurnCommand::try_new(
            turn::TurnCommandKind::ChannelBind,
            transaction.clone(),
            turn::TurnReferenceSet::new(None, None, None, None),
            Some(peer.clone()),
            None,
            None,
        ),
        Err(turn::TurnContractError::MissingChannelBindReference)
    );
    assert_eq!(
        turn::TurnCommand::try_new(
            turn::TurnCommandKind::RelayData,
            transaction.clone(),
            refs.clone(),
            Some(peer.clone()),
            None,
            None,
        ),
        Err(turn::TurnContractError::MissingRelayPacketReference)
    );
    let turn_command = turn::TurnCommand::try_new(
        turn::TurnCommandKind::RelayData,
        transaction.clone(),
        refs.clone(),
        Some(peer.clone()),
        Some(lifetime),
        Some(packet("turn-relay-packet")),
    )
    .expect("relay command includes required references");
    assert_eq!(turn_command.kind(), turn::TurnCommandKind::RelayData);
    assert_eq!(turn_command.transaction_id(), &transaction);
    assert_eq!(turn_command.references(), &refs);
    touch_eq(turn_command);

    let wrong_surface = decision(
        command::TargetSurface::Sfu,
        command::UseCaseOutcome::Rejected,
        command::DecisionReason::Cataloged(turn::TurnFailureKind::RelayDenied),
    );
    assert_eq!(
        turn::TurnDecision::new(turn::TurnDecisionKind::Relay, wrong_surface),
        Err(turn::TurnContractError::WrongTargetSurface)
    );
    let turn_surface = decision(
        command::TargetSurface::Turn,
        command::UseCaseOutcome::Accepted,
        command::DecisionReason::Absent,
    );
    touch_eq(
        turn::TurnDecision::new(turn::TurnDecisionKind::Relay, turn_surface)
            .expect("TURN target surface is required"),
    );

    for failure in [
        turn::TurnFailureKind::MalformedTurnMessage,
        turn::TurnFailureKind::CredentialMissing,
        turn::TurnFailureKind::CredentialInvalid,
        turn::TurnFailureKind::CredentialExpired,
        turn::TurnFailureKind::SecretGenerationNotAccepted,
        turn::TurnFailureKind::SecretKeyRevoked,
        turn::TurnFailureKind::SecretOverlapWindowExpired,
        turn::TurnFailureKind::SecretRotationStateUnavailable,
        turn::TurnFailureKind::AllocationCapacityExceeded,
        turn::TurnFailureKind::PermissionCapacityExceeded,
        turn::TurnFailureKind::PeerNotAllowed,
        turn::TurnFailureKind::PermissionNotFound,
        turn::TurnFailureKind::RelayDenied,
        turn::TurnFailureKind::AllocationNotFound,
        turn::TurnFailureKind::UnsupportedTurnMethod,
        turn::TurnFailureKind::UnsupportedTurnContractVersion,
        turn::TurnFailureKind::TurnLifetimeViolation,
        turn::TurnFailureKind::AllocationLifetimeExceeded,
        turn::TurnFailureKind::PermissionLifetimeExceeded,
        turn::TurnFailureKind::ChannelBindLifetimeExceeded,
        turn::TurnFailureKind::RefreshLimitExceeded,
        turn::TurnFailureKind::TurnRelayQueueBoundExceeded,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for state in [
        turn::AllocationState::Absent,
        turn::AllocationState::Requested,
        turn::AllocationState::Active,
        turn::AllocationState::Refreshing,
        turn::AllocationState::Expired,
        turn::AllocationState::Released,
        turn::AllocationState::Rejected,
    ] {
        touch_hash(state);
    }
    for state in [
        turn::PermissionState::Absent,
        turn::PermissionState::Requested,
        turn::PermissionState::Active,
        turn::PermissionState::Expired,
        turn::PermissionState::Revoked,
        turn::PermissionState::Rejected,
    ] {
        touch_hash(state);
    }
    for state in [
        turn::ChannelBindState::Unbound,
        turn::ChannelBindState::Requested,
        turn::ChannelBindState::Bound,
        turn::ChannelBindState::Expired,
        turn::ChannelBindState::Rejected,
    ] {
        touch_hash(state);
    }
    for rule in turn::TURN_LIFECYCLE_RULES {
        touch_eq(*rule);
        assert!(!rule.event().is_empty());
        assert!(!rule.allowed_pre_state().is_empty());
        assert!(!rule.success_state().is_empty());
        assert!(!rule.failure_state().is_empty());
        for failure in rule.reason_codes() {
            assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
        }
    }
}

#[test]
fn configuration_domain_features_state_and_security_edges_are_exercised() {
    // 周辺 core crate は境界語彙、accessor、reason catalog 接続を source 変更なしに実行します。
    touch_eq(configuration::CoreConfigurationSurface);
    for kind in [
        configuration::ConfigurationKind::CorePolicy,
        configuration::ConfigurationKind::DriverRuntime,
        configuration::ConfigurationKind::EntrypointComposition,
        configuration::ConfigurationKind::SdkClient,
        configuration::ConfigurationKind::Regulated,
    ] {
        touch_hash(kind);
    }
    for owner in [
        configuration::ConfigurationOwner::Core,
        configuration::ConfigurationOwner::Drivers,
        configuration::ConfigurationOwner::Entrypoints,
        configuration::ConfigurationOwner::Sdk,
        configuration::ConfigurationOwner::Regulated,
    ] {
        touch_hash(owner);
    }
    for source in [
        configuration::ConfigurationSourceClass::TypedInput,
        configuration::ConfigurationSourceClass::EnvironmentVariable,
        configuration::ConfigurationSourceClass::File,
        configuration::ConfigurationSourceClass::ProcessArgs,
        configuration::ConfigurationSourceClass::OsSettings,
        configuration::ConfigurationSourceClass::CloudMetadata,
    ] {
        touch_hash(source);
    }
    let core_policy = configuration::CorePolicyConfiguration::new("typed-policy");
    assert_eq!(*core_policy.policy(), "typed-policy");
    touch_eq(core_policy);
    let non_core = configuration::NonCoreConfigurationBoundary::new(
        configuration::ConfigurationKind::DriverRuntime,
        configuration::ConfigurationOwner::Drivers,
        configuration::ConfigurationSourceClass::EnvironmentVariable,
    );
    assert_eq!(
        non_core.kind(),
        configuration::ConfigurationKind::DriverRuntime
    );
    assert_eq!(non_core.owner(), configuration::ConfigurationOwner::Drivers);
    touch_eq(non_core);
    for allowed in [
        configuration::FeatureFlagAllowedEffect::SelectDriverImplementation,
        configuration::FeatureFlagAllowedEffect::EnableOptionalExporter,
        configuration::FeatureFlagAllowedEffect::ChooseExternalEncoding,
    ] {
        touch_hash(allowed);
    }
    for prohibited in [
        configuration::FeatureFlagProhibitedEffect::SignalingStateMachineChange,
        configuration::FeatureFlagProhibitedEffect::TurnLifecycleChange,
        configuration::FeatureFlagProhibitedEffect::SfuRoutingSemanticsChange,
        configuration::FeatureFlagProhibitedEffect::SecurityVerificationBypass,
        configuration::FeatureFlagProhibitedEffect::AuditRequirementBypass,
    ] {
        touch_hash(prohibited);
    }
    for failure in [
        configuration::ConfigurationFailureKind::CorePolicyConfigInvalid,
        configuration::ConfigurationFailureKind::RuntimeConfigMissing,
        configuration::ConfigurationFailureKind::RuntimeConfigInvalid,
        configuration::ConfigurationFailureKind::SecretUnavailable,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for boundary in [
        configuration::SecretBoundary::RawSecretOutsideCore,
        configuration::SecretBoundary::OpaqueCredentialReferenceOnly,
    ] {
        touch_hash(boundary);
    }

    touch_eq(domain::CoreDomainSurface);
    for family in [
        domain::AggregateFamily::SignalingRoomParticipant,
        domain::AggregateFamily::SfuSessionEndpointRoute,
        domain::AggregateFamily::TurnAllocationPermissionChannelBind,
        domain::AggregateFamily::CrossPlaneBindingScope,
        domain::AggregateFamily::AuditChainScope,
        domain::AggregateFamily::ConfigurationScope,
    ] {
        touch_hash(family);
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DummyId(u8);
    impl domain::ValueObject for DummyId {}
    struct DummyAggregate;
    impl domain::AggregateRoot for DummyAggregate {
        type Id = DummyId;

        fn family(&self) -> domain::AggregateFamily {
            domain::AggregateFamily::SfuSessionEndpointRoute
        }
    }
    struct DummyService;
    const DUMMY_FAMILIES: &[domain::AggregateFamily] = &[
        domain::AggregateFamily::SignalingRoomParticipant,
        domain::AggregateFamily::SfuSessionEndpointRoute,
    ];
    impl domain::DomainService for DummyService {
        fn aggregate_families(&self) -> &'static [domain::AggregateFamily] {
            DUMMY_FAMILIES
        }
    }
    assert_eq!(
        DummyAggregate.family(),
        domain::AggregateFamily::SfuSessionEndpointRoute
    );
    assert_eq!(DummyService.aggregate_families(), DUMMY_FAMILIES);
    let use_case = domain::UseCaseBoundary::<(), ()>::new(
        "join-room",
        domain::ENTRYPOINTLICATION_USE_CASE_ORDER,
    );
    assert_eq!(use_case.name(), "join-room");
    assert_eq!(use_case.steps(), domain::ENTRYPOINTLICATION_USE_CASE_ORDER);
    touch_eq(use_case);
    for step in domain::ENTRYPOINTLICATION_USE_CASE_ORDER {
        touch_hash(*step);
    }

    touch_eq(features::CoreFeaturesSurface);
    for class in [
        features::ExcludedFeatureClass::ChatApplicationSemantics,
        features::ExcludedFeatureClass::RecordingWorkflow,
        features::ExcludedFeatureClass::ScreenShareWorkflow,
        features::ExcludedFeatureClass::DataChannelApplicationSemantics,
        features::ExcludedFeatureClass::UiEndUserWorkflow,
        features::ExcludedFeatureClass::MediaCaptureWorkflow,
        features::ExcludedFeatureClass::RegulatedDomainWorkflow,
    ] {
        touch_hash(class);
    }
    for surface in [
        features::FeatureRequestedSurface::Core,
        features::FeatureRequestedSurface::Signaling,
        features::FeatureRequestedSurface::Sfu,
        features::FeatureRequestedSurface::Turn,
        features::FeatureRequestedSurface::Sdk,
        features::FeatureRequestedSurface::Driver,
        features::FeatureRequestedSurface::Entrypoints,
        features::FeatureRequestedSurface::Regulated,
    ] {
        touch_hash(surface);
    }
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
        touch_hash(requirement);
    }
    for decision_class in [
        features::FeatureAdmissionDecisionClass::Rejected,
        features::FeatureAdmissionDecisionClass::CloseNotClaimed,
        features::FeatureAdmissionDecisionClass::AdmittedByCanonical,
    ] {
        touch_hash(decision_class);
    }
    let feature_decision = features::FeatureAdmissionDecision::new(
        Some(correlation("feature-admission")),
        features::ExcludedFeatureClass::DataChannelApplicationSemantics,
        features::FeatureRequestedSurface::Sdk,
        features::FeatureAdmissionDecisionClass::Rejected,
        Some(features::FeatureAdmissionFailureKind::DataChannelNotSupported),
    );
    assert_eq!(
        feature_decision.feature_class(),
        features::ExcludedFeatureClass::DataChannelApplicationSemantics
    );
    assert_eq!(
        feature_decision.requested_surface(),
        features::FeatureRequestedSurface::Sdk
    );
    assert_eq!(
        feature_decision.decision_class(),
        features::FeatureAdmissionDecisionClass::Rejected
    );
    touch_eq(feature_decision);
    for failure in [
        features::FeatureAdmissionFailureKind::FeatureOutOfScope,
        features::FeatureAdmissionFailureKind::FeatureAdmissionNotDocumented,
        features::FeatureAdmissionFailureKind::ChatNotSupported,
        features::FeatureAdmissionFailureKind::RecordingNotSupported,
        features::FeatureAdmissionFailureKind::ScreenShareNotSupported,
        features::FeatureAdmissionFailureKind::DataChannelNotSupported,
        features::FeatureAdmissionFailureKind::UiWorkflowNotSupported,
        features::FeatureAdmissionFailureKind::MediaCaptureNotSupported,
        features::FeatureAdmissionFailureKind::RegulatedWorkflowNotSupported,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }

    touch_eq(state::CoreStateSurface);
    for (class, rule, source_of_truth) in [
        (
            state::StateClass::EphemeralCoreState,
            state::PersistenceRule::PersistenceNotRequired,
            true,
        ),
        (
            state::StateClass::CheckpointEligibleState,
            state::PersistenceRule::PersistOnlyViaPersistencePort,
            true,
        ),
        (
            state::StateClass::AuditOnlyState,
            state::PersistenceRule::AuditSinkOrHashChainOnly,
            true,
        ),
        (
            state::StateClass::DriverLocalState,
            state::PersistenceRule::DriverOwnedNotSourceOfTruth,
            false,
        ),
        (
            state::StateClass::ConfigurationScopeState,
            state::PersistenceRule::ConfigurationDecisionStartupReferences,
            true,
        ),
        (
            state::StateClass::SdkLocalState,
            state::PersistenceRule::SdkOwnedNotServerState,
            false,
        ),
    ] {
        touch_hash(class);
        touch_hash(rule);
        assert_eq!(class.persistence_rule(), rule);
        assert_eq!(class.can_be_core_source_of_truth(), source_of_truth);
    }
    for family in [
        state::StateFamily::SignalingRoom,
        state::StateFamily::SignalingParticipant,
        state::StateFamily::SignalingIdempotency,
        state::StateFamily::SfuForwardingState,
        state::StateFamily::TurnRelayAuthorizationState,
        state::StateFamily::AuditEvent,
        state::StateFamily::AuditHashChainRecord,
        state::StateFamily::AtomicityCompensationEvidence,
        state::StateFamily::ResourceBoundCounters,
        state::StateFamily::DriverRetryStore,
        state::StateFamily::MetricsBacklog,
        state::StateFamily::ConfigurationDecision,
        state::StateFamily::SdkConnectionState,
    ] {
        touch_hash(family);
        touch_hash(family.default_class());
        assert!(!family.implicit_durable_source_of_truth_allowed());
    }
    for owner in [
        state::CheckpointOwnerBoundary::CoreIntentAndVersion,
        state::CheckpointOwnerBoundary::DriverSchemaAndStorageLayout,
    ] {
        touch_hash(owner);
    }
    assert_eq!(
        state::CheckpointIntent::try_new(
            state::StateFamily::SfuForwardingState,
            state::StateClass::EphemeralCoreState,
            state::CheckpointOwnerBoundary::CoreIntentAndVersion,
            true,
            false,
        ),
        Err(state::CheckpointIntentError::StateClassNotCheckpointEligible)
    );
    assert_eq!(
        state::CheckpointIntent::try_new(
            state::StateFamily::SignalingIdempotency,
            state::StateClass::CheckpointEligibleState,
            state::CheckpointOwnerBoundary::CoreIntentAndVersion,
            false,
            false,
        ),
        Err(state::CheckpointIntentError::RestorePolicyRequired)
    );
    touch_hash(
        state::CheckpointIntent::try_new(
            state::StateFamily::SignalingIdempotency,
            state::StateClass::CheckpointEligibleState,
            state::CheckpointOwnerBoundary::CoreIntentAndVersion,
            true,
            true,
        )
        .expect("checkpoint intent includes restore policy"),
    );
    for claim in [
        state::SourceOfTruthClaim::NoImplicitDurableDomainSourceOfTruth,
        state::SourceOfTruthClaim::AuditReplayVerificationOnly,
        state::SourceOfTruthClaim::RestorePolicyRequired,
        state::SourceOfTruthClaim::DistributedPolicyRequired,
    ] {
        touch_hash(claim);
    }
    for failure in [
        state::StatePersistenceFailureKind::PersistenceUnavailable,
        state::StatePersistenceFailureKind::PersistenceRetryBoundExceeded,
        state::StatePersistenceFailureKind::PersistenceRetryDurationExceeded,
        state::StatePersistenceFailureKind::AuditBacklogBoundExceeded,
        state::StatePersistenceFailureKind::DriverShutdown,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for prohibited in [
        state::ProhibitedStatePersistenceBehavior::DriverSchemaAsDomainSourceOfTruth,
        state::ProhibitedStatePersistenceBehavior::CheckpointRestoreWithoutRestoreCanonical,
        state::ProhibitedStatePersistenceBehavior::SfuRouteDurableByDefault,
        state::ProhibitedStatePersistenceBehavior::TurnAllocationSilentlyRestored,
        state::ProhibitedStatePersistenceBehavior::AuditLogAsMutableStateStore,
        state::ProhibitedStatePersistenceBehavior::DriverDbTransactionAsAggregateCommitAuthority,
        state::ProhibitedStatePersistenceBehavior::SdkLocalStateAsServerParticipantState,
        state::ProhibitedStatePersistenceBehavior::DriverRetryQueueAsDomainState,
        state::ProhibitedStatePersistenceBehavior::PersistedStateAsFailoverReadyWithoutPolicy,
    ] {
        touch_hash(prohibited);
    }

    touch_eq(security::CoreSecuritySurface);
    for class in [
        security::AuthorizationContextClass::VerifiedCredentialContext,
        security::AuthorizationContextClass::ParticipantJoinContext,
        security::AuthorizationContextClass::PublicationPolicyContext,
        security::AuthorizationContextClass::SubscriptionPolicyContext,
        security::AuthorizationContextClass::TurnRelayPolicyContext,
        security::AuthorizationContextClass::AdmissionPolicyContext,
        security::AuthorizationContextClass::RegulatedAuthorizationContext,
    ] {
        touch_hash(class);
    }
    for source in [
        security::AuthorizationMappingSource::VerifiedCredential,
        security::AuthorizationMappingSource::ExternalApplicationContext,
        security::AuthorizationMappingSource::EdgeTrustedContext,
        security::AuthorizationMappingSource::RegulatedContext,
    ] {
        touch_hash(source);
    }
    for action in [
        security::CommunicationAction::Join,
        security::CommunicationAction::Publish,
        security::CommunicationAction::Subscribe,
        security::CommunicationAction::TurnAllocate,
        security::CommunicationAction::TurnPermission,
        security::CommunicationAction::TurnRelay,
        security::CommunicationAction::Admission,
    ] {
        touch_hash(action);
    }
    for lifetime in [
        security::AuthorizationLifetime::SingleDecision,
        security::AuthorizationLifetime::UntilCredentialExpiry,
        security::AuthorizationLifetime::Bounded("authorization-window"),
    ] {
        touch_hash(lifetime);
    }
    for redaction in [
        security::AuthorizationRedactionRule::RawClaimsExcluded,
        security::AuthorizationRedactionRule::SensitivePayloadExcluded,
    ] {
        touch_hash(redaction);
    }
    let mapping = security::AuthorizationMapping::new(
        security::AuthorizationMappingSource::VerifiedCredential,
        security::AuthorizationContextClass::TurnRelayPolicyContext,
        vec![
            command::TargetSurface::Turn,
            command::TargetSurface::Security,
        ],
        security::AuthorizationLifetime::UntilCredentialExpiry,
        security::AuthorizationRedactionRule::RawClaimsExcluded,
    );
    assert_eq!(
        mapping.context_class(),
        security::AuthorizationContextClass::TurnRelayPolicyContext
    );
    assert_eq!(
        mapping.lifetime(),
        security::AuthorizationLifetime::UntilCredentialExpiry
    );
    assert_eq!(mapping.allowed_target_surfaces().len(), 2);
    touch_eq(mapping);

    let policy_input = security::AuthorizationPolicyInput::new(
        correlation("auth-policy"),
        Some(credential("auth-credential")),
        security::AuthorizationContextClass::TurnRelayPolicyContext,
        command::TargetSurface::Turn,
        security::CommunicationAction::TurnRelay,
        allocation("auth-allocation"),
    );
    assert_eq!(policy_input.target_surface(), command::TargetSurface::Turn);
    assert_eq!(
        policy_input.action(),
        security::CommunicationAction::TurnRelay
    );
    touch_eq(policy_input);

    assert_eq!(
        security::AuthorizationPolicyDecision::new(
            command::UseCaseOutcome::Accepted,
            command::DecisionReason::Cataloged(
                security::AuthorizationFailureKind::RuntimeConfigInvalid
            ),
        ),
        Err(security::AuthorizationPolicyError::SuccessMustNotCarryReason)
    );
    assert_eq!(
        security::AuthorizationPolicyDecision::<security::AuthorizationFailureKind>::new(
            command::UseCaseOutcome::Denied,
            command::DecisionReason::Absent,
        ),
        Err(security::AuthorizationPolicyError::NonSuccessRequiresReason)
    );
    let auth_decision = security::AuthorizationPolicyDecision::new(
        command::UseCaseOutcome::Denied,
        command::DecisionReason::Cataloged(
            security::AuthorizationFailureKind::AuthorizationPolicyDenied,
        ),
    )
    .expect("non-success authorization decision carries a reason");
    assert_eq!(auth_decision.outcome(), command::UseCaseOutcome::Denied);
    assert!(matches!(
        auth_decision.reason(),
        command::DecisionReason::Cataloged(
            security::AuthorizationFailureKind::AuthorizationPolicyDenied
        )
    ));
    touch_eq(auth_decision);

    for failure in [
        security::AuthorizationFailureKind::AuthorizationContextMissing,
        security::AuthorizationFailureKind::AuthorizationContextInvalid,
        security::AuthorizationFailureKind::AuthorizationContextExpired,
        security::AuthorizationFailureKind::AuthorizationPolicyDenied,
        security::AuthorizationFailureKind::AuthorizationScopeNotAllowed,
        security::AuthorizationFailureKind::ForwardedHeaderUntrusted,
        security::AuthorizationFailureKind::ClientAddressUntrusted,
        security::AuthorizationFailureKind::TokenVerificationFailed,
        security::AuthorizationFailureKind::RuntimeConfigMissing,
        security::AuthorizationFailureKind::RuntimeConfigInvalid,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
    for claim in [
        security::RequiredTokenClaim::Issuer,
        security::RequiredTokenClaim::Audience,
        security::RequiredTokenClaim::Subject,
        security::RequiredTokenClaim::Expiration,
        security::RequiredTokenClaim::NotBefore,
        security::RequiredTokenClaim::IssuedAt,
        security::RequiredTokenClaim::KeyId,
    ] {
        touch_hash(claim);
    }
    let issuer_policy = security::IssuerPolicy::new(vec!["issuer-a", "issuer-b"]);
    assert_eq!(issuer_policy.accepted_issuers(), &["issuer-a", "issuer-b"]);
    touch_eq(issuer_policy.clone());
    let audience_policy = security::AudiencePolicy::new(vec!["aud-a"]);
    assert_eq!(audience_policy.accepted_audiences(), &["aud-a"]);
    touch_eq(audience_policy.clone());
    let algorithm_policy = security::TokenAlgorithmPolicy::new(vec!["EdDSA", "ES256"]);
    assert_eq!(algorithm_policy.accepted_algorithms(), &["EdDSA", "ES256"]);
    touch_eq(algorithm_policy.clone());
    let verification_request = security::TokenVerificationRequest::new(
        correlation("token-verification"),
        credential("token-credential"),
        vec![
            security::RequiredTokenClaim::Issuer,
            security::RequiredTokenClaim::Audience,
        ],
        issuer_policy,
        audience_policy,
        algorithm_policy,
    );
    assert_eq!(
        verification_request.credential_ref(),
        &credential("token-credential")
    );
    assert_eq!(verification_request.required_claims().len(), 2);
    touch_eq(verification_request);

    for temporal in [
        security::TokenTemporalDecision::Valid,
        security::TokenTemporalDecision::Expired,
        security::TokenTemporalDecision::NotYetValid,
    ] {
        touch_hash(temporal);
    }
    let verified = security::VerifiedCredential::new(
        correlation("verified-credential"),
        credential("verified-credential"),
        security::TokenTemporalDecision::Valid,
    );
    assert_eq!(
        verified.credential_ref(),
        &credential("verified-credential")
    );
    assert_eq!(
        verified.temporal_decision(),
        security::TokenTemporalDecision::Valid
    );
    touch_eq(verified.clone());
    touch_eq(security::TokenVerificationResult::Accepted(verified));
    touch_eq(security::TokenVerificationResult::Rejected(
        security::TokenVerificationFailureKind::TokenMalformed,
    ));
    for failure in [
        security::TokenVerificationFailureKind::TokenMissing,
        security::TokenVerificationFailureKind::TokenMalformed,
        security::TokenVerificationFailureKind::TokenSignatureInvalid,
        security::TokenVerificationFailureKind::TokenKeyUnavailable,
        security::TokenVerificationFailureKind::TokenIssuerMismatch,
        security::TokenVerificationFailureKind::TokenAudienceMismatch,
        security::TokenVerificationFailureKind::TokenExpired,
        security::TokenVerificationFailureKind::TokenNotYetValid,
        security::TokenVerificationFailureKind::TokenRequiredClaimMissing,
        security::TokenVerificationFailureKind::TokenUnsupportedAlgorithm,
    ] {
        touch_hash(failure);
        assert!(CatalogedReasonRef::from_code(failure.reason_code()).is_ok());
    }
}
