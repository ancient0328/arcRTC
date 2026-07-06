# 第11章 real-device

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 distro 領域における real-device command evidence の device class closed set、command matrix（Android / iOS / browser の全コマンド）、device context matrix、wrapper の CLI shape、package layout、platform command closed set、program resolution boundary、evidence field、post-execution requiredness matrix、exit status mapping、device identifier redaction rule、`KPI-008`（bounded command evidence）と `KPI-011`（real-device success）の execution mode boundary、`KPI-011` の required device class set、success evidence schema、success verdict 規則、Android emulator provider admission、required report binding、rerun condition、fail-closed 規則を、再現実装可能な粒度で固定します。本章は完全自己完結であり、他文書・実コードを参照せずに理解できます。同一仕様書内の他章番号のみ参照します。

real-device command success は bounded device command result です。real-device は live readiness ではありません。real-device command success は live readiness、production readiness、product completion、public distribution readiness、native application readiness、Kernel completion / freeze を単独では主張しません（MUST NOT）。

---

## 1. real-device test package ownership

real-device command wrapper は test surface が所有します。初期 package path は固定です。

```text
tests/real-device
```

初期 package name は固定です。

```text
arcrtc-distro-real-device-tests
```

wrapper command working directory は固定です。

```text
distro
```

## 2. device class closed set

| Device class | Meaning |
|---|---|
| `AndroidPhysical` | Android physical device connected through wrapper command |
| `AndroidEmulator` | Android emulator connected through wrapper command |
| `IosPhysical` | iOS physical device connected through wrapper command |
| `IosSimulator` | iOS simulator connected through wrapper command |
| `DesktopBrowser` | desktop browser controlled by bounded wrapper command |
| `MobileBrowser` | mobile browser or mobile emulation controlled by bounded wrapper command |

device class が closed set にない場合、real-device 仕様は崩れます（fail-closed）。

## 3. command matrix

| Evidence id | Command wrapper | Device class | Environment class | Expected outcome |
|---|---|---|---|---|
| `RD-ANDROID-001` | `cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-physical` | `AndroidPhysical` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-ANDROID-002` | `cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-emulator` | `AndroidEmulator` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-IOS-001` | `cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-physical` | `IosPhysical` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-IOS-002` | `cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-simulator` | `IosSimulator` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-BROWSER-001` | `cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser` | `DesktopBrowser` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-BROWSER-002` | `cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class mobile-browser` | `MobileBrowser` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |

## 4. device context matrix

| Device class | Runtime version class | Execution surface | Network class | Logs / metrics location rule |
|---|---|---|---|---|
| `AndroidPhysical` | `AndroidApiLevel` | `NativeSdkCommand` | `LocalUsb` | non-empty path under `target/distro-evidence/real-device/` |
| `AndroidEmulator` | `AndroidApiLevel` | `NativeSdkCommand` | `EmulatorLoopback` | non-empty path under `target/distro-evidence/real-device/` |
| `IosPhysical` | `IosSystemVersion` | `NativeSdkCommand` | `LocalUsb` | non-empty path under `target/distro-evidence/real-device/` |
| `IosSimulator` | `IosSystemVersion` | `NativeSdkCommand` | `SimulatorLoopback` | non-empty path under `target/distro-evidence/real-device/` |
| `DesktopBrowser` | `BrowserVersion` | `BrowserWebrtcCommand` | `LocalBrowser` | non-empty path under `target/distro-evidence/real-device/` |
| `MobileBrowser` | `BrowserVersion` | `BrowserWebrtcCommand` | `MobileBrowserEmulation` | non-empty path under `target/distro-evidence/real-device/` |

wrapper は次の preflight field を記録した後でのみ platform tool を internal に invoke できます（MUST）。target platform / device class / runtime version class / execution surface / network class / command string / toolchain version / logs / metrics location / expected outcome / non-claim scope。

post-execution evidence field は platform command が return した後にのみ記録します（MUST）。actual outcome / exit status / distro reason / `redacted_device_identifier`（`exit_status == 0` のとき literal `redacted` または `sha256:<64 lowercase hex characters>` に一致）。actual outcome と redacted device identifier を preflight 中に捏造しません（MUST NOT）。

## 5. wrapper CLI shape

`distro/` からの real-device wrapper command は固定です。

```text
cargo run --manifest-path tests/real-device/Cargo.toml -- <platform> --profile reference-local --device-class <device-class>
```

Allowed `<platform>` values:

- `android`
- `ios`
- `browser`

Allowed `<device-class>` values は command matrix（第3節）に従います。

## 6. wrapper package layout

`tests/real-device` は executable test package であり、次を含みます（MUST）。

| File | Role |
|---|---|
| `Cargo.toml` | manifest-path executable package manifest |
| `src/main.rs` | CLI entrypoint only |
| `src/cli.rs` | `clap` parser for platform / profile / device-class |
| `src/dispatch.rs` | platform command closed-set dispatcher |
| `src/evidence.rs` | `RealDeviceEvidenceRecord` builder / validation |
| `src/error.rs` | wrapper-local typed error |
| `src/platform_executor.rs` | `KPI-011` closed platform command executor |
| `src/command_output.rs` | platform command stdout / stderr / exit status closed output |
| `src/device_observation.rs` | platform command output to required device observation mapper |
| `src/redaction.rs` | real-device raw identifier rejection / redacted identifier output |
| `src/success.rs` | six required row success verdict validator |

`src/main.rs` は CLI args を parse し、dispatcher を呼び、evidence を build し、process exit status を返すだけです（MUST）。platform command policy / readiness claim / device identity formatting を所有しません（MUST NOT）。`src/evidence.rs` は evidence JSON を書く前に `validate_real_device_evidence_record(&record)` を呼びます（MUST）。

## 7. platform command closed set と program resolution

| Platform | Device class | Internal command class | Allowed command shape |
|---|---|---|---|
| `android` | `android-physical` | `AndroidDevice` | `adb devices -l` |
| `android` | `android-emulator` | `AndroidDevice` | `adb devices -l` |
| `ios` | `ios-physical` | `IosDevice` | `xcrun xctrace list devices` |
| `ios` | `ios-simulator` | `IosSimulator` | `xcrun simctl list devices` |
| `browser` | `desktop-browser` | `Browser` | wrapper-local capability record only |
| `browser` | `mobile-browser` | `Browser` | wrapper-local capability record only |

wrapper は unlisted platform command を invoke しません（MUST NOT）。

allowed command shape と executable path resolution は別です。canonical command identity は closed set の `Allowed command shape` 文字列のままです。wrapper は次の host location を通じて `adb` program を resolve できます（canonical command identity を変えません）。

- current process `PATH`
- `ANDROID_HOME/platform-tools/adb`
- `ANDROID_SDK_ROOT/platform-tools/adb`
- `$HOME/Library/Android/sdk/platform-tools/adb`

wrapper は argument を変えず、extra Android command variant を追加せず、standard SDK path が存在するという理由で missing Android physical device を success に変換しません（MUST NOT）。

## 8. wrapper evidence fields

wrapper evidence は次を含みます（MUST）。

- `platform`
- `device_class`
- `internal_command_class`
- `runtime_version_class`
- `execution_surface`
- `network_class`
- `toolchain_runtime_version`
- `exit_status`
- `actual_outcome`
- `logs_metrics_location`
- `non_claim_scope`

`platform` / `device_class` / `internal_command_class` / `runtime_version_class` / `execution_surface` / `network_class` / `logs_metrics_location` / `preflight_outcome` は preflight field です。`exit_status` / `actual_outcome` / `redacted_device_identifier` は post-execution field であり、platform command が return した後にのみ書きます（MUST）。

## 9. post-execution requiredness matrix

| Condition | `platform_command` | `exit_status` | `actual_outcome` | `redacted_device_identifier` |
|---|---|---|---|---|
| Android / iOS platform tool path | required and must match closed command shape | required | required | required only when `exit_status == 0`; otherwise absent |
| Browser wrapper-local capability path | absent | required | required | required only when `exit_status == 0`; otherwise absent |
| platform command unavailable | required for Android / iOS, absent for browser | required with status `3` | required | absent |
| no observed device, offline device, unauthorized Android device, or shutdown simulator | required for Android / iOS, absent for browser | required with status `2` | required | absent |
| evidence field missing | follows attempted platform class | required with status `4` | required | absent |

`redacted_device_identifier` は literal `redacted` または `sha256:<64 lowercase hex characters>` でなければなりません（MUST）。raw identifier marker を書きません（MUST NOT）。

## 10. exit status mapping

| Condition | exit status | distro reason |
|---|---|---|
| allowed platform command succeeded and evidence fields complete | `0` | `DISTRO_OK` |
| device class has no observed device | `2` | `REAL_DEVICE_SCOPE_MISMATCH` |
| platform command unavailable | `3` | `RUNTIME_EXECUTOR_ERROR` |
| evidence field missing | `4` | `EVIDENCE_FIELDS_INCOMPLETE` |
| command working directory outside distro | `5` | `COMMAND_SCOPE_MISMATCH` |

wrapper は unrecognized platform / device class / command class に対して fail closed します（MUST）。command matrix の追加 failure classification:

| Failure | Distro reason |
|---|---|
| device class absent | `REAL_DEVICE_SCOPE_MISMATCH` |
| command working directory outside distro | `COMMAND_SCOPE_MISMATCH` |
| evidence field missing | `EVIDENCE_FIELDS_INCOMPLETE` |
| readiness claim attempted while readiness is not admitted | `READINESS_NOT_ADMITTED` |

## 11. redaction rule（raw marker closed set）

`KPI-011` report と evidence row は、次の raw marker を含みません（MUST NOT）。

- `serial:`, `serial=`
- `udid:`, `udid=`
- `android_id:`, `android_id=`
- `device_id:`, `device_id=`
- `imei:`, `imei=`
- `meid:`, `meid=`
- `account:`, `account=`
- `token:`, `token=`
- `private_key:`, `private_key=`
- `device_name:`, `device_name=`

## 12. non-claim scope

すべての real-device evidence record は次を含みます（MUST）。

- `ProductionReadinessNotClaimed`
- `LiveReadinessNotClaimed`
- `PublicDistributionReadinessNotClaimed`
- `NativeApplicationReadinessNotClaimed`
- `KernelCompletionNotClaimed`

## 13. KPI-008 / KPI-011 execution mode boundary

`KPI-008` は bounded real-device command evidence であり、wrapper の command shape、preflight field、exit mapping、non-claim scope を検査します。`KPI-008` の output は `KPI-011` real-device success の代替になりません（MUST NOT）。

`KPI-011` は six required wrapper command を実行し、各 row が exit `0` と complete post-execution evidence を持つ場合だけ success evidence として採用できます（MUST）。`KPI-011` path では、Android / iOS の platform command を `PlatformCommandUnavailable` として事前に閉じる挙動を success evidence に転用しません（MUST NOT）。browser wrapper-local capability row は Android / iOS row の代替になりません（MUST NOT）。`KPI-011` report は、`cargo test --manifest-path tests/real-device/Cargo.toml ...success_matrix`、preflight-only output、exit `3` の platform command unavailable output、browser-only output を success evidence として採用しません（MUST NOT）。Android physical、Android emulator、iOS physical、iOS simulator、desktop browser、mobile browser の 6 row すべてが必要です。

## 14. KPI-011 required device class set

| KPI-011 row id | Matrix evidence id | Platform group | Wrapper command | Device class | OS / browser class | Runtime version class | Network class | Expected outcome |
|---|---|---|---|---|---|---|---|---|
| `KPI-011-ANDROID-PHYSICAL` | `RD-ANDROID-001` | Android | `cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-physical` | `AndroidPhysical` | `AndroidApiLevel` | `AndroidApiLevel` | `LocalUsb` | wrapper records command exit status and evidence JSON |
| `KPI-011-ANDROID-EMULATOR` | `RD-ANDROID-002` | Android | `cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-emulator` | `AndroidEmulator` | `AndroidApiLevel` | `AndroidApiLevel` | `EmulatorLoopback` | wrapper records command exit status and evidence JSON |
| `KPI-011-IOS-PHYSICAL` | `RD-IOS-001` | iOS | `cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-physical` | `IosPhysical` | `IosSystemVersion` | `IosSystemVersion` | `LocalUsb` | wrapper records command exit status and evidence JSON |
| `KPI-011-IOS-SIMULATOR` | `RD-IOS-002` | iOS | `cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-simulator` | `IosSimulator` | `IosSystemVersion` | `IosSystemVersion` | `SimulatorLoopback` | wrapper records command exit status and evidence JSON |
| `KPI-011-BROWSER-DESKTOP` | `RD-BROWSER-001` | browser | `cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser` | `DesktopBrowser` | `BrowserVersion` | `BrowserVersion` | `LocalBrowser` | wrapper records command exit status and evidence JSON |
| `KPI-011-BROWSER-MOBILE` | `RD-BROWSER-002` | browser | `cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class mobile-browser` | `MobileBrowser` | `BrowserVersion` | `BrowserVersion` | `MobileBrowserEmulation` | wrapper records command exit status and evidence JSON |

`KPI-011` の固定 scope: target KPI `KPI-011` / command owner `tests/real-device` wrapper / working directory `distro` / required platform groups Android, iOS, browser / required device class count 6 / success verdict rule（every required row has `exit_status == 0` and complete post-execution evidence）/ failure classification `RealDeviceSuccessEvidenceMissing` / adoption scope real-device success evidence only。

## 15. success evidence schema

`KPI-011` report は required device class row ごとに次の field を持ちます（MUST）。

- `kpi_row_id`
- `matrix_evidence_id`
- `platform_group`
- `command`
- `working_directory`
- `device_class`
- `os_or_browser_class`
- `runtime_version_class`
- `network_class`
- `expected_outcome`
- `actual_outcome`
- `exit_status`
- `toolchain_runtime_version`
- `redacted_device_identifier`
- `raw_identifier_absence_statement`
- `logs_metrics_location`
- `rerun_condition`
- `verdict`
- `failure_classification`

## 16. success verdict 規則

`verdict` は `Pass` または `Fail` だけを許可します（MUST）。`Pass` は、各 required row が次を全て満たす場合だけ使います（MUST）。

- command が `Required Device Class Set` の wrapper command と完全一致する。
- `working_directory` が `distro` である。
- `device_class`、`os_or_browser_class`、`runtime_version_class`、`network_class` が `Required Device Class Set` の row と完全一致する。
- `actual_outcome` が post-execution field として存在し、空文字列ではない。
- `exit_status` が `0` である。
- `toolchain_runtime_version` が空ではない。
- Android physical / Android emulator は `adb devices -l` によって target class が `device` 状態として観測され、missing tool / offline / unauthorized / no device ではない。
- iOS physical は `xcrun xctrace list devices` の `== Devices ==` section に iPhone または iPad が存在し、`== Devices Offline ==` section の device を success として採用しない。
- iOS simulator は `xcrun simctl list devices` で target simulator が `(Booted)` 状態として観測され、`(Shutdown)` の列挙だけでは success としない。
- `logs_metrics_location` が `target/distro-evidence/real-device/` 配下の非空 path である。
- `redacted_device_identifier` が literal `redacted` または `sha256:<64 lowercase hex characters>` である。
- `raw_identifier_absence_statement` が raw identifier marker absence を明示する。

`Fail` は上記のいずれかを満たさない場合に使います。

## 17. Android emulator provider admission

`AndroidEmulator` row は、AVD が Android SDK CLI を通じて launch され、canonical な `adb devices -l` surface で観測される場合に限り、local Mac の Android Studio managed AVD を使えます（MAY）。admitted provider shape:

- AVD inventory command: `$HOME/Library/Android/sdk/emulator/emulator -list-avds`
- AVD launch command: `$HOME/Library/Android/sdk/emulator/emulator -avd <AVD_NAME> -no-window -no-audio -no-boot-anim -gpu swiftshader_indirect`
- boot completion check: `$HOME/Library/Android/sdk/platform-tools/adb shell getprop sys.boot_completed`
- adoption command: `KPI-011-ANDROID-EMULATOR` の wrapper command
- observation surface: `adb devices -l`
- accepted observed shape: device state column が `device` の `emulator-*` row 一つ

Android Studio GUI state、device manager display、emulator process log、emulator gRPC log、crash log、public key log、token-bearing log を `KPI-011` success evidence として使いません（MUST NOT）。AVD provider は `AndroidPhysical` を満たしません。AVD provider は iOS physical / iOS simulator / desktop browser / mobile browser row の代替になりません。

## 18. required report binding

| Required item | Required value |
|---|---|
| report filename | `<YYYYMMDD>-distro-KPI-011-REAL-DEVICE-SUCCESS-EVIDENCE.md` |
| correlation id prefix | `IMPL-KPI-011-REAL-DEVICE-SUCCESS-EVIDENCE` |
| required row count | 6 |
| Android rows | `KPI-011-ANDROID-PHYSICAL`, `KPI-011-ANDROID-EMULATOR` |
| iOS rows | `KPI-011-IOS-PHYSICAL`, `KPI-011-IOS-SIMULATOR` |
| browser rows | `KPI-011-BROWSER-DESKTOP`, `KPI-011-BROWSER-MOBILE` |
| success summary row | `KPI-011-R1` |

## 19. rerun condition

次のいずれかが変わった場合、`KPI-011` real-device success evidence を再実行します（MUST）。

- real-device success 仕様
- real-device command matrix 仕様
- real-device wrapper command 仕様
- `tests/real-device` wrapper source
- Android / iOS / browser toolchain または runtime version
- connected device class
- network class
- evidence output path
- redaction rule

## 20. fail-closed / collapse 条件

次のいずれかに該当する場合、real-device 仕様は崩れます（fail-closed で不成立として扱います）。

- platform command を wrapper evidence record なしに実行する。
- raw device identifier / token / private key を report に書く。
- real-device command success を live readiness / native application readiness として使う。
- wrapper working directory が `distro` の外にある。
- device class が closed set にない。
- wrapper が unlisted platform command を invoke する。
- missing platform tool を success に変換する。
- wrapper command が manifest path を省略し distro workspace membership に依存する。
- Android SDK path resolution が canonical command identity または command argument を変える。
- required six device class row のいずれかが欠ける。
- wrapper command を介さない platform command result を採用する。
- `exit_status != 0` の row を success とする。
- offline / shutdown / unauthorized / no device の row を success とする。
- required field が欠けた row を `Pass` とする。
- real-device success を native application readiness / production readiness / live readiness / Kernel completion / freeze / full fixed-goal completion の代替証跡にする。
- network class を記録しない evidence を採用する。
- Kernel source を変更して real-device success を成立させる。
