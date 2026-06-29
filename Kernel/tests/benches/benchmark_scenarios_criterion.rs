//! v0.2 BENCH-001..016 を Criterion 統計として観測する test-side harness です。
//!
//! ここで得る値は benchmark observation であり、threshold / readiness / completion には採用しません。

use arcrtc_roadmap_tests::benchmark_runner::criterion_benchmark_cases;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn benchmark_scenarios_criterion(c: &mut Criterion) {
    let mut group = c.benchmark_group("v0_2_benchmark_scenarios");

    for case in criterion_benchmark_cases() {
        let bench_id = format!(
            "{}__{}__{}",
            case.scenario_id, case.workload_class, case.source_class
        );
        let workload = case.workload;
        group.bench_function(bench_id, |b| {
            b.iter(|| black_box(workload()));
        });

        if let Some(comparison) = case.comparison {
            let comparison_id = format!("{}__comparison__{}", case.scenario_id, comparison.label);
            let comparison_workload = comparison.workload;
            group.bench_function(comparison_id, |b| {
                b.iter(|| black_box(comparison_workload()));
            });
        }
    }

    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(750))
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = benchmark_scenarios_criterion
}
criterion_main!(benches);
