impl ObservabilitySignalClass {
    /// signal class ごとの required owner です。
    pub const fn required_owner(self) -> ObservabilitySignalOwner {
        match self {
            Self::AuditSignal => ObservabilitySignalOwner::CoreAudit,
            Self::QualityDecisionMetric => ObservabilitySignalOwner::CoreQualityPolicy,
            Self::ResourceBoundMetric => ObservabilitySignalOwner::CoreResourceBoundPolicy,
            Self::OperationalMetric
            | Self::TraceSpan
            | Self::StructuredLog
            | Self::ProfilingSignal => ObservabilitySignalOwner::DriverObservability,
            Self::AlertSignal => ObservabilitySignalOwner::EntrypointsOperationsPolicy,
        }
    }

    /// signal class から source-owned runtime use を導出します。
    pub const fn use_class(self) -> SignalUseClass {
        match self {
            Self::AuditSignal => SignalUseClass::AuditRecord,
            Self::QualityDecisionMetric => SignalUseClass::QualityDecisionInput,
            Self::ResourceBoundMetric => SignalUseClass::ResourceBoundDecisionInput,
            Self::OperationalMetric | Self::StructuredLog => SignalUseClass::OperationalObservation,
            Self::TraceSpan | Self::AlertSignal | Self::ProfilingSignal => {
                SignalUseClass::DiagnosticOnly
            }
        }
    }
}

/// sampling が source-owned audit / decision input を欠落させないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalSamplingGuard {
    signal_class: ObservabilitySignalClass,
    sampling_rule: SignalSamplingRule,
    audit_completeness_preserved: bool,
    resource_bound_signal_preserved: bool,
    quality_input_sampling_allowed: bool,
}

/// sampling guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalSamplingGuardError {
    /// sampling policy が必要なのにありません。
    SamplingPolicyMissing,
    /// sampling が required audit / decision signal を隠しています。
    SamplingHidesRequiredSignal,
}

impl SignalSamplingGuard {
    /// sampling が audit / quality / resource-bound input を壊さないことを確認します。
    pub const fn try_new(
        signal_class: ObservabilitySignalClass,
        sampling_rule: SignalSamplingRule,
        audit_completeness_preserved: bool,
        resource_bound_signal_preserved: bool,
        quality_input_sampling_allowed: bool,
    ) -> Result<Self, SignalSamplingGuardError> {
        if matches!(sampling_rule, SignalSamplingRule::MissingPolicy) {
            return Err(SignalSamplingGuardError::SamplingPolicyMissing);
        }
        if !audit_completeness_preserved
            || !resource_bound_signal_preserved
            || !quality_input_sampling_allowed
        {
            return Err(SignalSamplingGuardError::SamplingHidesRequiredSignal);
        }

        Ok(Self {
            signal_class,
            sampling_rule,
            audit_completeness_preserved,
            resource_bound_signal_preserved,
            quality_input_sampling_allowed,
        })
    }
}

/// observability signal failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySignalFailureKind {
    /// signal does not match allowed taxonomy.
    ObservabilitySignalInvalid,
    /// metric label cardinality bound exceeded.
    MetricCardinalityExceeded,
    /// required sampling policy absent.
    TelemetrySamplingPolicyMissing,
    /// alert signal attempts to drive domain decision.
    AlertSignalNotAllowed,
    /// export is not allowed by privacy/retention policy.
    ObservabilityExportNotAllowed,
    /// metrics export failed.
    MetricsExportFailed,
    /// metrics backlog bound exceeded.
    MetricsBacklogBoundExceeded,
}

impl ObservabilitySignalFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ObservabilitySignalInvalid => "observability_signal_invalid",
            Self::MetricCardinalityExceeded => "metric_cardinality_exceeded",
            Self::TelemetrySamplingPolicyMissing => "telemetry_sampling_policy_missing",
            Self::AlertSignalNotAllowed => "alert_signal_not_allowed",
            Self::ObservabilityExportNotAllowed => "observability_export_not_allowed",
            Self::MetricsExportFailed => "metrics_export_failed",
            Self::MetricsBacklogBoundExceeded => "metrics_backlog_bound_exceeded",
        }
    }
}

/// observability signal taxonomy failure です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObservabilitySignalFailure {
    kind: ObservabilitySignalFailureKind,
    reason: CatalogedReasonRef,
    metrics_failure: Option<MetricsSinkFailure>,
}

impl ObservabilitySignalFailure {
    /// cataloged reason と必要な metrics failure mapping に接続します。
    pub fn from_kind(kind: ObservabilitySignalFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("observability signal failure reason code must be registered");
        let metrics_failure = match kind {
            ObservabilitySignalFailureKind::ObservabilitySignalInvalid => Some(
                MetricsSinkFailure::from_kind(MetricsSinkFailureKind::ObservabilitySignalInvalid),
            ),
            ObservabilitySignalFailureKind::MetricCardinalityExceeded => Some(
                MetricsSinkFailure::from_kind(MetricsSinkFailureKind::MetricCardinalityExceeded),
            ),
            ObservabilitySignalFailureKind::TelemetrySamplingPolicyMissing => {
                Some(MetricsSinkFailure::from_kind(
                    MetricsSinkFailureKind::TelemetrySamplingPolicyMissing,
                ))
            }
            ObservabilitySignalFailureKind::MetricsExportFailed => Some(
                MetricsSinkFailure::from_kind(MetricsSinkFailureKind::MetricsExportFailed),
            ),
            ObservabilitySignalFailureKind::MetricsBacklogBoundExceeded => Some(
                MetricsSinkFailure::from_kind(MetricsSinkFailureKind::MetricsBacklogBoundExceeded),
            ),
            ObservabilitySignalFailureKind::AlertSignalNotAllowed
            | ObservabilitySignalFailureKind::ObservabilityExportNotAllowed => None,
        };

        Self {
            kind,
            reason,
            metrics_failure,
        }
    }
}

/// `observability_signal_decision` audit event に必要な shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservabilitySignalAuditShape {
    event_type_recorded: bool,
    signal_class_recorded: bool,
    signal_reference_recorded: bool,
    resource_owner_recorded_when_bound_failure: bool,
    cataloged_reason_recorded_for_non_success: bool,
}

/// signal audit shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySignalAuditShapeError {
    /// required audit field が不足しています。
    RequiredSignalAuditFieldMissing,
}

impl ObservabilitySignalAuditShape {
    /// observability signal decision の audit shape を確認します。
    pub const fn try_new(
        event_type_recorded: bool,
        signal_class_recorded: bool,
        signal_reference_recorded: bool,
        resource_owner_recorded_when_bound_failure: bool,
        cataloged_reason_recorded_for_non_success: bool,
    ) -> Result<Self, ObservabilitySignalAuditShapeError> {
        if !event_type_recorded
            || !signal_class_recorded
            || !signal_reference_recorded
            || !resource_owner_recorded_when_bound_failure
            || !cataloged_reason_recorded_for_non_success
        {
            return Err(ObservabilitySignalAuditShapeError::RequiredSignalAuditFieldMissing);
        }

        Ok(Self {
            event_type_recorded,
            signal_class_recorded,
            signal_reference_recorded,
            resource_owner_recorded_when_bound_failure,
            cataloged_reason_recorded_for_non_success,
        })
    }
}

/// observability signal taxonomy 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedObservabilitySignalTaxonomyBehavior {
    /// trace span name becomes audit event type.
    TraceSpanNameAsAuditEventType,
    /// alert status becomes domain decision.
    AlertStatusAsDomainDecision,
    /// sampled operational metric is used as a complete audit record.
    SampledMetricAsCompleteAuditRecord,
    /// metric label contains raw identity/token/packet/regulated payload.
    RawSensitiveLabelValue,
    /// dashboard state is treated as runtime correctness proof.
    DashboardStateAsRuntimeCorrectnessProof,
    /// exporter aggregation changes quality decision semantics.
    ExporterAggregationChangesQualityDecision,
    /// taxonomy differs by driver without a source contract change.
    DriverSpecificTaxonomyWithoutSourceContract,
}

/// privacy/redaction/retention 境界で扱う data class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyDataClass {
    /// core-owned opaque reference.
    CoreReference,
    /// closed reason category/code.
    CatalogReason,
    /// source allowlist 済みの non-sensitive tag.
    NonSensitiveTag,
    /// raw secret / credential material.
    RawSecret,
    /// raw bearer/session/JWT token material.
    RawToken,
    /// raw key material.
    RawKeyMaterial,
    /// RTP/RTCP/media payload bytes.
    RawPacketPayload,
    /// regulated domain payload.
    RegulatedPayload,
    /// non-authoritative diagnostic detail.
    DiagnosticDetail,
    /// edge/proxy metadata.
    EdgeProxyMetadata,
}

impl PrivacyDataClass {
    /// raw sensitive material として扱う data class です。
    pub const fn is_raw_sensitive(self) -> bool {
        matches!(
            self,
            Self::RawSecret
                | Self::RawToken
                | Self::RawKeyMaterial
                | Self::RawPacketPayload
                | Self::RegulatedPayload
        )
    }

    /// redacted excerpt ではなく opaque reference/drop/reject に閉じるべき data class です。
    pub const fn requires_opaque_drop_or_reject(self) -> bool {
        matches!(
            self,
            Self::RawSecret | Self::RawToken | Self::RawKeyMaterial | Self::RawPacketPayload
        )
    }
}

/// privacy boundary の target surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyTargetSurface {
    /// core state.
    CoreState,
    /// audit event / hash-chain.
    Audit,
    /// log / trace / metric sink.
    ObservabilitySink,
    /// SDK public error / client-visible surface.
    SdkPublicSurface,
    /// driver-local packet lifecycle.
    DriverPacketLifecycle,
    /// driver-local key cache.
    DriverKeyCache,
    /// regulated-only surface.
    RegulatedOnly,
}

/// redaction 後に境界へ渡す出力分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RedactedOutputClass {
    /// source allowlist 済み field.
    AllowlistedField,
    /// core-owned opaque reference.
    OpaqueReference,
    /// raw material を含まない redacted excerpt.
    RedactedExcerpt,
    /// sink/export せず drop する。
    Dropped,
    /// boundary 通過を拒否する。
    Rejected,
}

/// privacy/redaction 境界通過前の admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrivacyRedactionAdmission {
    data_class: PrivacyDataClass,
    target_surface: PrivacyTargetSurface,
    output_class: RedactedOutputClass,
    raw_sensitive_material_absent: bool,
    allowlisted_field: bool,
    redacted_reference_defined_when_needed: bool,
    external_sink_default_not_relied_on: bool,
    diagnostic_detail_non_authoritative: bool,
    regulated_payload_not_in_generic_path: bool,
    edge_proxy_trust_class_declared_when_used: bool,
}

/// privacy/redaction admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyRedactionAdmissionError {
    /// raw sensitive material が境界に残っています。
    RawSensitiveMaterialPresent,
    /// source allowlist 外の field です。
    FieldNotAllowlisted,
    /// redacted reference が未定義です。
    RedactedReferenceMissing,
    /// external sink の default redaction に依存しています。
    ExternalSinkDefaultReliedOn,
    /// diagnostic detail を authoritative reason として扱っています。
    DiagnosticDetailAuthoritative,
    /// regulated payload が generic path に混入しています。
    RegulatedPayloadInGenericPath,
    /// edge/proxy metadata の trust/redaction class が未宣言です。
    EdgeProxyTrustClassMissing,
}

impl PrivacyRedactionAdmission {
    /// 境界通過前に raw material と allowlist を検査します。
    pub const fn try_new(
        data_class: PrivacyDataClass,
        target_surface: PrivacyTargetSurface,
        output_class: RedactedOutputClass,
        raw_sensitive_material_absent: bool,
        allowlisted_field: bool,
        redacted_reference_defined_when_needed: bool,
        external_sink_default_not_relied_on: bool,
        diagnostic_detail_non_authoritative: bool,
        regulated_payload_not_in_generic_path: bool,
        edge_proxy_trust_class_declared_when_used: bool,
    ) -> Result<Self, PrivacyRedactionAdmissionError> {
        if !external_sink_default_not_relied_on {
            return Err(PrivacyRedactionAdmissionError::ExternalSinkDefaultReliedOn);
        }
        if data_class.is_raw_sensitive() && !raw_sensitive_material_absent {
            return Err(PrivacyRedactionAdmissionError::RawSensitiveMaterialPresent);
        }
        if matches!(output_class, RedactedOutputClass::AllowlistedField)
            && (!allowlisted_field || data_class.is_raw_sensitive())
        {
            return Err(PrivacyRedactionAdmissionError::FieldNotAllowlisted);
        }
        if data_class.requires_opaque_drop_or_reject()
            && !matches!(
                output_class,
                RedactedOutputClass::OpaqueReference
                    | RedactedOutputClass::Dropped
                    | RedactedOutputClass::Rejected
            )
        {
            return Err(PrivacyRedactionAdmissionError::RedactedReferenceMissing);
        }
        if matches!(output_class, RedactedOutputClass::OpaqueReference)
            && !redacted_reference_defined_when_needed
        {
            return Err(PrivacyRedactionAdmissionError::RedactedReferenceMissing);
        }
        if matches!(data_class, PrivacyDataClass::DiagnosticDetail)
            && !diagnostic_detail_non_authoritative
        {
            return Err(PrivacyRedactionAdmissionError::DiagnosticDetailAuthoritative);
        }
        if matches!(data_class, PrivacyDataClass::RegulatedPayload)
            && (!regulated_payload_not_in_generic_path
                || !matches!(target_surface, PrivacyTargetSurface::RegulatedOnly))
        {
            return Err(PrivacyRedactionAdmissionError::RegulatedPayloadInGenericPath);
        }
        if matches!(data_class, PrivacyDataClass::EdgeProxyMetadata)
            && !edge_proxy_trust_class_declared_when_used
        {
            return Err(PrivacyRedactionAdmissionError::EdgeProxyTrustClassMissing);
        }

        Ok(Self {
            data_class,
            target_surface,
            output_class,
            raw_sensitive_material_absent,
            allowlisted_field,
            redacted_reference_defined_when_needed,
            external_sink_default_not_relied_on,
            diagnostic_detail_non_authoritative,
            regulated_payload_not_in_generic_path,
            edge_proxy_trust_class_declared_when_used,
        })
    }
}

/// retention target の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyRetentionTarget {
    /// audit event fields.
    AuditEvent,
    /// audit hash-chain material.
    AuditHashChain,
    /// operational logs/traces.
    LogTrace,
    /// metrics and labels.
    Metrics,
    /// driver-local packet cache.
    PacketCache,
    /// driver-local key cache.
    KeyCache,
    /// SDK client local data.
    SdkClientLocalData,
    /// regulated-only payload store.
    RegulatedOnly,
}

/// retention owner の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyRetentionOwner {
    /// core owns closed audit/reference retention.
    Core,
    /// driver owns operational/cache retention.
    Driver,
    /// SDK client local owner.
    Sdk,
    /// regulated-only owner.
    Regulated,
}
