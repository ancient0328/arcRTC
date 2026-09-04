//! real-device platform command の closed-set executor です。
#![allow(dead_code)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use crate::{
    command_output::{RealDeviceCommandOutput, RealDeviceCommandUnavailable},
    dispatch::RealDeviceDispatch,
};

/// closed command runner の抽象境界です。
pub trait RealDeviceCommandRunner {
    /// program と args を実行し、stdout / stderr / raw exit status を返します。
    fn run(
        &self,
        program: &str,
        args: &[&str],
    ) -> Result<RealDeviceProcessOutput, RealDeviceCommandUnavailable>;
}

/// platform command の process output です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealDeviceProcessOutput {
    /// stdout です。
    pub stdout: String,
    /// stderr です。
    pub stderr: String,
    /// raw exit status です。
    pub exit_status: Option<i32>,
}

/// 標準process runnerです。
#[derive(Clone, Copy, Debug, Default)]
pub struct StdRealDeviceCommandRunner;

impl RealDeviceCommandRunner for StdRealDeviceCommandRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
    ) -> Result<RealDeviceProcessOutput, RealDeviceCommandUnavailable> {
        let output = Command::new(resolve_program(program))
            .args(args)
            .output()
            .map_err(classify_spawn_error)?;
        Ok(RealDeviceProcessOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_status: output.status.code(),
        })
    }
}

/// real-device closed-set executor です。
#[derive(Clone, Copy, Debug, Default)]
pub struct RealDevicePlatformExecutor;

impl RealDevicePlatformExecutor {
    /// default runner で dispatch を実行します。
    pub fn execute(dispatch: &RealDeviceDispatch) -> RealDeviceCommandOutput {
        Self::execute_with_runner(dispatch, &StdRealDeviceCommandRunner)
    }

    /// test runner を注入して dispatch を実行します。
    pub fn execute_with_runner<R: RealDeviceCommandRunner>(
        dispatch: &RealDeviceDispatch,
        runner: &R,
    ) -> RealDeviceCommandOutput {
        let Some(command) = closed_command(dispatch.platform_command) else {
            return RealDeviceCommandOutput::success(
                "browser wrapper local capability observed",
                "",
                "browser-wrapper-local",
            );
        };
        let runtime = command.runtime_summary();
        match runner.run(command.program, command.args) {
            Ok(output) if output.exit_status == Some(0) => {
                RealDeviceCommandOutput::success(output.stdout, output.stderr, runtime)
            }
            Ok(output) => RealDeviceCommandOutput::unavailable(
                RealDeviceCommandUnavailable::NonZeroExit,
                output.stdout,
                output.stderr,
                output.exit_status,
                runtime,
            ),
            Err(reason) => RealDeviceCommandOutput::unavailable(reason, "", "", None, runtime),
        }
    }
}

/// canonical closed command だけを実行する public helper です。
pub fn execute_real_device_platform_command(
    dispatch: &RealDeviceDispatch,
) -> RealDeviceCommandOutput {
    RealDevicePlatformExecutor::execute(dispatch)
}

struct ClosedPlatformCommand {
    program: &'static str,
    args: &'static [&'static str],
}

impl ClosedPlatformCommand {
    fn runtime_summary(&self) -> &'static str {
        match (self.program, self.args) {
            ("adb", ["devices", "-l"]) => "adb devices -l",
            ("xcrun", ["devicectl", "list", "devices"]) => "xcrun devicectl list devices",
            ("xcrun", ["xctrace", "list", "devices"]) => "xcrun xctrace list devices",
            ("xcrun", ["simctl", "list", "devices"]) => "xcrun simctl list devices",
            _ => "real-device-platform-command",
        }
    }
}

fn closed_command(command: Option<&'static str>) -> Option<ClosedPlatformCommand> {
    match command {
        Some("adb devices -l") => Some(ClosedPlatformCommand {
            program: "adb",
            args: &["devices", "-l"],
        }),
        Some("xcrun devicectl list devices") => Some(ClosedPlatformCommand {
            program: "xcrun",
            args: &["devicectl", "list", "devices"],
        }),
        Some("xcrun xctrace list devices") => Some(ClosedPlatformCommand {
            program: "xcrun",
            args: &["xctrace", "list", "devices"],
        }),
        Some("xcrun simctl list devices") => Some(ClosedPlatformCommand {
            program: "xcrun",
            args: &["simctl", "list", "devices"],
        }),
        Some(_) => None,
        None => None,
    }
}

fn classify_spawn_error(error: std::io::Error) -> RealDeviceCommandUnavailable {
    if error.kind() == std::io::ErrorKind::NotFound {
        RealDeviceCommandUnavailable::MissingProgram
    } else {
        RealDeviceCommandUnavailable::SpawnFailed
    }
}

fn resolve_program(program: &str) -> OsString {
    if program != "adb" {
        return OsString::from(program);
    }

    // adb は Android SDK の標準配置にある場合がある。command shape は
    // `adb devices -l` のまま固定し、program 解決だけを実行環境境界へ閉じる。
    for sdk_root in android_sdk_root_candidates() {
        let adb = sdk_root.join("platform-tools").join("adb");
        if adb.is_file() {
            return adb.into_os_string();
        }
    }

    OsString::from(program)
}

fn android_sdk_root_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for key in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Some(value) = std::env::var_os(key) {
            candidates.push(PathBuf::from(value));
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join("Library/Android/sdk"));
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        command_output::{RealDeviceCommandExitStatus, RealDeviceCommandUnavailable},
        evidence::{
            RealDeviceClass, RealDeviceExecutionSurface, RealDeviceInternalCommandClass,
            RealDeviceNetworkClass, RealDevicePlatform, RealDeviceVersionClass,
        },
    };
    use std::sync::{Mutex, OnceLock};

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

    struct Runner {
        result: Result<RealDeviceProcessOutput, RealDeviceCommandUnavailable>,
    }

    impl RealDeviceCommandRunner for Runner {
        fn run(
            &self,
            _program: &str,
            _args: &[&str],
        ) -> Result<RealDeviceProcessOutput, RealDeviceCommandUnavailable> {
            self.result.clone()
        }
    }

    fn dispatch(command: Option<&'static str>) -> RealDeviceDispatch {
        RealDeviceDispatch {
            platform: if command.is_some() {
                RealDevicePlatform::Android
            } else {
                RealDevicePlatform::Browser
            },
            device_class: if command.is_some() {
                RealDeviceClass::AndroidPhysical
            } else {
                RealDeviceClass::DesktopBrowser
            },
            internal_command_class: if command.is_some() {
                RealDeviceInternalCommandClass::AndroidDevice
            } else {
                RealDeviceInternalCommandClass::Browser
            },
            runtime_version_class: if command.is_some() {
                RealDeviceVersionClass::AndroidApiLevel
            } else {
                RealDeviceVersionClass::BrowserVersion
            },
            execution_surface: if command.is_some() {
                RealDeviceExecutionSurface::NativeSdkCommand
            } else {
                RealDeviceExecutionSurface::BrowserWebrtcCommand
            },
            network_class: if command.is_some() {
                RealDeviceNetworkClass::LocalUsb
            } else {
                RealDeviceNetworkClass::LocalBrowser
            },
            platform_command: command,
        }
    }

    #[test]
    fn platform_executor_unit_covers_browser_success_nonzero_and_unavailable_paths() {
        let browser = RealDevicePlatformExecutor::execute_with_runner(
            &dispatch(None),
            &Runner {
                result: Err(RealDeviceCommandUnavailable::CommandNotAdmitted),
            },
        );
        assert_eq!(browser.exit_status, RealDeviceCommandExitStatus::Success);
        assert_eq!(browser.toolchain_runtime_version, "browser-wrapper-local");

        let success = RealDevicePlatformExecutor::execute_with_runner(
            &dispatch(Some("adb devices -l")),
            &Runner {
                result: Ok(RealDeviceProcessOutput {
                    stdout: "device".to_owned(),
                    stderr: String::new(),
                    exit_status: Some(0),
                }),
            },
        );
        assert_eq!(success.exit_status, RealDeviceCommandExitStatus::Success);
        assert_eq!(success.toolchain_runtime_version, "adb devices -l");

        let nonzero = RealDevicePlatformExecutor::execute_with_runner(
            &dispatch(Some("xcrun devicectl list devices")),
            &Runner {
                result: Ok(RealDeviceProcessOutput {
                    stdout: String::new(),
                    stderr: "failed".to_owned(),
                    exit_status: Some(1),
                }),
            },
        );
        assert_eq!(
            nonzero.exit_status,
            RealDeviceCommandExitStatus::PlatformCommandUnavailable
        );
        assert_eq!(
            nonzero.unavailable,
            Some(RealDeviceCommandUnavailable::NonZeroExit)
        );
        assert_eq!(
            nonzero.toolchain_runtime_version,
            "xcrun devicectl list devices"
        );

        let unavailable = RealDevicePlatformExecutor::execute_with_runner(
            &dispatch(Some("xcrun simctl list devices")),
            &Runner {
                result: Err(RealDeviceCommandUnavailable::TimedOut),
            },
        );
        assert_eq!(
            unavailable.exit_status,
            RealDeviceCommandExitStatus::PlatformCommandUnavailable
        );
        assert_eq!(
            unavailable.unavailable,
            Some(RealDeviceCommandUnavailable::TimedOut)
        );
        assert_eq!(
            unavailable.toolchain_runtime_version,
            "xcrun simctl list devices"
        );
    }

    #[test]
    fn platform_executor_unit_covers_closed_command_and_spawn_helpers() {
        for (command, runtime) in [
            (Some("adb devices -l"), "adb devices -l"),
            (
                Some("xcrun devicectl list devices"),
                "xcrun devicectl list devices",
            ),
            (
                Some("xcrun xctrace list devices"),
                "xcrun xctrace list devices",
            ),
            (
                Some("xcrun simctl list devices"),
                "xcrun simctl list devices",
            ),
        ] {
            let closed = closed_command(command).expect("closed command must be admitted");
            assert_eq!(closed.runtime_summary(), runtime);
        }
        assert!(closed_command(Some("adb shell getprop")).is_none());
        assert!(closed_command(None).is_none());

        assert_eq!(
            classify_spawn_error(std::io::Error::new(std::io::ErrorKind::NotFound, "missing")),
            RealDeviceCommandUnavailable::MissingProgram
        );
        assert_eq!(
            classify_spawn_error(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "denied"
            )),
            RealDeviceCommandUnavailable::SpawnFailed
        );

        assert_eq!(resolve_program("xcrun"), OsString::from("xcrun"));
        let _ = android_sdk_root_candidates();
    }

    #[test]
    fn platform_executor_unit_covers_std_runner_and_resolve_program_paths() {
        let runner = StdRealDeviceCommandRunner;
        let output = runner
            .run("true", &[])
            .expect("true command should execute on the local test host");
        assert_eq!(output.exit_status, Some(0));

        assert_eq!(
            runner.run("arcrtc-command-that-must-not-exist", &[]),
            Err(RealDeviceCommandUnavailable::MissingProgram)
        );

        let fallback = ClosedPlatformCommand {
            program: "custom",
            args: &["arg"],
        };
        assert_eq!(fallback.runtime_summary(), "real-device-platform-command");

        assert_eq!(
            resolve_program("arcrtc-command-that-must-not-exist"),
            OsString::from("arcrtc-command-that-must-not-exist")
        );
        assert!(!resolve_program("adb").is_empty());
    }

    #[test]
    fn platform_executor_unit_covers_android_sdk_adb_resolution() {
        let _guard = env_lock().lock().expect("env lock");
        let previous_android_home = std::env::var_os("ANDROID_HOME");
        let previous_android_sdk_root = std::env::var_os("ANDROID_SDK_ROOT");
        let sdk_root = std::env::temp_dir().join(format!(
            "arcrtc-real-device-fake-sdk-{}",
            std::process::id()
        ));
        let platform_tools = sdk_root.join("platform-tools");
        std::fs::create_dir_all(&platform_tools).expect("fake platform-tools dir");
        let adb = platform_tools.join("adb");
        std::fs::write(&adb, "#!/bin/sh\nexit 0\n").expect("fake adb");

        std::env::set_var("ANDROID_HOME", &sdk_root);
        assert_eq!(resolve_program("adb"), adb.into_os_string());
        restore_env_var("ANDROID_HOME", previous_android_home);
        restore_env_var("ANDROID_SDK_ROOT", previous_android_sdk_root);
    }

    #[test]
    fn platform_executor_unit_covers_adb_resolution_without_sdk_candidates() {
        let _guard = env_lock().lock().expect("env lock");
        let previous_android_home = std::env::var_os("ANDROID_HOME");
        let previous_android_sdk_root = std::env::var_os("ANDROID_SDK_ROOT");
        let previous_home = std::env::var_os("HOME");
        let home = std::env::temp_dir().join(format!(
            "arcrtc-real-device-empty-home-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&home).expect("empty home dir");

        std::env::remove_var("ANDROID_HOME");
        std::env::remove_var("ANDROID_SDK_ROOT");
        std::env::set_var("HOME", &home);

        assert_eq!(resolve_program("adb"), OsString::from("adb"));

        restore_env_var("ANDROID_HOME", previous_android_home);
        restore_env_var("ANDROID_SDK_ROOT", previous_android_sdk_root);
        restore_env_var("HOME", previous_home);
    }

    #[test]
    fn platform_executor_unit_covers_adb_resolution_without_home_candidate() {
        let _guard = env_lock().lock().expect("env lock");
        let previous_android_home = std::env::var_os("ANDROID_HOME");
        let previous_android_sdk_root = std::env::var_os("ANDROID_SDK_ROOT");
        let previous_home = std::env::var_os("HOME");

        std::env::remove_var("ANDROID_HOME");
        std::env::remove_var("ANDROID_SDK_ROOT");
        std::env::remove_var("HOME");

        assert!(android_sdk_root_candidates().is_empty());
        assert_eq!(resolve_program("adb"), OsString::from("adb"));

        restore_env_var("ANDROID_HOME", previous_android_home);
        restore_env_var("ANDROID_SDK_ROOT", previous_android_sdk_root);
        restore_env_var("HOME", previous_home);
    }

    #[test]
    fn platform_executor_unit_covers_env_restore_helper_branches() {
        let _guard = env_lock().lock().expect("env lock");
        let key = "ARCRTC_REAL_DEVICE_RESTORE_HELPER_TEST";
        std::env::remove_var(key);

        restore_env_var(key, Some(OsString::from("restored")));
        assert_eq!(std::env::var_os(key), Some(OsString::from("restored")));

        restore_env_var(key, None);
        assert_eq!(std::env::var_os(key), None);
    }
}
