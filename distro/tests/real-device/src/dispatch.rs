//! real-device platform command closed-set dispatcher です。
#![allow(dead_code)]

use crate::{
    cli::{CliDeviceClass, CliPlatform},
    error::RealDeviceWrapperError,
    evidence::{
        RealDeviceClass, RealDeviceExecutionSurface, RealDeviceInternalCommandClass,
        RealDeviceNetworkClass, RealDevicePlatform, RealDeviceVersionClass,
    },
};

/// real-device dispatch context です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealDeviceDispatch {
    /// target platform です。
    pub platform: RealDevicePlatform,
    /// device class です。
    pub device_class: RealDeviceClass,
    /// internal command class です。
    pub internal_command_class: RealDeviceInternalCommandClass,
    /// runtime version class です。
    pub runtime_version_class: RealDeviceVersionClass,
    /// execution surface です。
    pub execution_surface: RealDeviceExecutionSurface,
    /// network class です。
    pub network_class: RealDeviceNetworkClass,
    /// platform command です。
    pub platform_command: Option<&'static str>,
}

/// CLI input を closed-set dispatch context に変換します。
pub fn dispatch_real_device_command(
    platform: CliPlatform,
    device_class: CliDeviceClass,
) -> Result<RealDeviceDispatch, RealDeviceWrapperError> {
    match (platform, device_class) {
        (CliPlatform::Android, CliDeviceClass::AndroidPhysical) => Ok(android(
            RealDeviceClass::AndroidPhysical,
            RealDeviceNetworkClass::LocalUsb,
        )),
        (CliPlatform::Android, CliDeviceClass::AndroidEmulator) => Ok(android(
            RealDeviceClass::AndroidEmulator,
            RealDeviceNetworkClass::EmulatorLoopback,
        )),
        (CliPlatform::Ios, CliDeviceClass::IosPhysical) => Ok(ios(
            RealDeviceClass::IosPhysical,
            RealDeviceInternalCommandClass::IosDevice,
            RealDeviceNetworkClass::LocalUsb,
            "xcrun xctrace list devices",
        )),
        (CliPlatform::Ios, CliDeviceClass::IosSimulator) => Ok(ios(
            RealDeviceClass::IosSimulator,
            RealDeviceInternalCommandClass::IosSimulator,
            RealDeviceNetworkClass::SimulatorLoopback,
            "xcrun simctl list devices",
        )),
        (CliPlatform::Browser, CliDeviceClass::DesktopBrowser) => Ok(browser(
            RealDeviceClass::DesktopBrowser,
            RealDeviceNetworkClass::LocalBrowser,
        )),
        (CliPlatform::Browser, CliDeviceClass::MobileBrowser) => Ok(browser(
            RealDeviceClass::MobileBrowser,
            RealDeviceNetworkClass::MobileBrowserEmulation,
        )),
        _ => Err(RealDeviceWrapperError::ScopeMismatch),
    }
}

/// KPI-T7 の bounded real-device command dispatch 境界です。
pub fn dispatch_kpi_bounded_real_device_command(
    platform: CliPlatform,
    device_class: CliDeviceClass,
) -> Result<RealDeviceDispatch, RealDeviceWrapperError> {
    dispatch_real_device_command(platform, device_class)
}

/// KPI-T11 の real-device success command dispatch 境界です。
pub fn dispatch_kpi_real_device_success_command(
    platform: CliPlatform,
    device_class: CliDeviceClass,
) -> Result<RealDeviceDispatch, RealDeviceWrapperError> {
    dispatch_real_device_command(platform, device_class)
}

fn android(
    device_class: RealDeviceClass,
    network_class: RealDeviceNetworkClass,
) -> RealDeviceDispatch {
    RealDeviceDispatch {
        platform: RealDevicePlatform::Android,
        device_class,
        internal_command_class: RealDeviceInternalCommandClass::AndroidDevice,
        runtime_version_class: RealDeviceVersionClass::AndroidApiLevel,
        execution_surface: RealDeviceExecutionSurface::NativeSdkCommand,
        network_class,
        platform_command: Some("adb devices -l"),
    }
}

fn ios(
    device_class: RealDeviceClass,
    internal_command_class: RealDeviceInternalCommandClass,
    network_class: RealDeviceNetworkClass,
    command: &'static str,
) -> RealDeviceDispatch {
    RealDeviceDispatch {
        platform: RealDevicePlatform::Ios,
        device_class,
        internal_command_class,
        runtime_version_class: RealDeviceVersionClass::IosSystemVersion,
        execution_surface: RealDeviceExecutionSurface::NativeSdkCommand,
        network_class,
        platform_command: Some(command),
    }
}

fn browser(
    device_class: RealDeviceClass,
    network_class: RealDeviceNetworkClass,
) -> RealDeviceDispatch {
    RealDeviceDispatch {
        platform: RealDevicePlatform::Browser,
        device_class,
        internal_command_class: RealDeviceInternalCommandClass::Browser,
        runtime_version_class: RealDeviceVersionClass::BrowserVersion,
        execution_surface: RealDeviceExecutionSurface::BrowserWebrtcCommand,
        network_class,
        platform_command: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_unit_covers_kpi_aliases_and_scope_mismatch() {
        assert!(dispatch_kpi_bounded_real_device_command(
            CliPlatform::Browser,
            CliDeviceClass::DesktopBrowser,
        )
        .is_ok());
        assert!(dispatch_kpi_real_device_success_command(
            CliPlatform::Browser,
            CliDeviceClass::MobileBrowser,
        )
        .is_ok());
        assert!(dispatch_kpi_bounded_real_device_command(
            CliPlatform::Android,
            CliDeviceClass::IosPhysical,
        )
        .is_err());
    }
}
