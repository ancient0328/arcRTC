//! KPI-010 benchmark threshold satisfaction の判定境界です。

use arcrtc_distro_benchmark_tests::{scenario_by_id, BenchmarkMeasurement};

/// 測定済み benchmark scenario に対する採用済み閾値です。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BenchmarkThreshold {
    /// scenario idです。
    pub scenario_id: &'static str,
    /// scenario nameです。
    pub scenario_name: &'static str,
    /// 判定 metric です。
    pub metric: &'static str,
    /// metric unit です。
    pub unit: &'static str,
    /// p95 nanoseconds の上限値です。
    pub threshold_value_ns: f64,
    /// 判定規則です。
    pub comparison_rule: &'static str,
    /// 閾値超過時の failure classification です。
    pub failure_classification: &'static str,
}

/// KPI-010 benchmark threshold verdict の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkThresholdVerdict {
    /// `measured_p95_ns <= threshold_value_ns` を満たす状態です。
    Pass,
    /// `measured_p95_ns <= threshold_value_ns` を満たさない状態です。
    Fail,
}

const METRIC_P95_NS: &str = "p95_ns";
const UNIT_NS: &str = "ns";
const COMPARISON_RULE: &str = "measured_p95_ns <= threshold_value_ns";
const FAILURE_CLASSIFICATION: &str = "BenchmarkThresholdExceeded";

const THRESHOLDS: &[BenchmarkThreshold] = &[
    threshold("BENCH-001", "signaling_join_room_single", 711.481),
    threshold("BENCH-002", "signaling_join_room_batch", 23693.240),
    threshold("BENCH-003", "signaling_offer_answer_candidate", 1800.954),
    threshold("BENCH-004", "signaling_turn_credential_request", 797.925),
    threshold("BENCH-005", "turn_allocate_single", 829.858),
    threshold("BENCH-006", "turn_permission_batch", 16159.280),
    threshold("BENCH-007", "turn_channel_bind_batch", 30965.723),
    threshold("BENCH-008", "turn_relay_data_path", 1790.202),
    threshold("BENCH-009", "sfu_contract_item_create", 398.566),
    threshold("BENCH-010", "sfu_borrowed_packet_view", 402.693),
    threshold("BENCH-011", "sfu_route_select_batch", 26114.378),
    threshold("BENCH-012", "sfu_packet_fanout_intent", 6191.481),
    threshold("BENCH-013", "composition_signaling_turn_binding", 12983.477),
    threshold("BENCH-014", "composition_signaling_sfu_binding", 51125.718),
    threshold("BENCH-015", "runtime_start_shutdown", 209.523),
    threshold("BENCH-016", "evidence_json_record_write", 2202.951),
    threshold("BENCH-017", "product_policy_evaluate", 331.043),
    threshold("BENCH-018", "product_projection_mapping", 83.386),
    threshold("BENCH-019", "product_runtime_select", 73.992),
    threshold("BENCH-020", "product_drain_plan_create", 110.471),
];

const fn threshold(
    scenario_id: &'static str,
    scenario_name: &'static str,
    threshold_value_ns: f64,
) -> BenchmarkThreshold {
    BenchmarkThreshold {
        scenario_id,
        scenario_name,
        metric: METRIC_P95_NS,
        unit: UNIT_NS,
        threshold_value_ns,
        comparison_rule: COMPARISON_RULE,
        failure_classification: FAILURE_CLASSIFICATION,
    }
}

/// pre-adopted threshold だけを使って KPI-010 benchmark verdict を返します。
pub fn evaluate_kpi_benchmark_threshold_satisfaction(
    measurement: &BenchmarkMeasurement,
) -> Result<BenchmarkThresholdVerdict, &'static str> {
    if !measurement.p95_ns.is_finite() || measurement.p95_ns < 0.0 {
        return Err("invalid-measured-p95");
    }

    let threshold = THRESHOLDS
        .iter()
        .find(|threshold| threshold.scenario_id == measurement.scenario_id)
        .ok_or("threshold-not-found")?;
    let scenario = scenario_by_id(&measurement.scenario_id).ok_or("scenario-not-found")?;

    // scenario 名の二重照合で、measurement と threshold table のずれを fail-closed にする。
    if measurement.scenario_name != scenario.name || threshold.scenario_name != scenario.name {
        return Err("scenario-name-mismatch");
    }

    if measurement.p95_ns <= threshold.threshold_value_ns {
        Ok(BenchmarkThresholdVerdict::Pass)
    } else {
        Ok(BenchmarkThresholdVerdict::Fail)
    }
}
