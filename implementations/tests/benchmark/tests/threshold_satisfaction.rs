//! KPI-010 benchmark threshold satisfaction の pre-adopted threshold 判定を検査します。

#[path = "../src/threshold.rs"]
mod threshold;

use arcrtc_implementation_benchmark_tests::{BenchmarkMeasurement, BENCHMARK_SCENARIOS};
use threshold::{
    evaluate_kpi_benchmark_threshold_satisfaction, BenchmarkThreshold, BenchmarkThresholdVerdict,
};

const KPI007_ADOPTED_MEASUREMENTS: &[(&str, &str, f64)] = &[
    ("BENCH-001", "signaling_join_room_single", 603.460),
    ("BENCH-002", "signaling_join_room_batch", 22760.000),
    ("BENCH-003", "signaling_offer_answer_candidate", 1748.703),
    ("BENCH-004", "signaling_turn_credential_request", 726.213),
    ("BENCH-005", "turn_allocate_single", 795.922),
    ("BENCH-006", "turn_permission_batch", 14374.523),
    ("BENCH-007", "turn_channel_bind_batch", 29812.691),
    ("BENCH-008", "turn_relay_data_path", 1743.243),
    ("BENCH-009", "sfu_contract_item_create", 389.266),
    ("BENCH-010", "sfu_borrowed_packet_view", 390.777),
    ("BENCH-011", "sfu_route_select_batch", 24414.086),
    ("BENCH-012", "sfu_packet_fanout_intent", 5314.215),
    ("BENCH-013", "composition_signaling_turn_binding", 12660.416),
    ("BENCH-014", "composition_signaling_sfu_binding", 42702.303),
    ("BENCH-015", "runtime_start_shutdown", 200.274),
    ("BENCH-016", "evidence_json_record_write", 2071.002),
    ("BENCH-017", "product_policy_evaluate", 298.028),
    ("BENCH-018", "product_projection_mapping", 70.618),
    ("BENCH-019", "product_runtime_select", 70.245),
    ("BENCH-020", "product_drain_plan_create", 105.443),
];

fn measurement(scenario_id: &str, scenario_name: &str, p95_ns: f64) -> BenchmarkMeasurement {
    BenchmarkMeasurement {
        scenario_id: scenario_id.to_owned(),
        scenario_name: scenario_name.to_owned(),
        criterion_group: "implementation_benchmark_scenarios".to_owned(),
        sample_size: 100,
        warm_up_seconds: 3,
        measurement_seconds: 10,
        noise_threshold: 0.05,
        confidence_level: 0.95,
        significance_level: 0.05,
        median_ns: p95_ns,
        mean_ns: p95_ns,
        std_dev_ns: 0.0,
        p95_ns,
        throughput_items_per_second: 1.0,
    }
}

#[test]
fn kpi_benchmark_threshold_satisfaction_verdict_uses_pre_adopted_thresholds() {
    assert_eq!(BENCHMARK_SCENARIOS.len(), 20);

    // 全 scenario が pre-adopted threshold table に存在することを、ゼロ値判定で検査する。
    for scenario in BENCHMARK_SCENARIOS {
        let verdict = evaluate_kpi_benchmark_threshold_satisfaction(&measurement(
            scenario.id,
            scenario.name,
            0.0,
        ))
        .expect("scenario threshold must be present");
        assert_eq!(verdict, BenchmarkThresholdVerdict::Pass);
    }

    let above_threshold = evaluate_kpi_benchmark_threshold_satisfaction(&measurement(
        "BENCH-001",
        "signaling_join_room_single",
        711.482,
    ))
    .expect("known scenario must evaluate");
    assert_eq!(above_threshold, BenchmarkThresholdVerdict::Fail);

    let name_mismatch = evaluate_kpi_benchmark_threshold_satisfaction(&measurement(
        "BENCH-001",
        "wrong_scenario_name",
        0.0,
    ));
    assert_eq!(name_mismatch, Err("scenario-name-mismatch"));

    let invalid_p95 = evaluate_kpi_benchmark_threshold_satisfaction(&measurement(
        "BENCH-001",
        "signaling_join_room_single",
        f64::NAN,
    ));
    assert_eq!(invalid_p95, Err("invalid-measured-p95"));

    let canonical_shape = BenchmarkThreshold {
        scenario_id: "BENCH-001",
        scenario_name: "signaling_join_room_single",
        metric: "p95_ns",
        unit: "ns",
        threshold_value_ns: 711.481,
        comparison_rule: "measured_p95_ns <= threshold_value_ns",
        failure_classification: "BenchmarkThresholdExceeded",
    };
    assert_eq!(canonical_shape.metric, "p95_ns");
    assert_eq!(canonical_shape.unit, "ns");
    assert_eq!(
        canonical_shape.comparison_rule,
        "measured_p95_ns <= threshold_value_ns"
    );
    assert_eq!(
        canonical_shape.failure_classification,
        "BenchmarkThresholdExceeded"
    );
}

#[test]
fn kpi_benchmark_threshold_satisfaction_accepts_adopted_kpi007_measurements() {
    assert_eq!(KPI007_ADOPTED_MEASUREMENTS.len(), BENCHMARK_SCENARIOS.len());

    for (scenario_id, scenario_name, p95_ns) in KPI007_ADOPTED_MEASUREMENTS {
        let verdict = evaluate_kpi_benchmark_threshold_satisfaction(&measurement(
            scenario_id,
            scenario_name,
            *p95_ns,
        ))
        .expect("adopted KPI-007 measurement must evaluate");
        assert_eq!(
            verdict,
            BenchmarkThresholdVerdict::Pass,
            "{scenario_id} adopted measurement should pass the pre-adopted threshold"
        );
    }
}
