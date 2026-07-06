# トラブルシューティング

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 **distro**（Kernel 外実装領域）における横断的なトラブルシューティングを、「症状 → 疑うべき境界 / 原因 → fail-closed 既定挙動 → 是正手順」の形式で自己完結的に固定します。distro は凍結済み Kernel contract を変更せず、reference / product distro と benchmark / real-device / readiness evidence surface を構築する領域であり、依存規則は `distro -> Kernel` の一方向です。本章は error / reason mapping、readiness matrix、Kernel contract import、command / CI quality gate の各境界に内在する fail-closed 挙動と是正を扱います。

代表的な境界違反として、Kernel contract 変更要求、3-type allow-list 逸脱、reference を product に fork、測定後の benchmark threshold 後付け、readiness の Kernel evidence 流用、専用の readiness surface なしの readiness 主張を含み、それぞれの検知と是正を後段の表で示します。

## reason / error mapping のトラブルシューティング

distro-local error enum は Kernel reason を追加・変更せず、evidence field の `distro_reason` に閉じます。各 error enum は `Unknown` / `Other` / raw `String` variant を持たず、raw dependency error を public variant にせず、`distro_reason(&self) -> arcrtc_distro_evidence::DistroEvidenceReason` を持ちます。`Display` は diagnostic text に限定し claim reason として採用しません。`DistroEvidenceReason` は `distro-support/evidence/src/reason.rs` が唯一の canonical Rust enum を持ち、`arcrtc-reference-ops/src/reason.rs` と `arcrtc-product-monitoring/src/evidence.rs` は再定義せず利用または re-export します。

| 症状 | 疑うべき境界 / 原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| error 出力に `UNKNOWN` / `Unknown` / `Other` / raw `String` reason が現れる | error enum が closed set を逸脱、または diagnostic を claim reason に転用 | `UNKNOWN` reason を含む output は正式 evidence に採用せず diagnostic 扱い | 該当 variant を closed set の `DistroEvidenceReason`（例: `KERNEL_CONTRACT_UNAVAILABLE`、`KERNEL_CONTRACT_MISMATCH`、`FIXTURE_IDENTITY_INVALID`、`STATE_BOUNDARY_VIOLATION`、`EVIDENCE_FIELDS_INCOMPLETE`、`RUNTIME_EXECUTOR_ERROR`、`COMMAND_SCOPE_MISMATCH`、`READINESS_NOT_ADMITTED`）へ mapping し、`Unknown` / `Other` / raw `String` variant を除去する |
| dependency の raw error が evidence reason として現れる | raw dependency error を public variant 化、または evidence reason へ昇格 | raw dependency error を含む reason mapping は採用しない | error mapper で Kernel reason / dependency error を distro-facing reason へ投影し、raw error を closed set variant に変換する |
| product error が Kernel reason catalog に昇格している | product error を Kernel semantic authority に混入 | Kernel reason catalog 変更を含む実装は fail-closed | product error variant（例: `ProductPolicyError::SecurityReasonMappingFailed` → `STATE_BOUNDARY_VIOLATION`）を `DistroEvidenceReason` へ閉じ込め、Kernel reason catalog を変更しない |
| `ReadinessNotAdmitted` を success として扱っている | readiness failure を success に転用 | `ReadinessNotAdmitted` は `READINESS_NOT_ADMITTED` reason として fail-closed | readiness failure を `READINESS_NOT_ADMITTED` として記録し、success path から除外する |
| package-local に `DistroEvidenceReason` enum が再定義されている | reason ownership 違反 | 再定義された reason set は variant 不一致で崩壊扱い | `distro-support/evidence/src/reason.rs` の canonical enum を利用または re-export し、package-local 再定義を削除する。variant set は observability の closed set と一致させる |
| mapping されない error variant が追加されている | error enum 拡張時の mapping 漏れ | mapping 欠落 variant は崩壊条件 | 追加 variant に対応する `DistroEvidenceReason` mapping を error / reason mapping 表へ追加する |

## readiness のトラブルシューティング

Readiness は source scaffold、build pass、test pass、benchmark result、real-device command success だけでは成立しません。production readiness と live readiness は別 claim であり、それぞれ専用の readiness surface でのみ成立します。`auth_provider_admission_ref` と `persistence_provider_admission_ref` は別フィールドであり統合してはなりません。readiness evidence field は raw secret / raw token / private key / raw packet payload / direct personal identifier を含んではなりません。

production readiness surface は、product workspace build、product behavior test、product auth provider admission、product persistence provider admission、product deployment profile、product monitoring、rollback / drain plan、security secret scan です。live readiness surface は、production readiness（prerequisite）、bounded live endpoint、public traversal、live monitoring probe、live rollback / drain execution、live shutdown drain、live restore です。第12章が readiness surface・source contract・failure classification の全体を固定します。

| 症状 | 疑うべき境界 / 原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| product test pass を production readiness として扱おうとしている | readiness を test pass で代替 | 専用の readiness surface なしの production readiness は崩壊条件 | production readiness surface をすべて成立させてから production-ready とみなす |
| production readiness を live readiness として扱っている | production を live の代替とする | live evidence なしの live readiness は崩壊条件 | production readiness を prerequisite とし、live readiness surface を別途成立させる |
| auth / persistence provider admission が provider 未採用で進行する | auth / persistence provider authority が absent | provider authority absent 時は `READINESS_NOT_ADMITTED` で fail closed | auth provider authority と persistence provider authority を別々に採用し、`auth_provider_admission_ref` と `persistence_provider_admission_ref` を分離して埋める |
| auth provider admission と persistence provider admission が同一の generic provider authority を共有している | provider admission 未分離 | 片方未採用の fail-closed を表現できず崩壊 | auth provider と persistence provider を別 authority・別 source owner（`product-distro/product-policy/` と `product-distro/persistence-topology/`）・別 ref field で扱う |
| live endpoint authority が absent のまま live admission を進める | live endpoint authority absent | 全 live readiness surface が `READINESS_NOT_ADMITTED` で fail closed | `ProductLiveEndpointAdmission` 由来の live profile からのみ `ProductHostClass::LiveAdmitted` を生成する |
| benchmark / real-device command result を readiness として採用している | command result を readiness に転用 | benchmark / real-device result 単独の readiness は崩壊条件 | benchmark threshold satisfaction と real-device success を独立 evidence として扱い、readiness surface とは分離する |
| provider-deferred state を admitted provider state として扱っている | deferred を admitted に昇格 | deferred state は admitted ではなく崩壊条件 | provider admission が実際に admitted authority を返すまで readiness を進めない |
| readiness evidence に secret / token / private key / raw packet payload が含まれる | redaction 違反 | secret を含む readiness evidence は崩壊条件 | readiness evidence field から raw secret / token / private key / raw packet payload / direct personal identifier を除去する |
| `auth_provider_admission_ref` と `persistence_provider_admission_ref` が統合されている | provider ref field 未分離 | 統合された provider ref では片方未採用の fail-closed を表現できない | auth provider ref と persistence provider ref を別フィールドのまま維持する |
| required field が欠けた readiness evidence を採用しようとする | required field 欠落 | `EVIDENCE_FIELDS_INCOMPLETE` で fail closed | readiness surface の欠落 required field を補う |

## Kernel contract import / consumption のトラブルシューティング

distro は Kernel public contract、Kernel SDK projection、core-owned port definition、documented command surface、Kernel report references だけを利用できます。Kernel crate は Kernel local path dependency matrix に列挙された package だけを import できます。distro は Kernel core semantics、Kernel reason catalog ownership、Kernel port definition ownership、Kernel dependency direction、Kernel completion / freeze claim authority を import / overwrite / duplicate してはなりません。

| 症状 | 疑うべき境界 / 原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| 実装中に Kernel contract の不足が判明する | Kernel contract gap | (1) distro 側 task を停止、(2) 不足を記録、(3) Kernel 側 versioned authority を要求、(4) 迂回・複製・拡張しない | Kernel 側で versioned authority を先に整備し、distro 側では代替型を作らない |
| dependency matrix にない Kernel crate を import している | import path rule 違反 | matrix 外 Kernel crate import は崩壊条件 | 追加 import が必要なら source 変更前に dependency matrix を更新し、Kernel semantic authority を侵食しない理由を report に記録する |
| distro 側で Kernel contract を複製 / 拡張 / 迂回している | consumption boundary 違反 | semantic authority 侵食は崩壊条件 | wrapper / mapper を Kernel contract の変換 / 呼び出しだけに限定し、Kernel semantics の新規定義を除去する |
| Kernel evidence を distro completion / readiness proof として採用している | evidence 流用 | Kernel evidence 流用は崩壊条件 | distro の readiness は専用の readiness surface でのみ成立させ、Kernel evidence を proof に転用しない |
| Kernel source modification を通常の task として扱っている | Kernel immutability 違反 | distro scope の Kernel source 変更は崩壊条件 | Kernel source は変更せず、Kernel contract gap は fail-closed として Kernel 側 authority を先に要求する |

## command / CI quality gate のトラブルシューティング

すべての distro command は working directory root を `distro` とします。Kernel workspace root の command success は distro readiness の根拠にしません。command class は format、build、test、benchmark、real-device、readiness で、各 class の claim boundary を超えて success を転用できません。正式 command evidence は correlation id、command、working directory、target scope、command class、toolchain / runtime version、expected outcome、actual outcome、environment class、distro reason、non-claim scope、rerun condition を持ちます。

| 症状 | 疑うべき境界 / 原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| Kernel working directory の command 結果を distro evidence にしている | working directory root 違反 | Kernel root command 結果の distro 採用は崩壊条件 | command を `distro` から実行し、その evidence だけを採用する |
| build success を test pass、test pass を threshold satisfaction として扱う等の転用 | command class boundary 超え | claim boundary 外への転用は崩壊条件 | command class ごとの claim boundary を守り、build / test / benchmark / real-device / readiness の success を相互代替しない |
| required fields を欠く command output を正式 evidence にしている | evidence required fields 欠落 | required field を欠く output は正式 evidence に不採用 | 第05章の command-class required field matrix（`target_package` / `input_fixture_or_workload` / `exit_status` の class 別必須性）に従い、欠落 field を補う |
| benchmark execution を threshold satisfaction として扱っている | benchmark を threshold に昇格 | benchmark execution の threshold 扱いは崩壊条件 | benchmark execution を measurement output に限定し、threshold satisfaction は pre-adopted threshold value と comparison rule（第10章）でのみ判定する |
| real-device command success を live readiness として扱っている | real-device を readiness に昇格 | real-device command success の live readiness 扱いは崩壊条件 | real-device command success を command scope に限定し、live readiness は専用の live readiness surface でのみ成立させる |
| CI success を production / live readiness として転用している | CI admission boundary 違反 | CI success の readiness 転用は崩壊条件 | CI workflow の claim boundary を docs / build / test 等の command class に限定し、readiness を CI で自動主張しない |

## 代表的な境界違反と検知・是正

distro 全体で頻出する境界違反を横断的にまとめます。いずれも fail-closed を既定とします。

| 境界違反 | 検知 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| Kernel contract 変更要求 | 実装中に Kernel contract gap が判明し、迂回・複製・拡張・上書きの誘惑が生じる | distro task を停止し、Kernel 側 versioned authority を先に要求 | Kernel source を変更せず、Kernel 側で contract を整備してから distro を再開する |
| 3-type allow-list 逸脱 | product plane が `arcrtc-reference-output` の `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` 以外（`ReferenceCompositionOutcome`、reference state / function / fixture / local auth / runtime / composition symbol、wildcard / module / re-export / type alias import）を消費 | allow-list 外入力は boundary test で検知され崩壊条件 | product 入力を exact 3 型に限定し、それ以外の reference symbol import を除去する |
| reference を product に fork | product distro が reference を fork / copy して product policy を混入 | reference success の product 継承は崩壊条件 | product は 3 outcome 型だけを入力として消費し、reference の fork / copy を削除する |
| benchmark threshold の後付け | benchmark 測定後に threshold value を結果へ合わせて設定 | post-measurement threshold mutation は崩壊条件 | threshold を verdict measurement より前に採用し、`measured_p95_ns <= threshold_value_ns` で判定する（第10章） |
| readiness の Kernel evidence 流用 | Kernel completion / freeze evidence を production / live readiness の proof に転用 | Kernel evidence 流用は崩壊条件 | readiness は専用の production / live readiness surface でのみ成立させ、Kernel evidence は Kernel completion / freeze binding に限定する |
| real-device success の不足 row 採用 | six required device class（Android physical / emulator、iOS physical / simulator、desktop / mobile browser）のいずれかを省略、または exit `0` 以外 / required field 欠落の row を success とする | 不足 class / nonzero exit / field 欠落 / redaction 崩壊は fail-closed | 6 row すべてを exit `0`、actual outcome、runtime version class、network class、toolchain / runtime version、`redacted` または `sha256:<64 lowercase hex characters>` の redacted identifier 付きとし、browser row で Android / iOS row を代替しない |
