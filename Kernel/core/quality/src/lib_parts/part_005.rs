/// telemetry signal class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TelemetrySignalClass {
    /// metric signal.
    Metric,
    /// log signal.
    Log,
    /// trace signal.
    Trace,
}

/// metric cardinality policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricCardinalityPolicy {
    max_series_per_metric: u32,
    label_allowlist_ref: &'static str,
}

impl MetricCardinalityPolicy {
    /// metric cardinality bound と label allowlist ref を束ねます。
    pub const fn new(max_series_per_metric: u32, label_allowlist_ref: &'static str) -> Self {
        Self {
            max_series_per_metric,
            label_allowlist_ref,
        }
    }
}

/// telemetry sampling policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TelemetrySamplingPolicy {
    sampling_rate_basis_points: u16,
    always_sample_error: bool,
}

impl TelemetrySamplingPolicy {
    /// sampling rate と error sampling policy を束ねます。
    pub const fn new(sampling_rate_basis_points: u16, always_sample_error: bool) -> Self {
        Self {
            sampling_rate_basis_points,
            always_sample_error,
        }
    }
}

/// trace correlation policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceCorrelationPolicy {
    correlation_id_required: bool,
    trace_parent_ref: &'static str,
}

impl TraceCorrelationPolicy {
    /// correlation requirement と trace parent ref を束ねます。
    pub const fn new(correlation_id_required: bool, trace_parent_ref: &'static str) -> Self {
        Self {
            correlation_id_required,
            trace_parent_ref,
        }
    }
}

/// observability signal rejection の閉じた理由です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySignalFailureKind {
    /// metric series 数が policy bound を超えました。
    MetricCardinalityExceeded,
    /// metric label allowlist ref がありません。
    LabelAllowlistMissing,
    /// sampling rate が 0..=10000 basis points を超えました。
    SamplingRateOutOfRange,
    /// required correlation id がありません。
    CorrelationIdMissing,
    /// trace parent ref がありません。
    TraceParentMissing,
}

/// observability signal decision です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilitySignalDecision {
    /// signal は受理されました。
    Accepted(TelemetrySignalClass),
    /// signal は閉じた理由で拒否されました。
    Rejected(ObservabilitySignalFailureKind),
}

/// observability signal classification input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservabilitySignalInput {
    signal_class: TelemetrySignalClass,
    metric_series_count: u32,
    metric_cardinality_policy: MetricCardinalityPolicy,
    sampling_policy: TelemetrySamplingPolicy,
    trace_correlation_policy: TraceCorrelationPolicy,
    correlation_id_present: bool,
}

impl ObservabilitySignalInput {
    /// telemetry signal の分類材料を束ねます。
    pub const fn new(
        signal_class: TelemetrySignalClass,
        metric_series_count: u32,
        metric_cardinality_policy: MetricCardinalityPolicy,
        sampling_policy: TelemetrySamplingPolicy,
        trace_correlation_policy: TraceCorrelationPolicy,
        correlation_id_present: bool,
    ) -> Self {
        Self {
            signal_class,
            metric_series_count,
            metric_cardinality_policy,
            sampling_policy,
            trace_correlation_policy,
            correlation_id_present,
        }
    }
}

/// metrics/logs/traces を quality policy として分類します。
///
/// この関数は audit ledger writer ではなく、telemetry sink 実行も行いません。
pub const fn classify_observability_signal(
    input: ObservabilitySignalInput,
) -> ObservabilitySignalDecision {
    if input.sampling_policy.sampling_rate_basis_points > 10_000 {
        return ObservabilitySignalDecision::Rejected(
            ObservabilitySignalFailureKind::SamplingRateOutOfRange,
        );
    }

    match input.signal_class {
        TelemetrySignalClass::Metric => {
            if input.metric_cardinality_policy.label_allowlist_ref.is_empty() {
                return ObservabilitySignalDecision::Rejected(
                    ObservabilitySignalFailureKind::LabelAllowlistMissing,
                );
            }
            if input.metric_series_count > input.metric_cardinality_policy.max_series_per_metric {
                return ObservabilitySignalDecision::Rejected(
                    ObservabilitySignalFailureKind::MetricCardinalityExceeded,
                );
            }
        }
        TelemetrySignalClass::Log => {}
        TelemetrySignalClass::Trace => {
            if input.trace_correlation_policy.trace_parent_ref.is_empty() {
                return ObservabilitySignalDecision::Rejected(
                    ObservabilitySignalFailureKind::TraceParentMissing,
                );
            }
        }
    }

    if input.trace_correlation_policy.correlation_id_required && !input.correlation_id_present {
        return ObservabilitySignalDecision::Rejected(
            ObservabilitySignalFailureKind::CorrelationIdMissing,
        );
    }

    ObservabilitySignalDecision::Accepted(input.signal_class)
}
