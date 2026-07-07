const BENCHMARK_POLICY: &str = include_str!("../../tools/benchmark/benchmark-scenario-policy.toml");
const CI_MATRIX: &str = include_str!("../../tools/ci/kernel-gate-matrix.toml");

#[test]
fn benchmark_policy_declares_normalized_units_measurement_window_and_thresholds() {
    assert!(BENCHMARK_POLICY.contains("[measurement_window]"));
    assert!(BENCHMARK_POLICY.contains("warmup_seconds = 30"));
    assert!(BENCHMARK_POLICY.contains("sample_seconds = 300"));
    assert!(BENCHMARK_POLICY.contains("cooldown_seconds = 30"));

    assert!(BENCHMARK_POLICY.contains("[resource_units]"));
    assert!(BENCHMARK_POLICY.contains("latency_unit = \"milliseconds\""));
    assert!(BENCHMARK_POLICY.contains("throughput_unit = \"operations-per-second\""));
    assert!(BENCHMARK_POLICY.contains("memory_unit = \"mebibytes\""));
    assert!(BENCHMARK_POLICY.contains("cpu_unit = \"core-percent\""));

    assert!(BENCHMARK_POLICY.contains("[threshold_policy]"));
    assert!(BENCHMARK_POLICY.contains("threshold_mode = \"closed-scenario-threshold\""));
    assert!(BENCHMARK_POLICY.contains("required_threshold_fields = [\"latency_p95\", \"throughput_min\", \"error_rate_max\", \"memory_max\"]"));
}

#[test]
fn benchmark_policy_declares_load_soak_and_concurrency_scenarios() {
    for (section, scenario_id, command_id, workload_class) in [
        (
            "[scenario_matrix.load]",
            "kernel-load",
            "benchmark-load-check",
            "load",
        ),
        (
            "[scenario_matrix.soak]",
            "kernel-soak",
            "benchmark-soak-check",
            "soak",
        ),
        (
            "[scenario_matrix.concurrency]",
            "kernel-concurrency",
            "benchmark-concurrency-check",
            "concurrency",
        ),
    ] {
        assert!(BENCHMARK_POLICY.contains(section), "{section}");
        assert!(BENCHMARK_POLICY.contains(&format!("scenario_id = \"{scenario_id}\"")));
        assert!(BENCHMARK_POLICY.contains(&format!("command_id = \"{command_id}\"")));
        assert!(BENCHMARK_POLICY.contains(&format!("workload_class = \"{workload_class}\"")));
        assert!(BENCHMARK_POLICY.contains("target_surface = \"kernel-resident-runtime\""));
    }
}

#[test]
fn benchmark_command_ids_resolve_through_ci_command_line_mapping() {
    for (command_id, command_line) in [
        (
            "benchmark-load-check",
            "cargo bench -p arcrtc-benchmarks --bench kernel_load",
        ),
        (
            "benchmark-soak-check",
            "cargo bench -p arcrtc-benchmarks --bench kernel_soak",
        ),
        (
            "benchmark-concurrency-check",
            "cargo bench -p arcrtc-benchmarks --bench kernel_concurrency",
        ),
    ] {
        assert!(CI_MATRIX.contains(&format!("command_id = \"{command_id}\"")));
        assert!(CI_MATRIX.contains("command_family = \"benchmark-scenario-check\""));
        assert!(CI_MATRIX.contains("gate_class = \"benchmark\""));
        assert!(CI_MATRIX.contains(&format!("command_line = \"{command_line}\"")));
    }
}

#[test]
fn benchmark_policy_is_not_adopted_as_readiness_evidence() {
    assert!(BENCHMARK_POLICY.contains("[non_adoption]"));
    assert!(BENCHMARK_POLICY.contains("benchmark_result_is_correctness_proof = false"));
    assert!(BENCHMARK_POLICY.contains("benchmark_result_is_readiness_proof = false"));
    assert!(BENCHMARK_POLICY.contains("benchmark_result_is_release_gate_evidence = false"));
}
