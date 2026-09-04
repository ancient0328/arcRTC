const BENCHMARK_POLICY: &str = include_str!("../../tools/benchmark/benchmark-scenario-policy.toml");
const CI_MATRIX: &str = include_str!("../../tools/ci/kernel-gate-matrix.toml");

fn policy_command_ids() -> Vec<&'static str> {
    BENCHMARK_POLICY
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("command_id = \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .collect()
}

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
fn benchmark_policy_command_ids_resolve_once_in_ci_matrix() {
    let command_ids = policy_command_ids();
    assert_eq!(command_ids.len(), 3);

    for command_id in command_ids {
        let match_count = CI_MATRIX
            .split("[[commands]]")
            .skip(1)
            .filter(|block| block.contains(&format!("command_id = \"{command_id}\"")))
            .count();
        assert_eq!(match_count, 1, "{command_id} must resolve exactly once");
    }
}
