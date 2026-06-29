# Testing and Evidence System

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter specifies the testing / evidence boundary of arcRTC v0.2 Kernel. It covers test tiers, the source-shape evidence rule, the code coverage thresholds and denominator scope definition, the fields of an evidence record, the test double / fake driver boundary, the test fixture / scenario data boundary, and the Native SDK command evidence rule. Tests treat design, implementation, and runtime evidence separately and MUST carry a correlation ID, closed-set reasons, and a reproduction procedure.

Normative terms used here mean: **MUST** = a condition that has to be satisfied; **MUST NOT** = a prohibited action; **MAY** = an allowed action; **fail-closed** = when a MUST condition cannot be satisfied or is unknown, fall to the failing side. `UNKNOWN`, free-text-only reason, unclassified skip, and diagnostic-only output MUST NOT count toward adoption. Passing tests do not by themselves establish readiness; readiness is a separate claim with its own evidence.

## Test Tiers

v0.2 testing tiers are limited to the following.

| Tier | Scope | Evidence class |
|---|---|---|
| architecture static test | dependency direction, forbidden imports, package boundary | source-shape evidence |
| core unit test | domain model, state machine, reason catalog, use case | source/runtime-in-test evidence |
| driver contract test | conversion, port implementation, resource bounds | driver behavior evidence |
| entrypoint composition test | startup wiring, typed config, fail-closed startup | composition evidence |
| SDK contract test | Signaling-only public client behavior | public contract evidence |
| regulated optional test | regulated boundary and enrichment only | optional support evidence |
| integration test | selected end-to-end path under controlled environment | runtime-in-test evidence |
| code coverage gate | production source line coverage for core and overall source set | code coverage quality evidence |

## Source-Shape Evidence Rule

Source-shape evidence MUST inspect the effective source set for the target package/module. When a Rust crate root uses source shards, tests MUST scan all `.rs` files under the relevant `src` surface, excluding generated/build output. When an SDK target is split across multiple platform source files, tests MUST scan the target source set for that platform.

Core source-shape evidence MUST test the semantic modular monolith boundary. It MUST reject forbidden driver/entrypoint/framework/runtime/socket/DB/browser/native/cloud/concrete transport imports in core, hidden semantic owner transfer, source shard scope drift, and source-set scan gaps.

For auditability, 600 lines is a review signal for source comprehension. Tests MAY record or inspect file size but MUST NOT fail solely because a source file exceeds 600 lines. Line count does not prove or disprove runtime behavior, production readiness, live readiness, or benchmark acceptance by itself.

## Code Coverage Rule (thresholds and denominator)

Code coverage evidence is a quality gate for production source execution reachability. It is not a standalone readiness / live / native command success proof. The internal label `enterprise-grade` denotes only the adopted line coverage threshold set; it MUST NOT be read as enterprise-ready, production-ready, live-ready, or native-runtime-ready. For external or abbreviated expression, this gate MAY be described as a `multi-language code coverage quality gate`.

### Definitions

| Term | Definition |
|---|---|
| core line coverage | line coverage of Rust production source under `Kernel/core/` |
| overall line coverage | weighted aggregate of covered / instrumented production lines reported by each language/tool for Rust `core/`, `drivers/`, `entrypoints/`, `regulated/` and SDK TypeScript / Swift / Android production source |
| project-level coverage indicator | the nature of `overall line coverage`; a project-level reachability indicator that does not claim each language/tool's coverable-line definition is semantically identical |
| core threshold | core line coverage `>= 90.00%` |
| overall threshold | overall line coverage `>= 85.00%` |
| core crate floor | each core crate line coverage `>= 80.00%` |
| assertion-bearing coverage | coverage reached by tests asserting production behavior / boundary / reject reason / state transition / value semantics |
| diagnostic-only coverage | import-only, smoke-only, text-inspection-only, generated-output-only, or command reachability coverage |

### Adopted Thresholds

| Target | Threshold |
|---|---:|
| core line coverage | `>= 90.00%` |
| overall line coverage | `>= 85.00%` |
| each core crate line coverage floor | `>= 80.00%` |

A pass of this gate does not by itself prove production readiness, live readiness, or native command success. This threshold set MUST NOT be treated as retrofitted to post-measurement numbers. If the threshold policy is changed, the before/after rule MUST be recorded before a measurement rerun.

### Current Verified Status

Last verified: 2026-06-29 (JST). This is the latest measured status against the fixed adopted thresholds; it is updated in place on each re-measurement and is reproducible by re-running the coverage commands defined in this chapter (each measurement records its correlation ID, command, and reproducible procedure). It is a code-coverage quality-gate status only and does not assert production / live / native readiness.

| Metric | Measured | Threshold | Result |
|---|---:|---:|:--|
| core line coverage | 97.50% (3548/3639) | `>= 90.00%` | PASS |
| overall line coverage (Rust + TypeScript / Swift / Android SDK) | 94.49% (10660/11282) | `>= 85.00%` | PASS |
| each core crate line coverage floor | min 86.44% (core/quality) | `>= 80.00%` | PASS (21/21) |

Per-language line coverage: Rust core 97.50%, drivers 97.55%, entrypoints 89.29%, regulated 90.91%; TypeScript SDK 91.85%; Swift SDK 97.64%; Android SDK 98.43%. Diagnostic-only entrypoint `main.rs` startup coverage (19.69%) is excluded; overall excluding it is 97.61%.

### Denominator Rule (denominator scope definition)

Included in the denominator:

- Rust production source under `core/`, `drivers/`, `entrypoints/`, `regulated/`
- TypeScript SDK production source
- Swift SDK production source
- Android / Kotlin SDK production source after coverage task configuration

Excluded from the denominator:

- Rust `tests/` and `integration-tests/` packages
- SDK test directories
- benchmark runner and benchmark support code
- fixtures
- generated build output
- `target/`, `.build/`, Gradle build output, V8 raw output

If the denominator is changed, the coverage report MUST record the delta between the changed and previous denominators. Overall line coverage is a project-level indicator, computed by aggregating covered / total lines reported by each tool as instrumented / coverable. This weighted aggregate does not prove that statement models, branch models, macro / generated code handling, and method/function instrumentation are identical across languages.

### Coverage Report Fields

A coverage report MUST record:

| Field | Required content |
|---|---|
| correlation ID | ID linking the coverage run and the report |
| command | per-language / per-target coverage command |
| working directory | command execution directory |
| toolchain | coverage tool and compiler/runtime version |
| denominator scope | included / excluded production source paths |
| exclusion reason | closed reason |
| core line coverage | covered / instrumented / percentage |
| overall line coverage | covered / instrumented / percentage |
| core crate floor table | each core crate covered / instrumented / percentage |
| threshold judgement | pass/fail per threshold |
| assertion-bearing classification | classification that coverage is assertion-bearing |
| diagnostic-only exclusion | basis for not adopting import-only / smoke-only / text-inspection-only coverage |
| rerun condition | procedure to rerun under identical conditions |

Even if overall coverage is satisfied first, it is not adopted unless core line coverage and core crate floor are also satisfied.

### Assertion-Bearing Test Rule

A coverage improvement test MUST assert at least one of:

- the normal path of accepted / admitted / allocated / approved / converted
- the abnormal path of reject / deny / violation / collapse reason
- the boundary that a driver concrete type does not enter a core signature
- the boundary that only a core-owned reference passes cross-plane
- the closed vocabulary of reason / command / lifecycle / evidence class
- the state transition of before / during / after / rollback / recovery
- inconsistency rejection of Signaling / SFU / TURN / transport binding

Coverage reached only by importing production source, only by starting a CLI or Kernel executable entrypoint, or only by inspecting source text is diagnostic-only and MUST NOT be adopted as the basis for reaching a coverage threshold.

### Non-Claim Boundary

Even if this gate passes, the following do not hold: branch coverage sufficiency, path coverage sufficiency, function / method coverage sufficiency, mutation score sufficiency, fuzzing coverage sufficiency, property-test completeness, protocol conformance completeness, security readiness, runtime readiness, production readiness, live readiness, native SDK runtime readiness, native application command success. Swift / Android command results MAY be adopted only as coverage-gate input evidence and MUST NOT be repurposed as native SDK runtime readiness, native entrypoint readiness, or general native command success claims.

### Fail-Closed Conditions

The code coverage gate is fail-closed when any of the following hold:

- core line coverage is below `90.00%`.
- overall line coverage is below `85.00%`.
- any core crate is below `80.00%`.
- denominator scope is not recorded.
- excluded paths are not classified with a closed reason.
- import-only / smoke-only / text-inspection-only coverage is adopted as assertion-bearing coverage.
- line coverage pass is treated as branch / path / function / mutation / fuzzing / property-test / protocol-conformance sufficiency.
- Swift / Android coverage-related command result is adopted as native readiness / native command success.
- command, working directory, toolchain, or rerun condition is missing.

## Evidence Record Fields (test evidence)

A test result MAY be adopted only when the record contains:

- correlation ID
- test command
- working directory
- target package/module
- source set scope when source-shape evidence is used
- input fixture or scenario
- fixture/scenario class and source
- canonical serialization format/version when golden or digest evidence is used
- SDK projection class when SDK public API contract is tested
- media negotiation class when codec/track/layer behavior is tested
- internal control-plane class when service-to-service behavior is tested
- ICE candidate/connectivity class when candidate policy, restart, connectivity, or consent is tested
- secure media session class when DTLS/SRTP behavior is tested
- operator/admin authorization class when privileged action is tested
- out-of-scope feature class when excluded/admitted feature behavior is tested
- public endpoint/connection lifecycle class when endpoint admission or lifecycle behavior is tested
- export/backup artifact class, redaction, retention, and integrity class when artifact behavior is tested
- release artifact class, provenance class, and distribution channel when release behavior is tested
- time synchronization trust class, node scope, skew policy, precision, and measurement window when timestamp trust is tested
- edge/proxy class, trusted metadata class, trusted upstream scope, and header precedence when ingress trust is tested
- runtime reconfiguration class, target surface, generation references, apply scope, and rollback/drain class when reconfiguration is tested
- packet rewrite/media transform class, copy allowance, execution owner, and transform admission status when rewrite/transform behavior is tested
- service discovery source, endpoint scope, resolution state, staleness, and fallback class when endpoint resolution is tested
- distributed state class, state family, owner node/scope, affinity key, conflict rule, and failover class when distributed state behavior is tested
- runtime task/worker class, supervision scope, owning layer, join/cancel bound, and task outcome when task lifecycle behavior is tested
- internal service identity/trust class, source/target service, credential/peer proof class, scope, and trust policy reference when service identity behavior is tested
- cross-plane binding class, source/target plane, source/target references, lifecycle state, and authorization context relation when binding behavior is tested
- coverage threshold judgement, denominator scope, and core crate floor table when code coverage is tested
- expected outcome
- actual outcome
- reason category/code for failure where applicable
- environment/toolchain sufficient for rerun

Test output lacking these fields is diagnostic output only.

## Native SDK Command Evidence Rule

Android native SDK command evidence is not a source marker test. It MUST execute the Android SDK Gradle wrapper command from `sdk/android` and include unit test, build, and lint tasks in the same recorded command surface:

- `:sdk:testDebugUnitTest`
- `:sdk:assembleDebug`
- `:sdk:lintDebug`
- `--warning-mode all`

The record MUST contain the exact `Command:` line, exact `Working directory:`, `BUILD SUCCESSFUL`, and `Warning/deprecation output: none observed`. The Android lint report MUST NOT contain warnings or errors. iOS native SDK command evidence MUST execute `swift test` from `sdk/ios` and MUST NOT be substituted by source marker assertions. TypeScript SDK tests MUST stay within `sdk/typescript`. Cross-platform Android/iOS projection checks MUST be owned by native tests or cross-platform governance tests, not duplicated inside the TypeScript package test. Coverage-related Swift / Android command success is not equivalent to native command success; native command success requires the dedicated command evidence fields.

## Mock / Fake Rule and Test Double Boundary

Mock, fake, deterministic clock, deterministic RNG, in-memory persistence, and simulated network are allowed for focused tests. They MUST be labeled as test doubles and MUST NOT be used as production / runtime evidence. When a fake driver is used, the test proves the core contract or composition path with that fake, not the concrete external implementation.

### Test Double Boundary

| Test double surface | Owner | Rule |
|---|---|---|
| core state/domain semantics | core | fake MUST NOT change |
| core port contract | core | fake limited to an implementation that satisfies the contract |
| fake driver behavior | testing support | deterministic execution only |
| deterministic clock/RNG | testing support | test evidence class only |
| captured observation | testing support | assertions for contract behavior |
| production driver | drivers | MUST NOT inherit fake behavior |

A fake driver is a driver implementation for tests, not an alternate domain authority.

### Test Double Classes

v0.2 initial architecture test double classes are limited to the following.

| Class | Allowed use | Prohibited use |
|---|---|---|
| `fake_port_driver` | contract tests for core port usage | production runtime |
| `stub_external_service` | deterministic external response in tests | domain policy definition |
| `spy_sink` | capture audit/metrics calls | audit meaning ownership |
| `deterministic_clock` | expiry/deadline tests | production evidence |
| `deterministic_rng` | ID/correlation reproducibility tests | security/runtime evidence |
| `fault_injection_driver` | failure mapping tests | default production behavior |

### Test Double Contract / Evidence Rules

Every fake driver MUST declare: port implemented; covered call shapes; supported success outcomes; supported failure reasons; bounded behavior; deterministic behavior source; unsupported behavior handling; evidence class where it may be used. Unsupported behavior MUST fail closed in the test with a contract violation, not silently succeed.

Test evidence using fake drivers MUST record: fake driver class; deterministic clock/RNG use; port contract covered; real driver behavior not claimed. A fake-based unit/contract test is not runtime proof. Fault injection success is not real failure recovery proof.

## Fixture / Scenario Data Boundary

An input fixture, generated scenario, captured packet metadata, benchmark data, or golden sample MUST declare whether it is synthetic, captured-redacted, generated, derived from v0.1, or manually authored. Fixture evidence MUST NOT include raw secret, raw token, raw packet payload, unredacted SDP/ICE material, regulated payload, or personal data.

### Fixture Boundary

| Data surface | Owner | Rule |
|---|---|---|
| core semantic fixture | testing support | core-owned type expectations only |
| driver wire fixture | driver test support | external encoding sample, not core API |
| golden file | testing support | canonical format/version required |
| captured packet/sample | driver/testing support | raw sensitive payload prohibited unless redacted/synthetic |
| regulated scenario data | regulated testing support | MUST NOT mix into generic core fixture |

A fixture is test input, not production evidence by itself.

### Fixture Classes

v0.2 initial architecture fixture classes are limited to the following.

| Class | Meaning | Rule |
|---|---|---|
| `synthetic_core_fixture` | hand-authored core semantic input | safe for unit/contract tests |
| `driver_wire_fixture` | external encoding sample | driver conversion tests only |
| `golden_canonical_fixture` | deterministic canonical encoding fixture | must cite canonical serialization version |
| `fault_injection_fixture` | failure scenario input | must map expected reason |
| `redacted_capture_fixture` | captured material after redaction | raw source not stored in repo |
| `regulated_fixture` | regulated support scenario | not generic core fixture |

### Redaction / Ownership Rule and Golden Fixture

A fixture MUST declare: fixture class; owner layer; source; synthetic/captured/redacted status; sensitive data handling; canonical serialization version when applicable; expected reason/outcome; allowed evidence class. Raw token, raw credential, raw key, raw RTP/RTCP/media payload, patient/user profile, or regulated payload MUST NOT be stored as generic fixture.

A golden fixture is valid only when: canonical format/version is declared; field set and ordering are deterministic; unknown field handling is explicit; redaction status is recorded; update procedure is documented; test command and expected digest/outcome are recorded. Golden fixture drift is a test/evidence issue, not automatic protocol breakage, until evaluated under the compatibility rules.

### Fixture Failure Mapping

| Failure | Required reason |
|---|---|
| fixture shape invalid | `fixture_invalid` |
| fixture requires redaction before use | `fixture_redaction_required` |
| canonical fixture mismatch | `canonical_serialization_mismatch` |
| driver wire fixture cannot decode | `external_decode_failed` |
| expected reason absent | `missing_required_wire_field` or fixture validation failure |

Evidence using fixture/scenario data MUST record: fixture class; fixture path or source reference; redaction status; expected outcome/reason; actual outcome/reason; update policy. Captured data without a redaction statement MUST NOT be adopted as evidence.

## Negative Test Rule

For each boundary, negative cases should be included where feasible. Examples: external concrete type cannot enter core signature; driver conversion failure does not call core; unsupported version fails closed; missing correlation ID fails closed; resource bound maps to closed reason; startup config failure is not treated as successful wiring; invalid fixture/scenario data is rejected before being accepted as evidence; canonical serialization mismatch is rejected as evidence; SDK public API projection drift is detected; media negotiation rejects unsupported mapping; internal control-plane rejects missing version/correlation/authorization; ICE candidate policy rejects disallowed exposure or restart; secure media session rejects missing peer verification; operator/admin action rejects missing authorization; out-of-scope feature request is rejected with cataloged reason; public endpoint rejects unadmitted class; export artifact rejects missing redaction; release artifact rejects missing provenance; time synchronization rejects skew exceeding policy; edge/proxy trust rejects unadmitted forwarded headers; runtime reconfiguration rejects unadmitted hot-swap; packet rewrite/media transform rejects unadmitted transform; service discovery rejects stale endpoint; distributed state/failover rejects split-brain risk; runtime task lifecycle rejects detached task; internal service trust rejects untrusted peer proof; cross-plane binding rejects implicit/missing binding.

## v0.1 Evidence Rule

v0.1 tests, integration tests, and benches are evidence-only historical input. They do not prove v0.2 behavior until rerun or requalified under v0.2 commands, boundaries, and reports.

## Evidence Record Template (all fields)

An evidence record MUST include all of the following fields.

- title
- report date/time in JST
- correlation ID
- target scope
- target classification
- time phase
- command or procedure
- working directory
- input files/packages/modules
- environment/toolchain when relevant
- expected outcome
- actual outcome
- closed reason classification for failure or skip
- evidence class
- sensitive data handling / redaction statement when source material can contain secrets, packet payload, SDP/ICE material, regulated payload, or personal data
- profile class when configuration profile affects the run
- test double class when fake/stub/mock/deterministic support affects the run
- fixture/scenario class when fixture or generated scenario data affects the run
- canonical serialization format/version when digest, hash-chain, golden, or compatibility evidence is used
- normalized unit, window, precision, and aggregation when measurement evidence is used
- atomicity class and commit boundary when partial success or compensation can affect the claim
- health/readiness/liveness/admin/maintenance class when operational probe or admin evidence is used
- process lifecycle class when crash, panic, unclean shutdown, or supervisor restart affects the claim
- SDK reconnect/session resumption class when client reconnect evidence is used
- deployment topology class and node scope when topology affects the run
- media negotiation class when codec/track/layer behavior affects the claim
- observability signal class, cardinality, and sampling when telemetry affects the claim
- supply-chain dependency class, license class, lockfile, vulnerability, and toolchain state when dependency evidence is used
- SDK public API projection class when SDK contract evidence is used
- secret rotation state when rotated credential or key material affects the claim
- internal control-plane class when service-to-service control affects the claim
- ICE candidate/connectivity class when candidate policy, restart, connectivity, or consent affects the claim
- secure media session class when DTLS/SRTP protected media path affects the claim
- operator/admin authorization class when privileged action affects the claim
- out-of-scope feature class when excluded or future-admitted feature behavior affects the claim
- public endpoint/connection lifecycle class when endpoint exposure or connection state affects the claim
- export/backup artifact class, redaction class, retention class, and integrity class when artifact evidence is used
- release artifact class, provenance class, digest/signature class, and distribution channel when release/distribution evidence is used
- time synchronization trust class, node scope, skew policy, precision, and measurement window when timestamp trust affects the claim
- edge/proxy class, trusted metadata class, trusted upstream scope, header precedence, and TLS termination relation when ingress trust affects the claim
- runtime reconfiguration class, target surface, current/proposed generation, apply scope, drain/restart class, and rollback class when runtime configuration change affects the claim
- packet rewrite/media transform class, copy allowance, execution owner, and transform admission status when packet byte mutation affects the claim
- service discovery source, endpoint scope, resolution state, TTL/cache/staleness, and fallback class when endpoint resolution affects the claim
- distributed state class, state family, owner node/scope, affinity key, replication/consensus admission, and failover class when multi-node or failover behavior affects the claim
- runtime task/worker class, supervision scope, owning layer, join/cancel bound, and task outcome when worker execution affects the claim
- internal service identity/trust class, source/target service, credential/peer proof class, accepted scope, and trust policy reference when service-to-service identity affects the claim
- cross-plane binding class, source/target plane, source/target references, lifecycle state, and authorization context relation when a claim crosses Signaling/SFU/TURN/ICE/secure media planes
- rerun condition

### Evidence Classes

The evidence class MUST NOT be omitted.

| Evidence class | Meaning |
|---|---|
| source-shape | file structure, dependency direction, static scan |
| build | build command result only |
| test | unit/contract/integration test result |
| runtime-in-test | controlled runtime test result |
| benchmark | measured performance/capacity result |
| live/operational | real environment observation |

### Skip / Failure Reason Rule

Skipped / failed / timed-out / unavailable evidence MUST use a closed reason classification. `UNKNOWN` is not an allowed report reason.

| Reason class | Meaning |
|---|---|
| `not_in_scope` | outside current target |
| `not_yet_scaffolded` | package/command target does not exist yet |
| `dependency_missing` | required local dependency/tool unavailable |
| `command_failed` | command ran and failed |
| `command_timed_out` | command exceeded bounded time |
| `environment_unavailable` | required runtime/live environment unavailable |
| `evidence_incomplete` | output lacks required fields |
| `superseded_by_newer_report` | replaced by newer report |

If build/test/runtime is not executed, the evidence record MUST state it. Unverified evidence MUST NOT be converted into a verified claim.

## Readiness Claim Boundary

Code coverage, benchmark value, CI pass, and an implementation checklist are not readiness proof by themselves. Production readiness and live readiness are separate claim kinds, each requiring its own field-complete evidence:

- `production_readiness` requires field-complete release provenance evidence, field-complete supply-chain gate evidence, field-complete production profile evidence, and field-complete operational probe evidence.
- `live_readiness` requires all production-readiness evidence plus field-complete live endpoint evidence, field-complete public network traversal evidence, and field-complete monitoring / rollback / shutdown-drain evidence.
- `native_android_command_success` requires a rerunnable Android native command report with an observed success marker; source-level SDK contract evidence or CI pass MUST NOT be substituted.
- `native_ios_command_success` requires a rerunnable iOS native command report with an observed success marker; source-level SDK contract evidence or CI pass MUST NOT be substituted.

Fields are adopted as a set; if even one is missing, the readiness claim is fail-closed. The release provenance report and the supply-chain report MUST be connected to the same claim evidence; if any of package/module, lockfile, toolchain, or correlation does not match, the production readiness claim is fail-closed. A native command success report MUST NOT be adopted from a success marker alone; if correlation ID, command, working directory, evidence class, closed reason none, rerun condition, and source-level replacement marker absence cannot be confirmed, the claim is fail-closed.

## Prohibitions (chapter-wide aggregation)

- passing unit tests are used to claim integration/runtime readiness.
- integration smoke is used to prove dependency boundary compliance.
- mocks are described as real driver behavior.
- v0.1 test success is inherited as v0.2 evidence.
- test failure reason is recorded as free-text only.
- unrerunnable local output is adopted as evidence.
- fixture/scenario class is hidden.
- golden fixture formatting is treated as canonical serialization without an explicit rule.
- fake driver owns domain acceptance/rejection semantics.
- fake driver returns success for unsupported operation.
- deterministic RNG/clock is used as production security evidence.
- test pass with fake is reported as live/runtime behavior.
- raw sensitive production data is committed as generic fixture.
- driver wire fixture becomes core API.
- import-only / smoke-only / text-inspection-only coverage is adopted as assertion-bearing coverage.
- line coverage is used to claim branch/path/function sufficiency, mutation score, fuzzing coverage, property-test completeness, protocol conformance completeness, security/runtime/production/live readiness, or native application command success.
- benchmark value is used as correctness / runtime readiness / production readiness proof.
- JS/TS evidence bypassing the `pnpm` rule is adopted.

## Collapse Conditions (chapter-wide aggregation)

This chapter's judgement collapses if: evidence class is not stated; the mock/fake boundary is hidden; fake driver behavior is treated as concrete driver proof; the failure reason cannot be mapped to closed vocabulary; runtime behavior is claimed from static source evidence; the test record lacks a correlation ID or rerun command; fixture/scenario data cannot be traced to an allowed class and redaction rule; code coverage evidence lacks denominator scope / threshold judgement / assertion-bearing classification / core crate floor; test evidence mixes into implementation source conditions; production readiness / live readiness / native command success is adopted while its required field-complete evidence is missing.
