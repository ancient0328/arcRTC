//! `benchmark-soak-check` が実行する soak lane の Criterion harness です。

use arcrtc_benchmarks::{cases_for_lane, BenchmarkLane};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn kernel_soak(c: &mut Criterion) {
    let cases = cases_for_lane(BenchmarkLane::Soak);
    assert!(
        !cases.is_empty(),
        "benchmark-soak-check requires at least one workload"
    );

    let mut group = c.benchmark_group("fixed_goal_v3_kernel_soak");
    for case in cases {
        let workload = case.workload;
        let bench_id = format!(
            "{}__{}__{}",
            case.scenario_id, case.workload_class, case.source_class
        );
        group.bench_function(bench_id, |b| {
            b.iter(|| black_box(workload()));
        });
    }
    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(50))
        .measurement_time(Duration::from_millis(250))
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = kernel_soak
}
criterion_main!(benches);
