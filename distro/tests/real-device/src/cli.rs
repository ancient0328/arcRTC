//! real-device wrapper CLI parser です。

use clap::{Parser, ValueEnum};

/// real-device wrapper command arguments です。
#[derive(Debug, Parser)]
#[command(name = "arcrtc-real-device-wrapper")]
pub struct RealDeviceCli {
    /// target platform です。
    #[arg(value_enum)]
    pub platform: CliPlatform,
    /// profile name です。
    #[arg(long)]
    pub profile: String,
    /// device class です。
    #[arg(long, value_enum)]
    pub device_class: CliDeviceClass,
}

/// CLI platform closed set です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliPlatform {
    /// Android platform です。
    Android,
    /// iOS platform です。
    Ios,
    /// browser platform です。
    Browser,
}

/// CLI device class closed set です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliDeviceClass {
    /// Android physical device です。
    AndroidPhysical,
    /// Android emulator です。
    AndroidEmulator,
    /// iOS physical device です。
    IosPhysical,
    /// iOS simulator です。
    IosSimulator,
    /// desktop browser です。
    DesktopBrowser,
    /// mobile browser です。
    MobileBrowser,
}
