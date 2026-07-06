//! real-device command output から観測結果へ写像する境界です。
#![allow(dead_code)]

use crate::{
    command_output::{RealDeviceCommandExitStatus, RealDeviceCommandOutput},
    dispatch::RealDeviceDispatch,
    evidence::{RealDeviceClass, RealDeviceNetworkClass, RealDeviceVersionClass},
    redaction::redact_real_device_identifier,
};

/// real-device observation です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealDeviceObservation {
    /// 観測された device class です。
    pub observed_device_class: RealDeviceClass,
    /// runtime version class です。
    pub runtime_version_class: RealDeviceVersionClass,
    /// network class です。
    pub network_class: RealDeviceNetworkClass,
    /// toolchain / runtime version summary です。
    pub toolchain_runtime_version: String,
    /// 証跡用 redacted identifier です。
    pub redacted_device_identifier: String,
}

/// real-device observation error の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealDeviceObservationError {
    /// command 自体が success ではありません。
    CommandNotSuccessful,
    /// 必須 device class が観測できません。
    RequiredDeviceNotObserved,
    /// redaction に失敗しました。
    RedactionFailed,
}

/// dispatch と command output から real-device observation を構築します。
pub fn parse_real_device_observation(
    dispatch: &RealDeviceDispatch,
    output: &RealDeviceCommandOutput,
) -> Result<RealDeviceObservation, RealDeviceObservationError> {
    if output.exit_status != RealDeviceCommandExitStatus::Success {
        return Err(RealDeviceObservationError::CommandNotSuccessful);
    }
    if !is_required_device_observed(dispatch.device_class, &output.stdout_summary) {
        return Err(RealDeviceObservationError::RequiredDeviceNotObserved);
    }
    let redacted_device_identifier = redact_real_device_identifier(None)
        .map_err(|_| RealDeviceObservationError::RedactionFailed)?;
    Ok(RealDeviceObservation {
        observed_device_class: dispatch.device_class,
        runtime_version_class: dispatch.runtime_version_class,
        network_class: dispatch.network_class,
        toolchain_runtime_version: output.toolchain_runtime_version.clone(),
        redacted_device_identifier,
    })
}

fn is_required_device_observed(device_class: RealDeviceClass, stdout: &str) -> bool {
    match device_class {
        RealDeviceClass::AndroidPhysical => observation_segments(stdout)
            .any(|line| android_device_state_is_device(line) && !line.starts_with("emulator-")),
        RealDeviceClass::AndroidEmulator => observation_segments(stdout)
            .any(|line| line.starts_with("emulator-") && android_device_state_is_device(line)),
        RealDeviceClass::IosPhysical => ios_physical_device_is_reachable(stdout),
        RealDeviceClass::IosSimulator => observation_segments(stdout).any(|line| {
            // simulator は列挙されるだけでは success にしない。Booted のみ実行可能状態として採用する。
            line.to_ascii_lowercase().contains("(booted)")
        }),
        RealDeviceClass::DesktopBrowser | RealDeviceClass::MobileBrowser => true,
    }
}

fn android_device_state_is_device(line: &str) -> bool {
    let mut columns = line.split_whitespace();
    let Some(_device_id) = columns.next() else {
        return false;
    };
    matches!(columns.next(), Some("device"))
}

fn ios_physical_device_is_reachable(stdout: &str) -> bool {
    let mut in_online_devices_section = false;
    for line in observation_segments(stdout) {
        let lower = line.to_ascii_lowercase();
        if lower == "== devices ==" {
            in_online_devices_section = true;
            continue;
        }
        if lower == "== devices offline ==" || lower == "== simulators ==" {
            in_online_devices_section = false;
            continue;
        }
        if in_online_devices_section
            && (lower.contains("iphone") || lower.contains("ipad"))
            && !lower.contains("simulator")
        {
            return true;
        }
    }
    false
}

fn observation_segments(stdout: &str) -> impl Iterator<Item = &str> {
    stdout
        .split('|')
        .flat_map(str::lines)
        .map(str::trim)
        .filter(|line| !line.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        command_output::{RealDeviceCommandOutput, RealDeviceCommandUnavailable},
        dispatch::RealDeviceDispatch,
        evidence::{
            RealDeviceClass, RealDeviceExecutionSurface, RealDeviceInternalCommandClass,
            RealDeviceNetworkClass, RealDevicePlatform, RealDeviceVersionClass,
        },
    };

    fn dispatch(device_class: RealDeviceClass) -> RealDeviceDispatch {
        RealDeviceDispatch {
            platform: RealDevicePlatform::Ios,
            device_class,
            internal_command_class: RealDeviceInternalCommandClass::IosDevice,
            runtime_version_class: RealDeviceVersionClass::IosSystemVersion,
            execution_surface: RealDeviceExecutionSurface::NativeSdkCommand,
            network_class: RealDeviceNetworkClass::LocalUsb,
            platform_command: Some("xcrun xctrace list devices"),
        }
    }

    #[test]
    fn observation_unit_covers_error_and_segment_branches() {
        let android = RealDeviceDispatch {
            platform: RealDevicePlatform::Android,
            device_class: RealDeviceClass::AndroidPhysical,
            internal_command_class: RealDeviceInternalCommandClass::AndroidDevice,
            runtime_version_class: RealDeviceVersionClass::AndroidApiLevel,
            execution_surface: RealDeviceExecutionSurface::NativeSdkCommand,
            network_class: RealDeviceNetworkClass::LocalUsb,
            platform_command: Some("adb devices -l"),
        };
        assert_eq!(
            parse_real_device_observation(
                &android,
                &RealDeviceCommandOutput::unavailable(
                    RealDeviceCommandUnavailable::MissingProgram,
                    "",
                    "",
                    None,
                    "adb devices -l",
                ),
            ),
            Err(RealDeviceObservationError::CommandNotSuccessful)
        );
        assert!(!android_device_state_is_device(""));
        assert!(!android_device_state_is_device("ABC offline"));

        let ios_physical = dispatch(RealDeviceClass::IosPhysical);
        assert_eq!(
            parse_real_device_observation(
                &ios_physical,
                &RealDeviceCommandOutput::success(
                    "== Devices Offline == | User iPhone (26.5)",
                    "",
                    "xcrun xctrace list devices",
                ),
            ),
            Err(RealDeviceObservationError::RequiredDeviceNotObserved)
        );
        assert!(!ios_physical_device_is_reachable(
            "== Devices == | iPhone Simulator (Booted)"
        ));
        assert!(!ios_physical_device_is_reachable(
            "== Devices == | == Devices Offline == | User iPhone"
        ));

        let android_emulator = RealDeviceDispatch {
            platform: RealDevicePlatform::Android,
            device_class: RealDeviceClass::AndroidEmulator,
            internal_command_class: RealDeviceInternalCommandClass::AndroidDevice,
            runtime_version_class: RealDeviceVersionClass::AndroidApiLevel,
            execution_surface: RealDeviceExecutionSurface::NativeSdkCommand,
            network_class: RealDeviceNetworkClass::EmulatorLoopback,
            platform_command: Some("adb devices -l"),
        };
        assert_eq!(
            parse_real_device_observation(
                &android_emulator,
                &RealDeviceCommandOutput::success("ABC123 device", "", "adb devices -l"),
            ),
            Err(RealDeviceObservationError::RequiredDeviceNotObserved)
        );
        assert_eq!(
            parse_real_device_observation(
                &android_emulator,
                &RealDeviceCommandOutput::success("emulator-5554 offline", "", "adb devices -l"),
            ),
            Err(RealDeviceObservationError::RequiredDeviceNotObserved)
        );
        assert!(android_device_state_is_device(
            "ABC123 device product:pixel"
        ));
        assert!(parse_real_device_observation(
            &ios_physical,
            &RealDeviceCommandOutput::success(
                "== Devices == | User iPhone (26.5) | == Simulators ==",
                "",
                "xcrun xctrace list devices",
            ),
        )
        .is_ok());
        assert!(ios_physical_device_is_reachable(
            "== Devices == | User iPad (26.5) | == Simulators =="
        ));

        let ios_simulator = dispatch(RealDeviceClass::IosSimulator);
        assert_eq!(
            parse_real_device_observation(
                &ios_simulator,
                &RealDeviceCommandOutput::success(
                    "== Devices == | iPhone 16 Pro (Shutdown)",
                    "",
                    "xcrun simctl list devices",
                ),
            ),
            Err(RealDeviceObservationError::RequiredDeviceNotObserved)
        );
        assert!(parse_real_device_observation(
            &ios_simulator,
            &RealDeviceCommandOutput::success(
                "== Devices == | iPhone 16 Pro (Booted)",
                "",
                "xcrun simctl list devices",
            ),
        )
        .is_ok());
    }
}
