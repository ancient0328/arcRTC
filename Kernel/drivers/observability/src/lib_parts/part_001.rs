// drivers/observability は log、trace、metric、redaction/retention の driver surface です。
//
// ここは core の audit/domain meaning を新規に作らず、外部観測基盤への
// projection を後続 task で配置する場所です。

use arcrtc_core_ports::{
    CorePort, MetricsSinkFailure, MetricsSinkFailureKind, MetricsSinkInput, MetricsSinkOutput,
    MetricsSinkPort, PortFamily,
};
use arcrtc_core_reason::CatalogedReasonRef;

/// observability driver package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservabilityDriverSurface;

/// observability driver が扱う concrete sink class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySinkClass {
    /// metrics exporter.
    MetricsExporter,
    /// log sink.
    LogSink,
    /// trace sink.
    TraceSink,
}

/// observability projection の入力分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityProjectionClass {
    /// core metric export projection.
    Metric,
    /// operational log projection.
    Log,
    /// operational trace/span projection.
    Trace,
}

/// observability projection が core 意味論を上書きしないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservabilityProjectionGuard {
    projection_class: ObservabilityProjectionClass,
    audit_meaning_remains_core_owned: bool,
    quality_decision_remains_core_owned: bool,
    domain_decision_not_changed: bool,
    exporter_aggregation_not_policy: bool,
    log_text_not_authoritative_reason: bool,
    trace_name_not_audit_event_type: bool,
    not_closeout_evidence_without_report: bool,
}

/// observability projection guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityProjectionError {
    /// audit/hash-chain meaning を driver が所有しようとしています。
    AuditMeaningTakenByDriver,
    /// quality decision / domain decision を exporter が変えています。
    DecisionSemanticsChangedByProjection,
    /// log/free-text/span 名を authoritative reason/audit event として使っています。
    FreeTextAsAuthority,
    /// reproducible report なしに closeout evidence として使っています。
    ObservabilityAsCloseoutEvidence,
}

impl ObservabilityProjectionGuard {
    /// log/trace/metric projection が診断情報の範囲に留まることを確認します。
    pub const fn try_new(
        projection_class: ObservabilityProjectionClass,
        audit_meaning_remains_core_owned: bool,
        quality_decision_remains_core_owned: bool,
        domain_decision_not_changed: bool,
        exporter_aggregation_not_policy: bool,
        log_text_not_authoritative_reason: bool,
        trace_name_not_audit_event_type: bool,
        not_closeout_evidence_without_report: bool,
    ) -> Result<Self, ObservabilityProjectionError> {
        if !audit_meaning_remains_core_owned {
            return Err(ObservabilityProjectionError::AuditMeaningTakenByDriver);
        }
        if !quality_decision_remains_core_owned
            || !domain_decision_not_changed
            || !exporter_aggregation_not_policy
        {
            return Err(ObservabilityProjectionError::DecisionSemanticsChangedByProjection);
        }
        if !log_text_not_authoritative_reason || !trace_name_not_audit_event_type {
            return Err(ObservabilityProjectionError::FreeTextAsAuthority);
        }
        if !not_closeout_evidence_without_report {
            return Err(ObservabilityProjectionError::ObservabilityAsCloseoutEvidence);
        }

        Ok(Self {
            projection_class,
            audit_meaning_remains_core_owned,
            quality_decision_remains_core_owned,
            domain_decision_not_changed,
            exporter_aggregation_not_policy,
            log_text_not_authoritative_reason,
            trace_name_not_audit_event_type,
            not_closeout_evidence_without_report,
        })
    }
}

/// log / trace / metric に sensitive material を載せないための guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservabilitySensitiveDataGuard {
    raw_secret_absent: bool,
    raw_token_absent: bool,
    raw_credential_absent: bool,
    packet_payload_absent: bool,
    regulated_payload_absent: bool,
    references_are_core_opaque: bool,
}

/// sensitive data guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySensitiveDataError {
    /// raw secret/token/credential/packet/regulated payload が observability signal に残っています。
    SensitiveMaterialInSignal,
    /// reference が core-owned opaque reference ではありません。
    NonOpaqueReferenceInSignal,
}

impl ObservabilitySensitiveDataGuard {
    /// observability signal に raw sensitive data が入っていないことを確認します。
    pub const fn try_new(
        raw_secret_absent: bool,
        raw_token_absent: bool,
        raw_credential_absent: bool,
        packet_payload_absent: bool,
        regulated_payload_absent: bool,
        references_are_core_opaque: bool,
    ) -> Result<Self, ObservabilitySensitiveDataError> {
        if !raw_secret_absent
            || !raw_token_absent
            || !raw_credential_absent
            || !packet_payload_absent
            || !regulated_payload_absent
        {
            return Err(ObservabilitySensitiveDataError::SensitiveMaterialInSignal);
        }
        if !references_are_core_opaque {
            return Err(ObservabilitySensitiveDataError::NonOpaqueReferenceInSignal);
        }

        Ok(Self {
            raw_secret_absent,
            raw_token_absent,
            raw_credential_absent,
            packet_payload_absent,
            regulated_payload_absent,
            references_are_core_opaque,
        })
    }
}

/// metrics export backlog の driver-local bound です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricsExportBacklogBound {
    maximum_events: usize,
    maximum_bytes: usize,
}

/// metrics export backlog bound の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricsExportBacklogBoundError {
    /// metrics export backlog が unbounded です。
    UnboundedMetricsBacklog,
}

impl MetricsExportBacklogBound {
    /// metrics backlog bound を作ります。
    pub const fn try_new(
        maximum_events: usize,
        maximum_bytes: usize,
    ) -> Result<Self, MetricsExportBacklogBoundError> {
        if maximum_events == 0 || maximum_bytes == 0 {
            return Err(MetricsExportBacklogBoundError::UnboundedMetricsBacklog);
        }
        Ok(Self {
            maximum_events,
            maximum_bytes,
        })
    }

    /// metrics backlog bound exceeded の core-owned failure です。
    pub fn bound_exceeded_failure(&self) -> MetricsSinkFailure {
        MetricsSinkFailure::from_kind(MetricsSinkFailureKind::MetricsBacklogBoundExceeded)
    }
}

/// observability driver failure source です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityFailureSource {
    /// metrics export failed.
    MetricsExportFailed,
    /// metrics backlog bound exceeded.
    MetricsBacklogBoundExceeded,
    /// signal taxonomy invalid.
    ObservabilitySignalInvalid,
    /// metric label cardinality exceeded.
    MetricCardinalityExceeded,
    /// telemetry sampling policy missing.
    TelemetrySamplingPolicyMissing,
    /// driver shutdown.
    DriverShutdown,
}

impl ObservabilityFailureSource {
    /// driver-local failure source を core-owned MetricsSink failure へ変換します。
    pub fn to_metrics_failure(self) -> MetricsSinkFailure {
        let kind = match self {
            Self::MetricsExportFailed => MetricsSinkFailureKind::MetricsExportFailed,
            Self::MetricsBacklogBoundExceeded => {
                MetricsSinkFailureKind::MetricsBacklogBoundExceeded
            }
            Self::ObservabilitySignalInvalid => MetricsSinkFailureKind::ObservabilitySignalInvalid,
            Self::MetricCardinalityExceeded => MetricsSinkFailureKind::MetricCardinalityExceeded,
            Self::TelemetrySamplingPolicyMissing => {
                MetricsSinkFailureKind::TelemetrySamplingPolicyMissing
            }
            Self::DriverShutdown => MetricsSinkFailureKind::DriverShutdown,
        };
        MetricsSinkFailure::from_kind(kind)
    }
}

/// observability output を evidence として採用する場合の最低 shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservabilityEvidenceShape {
    correlation_id_recorded: bool,
    command_recorded: bool,
    environment_recorded: bool,
    reproducible_procedure_recorded: bool,
    time_window_recorded: bool,
    source_recorded: bool,
    bound_definition_recorded: bool,
}

/// observability evidence shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityEvidenceShapeError {
    /// reproducible report として必要な field が不足しています。
    RequiredEvidenceFieldMissing,
}

impl ObservabilityEvidenceShape {
    /// observability sample を evidence に昇格する前の shape を確認します。
    pub const fn try_new(
        correlation_id_recorded: bool,
        command_recorded: bool,
        environment_recorded: bool,
        reproducible_procedure_recorded: bool,
        time_window_recorded: bool,
        source_recorded: bool,
        bound_definition_recorded: bool,
    ) -> Result<Self, ObservabilityEvidenceShapeError> {
        if !correlation_id_recorded
            || !command_recorded
            || !environment_recorded
            || !reproducible_procedure_recorded
            || !time_window_recorded
            || !source_recorded
            || !bound_definition_recorded
        {
            return Err(ObservabilityEvidenceShapeError::RequiredEvidenceFieldMissing);
        }

        Ok(Self {
            correlation_id_recorded,
            command_recorded,
            environment_recorded,
            reproducible_procedure_recorded,
            time_window_recorded,
            source_recorded,
            bound_definition_recorded,
        })
    }
}

/// drivers/observability が core-owned MetricsSinkPort を実装する marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservabilityMetricsSinkDriverPort;

impl CorePort for ObservabilityMetricsSinkDriverPort {
    const FAMILY: PortFamily = PortFamily::MetricsSink;
    type Input = MetricsSinkInput;
    type Output = MetricsSinkOutput;
    type Error = MetricsSinkFailure;
}

impl MetricsSinkPort for ObservabilityMetricsSinkDriverPort {}

/// observability boundary で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedObservabilityBoundaryBehavior {
    /// metrics exporter owns quality decision.
    MetricsExporterOwnsQualityDecision,
    /// log text becomes closed reason vocabulary.
    LogTextAsClosedReasonVocabulary,
    /// tracing span name becomes audit event type.
    SpanNameAsAuditEventType,
    /// direct dependency on network/persistence driver alters domain flow.
    CrossDriverDependencyAltersDomainFlow,
    /// raw sensitive payload is logged.
    RawSensitivePayloadLogged,
    /// missing metrics export is ignored while claiming verification success.
    MissingMetricsExportIgnoredForVerificationClaim,
    /// alert/metric/trace signal drives domain decision directly.
    ObservabilitySignalDrivesDomainDecision,
    /// metric label cardinality is unbounded.
    UnboundedMetricLabelCardinality,
}

/// v0.2 で許可する observability signal class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySignalClass {
    /// canonical audit event or hash-chain record.
    AuditSignal,
    /// metric used by quality decision.
    QualityDecisionMetric,
    /// metric used by resource bound decision.
    ResourceBoundMetric,
    /// diagnostic/exported metric.
    OperationalMetric,
    /// execution trace diagnostic.
    TraceSpan,
    /// redacted log diagnostic.
    StructuredLog,
    /// derived operator notification.
    AlertSignal,
    /// CPU/memory/runtime diagnostic.
    ProfilingSignal,
}

/// signal owner の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySignalOwner {
    /// audit Canonical / core model.
    CoreAudit,
    /// core quality policy.
    CoreQualityPolicy,
    /// resource bound owner tuple.
    CoreResourceBoundPolicy,
    /// driver observability.
    DriverObservability,
    /// entrypoints / operations policy.
    EntrypointsOperationsPolicy,
}

/// signal に許可される reference type class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalReferenceClass {
    /// core-owned opaque reference.
    CoreOpaqueReference,
    /// audit event / hash-chain reference.
    AuditReference,
    /// resource owner / bound reference.
    ResourceOwnerReference,
    /// reference を持たない signal.
    NoReference,
}

/// signal sensitive data classification です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalSensitiveDataClass {
    /// no sensitive material.
    NoSensitiveMaterial,
    /// redacted material only.
    RedactedOnly,
    /// sensitive material would require rejection.
    SensitiveMaterialRejected,
}

/// signal cardinality class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalCardinalityClass {
    /// small fixed set.
    FixedLow,
    /// bounded by explicit max.
    Bounded { maximum_distinct_values: usize },
    /// unbounded cardinality; prohibited.
    Unbounded,
}

/// signal sampling rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalSamplingRule {
    /// sampling does not apply.
    NotSampled,
    /// explicit sampling policy is present.
    ExplicitPolicy,
    /// sampling is forbidden for this signal.
    SamplingForbidden,
    /// sampling applies but policy is missing.
    MissingPolicy,
}

/// signal evidence adoption rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalEvidenceAdoptionRule {
    /// audit Canonical owns evidence meaning.
    AuditCanonical,
    /// quality Canonical owns decision evidence.
    QualityCanonical,
    /// resource bound Canonical owns decision evidence.
    ResourceBoundCanonical,
    /// report with rerunnable procedure is required.
    ReportRequiredForEvidence,
    /// diagnostic only.
    DiagnosticOnly,
}

/// observability signal taxonomy descriptor です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservabilitySignalDescriptor {
    signal_class: ObservabilitySignalClass,
    owner: ObservabilitySignalOwner,
    allowed_references: Vec<SignalReferenceClass>,
    sensitive_data_class: SignalSensitiveDataClass,
    cardinality_class: SignalCardinalityClass,
    sampling_rule: SignalSamplingRule,
    retention_redaction_rule_declared: bool,
    evidence_adoption_rule: SignalEvidenceAdoptionRule,
}

/// signal taxonomy descriptor の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySignalDescriptorError {
    /// signal owner が class と一致していません。
    SignalOwnerMismatch,
    /// allowed reference type が未宣言です。
    ReferenceClassMissing,
    /// sensitive data classification が不正です。
    SensitiveDataClassInvalid,
    /// cardinality が unbounded です。
    UnboundedCardinality,
    /// sampling policy が必要なのにありません。
    SamplingPolicyMissing,
    /// retention/redaction rule がありません。
    RetentionRedactionRuleMissing,
    /// evidence adoption rule が class と一致していません。
    EvidenceAdoptionRuleMismatch,
}

impl ObservabilitySignalDescriptor {
    /// signal class と採用条件を一体で検査します。
    pub fn try_new(
        signal_class: ObservabilitySignalClass,
        owner: ObservabilitySignalOwner,
        allowed_references: Vec<SignalReferenceClass>,
        sensitive_data_class: SignalSensitiveDataClass,
        cardinality_class: SignalCardinalityClass,
        sampling_rule: SignalSamplingRule,
        retention_redaction_rule_declared: bool,
        evidence_adoption_rule: SignalEvidenceAdoptionRule,
    ) -> Result<Self, ObservabilitySignalDescriptorError> {
        if owner != signal_class.required_owner() {
            return Err(ObservabilitySignalDescriptorError::SignalOwnerMismatch);
        }
        if allowed_references.is_empty() {
            return Err(ObservabilitySignalDescriptorError::ReferenceClassMissing);
        }
        if matches!(
            sensitive_data_class,
            SignalSensitiveDataClass::SensitiveMaterialRejected
        ) {
            return Err(ObservabilitySignalDescriptorError::SensitiveDataClassInvalid);
        }
        if matches!(cardinality_class, SignalCardinalityClass::Unbounded)
            || matches!(
                cardinality_class,
                SignalCardinalityClass::Bounded {
                    maximum_distinct_values: 0
                }
            )
        {
            return Err(ObservabilitySignalDescriptorError::UnboundedCardinality);
        }
        if matches!(sampling_rule, SignalSamplingRule::MissingPolicy) {
            return Err(ObservabilitySignalDescriptorError::SamplingPolicyMissing);
        }
        if !retention_redaction_rule_declared {
            return Err(ObservabilitySignalDescriptorError::RetentionRedactionRuleMissing);
        }
        if evidence_adoption_rule != signal_class.required_evidence_rule() {
            return Err(ObservabilitySignalDescriptorError::EvidenceAdoptionRuleMismatch);
        }

        Ok(Self {
            signal_class,
            owner,
            allowed_references,
            sensitive_data_class,
            cardinality_class,
            sampling_rule,
            retention_redaction_rule_declared,
            evidence_adoption_rule,
        })
    }
}

