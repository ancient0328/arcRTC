# トラブルシューティング（横断）

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 Kernel の横断的なトラブルシューティングを、実務的な「症状 → 疑うべき境界／原因 → fail-closed 既定挙動 → 是正手順」の表として自己完結で内在化します。本章だけを読めば、代表的な境界違反（依存方向違反、closed 語彙外、driver/entrypoint が semantics を定義、claim 転用）の検知・是正、external error mapping の落とし穴、crash/panic 分類ごとの対応、CI quality gate の fail-closed 既定挙動を把握できます。

依存方向の記法 `A <- B` は「B が A に依存する（B から A を参照してよい）」を表します。許可方向は `core <- drivers`、`core <- entrypoints`、`drivers <- entrypoints`、条件付き `core <- regulated`。

**fail-closed の原則。** 曖昧・閉集合外・unknown は採用せず、拒否側（No／reject／not-ready）に倒します。`UNKNOWN` は成功 state ではありません。close / complete / ready は未実施の検証範囲に対して主張しません。reason は閉集合であり、閉集合外（free-text や invented reason）は decision・audit・SDK mapping の正として不採用です。

## 1. 境界違反（依存方向・所有違反）

| 症状 | 疑うべき境界／原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| core が `axum`／`tokio::net`／`str0m` concrete／`sqlx`／browser·native SDK 型を import している | `core -> drivers` の依存方向違反。core が concrete I/O·runtime·DB·cloud·browser·native 型を参照 | architecture dependency gate が fail。close/complete/ready を主張させない | 外部型を driver 境界で core-owned type に変換する。core からは verification result・decision rule・borrowed view のみを参照する。port は core が所有し driver が実装する形に戻す |
| driver crate が port trait を定義している | port ownership 違反（driver が port interface を所有） | dependency boundary gate が fail（entrypoints/drivers が port trait を定義しないこと） | port trait を core に戻す。driver は core-owned port の実装に限定する |
| WebSocket handler／worker loop／socket read loop 内に accept·reject·routing·allocation decision の正がある | driver/entrypoint が core semantics を所有（Signaling state transition・SFU routing・TURN allocation の正の流出） | architecture dependency gate と source-shape test が fail | 当該 decision の正を `core/signaling`・`core/sfu`・`core/turn` へ戻す。driver は byte I/O・conversion・export に、entrypoint は wiring・lifecycle に限定する |
| `drivers -> entrypoints` の参照が発生 | drivers が entrypoints に依存（許可方向の逆） | dependency boundary gate が fail | entrypoints が drivers を compose する形に直す。drivers から entrypoints を参照しない |
| `core/drivers/entrypoints/sdk -> regulated` が発生、または `regulated -> sdk/drivers/entrypoints` | regulated independent boundary 違反 | dependency boundary gate が fail（sdk が regulated に依存しない／regulated が drivers·entrypoints·sdk に依存しない） | regulated を optional domain support の独立境界に戻す。許可方向は `regulated -> core`（opaque communication event・audit pointer・non-sensitive tag への参照に限定）のみ |
| SDK API が media／auth issuance／regulated workflow／PeerConnection を公開している | SDK Signaling-only boundary 違反 | contract test gate（SDK public contract）が fail | SDK を Signaling-only（command/event/connection lifecycle）に戻す。media・auth issuance・regulated・PeerConnection を SDK から除去する |
| entrypoint が domain rule／protocol semantics／product readiness を主張 | entrypoint reclassification 違反（entrypoint を product server と誤読） | entrypoint による product/readiness claim を拒否する。entrypoint は executable contract / composition evidence surface のみ | entrypoint を executable contract / composition evidence surface に戻す。product readiness の主張を除去する |
| 600 行超過を理由に source-shape test が fail している | source shard / semantic modular monolith の誤適用（line count を semantic boundary より上位の hard gate にした） | 600 行は review signal であり hard gate ではない。semantic owner·dependency·forbidden import の崩壊のみが fail 条件 | line count 単独では fail させない。semantic owner transfer・dependency inversion・forbidden import・hidden ownership が伴う場合のみ source-shape failure とする |

## 2. closed reason 語彙の逸脱

reason は閉集合です。reason は `category`（core 所有、closed set）、`code`（category 内 closed set）、`retryable`／`safe_to_expose`／`audit_required`（core 所有）、`details`（driver/entrypoint の optional・non-authoritative）で構成します。cross-cutting category は `malformed_input`、`unsupported_version`、`unauthorized`、`forbidden_state`、`duplicate`、`ordering_violation`、`expired`、`resource_exhausted`、`backpressure`、`quality_violation`、`driver_failure`、`shutdown` です。

| 症状 | 疑うべき境界／原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| decision／audit／SDK mapping が free-text reason を正として使っている | closed 語彙外の reason 採用（free-text は補足説明限定） | 閉集合外は不採用。decision の正にしない | reason を closed category/code に写す。free-text は `details` の non-authoritative 補足に限定する |
| metadata（retryable/safe_to_expose/audit_required）を実装都合で推測している | category default または code override に存在しない metadata の推測 | 推測 metadata は不採用 | category default と code override から metadata を決定する。存在しない場合は reason 規則を更新してから採用する |
| 非 retryable reason に retry hint を露出している | reason metadata 違反（例: `unauthorized`・`forbidden_state`・`expired` は `retryable=false`） | 非 retryable に retry hint を露出してはならない | reason metadata の `retryable` に従う。`resource_exhausted`・`backpressure`・`quality_violation`・`driver_failure` 等の `retryable=true` のみ retry hint を許可する |
| `safe_to_expose=false` の reason 詳細を外部に露出している | 露出制御違反（例: `unauthorized`・`driver_failure`・`token_key_unavailable`・`secret_unavailable` は `safe_to_expose=false`） | generic external failure class + opaque reference に倒す | safe exposure rule に従い、`safe_to_expose=false` なら category/code を露出せず opaque reference のみ返す。secret/key/token/backend detail を漏らさない |
| `audit_required=true` の path で audit event relation を記録していない | audit relation 欠落 | audit relation が無いと close evidence にその path を使えない | audit event relation を記録する。記録できない path を close evidence に採用しない |
| 閉集合外の新 reason／新 process failure class を追加した | 閉集合の無断拡張 | 閉集合外は不採用 | 新 reason category/code・新 process failure class は明示的な規則更新を経てから採用する |

## 3. external error mapping の落とし穴

external error mapping の owner: authoritative reason category/code は core、reason exposure metadata（retryable・safe_to_expose・audit_required）は core、external protocol status/wrapper は driver/sdk/cli（reason を失わない projection）、raw driver error detail は driver（non-authoritative・redacted）、audit event は core model + driver sink（external response の代替ではない）。external code は authoritative ではなく、authoritative failure は cataloged reason category/code または pre-core driver conversion reason に留まります。

| 症状 | 疑うべき境界／原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| HTTP status／WebSocket close code が core reason を置換している | external code を authoritative reason と誤認 | external code は authoritative ではない | core reason category/code または opaque error reference を保持する projection に直す。HTTP は network driver、WebSocket は network driver、STUN/TURN は TURN wire driver、SDK は sdk、CLI は entrypoints/cli が mapping owner |
| external response が core non-success 後に success を主張 | failure mapping 違反 | external response は core non-success 後に success を主張してはならない | core decision を正とし、response emission failure は driver observation として別記録する。encode 失敗時は `external_encode_failed`、network send 失敗時は `network_send_failed`、driver shutdown 時は `driver_shutdown` を使う |
| SDK が server event の decode 失敗に対し server reason を捏造 | SDK invents server reason | SDK は local decode failure に対し invented server reason を使ってはならない | SDK-local closed error を返し、server-originated でない reason を作らない。server-originated reason のみ preserve する |
| logs/traces を external error authority として使用 | redacted diagnostics の誤用 | logs/traces は external error authority ではない | logs/traces は redacted diagnostics のみとする。authoritative reason は core catalog に戻す |
| field 順序ミスで unsafe reason 露出や audit relation 欠落が起きている | naked multi-argument constructor の使用 | naked multi-argument constructor は admitted boundary ではない | `ExternalErrorProjectionInput` のような named input type で external surface・status/wrapper class・correlation reference・exposed/redacted reason・audit relation・retry hint を渡す |

## 4. crash / panic / supervisor restart 分類

process failure class は閉集合です。`panic_observed`（graceful shutdown ではない）、`task_panic_observed`（domain transition でも process recovery でもない）、`process_crash_observed`（domain close ではない）、`unclean_shutdown_detected`（graceful drain ではない）、`supervisor_restart_observed`（readiness/restore success ではない）、`startup_after_unclean_exit`（state claim 前に restore policy が必要）、`crash_recovery_evidence`（tested scope に限定）。unclean restart は graceful drain ではなく、supervisor restart は recovery success ではありません。

| 症状 | 疑うべき境界／原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| unclean crash を graceful shutdown として報告 | crash 分類違反 | unclean shutdown は graceful drain ではない | failure class を正しく分類する（`unclean_shutdown_detected`／`process_crash_observed`）。prior drain status または audit status が無い場合、restart を closeout claim 上 unclean として扱う |
| supervisor restart を readiness として報告 | restart=recovery の誤認 | supervisor restart は readiness/restore success ではない | restart 後の readiness class を別 evidence として記録する。process uptime を restored domain state の証明にしない |
| 再起動後に restore policy なしで previous domain state を再利用 | silent restore | crash 後の domain state は restore policy 適用時のみ core が扱い、silent restore は不可 | `startup_after_unclean_exit` 時は state claim 前に restore/replay policy を適用する（durable recovery 規則に従う）。適用不可なら restore-specific reason を使う |
| task panic を process readiness または generic driver failure としてのみ報告 | task panic と process crash の混同 | `task_panic_observed` は process crash と同一視しない | task panic は runtime task lifecycle boundary で分類し、`runtime_task_panic_detected`（process crash 観測時は `process_crash_detected`、process panic 観測時は `process_panic_detected`）を使う |
| panic/crash log text を authoritative reason として採用 | log text の権威誤認 | panic log text は authoritative reason ではない | reason は closed catalog から取る。logs は diagnostic support のみとする |
| crash evidence が prior audit/drain status や startup run boundary を欠く | evidence 欠落 | 欠落した crash evidence は採用しない | crash/restart evidence に exact command/procedure、supervisor/process runner、expected/observed failure class、startup run IDs、audit/restore/readiness status（別 evidence class）を記録する |

## 5. CI quality gate の fail-closed 既定挙動

CI は guardrail であり、それ単独では close/complete/ready を成立させません。CI output は単独では十分な evidence ではなく、未実施または unknown の検証範囲に対する CI 通過は close/complete/ready の主張になりません。CI logs は diagnostic output のみです。

| 症状 | 疑うべき境界／原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| 必要 gate が run できない／timeout／unknown state | gate execution 不能 | target scope を close/complete/ready として扱わない。`UNKNOWN` は成功 state ではない | gate を実行可能にし、再現可能な結果を得てから判定する |
| CI success を production readiness と主張 | close-like claim が検証範囲を bypass | close/complete/ready は未実施の検証範囲に対して主張しない | CI success は guardrail であり readiness の単独根拠ではない。production / live readiness は別の implementations 側 claim |
| enterprise code coverage が core line `<90%`／overall `<85%`／core crate floor `<80%`／denominator scope 欠落で pass している | coverage gate fail-open | core line<90% / overall<85% / core crate floor<80% / denominator 欠落 / diagnostic-only coverage 採用で fail-closed | threshold を満たす coverage を取得する。import-only・smoke-only・text-inspection-only・generated-output-only coverage を enterprise coverage として採用しない |
| v0.2 completion を CI success だけで主張 | 未実施の検証範囲に対する completion 主張 | completion は未実施の検証範囲に対して主張しない | CI success を当該 scope に限定する。Kernel completion は別 claim であり CI success 単独から導かない |
| npm／yarn で JS/TS evidence を生成 | tooling 違反 | npm/yarn は v0.2 CI evidence として不採用 | JS/TS command は `pnpm` を使う |
| source-only static gate で runtime behavior を、runtime smoke で architecture compliance を証明 | evidence class の混同 | evidence class を source/build/runtime/live で混ぜない | static gate は dependency 方向・source-shape の証明に、runtime smoke は composition の証明に限定する。それぞれ class-specific evidence fields を記録する |
| endpoint／artifact／release／time／edge·proxy／reconfiguration 等の gate を class-specific evidence なしに success と主張 | class-specific field 欠落 | 各 gate は class-specific evidence fields なしに success を主張してはならない | 各 gate class（public endpoint lifecycle、export/backup artifact、release artifact provenance、time synchronization、edge/proxy trust、runtime reconfiguration、packet rewrite/media transform、service discovery、distributed state/failover、runtime task/worker lifecycle、internal service identity/trust、cross-plane identity/session binding 等）の必須 field を記録する |

## 6. claim scope 転用の検知と是正

Kernel evidence と implementations readiness は別 claim であり、相互に代替してはなりません。次は claim scope 転用の代表症状です。

| 症状 | 疑うべき境界／原因 | fail-closed 既定挙動 | 是正手順 |
|---|---|---|---|
| Kernel evidence を implementations production／live readiness へ転用 | Kernel / implementations 境界違反 | implementations の存在/不在は Kernel completion claim に不採用 | Kernel evidence は Kernel claim に閉じる。implementations の readiness は別 evidence・別 claim とする |
| benchmark・coverage・native command success を readiness の証明として転用 | evidence scope の越境 | benchmark/coverage/native command success は Kernel evidence であり production/live/native application readiness の証明ではない | 各 evidence を当該 scope（measurement・diagnostic・command success）に限定する。readiness は別途 implementations 側で claim する |
| 未実施の検証範囲を実施済みとして扱う | scope 転用 | 未実施の検証範囲を実施済みとして扱わない。`UNKNOWN` は成功 state ではない | 検証を当該 scope で実施するか、close / complete / ready を主張しない |
| 存在する artifact を completion proof として扱う | 存在と proof の混同 | artifact 存在は completion proof ではない | artifact 存在を proof にしない。scope 外の artifact は別管理とし Kernel completion claim に採用しない |

## 7. 検知の最終原則（fail-closed summary）

- 曖昧・閉集合外・unknown・diagnostic-only・mixed-scope・readiness-smuggled の入力は、拒否側に倒します。`UNKNOWN` は成功 state ではありません。
- reason・process failure class・CI gate class は、すべて閉集合です。閉集合の拡張は明示的な規則更新を経てからのみ採用します。
- close/complete/ready/freeze は、未実施の検証範囲に対して主張しません。
- 境界違反・claim scope 転用が疑われる場合、是正は常に「正の owner（core semantics／core reason／core port）へ戻す」方向で行い、driver・entrypoint・SDK・regulated・implementations 側へ semantic authority を移しません。
