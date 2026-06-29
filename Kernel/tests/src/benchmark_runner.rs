//! BENCH-001..016 を個別 observation として生成する test-side runner です。
//!
//! ここで得る数値は性能観測であり、正しさ・完成・production readiness の証明には使いません。

mod boundary;
mod evidence;
mod ice;
mod media;
mod support;
mod v01_ice_shape;

use std::hint::black_box;
use std::time::Instant;

use crate::semantic_obligation_manifest::BENCHMARK_SCENARIOS;

const DEFAULT_WARMUP_COUNT: u32 = 1;
const DEFAULT_SAMPLE_COUNT: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkScenarioObservation {
    pub scenario_id: &'static str,
    pub label: &'static str,
    pub workload_class: &'static str,
    pub source_class: &'static str,
    pub status: &'static str,
    pub warmup_count: u32,
    pub sample_count: u32,
    pub measured_nanos: u128,
    pub comparison_label: Option<&'static str>,
    pub comparison_nanos: Option<u128>,
    pub non_adoption_reason: Option<&'static str>,
    pub correctness_claim: bool,
    pub production_readiness_claim: bool,
    pub threshold_claim: bool,
}

/// Criterion から直接実行する v0.2 benchmark scenario の workload です。
///
/// production source へ Criterion 依存を持ち込まず、test-side runner の測定対象だけを公開します。
#[derive(Clone, Copy)]
pub struct BenchmarkCriterionCase {
    /// `BENCH-001` のような scenario ID です。
    pub scenario_id: &'static str,
    /// scenario matrix の表示名です。
    pub label: &'static str,
    /// v0.2 で再分類した workload class です。
    pub workload_class: &'static str,
    /// v0.1.2 由来または再分類 source class です。
    pub source_class: &'static str,
    /// Criterion が反復実行する対象 workload です。
    pub workload: fn() -> u64,
    /// 比較対象がある場合のみ設定します。
    pub comparison: Option<BenchmarkCriterionComparison>,
}

/// Criterion で同一 scenario 内の比較対象を測るための workload です。
#[derive(Clone, Copy)]
pub struct BenchmarkCriterionComparison {
    /// 比較対象の label です。
    pub label: &'static str,
    /// Criterion が反復実行する比較 workload です。
    pub workload: fn() -> u64,
}

pub use v01_ice_shape::{
    ComparableIceCandidatePool, ComparableIceCandidateTemplate, V01_ICE_SHAPE_BENCH_FUNCTIONS,
    V01_ICE_SHAPE_GROUP, V01_ICE_SHAPE_ITERATION_COUNTS,
};

struct BenchmarkScenarioWorkload {
    workload_class: &'static str,
    source_class: &'static str,
    workload: fn() -> u64,
    comparison: Option<BenchmarkCriterionComparison>,
    sample_count: u32,
}

pub fn run_benchmark_scenarios() -> Vec<BenchmarkScenarioObservation> {
    BENCHMARK_SCENARIOS
        .iter()
        .map(|scenario| {
            let workload = workload_for(scenario.id);
            observe_scenario(scenario.id, scenario.label, workload)
        })
        .collect()
}

/// v0.2 BENCH scenario を Criterion harness へ渡すための test-side case list です。
///
/// 数値は Criterion 観測としてのみ扱い、correctness / readiness / threshold claim へ昇格しません。
pub fn criterion_benchmark_cases() -> Vec<BenchmarkCriterionCase> {
    BENCHMARK_SCENARIOS
        .iter()
        .map(|scenario| {
            let workload = workload_for(scenario.id);
            BenchmarkCriterionCase {
                scenario_id: scenario.id,
                label: scenario.label,
                workload_class: workload.workload_class,
                source_class: workload.source_class,
                workload: workload.workload,
                comparison: workload.comparison,
            }
        })
        .collect()
}

fn observe_scenario(
    scenario_id: &'static str,
    label: &'static str,
    workload: BenchmarkScenarioWorkload,
) -> BenchmarkScenarioObservation {
    let measured_nanos = median_nanos(
        DEFAULT_WARMUP_COUNT,
        workload.sample_count,
        workload.workload,
    );
    let (comparison_label, comparison_nanos) =
        workload.comparison.map_or((None, None), |comparison| {
            (
                Some(comparison.label),
                Some(median_nanos(
                    DEFAULT_WARMUP_COUNT,
                    workload.sample_count,
                    comparison.workload,
                )),
            )
        });

    BenchmarkScenarioObservation {
        scenario_id,
        label,
        workload_class: workload.workload_class,
        source_class: workload.source_class,
        status: "measured",
        warmup_count: DEFAULT_WARMUP_COUNT,
        sample_count: workload.sample_count,
        measured_nanos,
        comparison_label,
        comparison_nanos,
        non_adoption_reason: None,
        correctness_claim: false,
        production_readiness_claim: false,
        threshold_claim: false,
    }
}

fn workload_for(scenario_id: &str) -> BenchmarkScenarioWorkload {
    match scenario_id {
        "BENCH-001" => BenchmarkScenarioWorkload {
            workload_class: "candidate_generation_pool_vs_ondemand",
            source_class: "v0_1_2_core_ice_pool_and_signaling_ice_reimplemented",
            workload: ice::pooled_candidate_generation_workload,
            comparison: Some(BenchmarkCriterionComparison {
                label: "ondemand_candidate_generation",
                workload: ice::ondemand_candidate_generation_workload,
            }),
            sample_count: 7,
        },
        "BENCH-002" => BenchmarkScenarioWorkload {
            workload_class: "sfu_route_transition_decision_matrix",
            source_class: "v0_1_2_sfu_realistic_unit_quick_and_performance_reimplemented",
            workload: media::sfu_route_transition_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-003" => BenchmarkScenarioWorkload {
            workload_class: "sfu_borrowed_packet_semantic_view",
            source_class: "v0_1_2_sfu_packet_semantic_and_realistic_benchmarks_reimplemented",
            workload: media::packet_semantic_view_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-004" => BenchmarkScenarioWorkload {
            workload_class: "packet_rewrite_transform_intent_matrix",
            source_class: "v0_1_2_sfu_rewrite_transform_and_simd_intent_reframed",
            workload: media::packet_rewrite_transform_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-005" => BenchmarkScenarioWorkload {
            workload_class: "driver_command_conversion_guard",
            source_class: "v0_1_2_driver_boundary_performance_reimplemented",
            workload: boundary::driver_conversion_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-006" => BenchmarkScenarioWorkload {
            workload_class: "topology_service_discovery_guard",
            source_class: "v0_1_2_service_discovery_diagnostic_reimplemented",
            workload: boundary::service_discovery_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-007" => BenchmarkScenarioWorkload {
            workload_class: "distributed_owner_failover_policy",
            source_class: "v0_1_2_failover_and_phase4_integrated_scenarios_reframed",
            workload: boundary::failover_diagnostic_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-008" => BenchmarkScenarioWorkload {
            workload_class: "runtime_task_lifecycle_policy",
            source_class: "v0_1_2_concurrency_tokio_and_worker_benchmarks_reimplemented",
            workload: boundary::runtime_task_lifecycle_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-009" => BenchmarkScenarioWorkload {
            workload_class: "internal_service_trust_mapping",
            source_class: "v0_1_2_internal_trust_diagnostic_reimplemented",
            workload: boundary::internal_service_trust_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-010" => BenchmarkScenarioWorkload {
            workload_class: "cross_plane_binding_policy",
            source_class: "v0_1_2_integrated_cross_plane_scenarios_reframed",
            workload: boundary::cross_plane_binding_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-011" => BenchmarkScenarioWorkload {
            workload_class: "turn_wire_decode_to_core_command",
            source_class: "v0_1_2_turn_wire_benchmarks_reimplemented",
            workload: boundary::turn_wire_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-012" => BenchmarkScenarioWorkload {
            workload_class: "resource_bound_saturation_catalog",
            source_class: "v0_1_2_sfu_capacity_concurrency_and_worker_resource_reframed",
            workload: boundary::resource_saturation_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-013" => BenchmarkScenarioWorkload {
            workload_class: "audit_hash_chain_append_verify",
            source_class: "v0_1_2_core_ahash_benchmark_reframed_as_hash_chain",
            workload: evidence::audit_hash_chain_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-014" => BenchmarkScenarioWorkload {
            workload_class: "canonical_serialization_digest",
            source_class: "v0_1_2_core_ahash_benchmark_reframed_as_canonical_serialization",
            workload: evidence::canonical_serialization_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-015" => BenchmarkScenarioWorkload {
            workload_class: "persistence_export_port_intent",
            source_class: "v0_1_2_output_and_export_diagnostic_reimplemented",
            workload: evidence::persistence_export_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        "BENCH-016" => BenchmarkScenarioWorkload {
            workload_class: "sdk_signaling_roundtrip_projection",
            source_class: "v0_1_2_real_ice_and_sdk_signaling_projection_reimplemented",
            workload: evidence::sdk_signaling_roundtrip_workload,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
        _ => BenchmarkScenarioWorkload {
            workload_class: "invalid_benchmark_scenario",
            source_class: "invalid_scenario_not_adopted",
            workload: || 1,
            comparison: None,
            sample_count: DEFAULT_SAMPLE_COUNT,
        },
    }
}

fn median_nanos<T>(warmup_count: u32, sample_count: u32, mut workload: impl FnMut() -> T) -> u128 {
    for _ in 0..warmup_count {
        black_box(workload());
    }

    let mut samples = Vec::with_capacity(sample_count as usize);
    for _ in 0..sample_count {
        let started = Instant::now();
        black_box(workload());
        samples.push(started.elapsed().as_nanos().max(1));
    }
    samples.sort_unstable();
    samples[samples.len() / 2]
}
