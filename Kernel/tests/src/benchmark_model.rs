//! CE8 benchmark report 採用条件を表す test-side model です。

use crate::semantic_obligation_manifest::{contains_id, BENCHMARK_SCENARIOS};

#[derive(Debug, Clone)]
pub struct BenchmarkReportDraft {
    pub scenario_id: &'static str,
    pub command: Option<&'static str>,
    pub working_directory: Option<&'static str>,
    pub target_package: Option<&'static str>,
    pub hardware_os_runtime_toolchain: Option<&'static str>,
    pub warmup_rule: Option<&'static str>,
    pub sample_count: Option<u32>,
    pub measurement_window: Option<&'static str>,
    pub unit: Option<&'static str>,
    pub precision: Option<&'static str>,
    pub aggregation: Option<&'static str>,
    pub fixture_source: Option<&'static str>,
    pub fake_status: Option<&'static str>,
    pub measured_value: Option<f64>,
    pub claims_correctness: bool,
    pub uses_v01_threshold: bool,
    pub moves_domain_rule_to_driver: bool,
}

impl BenchmarkReportDraft {
    pub fn valid_for(scenario_id: &'static str) -> Self {
        Self {
            scenario_id,
            command: Some("cargo run -p arcrtc-roadmap-tests --bin benchmark_scenarios"),
            working_directory: Some("v0.2/Kernel"),
            target_package: Some("arcrtc-roadmap-tests"),
            hardware_os_runtime_toolchain: Some(
                "local-hardware; macOS; rustc; deterministic test profile",
            ),
            warmup_rule: Some("one dry run"),
            sample_count: Some(3),
            measurement_window: Some("single process monotonic window"),
            unit: Some("nanoseconds"),
            precision: Some("integer nanoseconds"),
            aggregation: Some("median"),
            fixture_source: Some("synthetic"),
            fake_status: Some("deterministic support labeled"),
            measured_value: Some(1.0),
            claims_correctness: false,
            uses_v01_threshold: false,
            moves_domain_rule_to_driver: false,
        }
    }
}

pub fn validate_benchmark_report(report: &BenchmarkReportDraft) -> Result<(), &'static str> {
    if !contains_id(BENCHMARK_SCENARIOS, report.scenario_id) {
        return Err("unknown_scenario");
    }
    if report.command.is_none()
        || report.working_directory.is_none()
        || report.target_package.is_none()
        || report.hardware_os_runtime_toolchain.is_none()
        || report.warmup_rule.is_none()
        || report.sample_count.is_none()
        || report.measurement_window.is_none()
        || report.unit.is_none()
        || report.precision.is_none()
        || report.aggregation.is_none()
        || report.fixture_source.is_none()
        || report.fake_status.is_none()
        || report.measured_value.is_none()
    {
        return Err("missing_required_field");
    }
    if report.claims_correctness {
        return Err("benchmark_correctness_substitution");
    }
    if report.uses_v01_threshold {
        return Err("v0_1_threshold_substitution");
    }
    if report.moves_domain_rule_to_driver {
        return Err("benchmark_domain_relocation");
    }
    Ok(())
}
