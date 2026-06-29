use arcrtc_roadmap_tests::benchmark_model::{validate_benchmark_report, BenchmarkReportDraft};
use arcrtc_roadmap_tests::benchmark_runner::{
    run_benchmark_scenarios, V01_ICE_SHAPE_BENCH_FUNCTIONS, V01_ICE_SHAPE_GROUP,
    V01_ICE_SHAPE_ITERATION_COUNTS,
};
use arcrtc_roadmap_tests::read_impl;
use arcrtc_roadmap_tests::semantic_obligation_manifest::{BENCHMARK_SCENARIOS, T_SERIES_TASKS};

#[test]
fn ce8_covers_every_benchmark_scenario_class() {
    assert_eq!(BENCHMARK_SCENARIOS.len(), 16);
    assert!(T_SERIES_TASKS.iter().any(|task| task.id == "T8.2"));
    for scenario in BENCHMARK_SCENARIOS {
        let report = BenchmarkReportDraft::valid_for(scenario.id);
        validate_benchmark_report(&report).expect(scenario.id);
    }
}

#[test]
fn ce8_generates_individual_observation_for_every_benchmark_scenario() {
    let observations = run_benchmark_scenarios();
    assert_eq!(observations.len(), BENCHMARK_SCENARIOS.len());

    for scenario in BENCHMARK_SCENARIOS {
        let observation = observations
            .iter()
            .find(|observation| observation.scenario_id == scenario.id)
            .unwrap_or_else(|| panic!("missing observation for {}", scenario.id));
        assert_eq!(observation.label, scenario.label);
        assert_eq!(observation.status, "measured");
        assert_ne!(
            observation.workload_class, "synthetic_boundary_workload",
            "{} must be a migrated scenario-specific workload",
            scenario.id
        );
        assert!(
            observation.source_class.starts_with("v0_1_2_"),
            "{} must retain v0.1.2 benchmark migration provenance",
            scenario.id
        );
        assert!(observation.warmup_count > 0, "{}", scenario.id);
        assert!(observation.sample_count >= 3, "{}", scenario.id);
        assert!(observation.measured_nanos > 0, "{}", scenario.id);
        assert_eq!(observation.non_adoption_reason, None, "{}", scenario.id);
        assert!(!observation.correctness_claim, "{}", scenario.id);
        assert!(!observation.production_readiness_claim, "{}", scenario.id);
        assert!(!observation.threshold_claim, "{}", scenario.id);
    }
}

#[test]
fn ce8_bench_migration_starts_with_v0_1_2_ice_pool_intent_without_threshold_import() {
    let observations = run_benchmark_scenarios();
    let ice_candidate = observations
        .iter()
        .find(|observation| observation.scenario_id == "BENCH-001")
        .expect("BENCH-001 observation must exist");

    assert_eq!(
        ice_candidate.workload_class,
        "candidate_generation_pool_vs_ondemand"
    );
    assert_eq!(
        ice_candidate.source_class,
        "v0_1_2_core_ice_pool_and_signaling_ice_reimplemented"
    );
    assert_eq!(
        ice_candidate.comparison_label,
        Some("ondemand_candidate_generation")
    );
    assert!(
        ice_candidate
            .comparison_nanos
            .is_some_and(|value| value > 0),
        "BENCH-001 must preserve the v0.1.2 comparison shape"
    );
    assert!(
        !ice_candidate.threshold_claim,
        "v0.1.2 1.5x target must not be imported as a v0.2 threshold"
    );
    assert!(
        !ice_candidate.correctness_claim && !ice_candidate.production_readiness_claim,
        "benchmark migration must not become correctness/readiness evidence"
    );
}

#[test]
fn ce8_v0_1_2_ice_shape_criterion_surface_is_comparable_without_readiness_claim() {
    let bench_source = read_impl("tests/benches/v01_ice_shape_criterion.rs");
    let manifest = read_impl("tests/Cargo.toml");

    assert!(
        manifest.contains("name = \"v01_ice_shape_criterion\""),
        "v0.2 must expose a dedicated Criterion target for v0.1.2 ice bench comparison"
    );
    assert!(
        bench_source.contains("ComparableIceCandidatePool"),
        "comparison target must stay on the v0.2 test-side comparable pool"
    );
    assert!(
        bench_source.contains("ComparableIceCandidateTemplate"),
        "comparison target must preserve the pool-vs-ondemand template surface"
    );

    for bench_function in V01_ICE_SHAPE_BENCH_FUNCTIONS {
        assert!(
            bench_source.contains(bench_function),
            "v0.1.2 bench function `{bench_function}` must exist in the v0.2 comparison target"
        );
    }

    assert!(
        bench_source.contains(V01_ICE_SHAPE_GROUP),
        "v0.1.2 benchmark group must exist in the v0.2 comparison target"
    );
    for iterations in V01_ICE_SHAPE_ITERATION_COUNTS {
        assert!(
            bench_source.contains(&iterations.to_string()),
            "v0.1.2 input size `{iterations}` must exist in the v0.2 comparison target"
        );
    }
}

#[test]
fn ce8_migrates_every_v0_1_2_benchmark_group_to_explicit_v0_2_workload_class() {
    let observations = run_benchmark_scenarios();
    let expected = [
        (
            "BENCH-001",
            "candidate_generation_pool_vs_ondemand",
            "v0_1_2_core_ice_pool_and_signaling_ice_reimplemented",
        ),
        (
            "BENCH-002",
            "sfu_route_transition_decision_matrix",
            "v0_1_2_sfu_realistic_unit_quick_and_performance_reimplemented",
        ),
        (
            "BENCH-003",
            "sfu_borrowed_packet_semantic_view",
            "v0_1_2_sfu_packet_semantic_and_realistic_benchmarks_reimplemented",
        ),
        (
            "BENCH-004",
            "packet_rewrite_transform_intent_matrix",
            "v0_1_2_sfu_rewrite_transform_and_simd_intent_reframed",
        ),
        (
            "BENCH-005",
            "driver_command_conversion_guard",
            "v0_1_2_driver_boundary_performance_reimplemented",
        ),
        (
            "BENCH-006",
            "topology_service_discovery_guard",
            "v0_1_2_service_discovery_diagnostic_reimplemented",
        ),
        (
            "BENCH-007",
            "distributed_owner_failover_policy",
            "v0_1_2_failover_and_phase4_integrated_scenarios_reframed",
        ),
        (
            "BENCH-008",
            "runtime_task_lifecycle_policy",
            "v0_1_2_concurrency_tokio_and_worker_benchmarks_reimplemented",
        ),
        (
            "BENCH-009",
            "internal_service_trust_mapping",
            "v0_1_2_internal_trust_diagnostic_reimplemented",
        ),
        (
            "BENCH-010",
            "cross_plane_binding_policy",
            "v0_1_2_integrated_cross_plane_scenarios_reframed",
        ),
        (
            "BENCH-011",
            "turn_wire_decode_to_core_command",
            "v0_1_2_turn_wire_benchmarks_reimplemented",
        ),
        (
            "BENCH-012",
            "resource_bound_saturation_catalog",
            "v0_1_2_sfu_capacity_concurrency_and_worker_resource_reframed",
        ),
        (
            "BENCH-013",
            "audit_hash_chain_append_verify",
            "v0_1_2_core_ahash_benchmark_reframed_as_hash_chain",
        ),
        (
            "BENCH-014",
            "canonical_serialization_digest",
            "v0_1_2_core_ahash_benchmark_reframed_as_canonical_serialization",
        ),
        (
            "BENCH-015",
            "persistence_export_port_intent",
            "v0_1_2_output_and_export_diagnostic_reimplemented",
        ),
        (
            "BENCH-016",
            "sdk_signaling_roundtrip_projection",
            "v0_1_2_real_ice_and_sdk_signaling_projection_reimplemented",
        ),
    ];

    for (scenario_id, workload_class, source_class) in expected {
        let observation = observations
            .iter()
            .find(|observation| observation.scenario_id == scenario_id)
            .unwrap_or_else(|| panic!("missing observation for {scenario_id}"));
        assert_eq!(observation.workload_class, workload_class, "{scenario_id}");
        assert_eq!(observation.source_class, source_class, "{scenario_id}");
    }
}

#[test]
fn ce8_rejects_unknown_or_incomplete_benchmark_report() {
    let unknown = BenchmarkReportDraft::valid_for("BENCH-999");
    assert_eq!(validate_benchmark_report(&unknown), Err("unknown_scenario"));

    let mut incomplete = BenchmarkReportDraft::valid_for("BENCH-001");
    incomplete.hardware_os_runtime_toolchain = None;
    assert_eq!(
        validate_benchmark_report(&incomplete),
        Err("missing_required_field")
    );
}

#[test]
fn ce8_rejects_correctness_or_v01_threshold_substitution() {
    let mut correctness = BenchmarkReportDraft::valid_for("BENCH-001");
    correctness.claims_correctness = true;
    assert_eq!(
        validate_benchmark_report(&correctness),
        Err("benchmark_correctness_substitution")
    );

    let mut v01 = BenchmarkReportDraft::valid_for("BENCH-001");
    v01.uses_v01_threshold = true;
    assert_eq!(
        validate_benchmark_report(&v01),
        Err("v0_1_threshold_substitution")
    );

    let mut relocation = BenchmarkReportDraft::valid_for("BENCH-001");
    relocation.moves_domain_rule_to_driver = true;
    assert_eq!(
        validate_benchmark_report(&relocation),
        Err("benchmark_domain_relocation")
    );
}
