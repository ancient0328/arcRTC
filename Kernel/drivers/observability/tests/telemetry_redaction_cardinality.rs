use arcrtc_core_ports::{MetricsExportAcknowledgement, MetricsSinkInput, MetricsSinkOutput};
use arcrtc_core_quality::{
    MetricCardinalityPolicy, ObservabilitySignalInput, QualityMetric, QualityMetricKind,
    TelemetrySamplingPolicy, TelemetrySignalClass, TraceCorrelationPolicy,
};
use arcrtc_driver_observability::{
    CardinalityBoundAdapter, FileMetricsSink, FileTelemetrySink, ObservabilityProjectionClass,
    ObservabilityProjectionGuard, ObservabilitySignalClass, SignalSamplingGuard,
    SignalSamplingGuardError, SignalSamplingRule, TelemetryAdapterFailureKind,
    TelemetryDriverSignal, TelemetryMetricSignal, TelemetryRedactionAdapter,
};

struct DirGuard(std::path::PathBuf);

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_dir() -> DirGuard {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time must be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "arcrtc_telemetry_redaction_cardinality_{}_{}",
        std::process::id(),
        nonce
    ));
    std::fs::create_dir_all(&path).expect("temp dir must be created");
    DirGuard(path)
}

fn accepted_observability_input(signal_class: TelemetrySignalClass) -> ObservabilitySignalInput {
    ObservabilitySignalInput::new(
        signal_class,
        2,
        MetricCardinalityPolicy::new(4, "metric-label-allowlist:v0"),
        TelemetrySamplingPolicy::new(100, true),
        TraceCorrelationPolicy::new(true, "traceparent:test"),
        true,
    )
}

fn redacted_signal(
    signal_ref: &'static str,
    signal_class: TelemetrySignalClass,
) -> TelemetryDriverSignal {
    TelemetryDriverSignal::new(signal_ref, signal_class, true)
}

#[test]
fn assert_metric_shape() {
    let dir = temp_dir();
    let path = dir.0.join("metrics.log");
    let sink = FileMetricsSink::new(path.clone());
    let input = MetricsSinkInput::SubmitQualityMetric(QualityMetric::new(
        QualityMetricKind::Rtt,
        42,
        "ms",
        "60s",
    ));

    assert_eq!(
        sink.export(&input),
        Ok(MetricsSinkOutput::Acknowledgement(
            MetricsExportAcknowledgement::new(true, true)
        ))
    );
    let text = std::fs::read_to_string(path).expect("metrics file must be readable");
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        vec!["kind=Rtt normalized_value=42 unit=ms window=60s"]
    );
    assert!(ObservabilityProjectionGuard::try_new(
        ObservabilityProjectionClass::Metric,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
}

#[test]
fn assert_log_shape() {
    let sink = FileTelemetrySink::new("sink:structured-log");
    let signal = redacted_signal("signal:structured-log", TelemetrySignalClass::Log);

    assert!(ObservabilityProjectionGuard::try_new(
        ObservabilityProjectionClass::Log,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        sink.write(signal),
        Ok(
            arcrtc_driver_observability::TelemetrySinkWriteObservation::new(
                "sink:structured-log",
                "signal:structured-log",
                TelemetrySignalClass::Log,
            )
        )
    );
}

#[test]
fn assert_trace_shape() {
    let sink = FileTelemetrySink::new("sink:trace");
    let signal = redacted_signal("signal:trace", TelemetrySignalClass::Trace);

    assert!(ObservabilityProjectionGuard::try_new(
        ObservabilityProjectionClass::Trace,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        sink.write(signal),
        Ok(
            arcrtc_driver_observability::TelemetrySinkWriteObservation::new(
                "sink:trace",
                "signal:trace",
                TelemetrySignalClass::Trace,
            )
        )
    );
}

#[test]
fn assert_redaction() {
    let sink = FileTelemetrySink::new("sink:redaction");
    let raw_signal =
        TelemetryDriverSignal::new("signal:redaction", TelemetrySignalClass::Log, false);
    let redaction = TelemetryRedactionAdapter::new("redaction-boundary:v0");

    assert_eq!(
        sink.write(raw_signal),
        Err(TelemetryAdapterFailureKind::RedactionFailed)
    );
    let redacted = redaction.apply(raw_signal).expect("redaction must apply");
    assert!(sink.write(redacted).is_ok());
}

#[test]
fn assert_cardinality() {
    let adapter = CardinalityBoundAdapter::new("cardinality-policy:v0");
    let accepted = TelemetryMetricSignal::new(
        redacted_signal("signal:metric:accepted", TelemetrySignalClass::Metric),
        accepted_observability_input(TelemetrySignalClass::Metric),
    );
    let rejected = TelemetryMetricSignal::new(
        redacted_signal("signal:metric:rejected", TelemetrySignalClass::Metric),
        ObservabilitySignalInput::new(
            TelemetrySignalClass::Metric,
            5,
            MetricCardinalityPolicy::new(4, "metric-label-allowlist:v0"),
            TelemetrySamplingPolicy::new(100, true),
            TraceCorrelationPolicy::new(true, "traceparent:test"),
            true,
        ),
    );

    assert!(adapter.bound(accepted).is_ok());
    assert_eq!(
        adapter.bound(rejected),
        Err(TelemetryAdapterFailureKind::CardinalityRejected)
    );
}

#[test]
fn assert_sampling() {
    assert!(SignalSamplingGuard::try_new(
        ObservabilitySignalClass::OperationalMetric,
        SignalSamplingRule::ExplicitPolicy,
        true,
        true,
        true,
    )
    .is_ok());
    assert_eq!(
        SignalSamplingGuard::try_new(
            ObservabilitySignalClass::OperationalMetric,
            SignalSamplingRule::MissingPolicy,
            true,
            true,
            true,
        ),
        Err(SignalSamplingGuardError::SamplingPolicyMissing)
    );
}
