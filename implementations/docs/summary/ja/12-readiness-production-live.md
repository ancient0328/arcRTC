# 第12章 readiness-production-live

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 implementations 領域における readiness（production readiness と live readiness）の完全仕様を固定します。具体的には、readiness の非導出原則、production readiness と live readiness の分離、production provider admission surface（auth / persistence の分離 authority、production runtime profile、monitoring probe、drain / restore plan、security scan）、live endpoint traversal surface（live endpoint admission、public traversal admission、live monitoring probe、live drain / restore execution、product plane public endpoint branch）、readiness evidence record の拡張フィールド、failure classification、および implementations 領域の fixed goal を、再現実装可能な粒度で記述します。本章は完全自己完結であり、他文書・実コード・Kernel 文書を参照せずに理解できます。同一仕様書内の他章番号のみ参照します。

依存規則として、implementations は Kernel に対して contract / SDK / command surface のみ依存します。production readiness と live readiness は Kernel evidence から自動導出せず、それぞれ専用の readiness surface でのみ成立します。production provider admission と live endpoint traversal は implementations の product implementation と readiness surface のみを所有し、Kernel source、Kernel semantic authority、（production provider admission の場合は）live endpoint authority を所有しません。

## 1. Readiness の非導出原則

readiness は、次のいずれか単独によっては成立しません（必須）。

- source scaffold（雛形生成）
- build pass
- test pass
- benchmark result
- real-device command success

readiness は、source scaffold、build pass、test pass、benchmark result、real-device command success のいずれか単独によっては admit（採用）されません。

## 2. Readiness Claim Separation（readiness 主張の分離）

production readiness と live readiness は別 claim です。production readiness は controlled production identity provider、controlled persistence provider、production runtime profile、production monitoring probe、production drain / restore plan を成立させます。live readiness は controlled public live endpoint、public traversal admission、live monitoring probe、live rollback / drain / restore execution を成立させ、production readiness を prerequisite とします。

| Claim | 依存してよい（May depend on） | 依存してはならない（Must not depend on） |
|---|---|---|
| production readiness | product build / test evidence、deployment profile、monitoring evidence、rollback plan evidence、provider admission evidence | live endpoint availability 単独 |
| live readiness | production readiness、live endpoint evidence、public traversal evidence、monitoring evidence、rollback / drain execution evidence、shutdown drain evidence、restore evidence | Kernel completion / freeze 単独 |

production readiness success は live readiness success ではありません。live endpoint availability 単独は production readiness ではありません。Kernel completion / freeze 単独は live readiness ではありません。

## 3. Production Readiness の必須 surface

production-ready な system は次の surface を成立させます。各 surface はそれ自身の result に閉じ、より上位の readiness を推論しません。

| 必須 surface | Expected boundary |
|---|---|
| product workspace build | build command result のみ |
| product behavior test | tested product behavior のみ |
| product auth provider admission | provider boundary のみ |
| product persistence provider admission | provider boundary のみ |
| product deployment profile | deployment profile のみ |
| product monitoring | observability field availability のみ |
| rollback / drain plan | plan generation のみ |
| security secret scan | scope 内に committed secret が存在しないことのみ |

auth provider authority と persistence provider authority は別個に検証します。両者は generic な単一 provider authority を共有しません。auth provider authority が absent の場合、auth provider admission は `READINESS_NOT_ADMITTED` で fail-closed します。persistence provider authority が absent の場合、persistence provider admission は `READINESS_NOT_ADMITTED` で fail-closed します。

## 4. Live Readiness の必須 surface

live-ready な system は、production readiness を prerequisite として次の surface を成立させます。

| 必須 surface | Expected boundary |
|---|---|
| production readiness | production readiness prerequisite のみ |
| bounded live endpoint | endpoint command result のみ |
| public traversal | traversal command result のみ |
| live monitoring probe | probe command result のみ |
| live rollback / drain execution | bounded operation result のみ |
| live shutdown drain | bounded shutdown result のみ |
| live restore | bounded restore result のみ |

live endpoint authority が absent（不在）の場合、すべての live readiness surface は `READINESS_NOT_ADMITTED` で fail-closed します（必須）。live endpoint admission と public traversal admission は別 authority であり、単一 authority として扱ってはなりません（禁止）。

## 5. Readiness Evidence Record フィールド

readiness evidence record は、implementation evidence record（common 基底 record）を拡張します。拡張フィールドは次の通りです。

- `readiness_claim`
- `build_evidence_ref`
- `behavior_test_evidence_ref`
- `auth_provider_admission_ref`
- `persistence_provider_admission_ref`
- `deployment_profile_ref`
- `monitoring_probe_ref`
- `rollback_plan_ref`
- `security_scan_ref`
- `production_readiness_report_ref`
- `live_endpoint_evidence_ref`
- `public_traversal_evidence_ref`
- `rollback_drain_execution_ref`
- `shutdown_drain_evidence_ref`
- `restore_evidence_ref`

`auth_provider_admission_ref` と `persistence_provider_admission_ref` は別フィールドであり、統合してはなりません（禁止）。いずれの readiness evidence フィールドも、raw secret、raw token、private key、raw packet payload、direct personal identifier を含んではなりません（禁止）。

## 6. Failure Classification（readiness surface 共通）

| Failure | Implementation reason |
|---|---|
| readiness surface not admitted | `READINESS_NOT_ADMITTED` |
| required field missing | `EVIDENCE_FIELDS_INCOMPLETE` |
| command scope mismatch | `COMMAND_SCOPE_MISMATCH` |
| auth provider not admitted | `READINESS_NOT_ADMITTED` |
| persistence provider not admitted | `READINESS_NOT_ADMITTED` |
| live endpoint authority absent | `READINESS_NOT_ADMITTED` |

## 7. Production Provider Admission

本節は、production readiness surface の production provider admission、production profile、monitoring probe、rollback / drain plan、security scan を固定します。この admission は implementations の product implementation と readiness evidence に限定され、Kernel source、Kernel semantic authority、live endpoint authority は所有しません。

### 7.1 Authority Split（authority の分離）

auth provider authority と persistence provider authority は別 authority です。

| Authority state field | Evidence ref field | Source function |
|---|---|---|
| `ReadinessValidationContext.auth_provider_authority` | `ReadinessEvidenceRecord.auth_provider_admission_ref` | `admit_product_auth_provider` |
| `ReadinessValidationContext.persistence_provider_authority` | `ReadinessEvidenceRecord.persistence_provider_admission_ref` | `admit_product_persistence_provider` |

`auth_provider_authority == Absent` の場合、auth provider admission は `AuthProviderAuthorityNotAdmitted` で fail-closed します。`persistence_provider_authority == Absent` の場合、persistence provider admission は `PersistenceProviderAuthorityNotAdmitted` で fail-closed します。単一の generic provider authority では、両者のうち片方だけが未採用である fail-closed 状態を表現できません。

### 7.2 Auth Provider Admission（source contract）

`product-implementation/product-policy/src/provider_admission.rs` は次を定義します。

| Item | Required shape |
|---|---|
| `ProductAuthProviderClass` | enum `NotAdmitted`, `ControlledProductionIdentity` |
| `ProductAuthProviderAdmission` | `provider_class`, `credential_source`, `token_validation_rule`, `secret_storage_boundary`, `failure_reason_closed_set`, `audit_evidence_boundary`, `implementation_reason` |
| `admit_product_auth_provider` | `(ProductAuthProviderClass) -> Result<ProductAuthProviderAdmission, ProductPolicyError>` |

`ControlledProductionIdentity` だけが admitted authority です。`NotAdmitted` は `ProductPolicyError::ReadinessNotAdmitted` を返します。

### 7.3 Persistence Provider Admission（source contract）

`product-implementation/persistence-topology/src/provider_admission.rs` は次を定義します。

| Item | Required shape |
|---|---|
| `ProductPersistenceProviderClass` | enum `NotAdmitted`, `ControlledProjectionStore` |
| `ProductPersistenceProviderAdmission` | `provider_class`, `record_shape_boundary`, `storage_scope`, `schema_owner`, `failure_reason_closed_set`, `implementation_reason` |
| `admit_product_persistence_provider` | `(ProductPersistenceProviderClass) -> Result<ProductPersistenceProviderAdmission, ProductPersistenceTopologyError>` |

`ControlledProjectionStore` だけが admitted authority です。`NotAdmitted` は `ProductPersistenceTopologyError::ProviderNotAdmitted` を返します。

### 7.4 Production Runtime Profile（source contract）

`product-implementation/deployment/src/production_profile.rs` は次を定義します。

| Item | Required shape |
|---|---|
| `build_product_production_profile` | `() -> ProductRuntimeProfile` |

`ProductHostClass` は `ProductionAdmitted` を持ちます。`build_product_production_profile()` は `ProductHostClass::ProductionAdmitted`、`ImplementationEnvironmentClass::ProductionDeferred`、`public_endpoint_claimed = false` を返します。`select_product_runtime` は `ProductionAdmitted` profile を `ImplementationOk` として扱います。production profile は public endpoint availability を主張してはなりません（禁止）。

### 7.5 Production Monitoring Probe（source contract）

`product-implementation/monitoring/src/production_probe.rs` は次を定義します。

| Item | Required shape |
|---|---|
| `ProductProductionMonitoringProbe` | `correlation_id`, `target_plane`, `metric_name`, `probe_boundary`, `implementation_reason` |
| `build_production_monitoring_probe` | `(CorrelationId, ImplementationPlane, &'static str) -> Result<ProductProductionMonitoringProbe, ProductMonitoringError>` |

empty な metric name は `ProductMonitoringError::EvidenceFieldsIncomplete` を返します。

### 7.6 Production Operation Plan（source contract）

`product-implementation/rollback/src/production_operation.rs` は次を定義します。

| Item | Required shape |
|---|---|
| `plan_production_drain` | `(CorrelationId, Vec<ImplementationPlane>) -> Result<ProductDrainPlan, ProductRollbackError>` |
| `plan_production_restore` | `(CorrelationId) -> Result<ProductRestorePlan, ProductRollbackError>` |

empty plane の drain は `ProductRollbackError::RuntimeExecutorError` を返します。restore source は `ProductRestoreSource::EvidenceReport` に固定します。production drain / restore plan は live operation execution に転用してはなりません（禁止）。

### 7.7 Production Source Admission（owner package 限定）

production readiness の source は次の owner に限定します。

| Source | Owner package | Purpose |
|---|---|---|
| `provider_admission.rs` | `arcrtc-product-policy` | auth provider admission |
| `provider_admission.rs` | `arcrtc-product-persistence-topology` | persistence provider admission |
| `production_profile.rs` | `arcrtc-product-deployment` | production runtime profile |
| `production_probe.rs` | `arcrtc-product-monitoring` | production monitoring probe |
| `production_operation.rs` | `arcrtc-product-rollback` | production drain / restore plan |

既存ファイルは module registration、enum admission、delegation のためにのみ更新できます。Kernel source の modification は禁止です。

### 7.8 Production Provider Admission Failure Classification

| Failure | Classification |
|---|---|
| auth provider authority absent | `READINESS_NOT_ADMITTED` |
| persistence provider authority absent | `READINESS_NOT_ADMITTED` |
| missing required ref | `EVIDENCE_FIELDS_INCOMPLETE` |
| empty required ref | `EVIDENCE_FIELDS_INCOMPLETE` |
| unexpected ref populated | `COMMAND_SCOPE_MISMATCH` |
| secret-like marker in ref | `EVIDENCE_FIELDS_INCOMPLETE` |

### 7.9 Production Provider Admission Non-Claim Scope

production provider admission は次を主張しません（禁止）。live readiness、public endpoint availability、Kernel completion、Kernel freeze。

## 8. Live Endpoint Traversal

本節は、live readiness surface の live endpoint admission、public traversal admission、live monitoring probe、live drain / restore operation、product plane public endpoint branch を固定します。本節は Kernel source、Kernel semantic authority、Kernel completion / freeze evidence を所有しません。production readiness は live readiness の prerequisite であり、live endpoint、public traversal、live drain / restore、live readiness を代替しません。

### 8.1 Deployment Source Contract

`product-implementation/deployment/src/live_endpoint.rs` は次を所有します。

- `ProductLiveEndpointClass`
- `ProductPublicTraversalClass`
- `ProductLiveEndpointAdmission`
- `ProductPublicTraversalAdmission`
- `admit_product_live_endpoint`
- `admit_public_traversal`
- `build_product_live_profile`

`ProductLiveEndpointClass` は閉集合です。

- `NotAdmitted`
- `ControlledPublicEndpoint`

`ProductPublicTraversalClass` は閉集合です。

- `NotAdmitted`
- `ControlledPublicTraversal`

`admit_product_live_endpoint(ProductLiveEndpointClass::ControlledPublicEndpoint)` は `ImplementationOk` を返します。`admit_product_live_endpoint(ProductLiveEndpointClass::NotAdmitted)` は `ProductRuntimeError::ReadinessNotAdmitted` を返します。

`admit_public_traversal(ProductPublicTraversalClass::ControlledPublicTraversal)` は `ImplementationOk` を返します。`admit_public_traversal(ProductPublicTraversalClass::NotAdmitted)` は `ProductRuntimeError::ReadinessNotAdmitted` を返します。

`build_product_live_profile(&ProductLiveEndpointAdmission)` は次を返します。

- `ProductHostClass::LiveAdmitted`
- `ImplementationEnvironmentClass::LiveDeferred`
- `public_endpoint_claimed == true`

`select_product_runtime` は次の場合に限り public endpoint claim を受理します。

- `host_class == ProductHostClass::LiveAdmitted`
- `environment_class == ImplementationEnvironmentClass::LiveDeferred`
- `public_endpoint_claimed == true`

これ以外のすべての `public_endpoint_claimed == true` の組み合わせは `ReadinessNotAdmitted` で fail-closed します（必須）。`ProductHostClass::LiveAdmitted` は `ProductLiveEndpointAdmission` から生成される live profile に限定して使用します。

### 8.2 Monitoring Source Contract

`product-implementation/monitoring/src/live_probe.rs` は次を所有します。

- `ProductLiveMonitoringProbe`
- `build_live_monitoring_probe`

`build_live_monitoring_probe` は非 empty な `metric_name` を必須とし、empty metric に対しては `ProductMonitoringError::EvidenceFieldsIncomplete` を返します。成功した live monitoring probe は `ImplementationOk` を使用します。

### 8.3 Rollback Source Contract

`product-implementation/rollback/src/live_operation.rs` は次を所有します。

- `execute_live_shutdown_drain`
- `execute_live_restore`

`execute_live_shutdown_drain` は非 empty な plane list を必須とし、empty list に対しては `ProductRollbackError::RuntimeExecutorError` を返します。成功した live shutdown drain は `ImplementationOk` を使用します。`execute_live_restore` は correlation id を伴って呼び出された場合に `ImplementationOk` を返します。live operation source は rollback owner package に置き、deployment / monitoring / plane package に混在させません（禁止）。

### 8.4 Product Plane Public Endpoint Branch

product plane の owner ファイルは public endpoint branch 関数を所有します。

| package | function | branch input |
|---|---|---|
| `product-implementation/signaling` | `build_live_product_signaling_runtime` | admitted live endpoint evidence ref |
| `product-implementation/turn` | `build_live_product_turn_runtime` | admitted live endpoint evidence ref |
| `product-implementation/sfu` | `build_live_product_sfu_runtime` | admitted live endpoint evidence ref |

各関数は、empty または空白のみの live endpoint evidence ref を `ReadinessNotAdmitted` で拒否します。各成功関数は plane 固有の public endpoint flag を `true` に設定し、`ImplementationOk` を使用します。既存の local runtime builder は public endpoint flag を `false` のまま維持します。Signaling / TURN / SFU public endpoint branch は admitted live endpoint evidence ref なしに有効化してはなりません（禁止）。

### 8.5 Live Endpoint Traversal Non-Claim Scope

この live endpoint traversal は次を主張しません（禁止）。live readiness 単独、Kernel completion、Kernel freeze。

## 9. implementations 領域の Fixed Goal

implementations の fixed goal は、production readiness、live readiness、benchmark threshold satisfaction、real-device success、Kernel completion / freeze を完全に全て達成させることです。この fixed goal は、現実的な落としどころ、解釈の再編、限定 claim への縮小を採用しません（禁止）。凍結済み Kernel は implementations 側の Kernel source mutation なしに消費し、Kernel contract の不足は fail-closed として Kernel 側 versioned authority を先に必要とします。

fixed goal は次の独立した evidence surface を接続します。

- implementations 領域は admitted Kernel crate のみを利用し、Kernel source mutation を行わない。
- reference implementation は Signaling / TURN / SFU / composition behavior を Kernel contract 経由で実行する。
- product implementation は product Signaling / TURN / SFU / policy / persistence / deployment / monitoring / rollback surface を Kernel semantics を所有せずに実行する。
- local controlled profile は start、bounded command flow 実行、shutdown、invalid lifecycle order の拒否ができる。
- benchmark surface は actual workload runner を実行し measurement record を emit し、benchmark threshold satisfaction は pre-adopted threshold value と comparison rule で判定する（第10章）。
- real-device success は、wrapper-only fixture success ではなく、required device class set 全体にわたる bounded device-class command execution evidence で証明される（第11章）。
- production readiness はその必須 surface を成立させ、benchmark result、product behavior evidence、real-device result、Kernel evidence 単独ではなく、専用の production readiness surface でのみ成立する。
- live readiness は production readiness を prerequisite としてその必須 surface を成立させ、production readiness 単独、benchmark result、real-device fixture、Kernel evidence 単独ではなく、専用の live readiness surface でのみ成立する。
- Kernel completion / freeze は implementations 側の Kernel source mutation なしに evidence 化される。

本 fixed goal が許可する唯一の final claim は次です。

```text
implementations satisfies production readiness, live readiness, benchmark threshold satisfaction, real-device success, and Kernel completion / freeze, while consuming the frozen Kernel without implementations-side Kernel mutation.
```

## 10. Collapse Conditions（不変条件・fail-closed 条件）

本章の正典は次の場合に崩れます。

- product test pass を production readiness として使う。
- production readiness を live evidence なしに live readiness として使う。
- benchmark または real-device command result をそれ単独で readiness として使う。
- provider-deferred state を admitted provider state として扱う。
- readiness evidence に secret、token、private key、raw packet payload を含める。
- auth provider admission と persistence provider admission を単一 generic provider authority として扱う。
- `auth_provider_admission_ref` と `persistence_provider_admission_ref` を統合する。
- `NotAdmitted` provider class を success として扱う。
- production profile が public endpoint availability を主張する。
- monitoring probe success を live readiness に転用する。
- production drain / restore plan を live operation execution に転用する。
- security scan を実行せず production readiness を主張する。
- production readiness success を live readiness success として扱う。
- live endpoint admission と public traversal admission を単一 authority として扱う。
- live endpoint evidence ref なしに Signaling / TURN / SFU public endpoint branch を有効化する。
- live operation source を deployment、monitoring、Signaling、TURN、SFU owner package に混在させる。
- Kernel source modification を implementations readiness surface の充足に使う。
