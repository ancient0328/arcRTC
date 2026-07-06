# Chapter 11 real-device

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes, at a granularity sufficient for reproducible re-implementation, the real-device command evidence domain of the arcRTC v0.2 distro domain: the device class closed set, command matrix (all Android / iOS / browser commands), device context matrix, wrapper CLI shape, package layout, platform command closed set, program resolution boundary, evidence fields, post-execution requiredness matrix, exit status mapping, device identifier redaction rule, the execution mode boundary of `KPI-008` (bounded command evidence) and `KPI-011` (real-device success), the `KPI-011` required device class set, success evidence schema, success verdict rules, Android emulator provider admission, required report binding, rerun condition, and fail-closed rules. This chapter is fully self-contained and is understandable without consulting other documents or source code. It references only other chapter numbers within this same specification.

real-device command success is a bounded device command result. real-device is not live readiness. real-device command success MUST NOT claim live readiness, production readiness, product completion, public distribution readiness, native application readiness, or Kernel completion / freeze by itself.

---

## 1. Real-device test package ownership

Real-device command wrappers are owned by the test surface. The initial package path is fixed.

```text
tests/real-device
```

The initial package name is fixed.

```text
arcrtc-distro-real-device-tests
```

The wrapper command working directory is fixed.

```text
distro
```

## 2. Device class closed set

| Device class | Meaning |
|---|---|
| `AndroidPhysical` | Android physical device connected through wrapper command |
| `AndroidEmulator` | Android emulator connected through wrapper command |
| `IosPhysical` | iOS physical device connected through wrapper command |
| `IosSimulator` | iOS simulator connected through wrapper command |
| `DesktopBrowser` | desktop browser controlled by bounded wrapper command |
| `MobileBrowser` | mobile browser or mobile emulation controlled by bounded wrapper command |

If a device class is not in the closed set, the real-device specification collapses (fail-closed).

## 3. Command matrix

| Evidence id | Command wrapper | Device class | Environment class | Expected outcome |
|---|---|---|---|---|
| `RD-ANDROID-001` | `cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-physical` | `AndroidPhysical` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-ANDROID-002` | `cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-emulator` | `AndroidEmulator` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-IOS-001` | `cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-physical` | `IosPhysical` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-IOS-002` | `cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-simulator` | `IosSimulator` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-BROWSER-001` | `cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser` | `DesktopBrowser` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |
| `RD-BROWSER-002` | `cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class mobile-browser` | `MobileBrowser` | `RealDeviceBounded` | wrapper records command exit status and evidence JSON |

## 4. Device context matrix

| Device class | Runtime version class | Execution surface | Network class | Logs / metrics location rule |
|---|---|---|---|---|
| `AndroidPhysical` | `AndroidApiLevel` | `NativeSdkCommand` | `LocalUsb` | non-empty path under `target/distro-evidence/real-device/` |
| `AndroidEmulator` | `AndroidApiLevel` | `NativeSdkCommand` | `EmulatorLoopback` | non-empty path under `target/distro-evidence/real-device/` |
| `IosPhysical` | `IosSystemVersion` | `NativeSdkCommand` | `LocalUsb` | non-empty path under `target/distro-evidence/real-device/` |
| `IosSimulator` | `IosSystemVersion` | `NativeSdkCommand` | `SimulatorLoopback` | non-empty path under `target/distro-evidence/real-device/` |
| `DesktopBrowser` | `BrowserVersion` | `BrowserWebrtcCommand` | `LocalBrowser` | non-empty path under `target/distro-evidence/real-device/` |
| `MobileBrowser` | `BrowserVersion` | `BrowserWebrtcCommand` | `MobileBrowserEmulation` | non-empty path under `target/distro-evidence/real-device/` |

The wrapper MAY invoke platform tools internally only after it records the following preflight fields: target platform / device class / runtime version class / execution surface / network class / command string / toolchain version / logs / metrics location / expected outcome / non-claim scope.

Post-execution evidence fields MUST be recorded only after the platform command returns: actual outcome / exit status / distro reason / `redacted_device_identifier` (matching literal `redacted` or `sha256:<64 lowercase hex characters>` when `exit_status == 0`). actual outcome and redacted device identifier MUST NOT be fabricated during preflight.

## 5. Wrapper CLI shape

The real-device wrapper command from `distro/` is fixed.

```text
cargo run --manifest-path tests/real-device/Cargo.toml -- <platform> --profile reference-local --device-class <device-class>
```

Allowed `<platform>` values:

- `android`
- `ios`
- `browser`

Allowed `<device-class>` values follow the command matrix (Section 3).

## 6. Wrapper package layout

`tests/real-device` is an executable test package and MUST contain the following.

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

`src/main.rs` MUST only parse CLI args, call the dispatcher, build evidence, and return the process exit status. It MUST NOT own platform command policy / readiness claim / device identity formatting. `src/evidence.rs` MUST call `validate_real_device_evidence_record(&record)` before writing evidence JSON.

## 7. Platform command closed set and program resolution

| Platform | Device class | Internal command class | Allowed command shape |
|---|---|---|---|
| `android` | `android-physical` | `AndroidDevice` | `adb devices -l` |
| `android` | `android-emulator` | `AndroidDevice` | `adb devices -l` |
| `ios` | `ios-physical` | `IosDevice` | `xcrun xctrace list devices` |
| `ios` | `ios-simulator` | `IosSimulator` | `xcrun simctl list devices` |
| `browser` | `desktop-browser` | `Browser` | wrapper-local capability record only |
| `browser` | `mobile-browser` | `Browser` | wrapper-local capability record only |

The wrapper MUST NOT invoke an unlisted platform command.

Allowed command shape and executable path resolution are separate. The canonical command identity remains the `Allowed command shape` string in the closed set. The wrapper MAY resolve the `adb` program through the following host locations without changing the canonical command identity.

- current process `PATH`
- `ANDROID_HOME/platform-tools/adb`
- `ANDROID_SDK_ROOT/platform-tools/adb`
- `$HOME/Library/Android/sdk/platform-tools/adb`

The wrapper MUST NOT change arguments, add extra Android command variants, or convert a missing Android physical device into success because a standard SDK path exists.

## 8. Wrapper evidence fields

Wrapper evidence MUST include the following.

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

`platform` / `device_class` / `internal_command_class` / `runtime_version_class` / `execution_surface` / `network_class` / `logs_metrics_location` / `preflight_outcome` are preflight fields. `exit_status` / `actual_outcome` / `redacted_device_identifier` are post-execution fields and MUST be written only after the platform command returns.

## 9. Post-execution requiredness matrix

| Condition | `platform_command` | `exit_status` | `actual_outcome` | `redacted_device_identifier` |
|---|---|---|---|---|
| Android / iOS platform tool path | required and must match closed command shape | required | required | required only when `exit_status == 0`; otherwise absent |
| Browser wrapper-local capability path | absent | required | required | required only when `exit_status == 0`; otherwise absent |
| platform command unavailable | required for Android / iOS, absent for browser | required with status `3` | required | absent |
| no observed device, offline device, unauthorized Android device, or shutdown simulator | required for Android / iOS, absent for browser | required with status `2` | required | absent |
| evidence field missing | follows attempted platform class | required with status `4` | required | absent |

`redacted_device_identifier` MUST be literal `redacted` or `sha256:<64 lowercase hex characters>`. Raw identifier markers MUST NOT be written.

## 10. Exit status mapping

| Condition | exit status | distro reason |
|---|---|---|
| allowed platform command succeeded and evidence fields complete | `0` | `DISTRO_OK` |
| device class has no observed device | `2` | `REAL_DEVICE_SCOPE_MISMATCH` |
| platform command unavailable | `3` | `RUNTIME_EXECUTOR_ERROR` |
| evidence field missing | `4` | `EVIDENCE_FIELDS_INCOMPLETE` |
| command working directory outside distro | `5` | `COMMAND_SCOPE_MISMATCH` |

The wrapper MUST fail closed for any unrecognized platform / device class / command class. Additional command matrix failure classification:

| Failure | Distro reason |
|---|---|
| device class absent | `REAL_DEVICE_SCOPE_MISMATCH` |
| command working directory outside distro | `COMMAND_SCOPE_MISMATCH` |
| evidence field missing | `EVIDENCE_FIELDS_INCOMPLETE` |
| readiness claim attempted while readiness is not admitted | `READINESS_NOT_ADMITTED` |

## 11. Redaction rule (raw marker closed set)

The `KPI-011` report and evidence rows MUST NOT contain the following raw markers.

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

## 12. Non-claim scope

Every real-device evidence record MUST include the following.

- `ProductionReadinessNotClaimed`
- `LiveReadinessNotClaimed`
- `PublicDistributionReadinessNotClaimed`
- `NativeApplicationReadinessNotClaimed`
- `KernelCompletionNotClaimed`

## 13. KPI-008 / KPI-011 execution mode boundary

`KPI-008` is bounded real-device command evidence and inspects the wrapper command shape, preflight fields, exit mapping, and non-claim scope. The output of `KPI-008` MUST NOT be a substitute for `KPI-011` real-device success.

`KPI-011` runs the six required wrapper commands and MAY be adopted as success evidence only when each row holds exit `0` and complete post-execution evidence. On the `KPI-011` path, the behavior of pre-closing Android / iOS platform commands as `PlatformCommandUnavailable` MUST NOT be repurposed as success evidence. A browser wrapper-local capability row MUST NOT be a substitute for an Android / iOS row. The `KPI-011` report MUST NOT adopt `cargo test --manifest-path tests/real-device/Cargo.toml ...success_matrix`, preflight-only output, exit `3` platform command unavailable output, or browser-only output as success evidence. All six rows of Android physical, Android emulator, iOS physical, iOS simulator, desktop browser, and mobile browser are required.

## 14. KPI-011 required device class set

| KPI-011 row id | Matrix evidence id | Platform group | Wrapper command | Device class | OS / browser class | Runtime version class | Network class | Expected outcome |
|---|---|---|---|---|---|---|---|---|
| `KPI-011-ANDROID-PHYSICAL` | `RD-ANDROID-001` | Android | `cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-physical` | `AndroidPhysical` | `AndroidApiLevel` | `AndroidApiLevel` | `LocalUsb` | wrapper records command exit status and evidence JSON |
| `KPI-011-ANDROID-EMULATOR` | `RD-ANDROID-002` | Android | `cargo run --manifest-path tests/real-device/Cargo.toml -- android --profile reference-local --device-class android-emulator` | `AndroidEmulator` | `AndroidApiLevel` | `AndroidApiLevel` | `EmulatorLoopback` | wrapper records command exit status and evidence JSON |
| `KPI-011-IOS-PHYSICAL` | `RD-IOS-001` | iOS | `cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-physical` | `IosPhysical` | `IosSystemVersion` | `IosSystemVersion` | `LocalUsb` | wrapper records command exit status and evidence JSON |
| `KPI-011-IOS-SIMULATOR` | `RD-IOS-002` | iOS | `cargo run --manifest-path tests/real-device/Cargo.toml -- ios --profile reference-local --device-class ios-simulator` | `IosSimulator` | `IosSystemVersion` | `IosSystemVersion` | `SimulatorLoopback` | wrapper records command exit status and evidence JSON |
| `KPI-011-BROWSER-DESKTOP` | `RD-BROWSER-001` | browser | `cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class desktop-browser` | `DesktopBrowser` | `BrowserVersion` | `BrowserVersion` | `LocalBrowser` | wrapper records command exit status and evidence JSON |
| `KPI-011-BROWSER-MOBILE` | `RD-BROWSER-002` | browser | `cargo run --manifest-path tests/real-device/Cargo.toml -- browser --profile reference-local --device-class mobile-browser` | `MobileBrowser` | `BrowserVersion` | `BrowserVersion` | `MobileBrowserEmulation` | wrapper records command exit status and evidence JSON |

The fixed scope of `KPI-011`: target KPI `KPI-011` / command owner `tests/real-device` wrapper / working directory `distro` / required platform groups Android, iOS, browser / required device class count 6 / success verdict rule (every required row has `exit_status == 0` and complete post-execution evidence) / failure classification `RealDeviceSuccessEvidenceMissing` / adoption scope real-device success evidence only.

## 15. Success evidence schema

The `KPI-011` report MUST hold the following fields per required device class row.

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

## 16. Success verdict rule

`verdict` MUST allow only `Pass` or `Fail`. `Pass` MUST be used only when each required row satisfies all of the following.

- The command exactly matches the wrapper command of the `Required Device Class Set`.
- `working_directory` is `distro`.
- `device_class`, `os_or_browser_class`, `runtime_version_class`, and `network_class` exactly match the `Required Device Class Set` row.
- `actual_outcome` exists as a post-execution field and is not an empty string.
- `exit_status` is `0`.
- `toolchain_runtime_version` is not empty.
- Android physical / Android emulator are observed by `adb devices -l` with the target class in the `device` state, not missing tool / offline / unauthorized / no device.
- iOS physical has an iPhone or iPad in the `== Devices ==` section of `xcrun xctrace list devices`, and a device in the `== Devices Offline ==` section MUST NOT be adopted as success.
- iOS simulator is observed by `xcrun simctl list devices` with the target simulator in the `(Booted)` state, and an enumeration of only `(Shutdown)` MUST NOT be success.
- `logs_metrics_location` is a non-empty path under `target/distro-evidence/real-device/`.
- `redacted_device_identifier` is literal `redacted` or `sha256:<64 lowercase hex characters>`.
- `raw_identifier_absence_statement` explicitly states the absence of raw identifier markers.

`Fail` MUST be used when any of the above is not satisfied.

## 17. Android emulator provider admission

The `AndroidEmulator` row MAY use an Android Studio managed AVD on the local Mac only when the AVD is launched through the Android SDK CLI and observed through the canonical `adb devices -l` surface. Admitted provider shape:

- AVD inventory command: `$HOME/Library/Android/sdk/emulator/emulator -list-avds`
- AVD launch command: `$HOME/Library/Android/sdk/emulator/emulator -avd <AVD_NAME> -no-window -no-audio -no-boot-anim -gpu swiftshader_indirect`
- boot completion check: `$HOME/Library/Android/sdk/platform-tools/adb shell getprop sys.boot_completed`
- adoption command: the wrapper command for `KPI-011-ANDROID-EMULATOR`
- observation surface: `adb devices -l`
- accepted observed shape: one `emulator-*` row whose device state column is `device`

Android Studio GUI state, device manager display, emulator process log, emulator gRPC log, crash log, public key log, and token-bearing log MUST NOT be used as `KPI-011` success evidence. The AVD provider does not satisfy `AndroidPhysical`. The AVD provider does not substitute for iOS physical, iOS simulator, desktop browser, or mobile browser rows.

## 18. Required report binding

| Required item | Required value |
|---|---|
| report filename | `<YYYYMMDD>-distro-KPI-011-REAL-DEVICE-SUCCESS-EVIDENCE.md` |
| correlation id prefix | `IMPL-KPI-011-REAL-DEVICE-SUCCESS-EVIDENCE` |
| required row count | 6 |
| Android rows | `KPI-011-ANDROID-PHYSICAL`, `KPI-011-ANDROID-EMULATOR` |
| iOS rows | `KPI-011-IOS-PHYSICAL`, `KPI-011-IOS-SIMULATOR` |
| browser rows | `KPI-011-BROWSER-DESKTOP`, `KPI-011-BROWSER-MOBILE` |
| success summary row | `KPI-011-R1` |

## 19. Rerun condition

If any of the following changes, `KPI-011` real-device success evidence MUST be rerun.

- real-device success specification
- real-device command matrix specification
- real-device wrapper command specification
- `tests/real-device` wrapper source
- Android / iOS / browser toolchain or runtime version
- connected device class
- network class
- evidence output path
- redaction rule

## 20. Fail-closed / collapse conditions

If any of the following holds, the real-device specification collapses (treated as fail-closed and not established).

- a platform command is run without a wrapper evidence record.
- a raw device identifier / token / private key is written to a report.
- real-device command success is used as live readiness / native application readiness.
- the wrapper working directory is outside `distro`.
- a device class is not in the closed set.
- the wrapper invokes an unlisted platform command.
- a missing platform tool is converted into success.
- the wrapper command omits the manifest path and depends on distro workspace membership.
- Android SDK path resolution changes the canonical command identity or command arguments.
- any of the required six device class rows is missing.
- a platform command result not mediated by the wrapper command is adopted.
- a row with `exit_status != 0` is treated as success.
- an offline / shutdown / unauthorized / no device row is treated as success.
- a row missing a required field is treated as `Pass`.
- real-device success is used as a substitute evidence for native application readiness / production readiness / live readiness / Kernel completion / freeze / full fixed-goal completion.
- evidence that does not record the network class is adopted.
- Kernel source is modified to establish real-device success.
