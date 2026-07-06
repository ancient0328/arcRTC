//! benchmark harness layout 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn benchmark_harness_layout_matches_canonical_file_contract() {
    for path in [
        "tests/benchmark/Cargo.toml",
        "tests/benchmark/benches/benchmark_scenarios.rs",
        "tests/benchmark/src/lib.rs",
        "tests/benchmark/src/scenario.rs",
        "tests/benchmark/src/workload.rs",
        "tests/benchmark/src/evidence.rs",
    ] {
        assert!(root().join(path).is_file(), "missing {path}");
    }
}

#[test]
fn benchmark_harness_declares_single_canonical_criterion_group() {
    let source = fs::read_to_string(root().join("tests/benchmark/benches/benchmark_scenarios.rs"))
        .expect("benchmark entrypoint must be readable");
    assert!(source
        .contains("criterion_group!(distro_benchmark_scenarios, bench_all_scenarios);"));
    assert!(source.contains("criterion_main!(distro_benchmark_scenarios);"));
    assert!(source.contains("group.sample_size(100);"));
    assert!(source.contains("execute_kpi_actual_workload(scenario)"));
    assert!(!source.contains("workload_summary.len()"));
    assert!(!source.contains("scenario.name.len()"));
}
