# arcRTC v0.2 implementations — Benchmark Results

These are actual measured benchmark numbers (empirical evidence that the workloads run). They are **measurement evidence only** — not acceptance-threshold verdicts, and not a production / live readiness claim. The specification (scenario definitions, measurement boundary, how a threshold must be defined) lives in `docs/summary/en/10-benchmark.md`; this file holds the numbers and the exact conditions under which they were measured. A Japanese edition is at `docs/benchmark-results/ja.md`.

## Run environment

- Date: `2026-06-29 15:12:58 JST`
- Working directory: `implementations`
- Toolchain: `rustc 1.96.0 (ac68faa20 2026-05-25)`, host `aarch64-apple-darwin`, LLVM `22.1.2`, `cargo 1.96.0 (30a34c682 2026-05-25)`
- Command: `cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios`
- Output: `target/criterion`; exit status `0`; all 20 scenarios executed the actual workload.

## Measurement conditions (common to every scenario)

All scenarios run under one Criterion configuration. Values below are point estimates from Criterion `new/estimates.json`, in nanoseconds (ns).

| Field | Value |
|---|---|
| harness | Criterion |
| warm up time | 3 seconds |
| measurement time | 10 seconds |
| sample size | 100 |
| noise threshold | 0.05 |
| confidence level | 0.95 |
| significance level | 0.05 |
| unit | nanoseconds (ns) |
| fixtures | deterministic; packet bytes allocated before the timed closure (except the evidence-JSON scenario) |

## Scenario definitions (what each scenario measures)

Each row is the workload exercised inside the timed closure. "reference" scenarios run the reference SFU / TURN / Signaling implementation on the frozen Kernel contract; "product" scenarios run the product-side decision functions.

| Scenario id | Name | Layer | Plane | Workload (what is measured) |
|---|---|---|---|---|
| `BENCH-001` | 598.709 | 598.089 | 599.536 | 6.898 | 619.403 |
| `BENCH-002` | 23853.043 | 23842.407 | 23823.209 | 305.666 | 24770.041 |
| `BENCH-003` | 1790.826 | 1788.390 | 1789.756 | 22.216 | 1857.475 |
| `BENCH-004` | 734.091 | 733.996 | 733.422 | 9.194 | 761.672 |
| `BENCH-005` | 813.510 | 814.439 | 814.392 | 9.392 | 841.684 |
| `BENCH-006` | 14737.839 | 14729.640 | 14747.907 | 173.837 | 15259.352 |
| `BENCH-007` | 30303.729 | 30300.459 | 30350.075 | 376.321 | 31432.692 |
| `BENCH-008` | 1726.317 | 1727.356 | 1727.509 | 18.629 | 1782.203 |
| `BENCH-009` | 377.132 | 376.955 | 377.401 | 3.478 | 387.567 |
| `BENCH-010` | 374.942 | 374.824 | 377.572 | 5.224 | 390.613 |
| `BENCH-011` | 24271.012 | 24254.305 | 24263.718 | 246.667 | 25011.012 |
| `BENCH-012` | 5330.443 | 5316.376 | 5318.359 | 65.489 | 5526.910 |
| `BENCH-013` | 12525.378 | 12541.052 | 12588.204 | 145.858 | 12962.951 |
| `BENCH-014` | 41820.773 | 41795.509 | 41737.878 | 338.866 | 42837.373 |
| `BENCH-015` | 191.095 | 190.957 | 190.824 | 1.700 | 196.195 |
| `BENCH-016` | 2004.430 | 2002.410 | 2000.652 | 21.451 | 2068.782 |
| `BENCH-017` | 292.081 | 292.007 | 291.813 | 3.256 | 301.848 |
| `BENCH-018` | 70.292 | 70.225 | 70.434 | 0.792 | 72.667 |
| `BENCH-019` | 72.524 | 72.395 | 72.410 | 0.707 | 74.644 |
| `BENCH-020` | 104.085 | 104.046 | 103.990 | 1.172 | 107.600 |

## Measured values (Criterion point estimates, ns)

| Scenario id | Mean | Median | Slope | Std dev | Derived threshold (Mean + 3×Std dev) |
|---|---:|---:|---:|---:|---:|
| `BENCH-001` | 598.709 | 598.089 | 599.536 | 6.898 | 619.403 |
| `BENCH-002` | 23853.043 | 23842.407 | 23823.209 | 305.666 | 24770.041 |
| `BENCH-003` | 1790.826 | 1788.390 | 1789.756 | 22.216 | 1857.475 |
| `BENCH-004` | 734.091 | 733.996 | 733.422 | 9.194 | 761.672 |
| `BENCH-005` | 813.510 | 814.439 | 814.392 | 9.392 | 841.684 |
| `BENCH-006` | 14737.839 | 14729.640 | 14747.907 | 173.837 | 15259.352 |
| `BENCH-007` | 30303.729 | 30300.459 | 30350.075 | 376.321 | 31432.692 |
| `BENCH-008` | 1726.317 | 1727.356 | 1727.509 | 18.629 | 1782.203 |
| `BENCH-009` | 377.132 | 376.955 | 377.401 | 3.478 | 387.567 |
| `BENCH-010` | 374.942 | 374.824 | 377.572 | 5.224 | 390.613 |
| `BENCH-011` | 24271.012 | 24254.305 | 24263.718 | 246.667 | 25011.012 |
| `BENCH-012` | 5330.443 | 5316.376 | 5318.359 | 65.489 | 5526.910 |
| `BENCH-013` | 12525.378 | 12541.052 | 12588.204 | 145.858 | 12962.951 |
| `BENCH-014` | 41820.773 | 41795.509 | 41737.878 | 338.866 | 42837.373 |
| `BENCH-015` | 191.095 | 190.957 | 190.824 | 1.700 | 196.195 |
| `BENCH-016` | 2004.430 | 2002.410 | 2000.652 | 21.451 | 2068.782 |
| `BENCH-017` | 292.081 | 292.007 | 291.813 | 3.256 | 301.848 |
| `BENCH-018` | 70.292 | 70.225 | 70.434 | 0.792 | 72.667 |
| `BENCH-019` | 72.524 | 72.395 | 72.410 | 0.707 | 74.644 |
| `BENCH-020` | 104.085 | 104.046 | 103.990 | 1.172 | 107.600 |

## Notes

- Cross-reference each value to its scenario definition above by `Scenario id` (e.g., `BENCH-003` = a joined pair exchanging offer / answer / 4 ICE candidates).
- The "Derived threshold" column is `Mean + 3 × Std dev`, computed from this same run, shown for reference only. Adopting it as a pass/fail threshold is a separate decision that MUST be fixed before the verdict measurement (see the acceptance-threshold rule in the benchmark chapter).
- Supplemental checks in the same phase: `cargo fmt --check` (no diff), `cargo test` (`23 passed`), `cargo clippy --all-targets -- -D warnings` (no warnings) — all exit `0`.
