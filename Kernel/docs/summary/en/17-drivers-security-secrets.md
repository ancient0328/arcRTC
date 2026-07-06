# drivers-security-secrets

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the security / secrets driver family of arcRTC v0.2 Kernel, at a granularity sufficient for re-implementation from this chapter alone. This chapter internalizes the security key source / verifier driver boundary (key fetch / cache / refresh / cryptographic backend / failure mapping), the transport security configuration (TLS/mTLS, DTLS/SRTP backend, certificate/key source, allowed cipher/profile, secret handling, startup failure), and all states and procedures of the secret rotation lifecycle, down to owners, closed-set vocabulary, state machines, failure mapping, prohibitions/permissions, and fail-closed conditions, omitting none.

Dependency direction notation: `A <- B` means "B depends on A". Token verification semantics and issuer/audience/claim policy are owned by core, and key fetch / cache / refresh / concrete crypto backend are owned by the driver. Entrypoints pass typed configuration / typed reference, and the driver implements it. arcRTC does not own token issuance / credential issuance and holds only the verification boundary of externally issued token / credential. This chapter does not claim secret manager implementation, certificate deployment, cryptographic certification, or runtime rotation / security verification success. There is no insecure fallback when a required secure mode fails (fail-closed).

---

## 1. Security Key Source / Verifier Driver Boundary

### 1.1 Boundary (ownership assignment, closed set)

Token verification semantics are defined by the token verification rule, and this section fixes the driver boundary of key fetch, cache, refresh, cryptographic backend, and failure mapping. The secret rotation lifecycle follows Section 3.

| Surface | Owner | Rule |
|---|---|---|
| verification request/result semantics | core | token verification decision and reason |
| issuer/audience/claim policy | core | abstract policy input |
| key source configuration | entrypoints | typed configuration only |
| key fetch/cache/refresh | driver | bounded external implementation |
| JWT/JWK/parser/crypto library | driver | concrete implementation only |
| token issuance | external system | arcRTC does not own |

### 1.2 Key Source Types (closed set)

The v0.2 initial key source types are as follows. A new key source type is out of the v0.2 initial scope.

| Key source | Owner | Rule |
|---|---|---|
| static typed key material | entrypoints supplies typed config; driver verifies shape | no raw secret in core |
| JWKS endpoint | driver fetch/cache | bounded HTTP and cache policy required |
| file/secret manager reference | entrypoints supplies typed reference; driver loads | core sees opaque credential/key reference only |

### 1.3 Cache / Refresh Rule (required bounds)

The key cache is driver-owned and MUST be bounded. Required bounds: maximum keys, maximum key material bytes, maximum cache age, maximum failed refresh attempts, and maximum refresh wait time. A cache miss or stale key MAY trigger a driver refresh, but a refresh failure MUST NOT silently accept the token. If a key cannot be obtained within bounds, verification fails closed with `token_key_unavailable`. Generation overlap, revocation, and stale key acceptance are governed by the secret rotation rule (Section 3).

### 1.4 Verification Flow (fixed order)

1. entrypoints wires typed verifier configuration and selected driver.
2. driver loads/fetches bounded key material.
3. driver parses token and performs concrete crypto verification.
4. driver maps crypto/key errors to core-owned verification result.
5. core applies issuer/audience/claim/time policy and emits decision.

The driver MUST NOT apply entrypoint-specific domain role authorization. Core MUST NOT perform an HTTP key fetch or a concrete crypto library call.

### 1.5 Failure Mapping (closed set)

Driver-local crypto/library errors MUST be converted to one of the following cataloged reasons.

| Failure | Required reason |
|---|---|
| token absent | `token_missing` |
| token cannot be decoded | `token_malformed` |
| signature invalid | `token_signature_invalid` |
| key source unavailable / bounded lookup failed | `token_key_unavailable` |
| issuer mismatch | `token_issuer_mismatch` |
| audience mismatch | `token_audience_mismatch` |
| token expired | `token_expired` |
| token not yet valid | `token_not_yet_valid` |
| required claim absent | `token_required_claim_missing` |
| unsupported algorithm | `token_unsupported_algorithm` |
| required secret/key source config missing | `secret_unavailable` or `runtime_config_missing` by startup phase |
| key generation revoked or outside overlap | secret rotation reason from Section 3 |

### 1.6 Secret Handling Rule

Raw key material, raw token, raw secret, and cryptographic backend error detail MUST NOT appear in an audit event, log, metric label, SDK public error, or core domain state. Privacy / redaction / retention handling follows the privacy / redaction / retention rule. Transport certificate/private-key handling follows the transport security configuration rule (Section 2) when the secret is part of transport security. Core MAY carry only an opaque credential/key reference and verification outcome. The driver MAY carry operational detail only as non-authoritative diagnostic detail that is not exposed as a canonical reason.

### 1.7 Prohibitions (prohibition, closed set)

- Core fetches JWKS or file/secret material.
- The driver accepts a token when key refresh fails.
- The driver accepts a revoked or expired generation without a rotation policy.
- The key cache is unbounded.
- Raw token/key material is persisted or logged as evidence.
- Entrypoint-specific role authorization is mixed into generic token verification.
- Token issuance is treated as an arcRTC responsibility.
- A driver error remains an open-ended string in the core decision.

### 1.8 Key Source / Verifier Collapse Conditions

Token verification can fail open on key source failure. A raw secret or token crosses into core/audit/log as authoritative data. The key source cache has no bound or no refresh failure reason. Key generation overlap/revocation behavior is implicit. The verifier driver owns domain authorization. Token issuance becomes a v0.2 communication-core responsibility.

---

## 2. Transport Security Configuration

### 2.1 Boundary (ownership assignment, closed set)

This section fixes the owners of TLS/mTLS, DTLS/SRTP backend, certificate/key source, allowed cipher/profile, secret handling, and startup failure. The secret rotation lifecycle follows Section 3; secure media session lifecycle follows the secure media session lifecycle rule; edge TLS termination and downstream security trust follow the edge/proxy trust boundary rule; internal service identity / trust follows the internal service identity trust rule.

| Concern | Owner | Rule |
|---|---|---|
| abstract security requirement | core | transport must satisfy accepted policy before sensitive path |
| accepted protocol/profile policy | core | evaluated as typed policy input |
| certificate/key source reference | entrypoints | passes a typed runtime/config reference |
| raw certificate/private key loading | driver | concrete file/secret manager/backend handling |
| TLS/mTLS/DTLS/SRTP implementation | driver | concrete library and runtime detail |
| internal service peer identity | service identity policy | peer proof is mapped before internal control authorization |
| endpoint/listener selection | entrypoints | driver wiring only |
| edge TLS termination | edge/proxy trust boundary | not backend security proof by default |
| redaction/retention | observability/reporting boundary | does not emit raw secret material |

Core does not own a raw private key, raw certificate file, secret manager client, TLS library type, or DTLS/SRTP session object.

### 2.2 Configuration Rule

Entrypoints MAY read environment variables, files, process args, deployment settings, or secret references. Entrypoints MUST convert them into typed runtime configuration and typed core policy input.

Core policy MAY include (permission): required transport security mode; accepted protocol/profile version; peer verification requirement as abstract policy; minimum verification result required before the command path is accepted; accepted media/security contract version.

Driver runtime configuration MAY include (permission): listener bind setting; certificate/key source reference; trust anchor source reference; concrete backend selection; driver-local timeout/bounds; rotation source reference and reload bounds when supported.

Driver runtime configuration MUST NOT redefine core security policy.

### 2.3 Secret Handling Rule

A raw private key, raw shared secret, raw token, and raw certificate private material MUST NOT enter: core state; audit event; log/trace; metric label; SDK public error; the evidence report body. Reports MAY include an opaque secret source reference, a certificate fingerprint/hash when policy allows it, and a redacted diagnostic summary.

### 2.4 Failure Mapping (closed set)

Transport security startup/path failure MUST fail closed. No insecure fallback is allowed unless a non-sensitive development-only composition is explicitly defined and its evidence is not used as production readiness.

| Failure | Required reason |
|---|---|
| required runtime security configuration missing | `runtime_config_missing` |
| runtime security configuration invalid | `runtime_config_invalid` |
| required secret/key source unavailable | `secret_unavailable` |
| required secret rotation state unavailable | `secret_rotation_state_unavailable` |
| transport secret/key generation revoked | `secret_key_revoked` |
| transport/media contract version unsupported | `unsupported_media_contract_version` |
| network receive failed after secure transport setup attempt | `network_receive_failed` |
| network send failed after secure transport setup attempt | `network_send_failed` |
| driver shutdown | `driver_shutdown` |

### 2.5 Verification Evidence Rule (verification evidence distinctions, closed set)

Transport security evidence MUST distinguish the following. Evidence for one class does not prove another class.

- configuration shape validation;
- secret source availability;
- listener startup;
- peer verification behavior;
- internal service identity/trust mapping when mTLS or a service credential is used for internal control;
- DTLS/SRTP or TLS session establishment;
- negative test for invalid peer or missing secret;
- redaction of secret material in logs/reports;
- rotation generation/overlap evidence when rotation affects the claim;
- secure media session evidence when a DTLS/SRTP protected media path is part of the claim;
- edge termination and downstream protection evidence when TLS terminates before the entrypoint listener.

### 2.6 Prohibitions (prohibition, closed set)

- Core reads certificate/key files.
- The driver accepts an insecure fallback after a required secure mode fails.
- A TLS/mTLS/DTLS/SRTP concrete library object crosses into core.
- A raw key/token/secret appears in audit/log/report.
- An entrypoint configuration branch changes core security semantics silently.
- Listener startup success is treated as peer verification success.
- Peer verification success is treated as internal service authorization success.
- A stale or revoked transport secret is accepted without a rotation policy.
- Edge TLS termination is treated as backend secure transport without an edge trust policy.

### 2.7 Transport Security Configuration Collapse Conditions

A raw transport secret becomes core-owned. A required secure mode can fail open to insecure mode. Driver runtime configuration overrides core security policy. Security evidence conflates listener startup with peer verification. Reports/logs expose secret material. Rotation state is omitted while claiming secure transport readiness. Secure media session readiness is claimed from transport configuration alone. The TLS termination boundary is implicit while claiming secure transport readiness. Internal service trust is inferred from TLS/mTLS configuration alone.

---

## 3. Secret Rotation Lifecycle

### 3.1 Boundary (ownership assignment, closed set)

This section fixes the rotation, overlap, revocation, and stale-key handling of JWT/JWKS, TURN shared secret, TLS certificate/private key, DTLS/SRTP material, and opaque secret references so that they do not become fail-open. Raw secret material MUST NOT enter core, audit, logs, metrics, SDK public error, or reports.

| Concern | Owner | Rule |
|---|---|---|
| secret source reference | entrypoints typed configuration | a reference, not a raw secret |
| raw secret/key material load | driver | concrete file/secret-manager/backend detail |
| verification/acceptance policy | core typed policy where semantic | accepted generation, overlap, expiry |
| rotation execution | driver/entrypoints | reload/fetch/swap implementation |
| token/key verification | service identity policy | token result and key source failure mapping |
| transport security material | transport security policy | TLS/DTLS/SRTP boundary |
| evidence/reporting | reports | redacted rotation state only |

### 3.2 Secret Classes (closed set)

The secret classes of the v0.2 initial architecture are limited to the following. A new secret class is out of the v0.2 initial scope.

| Class | Meaning | Rule |
|---|---|---|
| `jwt_verification_key` | token verification key/JWKS material | key source driver owns fetch/cache |
| `turn_shared_secret` | TURN credential derivation or HMAC secret | rotation overlap required when active credentials exist |
| `transport_certificate_key` | TLS/mTLS/DTLS/SRTP private material | transport security policy applies |
| `opaque_secret_reference` | reference to external secret manager/file/env source | entrypoints may pass typed reference |
| `ephemeral_session_key_material` | backend session material | driver-owned, no report/log exposure |

### 3.3 Rotation State Rule (rotation state machine, closed set)

The rotation lifecycle uses the following states. State names are policy/evidence classifications, not raw key identifiers.

| State | Meaning |
|---|---|
| `current_generation` | accepted for new verification/issuance use where applicable |
| `previous_generation_overlap` | accepted only for bounded verification overlap |
| `pending_generation` | loaded but not accepted for decision |
| `revoked_generation` | must not be accepted |
| `expired_generation` | lifetime ended |

### 3.4 Overlap and Revocation Rule (required defined items)

A rotation policy MUST define the following: secret class; generation reference format; maximum overlap window; revocation behavior; active credential/session relation; failure reason for stale or revoked material; audit/evidence relation; redaction rule. If rotation state cannot be determined where required, the affected secure path MUST fail closed.

### 3.5 Failure Mapping (closed set)

| Failure | Required reason |
|---|---|
| required rotation has not occurred | `secret_rotation_required` |
| presented generation is not accepted | `secret_generation_not_accepted` |
| key/secret generation is revoked | `secret_key_revoked` |
| overlap window expired | `secret_overlap_window_expired` |
| rotation state unavailable | `secret_rotation_state_unavailable` |
| secret source unavailable | `secret_unavailable` |
| runtime security configuration missing | `runtime_config_missing` |
| runtime security configuration invalid | `runtime_config_invalid` |
| token key unavailable | `token_key_unavailable` |

### 3.6 Evidence Rule

Rotation evidence MAY include an opaque secret reference, a generation reference hash/fingerprint when policy allows, rotation state, overlap window, and a redacted diagnostic summary. It MUST NOT include a raw secret, raw token, raw private key, raw certificate private material, or backend-specific secret payload.

### 3.7 Audit Rule

Secret rotation decisions use audit event type `secret_rotation_decision`. The event MUST carry secret class, opaque generation reference, startup run ID, and `CorrelationId` when the decision is command-scoped.

### 3.8 Prohibitions and Collapse Conditions

Prohibitions (closed set): a stale generation is accepted without an overlap policy; a revoked key is accepted due to cache convenience; secret rotation failure falls back to insecure mode; raw secret material is written to audit/log/report; driver-local rotation state changes core security policy silently; token issuance is treated as an arcRTC responsibility. Collapse conditions: rotation state is implicit; the overlap window is unbounded; revocation can fail open; reports expose raw secret material; current/previous/pending generation semantics differ by driver without a core contract update.

---

## 4. Chapter-wide Fail-closed Invariants

The fail-closed invariants common to all security / secrets drivers in this chapter are as follows (MUST). Token verification does not fail open on key source failure / refresh failure / rotation state unavailable; it fails closed with `token_key_unavailable` or the corresponding cataloged reason. When a required secure mode fails, no insecure fallback is allowed and it fails closed. The key cache / rotation overlap window is bounded, and a stale or revoked generation is not accepted without an overlap/rotation policy. Raw secret / token / key material / certificate private material / cryptographic backend error detail does not enter any of core state / audit event / log / metric label / SDK public error / report. arcRTC does not own token / credential issuance and holds only the verification boundary. Listener startup success / peer verification success / TLS configuration does not, respectively, prove peer verification / internal service authorization / secure media session readiness. A path that does not satisfy these invariants MUST NOT be adopted as close / complete / ready evidence.
