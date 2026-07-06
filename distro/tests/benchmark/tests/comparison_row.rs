//! benchmark comparison row admission 境界を検査します。

use arcrtc_distro_benchmark_tests::{scenario_by_id, workload_id_for_scenario};
use arcrtc_distro_evidence::{
    DistroLayer, DistroNonClaimScope, DistroPlane,
};

struct ComparisonRow {
    scenario_id: String,
    distro_layer: DistroLayer,
    target_plane: DistroPlane,
    workload_id: String,
    metric: String,
    unit: String,
    aggregation_rule: String,
    environment_class: String,
    toolchain_runtime: String,
    sample_count: usize,
    warmup_rule: String,
    timestamp: String,
    non_claim_scope: Vec<DistroNonClaimScope>,
}

fn valid_row() -> ComparisonRow {
    ComparisonRow {
        scenario_id: "BENCH-001".to_owned(),
        distro_layer: DistroLayer::Reference,
        target_plane: DistroPlane::Signaling,
        workload_id: workload_id_for_scenario(scenario_by_id("BENCH-001").expect("scenario")),
        metric: "throughput_items_per_second".to_owned(),
        unit: "items_per_second".to_owned(),
        aggregation_rule: "criterion median/mean/p95 projection".to_owned(),
        environment_class: "benchmark_host".to_owned(),
        toolchain_runtime: "rustc 1.96".to_owned(),
        sample_count: 100,
        warmup_rule: "3 seconds".to_owned(),
        timestamp: "2026-06-21T00:00:00+09:00".to_owned(),
        non_claim_scope: vec![DistroNonClaimScope::BenchmarkThresholdNotClaimed],
    }
}

fn validate_row(row: &ComparisonRow) -> Result<(), &'static str> {
    let scenario = scenario_by_id(&row.scenario_id).ok_or("scenario")?;
    if row.distro_layer != scenario.layer || row.target_plane != scenario.plane {
        return Err("layer-plane");
    }
    if row.workload_id.is_empty() || row.workload_id != workload_id_for_scenario(scenario) {
        return Err("workload");
    }
    if row.metric != "throughput_items_per_second" || row.unit != "items_per_second" {
        return Err("metric-unit");
    }
    if row.aggregation_rule != "criterion median/mean/p95 projection" {
        return Err("aggregation");
    }
    if row.environment_class.is_empty()
        || row.toolchain_runtime.is_empty()
        || row.timestamp.is_empty()
    {
        return Err("field");
    }
    if row.warmup_rule != "3 seconds" {
        return Err("warmup");
    }
    if row.sample_count != 100 {
        return Err("sample");
    }
    if !row
        .non_claim_scope
        .contains(&DistroNonClaimScope::BenchmarkThresholdNotClaimed)
    {
        return Err("non-claim");
    }
    Ok(())
}

#[test]
fn kpi_benchmark_comparison_row_rejects_threshold_claim_without_admission() {
    assert_eq!(validate_row(&valid_row()), Ok(()));

    let mut missing_threshold_non_claim = valid_row();
    missing_threshold_non_claim.non_claim_scope.clear();
    assert_eq!(validate_row(&missing_threshold_non_claim), Err("non-claim"));

    let mut invalid_scenario = valid_row();
    invalid_scenario.scenario_id = "BENCH-999".to_owned();
    assert_eq!(validate_row(&invalid_scenario), Err("scenario"));

    let mut wrong_layer = valid_row();
    wrong_layer.distro_layer = DistroLayer::Product;
    assert_eq!(validate_row(&wrong_layer), Err("layer-plane"));

    let mut wrong_plane = valid_row();
    wrong_plane.target_plane = DistroPlane::Turn;
    assert_eq!(validate_row(&wrong_plane), Err("layer-plane"));

    let mut wrong_workload = valid_row();
    wrong_workload.workload_id = "different".to_owned();
    assert_eq!(validate_row(&wrong_workload), Err("workload"));

    let mut empty_workload = valid_row();
    empty_workload.workload_id.clear();
    assert_eq!(validate_row(&empty_workload), Err("workload"));

    let mut wrong_metric = valid_row();
    wrong_metric.metric = "median_ns".to_owned();
    assert_eq!(validate_row(&wrong_metric), Err("metric-unit"));

    let mut wrong_unit = valid_row();
    wrong_unit.unit = "nanoseconds".to_owned();
    assert_eq!(validate_row(&wrong_unit), Err("metric-unit"));

    let mut wrong_sample = valid_row();
    wrong_sample.sample_count = 99;
    assert_eq!(validate_row(&wrong_sample), Err("sample"));

    let mut missing_environment = valid_row();
    missing_environment.environment_class.clear();
    assert_eq!(validate_row(&missing_environment), Err("field"));

    let mut missing_aggregation = valid_row();
    missing_aggregation.aggregation_rule.clear();
    assert_eq!(validate_row(&missing_aggregation), Err("aggregation"));

    let mut wrong_aggregation = valid_row();
    wrong_aggregation.aggregation_rule = "mean only".to_owned();
    assert_eq!(validate_row(&wrong_aggregation), Err("aggregation"));

    let mut missing_toolchain = valid_row();
    missing_toolchain.toolchain_runtime.clear();
    assert_eq!(validate_row(&missing_toolchain), Err("field"));

    let mut missing_warmup = valid_row();
    missing_warmup.warmup_rule.clear();
    assert_eq!(validate_row(&missing_warmup), Err("warmup"));

    let mut wrong_warmup = valid_row();
    wrong_warmup.warmup_rule = "1 second".to_owned();
    assert_eq!(validate_row(&wrong_warmup), Err("warmup"));

    let mut missing_timestamp = valid_row();
    missing_timestamp.timestamp.clear();
    assert_eq!(validate_row(&missing_timestamp), Err("field"));
}
