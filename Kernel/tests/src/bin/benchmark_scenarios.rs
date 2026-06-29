//! BENCH-001..016 の個別 measurement observation を出力します。

use arcrtc_roadmap_tests::benchmark_runner::run_benchmark_scenarios;

fn main() {
    println!("correlation_id=ARCRTC-V02-BENCHMARK-SCENARIOS-20260615-001");
    println!("evidence_class=benchmark");
    println!("correctness_claim=false");
    println!("production_readiness_claim=false");

    for observation in run_benchmark_scenarios() {
        println!(
            "scenario_id={} label=\"{}\" workload_class={} source_class={} status={} warmup_count={} sample_count={} measured_nanos={} comparison_label={} comparison_nanos={} non_adoption_reason={} correctness_claim={} production_readiness_claim={} threshold_claim={}",
            observation.scenario_id,
            observation.label,
            observation.workload_class,
            observation.source_class,
            observation.status,
            observation.warmup_count,
            observation.sample_count,
            observation.measured_nanos,
            observation.comparison_label.unwrap_or("none"),
            observation
                .comparison_nanos
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            observation.non_adoption_reason.unwrap_or("none"),
            observation.correctness_claim,
            observation.production_readiness_claim,
            observation.threshold_claim
        );
    }
}
