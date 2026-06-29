# Entrypoints Composition Root and Configuration Boundary

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter specifies, in a fully self-contained form (understandable without opening any other file, source dev-doc, or implementation code), the composition root (dependency injection / wiring) boundary of `entrypoints/` in the arcRTC v0.2 Kernel, the core-owned configuration boundary types, the configuration profile / policy bundle, and the complete procedures and prohibitions for runtime reconfiguration / policy hot-swap. The granularity is sufficient for reimplementation from this chapter alone.

entrypoints own the Kernel executable contract, CLI, demo, dependency wiring, and composition evidence surface, but MUST NOT own domain rule, protocol semantics, port contract, or the SFU / TURN / Signaling product implementation.

## Section 1. Entrypoint Set (closed set)

The Kernel entrypoint set of the v0.2 initial architecture is limited to the following. Any new entrypoint is out of the v0.2 initial scope.

| Entrypoint | Responsibility |
|---|---|
| `entrypoints/signaling-server` | Signaling executable contract / composition evidence surface |
| `entrypoints/sfu-server` | SFU executable contract / composition evidence surface |
| `entrypoints/turn-server` | TURN executable contract / composition evidence surface |
| `entrypoints/cli` | operator / developer CLI |
| `entrypoints/demo` | demo composition |
| `entrypoints/configuration` | configuration profile, policy bundle, and feature/capability wiring |
| `entrypoints/endpoints` | public endpoint lifecycle and edge/proxy trust wiring |
| `entrypoints/topology` | deployment topology, service discovery, and endpoint resolution wiring |
| `entrypoints/internal-control` | internal service identity and control-plane wiring |
| `entrypoints/admin` | health/readiness/liveness/admin/operator wiring |

## Section 2. Composition Rule

entrypoints MAY perform the following actions.

- parse environment variables, files, process args, and deployment settings;
- construct typed configuration;
- choose driver implementations;
- wire core use cases to driver implementations;
- start and stop executable process;
- expose CLI/demo commands that call core use cases through allowed boundaries.

entrypoints MUST NOT define an alternate domain decision, reason vocabulary, port trait, or state transition.
entrypoints MUST NOT be treated as reference implementation, product implementation, production readiness, or live readiness evidence.

## Section 3. Wiring Rule (dependency direction)

entrypoints wire dependencies in this direction only. The dependency notation `A <- B` means "B depends on A".

```text
entrypoints -> core
entrypoints -> drivers
drivers -> core
```

That is, `core <- drivers`, `core <- entrypoints`, and `drivers <- entrypoints`.

- entrypoints MUST NOT introduce a dependency from core to entrypoints or from drivers to entrypoints.
- entrypoints MUST NOT wire `regulated` into a generic communication path unless a regulated boundary interaction is explicitly permitted.
- entrypoints MAY depend on core and drivers but MUST NOT own domain rule. entrypoints MUST NOT own the SFU / TURN / Signaling product system. `entrypoints -> regulated` on the generic path is prohibited.

## Section 4. Startup Fail-Closed Rule

startup / wiring MUST fail closed when required configuration, driver implementation, secret source, clock, RNG, runtime, persistence, audit sink, or metrics sink cannot be initialized.

| Failure | Required reason |
|---|---|
| required runtime configuration missing | `runtime_config_missing` |
| runtime configuration cannot initialize selected driver/entrypoint | `runtime_config_invalid` |
| required secret source unavailable | `secret_unavailable` |
| selected driver unavailable after bounded initialization | corresponding driver failure reason |
| selected deployment topology unsupported | `deployment_topology_unsupported` |
| service discovery unavailable for required dependency | `service_discovery_unavailable` |
| service discovery fallback or endpoint scope not admitted | `service_endpoint_fallback_not_allowed` or `service_endpoint_scope_conflict` |
| distributed state, replication, or failover not admitted | `distributed_state_not_admitted`, `state_replication_not_admitted`, or `failover_not_proven` |
| runtime task supervision/spawn/join/cancel failure | runtime task reason |
| internal service identity/trust mapping failure | internal service identity reason |
| public endpoint class not admitted for selected entrypoint/profile | `public_endpoint_not_allowed` |
| runtime reconfiguration not admitted for selected entrypoint/profile | `runtime_reconfiguration_not_allowed` |

If startup failure cannot be recorded by the normal audit sink, the reserved bootstrap audit record path applies. If neither path records the failure, that startup attempt MUST NOT be used as closeout evidence.

## Section 5. Configuration Ownership (core-owned boundary types)

configuration is separated into core policy input and driver/entrypoint runtime input.

| Configuration kind | Owner | Examples |
|---|---|---|
| core policy configuration | core | thresholds, bounds, accepted versions |
| driver runtime configuration | drivers | socket address, TLS files, DB DSN, S3 bucket |
| entrypoint composition configuration | entrypoints | selected drivers, process options |
| SDK client configuration | sdk | signaling endpoint URL, reconnect behavior |
| regulated configuration | regulated | domain-specific mapping and enrichment |

### Core Configuration Rule
core MAY receive configuration only as core-owned typed policy. core MUST NOT read environment variables, files, process args, OS settings, or cloud metadata.

### Driver Configuration Rule
driver MAY receive external configuration only as typed runtime configuration wired by entrypoints. driver MUST NOT derive core policy from runtime settings. driver-local runtime configuration is limited to external implementation initialization and I/O execution. driver-local initialization MUST NOT read environment variables, files, process args, or deployment metadata directly.

### Entrypoints Configuration Rule
entrypoints MAY read environment variables, files, process args, and deployment settings. entrypoints do not own domain decision. entrypoints wire configuration into core/drivers as typed input. entrypoints MUST NOT silently substitute defaults for required policy or runtime inputs.

### Policy Validation Rule
core owns interpretation and validation of core policy configuration. entrypoints MAY parse external input into typed configuration structures, but policy acceptance / rejection belongs to core. driver MAY validate driver-local runtime settings needed to initialize external implementation, but MUST NOT turn invalid core policy into accepted behavior.

## Section 6. Configuration Failure Rule (fail-closed)

configuration failure MUST fail closed.

| Failure | Reason code |
|---|---|
| typed core policy configuration is invalid | `core_policy_config_invalid` |
| required runtime configuration is missing | `runtime_config_missing` |
| runtime configuration cannot initialize selected driver/entrypoint | `runtime_config_invalid` |
| required secret source is unavailable | `secret_unavailable` |

Missing or invalid required configuration MUST stop startup or reject the affected command path with the listed reason. Implicit fallback, silent defaulting, or partial enablement without reason is prohibited. configuration failure MUST emit a `configuration_decision` audit event with cataloged reason before the startup path is stopped or the affected command path is rejected. If the failed configuration prevents selected audit sink initialization, `configuration_decision` MUST be recorded through the reserved bootstrap audit record path. If neither the normal audit sink nor the bootstrap path can record the failure, startup MUST stop and that startup attempt MUST NOT be used as closeout evidence. Startup configuration acceptance does not admit runtime reconfiguration.

## Section 7. Feature Flag Rule

feature flag MUST NOT change core semantics implicitly.

Allowed:
- select driver implementation;
- enable optional exporter;
- choose external encoding.

Prohibited:
- silently change Signaling state machine;
- silently change TURN lifecycle;
- silently change SFU routing semantics;
- bypass security verification;
- bypass audit requirement.

## Section 8. Secret Rule

secrets are driver/entrypoint concerns. core MUST NOT own raw secrets. core MAY own an opaque credential reference or verification result.

## Section 9. Configuration Profile / Policy Bundle

configuration validates multiple core policies, driver runtime configs, and entrypoint composition configs as a bundle. Bundle composition MUST NOT move policy ownership.

| Bundle | Owner | Rule |
|---|---|---|
| core policy bundle | core | thresholds, accepted versions, bounds, security requirements |
| driver runtime bundle | driver | socket, DB, exporter, TLS/key source references |
| entrypoint composition bundle | entrypoints | selected drivers and startup mode |
| deployment topology bundle | entrypoints | entrypoint/service topology class, service discovery, node affinity |
| service discovery bundle | entrypoints/drivers | discovery source, endpoint scope, TTL/cache, fallback |
| internal service trust bundle | entrypoints/drivers | trust class, source/target service, credential/peer proof reference, scope |
| distributed state bundle | entrypoints/deployment | state class, owner scope, affinity, failover/replication admission |
| runtime task bundle | entrypoints/drivers | task class, supervision scope, join/cancel bound |
| secret rotation bundle | driver | secret source references, accepted generations, overlap/revocation policy |
| supply-chain bundle | CI/release scope | dependency, license, vulnerability, lockfile, toolchain evidence |
| SDK client bundle | sdk | Signaling endpoint and client-local behavior |
| regulated mapping bundle | regulated | optional domain support, not generic core policy |
| test profile bundle | testing scope | deterministic and fake settings only for evidence class |

### Profile Classes (closed set)

| Profile class | Meaning | Adoption rule |
|---|---|---|
| `development_local` | local manual run profile | not production evidence |
| `test_deterministic` | deterministic clock/RNG/fake driver profile | test evidence only |
| `integration_controlled` | controlled integration profile | integration evidence only |
| `benchmark_controlled` | benchmark profile | benchmark evidence only |
| `production_candidate` | candidate production-like profile | requires explicit evidence reports before runtime claim |

A new profile class is out of the v0.2 initial scope.

### Bundle Validation Rule (order)

startup/wiring MUST validate bundles in this order:

1. entrypoint composition bundle is parseable and complete;
2. selected driver runtime bundles are present and internally valid;
3. core policy bundle is present and accepted by core;
4. cross-bundle references resolve without raw secret leakage;
5. feature/capability settings are allowed by the feature lifecycle authority;
6. deployment topology class is explicit and accepted;
7. service discovery source, endpoint scope, and fallback policy are explicit when discovery is configured;
8. distributed state class and owner scope are explicit when topology can touch node-local state across nodes;
9. internal service trust class and accepted scope are explicit when service-to-service identity affects the profile;
10. runtime task class and supervision scope are explicit when worker execution affects the profile;
11. secret rotation policy source is present for any configured credential verifier/key source;
12. supply-chain/toolchain evidence class is declared for build/release claims;
13. evidence class of the profile is declared.

Partial bundle acceptance is prohibited unless a degraded mode and its close-not-claimed scope are defined. Profile validation at startup does not authorize runtime profile swap unless a reconfiguration class admits it.

### Bundle Failure Mapping

| Failure | Required reason |
|---|---|
| required entrypoint/runtime bundle missing | `runtime_config_missing` |
| entrypoint/runtime bundle cannot initialize selected driver/entrypoint | `runtime_config_invalid` |
| core policy bundle invalid | `core_policy_config_invalid` |
| required secret source unavailable | `secret_unavailable` |
| secret rotation policy/state unavailable where required | `secret_rotation_state_unavailable` |
| selected deployment topology unsupported | `deployment_topology_unsupported` |
| service discovery source unavailable | `service_discovery_unavailable` |
| discovery source, endpoint scope, or fallback policy invalid | `service_discovery_source_not_admitted`, `service_endpoint_scope_conflict`, or `service_endpoint_fallback_not_allowed` |
| distributed state, replication, consensus, or failover policy invalid | `distributed_state_not_admitted`, `state_replication_not_admitted`, `consensus_not_admitted`, or `failover_not_proven` |
| internal service trust profile invalid or not admitted | internal service identity reason |
| runtime task profile invalid or not admitted | runtime task reason |
| toolchain or lockfile does not match accepted profile | `toolchain_version_mismatch` or `lockfile_drift_detected` |
| dependency/license/vulnerability policy rejected | `dependency_policy_violation`, `license_policy_violation`, or `vulnerability_gate_failed` |
| selected feature/capability disabled | `capability_not_enabled` |
| test-only profile used for non-test evidence | evidence report rejection, not runtime success |
| runtime profile swap not admitted | `runtime_reconfiguration_not_allowed` |

### Evidence Rule
An evidence report MUST declare profile class and bundle sources without raw secret material. Evidence produced under `test_deterministic` MUST NOT be used as runtime or production evidence. Evidence produced under one profile class MUST NOT be promoted to another evidence class without a new report.

## Section 10. Runtime Reconfiguration / Policy Hot-Swap

Sections 5–9 handle configuration validation at startup/wiring time; this section fixes the conditions under which settings or policy MAY change after startup, or prohibits it. Runtime reconfiguration is an operation that attempts to change effective settings such as core policy, driver runtime configuration, entrypoint composition, feature/capability, endpoint exposure, security material, topology, or observability export after process start. In v0.2, everything except an explicitly admitted reconfiguration class is treated as startup-only.

### Reconfiguration Classes (closed set)

| Reconfiguration class | Meaning | Rule |
|---|---|---|
| `startup_only` | change requires restart and startup validation | default for all settings |
| `secret_rotation_reload` | reload secret/key material under the rotation authority | not general policy change |
| `observability_export_reload` | change exporter sink/level within taxonomy bounds | MUST NOT affect domain decisions |
| `maintenance_mode_switch` | enter/exit maintenance or drain mode | health/admin and shutdown authorities apply |
| `test_profile_swap` | deterministic/test-only profile swap | test evidence only |
| `runtime_policy_hotswap` | core policy generation changes without restart | prohibited in the v0.2 initial scope |

A new reconfiguration class is out of the v0.2 initial scope.

### Generation State Rule (closed set)

Runtime reconfiguration MUST use these closed generation states.

| State | Meaning |
|---|---|
| `current_generation` | active and accepted for its declared scope |
| `pending_generation` | parsed but not accepted |
| `validating_generation` | under validation and not active |
| `rejected_generation` | failed validation and must not apply |
| `rollback_generation` | prior accepted generation used for rollback |
| `retired_generation` | no longer active and not accepted for new decisions |

Generation names are evidence references, not raw secret values or unredacted config payload.

### Admission Rule
A runtime reconfiguration request MUST record: reconfiguration class; target configuration surface; target owner; current generation reference; proposed generation reference; apply scope; affected active sessions/connections/allocations/routes; drain/restart requirement; rollback behavior; audit event type; evidence class; close-not-claimed scope. A request that cannot supply those fields MUST be rejected before apply.

### Apply Rule
Runtime reconfiguration MUST NOT mutate an already accepted domain decision retroactively. Existing Signaling rooms, SFU sessions/routes, TURN allocations/permissions, packet caches, idempotency windows, and audit hash-chain scopes keep the generation under which their decisions were accepted unless the target contract explicitly defines migration or re-evaluation. When a setting affects public endpoint exposure, security mode, topology, authorization, rate/quota, or protocol compatibility, startup/restart or controlled drain is required unless hot-swap for that exact class is admitted in the v0.2 initial scope.

### Rollback Rule
Rollback is not automatic success. Rollback MUST define: rollback generation; rollback trigger; rollback apply scope; rollback evidence; affected in-flight operation handling; reason when rollback fails. If rollback is required but cannot be proven, the target path MUST NOT be used as readiness or closeout evidence.

### Reconfiguration Failure Mapping

| Failure | Required reason |
|---|---|
| runtime reconfiguration class is not admitted | `runtime_reconfiguration_not_allowed` |
| required configuration generation reference is missing | `configuration_generation_missing` |
| proposed generation conflicts with active generation/order | `configuration_generation_conflict` |
| proposed generation fails validation | `runtime_reconfiguration_validation_failed` |
| apply is not allowed for target surface or active scope | `runtime_reconfiguration_apply_not_allowed` |
| drain/restart is required before apply | `runtime_reconfiguration_drain_required` |
| rollback is required before the evidence is accepted | `runtime_reconfiguration_rollback_required` |
| rollback execution failed | `runtime_reconfiguration_rollback_failed` |

### Audit Rule
Runtime reconfiguration decisions use audit event type `runtime_reconfiguration_decision`. The event MUST carry reconfiguration class, target surface, current generation, proposed generation, apply scope, drain/restart class, rollback class when applicable, `StartupRunId`, and `CorrelationId` when command-scoped.

## Section 11. CLI / Demo Rule

CLI and demo MAY expose convenient flows, but they are not an alternate domain authority. CLI/demo commands MUST map to core-owned command/use case boundaries. Demo defaults MUST NOT become production policy. Admin and maintenance commands MAY be wired by entrypoints, but their decision semantics, evidence requirements, and failure mapping remain under the health/admin authority. Operator/admin authorization MUST be satisfied before a privileged admin or maintenance action is executed. CLI/demo MUST reject or mark close-not-claimed any out-of-scope feature request that lacks core contract admission. CLI/demo surfaces MUST expose their own fail-closed composition guards so that server-wide entrypoint composition tests do not become a marker-only substitution for CLI/demo-specific boundaries. A CLI/demo path bypassing a core use case is prohibited.

## Section 12. Prohibitions

- entrypoints define port traits.
- entrypoints own Signaling join acceptance, SFU route selection, or TURN permission decision.
- entrypoints own SFU / TURN / Signaling reference implementation or product implementation.
- entrypoints implement retry/backpressure/resource policy outside core/drivers contract boundaries.
- entrypoints silently substitute defaults for required policy or runtime configuration.
- entrypoints spawn detached workers outside admitted task supervision.
- entrypoints treat endpoint resolution or TLS listener success as internal service trust.
- a CLI/demo path bypasses a core use case.
- entrypoints depend on regulated support in a generic communication path.
- an entrypoint health/readiness probe invents domain state or readiness success without required dependency observation.
- supervisor restart handling is treated as domain recovery without a recovery policy.
- deployment topology changes domain semantics or dependency direction.
- service discovery fallback changes endpoint or owner scope without policy.
- entrypoints imply replication, consensus, or failover from multi-node wiring.
- entrypoints expose public/internal endpoint class without an endpoint admission record.
- entrypoints treat a packaged binary as release/distribution evidence without provenance.
- entrypoints trust edge/proxy metadata without an edge trust policy.
- entrypoints hot-swap policy/profile without a reconfiguration generation.
- entrypoint profile changes core semantics silently.
- driver runtime config derives core policy.
- a missing required bundle falls back to default.
- test profile becomes production profile.
- profile evidence omits profile class.
- runtime profile swap is inferred from successful startup validation.

## Section 13. Collapse Conditions

- executable entrypoint owns domain rule.
- executable entrypoint is treated as product implementation.
- composition root defines a second reason catalog.
- entrypoints bypass core-owned ports to call driver internals as domain authority.
- startup failure is hidden while claiming close / complete / ready.
- demo configuration becomes implicit production policy.
- entrypoint profile or feature flag silently changes core semantics.
- entrypoint-owned admin, maintenance, health, or process lifecycle path becomes domain authority.
- single-process or local topology evidence is used as split-service or multi-node proof.
- operator/admin authorization is inferred from CLI access or localhost.
- CLI/demo admits an out-of-scope feature into production scope.
- entrypoint listener wiring is treated as public endpoint readiness without endpoint lifecycle evidence.
- entrypoint reconfiguration path changes core semantics without a target-contract definition.
- bundle ownership changes policy ownership.
- partial config acceptance hides missing required policy.
- raw secret material appears in profile evidence.
- runtime reconfiguration class is absent.
- active/pending/rejected generation state is implicit.
- apply scope is not recorded.
- hot-swap can change core semantics without a target-contract definition.
- rollback is claimed without rollback generation and evidence.

## Section 14. Invariants (summary)

- The dependency direction is only `entrypoints -> core`, `entrypoints -> drivers`, and `drivers -> core`. The reverse directions and `entrypoints -> regulated` on the generic path are invariantly prohibited.
- core does not read the external environment (env/file/args/OS/cloud metadata) and receives only typed policy.
- Missing or invalid required configuration / bundle / secret / driver always fails closed and leaves an audit with a cataloged reason.
- startup-only is the default; settings do not change at runtime except for an explicitly admitted reconfiguration class.
- An already accepted domain decision is not retroactively reinterpreted under a new generation.
- raw secret does not appear in core/audit/log/report/evidence (only an opaque reference does).
