//! T8 用の benchmark smoke command です。
//!
//! 性能値を正しさの証明に使わず、measurement evidence として採用可能な字段を出力します。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() {
    let root = implementation_root();
    let started = Instant::now();
    let mut scanned_files = 0usize;
    let mut scanned_bytes = 0usize;

    for directory in [
        "core",
        "drivers",
        "entrypoints",
        "tests",
        "integration-tests",
    ] {
        let path = root.join(directory);
        scan_utf8_files(&path, &mut scanned_files, &mut scanned_bytes);
    }

    let elapsed = started.elapsed();

    println!("correlation_id=ARCRTC-V02-BENCHMARK-SMOKE-20260615-001");
    println!("evidence_class=benchmark");
    println!("workload=static_asset_and_source_scan");
    println!("working_directory={}", root.display());
    println!("scanned_files={scanned_files}");
    println!("scanned_bytes={scanned_bytes}");
    println!("elapsed_nanos={}", elapsed.as_nanos());
    println!("correctness_claim=false");
    println!("production_readiness_claim=false");
    println!(
        "rerun_condition=rerun when source, tests, integration-tests, or benchmark asset changes"
    );
}

fn implementation_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tests crate must live directly under implementation root")
        .to_path_buf()
}

fn scan_utf8_files(path: &Path, scanned_files: &mut usize, scanned_bytes: &mut usize) {
    let entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to list {}: {error}", path.display()));

    for entry in entries {
        let entry = entry.expect("directory entry must be readable");
        let path = entry.path();
        if path.is_dir() {
            scan_utf8_files(&path, scanned_files, scanned_bytes);
            continue;
        }

        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("rs" | "toml" | "md")
        ) {
            continue;
        }

        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        *scanned_files += 1;
        *scanned_bytes += bytes.len();
    }
}
