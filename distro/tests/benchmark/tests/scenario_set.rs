//! benchmark scenario set 境界を検査します。

use arcrtc_distro_benchmark_tests::{
    execute_kpi_actual_workload, scenario_by_id, workload_id_for_scenario, BENCHMARK_SCENARIOS,
};

#[test]
fn all_canonical_benchmark_scenarios_are_present() {
    assert_eq!(BENCHMARK_SCENARIOS.len(), 20);
    for index in 1..=20 {
        let id = format!("BENCH-{index:03}");
        let scenario = scenario_by_id(&id).expect("scenario id must be present");
        assert!(!scenario.name.is_empty());
        assert!(!scenario.workload_summary.is_empty());
    }
}

#[test]
fn all_canonical_benchmark_scenarios_execute_actual_workloads() {
    for scenario in BENCHMARK_SCENARIOS {
        let workload = execute_kpi_actual_workload(scenario);
        assert_eq!(workload.workload_id, workload_id_for_scenario(scenario));
        assert!(
            workload.completed_items > 0,
            "{} must execute actual workload",
            scenario.id
        );
    }
}
