# Troubleshooting

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes, in a self-contained manner, cross-cutting troubleshooting for arcRTC v0.2 **implementations** (the out-of-Kernel implementation region) in the form "symptom -> boundary / cause to suspect -> fail-closed default behavior -> remediation". implementations is the region that, without modifying the frozen Kernel contract, builds the reference / product implementation and the benchmark / real-device / readiness evidence surfaces, with the one-way dependency rule `implementations -> Kernel`. This chapter handles the fail-closed behavior and remediation internal to the boundaries of error / reason mapping, readiness matrix, Kernel contract import, and command / CI quality gate.

Representative boundary violations include a Kernel contract change request, deviation from the 3-type allow-list, forking the reference into product, retrofitting a benchmark threshold after measurement, diverting Kernel evidence for readiness, and claiming readiness without its dedicated readiness surface, with detection and remediation shown in the later tables.

## Reason / Error Mapping Troubleshooting

The implementations-local error enum does not add or change Kernel reason and is closed to the evidence field `implementation_reason`. Each error enum has no `Unknown` / `Other` / raw `String` variant, does not make a raw dependency error a public variant, and has `implementation_reason(&self) -> arcrtc_implementation_evidence::ImplementationEvidenceReason`. `Display` is limited to diagnostic text and is not adopted as a claim reason. `ImplementationEvidenceReason` has its only canonical Rust enum in `implementation-support/evidence/src/reason.rs`, and `arcrtc-reference-ops/src/reason.rs` and `arcrtc-product-monitoring/src/evidence.rs` use or re-export it without redefinition.

| Symptom | Boundary / cause to suspect | Fail-closed default behavior | Remediation |
|---|---|---|---|
| `UNKNOWN` / `Unknown` / `Other` / raw `String` reason appears in error output | error enum deviates from the closed set, or diagnostic is diverted to claim reason | output containing an `UNKNOWN` reason is not adopted as formal evidence and is treated as diagnostic | map the variant to a closed-set `ImplementationEvidenceReason` (e.g. `KERNEL_CONTRACT_UNAVAILABLE`, `KERNEL_CONTRACT_MISMATCH`, `FIXTURE_IDENTITY_INVALID`, `STATE_BOUNDARY_VIOLATION`, `EVIDENCE_FIELDS_INCOMPLETE`, `RUNTIME_EXECUTOR_ERROR`, `COMMAND_SCOPE_MISMATCH`, `READINESS_NOT_ADMITTED`) and remove `Unknown` / `Other` / raw `String` variants |
| a dependency raw error appears as an evidence reason | a raw dependency error is made a public variant or promoted to an evidence reason | reason mapping containing a raw dependency error is not adopted | project the Kernel reason / dependency error to an implementation-facing reason in the error mapper and convert the raw error to a closed-set variant |
| a product error is promoted to the Kernel reason catalog | the product error is mixed into Kernel semantic authority | an implementation containing a Kernel reason catalog change is fail-closed | close the product error variant (e.g. `ProductPolicyError::SecurityReasonMappingFailed` -> `STATE_BOUNDARY_VIOLATION`) into `ImplementationEvidenceReason` and do not change the Kernel reason catalog |
| `ReadinessNotAdmitted` is treated as success | a readiness failure is diverted to success | `ReadinessNotAdmitted` is fail-closed as the `READINESS_NOT_ADMITTED` reason | record the readiness failure as `READINESS_NOT_ADMITTED` and exclude it from the success path |
| an `ImplementationEvidenceReason` enum is redefined package-locally | reason ownership violation | a redefined reason set is treated as collapsed due to variant mismatch | use or re-export the canonical enum in `implementation-support/evidence/src/reason.rs` and delete the package-local redefinition; keep the variant set consistent with the observability closed set |
| an unmapped error variant is added | a mapping omission when extending the error enum | a variant with a missing mapping is a collapse condition | add the `ImplementationEvidenceReason` mapping for the added variant to the error / reason mapping table |

## Readiness Troubleshooting

Readiness is not established by source scaffold, build pass, test pass, benchmark result, or real-device command success alone. Production readiness and live readiness are separate claims, each established only by its own dedicated readiness surface. The `auth_provider_admission_ref` and the `persistence_provider_admission_ref` are separate fields and MUST NOT be merged. A readiness evidence field MUST NOT contain raw secret / raw token / private key / raw packet payload / direct personal identifier.

The production readiness surfaces are: product workspace build, product behavior test, product auth provider admission, product persistence provider admission, product deployment profile, product monitoring, rollback / drain plan, and security secret scan. The live readiness surfaces are: production readiness (prerequisite), bounded live endpoint, public traversal, live monitoring probe, live rollback / drain execution, live shutdown drain, and live restore. Chapter 12 fixes the full readiness surface, source contracts, and failure classification.

| Symptom | Boundary / cause to suspect | Fail-closed default behavior | Remediation |
|---|---|---|---|
| product test pass is about to be treated as production readiness | readiness substituted by test pass | production readiness without its dedicated readiness surface is a collapse condition | establish all production readiness surfaces before treating the system as production-ready |
| production readiness is treated as live readiness | production used as a substitute for live | live readiness without live evidence is a collapse condition | make production readiness the prerequisite and establish the live readiness surfaces separately |
| auth / persistence provider admission proceeds with the provider unadopted | auth / persistence provider authority is absent | when provider authority is absent, fail closed with `READINESS_NOT_ADMITTED` | adopt auth provider authority and persistence provider authority separately and fill `auth_provider_admission_ref` and `persistence_provider_admission_ref` separately |
| auth and persistence provider admission share one generic provider authority | provider admission not separated | unable to express one-side-unadopted fail-closed, hence collapses | handle auth provider and persistence provider with separate authorities, separate source owners (`product-implementation/product-policy/` and `product-implementation/persistence-topology/`), and separate ref fields |
| live admission proceeds while live endpoint authority is absent | live endpoint authority absent | all live readiness surfaces fail closed with `READINESS_NOT_ADMITTED` | generate `ProductHostClass::LiveAdmitted` only from the live profile derived from `ProductLiveEndpointAdmission` |
| a benchmark / real-device command result is adopted as readiness | command result diverted to readiness | readiness from a benchmark / real-device result alone is a collapse condition | handle benchmark threshold satisfaction and real-device success as independent evidence and separate them from the readiness surfaces |
| a provider-deferred state is treated as an admitted provider state | deferred promoted to admitted | a deferred state is not admitted and is a collapse condition | do not proceed with readiness until provider admission actually returns admitted authority |
| readiness evidence contains secret / token / private key / raw packet payload | redaction violation | readiness evidence containing a secret is a collapse condition | remove raw secret / token / private key / raw packet payload / direct personal identifier from readiness evidence fields |
| `auth_provider_admission_ref` and `persistence_provider_admission_ref` are merged | provider ref fields not separated | a merged provider ref cannot express one-side-unadopted fail-closed | keep the auth provider ref and the persistence provider ref as separate fields |
| readiness evidence with a missing required field is about to be adopted | required field missing | fail closed with `EVIDENCE_FIELDS_INCOMPLETE` | supply the missing required field for the readiness surface |

## Kernel Contract Import / Consumption Troubleshooting

implementations may use only Kernel public contract, Kernel SDK projection, core-owned port definition, documented command surface, and Kernel report references. The Kernel crate may import only packages listed in the Kernel local path dependency matrix. implementations MUST NOT import / overwrite / duplicate Kernel core semantics, Kernel reason catalog ownership, Kernel port definition ownership, Kernel dependency direction, or Kernel completion / freeze claim authority.

| Symptom | Boundary / cause to suspect | Fail-closed default behavior | Remediation |
|---|---|---|---|
| a Kernel contract gap is found during implementation | Kernel contract gap | (1) stop the implementations-side task, (2) record the gap, (3) require a Kernel-side versioned authority, (4) do not bypass / duplicate / extend | first prepare a versioned authority on the Kernel side and do not create a substitute type on the implementations side |
| a Kernel crate not in the dependency matrix is imported | import path rule violation | importing a Kernel crate outside the matrix is a collapse condition | if an additional import is needed, update the dependency matrix before the source change and record in the report why it does not encroach on Kernel semantic authority |
| the Kernel contract is duplicated / extended / bypassed on the implementations side | consumption boundary violation | semantic authority encroachment is a collapse condition | limit the wrapper / mapper to converting / calling the Kernel contract and remove any new definition of Kernel semantics |
| Kernel evidence is adopted as implementations completion / readiness proof | evidence diversion | diverting Kernel evidence is a collapse condition | establish implementations readiness only with its dedicated readiness surface and do not divert Kernel evidence to proof |
| Kernel source modification is treated as an ordinary task | Kernel immutability violation | Kernel source change in implementations scope is a collapse condition | do not modify Kernel source and require Kernel-side authority first as fail-closed for the Kernel contract gap |

## Command / CI Quality Gate Troubleshooting

All implementations commands have the working directory root `implementations`. Kernel workspace root command success is not grounds for implementations readiness. The command classes are format, build, test, benchmark, real-device, and readiness, and success cannot be diverted beyond the claim boundary of each class. Formal command evidence has correlation id, command, working directory, target scope, command class, toolchain / runtime version, expected outcome, actual outcome, environment class, implementation reason, non-claim scope, and rerun condition.

| Symptom | Boundary / cause to suspect | Fail-closed default behavior | Remediation |
|---|---|---|---|
| a Kernel working directory command result is made implementations evidence | working directory root violation | adopting a Kernel root command result for implementations is a collapse condition | run commands from `implementations` and adopt only that evidence |
| diversions such as treating build success as test pass or test pass as threshold satisfaction | exceeding the command class boundary | diversion outside the claim boundary is a collapse condition | keep the claim boundary per command class and do not mutually substitute build / test / benchmark / real-device / readiness success |
| command output lacking required fields is made formal evidence | evidence required fields missing | output lacking a required field is not adopted as formal evidence | follow the command-class required field matrix in Chapter 05 (per-class requiredness of `target_package` / `input_fixture_or_workload` / `exit_status`) and supply the missing field |
| benchmark execution is treated as threshold satisfaction | benchmark promoted to threshold | treating benchmark execution as threshold is a collapse condition | limit benchmark execution to measurement output and judge threshold satisfaction only with a pre-adopted threshold value and comparison rule (Chapter 10) |
| real-device command success is treated as live readiness | real-device promoted to readiness | treating real-device command success as live readiness is a collapse condition | limit real-device command success to command scope and establish live readiness only with the dedicated live readiness surface |
| CI success is diverted to production / live readiness | CI admission boundary violation | diverting CI success to readiness is a collapse condition | limit the CI workflow claim boundary to command classes such as docs / build / test and do not auto-claim readiness via CI |

## Representative Boundary Violations and Detection / Remediation

This section cross-cuts the frequently recurring boundary violations across implementations. All default to fail-closed.

| Boundary violation | Detection | Fail-closed default behavior | Remediation |
|---|---|---|---|
| Kernel contract change request | a Kernel contract gap is found during implementation and a temptation to bypass / duplicate / extend / override arises | stop the implementations task and require a Kernel-side versioned authority first | do not modify Kernel source; prepare the contract on the Kernel side and then resume implementations |
| 3-type allow-list deviation | the product plane consumes anything other than `arcrtc-reference-output`'s `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` (`ReferenceCompositionOutcome`, reference state / function / fixture / local auth / runtime / composition symbol, wildcard / module / re-export / type alias import) | input outside the allow-list is detected by the boundary test and is a collapse condition | limit product input to the exact 3 types and remove other reference symbol imports |
| forking the reference into product | the product implementation forks / copies the reference to mix in product policy | inheriting reference success into product is a collapse condition | the product consumes only the 3 outcome types as input and deletes the reference fork / copy |
| retrofitting a benchmark threshold | the threshold value is set to fit the result after benchmark measurement | post-measurement threshold mutation is a collapse condition | adopt the threshold before the verdict measurement and judge by `measured_p95_ns <= threshold_value_ns` (Chapter 10) |
| diverting Kernel evidence for readiness | Kernel completion / freeze evidence is diverted to proof of production / live readiness | diverting Kernel evidence is a collapse condition | establish readiness only with the dedicated production / live readiness surfaces, and limit Kernel evidence to the Kernel completion / freeze binding |
| real-device success with insufficient rows | any of the six required device classes (Android physical / emulator, iOS physical / simulator, desktop / mobile browser) is omitted, or a row with non-`0` exit / missing required field is treated as success | missing class / nonzero exit / field omission / redaction collapse is fail-closed | mark all 6 rows with exit `0`, actual outcome, runtime version class, network class, toolchain / runtime version, and a redacted identifier of `redacted` or `sha256:<64 lowercase hex characters>`, and do not substitute Android / iOS rows with browser rows |
