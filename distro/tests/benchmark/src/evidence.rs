//! benchmark measurement を evidence extension record へ変換します。

use arcrtc_distro_evidence::{
    validate_evidence_record, EvidenceValidationError, DistroCommandClass,
    DistroEvidenceRecord, DistroLayer, DistroNonClaimScope,
    DistroPlane,
};

use crate::scenario::scenario_by_id;

/// Criterion measurement の固定 field set です。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkMeasurement {
    /// scenario idです。
    pub scenario_id: String,
    /// scenario nameです。
    pub scenario_name: String,
    /// Criterion groupです。
    pub criterion_group: String,
    /// sample sizeです。
    pub sample_size: usize,
    /// warm up secondsです。
    pub warm_up_seconds: u64,
    /// measurement secondsです。
    pub measurement_seconds: u64,
    /// noise thresholdです。
    pub noise_threshold: f64,
    /// confidence levelです。
    pub confidence_level: f64,
    /// significance levelです。
    pub significance_level: f64,
    /// median nanosecondsです。
    pub median_ns: f64,
    /// mean nanosecondsです。
    pub mean_ns: f64,
    /// standard deviation nanosecondsです。
    pub std_dev_ns: f64,
    /// p95 nanosecondsです。
    pub p95_ns: f64,
    /// throughput items per secondです。
    pub throughput_items_per_second: f64,
}

/// benchmark evidence extension record です。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkEvidenceRecord {
    /// base evidence recordです。
    #[serde(flatten)]
    pub base: DistroEvidenceRecord,
    /// scenario idです。
    pub scenario_id: String,
    /// scenario nameです。
    pub scenario_name: String,
    /// distro layerです。
    pub layer: DistroLayer,
    /// target planeです。
    pub plane: DistroPlane,
    /// workload summaryです。
    pub workload_summary: String,
    /// Criterion groupです。
    pub criterion_group: String,
    /// sample sizeです。
    pub sample_size: usize,
    /// warm up secondsです。
    pub warm_up_seconds: u64,
    /// measurement secondsです。
    pub measurement_seconds: u64,
    /// noise thresholdです。
    pub noise_threshold: f64,
    /// confidence levelです。
    pub confidence_level: f64,
    /// significance levelです。
    pub significance_level: f64,
    /// median nanosecondsです。
    pub median_ns: f64,
    /// mean nanosecondsです。
    pub mean_ns: f64,
    /// standard deviation nanosecondsです。
    pub std_dev_ns: f64,
    /// p95 nanosecondsです。
    pub p95_ns: f64,
    /// throughput items per secondです。
    pub throughput_items_per_second: f64,
}

/// benchmark evidence validation error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BenchmarkEvidenceValidationError {
    /// base validation errorです。
    Base(EvidenceValidationError),
    /// command class が Benchmark ではありません。
    CommandClassMismatch,
    /// scenario id が一覧外です。
    ScenarioIdNotListed,
    /// scenario name が一致しません。
    ScenarioNameMismatch,
    /// layer が一致しません。
    ScenarioLayerMismatch,
    /// plane が一致しません。
    ScenarioPlaneMismatch,
    /// workload summary が一致しません。
    WorkloadSummaryMismatch,
    /// Criterion group が一致しません。
    CriterionGroupMismatch,
    /// Criterion configuration が一致しません。
    CriterionConfigurationMismatch,
    /// timing value が不正です。
    InvalidTimingValue,
    /// throughput value が不正です。
    InvalidThroughputValue,
}

/// measurement を benchmark evidence record に変換します。
pub fn build_benchmark_evidence_record(
    base: DistroEvidenceRecord,
    measurement: BenchmarkMeasurement,
) -> Result<BenchmarkEvidenceRecord, BenchmarkEvidenceValidationError> {
    let scenario = scenario_by_id(&measurement.scenario_id)
        .ok_or(BenchmarkEvidenceValidationError::ScenarioIdNotListed)?;
    let record = BenchmarkEvidenceRecord {
        layer: base.distro_layer,
        plane: base.target_plane,
        workload_summary: scenario.workload_summary.to_owned(),
        scenario_id: measurement.scenario_id,
        scenario_name: measurement.scenario_name,
        criterion_group: measurement.criterion_group,
        sample_size: measurement.sample_size,
        warm_up_seconds: measurement.warm_up_seconds,
        measurement_seconds: measurement.measurement_seconds,
        noise_threshold: measurement.noise_threshold,
        confidence_level: measurement.confidence_level,
        significance_level: measurement.significance_level,
        median_ns: measurement.median_ns,
        mean_ns: measurement.mean_ns,
        std_dev_ns: measurement.std_dev_ns,
        p95_ns: measurement.p95_ns,
        throughput_items_per_second: measurement.throughput_items_per_second,
        base,
    };
    validate_benchmark_evidence_record(&record)?;
    Ok(record)
}

/// KPI-T6 用の measurement-to-evidence conversion entrypoint です。
pub fn build_kpi_benchmark_evidence_record_from_measurement(
    base: DistroEvidenceRecord,
    measurement: BenchmarkMeasurement,
) -> Result<BenchmarkEvidenceRecord, BenchmarkEvidenceValidationError> {
    build_benchmark_evidence_record(base, measurement)
}

/// benchmark evidence record を検証します。
pub fn validate_benchmark_evidence_record(
    record: &BenchmarkEvidenceRecord,
) -> Result<(), BenchmarkEvidenceValidationError> {
    validate_evidence_record(&record.base).map_err(BenchmarkEvidenceValidationError::Base)?;
    if record.base.command_class != DistroCommandClass::Benchmark {
        return Err(BenchmarkEvidenceValidationError::CommandClassMismatch);
    }
    if !record
        .base
        .non_claim_scope
        .contains(&DistroNonClaimScope::BenchmarkThresholdNotClaimed)
    {
        return Err(BenchmarkEvidenceValidationError::CommandClassMismatch);
    }
    let scenario = scenario_by_id(&record.scenario_id)
        .ok_or(BenchmarkEvidenceValidationError::ScenarioIdNotListed)?;
    if record.scenario_name != scenario.name {
        return Err(BenchmarkEvidenceValidationError::ScenarioNameMismatch);
    }
    if record.layer != record.base.distro_layer || record.layer != scenario.layer {
        return Err(BenchmarkEvidenceValidationError::ScenarioLayerMismatch);
    }
    if record.plane != record.base.target_plane || record.plane != scenario.plane {
        return Err(BenchmarkEvidenceValidationError::ScenarioPlaneMismatch);
    }
    if record.workload_summary != scenario.workload_summary {
        return Err(BenchmarkEvidenceValidationError::WorkloadSummaryMismatch);
    }
    if record.criterion_group != "distro_benchmark_scenarios" {
        return Err(BenchmarkEvidenceValidationError::CriterionGroupMismatch);
    }
    if record.sample_size != 100
        || record.warm_up_seconds != 3
        || record.measurement_seconds != 10
        || record.noise_threshold != 0.05
        || record.confidence_level != 0.95
        || record.significance_level != 0.05
    {
        return Err(BenchmarkEvidenceValidationError::CriterionConfigurationMismatch);
    }
    for value in [
        record.median_ns,
        record.mean_ns,
        record.std_dev_ns,
        record.p95_ns,
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(BenchmarkEvidenceValidationError::InvalidTimingValue);
        }
    }
    if !record.throughput_items_per_second.is_finite() || record.throughput_items_per_second <= 0.0
    {
        return Err(BenchmarkEvidenceValidationError::InvalidThroughputValue);
    }
    Ok(())
}
