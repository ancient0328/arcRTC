# Kernel contract consumption（Kernel 契約消費）

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 の implementations 領域（Kernel 外実装領域）が、frozen 済みの Kernel public contract をどのように import し、version を pin し、reference / product builder に mapping し、consumed Kernel source を凍結 read-only 入力として pin するかを完全に固定します。本章は implementations 側が Kernel semantic authority を所有・上書き・複製しないことを不変条件として、import 可能 surface、import path 規則、builder constructor chain の全 mapping、Kernel source pin の必須入力と fail-closed 条件を内在化します。本章は自己完結であり、他文書を開かずに再現実装できる粒度で記述します。

## 依存方向の不変条件

implementations は Kernel public contract / SDK projection / documented command surface に依存します（`implementations -> Kernel`）。逆方向の依存（`Kernel -> implementations`）は禁止です。implementations は Kernel semantic authority を所有・上書きしません。Kernel contract の変更が必要になった場合は fail-closed とし、Kernel 側の正式な版管理された契約改定が先行する必要があります。implementations 側で Kernel contract を迂回・複製・拡張してはなりません（MUST NOT）。

## 1. Importable surface（利用可能 surface）

implementations は次の Kernel surface を利用できます（MAY）。これは閉集合であり、表に列挙されない surface の利用は禁止です。

| Surface | 利用方法 |
|---|---|
| Kernel public contract | SFU / TURN / Signaling implementation の入力契約として利用する |
| Kernel SDK projection | Signaling-side public API projection として利用する |
| core-owned port definition | driver / implementation boundary の契約として利用する |
| documented command surface | benchmark / test / evidence command の起点として利用する |
| Kernel report references | non-claim boundary の根拠として参照する |

implementations が利用できる Kernel surface の利用方法（consumption boundary）を別表で確定します。

| Kernel surface | implementations 側の利用方法 |
|---|---|
| public contract | Signaling / TURN / SFU 実装の入力契約として利用する |
| SDK projection | Signaling public projection として利用する |
| core-owned port definition | implementation mapper / runtime wrapper の接続契約として利用する |
| documented command surface | benchmark / test / evidence command の起点として利用する |
| Kernel report reference | non-claim / boundary 根拠として参照する |

## 2. Non-importable surface（利用禁止 surface）

implementations は次を import / overwrite / duplicate / 所有してはなりません（MUST NOT）。これは閉集合です。

- Kernel core semantics
- Kernel reason catalog ownership
- Kernel port definition ownership
- Kernel dependency direction
- Kernel final Closed Gate authority
- Kernel completion / freeze claim authority
- Kernel evidence authority

## 3. Import path 規則

Kernel crate は、dependency admission の Kernel local path dependency matrix に列挙された package だけを import できます（本仕様書第04章「Toolchain / runtime / dependency」の Kernel local path dependency matrix が当該 matrix の正です）。追加 import が必要になった場合は、source 変更前に dependency matrix を更新し、その追加 import は Kernel semantic authority を侵食してはなりません（MUST）。

Kernel 参照方式は local path dependency / local source dependency に固定します。次の規則を固定します。

| Option | 規則 | 理由 |
|---|---|---|
| local path dependency | 採用 | 同一 repository 内の frozen Kernel contract を直接参照できる |
| published crate dependency | 不採用 | publication / version distribution readiness をまだ主張しないため |
| binary / artifact dependency | 不採用 | source-level contract consumption と evidence 接続が弱くなるため |
| source copy | 禁止 | Kernel freeze boundary を破壊するため |

## 4. Version pin 規則

implementations は Kernel を次の条件で参照します（MUST）。

- 参照先は `Kernel/` に限定する。
- 参照は source copy ではなく dependency reference とする。
- dependency reference は commit / path / version-equivalent identifier を evidence report に記録する。
- Kernel 側の変更が必要な場合、implementations では fail-closed とする。

### Upgrade 規則

Kernel dependency を更新する場合、implementations 側は次を実施します（MUST）。

1. Kernel 側の正式な版管理された契約改定の有無を確認する。
2. affected implementations surface を列挙する。
3. reference / product / tests / benchmark / readiness への影響を分離する。
4. 当該変更について専用 evidence と verdict を接続する。

## 5. Consumption 設計手順

Kernel contract を利用する実装は、次の順で設計します（MUST）。

1. 利用する Kernel surface を source path で特定する。
2. implementations 側で必要な wrapper / mapper / runtime configuration を定義する。
3. wrapper / mapper は Kernel contract を変換または呼び出すだけに限定する。
4. wrapper / mapper は Kernel semantics を新規定義しない。
5. Kernel contract 不足が判明した場合、implementations task を停止する。
6. Kernel 側の正式な版管理された契約改定が存在しない contract は、implementations 側で代替定義しない。

### Allowed wrapper（許可 wrapper）

implementations 側に置いてよい wrapper は次の閉集合に限定します（MAY）。

| Wrapper class | 許可される責務 |
|---|---|
| runtime wrapper | process / async runtime / socket / environment の接続 |
| configuration mapper | product / reference config を Kernel contract input に変換 |
| error mapper | Kernel reason を implementation-facing report reason に投影 |
| command wrapper | benchmark / test / execution command の起動補助 |
| observation mapper | metrics / log / trace output の implementation evidence shape への変換 |

### Prohibited wrapper（禁止 wrapper）

implementations 側に置いてはならない wrapper は次です（MUST NOT）。

- Kernel reason を新規定義する wrapper
- Kernel port definition を拡張する wrapper
- Kernel state transition を上書きする wrapper
- Kernel routing / allocation / signaling decision を再実装する wrapper
- Kernel completion evidence を implementations readiness proof に変換する wrapper

## 6. Builder mapping（Kernel public constructor 呼び出し規則）

reference builder が Kernel public constructor をどの順序・どの引数で呼び出すかを固定します。この mapping は Kernel semantics を再定義しません。Kernel source は変更せず、implementations は Kernel public API を消費します（MUST）。

### Shared builder 規則

reference builder は次を満たします（MUST / MUST NOT）。

- Kernel private field を読まない（MUST NOT）。
- Kernel constructor error を raw error として漏らさない（MUST NOT）。
- Kernel constructor error は local error variant に変換する（MUST）。
- command type literal は本章の table だけを使用する（MUST）。
- command version は初期 reference implementation では `CommandVersion::new(1)` に固定する（MUST）。

### 6.1 Signaling builder mapping

`build_kernel_signaling_command` は次の constructor chain に固定します。

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `SignalingSubject::new` | `input.room_id`, `input.participant_id` |
| 2 | `CommandType::new` | table `Signaling Command Type Literal` |
| 3 | `CommandVersion::new` | `1` |
| 4 | `CommandEnvelope::new` | `input.correlation_id`, command type, command version, `TargetSurface::Signaling`, signaling subject |
| 5 | `SignalingCommand::new` | envelope, `input.kind`, `input.payload` |

#### Signaling Command Type Literal（閉集合）

| `SignalingCommandKind` | `CommandType` literal |
|---|---|
| `JoinRoom` | `reference.signaling.join_room` |
| `LeaveRoom` | `reference.signaling.leave_room` |
| `SendOffer` | `reference.signaling.send_offer` |
| `SendAnswer` | `reference.signaling.send_answer` |
| `SendIceCandidate` | `reference.signaling.send_ice_candidate` |
| `RequestTurnCredential` | `reference.signaling.request_turn_credential` |
| `AcknowledgeForward` | `reference.signaling.acknowledge_forward` |

Kernel 側で `CommandEnvelope::new` または `SignalingCommand::new` の shape が変化した場合、builder は `ReferenceSignalingError::KernelContractMismatch` で fail-closed しなければなりません（MUST）。

### 6.2 Signaling event projection mapping

`project_reference_signaling_event` は次に固定します。

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `SignalingSubject::new` | `outcome.room_id`, `outcome.participant_id` |
| 2 | `SignalingEvent::new` | `outcome.correlation_id`, `outcome.kind`, subject, payload |

`project_reference_signaling_event` は Kernel private command payload から event payload を導出してはなりません（MUST NOT）。

### 6.3 TURN builder mapping

`build_kernel_turn_command` は次に固定します。

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `TurnCommand::try_new` | `input.kind`, `input.transaction_id`, `input.references`, `input.peer_address`, `input.requested_lifetime`, `input.relay_packet_id` |

Kernel `TurnContractError` は `ReferenceTurnError::KernelContractMismatch` に mapping します。

#### TURN required field gate

builder は `TurnCommand::try_new` を直接呼び出さなければならず（MUST）、Kernel validation を authoritative semantics として複製してはなりません（MUST NOT）。local pre-check は Kernel construction の前に fixture incompleteness を分類することだけが許されます（MAY）。

| Command kind | Kernel call 前に必須となる local fixture | 欠落時の reason |
|---|---|---|
| `Allocate` | transaction id and credential fixture reference | `InvalidFixtureCredential` |
| `Refresh` | active allocation id reference | `StateBoundaryViolation` |
| `CreatePermission` | active allocation id reference and peer address | `StateBoundaryViolation` |
| `ChannelBind` | permission id, channel bind id, peer address | `StateBoundaryViolation` |
| `RelayData` | allocation id, permission id, peer address, packet id | `StateBoundaryViolation` |

### 6.4 SFU builder mapping

`build_kernel_sfu_item` は次に固定します。

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `SfuContractItem::new` | `input.model_kind`, `input.references`, `input.payload` |

`build_borrowed_packet_view` は次に固定します。

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `SfuPacketView::new` | `packet_id`, `stream_id`, `source_endpoint_id`, `header`, `raw_packet`, `payload` |

`raw_packet` と `payload` の引数は caller scope から受け取った borrowed slice でなければなりません（MUST）。builder は新規 buffer を allocate せず、bytes を clone せず、byte slice を reference state に保持してはなりません（MUST NOT）。

### 6.5 Builder error mapping（閉集合）

| Builder | Kernel / local failure | Local error |
|---|---|---|
| `build_kernel_signaling_command` | constructor shape mismatch | `ReferenceSignalingError::KernelContractMismatch` |
| `project_reference_signaling_event` | missing outcome field | `ReferenceSignalingError::EvidenceFieldsIncomplete` |
| `build_kernel_turn_command` | `TurnCommand::try_new` error | `ReferenceTurnError::KernelContractMismatch` |
| `build_kernel_sfu_item` | missing reference set member required by fixture | `ReferenceSfuError::KernelContractMismatch` |
| `build_borrowed_packet_view` | missing packet id / stream id / endpoint id fixture | `ReferenceSfuError::KernelContractMismatch` |

## 7. Kernel source pin（凍結 contract）

implementations は consumed Kernel source snapshot を read-only 入力として pin します。implementations 側は Kernel source を変更せず（MUST NOT）、Kernel source を production readiness / live readiness / benchmark threshold satisfaction / real-device success の代替 evidence として扱いません（MUST NOT）。

### 7.1 Pin fields（必須入力）

| Field | 必須値 |
|---|---|
| `kernel_root` | `Kernel/` |
| `kernel_source_file_count` | `216` |
| `kernel_source_snapshot_identifier` | `sha256:0f4963a6b3e57624dbfcca0bc4aa0372a72ccc2c7bd7e936d7ffb2d50097c71e` |
| `implementations_kernel_mutation_absence` | `Kernel source is read-only input for the binding; the binding writes only under implementations/` |

### 7.2 Snapshot command

consumed Kernel source snapshot identifier は implementations root から次の command で算出します。

```sh
find ../Kernel -path '*/target' -prune -o -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -print | sort | while IFS= read -r f; do shasum -a 256 "$f"; done | shasum -a 256
```

command 出力の第1フィールドは次に一致しなければなりません（MUST）。

```text
0f4963a6b3e57624dbfcca0bc4aa0372a72ccc2c7bd7e936d7ffb2d50097c71e
```

consumed file count は implementations root から次の command で算出します。

```sh
find ../Kernel -path '*/target' -prune -o -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -print | sort | wc -l
```

command 出力は次でなければなりません（MUST）。

```text
216
```

### 7.3 Binding assertion

`tests/boundary/tests/kernel_completion_freeze_binding.rs` が次の assertion を所有します。

```text
kpi_kernel_completion_freeze_binding_accepts_consumed_snapshot_without_kernel_mutation
```

この assertion は次を検証しなければなりません（MUST）。

- 現在の Kernel source snapshot digest が `0f4963a6b3e57624dbfcca0bc4aa0372a72ccc2c7bd7e936d7ffb2d50097c71e` に一致する。
- 本 pin が同一の snapshot digest と file count を含む。
- 本 pin が Kernel source を read-only 入力として扱い、Kernel source mutation を認めない。

### 7.4 Binding report 必須フィールド

binding report は次を含まなければなりません（MUST）。

| 必須フィールド | 値の source |
|---|---|
| correlation id | report header |
| command | `cargo test --manifest-path tests/boundary/Cargo.toml kpi_kernel_completion_freeze_binding_accepts_consumed_snapshot_without_kernel_mutation` |
| working directory | `implementations` |
| target package | `arcrtc-implementation-boundary-tests` |
| target scope | Kernel source pin（凍結 contract） |
| expected outcome | consumed Kernel snapshot is connected to Kernel final report and Kernel mutation is not used |
| actual outcome | command stdout / stderr summary and exit status |
| environment / toolchain | `rustc -Vv`, `cargo -V`, `shasum -a 256` |
| reason classification | `ImplementationOk` |
| non-claim scope | `none` |
| rerun condition | any Kernel source file, Kernel final report, this binding, or the binding report/index rule changes |

### 7.5 Rerun condition（再実行条件）

pin validation は次のいずれかが変化した場合に再実行しなければなりません（MUST）。

- `Kernel/**/*.rs`
- `Kernel/**/Cargo.toml`
- `Kernel/**/Cargo.lock`
- `implementations/tests/boundary/tests/kernel_completion_freeze_binding.rs`

### 7.6 pin の根拠

implementations 側は Kernel source を、snapshot digest と file count で識別される read-only かつ version-pinned な入力として消費します。implementations 側は consumed Kernel source を implementations product completion や readiness evidence に転用してはなりません（MUST NOT）。production readiness / live readiness / benchmark threshold satisfaction / real-device success は implementations 側の専用 evidence で成立し、Kernel source pin から導出しません。implementations 側の正しい責務は、Kernel source を改変なしに消費し、再実行条件を固定することに限定されます。

## 8. Fail-closed 規則

実装時に Kernel contract の不足が見つかった場合、次の順に扱います（MUST）。

1. implementations 側の task を停止する。
2. 不足内容を implementations report に記録する。
3. Kernel 側の正式な版管理された契約改定が必要であることを明示する。
4. Kernel contract を implementations 側で迂回・複製・拡張しない。

### Kernel source pin の fail-closed 条件

次のいずれかが発生した場合、Kernel source pin を妥当として受理してはなりません（fail-closed / MUST NOT accept）。

- consumed Kernel source snapshot identifier が再計算値と一致しない。
- consumed Kernel source file count が再計算値と一致しない。
- implementations 側の fixed-goal 達成作業で Kernel source を変更する。
- Kernel source pin を production readiness / live readiness / benchmark threshold satisfaction / real-device success / implementations product completion の代替 evidence にする。

## 9. 不変条件（collapse conditions）

本章が定義する境界は、次のいずれかが発生した時点で崩壊します。これらは禁止であり、発生時は fail-closed とします（MUST NOT）。

- implementations が Kernel semantic authority を変更する。
- Kernel contract 不足を implementations 側の代替型で補う。
- dependency matrix にない Kernel crate を import する。
- Kernel evidence を implementations completion / readiness proof として採用する。
- Kernel source modification を implementations の通常 task として扱う。
- implementations が Kernel contract を複製・拡張・迂回する。
- wrapper / mapper が Kernel semantics を所有する。
- Kernel contract 不足時に implementations task を継続する。
- implementations が Kernel source をコピーする。
- published crate / artifact dependency を readiness proof なしに採用する。
- Kernel dependency 更新が evidence なしに行われる。
- path dependency を理由に Kernel source modification を通常 task として扱う。
- builder が Kernel private field を読む。
- builder が本章にない command type literal を使う。
- builder が Kernel constructor error を raw dependency error として公開する。
- `build_borrowed_packet_view` が packet bytes を copy / allocate / persist する。
- builder success を reference behavior completion / benchmark success / readiness success として扱う。

## 10. Non-claim

本章は、Kernel final report の再判定、Kernel source の再凍結、production readiness、live readiness、benchmark threshold satisfaction、real-device success、または full fixed-goal completion を単独では主張しません。本章は implementations 側の Kernel contract consumption / version pin / builder mapping / Kernel source pin の境界のみを固定します。
