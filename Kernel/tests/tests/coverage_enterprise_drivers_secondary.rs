use arcrtc_core_ports::{CorePort, MetricsSinkFailureKind, PersistencePortFailureKind, PortFamily};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_security::TokenVerificationFailureKind;
use arcrtc_core_state::StateClass;
use arcrtc_driver_observability::{
    MetricsExportBacklogBound, MetricsExportBacklogBoundError, ObservabilityDriverSurface,
    ObservabilityFailureSource, ObservabilityMetricsSinkDriverPort, ObservabilityProjectionClass,
    ObservabilityProjectionError, ObservabilityProjectionGuard, ObservabilitySensitiveDataError,
    ObservabilitySensitiveDataGuard, ObservabilitySignalAuditShape,
    ObservabilitySignalAuditShapeError, ObservabilitySignalClass, ObservabilitySignalDescriptor,
    ObservabilitySignalDescriptorError, ObservabilitySignalFailure, ObservabilitySignalFailureKind,
    ObservabilitySignalOwner, PrivacyDataClass, PrivacyLabelGuard, PrivacyLabelGuardError,
    PrivacyRedactionAdmission, PrivacyRedactionAdmissionError, PrivacyRedactionRetentionFailure,
    PrivacyRedactionRetentionFailureKind, PrivacyRetentionBoundClass, PrivacyRetentionOwner,
    PrivacyRetentionPolicy, PrivacyRetentionPolicyError, PrivacyRetentionTarget,
    PrivacyTargetSurface, ProhibitedObservabilityBoundaryBehavior,
    ProhibitedObservabilitySignalTaxonomyBehavior, ProhibitedPrivacyRedactionRetentionBehavior,
    RedactedOutputClass, SignalCardinalityClass, SignalReferenceClass, SignalSamplingGuard,
    SignalSamplingGuardError, SignalSamplingRule, SignalSensitiveDataClass, SignalUseClass,
};
use arcrtc_driver_persistence::{
    ArtifactIntegrityClass, ArtifactIntegrityGuard, ArtifactIntegrityGuardError,
    ArtifactRestoreImportGuard, ArtifactRestoreImportGuardError, ArtifactSensitiveDataError,
    ArtifactSensitiveDataGuard, ExportBackupArtifactAuditShape,
    ExportBackupArtifactAuditShapeError, ExportBackupArtifactFailure,
    ExportBackupArtifactFailureKind, MigrationCompatibilityRule, MigrationCompatibilityRuleError,
    MigrationModeSelectionOwner, PersistenceBackendClass, PersistenceDriverAdmissionError,
    PersistenceDriverAdmissionGuard, PersistenceDriverFailureSource, PersistenceDriverPort,
    PersistenceDriverSurface, PersistenceRetryBoundError, PersistenceRetryBoundGuard,
    PersistenceStorageShapeError, PersistenceStorageShapeGuard,
    ProhibitedExportBackupArtifactBehavior, ProhibitedPersistenceDriverBehavior,
    ProhibitedSchemaMigrationBehavior, RestoreReplayApplicability, SchemaMigrationClass,
    SchemaMigrationExecutionGuard, SchemaMigrationExecutionGuardError, SchemaMigrationFailure,
    SchemaMigrationFailureKind, UnknownPersistedFieldHandling,
};
use arcrtc_driver_security::{
    KeyCacheRefreshBounds, KeyCacheRefreshBoundsError, KeyLookupRefreshError,
    KeyLookupRefreshGuard, KeyLookupRefreshState, KeySourceConfigurationError,
    KeySourceConfigurationGuard, KeySourceConfigurationOrigin,
    ProhibitedSecretRotationLifecycleBehavior, ProhibitedSecurityVerifierDriverBehavior,
    SecretGenerationState, SecretGenerationStateAdmission, SecretGenerationStateAdmissionError,
    SecretRotationClass, SecretRotationExecutionBoundaryError,
    SecretRotationExecutionBoundaryGuard, SecretRotationFailure, SecretRotationFailureKind,
    SecretRotationPolicyError, SecretRotationPolicyGuard, SecurityDriverSurface,
    SecurityKeySourceType, SecurityTokenVerifierDriverPort, TokenVerifierDriverBoundaryError,
    TokenVerifierDriverBoundaryGuard, VerifierBackendFailureClass, VerifierDriverFailure,
};

fn assert_cataloged(code: &str) {
    let reason = CatalogedReasonRef::from_code(code).expect("reason code must be cataloged");
    assert_eq!(reason.definition().code().as_str(), code);
}

fn assert_debug_contains_reason<T: core::fmt::Debug>(value: T, reason_code: &str) {
    assert!(
        format!("{value:?}").contains(reason_code),
        "debug output must expose the connected closed reason code"
    );
}

#[test]
fn coverage_observability_guards_cover_success_and_fail_closed_edges() {
    assert_eq!(
        format!("{ObservabilityDriverSurface:?}"),
        "ObservabilityDriverSurface"
    );
    assert_eq!(
        <ObservabilityMetricsSinkDriverPort as CorePort>::FAMILY,
        PortFamily::MetricsSink
    );

    for projection_class in [
        ObservabilityProjectionClass::Metric,
        ObservabilityProjectionClass::Log,
        ObservabilityProjectionClass::Trace,
    ] {
        assert!(ObservabilityProjectionGuard::try_new(
            projection_class,
            true,
            true,
            true,
            true,
            true,
            true
        )
        .is_ok());
    }
    assert_eq!(
        ObservabilityProjectionGuard::try_new(
            ObservabilityProjectionClass::Metric,
            false,
            true,
            true,
            true,
            true,
            true
        ),
        Err(ObservabilityProjectionError::AuditMeaningTakenByDriver)
    );
    assert_eq!(
        ObservabilityProjectionGuard::try_new(
            ObservabilityProjectionClass::Metric,
            true,
            false,
            true,
            true,
            true,
            true
        ),
        Err(ObservabilityProjectionError::DecisionSemanticsChangedByProjection)
    );
    assert_eq!(
        ObservabilityProjectionGuard::try_new(
            ObservabilityProjectionClass::Log,
            true,
            true,
            true,
            true,
            false,
            true
        ),
        Err(ObservabilityProjectionError::FreeTextAsAuthority)
    );
    assert!(ObservabilitySensitiveDataGuard::try_new(true, true, true, true, true, true).is_ok());
    assert_eq!(
        ObservabilitySensitiveDataGuard::try_new(false, true, true, true, true, true),
        Err(ObservabilitySensitiveDataError::SensitiveMaterialInSignal)
    );
    assert_eq!(
        ObservabilitySensitiveDataGuard::try_new(true, true, true, true, true, false),
        Err(ObservabilitySensitiveDataError::NonOpaqueReferenceInSignal)
    );

    let backlog = MetricsExportBacklogBound::try_new(16, 4096).expect("bounded backlog");
    assert_eq!(
        backlog.bound_exceeded_failure().kind(),
        MetricsSinkFailureKind::MetricsBacklogBoundExceeded
    );
    assert_eq!(
        backlog
            .bound_exceeded_failure()
            .reason()
            .definition()
            .code()
            .as_str(),
        MetricsSinkFailureKind::MetricsBacklogBoundExceeded.reason_code()
    );
    assert_eq!(
        MetricsExportBacklogBound::try_new(0, 4096),
        Err(MetricsExportBacklogBoundError::UnboundedMetricsBacklog)
    );

    for (source, expected_kind) in [
        (
            ObservabilityFailureSource::MetricsExportFailed,
            MetricsSinkFailureKind::MetricsExportFailed,
        ),
        (
            ObservabilityFailureSource::MetricsBacklogBoundExceeded,
            MetricsSinkFailureKind::MetricsBacklogBoundExceeded,
        ),
        (
            ObservabilityFailureSource::ObservabilitySignalInvalid,
            MetricsSinkFailureKind::ObservabilitySignalInvalid,
        ),
        (
            ObservabilityFailureSource::MetricCardinalityExceeded,
            MetricsSinkFailureKind::MetricCardinalityExceeded,
        ),
        (
            ObservabilityFailureSource::TelemetrySamplingPolicyMissing,
            MetricsSinkFailureKind::TelemetrySamplingPolicyMissing,
        ),
        (
            ObservabilityFailureSource::DriverShutdown,
            MetricsSinkFailureKind::DriverShutdown,
        ),
    ] {
        let failure = source.to_metrics_failure();
        assert_eq!(failure.kind(), expected_kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            expected_kind.reason_code()
        );
    }

    for (signal_class, owner, use_class) in [
        (
            ObservabilitySignalClass::AuditSignal,
            ObservabilitySignalOwner::CoreAudit,
            SignalUseClass::AuditRecord,
        ),
        (
            ObservabilitySignalClass::QualityDecisionMetric,
            ObservabilitySignalOwner::CoreQualityPolicy,
            SignalUseClass::QualityDecisionInput,
        ),
        (
            ObservabilitySignalClass::ResourceBoundMetric,
            ObservabilitySignalOwner::CoreResourceBoundPolicy,
            SignalUseClass::ResourceBoundDecisionInput,
        ),
        (
            ObservabilitySignalClass::OperationalMetric,
            ObservabilitySignalOwner::DriverObservability,
            SignalUseClass::OperationalObservation,
        ),
        (
            ObservabilitySignalClass::TraceSpan,
            ObservabilitySignalOwner::DriverObservability,
            SignalUseClass::DiagnosticOnly,
        ),
        (
            ObservabilitySignalClass::StructuredLog,
            ObservabilitySignalOwner::DriverObservability,
            SignalUseClass::OperationalObservation,
        ),
        (
            ObservabilitySignalClass::AlertSignal,
            ObservabilitySignalOwner::EntrypointsOperationsPolicy,
            SignalUseClass::DiagnosticOnly,
        ),
        (
            ObservabilitySignalClass::ProfilingSignal,
            ObservabilitySignalOwner::DriverObservability,
            SignalUseClass::DiagnosticOnly,
        ),
    ] {
        assert_eq!(signal_class.required_owner(), owner);
        assert_eq!(signal_class.use_class(), use_class);
        assert!(ObservabilitySignalDescriptor::try_new(
            signal_class,
            owner,
            vec![SignalReferenceClass::CoreOpaqueReference],
            SignalSensitiveDataClass::NoSensitiveMaterial,
            SignalCardinalityClass::FixedLow,
            SignalSamplingRule::NotSampled,
            true
        )
        .is_ok());
    }

    assert_eq!(
        ObservabilitySignalDescriptor::try_new(
            ObservabilitySignalClass::AuditSignal,
            ObservabilitySignalOwner::DriverObservability,
            vec![SignalReferenceClass::AuditReference],
            SignalSensitiveDataClass::NoSensitiveMaterial,
            SignalCardinalityClass::FixedLow,
            SignalSamplingRule::NotSampled,
            true
        ),
        Err(ObservabilitySignalDescriptorError::SignalOwnerMismatch)
    );
    assert_eq!(
        ObservabilitySignalDescriptor::try_new(
            ObservabilitySignalClass::OperationalMetric,
            ObservabilitySignalOwner::DriverObservability,
            Vec::new(),
            SignalSensitiveDataClass::NoSensitiveMaterial,
            SignalCardinalityClass::FixedLow,
            SignalSamplingRule::NotSampled,
            true
        ),
        Err(ObservabilitySignalDescriptorError::ReferenceClassMissing)
    );
    assert_eq!(
        ObservabilitySignalDescriptor::try_new(
            ObservabilitySignalClass::OperationalMetric,
            ObservabilitySignalOwner::DriverObservability,
            vec![SignalReferenceClass::NoReference],
            SignalSensitiveDataClass::SensitiveMaterialRejected,
            SignalCardinalityClass::FixedLow,
            SignalSamplingRule::NotSampled,
            true
        ),
        Err(ObservabilitySignalDescriptorError::SensitiveDataClassInvalid)
    );
    assert_eq!(
        ObservabilitySignalDescriptor::try_new(
            ObservabilitySignalClass::OperationalMetric,
            ObservabilitySignalOwner::DriverObservability,
            vec![SignalReferenceClass::ResourceOwnerReference],
            SignalSensitiveDataClass::NoSensitiveMaterial,
            SignalCardinalityClass::Unbounded,
            SignalSamplingRule::NotSampled,
            true
        ),
        Err(ObservabilitySignalDescriptorError::UnboundedCardinality)
    );
    assert_eq!(
        ObservabilitySignalDescriptor::try_new(
            ObservabilitySignalClass::OperationalMetric,
            ObservabilitySignalOwner::DriverObservability,
            vec![SignalReferenceClass::ResourceOwnerReference],
            SignalSensitiveDataClass::NoSensitiveMaterial,
            SignalCardinalityClass::Bounded {
                maximum_distinct_values: 0
            },
            SignalSamplingRule::NotSampled,
            true
        ),
        Err(ObservabilitySignalDescriptorError::UnboundedCardinality)
    );
    assert_eq!(
        ObservabilitySignalDescriptor::try_new(
            ObservabilitySignalClass::OperationalMetric,
            ObservabilitySignalOwner::DriverObservability,
            vec![SignalReferenceClass::ResourceOwnerReference],
            SignalSensitiveDataClass::NoSensitiveMaterial,
            SignalCardinalityClass::FixedLow,
            SignalSamplingRule::MissingPolicy,
            true
        ),
        Err(ObservabilitySignalDescriptorError::SamplingPolicyMissing)
    );
    assert_eq!(
        ObservabilitySignalDescriptor::try_new(
            ObservabilitySignalClass::OperationalMetric,
            ObservabilitySignalOwner::DriverObservability,
            vec![SignalReferenceClass::ResourceOwnerReference],
            SignalSensitiveDataClass::NoSensitiveMaterial,
            SignalCardinalityClass::FixedLow,
            SignalSamplingRule::NotSampled,
            false
        ),
        Err(ObservabilitySignalDescriptorError::RetentionRedactionRuleMissing)
    );
    assert!(SignalSamplingGuard::try_new(
        ObservabilitySignalClass::AuditSignal,
        SignalSamplingRule::SamplingForbidden,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        SignalSamplingGuard::try_new(
            ObservabilitySignalClass::OperationalMetric,
            SignalSamplingRule::MissingPolicy,
            true,
            true,
            true
        ),
        Err(SignalSamplingGuardError::SamplingPolicyMissing)
    );
    assert_eq!(
        SignalSamplingGuard::try_new(
            ObservabilitySignalClass::AuditSignal,
            SignalSamplingRule::SamplingForbidden,
            false,
            true,
            true
        ),
        Err(SignalSamplingGuardError::SamplingHidesRequiredSignal)
    );
    assert!(ObservabilitySignalAuditShape::try_new(true, true, true, true, true).is_ok());
    assert_eq!(
        ObservabilitySignalAuditShape::try_new(true, true, false, true, true),
        Err(ObservabilitySignalAuditShapeError::RequiredSignalAuditFieldMissing)
    );
}

#[test]
fn coverage_observability_privacy_reasons_and_prohibited_enums_are_closed() {
    assert!(PrivacyDataClass::RawSecret.is_raw_sensitive());
    assert!(PrivacyDataClass::RawToken.requires_opaque_drop_or_reject());
    assert!(!PrivacyDataClass::RegulatedPayload.requires_opaque_drop_or_reject());
    assert!(!PrivacyDataClass::CatalogReason.is_raw_sensitive());
    assert!(PrivacyDataClass::RawSecret.is_forbidden_retention_for(PrivacyRetentionTarget::Metrics));
    assert!(PrivacyDataClass::RegulatedPayload
        .is_forbidden_retention_for(PrivacyRetentionTarget::SdkClientLocalData));
    assert!(!PrivacyDataClass::RegulatedPayload
        .is_forbidden_retention_for(PrivacyRetentionTarget::RegulatedOnly));

    assert!(PrivacyRedactionAdmission::try_new(
        PrivacyDataClass::NonSensitiveTag,
        PrivacyTargetSurface::ObservabilitySink,
        RedactedOutputClass::AllowlistedField,
        true,
        true,
        true,
        true,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        PrivacyRedactionAdmission::try_new(
            PrivacyDataClass::NonSensitiveTag,
            PrivacyTargetSurface::ObservabilitySink,
            RedactedOutputClass::AllowlistedField,
            true,
            true,
            true,
            false,
            true,
            true,
            true
        ),
        Err(PrivacyRedactionAdmissionError::ExternalSinkDefaultReliedOn)
    );
    assert_eq!(
        PrivacyRedactionAdmission::try_new(
            PrivacyDataClass::RawToken,
            PrivacyTargetSurface::ObservabilitySink,
            RedactedOutputClass::Dropped,
            false,
            false,
            true,
            true,
            true,
            true,
            true
        ),
        Err(PrivacyRedactionAdmissionError::RawSensitiveMaterialPresent)
    );
    assert_eq!(
        PrivacyRedactionAdmission::try_new(
            PrivacyDataClass::NonSensitiveTag,
            PrivacyTargetSurface::ObservabilitySink,
            RedactedOutputClass::AllowlistedField,
            true,
            false,
            true,
            true,
            true,
            true,
            true
        ),
        Err(PrivacyRedactionAdmissionError::FieldNotAllowlisted)
    );
    assert_eq!(
        PrivacyRedactionAdmission::try_new(
            PrivacyDataClass::RawKeyMaterial,
            PrivacyTargetSurface::DriverKeyCache,
            RedactedOutputClass::RedactedExcerpt,
            true,
            false,
            true,
            true,
            true,
            true,
            true
        ),
        Err(PrivacyRedactionAdmissionError::RedactedReferenceMissing)
    );
    assert_eq!(
        PrivacyRedactionAdmission::try_new(
            PrivacyDataClass::RawKeyMaterial,
            PrivacyTargetSurface::DriverKeyCache,
            RedactedOutputClass::OpaqueReference,
            true,
            false,
            false,
            true,
            true,
            true,
            true
        ),
        Err(PrivacyRedactionAdmissionError::RedactedReferenceMissing)
    );
    assert_eq!(
        PrivacyRedactionAdmission::try_new(
            PrivacyDataClass::DiagnosticDetail,
            PrivacyTargetSurface::ObservabilitySink,
            RedactedOutputClass::RedactedExcerpt,
            true,
            false,
            true,
            true,
            false,
            true,
            true
        ),
        Err(PrivacyRedactionAdmissionError::DiagnosticDetailAuthoritative)
    );
    assert_eq!(
        PrivacyRedactionAdmission::try_new(
            PrivacyDataClass::RegulatedPayload,
            PrivacyTargetSurface::ObservabilitySink,
            RedactedOutputClass::Rejected,
            true,
            false,
            true,
            true,
            true,
            false,
            true
        ),
        Err(PrivacyRedactionAdmissionError::RegulatedPayloadInGenericPath)
    );
    assert_eq!(
        PrivacyRedactionAdmission::try_new(
            PrivacyDataClass::EdgeProxyMetadata,
            PrivacyTargetSurface::ObservabilitySink,
            RedactedOutputClass::RedactedExcerpt,
            true,
            false,
            true,
            true,
            true,
            true,
            false
        ),
        Err(PrivacyRedactionAdmissionError::EdgeProxyTrustClassMissing)
    );

    assert_eq!(
        PrivacyRetentionTarget::AuditEvent.required_owner(),
        PrivacyRetentionOwner::Core
    );
    assert_eq!(
        PrivacyRetentionTarget::PacketCache.required_owner(),
        PrivacyRetentionOwner::Driver
    );
    assert!(PrivacyRetentionPolicy::try_new(
        PrivacyRetentionTarget::AuditEvent,
        PrivacyRetentionOwner::Core,
        PrivacyDataClass::CatalogReason,
        PrivacyRetentionBoundClass::ClosedFieldSet,
        true,
        true,
        true,
        true
    )
    .is_ok());
    assert!(PrivacyRetentionPolicy::try_new(
        PrivacyRetentionTarget::PacketCache,
        PrivacyRetentionOwner::Driver,
        PrivacyDataClass::RawPacketPayload,
        PrivacyRetentionBoundClass::ResourceBoundPolicy,
        true,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        PrivacyRetentionPolicy::try_new(
            PrivacyRetentionTarget::AuditEvent,
            PrivacyRetentionOwner::Driver,
            PrivacyDataClass::CatalogReason,
            PrivacyRetentionBoundClass::ClosedFieldSet,
            true,
            true,
            true,
            true
        ),
        Err(PrivacyRetentionPolicyError::RetentionOwnerMismatch)
    );
    assert_eq!(
        PrivacyRetentionPolicy::try_new(
            PrivacyRetentionTarget::AuditEvent,
            PrivacyRetentionOwner::Core,
            PrivacyDataClass::CatalogReason,
            PrivacyRetentionBoundClass::Unbounded,
            true,
            true,
            true,
            true
        ),
        Err(PrivacyRetentionPolicyError::RetentionBoundMissing)
    );
    assert_eq!(
        PrivacyRetentionPolicy::try_new(
            PrivacyRetentionTarget::LogTrace,
            PrivacyRetentionOwner::Driver,
            PrivacyDataClass::RawSecret,
            PrivacyRetentionBoundClass::SpecializedRetentionPolicy,
            true,
            true,
            true,
            true
        ),
        Err(PrivacyRetentionPolicyError::DataClassCannotBeRetained)
    );
    assert_eq!(
        PrivacyRetentionPolicy::try_new(
            PrivacyRetentionTarget::AuditHashChain,
            PrivacyRetentionOwner::Core,
            PrivacyDataClass::CatalogReason,
            PrivacyRetentionBoundClass::ClosedFieldSet,
            true,
            true,
            false,
            true
        ),
        Err(PrivacyRetentionPolicyError::HashChainRetainsMutableState)
    );
    assert_eq!(
        PrivacyRetentionPolicy::try_new(
            PrivacyRetentionTarget::KeyCache,
            PrivacyRetentionOwner::Driver,
            PrivacyDataClass::RawKeyMaterial,
            PrivacyRetentionBoundClass::ClosedFieldSet,
            true,
            true,
            true,
            false
        ),
        Err(PrivacyRetentionPolicyError::DriverCacheNotBoundedLocal)
    );

    assert!(PrivacyLabelGuard::try_new(
        PrivacyDataClass::NonSensitiveTag,
        true,
        true,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        PrivacyLabelGuard::try_new(
            PrivacyDataClass::NonSensitiveTag,
            false,
            true,
            true,
            true,
            true
        ),
        Err(PrivacyLabelGuardError::LabelNotAllowlisted)
    );
    assert_eq!(
        PrivacyLabelGuard::try_new(PrivacyDataClass::RawToken, true, true, true, true, true),
        Err(PrivacyLabelGuardError::LabelNotAllowlisted)
    );
    assert_eq!(
        PrivacyLabelGuard::try_new(
            PrivacyDataClass::NonSensitiveTag,
            true,
            false,
            true,
            true,
            true
        ),
        Err(PrivacyLabelGuardError::SensitiveLabelValue)
    );
    assert_eq!(
        PrivacyLabelGuard::try_new(
            PrivacyDataClass::NonSensitiveTag,
            true,
            true,
            true,
            false,
            true
        ),
        Err(PrivacyLabelGuardError::UnboundedLabelCardinality)
    );
    assert_eq!(
        PrivacyLabelGuard::try_new(
            PrivacyDataClass::NonSensitiveTag,
            true,
            true,
            true,
            true,
            false
        ),
        Err(PrivacyLabelGuardError::DropOrRewriteMissing)
    );

    for kind in [
        ObservabilitySignalFailureKind::ObservabilitySignalInvalid,
        ObservabilitySignalFailureKind::MetricCardinalityExceeded,
        ObservabilitySignalFailureKind::TelemetrySamplingPolicyMissing,
        ObservabilitySignalFailureKind::AlertSignalNotAllowed,
        ObservabilitySignalFailureKind::ObservabilityExportNotAllowed,
        ObservabilitySignalFailureKind::MetricsExportFailed,
        ObservabilitySignalFailureKind::MetricsBacklogBoundExceeded,
    ] {
        assert_cataloged(kind.reason_code());
        assert_debug_contains_reason(
            ObservabilitySignalFailure::from_kind(kind),
            kind.reason_code(),
        );
    }
    for kind in [
        PrivacyRedactionRetentionFailureKind::ExportRedactionRequired,
        PrivacyRedactionRetentionFailureKind::SecretUnavailable,
        PrivacyRedactionRetentionFailureKind::UnsafeVerificationDetailSuppressed,
        PrivacyRedactionRetentionFailureKind::PacketPayloadExportRejected,
        PrivacyRedactionRetentionFailureKind::SensitiveLabelRejected,
        PrivacyRedactionRetentionFailureKind::SensitiveCardinalityRejected,
        PrivacyRedactionRetentionFailureKind::RetentionPolicyInvalid,
        PrivacyRedactionRetentionFailureKind::RotationDetailExportRejected,
    ] {
        assert_cataloged(kind.reason_code());
        assert_debug_contains_reason(
            PrivacyRedactionRetentionFailure::from_kind(kind),
            kind.reason_code(),
        );
    }

    assert_eq!(
        [
            ProhibitedObservabilityBoundaryBehavior::MetricsExporterOwnsQualityDecision,
            ProhibitedObservabilityBoundaryBehavior::LogTextAsClosedReasonVocabulary,
            ProhibitedObservabilityBoundaryBehavior::SpanNameAsAuditEventType,
            ProhibitedObservabilityBoundaryBehavior::CrossDriverDependencyAltersDomainFlow,
            ProhibitedObservabilityBoundaryBehavior::RawSensitivePayloadLogged,
            ProhibitedObservabilityBoundaryBehavior::MissingMetricsExportIgnoredForVerificationClaim,
            ProhibitedObservabilityBoundaryBehavior::ObservabilitySignalDrivesDomainDecision,
            ProhibitedObservabilityBoundaryBehavior::UnboundedMetricLabelCardinality,
        ]
        .len(),
        8
    );
    assert_eq!(
        [
            ProhibitedObservabilitySignalTaxonomyBehavior::TraceSpanNameAsAuditEventType,
            ProhibitedObservabilitySignalTaxonomyBehavior::AlertStatusAsDomainDecision,
            ProhibitedObservabilitySignalTaxonomyBehavior::SampledMetricAsCompleteAuditRecord,
            ProhibitedObservabilitySignalTaxonomyBehavior::RawSensitiveLabelValue,
            ProhibitedObservabilitySignalTaxonomyBehavior::DashboardStateAsRuntimeCorrectnessProof,
            ProhibitedObservabilitySignalTaxonomyBehavior::ExporterAggregationChangesQualityDecision,
            ProhibitedObservabilitySignalTaxonomyBehavior::DriverSpecificTaxonomyWithoutSourceContract,
        ]
        .len(),
        7
    );
    assert_eq!(
        [
            ProhibitedPrivacyRedactionRetentionBehavior::RawCredentialMaterialExported,
            ProhibitedPrivacyRedactionRetentionBehavior::RawPacketPayloadExported,
            ProhibitedPrivacyRedactionRetentionBehavior::RegulatedPayloadInGenericExport,
            ProhibitedPrivacyRedactionRetentionBehavior::ExternalSinkDefaultRedaction,
            ProhibitedPrivacyRedactionRetentionBehavior::DiagnosticDetailAsAuthoritativeReason,
            ProhibitedPrivacyRedactionRetentionBehavior::SensitiveMaterialRetainedForDiagnostics,
            ProhibitedPrivacyRedactionRetentionBehavior::RawEdgeProxyMetadataAsIdentity,
            ProhibitedPrivacyRedactionRetentionBehavior::UnownedOrUnboundedRetention,
        ]
        .len(),
        8
    );
}

#[test]
fn coverage_persistence_guards_cover_success_and_fail_closed_edges() {
    assert_eq!(
        format!("{PersistenceDriverSurface:?}"),
        "PersistenceDriverSurface"
    );
    assert_eq!(
        <PersistenceDriverPort as CorePort>::FAMILY,
        PortFamily::Persistence
    );

    assert!(PersistenceStorageShapeGuard::try_new(
        PersistenceBackendClass::SqlDatabase,
        true,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        PersistenceStorageShapeGuard::try_new(
            PersistenceBackendClass::ObjectStore,
            false,
            true,
            true,
            true
        ),
        Err(PersistenceStorageShapeError::StorageLayoutExposedAsCoreApi)
    );
    assert_eq!(
        PersistenceStorageShapeGuard::try_new(
            PersistenceBackendClass::FileSystem,
            true,
            true,
            false,
            true
        ),
        Err(PersistenceStorageShapeError::MigrationFileAsDomainModel)
    );
    assert_eq!(
        PersistenceStorageShapeGuard::try_new(
            PersistenceBackendClass::KeyValueStore,
            true,
            true,
            true,
            false
        ),
        Err(PersistenceStorageShapeError::StorageReplicationAsDomainFailoverProof)
    );

    assert!(PersistenceDriverAdmissionGuard::try_new(
        StateClass::CheckpointEligibleState,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        PersistenceDriverAdmissionGuard::try_new(StateClass::CheckpointEligibleState, false, true),
        Err(PersistenceDriverAdmissionError::DriverOwnsDomainSemantics)
    );
    assert_eq!(
        PersistenceDriverAdmissionGuard::try_new(StateClass::CheckpointEligibleState, true, false),
        Err(PersistenceDriverAdmissionError::DriverTransactionAsDomainCommit)
    );

    let retry_bound =
        PersistenceRetryBoundGuard::try_new(true, true, true, true).expect("bounded retry store");
    assert_eq!(
        retry_bound.bound_exceeded_failure().kind(),
        PersistencePortFailureKind::PersistenceRetryBoundExceeded
    );
    assert_eq!(
        retry_bound.duration_exceeded_failure().kind(),
        PersistencePortFailureKind::PersistenceRetryDurationExceeded
    );
    assert_eq!(
        PersistenceRetryBoundGuard::try_new(true, false, true, true),
        Err(PersistenceRetryBoundError::UnboundedRetryStore)
    );

    for (source, expected_kind) in [
        (
            PersistenceDriverFailureSource::ConcreteStoreUnavailable,
            PersistencePortFailureKind::PersistenceUnavailable,
        ),
        (
            PersistenceDriverFailureSource::RetryBoundExceeded,
            PersistencePortFailureKind::PersistenceRetryBoundExceeded,
        ),
        (
            PersistenceDriverFailureSource::RetryDurationExceeded,
            PersistencePortFailureKind::PersistenceRetryDurationExceeded,
        ),
        (
            PersistenceDriverFailureSource::AuditBacklogExceeded,
            PersistencePortFailureKind::AuditBacklogBoundExceeded,
        ),
        (
            PersistenceDriverFailureSource::DriverShutdown,
            PersistencePortFailureKind::DriverShutdown,
        ),
    ] {
        let failure = source.to_port_failure();
        assert_eq!(failure.kind(), expected_kind);
        assert_eq!(
            failure.reason().definition().code().as_str(),
            expected_kind.reason_code()
        );
    }

    assert!(MigrationCompatibilityRule::try_new(
        vec!["v1"],
        vec!["v0"],
        Some("canonical-v1"),
        true,
        true,
        UnknownPersistedFieldHandling::RejectWithCatalogedReason,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        MigrationCompatibilityRule::try_new(
            Vec::new(),
            vec!["v0"],
            Some("canonical-v1"),
            true,
            true,
            UnknownPersistedFieldHandling::RejectWithCatalogedReason,
            true,
            true,
            true
        ),
        Err(MigrationCompatibilityRuleError::VersionSetMissing)
    );
    assert_eq!(
        MigrationCompatibilityRule::try_new(
            vec!["v1"],
            vec!["v0"],
            None,
            true,
            true,
            UnknownPersistedFieldHandling::PreserveAsDriverDetailOnly,
            true,
            true,
            true
        ),
        Err(MigrationCompatibilityRuleError::CanonicalFormatVersionMissing)
    );
    assert_eq!(
        MigrationCompatibilityRule::try_new(
            vec!["v1"],
            vec!["v0"],
            Some("canonical-v1"),
            false,
            true,
            UnknownPersistedFieldHandling::IgnoreByExplicitRule,
            true,
            true,
            true
        ),
        Err(MigrationCompatibilityRuleError::RequiredFieldsMissing)
    );
    assert_eq!(
        MigrationCompatibilityRule::try_new(
            vec!["v1"],
            vec!["v0"],
            Some("canonical-v1"),
            true,
            false,
            UnknownPersistedFieldHandling::IgnoreByExplicitRule,
            true,
            true,
            true
        ),
        Err(MigrationCompatibilityRuleError::OptionalFieldDefaultsMissing)
    );
    assert_eq!(
        MigrationCompatibilityRule::try_new(
            vec!["v1"],
            vec!["v0"],
            Some("canonical-v1"),
            true,
            true,
            UnknownPersistedFieldHandling::IgnoreByExplicitRule,
            false,
            true,
            true
        ),
        Err(MigrationCompatibilityRuleError::RollbackConditionMissing)
    );
    assert_eq!(
        MigrationCompatibilityRule::try_new(
            vec!["v1"],
            vec!["v0"],
            Some("canonical-v1"),
            true,
            true,
            UnknownPersistedFieldHandling::IgnoreByExplicitRule,
            true,
            false,
            true
        ),
        Err(MigrationCompatibilityRuleError::DataLossConditionMissing)
    );

    assert!(SchemaMigrationExecutionGuard::try_new(
        SchemaMigrationClass::NoMigrationRequired,
        MigrationModeSelectionOwner::EntrypointsTypedConfiguration,
        false,
        true
    )
    .is_ok());
    assert!(SchemaMigrationExecutionGuard::try_new(
        SchemaMigrationClass::DriverSchemaForward,
        MigrationModeSelectionOwner::EntrypointsTypedConfiguration,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        SchemaMigrationExecutionGuard::try_new(
            SchemaMigrationClass::DriverSchemaForward,
            MigrationModeSelectionOwner::DriverSelfSelection,
            true,
            true
        ),
        Err(SchemaMigrationExecutionGuardError::MigrationModeNotSelectedByEntrypoints)
    );
    assert_eq!(
        SchemaMigrationExecutionGuard::try_new(
            SchemaMigrationClass::DriverSchemaForward,
            MigrationModeSelectionOwner::EntrypointsTypedConfiguration,
            false,
            true
        ),
        Err(SchemaMigrationExecutionGuardError::DryRunOrValidationMissing)
    );
    assert_eq!(
        SchemaMigrationExecutionGuard::try_new(
            SchemaMigrationClass::NoMigrationRequired,
            MigrationModeSelectionOwner::EntrypointsTypedConfiguration,
            false,
            false
        ),
        Err(SchemaMigrationExecutionGuardError::MigrationSuccessAsRestoreSuccess)
    );
}

#[test]
fn coverage_persistence_artifact_reasons_and_prohibited_enums_are_closed() {
    assert!(ArtifactSensitiveDataGuard::try_new(true, true, true, true, true, true).is_ok());
    assert_eq!(
        ArtifactSensitiveDataGuard::try_new(true, true, false, true, true, true),
        Err(ArtifactSensitiveDataError::SensitiveMaterialRequiresRedaction)
    );
    assert!(ArtifactIntegrityGuard::try_new(
        ArtifactIntegrityClass::AuditHashChainReference,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        ArtifactIntegrityGuard::try_new(ArtifactIntegrityClass::StorageChecksum, false, true, true),
        Err(ArtifactIntegrityGuardError::IntegrityClassConflated)
    );
    assert_eq!(
        ArtifactIntegrityGuard::try_new(ArtifactIntegrityClass::Digest, true, true, false),
        Err(ArtifactIntegrityGuardError::ExportFormatAsCanonicalSerialization)
    );

    assert!(ArtifactRestoreImportGuard::try_new(
        RestoreReplayApplicability::NotIntendedForRestoreReplay,
        false,
        false
    )
    .is_ok());
    assert!(ArtifactRestoreImportGuard::try_new(
        RestoreReplayApplicability::RestoreInput,
        true,
        true
    )
    .is_ok());
    assert!(ArtifactRestoreImportGuard::try_new(
        RestoreReplayApplicability::ReplayVerificationInput,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        ArtifactRestoreImportGuard::try_new(RestoreReplayApplicability::RestoreInput, false, true),
        Err(ArtifactRestoreImportGuardError::RestoreReplayPolicyDenied)
    );
    assert_eq!(
        ArtifactRestoreImportGuard::try_new(
            RestoreReplayApplicability::ReplayVerificationInput,
            true,
            false
        ),
        Err(ArtifactRestoreImportGuardError::RestoreReplayVerificationMissing)
    );

    assert!(ExportBackupArtifactAuditShape::try_new(
        true, true, true, true, true, true, true, true
    )
    .is_ok());
    assert_eq!(
        ExportBackupArtifactAuditShape::try_new(true, true, true, true, true, false, true, true),
        Err(ExportBackupArtifactAuditShapeError::RequiredAuditFieldMissing)
    );

    for kind in [
        SchemaMigrationFailureKind::PersistenceUnavailable,
        SchemaMigrationFailureKind::RuntimeConfigMissing,
        SchemaMigrationFailureKind::RuntimeConfigInvalid,
        SchemaMigrationFailureKind::ExternalDecodeFailed,
        SchemaMigrationFailureKind::UnsupportedDriverWireVersion,
        SchemaMigrationFailureKind::MissingRequiredWireField,
        SchemaMigrationFailureKind::ExternalEnumUnmapped,
        SchemaMigrationFailureKind::PersistenceRetryBoundExceeded,
        SchemaMigrationFailureKind::PersistenceRetryDurationExceeded,
        SchemaMigrationFailureKind::DriverShutdown,
    ] {
        assert_cataloged(kind.reason_code());
        assert_debug_contains_reason(SchemaMigrationFailure::from_kind(kind), kind.reason_code());
    }
    for kind in [
        ExportBackupArtifactFailureKind::ExportSurfaceNotAllowed,
        ExportBackupArtifactFailureKind::ExportRedactionRequired,
        ExportBackupArtifactFailureKind::BackupArtifactUnavailable,
        ExportBackupArtifactFailureKind::ArtifactIntegrityMismatch,
        ExportBackupArtifactFailureKind::ArtifactRestoreNotAllowed,
    ] {
        assert_cataloged(kind.reason_code());
        assert_debug_contains_reason(
            ExportBackupArtifactFailure::from_kind(kind),
            kind.reason_code(),
        );
    }

    assert_eq!(
        [
            ProhibitedPersistenceDriverBehavior::StorageSchemaAsCoreDomainApi,
            ProhibitedPersistenceDriverBehavior::DriverOwnsStateTransitionSemantics,
            ProhibitedPersistenceDriverBehavior::UnboundedOrUnauditedRetryQueue,
            ProhibitedPersistenceDriverBehavior::StorageOwnsAuditHashChainMeaning,
            ProhibitedPersistenceDriverBehavior::FailedPersistenceOutputAsSuccess,
            ProhibitedPersistenceDriverBehavior::StorageTransactionAsAggregateCommitAuthority,
            ProhibitedPersistenceDriverBehavior::ArtifactLeavesWithoutExportBackupClassification,
            ProhibitedPersistenceDriverBehavior::BackendReplicationAsDomainOwnershipProof,
        ]
        .len(),
        8
    );
    assert_eq!(
        [
            ProhibitedSchemaMigrationBehavior::MigrationFileDefinesDomainInvariant,
            ProhibitedSchemaMigrationBehavior::SchemaExistenceAsRestoreReplaySuccess,
            ProhibitedSchemaMigrationBehavior::CoreImportsDbMigrationLibrary,
            ProhibitedSchemaMigrationBehavior::MigrationFailureSilentlySkipped,
            ProhibitedSchemaMigrationBehavior::BestEffortDecodeOfIncompatibleRepresentation,
            ProhibitedSchemaMigrationBehavior::StorageLayoutAsCanonicalSerialization,
            ProhibitedSchemaMigrationBehavior::RollbackPlanOmitted,
            ProhibitedSchemaMigrationBehavior::MigrationSnapshotAsRestoreProof,
        ]
        .len(),
        8
    );
    assert_eq!(
        [
            ProhibitedExportBackupArtifactBehavior::DatabaseDumpShapeAsPublicApi,
            ProhibitedExportBackupArtifactBehavior::BackupExistenceAsRestoreSuccess,
            ProhibitedExportBackupArtifactBehavior::StorageChecksumAsAuditHashChainProof,
            ProhibitedExportBackupArtifactBehavior::RawSensitiveMaterialInGeneralExport,
            ProhibitedExportBackupArtifactBehavior::ArtifactPathAsCoreDomainIdentity,
            ProhibitedExportBackupArtifactBehavior::ExportFormatAsCanonicalSerialization,
            ProhibitedExportBackupArtifactBehavior::ReleaseDistributionArtifactHiddenAsBackup,
        ]
        .len(),
        7
    );
}

#[test]
fn coverage_security_guards_cover_success_and_fail_closed_edges() {
    assert_eq!(
        format!("{SecurityDriverSurface:?}"),
        "SecurityDriverSurface"
    );
    assert_eq!(
        <SecurityTokenVerifierDriverPort as CorePort>::FAMILY,
        PortFamily::TokenVerifier
    );

    assert!(KeySourceConfigurationGuard::try_new(
        SecurityKeySourceType::JwksEndpoint,
        KeySourceConfigurationOrigin::EntrypointsTypedConfiguration,
        true,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        KeySourceConfigurationGuard::try_new(
            SecurityKeySourceType::StaticTypedKeyMaterial,
            KeySourceConfigurationOrigin::DriverEnvironmentRead,
            true,
            true,
            true,
            true
        ),
        Err(KeySourceConfigurationError::ConfigurationOriginInvalid)
    );
    assert_eq!(
        KeySourceConfigurationGuard::try_new(
            SecurityKeySourceType::StaticTypedKeyMaterial,
            KeySourceConfigurationOrigin::EntrypointsTypedConfiguration,
            false,
            true,
            true,
            true
        ),
        Err(KeySourceConfigurationError::TypedConfigurationMissing)
    );
    assert_eq!(
        KeySourceConfigurationGuard::try_new(
            SecurityKeySourceType::FileOrSecretManagerReference,
            KeySourceConfigurationOrigin::EntrypointsTypedConfiguration,
            true,
            false,
            true,
            true
        ),
        Err(KeySourceConfigurationError::RawKeyReferenceCrossesToCore)
    );
    assert_eq!(
        KeySourceConfigurationGuard::try_new(
            SecurityKeySourceType::FileOrSecretManagerReference,
            KeySourceConfigurationOrigin::EntrypointsTypedConfiguration,
            true,
            true,
            true,
            false
        ),
        Err(KeySourceConfigurationError::DriverDirectConfigurationRead)
    );

    assert!(KeyCacheRefreshBounds::try_new(8, 4096, 60_000, 3, 1_000).is_ok());
    assert_eq!(
        KeyCacheRefreshBounds::try_new(0, 4096, 60_000, 3, 1_000),
        Err(KeyCacheRefreshBoundsError::RequiredBoundMissing)
    );

    assert!(KeyLookupRefreshGuard::try_new(
        KeyLookupRefreshState::CacheHitFresh,
        true,
        false,
        true
    )
    .is_ok());
    assert!(KeyLookupRefreshGuard::try_new(
        KeyLookupRefreshState::RefreshFailed,
        true,
        false,
        true
    )
    .is_ok());
    assert_eq!(
        KeyLookupRefreshGuard::try_new(
            KeyLookupRefreshState::CacheMissRefreshRequired,
            false,
            false,
            true
        ),
        Err(KeyLookupRefreshError::RefreshBoundExceeded)
    );
    assert_eq!(
        KeyLookupRefreshGuard::try_new(KeyLookupRefreshState::RefreshFailed, true, true, true),
        Err(KeyLookupRefreshError::RefreshFailureWouldAcceptToken)
    );
    assert_eq!(
        KeyLookupRefreshGuard::try_new(
            KeyLookupRefreshState::KeySourceUnavailable,
            true,
            false,
            false
        ),
        Err(KeyLookupRefreshError::KeyUnavailableReasonMissing)
    );

    assert!(TokenVerifierDriverBoundaryGuard::try_new(
        true, true, true, true, true, true, true, true
    )
    .is_ok());
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(false, true, true, true, true, true, true, true),
        Err(TokenVerifierDriverBoundaryError::ConcreteImplementationEscapesDriver)
    );
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(true, true, false, true, true, true, true, true),
        Err(TokenVerifierDriverBoundaryError::CorePolicyTakenByDriver)
    );
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(true, true, true, false, true, true, true, true),
        Err(TokenVerifierDriverBoundaryError::EntrypointSpecificAuthorizationMixed)
    );
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(true, true, true, true, false, true, true, true),
        Err(TokenVerifierDriverBoundaryError::TokenIssuanceMixed)
    );
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(true, true, true, true, true, false, true, true),
        Err(TokenVerifierDriverBoundaryError::ExternalErrorNotMapped)
    );
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(true, true, true, true, true, true, false, true),
        Err(TokenVerifierDriverBoundaryError::RawSecurityMaterialExposed)
    );
    assert_eq!(
        TokenVerifierDriverBoundaryGuard::try_new(true, true, true, true, true, true, true, false),
        Err(TokenVerifierDriverBoundaryError::RawCredentialCrossesToCore)
    );
}

#[test]
fn coverage_security_rotation_reasons_and_prohibited_enums_are_closed() {
    assert!(SecretRotationPolicyGuard::try_new(
        SecretRotationClass::JwtVerificationKey,
        true,
        60_000,
        true,
        true,
        true,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::JwtVerificationKey,
            false,
            60_000,
            true,
            true,
            true,
            true,
            true
        ),
        Err(SecretRotationPolicyError::GenerationReferenceFormatMissing)
    );
    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::TurnSharedSecret,
            true,
            0,
            true,
            true,
            true,
            true,
            true
        ),
        Err(SecretRotationPolicyError::OverlapWindowMissing)
    );
    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::TransportCertificateKey,
            true,
            60_000,
            false,
            true,
            true,
            true,
            true
        ),
        Err(SecretRotationPolicyError::RevocationBehaviorMissing)
    );
    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::OpaqueSecretReference,
            true,
            60_000,
            true,
            false,
            true,
            true,
            true
        ),
        Err(SecretRotationPolicyError::ActiveCredentialRelationMissing)
    );
    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::EphemeralSessionKeyMaterial,
            true,
            60_000,
            true,
            true,
            false,
            true,
            true
        ),
        Err(SecretRotationPolicyError::FailureReasonMissing)
    );
    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::JwtVerificationKey,
            true,
            60_000,
            true,
            true,
            true,
            false,
            true
        ),
        Err(SecretRotationPolicyError::AuditRelationMissing)
    );
    assert_eq!(
        SecretRotationPolicyGuard::try_new(
            SecretRotationClass::JwtVerificationKey,
            true,
            60_000,
            true,
            true,
            true,
            true,
            false
        ),
        Err(SecretRotationPolicyError::RedactionRuleMissing)
    );

    assert!(SecretGenerationStateAdmission::try_new(
        SecretGenerationState::CurrentGeneration,
        true,
        true,
        false,
        true,
        true
    )
    .is_ok());
    assert!(SecretGenerationStateAdmission::try_new(
        SecretGenerationState::PreviousGenerationOverlap,
        true,
        false,
        true,
        true,
        true
    )
    .is_ok());
    assert!(SecretGenerationStateAdmission::try_new(
        SecretGenerationState::PendingGeneration,
        true,
        false,
        false,
        true,
        true
    )
    .is_ok());
    assert!(SecretGenerationStateAdmission::try_new(
        SecretGenerationState::RevokedGeneration,
        true,
        false,
        false,
        true,
        true
    )
    .is_ok());
    assert!(SecretGenerationStateAdmission::try_new(
        SecretGenerationState::ExpiredGeneration,
        true,
        false,
        false,
        true,
        true
    )
    .is_ok());
    assert_eq!(
        SecretGenerationStateAdmission::try_new(
            SecretGenerationState::CurrentGeneration,
            false,
            true,
            false,
            true,
            true
        ),
        Err(SecretGenerationStateAdmissionError::RotationStateUnavailable)
    );
    assert_eq!(
        SecretGenerationStateAdmission::try_new(
            SecretGenerationState::CurrentGeneration,
            true,
            false,
            false,
            true,
            true
        ),
        Err(SecretGenerationStateAdmissionError::CurrentGenerationNotAccepted)
    );
    assert_eq!(
        SecretGenerationStateAdmission::try_new(
            SecretGenerationState::PreviousGenerationOverlap,
            true,
            true,
            true,
            true,
            true
        ),
        Err(SecretGenerationStateAdmissionError::PreviousOverlapInvalid)
    );
    assert_eq!(
        SecretGenerationStateAdmission::try_new(
            SecretGenerationState::PendingGeneration,
            true,
            false,
            true,
            true,
            true
        ),
        Err(SecretGenerationStateAdmissionError::PendingGenerationAccepted)
    );
    assert_eq!(
        SecretGenerationStateAdmission::try_new(
            SecretGenerationState::RevokedGeneration,
            true,
            false,
            false,
            true,
            false
        ),
        Err(SecretGenerationStateAdmissionError::RevokedOrExpiredGenerationAccepted)
    );

    assert!(SecretRotationExecutionBoundaryGuard::try_new(
        true, true, true, true, true, true, true, true
    )
    .is_ok());
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            false, true, true, true, true, true, true, true
        ),
        Err(SecretRotationExecutionBoundaryError::RawSecretLoadEscapesDriver)
    );
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            true, false, true, true, true, true, true, true
        ),
        Err(SecretRotationExecutionBoundaryError::RotationExecutionOwnerMissing)
    );
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            true, true, false, true, true, true, true, true
        ),
        Err(SecretRotationExecutionBoundaryError::DriverStateSilentlyChangesCorePolicy)
    );
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            true, true, true, false, true, true, true, true
        ),
        Err(SecretRotationExecutionBoundaryError::InsecureFallback)
    );
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            true, true, true, true, false, true, true, true
        ),
        Err(SecretRotationExecutionBoundaryError::TokenIssuanceMixed)
    );
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            true, true, true, true, true, false, true, true
        ),
        Err(SecretRotationExecutionBoundaryError::RawSecretMaterialExposed)
    );
    assert_eq!(
        SecretRotationExecutionBoundaryGuard::try_new(
            true, true, true, true, true, true, false, true
        ),
        Err(SecretRotationExecutionBoundaryError::RawSecretMaterialCrossesCoreOrSdk)
    );

    for (backend_failure, expected_token_kind) in [
        (
            VerifierBackendFailureClass::TokenAbsent,
            Some(TokenVerificationFailureKind::TokenMissing),
        ),
        (
            VerifierBackendFailureClass::TokenDecodeFailed,
            Some(TokenVerificationFailureKind::TokenMalformed),
        ),
        (
            VerifierBackendFailureClass::SignatureInvalid,
            Some(TokenVerificationFailureKind::TokenSignatureInvalid),
        ),
        (
            VerifierBackendFailureClass::KeyUnavailable,
            Some(TokenVerificationFailureKind::TokenKeyUnavailable),
        ),
        (
            VerifierBackendFailureClass::IssuerMismatch,
            Some(TokenVerificationFailureKind::TokenIssuerMismatch),
        ),
        (
            VerifierBackendFailureClass::AudienceMismatch,
            Some(TokenVerificationFailureKind::TokenAudienceMismatch),
        ),
        (
            VerifierBackendFailureClass::TokenExpired,
            Some(TokenVerificationFailureKind::TokenExpired),
        ),
        (
            VerifierBackendFailureClass::TokenNotYetValid,
            Some(TokenVerificationFailureKind::TokenNotYetValid),
        ),
        (
            VerifierBackendFailureClass::RequiredClaimMissing,
            Some(TokenVerificationFailureKind::TokenRequiredClaimMissing),
        ),
        (
            VerifierBackendFailureClass::UnsupportedAlgorithm,
            Some(TokenVerificationFailureKind::TokenUnsupportedAlgorithm),
        ),
        (VerifierBackendFailureClass::RequiredConfigMissing, None),
        (VerifierBackendFailureClass::SecretUnavailable, None),
        (VerifierBackendFailureClass::RotationPolicyRejected, None),
    ] {
        assert_eq!(backend_failure.token_failure_kind(), expected_token_kind);
        assert_cataloged(backend_failure.reason_code());
        assert_debug_contains_reason(
            VerifierDriverFailure::from_backend_failure(backend_failure),
            backend_failure.reason_code(),
        );
    }

    for kind in [
        SecretRotationFailureKind::SecretRotationRequired,
        SecretRotationFailureKind::SecretGenerationNotAccepted,
        SecretRotationFailureKind::SecretKeyRevoked,
        SecretRotationFailureKind::SecretOverlapWindowExpired,
        SecretRotationFailureKind::SecretRotationStateUnavailable,
        SecretRotationFailureKind::SecretUnavailable,
        SecretRotationFailureKind::RuntimeConfigMissing,
        SecretRotationFailureKind::RuntimeConfigInvalid,
        SecretRotationFailureKind::TokenKeyUnavailable,
    ] {
        assert_cataloged(kind.reason_code());
        assert_debug_contains_reason(SecretRotationFailure::from_kind(kind), kind.reason_code());
    }

    assert_eq!(
        [
            ProhibitedSecurityVerifierDriverBehavior::CoreFetchesKeyMaterial,
            ProhibitedSecurityVerifierDriverBehavior::RefreshFailureAcceptsToken,
            ProhibitedSecurityVerifierDriverBehavior::UnboundedKeyCache,
            ProhibitedSecurityVerifierDriverBehavior::RawTokenOrKeyMaterialPersistedOrLogged,
            ProhibitedSecurityVerifierDriverBehavior::EntrypointSpecificRoleAuthorizationInVerifier,
            ProhibitedSecurityVerifierDriverBehavior::TokenIssuanceOwnedByArcRtc,
            ProhibitedSecurityVerifierDriverBehavior::OpenEndedDriverErrorInCoreDecision,
            ProhibitedSecurityVerifierDriverBehavior::VerificationSuccessAsAuthorizationSuccess,
        ]
        .len(),
        8
    );
    assert_eq!(
        [
            ProhibitedSecretRotationLifecycleBehavior::StaleGenerationAcceptedWithoutOverlapPolicy,
            ProhibitedSecretRotationLifecycleBehavior::RevokedGenerationAcceptedFromCache,
            ProhibitedSecretRotationLifecycleBehavior::RotationFailureInsecureFallback,
            ProhibitedSecretRotationLifecycleBehavior::RawSecretMaterialWrittenToAuditOrObservation,
            ProhibitedSecretRotationLifecycleBehavior::RawSecretMaterialCrossesCoreOrSdk,
            ProhibitedSecretRotationLifecycleBehavior::DriverStateChangesCorePolicySilently,
            ProhibitedSecretRotationLifecycleBehavior::TokenIssuanceOwnedByArcRtc,
            ProhibitedSecretRotationLifecycleBehavior::DriverSpecificGenerationSemantics,
        ]
        .len(),
        8
    );
}
