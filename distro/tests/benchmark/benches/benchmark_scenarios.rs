//! Criterion benchmark scenario harness の entrypoint です。
#![allow(missing_docs)]

// Criterion macro が生成する関数名は個別 doc comment を付与できないため、
// bench entrypoint ファイルの lint 境界として missing_docs を局所的に許可します。

use arcrtc_distro_benchmark_tests::{execute_kpi_actual_workload, BENCHMARK_SCENARIOS};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_all_scenarios(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("distro_benchmark_scenarios");
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(100);
    group.noise_threshold(0.05);
    group.confidence_level(0.95);
    group.significance_level(0.05);

    for scenario in BENCHMARK_SCENARIOS {
        let function_name = format!(
            "bench_{}_{}",
            scenario.id.to_ascii_lowercase().replace('-', "_"),
            scenario.name
        );
        group.bench_function(function_name, |bencher| {
            // 測定対象は scenario descriptor ではなく actual workload dispatch です。
            bencher.iter(|| black_box(execute_kpi_actual_workload(scenario)));
        });
    }
    group.finish();
}

criterion_group!(distro_benchmark_scenarios, bench_all_scenarios);
criterion_main!(distro_benchmark_scenarios);
