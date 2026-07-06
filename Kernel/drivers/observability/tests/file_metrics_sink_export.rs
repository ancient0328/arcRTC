use arcrtc_core_ports::{MetricsExportAcknowledgement, MetricsSinkInput, MetricsSinkOutput};
use arcrtc_core_quality::{QualityMetric, QualityMetricKind};
use arcrtc_driver_observability::FileMetricsSink;

struct DirGuard(std::path::PathBuf);

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_dir() -> DirGuard {
    let path =
        std::env::temp_dir().join(format!("arcrtc_file_metrics_sink_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temp dir must be created");
    DirGuard(path)
}

#[test]
fn file_metrics_sink_exports_and_appends_lines() {
    let dir = temp_dir();
    let path = dir.0.join("metrics.log");
    let sink = FileMetricsSink::new(path.clone());
    let input = MetricsSinkInput::SubmitQualityMetric(QualityMetric::new(
        QualityMetricKind::Rtt,
        42,
        "ms",
        "60s",
    ));

    let first = sink.export(&input);
    println!("metrics_path={path:?} first_output={first:?}");
    assert_eq!(
        first,
        Ok(MetricsSinkOutput::Acknowledgement(
            MetricsExportAcknowledgement::new(true, true)
        ))
    );
    let text = std::fs::read_to_string(&path).expect("metrics file must exist");
    println!("metrics_text_after_first={text:?}");
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        vec!["kind=Rtt normalized_value=42 unit=ms window=60s"]
    );

    let second = sink.export(&input);
    println!("second_output={second:?}");
    assert_eq!(
        second,
        Ok(MetricsSinkOutput::Acknowledgement(
            MetricsExportAcknowledgement::new(true, true)
        ))
    );
    let appended = std::fs::read_to_string(&path).expect("metrics file must be readable");
    println!("metrics_text_after_second={appended:?}");
    assert_eq!(appended.lines().count(), 2);
}
