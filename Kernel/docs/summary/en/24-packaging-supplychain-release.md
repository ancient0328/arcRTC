# Packaging / supply-chain / release: dependency, license, toolchain gate, and release artifact, distribution, provenance

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter specifies, in a fully self-contained form (understandable without opening any other file, source dev-doc, or the actual code), the supply chain and release boundaries of the arcRTC v0.2 Kernel. It covers two areas: supply chain / dependency / license / toolchain gate; and release artifact / distribution / provenance. The granularity is sufficient for re-implementation from this chapter alone.

The detail of the crate / package boundary is owned by Chapter 02. Taking that as given, this chapter internalizes supply chain and release. It fixes, as owners and fail-closed conditions, the conditions under which crate / npm / Gradle / SwiftPM / toolchain dependencies do not break the architecture boundary, security posture, or evidence claim, and under which a generated binary / crate / package / image / SDK package / docs bundle is treated as a distributable artifact.

This chapter does not claim dependency install, SBOM generation, license audit, vulnerability scan, release execution, or publish success. These are handled by a separate report after the evidence command class is made explicit.

---

## Part A. Supply Chain / Dependency / License / Toolchain Gate

### A-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| architecture dependency direction | Chapter 02 | core/drivers/entrypoints/sdk/regulated boundary |
| package dependency admission | package/scaffold governance | dependency class and owner layer required |
| third-party implementation dependency | package owner | must not invert layer dependency |
| license policy | project governance | allow/deny/review classification |
| vulnerability policy | CI/security governance | scan evidence required before adoption claim |
| toolchain version | scaffold/CI governance | pinned or explicitly ranged |
| generated artifacts | build tooling | not source authority unless admitted |

Workspace membership or package manager success does not imply architecture permission.

### A-2 Dependency Classes (closed set)

The dependency classes of v0.2 initial architecture are limited to the following.

| Class | Meaning | Rule |
|---|---|---|
| `core_semantic_dependency` | dependency used by a core semantic package | no driver/runtime concrete dependency |
| `driver_implementation_dependency` | concrete I/O/runtime/parser/backend dependency | driver package only |
| `entrypoint_composition_dependency` | executable wiring/config dependency | entrypoints package only |
| `sdk_public_dependency` | SDK platform public or implementation dependency | no regulated/core internals leakage |
| `regulated_optional_dependency` | regulated support dependency | must not become a generic core requirement |
| `test_tool_dependency` | test/build/fixture support | not a production dependency |
| `build_tool_dependency` | formatter/codegen/build helper | evidence class required |

A new dependency class is out of the v0.2 initial scope.

### A-3 Admission Rule (mandatory record at dependency adoption)

Dependency admission MUST record:

- package manager and package name;
- owner layer;
- dependency class;
- direct/transitive status;
- license class;
- vulnerability scan status when security-sensitive;
- lockfile impact;
- architecture dependency impact;
- replacement/removal condition when experimental.

A dependency MUST NOT be adopted as implementation evidence unless its boundary and owner are recorded.

### A-4 License and Vulnerability Rule

License and vulnerability evidence are separate from build/test evidence.

- Build success does not prove license acceptance.
- License acceptance does not prove vulnerability clearance.
- Vulnerability scan pass does not prove runtime security.

If license or vulnerability state blocks adoption, the affected dependency path MUST fail closed for release/readiness claims. Dependency gate success is an input to release artifact provenance, not a release artifact by itself.

### A-5 Failure Mapping (closed-set reasons)

| Failure | Required reason |
|---|---|
| dependency violates architecture policy | `dependency_policy_violation` |
| dependency license not accepted | `license_policy_violation` |
| vulnerability gate blocks adoption | `vulnerability_gate_failed` |
| toolchain version does not match policy | `toolchain_version_mismatch` |
| lockfile drift detected | `lockfile_drift_detected` |
| required dependency/tool missing for evidence command | `dependency_missing` as report reason |

### A-6 Evidence Rule / Audit Rule (supply chain)

Supply-chain evidence MUST record: package manager, command, working directory, lockfile state, dependency class, license result, vulnerability scan class when used, and close-not-claimed scope. Package install output lacking these fields is diagnostic only.

Supply-chain decisions use audit event type `supply_chain_decision`. The event MUST carry the package/toolchain reference, the dependency class, and the `CorrelationId` from the evidence or gate run.

### A-7 Prohibitions (supply chain)

- a core package imports a driver/runtime concrete dependency through a feature flag.
- package manager install success is treated as dependency policy acceptance.
- a generated/cache/local artifact is source authority.
- a license/vulnerability result is omitted for a release/readiness claim.
- npm/yarn is used for JS/TS evidence where project policy requires pnpm.
- a test helper becomes a hidden production dependency.
- dependency gate output is treated as release artifact provenance without artifact class, source ref, command, and digest.

### A-8 Part A Collapse Conditions

- a dependency class is absent.
- a license or vulnerability gate can be bypassed silently.
- lockfile drift is ignored while claiming reproducibility.
- package dependency direction violates the architecture boundary.
- toolchain evidence lacks version and working directory.
- a release/distribution claim bypasses artifact provenance classification.

---

## Part B. Release Artifact / Distribution / Provenance

### B-1 Boundary

A release artifact is something built / packaged / signed / verified / published from the source tree. A release artifact is not source authority; it is a derivative connected to its originating source ref, toolchain, dependency gate, test/evidence, and provenance.

| Concern | Owner | Rule |
|---|---|---|
| source authority | repository / core contract | release artifact does not replace source |
| package boundary | crate/package contract (Chapter 02) | package ownership and layer direction required |
| dependency / license / vulnerability | supply-chain (Part A) | release claim cannot bypass gates |
| build and test evidence | CI/testing policy | command evidence class must be explicit |
| artifact provenance | release governance | source ref, toolchain, command, digest required |
| distribution channel | release governance / entrypoints / SDK owner | channel must be admitted |

### B-2 Release Artifact Classes (closed set)

The release artifact classes of v0.2 initial architecture are limited to the following.

| Artifact class | Meaning |
|---|---|
| `rust_crate_package` | Rust crate package for internal or external distribution |
| `kernel_contract_entrypoint_binary` | Kernel executable contract / composition evidence entrypoint artifact |
| `container_image` | containerized Kernel entrypoint artifact |
| `typescript_sdk_package` | pnpm/npm-distributable SDK package, built with pnpm evidence |
| `android_sdk_package` | Android SDK package artifact |
| `ios_sdk_package` | iOS/Swift package artifact |
| `documentation_bundle` | docs bundle intended for release distribution |
| `ci_evidence_bundle` | release-support evidence bundle |

A new release artifact class is out of the v0.2 initial scope.

### B-3 Provenance Rule (mandatory record per release claim)

Every release artifact claim MUST record:

- artifact class;
- source ref or immutable source snapshot;
- package/module name;
- build command and working directory;
- toolchain version;
- dependency/lockfile state;
- license and vulnerability gate state when applicable;
- test/evidence reports adopted for the claim;
- artifact digest;
- signature or explicit unsigned class;
- distribution channel;
- rollback/removal condition;
- correlation ID.

An artifact digest without source ref and command is not provenance. Build success without dependency/license/vulnerability state is not release readiness.

### B-4 Distribution Channel Rule (closed-set channels)

The distribution channel is closed to the following initial vocabulary.

| Channel | Rule |
|---|---|
| `local_artifact_only` | local build output, not a public release |
| `internal_registry` | private registry or artifact store |
| `public_registry` | public package/container registry |
| `github_release` | GitHub release asset |
| `documentation_site` | published docs site or docs artifact host |

A new channel is out of the v0.2 initial scope. Public distribution requires explicit public channel admission, provenance, redaction review, and supply-chain gate evidence.

### B-5 Release Claim Rule

Release artifact evidence MAY support only the scope stated by its report. It MUST NOT imply production readiness, live operational readiness, security certification, protocol compatibility, SDK parity, or regulated workflow support unless those claims have separate evidence.

### B-6 Failure Mapping (closed-set reasons)

| Failure | Required reason |
|---|---|
| release artifact was not built for the claimed class | `release_artifact_not_built` |
| provenance fields are missing | `release_artifact_provenance_missing` |
| artifact digest/signature verification fails | `release_artifact_integrity_failed` |
| source/package/version does not match the claimed release | `release_version_mismatch` |
| distribution channel is not admitted | `distribution_channel_not_allowed` |

### B-7 Evidence Rule / Audit Rule (release)

Release evidence MUST record: artifact class, source ref, package/module, build command, working directory, toolchain, dependency/license/vulnerability state, digest/signature class, distribution channel, adopted tests/reports, and rerun condition. Package manager or registry output alone is diagnostic only.

Release artifact decisions use audit event type `release_artifact_distribution_decision`. The event MUST carry the artifact class, source ref, package/module reference, artifact digest reference, distribution channel, provenance class, and `CorrelationId`.

### B-8 Prohibitions (release)

- a generated release artifact becomes source authority.
- build output is treated as dependency/license/vulnerability clearance.
- a local artifact is described as public distribution.
- public registry publish is attempted without an admitted distribution channel.
- an unsigned artifact is described as signed.
- release artifact evidence is used as runtime/live readiness proof.
- an SDK package release is used to claim server semantic correctness.

### B-9 Part B Collapse Conditions

- an artifact class is absent.
- source ref, command, toolchain, or digest is absent from provenance.
- the distribution channel is open-ended.
- a supply-chain gate can be bypassed for a release claim.
- a generated artifact is treated as canonical source.

---

## Part C. Invariants (summary)

- The detail of the crate / package boundary is owned by Chapter 02. This chapter internalizes supply chain and release.
- Workspace membership or package manager / build / install success is neither architecture permission nor dependency policy acceptance.
- A dependency is not adopted as implementation evidence unless its dependency class and owner layer are recorded, and it does not invert the architecture dependency direction.
- License, vulnerability, and build/test evidence are mutually independent, and a pass of one does not prove another. On block, the affected path fails closed for release/readiness claims.
- A release artifact is a derivative, not source authority, and provenance does not hold without artifact class / source ref / build command / working directory / toolchain / dependency-license-vulnerability state / digest / signature-or-unsigned-class / distribution channel / correlation ID.
- The distribution channel is a closed set (`local_artifact_only` / `internal_registry` / `public_registry` / `github_release` / `documentation_site`), and public distribution requires explicit admission, provenance, redaction review, and supply-chain gate evidence.
- Release/supply-chain evidence supports only the report's scope and does not imply production / live / security certification / protocol compatibility / SDK parity / regulated workflow without separate evidence.
