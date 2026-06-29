# 第14章 ci-quality-gates

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 implementations 領域における command CI quality gate の完全仕様を固定します。具体的には、command root（working directory）、command class の閉集合（format / build / test / benchmark / real-device / readiness）とその command shape・claim boundary、command evidence の required field、CI admission の文書化必須項目、current gate state（各 gate の status と claim boundary）、command class 別 claim boundary の不可侵則、採用可能 gate scope、そして fail-closed / collapse 条件を、再現実装可能な粒度で記述します。本章は完全自己完結であり、他文書・実コードを参照せずに理解できます。同一仕様書内の他章番号のみ参照します。

依存規則として、implementations は Kernel に対して contract / SDK / command surface のみ依存します。command success は command class の claim boundary を超えません。CI success は production readiness / live readiness を自動主張しません。close-like claim は CI success のみを根拠には行いません。

## 1. Command Root（working directory）

すべての implementations command は次の working directory を root とします。

```text
implementations
```

Kernel workspace root の command success は implementations readiness の根拠にしません（禁止）。

## 2. Command Classes（閉集合）

| Class | Command shape | Claim boundary |
|---|---|---|
| format | `cargo fmt --check` | formatting のみ |
| build | `cargo build --workspace --all-targets` | build success のみ |
| test | `cargo test --workspace --all-targets` | tested behavior のみ |
| benchmark | `cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios` | measurement output のみ |
| real-device | bounded device command | command result のみ |
| readiness | readiness-specific command matrix | admit された readiness gate を伴う readiness claim のみ |

command class ごとの owner と claim boundary は次の通りです。

| Command class | Owner | Claim boundary |
|---|---|---|
| build | implementation source | build success に限定 |
| unit / integration test | test source | tested behavior に限定 |
| benchmark | benchmark harness | measurement output に限定 |
| real-device | real-device harness | command scope に限定 |
| readiness | product / operations readiness | admit された readiness gate が必要 |

command success は command class の claim boundary を超えません（禁止）。build success は test pass を意味しません。test pass は benchmark threshold satisfaction を意味しません。benchmark execution は production readiness を意味しません。real-device command success は live readiness を意味しません。

## 3. Command Evidence Required Fields

正式 command evidence は次の共通 field を持ちます。

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

`target_package`、`input_fixture_or_workload`、`exit_status` の command class 別必須性は、command class ごとに異なるため、command-class required field matrix（evidence schema）に従います。該当 matrix で required とされた field を欠く command output は正式 evidence として採用しません（禁止）。closed enum wire value / non-claim scope closed set、benchmark / real-device / readiness extension record の詳細は、本仕様書の evidence 関連章に従います。

## 4. CI Admission

CI workflow を採用する場合、先に次を文書化します（必須）。

- workflow file path
- trigger
- command class
- target scope
- command-class target package requirement
- artifact path（required artifact）
- failure classification
- rerun condition
- claim boundary

CI success は production readiness / live readiness を自動主張しません（禁止）。CI success を production readiness または live readiness に転用してはなりません（禁止）。

## 5. 採用可能 Gate Scope

implementation source とそれを規定する仕様が成立したため、format / build / test command class は採用可能です。benchmark / real-device / readiness command は、該当 source とそれを規定する仕様が存在した後に採用します。

## 6. Current Gate State（current quality gate state）

本章は implementations 側 CI / command matrix を固定する領域です。implementation source とそれを規定する仕様が成立した今、format / build / test gate は採用可能です。benchmark / real-device / readiness gate は、該当 source とそれを規定する仕様が成立した後に採用します。

current gate state は次の通りです。

| Gate | Current status | Claim boundary |
|---|---|---|
| format | 採用可能 | formatting only |
| build | 採用可能 | build success only |
| test | test source が成立した今は採用可能 | tested behavior only |
| benchmark | benchmark scenario workload 規則および benchmark harness layout 規則の下で採用 | measurement output only |
| real-device | real-device command matrix 規則および real-device wrapper command 規則の下で採用 | bounded command result only |
| readiness | 第12章で規定する readiness matrix と admit された readiness gate が必要 | bounded readiness only |

いかなる command gate も、自身の claim boundary を超えて reference / product implementation の completion、production readiness、live readiness の根拠にはしません（禁止）。

## 7. Non-Claim

CI quality gate の文書群は、CI pass、build pass、test pass、benchmark pass、production readiness、live readiness、CI workflow completion を主張しません（禁止）。

## 8. Collapse Conditions（不変条件・fail-closed 条件）

本章の正典は次の場合に崩れます。

- Kernel working directory の command result を implementations evidence として採用する。
- command class を越えて success claim を転用する（command success を command class の claim boundary 外へ転用する）。
- required field を欠く command output を正式 evidence にする。
- benchmark execution を threshold satisfaction として扱う。
- benchmark command output を threshold satisfaction として扱う。
- real-device command success を live readiness として扱う。
- Kernel working directory の command success を implementations readiness として扱う。
