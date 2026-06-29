//! benchmark measurement schema 境界を検査します。

use arcrtc_implementation_benchmark_tests::{
    build_benchmark_evidence_record, build_kpi_benchmark_evidence_record_from_measurement,
    execute_kpi_actual_workload, validate_benchmark_evidence_record, workload_id_for_scenario,
    BenchmarkEvidenceValidationError, BenchmarkMeasurement, BENCHMARK_SCENARIOS,
};
use arcrtc_implementation_evidence::{
    EvidenceValidationError, ImplementationCommandClass, ImplementationEnvironmentClass,
    ImplementationEvidenceReason, ImplementationEvidenceRecord, ImplementationLayer,
    ImplementationNonClaimScope, ImplementationPlane, IMPLEMENTATIONS_COMMAND_ROOT,
};

fn base() -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "benchmark-measurement-schema".to_owned(),
        command:
            "cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios"
                .to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-benchmark-tests".to_owned()),
        target_scope: "tests/benchmark".to_owned(),
        command_class: ImplementationCommandClass::Benchmark,
        implementation_layer: ImplementationLayer::Reference,
        target_plane: ImplementationPlane::Signaling,
        environment_class: ImplementationEnvironmentClass::BenchmarkHost,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("BENCH-001:signaling_join_room_single".to_owned()),
        expected_outcome: "measurement emitted".to_owned(),
        actual_outcome: "measurement emitted".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::BenchmarkThresholdNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "rerun when benchmark scenario changes".to_owned(),
    }
}

fn measurement() -> BenchmarkMeasurement {
    BenchmarkMeasurement {
        scenario_id: "BENCH-001".to_owned(),
        scenario_name: "signaling_join_room_single".to_owned(),
        criterion_group: "implementation_benchmark_scenarios".to_owned(),
        sample_size: 100,
        warm_up_seconds: 3,
        measurement_seconds: 10,
        noise_threshold: 0.05,
        confidence_level: 0.95,
        significance_level: 0.05,
        median_ns: 1.0,
        mean_ns: 1.1,
        std_dev_ns: 0.1,
        p95_ns: 2.0,
        throughput_items_per_second: 1000.0,
    }
}

#[test]
fn benchmark_measurement_schema_accepts_canonical_configuration() {
    let record = build_benchmark_evidence_record(base(), measurement()).expect("valid benchmark");
    assert_eq!(validate_benchmark_evidence_record(&record), Ok(()));
}

#[test]
fn benchmark_measurement_schema_rejects_invalid_scope_and_values() {
    let mut wrong_command_class = base();
    wrong_command_class.command_class = ImplementationCommandClass::Test;
    assert_eq!(
        build_benchmark_evidence_record(wrong_command_class, measurement()).unwrap_err(),
        BenchmarkEvidenceValidationError::CommandClassMismatch
    );

    let mut missing_threshold_non_claim = base();
    missing_threshold_non_claim.non_claim_scope = vec![
        ImplementationNonClaimScope::ProductionReadinessNotClaimed,
        ImplementationNonClaimScope::LiveReadinessNotClaimed,
    ];
    assert_eq!(
        build_benchmark_evidence_record(missing_threshold_non_claim, measurement()).unwrap_err(),
        BenchmarkEvidenceValidationError::Base(
            EvidenceValidationError::MissingRequiredNonClaimScope
        )
    );

    let mut unknown_scenario = measurement();
    unknown_scenario.scenario_id = "BENCH-999".to_owned();
    assert_eq!(
        build_benchmark_evidence_record(base(), unknown_scenario).unwrap_err(),
        BenchmarkEvidenceValidationError::ScenarioIdNotListed
    );

    let mut wrong_name = measurement();
    wrong_name.scenario_name = "different".to_owned();
    assert_eq!(
        build_benchmark_evidence_record(base(), wrong_name).unwrap_err(),
        BenchmarkEvidenceValidationError::ScenarioNameMismatch
    );

    let mut wrong_group = measurement();
    wrong_group.criterion_group = "other".to_owned();
    assert_eq!(
        build_benchmark_evidence_record(base(), wrong_group).unwrap_err(),
        BenchmarkEvidenceValidationError::CriterionGroupMismatch
    );

    let mut wrong_configuration = measurement();
    wrong_configuration.sample_size = 99;
    assert_eq!(
        build_benchmark_evidence_record(base(), wrong_configuration).unwrap_err(),
        BenchmarkEvidenceValidationError::CriterionConfigurationMismatch
    );

    let mut wrong_timing = measurement();
    wrong_timing.median_ns = f64::NAN;
    assert_eq!(
        build_benchmark_evidence_record(base(), wrong_timing).unwrap_err(),
        BenchmarkEvidenceValidationError::InvalidTimingValue
    );

    let mut wrong_throughput = measurement();
    wrong_throughput.throughput_items_per_second = 0.0;
    assert_eq!(
        build_benchmark_evidence_record(base(), wrong_throughput).unwrap_err(),
        BenchmarkEvidenceValidationError::InvalidThroughputValue
    );
}

#[test]
fn benchmark_record_validation_rejects_mutated_scenario_projection_fields() {
    let mut record =
        build_benchmark_evidence_record(base(), measurement()).expect("valid benchmark");
    record.layer = ImplementationLayer::Product;
    assert_eq!(
        validate_benchmark_evidence_record(&record),
        Err(BenchmarkEvidenceValidationError::ScenarioLayerMismatch)
    );

    let mut record =
        build_benchmark_evidence_record(base(), measurement()).expect("valid benchmark");
    record.plane = ImplementationPlane::Turn;
    assert_eq!(
        validate_benchmark_evidence_record(&record),
        Err(BenchmarkEvidenceValidationError::ScenarioPlaneMismatch)
    );

    let mut record =
        build_benchmark_evidence_record(base(), measurement()).expect("valid benchmark");
    record.workload_summary = "different workload".to_owned();
    assert_eq!(
        validate_benchmark_evidence_record(&record),
        Err(BenchmarkEvidenceValidationError::WorkloadSummaryMismatch)
    );
}

#[test]
fn kpi_benchmark_measurement_requires_actual_workload_fields() {
    let scenario = BENCHMARK_SCENARIOS
        .iter()
        .find(|scenario| scenario.id == "BENCH-001")
        .expect("BENCH-001 must exist");
    let workload = execute_kpi_actual_workload(scenario);
    assert_eq!(workload.workload_id, workload_id_for_scenario(scenario));
    assert!(workload.completed_items > 0);

    let record = build_kpi_benchmark_evidence_record_from_measurement(base(), measurement())
        .expect("KPI benchmark evidence record must be accepted");
    assert!(record.throughput_items_per_second > 0.0);
    assert!(record
        .base
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::BenchmarkThresholdNotClaimed));

    let mut descriptor_only = base();
    descriptor_only.input_fixture_or_workload = Some("signaling_join_room_single".to_owned());
    assert_ne!(
        descriptor_only.input_fixture_or_workload.as_deref(),
        Some(workload.workload_id.as_str())
    );
}
