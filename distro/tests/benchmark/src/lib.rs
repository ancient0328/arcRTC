//! benchmark test package の公開 helper 境界です。

pub mod evidence;
pub mod scenario;
pub mod workload;

pub use evidence::{
    build_benchmark_evidence_record, build_kpi_benchmark_evidence_record_from_measurement,
    validate_benchmark_evidence_record, BenchmarkEvidenceRecord, BenchmarkEvidenceValidationError,
    BenchmarkMeasurement,
};
pub use scenario::{scenario_by_id, BenchmarkScenario, BENCHMARK_SCENARIOS};
pub use workload::{
    execute_kpi_actual_workload, workload_id_for_scenario, BenchmarkWorkload,
    KernelProductionBenchmarkWorkload,
};
