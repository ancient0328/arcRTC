//! `benchmark-load-check` が実行する load lane の Criterion harness です。

use arcrtc_benchmarks::{benchmark_lane_window, cases_for_lane, BenchmarkLane};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{thread, time::Instant};

fn kernel_load(c: &mut Criterion) {
    let window = benchmark_lane_window(BenchmarkLane::Load)
        .unwrap_or_else(|error| panic!("benchmark load window rejected: {error:?}"));
    let cases = cases_for_lane(BenchmarkLane::Load);
    assert_eq!(cases.len(), window.case_count());

    let mut group = c.benchmark_group("kernel_resident_runtime_load");
    group.sample_size(10);
    group.warm_up_time(window.warmup_per_case_duration());
    group.measurement_time(window.measurement_per_case_duration());
    let started_at = Instant::now();
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
    thread::sleep(window.cooldown_duration());
    println!(
        "benchmark_measurement_window window_scope=lane lane={} warmup_total_seconds={} sample_total_seconds={} cooldown_seconds={} case_count={} warmup_per_case_seconds={} measurement_per_case_seconds={} actual_elapsed_milliseconds={}",
        window.lane().as_str(),
        window.total_warmup_seconds(),
        window.total_sample_seconds(),
        window.cooldown_seconds(),
        window.case_count(),
        window.warmup_per_case_seconds(),
        window.measurement_per_case_seconds(),
        started_at.elapsed().as_millis(),
    );
}

criterion_group!(benches, kernel_load);
criterion_main!(benches);
