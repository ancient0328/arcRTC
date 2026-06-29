//! v0.1.2 `ice_pool_bench` と比較するための v0.2 test-side Criterion harness です。
//!
//! bench 名・group 名・入力数を v0.1.2 と揃えます。
//! この結果は比較観測のみであり、v0.2 production readiness や completion claim には使いません。

use arcrtc_roadmap_tests::benchmark_runner::{
    ComparableIceCandidatePool, ComparableIceCandidateTemplate,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_pool_generation(c: &mut Criterion) {
    let pool = ComparableIceCandidatePool::new();

    c.bench_function("pool_generate_candidate", |b| {
        let mut i = 0u32;
        b.iter(|| {
            let ip_num = i % 255;
            let ip = format!("192.168.1.{ip_num}");
            let port = 3478 + (i % 1000) as u16;
            i = i.wrapping_add(1);
            black_box(pool.generate_candidate(&ip, port))
        })
    });
}

fn bench_ondemand_generation(c: &mut Criterion) {
    c.bench_function("ondemand_generate_candidate", |b| {
        let mut i = 0u32;
        b.iter(|| {
            let ip_num = i % 255;
            let ip = format!("192.168.1.{ip_num}");
            let port = 3478 + (i % 1000) as u16;
            let template = ComparableIceCandidateTemplate::new(i);
            i = i.wrapping_add(1);
            black_box(template.generate_candidate(&ip, port))
        })
    });
}

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("ice_candidate_generation");

    let pool = ComparableIceCandidatePool::new();

    for iterations in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("pool", iterations), iterations, |b, &n| {
            b.iter(|| {
                for i in 0..n {
                    let ip_num = i % 255;
                    let ip = format!("192.168.1.{ip_num}");
                    let port = 3478 + (i % 1000) as u16;
                    black_box(pool.generate_candidate(&ip, port));
                }
            })
        });

        group.bench_with_input(
            BenchmarkId::new("ondemand", iterations),
            iterations,
            |b, &n| {
                b.iter(|| {
                    for i in 0..n {
                        let ip_num = i % 255;
                        let ip = format!("192.168.1.{ip_num}");
                        let port = 3478 + (i % 1000) as u16;
                        let template = ComparableIceCandidateTemplate::new(i as u32);
                        black_box(template.generate_candidate(&ip, port));
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_bulk_generation(c: &mut Criterion) {
    let pool = ComparableIceCandidatePool::new();
    let interfaces: Vec<(String, u16)> = (0..10)
        .map(|i| (format!("192.168.1.{i}"), 3478 + i))
        .collect();

    c.bench_function("bulk_generate_candidates", |b| {
        b.iter(|| black_box(pool.generate_candidates(&interfaces)))
    });
}

criterion_group!(
    benches,
    bench_pool_generation,
    bench_ondemand_generation,
    bench_comparison,
    bench_bulk_generation,
);

criterion_main!(benches);
