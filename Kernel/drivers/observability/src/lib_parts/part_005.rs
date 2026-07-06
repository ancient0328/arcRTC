use arcrtc_core_quality::{
    classify_observability_signal, ObservabilitySignalDecision, ObservabilitySignalInput,
    TelemetrySignalClass,
};

/// driver が扱う redacted telemetry signal ref です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TelemetryDriverSignal {
    signal_ref: &'static str,
    signal_class: TelemetrySignalClass,
    redacted: bool,
}

impl TelemetryDriverSignal {
    /// raw payload を持たない signal ref を作ります。
    pub const fn new(
        signal_ref: &'static str,
        signal_class: TelemetrySignalClass,
        redacted: bool,
    ) -> Self {
        Self {
            signal_ref,
            signal_class,
            redacted,
        }
    }

    /// signal ref です。raw payload ではありません。
    pub const fn signal_ref(&self) -> &'static str {
        self.signal_ref
    }
}

/// metric bound adapter が受ける core-quality policy input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TelemetryMetricSignal {
    signal: TelemetryDriverSignal,
    quality_input: ObservabilitySignalInput,
}

impl TelemetryMetricSignal {
    /// redacted signal ref と core-quality classification input を束ねます。
    pub const fn new(
        signal: TelemetryDriverSignal,
        quality_input: ObservabilitySignalInput,
    ) -> Self {
        Self {
            signal,
            quality_input,
        }
    }
}

/// telemetry adapter failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TelemetryAdapterFailureKind {
    /// sink write が失敗しました。
    WriteFailed,
    /// redaction boundary が満たされません。
    RedactionFailed,
    /// cardinality policy により拒否されました。
    CardinalityRejected,
}

/// file telemetry sink です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileTelemetrySink {
    sink_path_ref: &'static str,
}

impl FileTelemetrySink {
    /// sink path ref を保持します。path format semantics は driver-local に閉じます。
    pub const fn new(sink_path_ref: &'static str) -> Self {
        Self { sink_path_ref }
    }

    /// redacted signal ref を sink への write observation に写像します。
    pub const fn write(
        &self,
        signal: TelemetryDriverSignal,
    ) -> Result<TelemetrySinkWriteObservation, TelemetryAdapterFailureKind> {
        if self.sink_path_ref.is_empty() || signal.signal_ref.is_empty() {
            return Err(TelemetryAdapterFailureKind::WriteFailed);
        }
        if !signal.redacted {
            return Err(TelemetryAdapterFailureKind::RedactionFailed);
        }
        Ok(TelemetrySinkWriteObservation::new(
            self.sink_path_ref,
            signal.signal_ref,
            signal.signal_class,
        ))
    }
}

/// telemetry sink write observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TelemetrySinkWriteObservation {
    sink_path_ref: &'static str,
    signal_ref: &'static str,
    signal_class: TelemetrySignalClass,
}

impl TelemetrySinkWriteObservation {
    /// sink write observation を作ります。signal payload は保持しません。
    pub const fn new(
        sink_path_ref: &'static str,
        signal_ref: &'static str,
        signal_class: TelemetrySignalClass,
    ) -> Self {
        Self {
            sink_path_ref,
            signal_ref,
            signal_class,
        }
    }
}

/// telemetry redaction adapter です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TelemetryRedactionAdapter {
    boundary_ref: &'static str,
}

impl TelemetryRedactionAdapter {
    /// redaction boundary ref を保持します。redaction policy semantics は所有しません。
    pub const fn new(boundary_ref: &'static str) -> Self {
        Self { boundary_ref }
    }

    /// boundary ref がある signal だけを redacted として返します。
    pub const fn apply(
        &self,
        signal: TelemetryDriverSignal,
    ) -> Result<TelemetryDriverSignal, TelemetryAdapterFailureKind> {
        if self.boundary_ref.is_empty() || signal.signal_ref.is_empty() {
            return Err(TelemetryAdapterFailureKind::RedactionFailed);
        }
        Ok(TelemetryDriverSignal::new(
            signal.signal_ref,
            signal.signal_class,
            true,
        ))
    }
}

/// cardinality bound adapter です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CardinalityBoundAdapter {
    policy_ref: &'static str,
}

impl CardinalityBoundAdapter {
    /// cardinality policy ref を保持します。policy semantics は core-quality input に閉じます。
    pub const fn new(policy_ref: &'static str) -> Self {
        Self { policy_ref }
    }

    /// core-quality の分類結果を driver failure へ写像します。
    pub const fn bound(
        &self,
        metric: TelemetryMetricSignal,
    ) -> Result<TelemetryMetricSignal, TelemetryAdapterFailureKind> {
        if self.policy_ref.is_empty() {
            return Err(TelemetryAdapterFailureKind::CardinalityRejected);
        }

        match classify_observability_signal(metric.quality_input) {
            ObservabilitySignalDecision::Accepted(_) => Ok(metric),
            ObservabilitySignalDecision::Rejected(_) => {
                Err(TelemetryAdapterFailureKind::CardinalityRejected)
            }
        }
    }
}
