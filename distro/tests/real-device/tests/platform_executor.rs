//! KPI-T11S real-device platform executor の closed command 境界を検査します。

use std::{
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/command_output.rs"]
mod command_output;
#[path = "../src/device_observation.rs"]
mod device_observation;
#[path = "../src/dispatch.rs"]
mod dispatch;
#[path = "../src/error.rs"]
mod error;
#[path = "../src/evidence.rs"]
mod evidence;
#[path = "../src/platform_executor.rs"]
mod platform_executor;
#[path = "../src/redaction.rs"]
mod redaction;

use cli::{CliDeviceClass, CliPlatform};
use command_output::{
    RealDeviceCommandExitStatus, RealDeviceCommandOutput, RealDeviceCommandUnavailable,
};
use device_observation::{parse_real_device_observation, RealDeviceObservationError};
use dispatch::dispatch_kpi_real_device_success_command;
use platform_executor::{
    RealDeviceCommandRunner, RealDevicePlatformExecutor, RealDeviceProcessOutput,
    StdRealDeviceCommandRunner,
};

struct ExpectingRunner {
    program: &'static str,
    args: &'static [&'static str],
    output: RealDeviceProcessOutput,
}

impl RealDeviceCommandRunner for ExpectingRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
    ) -> Result<RealDeviceProcessOutput, RealDeviceCommandUnavailable> {
        assert_eq!(program, self.program);
        assert_eq!(args, self.args);
        Ok(self.output.clone())
    }
}

struct UnavailableRunner;

impl RealDeviceCommandRunner for UnavailableRunner {
    fn run(
        &self,
        _program: &str,
        _args: &[&str],
    ) -> Result<RealDeviceProcessOutput, RealDeviceCommandUnavailable> {
        Err(RealDeviceCommandUnavailable::MissingProgram)
    }
}

struct PanicRunner;

impl RealDeviceCommandRunner for PanicRunner {
    fn run(
        &self,
        _program: &str,
        _args: &[&str],
    ) -> Result<RealDeviceProcessOutput, RealDeviceCommandUnavailable> {
        panic!("browser wrapper local capability must not execute platform command")
    }
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn restore_env_var(key: &str, previous: Option<OsString>) {
    match previous {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn temporary_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("arcrtc-real-device-{name}-{}", std::process::id()))
}

#[test]
fn kpi_real_device_platform_executor_runs_only_closed_commands() {
    let android = dispatch_kpi_real_device_success_command(
        CliPlatform::Android,
        CliDeviceClass::AndroidPhysical,
    )
    .expect("android physical dispatch");
    let android_output = RealDevicePlatformExecutor::execute_with_runner(
        &android,
        &ExpectingRunner {
            program: "adb",
            args: &["devices", "-l"],
            output: RealDeviceProcessOutput {
                stdout: "List of devices attached\nABC123\tdevice product:pixel".to_owned(),
                stderr: String::new(),
                exit_status: Some(0),
            },
        },
    );
    assert_eq!(
        android_output.exit_status,
        RealDeviceCommandExitStatus::Success
    );

    let ios_simulator =
        dispatch_kpi_real_device_success_command(CliPlatform::Ios, CliDeviceClass::IosSimulator)
            .expect("ios simulator dispatch");
    let ios_output = RealDevicePlatformExecutor::execute_with_runner(
        &ios_simulator,
        &ExpectingRunner {
            program: "xcrun",
            args: &["simctl", "list", "devices"],
            output: RealDeviceProcessOutput {
                stdout: "iPhone 15 Simulator (Booted)".to_owned(),
                stderr: String::new(),
                exit_status: Some(0),
            },
        },
    );
    assert_eq!(ios_output.exit_status, RealDeviceCommandExitStatus::Success);

    let browser = dispatch_kpi_real_device_success_command(
        CliPlatform::Browser,
        CliDeviceClass::MobileBrowser,
    )
    .expect("browser dispatch");
    let browser_output = RealDevicePlatformExecutor::execute_with_runner(&browser, &PanicRunner);
    assert_eq!(
        browser_output.exit_status,
        RealDeviceCommandExitStatus::Success
    );

    let unavailable = RealDevicePlatformExecutor::execute_with_runner(&android, &UnavailableRunner);
    assert_eq!(
        unavailable.exit_status,
        RealDeviceCommandExitStatus::PlatformCommandUnavailable
    );
    assert_eq!(
        unavailable.unavailable,
        Some(RealDeviceCommandUnavailable::MissingProgram)
    );

    assert!(
        parse_real_device_observation(
            &android,
            &RealDeviceCommandOutput::success(
                "List of devices attached\nemulator-5554          device product:sdk_gphone64_arm64",
                "",
                "adb devices -l",
            ),
        )
        .is_err(),
        "physical Android row must not accept emulator inventory"
    );

    let android_emulator = dispatch_kpi_real_device_success_command(
        CliPlatform::Android,
        CliDeviceClass::AndroidEmulator,
    )
    .expect("android emulator dispatch");
    assert!(
        parse_real_device_observation(
            &android_emulator,
            &RealDeviceCommandOutput::success(
                "List of devices attached\nemulator-5554          device product:sdk_gphone64_arm64",
                "",
                "adb devices -l",
            ),
        )
        .is_ok(),
        "emulator row must accept whitespace-separated adb device state"
    );

    let ios_physical =
        dispatch_kpi_real_device_success_command(CliPlatform::Ios, CliDeviceClass::IosPhysical)
            .expect("ios physical dispatch");
    let offline_physical = RealDeviceCommandOutput::success(
        "== Devices ==\nMacBookPro (host)\n== Devices Offline ==\nUser iPhone (26.5)",
        "",
        "xcrun xctrace list devices",
    );
    assert_eq!(
        parse_real_device_observation(&ios_physical, &offline_physical),
        Err(RealDeviceObservationError::RequiredDeviceNotObserved)
    );

    let shutdown_simulator = RealDeviceCommandOutput::success(
        "== Devices ==\n    iPhone 16 Pro (ABC) (Shutdown)",
        "",
        "xcrun simctl list devices",
    );
    assert_eq!(
        parse_real_device_observation(&ios_simulator, &shutdown_simulator),
        Err(RealDeviceObservationError::RequiredDeviceNotObserved)
    );

    let booted_simulator = RealDeviceCommandOutput::success(
        "== Devices ==\n    iPhone 16 Pro (ABC) (Booted)",
        "",
        "xcrun simctl list devices",
    );
    assert!(parse_real_device_observation(&ios_simulator, &booted_simulator).is_ok());
}

#[test]
fn kpi_real_device_platform_executor_covers_std_runner_spawn_and_env_resolution_branches() {
    let _guard = env_lock().lock().expect("env lock");
    let runner = StdRealDeviceCommandRunner;

    let true_output = runner
        .run("true", &[])
        .expect("true command must execute on the local host");
    assert_eq!(true_output.exit_status, Some(0));

    assert_eq!(
        runner.run("arcrtc-command-that-must-not-exist", &[]),
        Err(RealDeviceCommandUnavailable::MissingProgram)
    );

    let denied = temporary_path("permission-denied-command");
    fs::write(&denied, "#!/bin/sh\nexit 0\n").expect("denied command fixture must be written");
    let mut permissions = fs::metadata(&denied)
        .expect("denied command metadata")
        .permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(&denied, permissions).expect("denied command mode must be set");
    assert_eq!(
        runner.run(
            denied
                .to_str()
                .expect("denied command path must be representable as UTF-8"),
            &[]
        ),
        Err(RealDeviceCommandUnavailable::SpawnFailed)
    );

    let previous_android_home = std::env::var_os("ANDROID_HOME");
    let previous_android_sdk_root = std::env::var_os("ANDROID_SDK_ROOT");
    let previous_home = std::env::var_os("HOME");
    let previous_path = std::env::var_os("PATH");
    let empty_path = temporary_path("empty-path");
    fs::create_dir_all(&empty_path).expect("empty PATH fixture must be created");

    std::env::remove_var("ANDROID_HOME");
    std::env::remove_var("ANDROID_SDK_ROOT");
    std::env::remove_var("HOME");
    std::env::set_var("PATH", &empty_path);
    assert_eq!(
        runner.run("adb", &[]),
        Err(RealDeviceCommandUnavailable::MissingProgram)
    );

    restore_env_var("ANDROID_HOME", previous_android_home);
    restore_env_var("ANDROID_SDK_ROOT", previous_android_sdk_root);
    restore_env_var("HOME", previous_home);
    restore_env_var("PATH", previous_path);
}
