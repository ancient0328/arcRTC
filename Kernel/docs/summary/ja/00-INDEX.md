# arcRTC v0.2 Kernel システム／開発サマリー（日本語版）

状態: public summary projection
日付: 2026-07-06 JST
対象: `Kernel/`

## このサマリーの位置づけ

本サマリー（`docs/summary/ja/` 配下の全章）は、arcRTC v0.2 Kernel の構造と evidence scope を公開向けに投影した文書です。
本サマリーは、v0.2 Kernel の意図・目的・理念・構造・経路・性質・運用・検証・トラブルシューティングを確認する入口として用います。

本サマリーは「作成経過の歴史書」ではありません。また、現在の `dev-docs/` authority や source-level verification の代替でもありません。completion、evidence、closure claim は `dev-docs/` と対応 report によって扱います。

Kernel の実装可能性（これらの契約が実通信経路へ構成可能であること）は、Kernel tree 内の Kernel-owned drivers、entrypoints、実 socket/datagram 交換、`dev-docs/90-reports/` 配下の report によって評価します。Kernel 外 distro は Kernel contract を消費できますが、Kernel completion evidence は供給しません。

### Summary 原則（全章共通）

- 本サマリーは orientation surface です。`dev-docs/`、source code、tests、reports を上書きしません。
- 章どうしの相互参照は、同一 summary 内のナビゲーション（章番号）に限ります。各章は review orientation に必要な範囲で自己完結を保ちます。
- 規範語: 「必須（MUST）」「禁止（MUST NOT）」「許可（MAY）」「fail-closed（曖昧・閉集合外は不採用＝拒否側に倒す）」を一貫して用います。
- 用語: 技術固有名詞（core / drivers / entrypoints / sdk / regulated / port / Sans-IO / SFU / TURN / Signaling 等）は英語表記のまま保持し、説明は日本語丁寧語で記します。

### 依存方向の記法

`A <- B` は「B が A に依存する（B から A を参照してよい）」を表します。本系の正式な構造軸は次です。

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints

sdk:       独立した Signaling-only 境界
regulated: optional domain support（独立）
```

禁止される依存方向（一部でも発生したら境界違反）:

```text
core -> drivers / core -> entrypoints / core -> regulated
drivers -> entrypoints / drivers -> regulated
entrypoints -> regulated / sdk -> regulated
```

`regulated -> core` のみ、ADR で明示された opaque communication event・audit pointer・non-sensitive tag の参照に限り許可し得ます。

## 章構成

| 章 | 内容 |
|---|---|
| 00-INDEX | 本索引・summary 原則・用語・記法 |
| 01-overview-and-scope | Mission、価値、Kernel の定義、System Boundary、Non-goals、用語集 |
| 02-architecture-and-boundaries | 層モデル、依存方向、crate/package 境界、semantic modular monolith、source shard |
| 03-core-domain-and-identity | DDD domain model、aggregate、use case、identity/reference、cross-plane identity/session binding |
| 04-core-command-and-reason | command/decision/event/result shape、idempotency/replay/correlation、reason catalog、external error mapping |
| 05-core-ports | core ports、port contract shape |
| 06-core-protocol-and-serialization | deterministic encoding、protocol versioning、compatibility/deprecation、wire envelope、feature flag/capability lifecycle |
| 07-core-transport-and-media | core transport contract (Sans-IO)、SDP/ICE negotiation、ICE candidate lifecycle、media codec/track/layer、packet rewrite/transform、secure media session lifecycle |
| 08-core-signaling-plane | Signaling contract、Signaling state machine |
| 09-core-sfu-plane | SFU contract、SFU state machine、packet semantic view、packet buffer lifecycle、congestion/pacing/retransmission |
| 10-core-turn-plane | TURN contract、TURN lifecycle |
| 11-core-runtime-time-concurrency | runtime/clock/randomness、task/worker lifecycle、retry/timeout/cancellation、concurrency/ordering/lock、atomicity/transaction/compensation、unit/measurement、time sync/clock skew、resource bounds/backpressure |
| 12-core-security-and-audit | token verification、authorization context/policy、audit event、audit hash-chain |
| 13-core-quality-and-admission | quality metrics、rate limit/quota/admission |
| 14-drivers-transport-network | driver conversion、network I/O、transport driver、browser/native driver、TURN wire driver |
| 15-drivers-persistence-state | persistence、state persistence policy、durable recovery/restore/replay、schema/migration、export/backup、distributed state/replication/failover |
| 16-drivers-observability-privacy | observability boundary、signal taxonomy、privacy/redaction/retention |
| 17-drivers-security-secrets | security key source/verifier driver、transport security configuration、secret rotation lifecycle |
| 18-entrypoints-composition-config | composition root、configuration boundary、profile/policy bundle、runtime reconfiguration/hot-swap |
| 19-entrypoints-endpoints-edge | public endpoint/connection lifecycle、edge/proxy trust boundary |
| 20-entrypoints-operations-lifecycle | health/readiness/liveness/admin/maintenance、cross-plane shutdown/drain、crash/panic/supervisor restart、operator/admin authorization |
| 21-entrypoints-topology-control-plane | deployment topology/service boundary、internal control-plane contract、internal service identity/trust、service discovery/endpoint resolution |
| 22-sdk | SDK Signaling-only、platform parity、reconnect/session resumption、public API contract generation |
| 23-regulated | regulated boundary、enrichment lifecycle、out-of-scope feature admission |
| 24-packaging-supplychain-release | crate/package boundary（補遺）、supply chain/dependency/license、release artifact/distribution/provenance |
| 25-ci-quality-gates | CI quality gate、CI/testing command matrix |
| 26-testing-and-evidence | testing evidence、code coverage の閾値と分母、test double/fixture boundary、evidence record の field |
| 27-benchmark | benchmark scope、benchmark scenario matrix |
| 28-troubleshooting | 横断的な失敗モード、fail-closed 運用、境界違反の検知と是正 |

## 現在の受容範囲（claim / non-claim）

本サマリーが投影する受容範囲は次のとおりです。

- claim（現在の `dev-docs/90-reports/` evidence に裏付けられる範囲でのみ採用）: core / drivers / entrypoints / sdk / regulated の境界固定、Signaling/SFU/TURN semantics の core 配置、明示的に report 済みの scope における Kernel-owned runtime evidence。
- non-claim（不主張）: production readiness、live readiness、native application readiness、distro（Kernel 外）の reference/product 完成、benchmark acceptance threshold 充足、v0.1 挙動の継承。

## 英語版

同一構造の英語版は `docs/summary/en/` 配下にあります（章番号・章名は対応）。
