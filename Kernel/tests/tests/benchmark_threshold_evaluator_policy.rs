// Test Roadmap が `assert_<task>__<target>` 形式を要求するため、この test file に限り許可します。
#![allow(non_snake_case)]

const BENCHMARK_POLICY: &str = include_str!("../../tools/benchmark/benchmark-scenario-policy.toml");

#[test]
fn benchmark_threshold_evaluator_policy_covers_exact_values() {
    assert_t_perf_02__threshold_values_load_exact_values();
    assert_t_perf_02__threshold_values_soak_exact_values();
    assert_t_perf_02__threshold_values_concurrency_exact_values();
    assert_t_perf_02__threshold_evaluator_fields();
}

fn assert_t_perf_02__threshold_values_load_exact_values() {
    assert!(BENCHMARK_POLICY.contains("[threshold_values.load]"));
    assert!(BENCHMARK_POLICY.contains("latency_p95_ms_max = 0.005"));
    assert!(BENCHMARK_POLICY.contains("throughput_ops_per_sec_min = 200000"));
    assert!(BENCHMARK_POLICY.contains("error_rate_ratio_max = 0.0"));
    assert!(BENCHMARK_POLICY.contains("memory_mib_max = 512"));
}

fn assert_t_perf_02__threshold_values_soak_exact_values() {
    assert!(BENCHMARK_POLICY.contains("[threshold_values.soak]"));
    assert!(BENCHMARK_POLICY.contains("latency_p95_ms_max = 0.075"));
    assert!(BENCHMARK_POLICY.contains("throughput_ops_per_sec_min = 10000"));
    assert!(BENCHMARK_POLICY.contains("error_rate_ratio_max = 0.0"));
    assert!(BENCHMARK_POLICY.contains("memory_mib_max = 512"));
}

fn assert_t_perf_02__threshold_values_concurrency_exact_values() {
    assert!(BENCHMARK_POLICY.contains("[threshold_values.concurrency]"));
    assert!(BENCHMARK_POLICY.contains("latency_p95_ms_max = 0.150"));
    assert!(BENCHMARK_POLICY.contains("throughput_ops_per_sec_min = 5000"));
    assert!(BENCHMARK_POLICY.contains("error_rate_ratio_max = 0.0"));
    assert!(BENCHMARK_POLICY.contains("memory_mib_max = 512"));
}

fn assert_t_perf_02__threshold_evaluator_fields() {
    assert!(BENCHMARK_POLICY.contains("[threshold_evaluator]"));
    assert!(BENCHMARK_POLICY.contains("pass_condition = \"all_scenarios_within_thresholds\""));
    assert!(BENCHMARK_POLICY.contains("missing_metric_reason = \"MISSING_REQUIRED_FIELD\""));
    assert!(BENCHMARK_POLICY.contains("failed_metric_reason = \"BENCHMARK_THRESHOLD_FAILED\""));
    assert!(BENCHMARK_POLICY.contains(
        "non_adoption = \"threshold evaluator source is not benchmark output or readiness proof\""
    ));
}
