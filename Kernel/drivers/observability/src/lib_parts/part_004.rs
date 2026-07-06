use std::io::Write;

/// metrics を driver-owned file sink に 1 行ずつ export します。
pub struct FileMetricsSink {
    path: std::path::PathBuf,
}

impl FileMetricsSink {
    /// 出力先 path を保持します。この時点では I/O を行いません。
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path }
    }

    /// core-owned metric input を driver-owned file line へ投影します。
    pub fn export(&self, input: &MetricsSinkInput) -> Result<MetricsSinkOutput, MetricsSinkFailure> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| {
                MetricsSinkFailure::from_kind(MetricsSinkFailureKind::MetricsExportFailed)
            })?;
        let MetricsSinkInput::SubmitQualityMetric(metric) = input;

        writeln!(
            file,
            "kind={:?} normalized_value={} unit={} window={}",
            metric.kind(),
            metric.normalized_value(),
            metric.unit(),
            metric.window()
        )
        .map_err(|_| MetricsSinkFailure::from_kind(MetricsSinkFailureKind::MetricsExportFailed))?;
        file.sync_all()
            .map_err(|_| MetricsSinkFailure::from_kind(MetricsSinkFailureKind::MetricsExportFailed))?;

        Ok(MetricsSinkOutput::Acknowledgement(
            arcrtc_core_ports::MetricsExportAcknowledgement::new(true, true),
        ))
    }
}
