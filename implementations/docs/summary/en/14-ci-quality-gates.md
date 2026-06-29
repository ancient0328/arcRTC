# Chapter 14 ci-quality-gates

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes the complete specification of the command CI quality gate in the arcRTC v0.2 implementations domain. Specifically, it describes, at a granularity sufficient for reproducible implementation: the command root (working directory); the closed set of command classes (format / build / test / benchmark / real-device / readiness) with their command shapes and claim boundaries; the required fields of command evidence; the mandatory documentation items for CI admission; the current gate state (the status and claim boundary of each gate); the inviolable rule of per-command-class claim boundaries; the adoptable gate scope; and the fail-closed / collapse conditions. This chapter is fully self-contained and is understandable without consulting other documents or source code. It references only other chapter numbers within this same specification.

As a dependency rule, implementations depends on the Kernel only through contract / SDK / command surface. Command success MUST NOT exceed the claim boundary of the command class. CI success MUST NOT auto-claim production readiness / live readiness. A close-like claim MUST NOT be made on the basis of CI success alone.

## 1. Command Root (working directory)

All implementations commands use the following working directory as their root.

```text
implementations
```

Command success at the Kernel workspace root MUST NOT be used as the basis for implementations readiness.

## 2. Command Classes (closed set)

| Class | Command shape | Claim boundary |
|---|---|---|
| format | `cargo fmt --check` | formatting only |
| build | `cargo build --workspace --all-targets` | build success only |
| test | `cargo test --workspace --all-targets` | tested behavior only |
| benchmark | `cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios` | measurement output only |
| real-device | bounded device command | command result only |
| readiness | readiness-specific command matrix | readiness claim only with an admitted readiness gate |

The owner and claim boundary per command class are as follows.

| Command class | Owner | Claim boundary |
|---|---|---|
| build | implementation source | limited to build success |
| unit / integration test | test source | limited to tested behavior |
| benchmark | benchmark harness | limited to measurement output |
| real-device | real-device harness | limited to command scope |
| readiness | product / operations readiness | requires an admitted readiness gate |

Command success MUST NOT exceed the claim boundary of the command class. Build success does not mean test pass. Test pass does not mean benchmark threshold satisfaction. Benchmark execution does not mean production readiness. Real-device command success does not mean live readiness.

## 3. Command Evidence Required Fields

Formal command evidence has the following common fields.

- correlation id
- command
- working directory
- target scope
- command class
- toolchain / runtime version
- expected outcome
- actual outcome
- environment class
- implementation reason
- non-claim scope
- rerun condition

The per-command-class requiredness of `target_package`, `input_fixture_or_workload`, and `exit_status` differs by command class and therefore follows the command-class required field matrix (evidence schema). A command output that lacks a field required by that matrix MUST NOT be adopted as formal evidence. The details of the closed enum wire values / non-claim scope closed set and the benchmark / real-device / readiness extension records follow the evidence-related chapters of this specification.

## 4. CI Admission

When adopting a CI workflow, the following MUST be documented first.

- workflow file path
- trigger
- command class
- target scope
- command-class target package requirement
- artifact path (required artifact)
- failure classification
- rerun condition
- claim boundary

CI success MUST NOT auto-claim production readiness / live readiness. CI success MUST NOT be repurposed as production readiness or live readiness.

## 5. Adoptable Gate Scope

The implementation source and its governing specification now exist, so the format / build / test command classes are adoptable. The benchmark / real-device / readiness commands are adopted after their relevant source and governing specification exist.

## 6. Current Gate State (current quality gate state)

This chapter fixes the implementations-side CI / command matrix. Now that the implementation source and its governing specification exist, the format / build / test gates are adoptable. The benchmark / real-device / readiness gates are adopted after the relevant source and its governing specification are established.

The current gate state is as follows.

| Gate | Current status | Claim boundary |
|---|---|---|
| format | adoptable | formatting only |
| build | adoptable | build success only |
| test | adoptable now that the test source is established | tested behavior only |
| benchmark | adopted under the benchmark scenario workload rule and the benchmark harness layout rule | measurement output only |
| real-device | adopted under the real-device command matrix rule and the real-device wrapper command rule | bounded command result only |
| readiness | requires the readiness matrix defined in Chapter 12 and an admitted readiness gate | bounded readiness only |

No command gate MUST be used as the basis for reference / product implementation completion, production readiness, or live readiness beyond its own claim boundary.

## 7. Non-Claim

The CI quality gate documents MUST NOT claim CI pass, build pass, test pass, benchmark pass, production readiness, live readiness, or CI workflow completion.

## 8. Collapse Conditions (invariants and fail-closed conditions)

The authority of this chapter collapses if any of the following occur.

- a command result from the Kernel working directory is adopted as implementations evidence.
- a success claim is repurposed across command classes (command success is repurposed outside the claim boundary of the command class).
- a command output lacking a required field is made formal evidence.
- benchmark execution is treated as threshold satisfaction.
- benchmark command output is treated as threshold satisfaction.
- real-device command success is treated as live readiness.
- command success at the Kernel working directory is treated as implementations readiness.
