# CI Quality Gate and Command Matrix

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter specifies the CI / quality gate boundary of arcRTC v0.2 Kernel and the CI / testing command matrix. CI is a guardrail; CI output by itself is not a sufficient correctness or readiness claim. This chapter fixes which gate carries which evidence class, the pass/fail rules of each gate, and the per-language (Rust / Android / Swift / TypeScript) build / test / coverage / lint command strings.

Normative terms used here mean the following. **MUST** = a condition that has to be satisfied; **MUST NOT** = a prohibited action; **MAY** = an allowed action; **fail-closed** = when a MUST condition cannot be satisfied, is unknown, or times out, fall to the failing side. `UNKNOWN` is not an allowed success state.

## Position of the CI Quality Gate

CI output by itself is not sufficient evidence. To be adopted as evidence, a report MUST record the required fields below. A CI log without those fields is treated as diagnostic output only.

## Gate Classes

v0.2 CI gate classes are limited to the following.

| Gate class | Purpose |
|---|---|
| docs structure gate | documentation required files, naming, link presence |
| architecture dependency gate | core/drivers/entrypoints/sdk/regulated dependency direction |
| source format/lint gate | language-specific format and lint |
| unit test gate | core domain and driver-local unit tests |
| contract test gate | port, driver conversion, SDK public contract |
| integration test gate | entrypoints composition and selected driver integration |
| benchmark gate | performance measurement only when benchmark scope is defined |
| code coverage gate | core line coverage, overall line coverage, and core crate floor threshold judgement |
| supply-chain gate | dependency, license, vulnerability, lockfile, toolchain policy |
| public endpoint lifecycle gate | endpoint class, protocol, target contract, and lifecycle policy |
| export/backup artifact gate | artifact class, redaction, retention, and integrity |
| release artifact provenance gate | source ref, build command, digest/signature, and distribution channel |
| time synchronization gate | time trust class, node scope, skew policy, precision, and window |
| edge/proxy trust gate | edge class, trusted metadata class, trusted upstream scope, header precedence, and TLS termination relation |
| runtime reconfiguration gate | reconfiguration class, generation references, apply scope, drain/restart, and rollback class |
| packet rewrite/media transform gate | rewrite/transform class, copy allowance, execution owner, transform admission, and copy bound |
| service discovery/endpoint resolution gate | discovery source, endpoint scope, resolution state, staleness, fallback, and contract reference |
| distributed state/failover gate | distributed state class, state family, owner scope, affinity, conflict rule, and failover admission |
| runtime task/worker lifecycle gate | task class, supervision scope, owning layer, join/cancel bound, and task outcome |
| internal service identity/trust gate | trust class, source/target service, credential/peer proof class, scope, and trust policy reference |
| cross-plane identity/session binding gate | binding class, source/target plane, references, lifecycle state, and authorization relation |

## Gate Asset Classification

The gate classes are classified as:

- source-shape gate
- build gate
- test gate
- benchmark reportability gate

Forbidden substitution:

- build must not substitute runtime
- benchmark must not substitute correctness
- skipped command must not substitute pass

Every gate failure or skip MUST include a closed reason classification and a rerun condition.

## Evidence Rule (required record fields)

To adopt CI output as evidence, a report MUST record at least:

- correlation ID
- command
- working directory
- toolchain / runtime version when relevant
- input scope
- result
- failure reason classification
- re-run condition
- denominator scope, threshold judgement, and core crate floor when the code coverage gate is executed

CI logs without these fields are diagnostic output only.

## Fail-Closed Rule

If a required gate is configured for the target scope and cannot run, times out, or reports unknown state, the target scope MUST be treated as failing. `UNKNOWN` is not an allowed success state.

The code coverage gate MUST fail closed when:

- core line coverage is below `90.00%`.
- overall line coverage is below `85.00%`.
- any core crate line coverage is below `80.00%`.
- denominator scope is missing.
- diagnostic-only coverage is adopted as assertion-bearing coverage.

## Dependency Boundary Gate

The dependency boundary gate MUST check at least:

- core does not depend on drivers/entrypoints/framework/runtime/DB/cloud SDK/browser/native concrete APIs
- drivers do not depend on entrypoints
- entrypoints do not define port traits
- sdk does not depend on regulated
- regulated does not depend on drivers/entrypoints/sdk
- feature flags do not invert dependency direction

## Tooling Rule

JavaScript / TypeScript commands MUST use `pnpm`. npm and yarn are not accepted for v0.2 CI evidence.

## CI Quality Gate Prohibitions

- failed or skipped CI is described as successful.
- v0.1 CI/test/benchmark output is reused as v0.2 evidence without requalification.
- source-only static gate is used to prove runtime behavior.
- runtime smoke is used to prove architecture dependency compliance.
- dependency install success is used to bypass license/vulnerability/toolchain policy.
- endpoint, artifact, release, or time gate success is claimed without its class-specific evidence fields.
- edge/proxy trust or runtime reconfiguration gate success is claimed without its class-specific evidence fields.
- packet rewrite/media transform, service discovery, or distributed state/failover gate success is claimed without its class-specific evidence fields.
- runtime task/worker lifecycle or internal service identity/trust gate success is claimed without its class-specific evidence fields.
- cross-plane identity/session binding gate success is claimed without its class-specific evidence fields.
- code coverage pass is used as correctness, production readiness, live readiness, or native command success proof.

## CI Quality Gate Collapse Conditions

This judgement collapses if:

- required gate can fail open.
- CI output is adopted without correlation ID and reproducible command.
- evidence class is mixed across source/build/runtime/live.
- npm/yarn is used for JS/TS v0.2 evidence.
- supply-chain gate failure is treated as release/readiness success.
- public endpoint, export/backup artifact, release artifact, or time synchronization gate fails open.
- edge/proxy trust or runtime reconfiguration gate fails open.
- packet rewrite/media transform, service discovery, or distributed state/failover gate fails open.
- runtime task/worker lifecycle or internal service identity/trust gate fails open.
- cross-plane identity/session binding gate fails open.
- code coverage gate fails open or treats import-only, smoke-only, text-inspection-only, or generated-output-only coverage as adopted coverage.

## Position of the Command Matrix

The command matrix is the authority for command slots. `Kernel` has a Rust workspace, TypeScript SDK test, and benchmark smoke command surface. Android / iOS SDK have a source-level public contract surface and native command evidence. Native toolchain command success is adopted only after an evidence report records the command, working directory, toolchain condition, and observed result.

## Command Surface

| Gate | Command surface |
|---|---|
| Rust format/lint | `cargo fmt --all -- --check` from `Kernel` |
| Rust unit / contract / integration tests | `cargo test --workspace --all-targets` from `Kernel` |
| SDK TypeScript tests | `pnpm test` from `Kernel/sdk/typescript` |
| SDK Android unit/build/lint command | `cd sdk/android && ANDROID_HOME="$ANDROID_HOME" ANDROID_SDK_ROOT="$ANDROID_HOME" ./gradlew :sdk:testDebugUnitTest :sdk:assembleDebug :sdk:lintDebug --no-daemon --warning-mode all` plus source-level contract tests in the Rust test suite |
| SDK iOS tests | `swift test` plus source-level contract tests in the Rust test suite |
| benchmark smoke and scenarios | `cargo run -p arcrtc-roadmap-tests --bin benchmark_smoke` and `cargo run -p arcrtc-roadmap-tests --bin benchmark_scenarios` from `Kernel` |
| benchmark Criterion observation | `cargo bench -p arcrtc-roadmap-tests --bench benchmark_scenarios_criterion -- --noplot` from `Kernel`; Criterion-only flags must target a concrete Criterion bench |
| v0.1.2 ice benchmark comparison observation | `cargo bench -p arcrtc-roadmap-tests --bench v01_ice_shape_criterion -- --noplot` from `Kernel`; this does not claim production performance acceptance |
| code coverage | Rust `cargo llvm-cov`, TypeScript Node/V8 coverage, Swift `swift test --enable-code-coverage`, Android `jacocoDebugUnitTestReport`, aggregate summary `coverage-aggregation-summary.json` |

## Command Classes and Working Directories

The command classes at the command matrix asset level are limited to:

- `cargo-test-workspace`
- `cargo-test-roadmap`
- `cargo-test-integration`
- `pnpm-test-typescript-sdk`
- `cargo-run-benchmark-smoke`

Working directories are limited to:

- `Kernel`
- `Kernel/sdk/typescript`

Unavailable command targets MUST use a closed reason classification and MUST NOT be adopted as pass.

## Required Gate Matrix

| Gate | Target | Command owner | Evidence class | Close condition |
|---|---|---|---|---|
| docs structure | documentation | docs/CI | source-shape | required docs and links present |
| architecture dependency | `Kernel` packages | CI | source-shape | forbidden dependency/import absent |
| Rust format/lint | Rust packages | CI | source-shape/build-tool | command recorded |
| Rust unit tests | core/drivers/entrypoints Rust packages | CI | test evidence | command recorded |
| SDK TypeScript tests | `sdk/typescript` | CI | test evidence | `pnpm` command recorded |
| SDK Android unit/build/lint command | `sdk/android` | CI | native test/build/lint evidence | Gradle wrapper command recorded with native command report |
| SDK iOS tests | `sdk/ios` | CI | test evidence | xcodebuild/swift command recorded |
| SDK public API projection tests | SDK packages | CI | public contract evidence | source contract projection and golden matrix recorded |
| internal control-plane contract tests | entrypoints/drivers/core contract packages | CI | contract/runtime-in-test evidence | control-plane class, version, correlation, authorization, and topology recorded |
| ICE candidate/connectivity contract tests | transport/driver packages | CI | driver contract/runtime-in-test evidence | candidate/connectivity class and exposure policy recorded |
| secure media session contract tests | transport/security driver packages | CI | driver contract/runtime-in-test evidence | secure media session class and protection evidence recorded |
| operator/admin authorization tests | entrypoints/cli/admin packages | CI | contract/runtime-in-test evidence | operator/admin class and target action scope recorded |
| out-of-scope feature rejection tests | SDK/entrypoints/signaling packages | CI | contract evidence | excluded feature class and cataloged rejection recorded |
| public endpoint/connection lifecycle tests | entrypoints/drivers/network packages | CI | contract/runtime-in-test evidence | endpoint class, protocol class, lifecycle state, and target contract recorded |
| export/backup artifact tests | persistence/audit/tooling packages | CI | source-shape/runtime-in-test evidence | artifact class, redaction, retention, and integrity class recorded |
| release artifact/provenance tests | package/build/release tooling | CI | source-shape/build evidence | artifact class, source ref, command, digest, and distribution channel recorded |
| time synchronization/clock skew tests | runtime/topology/test support packages | CI | runtime-in-test/source-shape evidence | time trust class, node scope, skew policy, precision, and window recorded |
| edge/proxy trust tests | entrypoints/drivers/network/topology packages | CI | contract/runtime-in-test evidence | edge class, trusted metadata class, trusted upstream scope, and header precedence recorded |
| runtime reconfiguration tests | entrypoints/config/core policy packages | CI | contract/runtime-in-test evidence | reconfiguration class, generation references, apply scope, drain/restart, and rollback class recorded |
| packet rewrite/media transform tests | core/sfu and drivers/media/network packages | CI | contract/runtime-in-test evidence | rewrite/transform class, copy allowance, execution owner, and transform admission recorded |
| service discovery/endpoint resolution tests | entrypoints/drivers/network/topology packages | CI | contract/runtime-in-test evidence | discovery source, endpoint scope, resolution state, staleness, and fallback class recorded |
| distributed state/failover tests | topology/recovery/entrypoint composition packages | CI | contract/runtime-in-test evidence | distributed state class, owner scope, affinity, conflict rule, and failover admission recorded |
| runtime task/worker lifecycle tests | entrypoints/drivers/runtime packages | CI | contract/runtime-in-test evidence | task class, supervision scope, owning layer, join/cancel bound, and task outcome recorded |
| internal service identity/trust tests | entrypoints/drivers/security/topology packages | CI | contract/runtime-in-test evidence | trust class, source/target service, credential/peer proof class, accepted scope, and trust policy recorded |
| cross-plane identity/session binding tests | core/signaling/sfu/turn/transport packages | CI | contract/runtime-in-test evidence | binding class, source/target plane, references, lifecycle state, and authorization relation recorded |
| fake driver contract tests | core port fake/test support packages | CI | test evidence | fake driver class and port contract recorded |
| supply-chain gate | all manifests/lockfiles/toolchains | CI | source-shape/evidence | dependency, license, vulnerability, lockfile, and toolchain policy recorded |
| integration tests | selected entrypoint/driver composition | CI | runtime-in-test | controlled environment report required |
| benchmark smoke | benchmark packages | CI | benchmark diagnostic unless adopted | benchmark report required for evidence |
| code coverage | production source under Rust core/drivers/entrypoints/regulated and SDK TypeScript/Swift/Android | CI | code coverage quality evidence | core line coverage `>= 90.00%`, overall line coverage `>= 85.00%`, each core crate `>= 80.00%`, denominator scope locked, aggregate summary `coverage-aggregation-summary.json` |

## Command Adoption Rule (required fields at adoption)

A command becomes adopted only after the package/workspace it targets exists and an evidence report records:

- command
- working directory
- target package/module
- dependency/license/toolchain class when supply-chain gate is executed
- SDK projection class when SDK public API gate is executed
- internal control-plane class when service-to-service gate is executed
- ICE candidate/connectivity class when ICE gate is executed
- secure media session class when secure media gate is executed
- operator/admin authorization class when privileged action gate is executed
- out-of-scope feature class when feature exclusion gate is executed
- public endpoint/connection lifecycle class when endpoint gate is executed
- export/backup artifact class, redaction, retention, and integrity class when artifact gate is executed
- release artifact class, provenance class, digest/signature class, and distribution channel when release gate is executed
- time synchronization trust class, node scope, skew policy, precision, and measurement window when time gate is executed
- edge/proxy class, trusted metadata class, trusted upstream scope, and header precedence when edge trust gate is executed
- runtime reconfiguration class, target surface, generation references, apply scope, and rollback/drain class when reconfiguration gate is executed
- packet rewrite/media transform class, copy allowance, execution owner, and transform admission status when rewrite/transform gate is executed
- service discovery source, endpoint scope, resolution state, TTL/cache/staleness, and fallback class when endpoint resolution gate is executed
- distributed state class, state family, owner node/scope, affinity key, replication/consensus admission, and failover class when distributed state gate is executed
- runtime task/worker class, supervision scope, owning layer, join/cancel bound, and task outcome when task lifecycle gate is executed
- internal service identity/trust class, source/target service, credential/peer proof class, accepted scope, and trust policy reference when service identity gate is executed
- cross-plane binding class, source/target plane, source/target references, lifecycle state, and authorization context relation when cross-plane binding gate is executed
- denominator scope, included/excluded paths, exclusion reasons, core line coverage, overall line coverage, core crate floor table, threshold judgement, assertion-bearing classification, and diagnostic-only exclusion when the code coverage gate is executed
- toolchain/runtime version where relevant
- result
- failure reason classification
- rerun condition

## Benchmark Command Boundary

Workspace benchmark commands and Criterion bench-target commands are separate command classes.

The workspace form is:

- `cargo bench --workspace`

The Criterion target form is:

- `cargo bench -p arcrtc-roadmap-tests --bench benchmark_scenarios_criterion -- --noplot`
- `cargo bench -p arcrtc-roadmap-tests --bench v01_ice_shape_criterion -- --noplot`

Criterion-only options such as `--noplot` MUST NOT be appended to a workspace-wide `cargo bench --workspace` command. `cargo bench --workspace -- --noplot` is invalid for mixed workspace surfaces because the option is also passed to non-Criterion libtest binaries. If a workspace benchmark comparison requires plot suppression, each Criterion bench target MUST be invoked explicitly instead of forwarding Criterion-only flags to the whole workspace.

## pnpm Rule

JavaScript / TypeScript SDK and tooling commands MUST use `pnpm`. npm and yarn are not accepted for v0.2 evidence.

## Command Matrix Fail-Closed Rule

If a gate is required for the target scope and the command is missing, skipped, timed out, or cannot be associated with an evidence report, that target MUST be treated as failing.

## Command Matrix Prohibitions

- skipped gate is treated as pass.
- Criterion-only benchmark option is forwarded to mixed workspace bench command, for example `cargo bench --workspace -- --noplot`.
- JS/TS evidence uses npm or yarn.
- unit test gate is used as integration/runtime proof.
- fake driver contract test is used as concrete driver proof.
- benchmark smoke is used as correctness proof.
- CI command output is adopted without an evidence report.
- supply-chain command result is adopted without policy class.
- SDK projection command is adopted without source contract reference.
- internal control-plane command is adopted without control-plane class/version/correlation.
- ICE or secure media command is adopted as runtime readiness without evidence class.
- operator/admin command is adopted without authorization class and target scope.
- out-of-scope feature test is treated as feature admission.
- public endpoint command is adopted from listener bind without lifecycle/contract class.
- export/backup artifact command is adopted from file existence without artifact/redaction/integrity class.
- release artifact command is adopted from build output without provenance/distribution class.
- time synchronization command is adopted from timestamp output without skew/trust class.
- edge/proxy trust command is adopted from header presence or proxy route name without trust class.
- runtime reconfiguration command is adopted from startup validation without generation/apply-scope class.
- packet rewrite/media transform command is adopted from forwarding success without rewrite/transform class.
- service discovery command is adopted from DNS/registry output without resolution state and scope.
- distributed state/failover command is adopted from process startup or health success without owner/conflict/failover evidence.
- runtime task command is adopted from spawn success or logs without supervision/join/cancel evidence.
- internal service trust command is adopted from endpoint resolution, TLS listener startup, or mesh route name without trust evidence.
- cross-plane binding command is adopted from same correlation, token subject, Signaling join, TURN credential, ICE relay, or secure media observation without binding evidence.
- code coverage command is adopted without denominator scope, threshold judgement, core crate floor, and assertion-bearing classification.
- code coverage pass is used as correctness, production readiness, live readiness, or native command success proof.

## Command Matrix Collapse Conditions

This judgement collapses if:

- command matrix does not identify evidence class.
- command is adopted before target package exists.
- workspace benchmark command and Criterion bench-target command are collapsed into one command class.
- skipped/timed-out command passes as success.
- package scaffold changes without matrix update.
- `pnpm` rule is bypassed for JS/TS evidence.
- supply-chain or SDK projection gate hides its evidence class.
- internal control-plane, ICE candidate/connectivity, secure media, operator/admin, or out-of-scope feature gate hides its evidence class.
- public endpoint, export/backup artifact, release artifact, or time synchronization gate hides its evidence class.
- edge/proxy trust or runtime reconfiguration gate hides its evidence class.
- packet rewrite/media transform, service discovery, or distributed state/failover gate hides its evidence class.
- runtime task/worker lifecycle or internal service identity/trust gate hides its evidence class.
- cross-plane identity/session binding gate hides its evidence class.
- code coverage gate hides denominator scope, threshold judgement, core crate floor, or diagnostic-only exclusion.
