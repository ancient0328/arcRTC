//! real-device wrapper command 境界を検査します。

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
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
#[path = "../src/redaction.rs"]
mod redaction;
#[path = "../src/wrapper.rs"]
mod wrapper;

use arcrtc_distro_evidence::{DistroEvidenceReason, DistroNonClaimScope};
use cli::{CliDeviceClass, CliPlatform};
use dispatch::{dispatch_kpi_bounded_real_device_command, dispatch_real_device_command};
use evidence::{
    RealDeviceEvidenceValidationError, REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE,
    REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
};
use serde_json::Value;
use wrapper::{
    all_wrapper_exit_codes, build_kpi_bounded_real_device_record, build_real_device_record,
    distro_reason_for_exit, is_current_working_directory_distro_root, planned_exit_for_dispatch,
    validated_exit_code, RealDeviceWrapperExit,
};

fn run_wrapper(platform: &str, device_class: &str, profile: &str) -> std::process::Output {
    let isolated_root = isolated_distro_root();
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-distro-real-device-tests"))
        .current_dir(&isolated_root)
        .args([
            platform,
            "--profile",
            profile,
            "--device-class",
            device_class,
        ])
        .output()
        .expect("wrapper binary must execute");
    let _ = fs::remove_dir_all(isolated_root);
    output
}

fn isolated_distro_root() -> PathBuf {
    static NEXT_ROOT_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_ROOT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir()
        .join(format!(
            "arcrtc-real-device-wrapper-{id}-{}",
            std::process::id()
        ))
        .join("distro");
    fs::create_dir_all(&root).expect("isolated distro root must be created");
    root
}

fn fake_android_sdk_with_adb(script: &str) -> PathBuf {
    static NEXT_SDK_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_SDK_ID.fetch_add(1, Ordering::Relaxed);
    let sdk_root = std::env::temp_dir().join(format!(
        "arcrtc-real-device-wrapper-fake-sdk-{}-{id}",
        std::process::id(),
    ));
    let platform_tools = sdk_root.join("platform-tools");
    fs::create_dir_all(&platform_tools).expect("fake platform-tools must be created");
    let adb = platform_tools.join("adb");
    fs::write(&adb, script).expect("fake adb script must be written");
    let mut permissions = fs::metadata(&adb).expect("fake adb metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&adb, permissions).expect("fake adb must be executable");
    sdk_root
}

fn stdout_json(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("wrapper stdout must be evidence json")
}

fn assert_json_non_claim_scope(json: &Value) {
    let scopes = json["non_claim_scope"]
        .as_array()
        .expect("non_claim_scope must be array");
    for expected in [
        DistroNonClaimScope::NativeApplicationReadinessNotClaimed,
        DistroNonClaimScope::PublicDistributionReadinessNotClaimed,
        DistroNonClaimScope::ProductionReadinessNotClaimed,
        DistroNonClaimScope::LiveReadinessNotClaimed,
        DistroNonClaimScope::KernelCompletionNotClaimed,
    ] {
        let expected = serde_json::to_value(expected).expect("scope must serialize");
        assert!(scopes.contains(&expected), "missing {expected}");
    }
}

#[test]
fn wrapper_dispatch_admits_only_closed_platform_command_matrix() {
    assert!(
        dispatch_real_device_command(CliPlatform::Android, CliDeviceClass::AndroidPhysical)
            .expect("android physical")
            .platform_command
            .is_some_and(|command| command == "adb devices -l")
    );
    assert!(
        dispatch_real_device_command(CliPlatform::Ios, CliDeviceClass::IosSimulator)
            .expect("ios simulator")
            .platform_command
            .is_some_and(|command| command == "xcrun simctl list devices")
    );
    assert!(
        dispatch_real_device_command(CliPlatform::Ios, CliDeviceClass::IosPhysical)
            .expect("ios physical")
            .platform_command
            .is_some_and(|command| command == "xcrun devicectl list devices")
    );
    assert!(
        dispatch_real_device_command(CliPlatform::Browser, CliDeviceClass::DesktopBrowser)
            .expect("browser")
            .platform_command
            .is_none()
    );
}

#[test]
fn wrapper_dispatch_rejects_platform_device_class_mismatch() {
    assert!(
        dispatch_real_device_command(CliPlatform::Android, CliDeviceClass::IosSimulator).is_err()
    );
}

#[test]
fn kpi_real_device_wrapper_rejects_unbounded_command_execution() {
    for (
        platform,
        device_class,
        platform_arg,
        device_class_arg,
        expected_exit,
        expected_platform_command,
        expected_preflight,
    ) in [
        (
            CliPlatform::Android,
            CliDeviceClass::AndroidPhysical,
            "android",
            "android-physical",
            RealDeviceWrapperExit::PlatformCommandUnavailable,
            Some("adb devices -l"),
            REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE,
        ),
        (
            CliPlatform::Android,
            CliDeviceClass::AndroidEmulator,
            "android",
            "android-emulator",
            RealDeviceWrapperExit::PlatformCommandUnavailable,
            Some("adb devices -l"),
            REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE,
        ),
        (
            CliPlatform::Ios,
            CliDeviceClass::IosPhysical,
            "ios",
            "ios-physical",
            RealDeviceWrapperExit::PlatformCommandUnavailable,
            Some("xcrun devicectl list devices"),
            REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE,
        ),
        (
            CliPlatform::Ios,
            CliDeviceClass::IosSimulator,
            "ios",
            "ios-simulator",
            RealDeviceWrapperExit::PlatformCommandUnavailable,
            Some("xcrun simctl list devices"),
            REAL_DEVICE_PREFLIGHT_PLATFORM_COMMAND_UNAVAILABLE,
        ),
        (
            CliPlatform::Browser,
            CliDeviceClass::DesktopBrowser,
            "browser",
            "desktop-browser",
            RealDeviceWrapperExit::Success,
            None,
            REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
        ),
        (
            CliPlatform::Browser,
            CliDeviceClass::MobileBrowser,
            "browser",
            "mobile-browser",
            RealDeviceWrapperExit::Success,
            None,
            REAL_DEVICE_PREFLIGHT_WRAPPER_LOCAL_CAPABILITY,
        ),
    ] {
        let dispatch = dispatch_kpi_bounded_real_device_command(platform, device_class)
            .expect("canonical dispatch");
        let planned_exit = planned_exit_for_dispatch(&dispatch);
        assert_eq!(planned_exit, expected_exit);
        let record = build_kpi_bounded_real_device_record(
            dispatch,
            "reference-local".to_owned(),
            planned_exit,
        );
        assert_eq!(
            validated_exit_code(&record, planned_exit),
            Ok(expected_exit.code())
        );

        if expected_exit == RealDeviceWrapperExit::Success {
            let output = run_wrapper(platform_arg, device_class_arg, "reference-local");
            assert_eq!(output.status.code(), Some(expected_exit.code()));
            let json = stdout_json(&output);
            assert_eq!(
                json["preflight_outcome"],
                evidence::REAL_DEVICE_EXECUTION_WRAPPER_LOCAL_CAPABILITY
            );
            match expected_platform_command {
                Some(command) => assert_eq!(json["platform_command"], command),
                None => assert!(json["platform_command"].is_null()),
            }
            assert_json_non_claim_scope(&json);
            assert_eq!(json["redacted_device_identifier"], "redacted");
        }
        let _ = (platform_arg, device_class_arg, expected_preflight);
    }

    let browser = dispatch_kpi_bounded_real_device_command(
        CliPlatform::Browser,
        CliDeviceClass::DesktopBrowser,
    )
    .expect("browser dispatch");
    let browser_exit = planned_exit_for_dispatch(&browser);
    let unsafe_profile_record =
        build_kpi_bounded_real_device_record(browser, "serial=ABC123".to_owned(), browser_exit);
    assert_eq!(
        validated_exit_code(&unsafe_profile_record, browser_exit),
        Err(RealDeviceEvidenceValidationError::ProfileNotAdmitted)
    );

    let unsafe_profile_output = run_wrapper("browser", "desktop-browser", "serial=ABC123");
    assert_eq!(
        unsafe_profile_output.status.code(),
        Some(RealDeviceWrapperExit::EvidenceValidationFailure.code())
    );
    assert!(unsafe_profile_output.stdout.is_empty());
}

#[test]
fn wrapper_exit_mapping_separates_success_platform_unavailable_and_validation_failure() {
    assert_eq!(all_wrapper_exit_codes(), [0, 2, 3, 4, 5]);
    let _ = is_current_working_directory_distro_root();

    let browser =
        dispatch_real_device_command(CliPlatform::Browser, CliDeviceClass::DesktopBrowser)
            .expect("browser dispatch");
    let browser_exit = planned_exit_for_dispatch(&browser);
    assert_eq!(browser_exit, RealDeviceWrapperExit::Success);
    let browser_record =
        build_real_device_record(browser, "reference-local".to_owned(), browser_exit);
    assert_eq!(
        validated_exit_code(&browser_record, browser_exit),
        Ok(RealDeviceWrapperExit::Success.code())
    );

    let android =
        dispatch_real_device_command(CliPlatform::Android, CliDeviceClass::AndroidPhysical)
            .expect("android dispatch");
    let android_exit = planned_exit_for_dispatch(&android);
    assert_eq!(
        android_exit,
        RealDeviceWrapperExit::PlatformCommandUnavailable
    );
    let android_record =
        build_real_device_record(android, "reference-local".to_owned(), android_exit);
    assert_eq!(
        validated_exit_code(&android_record, android_exit),
        Ok(RealDeviceWrapperExit::PlatformCommandUnavailable.code())
    );

    let mut invalid_record = browser_record;
    invalid_record.redacted_device_identifier = None;
    assert_eq!(
        validated_exit_code(&invalid_record, browser_exit),
        Err(RealDeviceEvidenceValidationError::MissingPostExecutionField)
    );
    assert_eq!(RealDeviceWrapperExit::EvidenceValidationFailure.code(), 4);
    assert_eq!(
        distro_reason_for_exit(RealDeviceWrapperExit::EvidenceValidationFailure),
        DistroEvidenceReason::EvidenceFieldsIncomplete
    );
    assert_eq!(RealDeviceWrapperExit::ScopeMismatch.code(), 2);
    assert_eq!(RealDeviceWrapperExit::CommandScopeMismatch.code(), 5);
    assert_eq!(
        distro_reason_for_exit(RealDeviceWrapperExit::CommandScopeMismatch),
        DistroEvidenceReason::CommandScopeMismatch
    );
}

#[test]
fn wrapper_binary_exits_five_outside_distro_root() {
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-distro-real-device-tests"))
        .current_dir(std::env::temp_dir())
        .output()
        .expect("wrapper binary must execute");

    assert_eq!(output.status.code(), Some(5));
}

#[test]
fn wrapper_binary_reports_scope_mismatch_for_platform_device_mismatch() {
    let output = run_wrapper("android", "ios-simulator", "reference-local");

    assert_eq!(
        output.status.code(),
        Some(RealDeviceWrapperExit::ScopeMismatch.code())
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn wrapper_binary_reports_required_device_not_observed_for_empty_adb_output() {
    let sdk_root = fake_android_sdk_with_adb("#!/bin/sh\nprintf 'List of devices attached\\n'\n");
    let isolated_root = isolated_distro_root();
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-distro-real-device-tests"))
        .current_dir(&isolated_root)
        .env("ANDROID_HOME", sdk_root)
        .args([
            "android",
            "--profile",
            "reference-local",
            "--device-class",
            "android-physical",
        ])
        .output()
        .expect("wrapper binary must execute fake adb");
    let _ = fs::remove_dir_all(isolated_root);

    assert_eq!(
        output.status.code(),
        Some(RealDeviceWrapperExit::ScopeMismatch.code())
    );
    let json = stdout_json(&output);
    assert_eq!(
        json["exit_status"],
        RealDeviceWrapperExit::ScopeMismatch.code()
    );
    assert!(json["redacted_device_identifier"].is_null());
    assert_eq!(
        json["preflight_outcome"],
        evidence::REAL_DEVICE_EXECUTION_PLATFORM_COMMAND_EXECUTED
    );
}

#[test]
fn wrapper_binary_persists_the_same_validated_record_it_prints() {
    let sdk_root = fake_android_sdk_with_adb(
        "#!/bin/sh\nprintf 'List of devices attached\\nABC123 device product:pixel\\n'\n",
    );
    let isolated_root = isolated_distro_root();
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-distro-real-device-tests"))
        .current_dir(&isolated_root)
        .env("ANDROID_HOME", sdk_root)
        .args([
            "android",
            "--profile",
            "reference-local",
            "--device-class",
            "android-physical",
        ])
        .output()
        .expect("wrapper binary must execute fake adb");
    assert_eq!(output.status.code(), Some(0));
    let persisted_path =
        isolated_root.join("target/distro-evidence/real-device/android-physical.json");
    let stdout_record: Value = stdout_json(&output);
    let persisted_record: Value = serde_json::from_slice(
        &fs::read(&persisted_path).expect("persisted evidence must be readable"),
    )
    .expect("persisted evidence must be JSON");
    assert_eq!(persisted_record, stdout_record);
    assert!(
        fs::read_dir(persisted_path.parent().expect("evidence directory"))
            .expect("evidence directory must be readable")
            .all(|entry| !entry
                .expect("entry")
                .path()
                .to_string_lossy()
                .ends_with(".tmp"))
    );
    let _ = fs::remove_dir_all(isolated_root);
}

#[test]
fn wrapper_binary_fails_closed_when_evidence_directory_cannot_be_created() {
    let isolated_root = isolated_distro_root();
    fs::create_dir_all(isolated_root.join("target")).expect("target directory");
    fs::write(isolated_root.join("target/distro-evidence"), "blocked").expect("blocking file");
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-distro-real-device-tests"))
        .current_dir(&isolated_root)
        .args([
            "browser",
            "--profile",
            "reference-local",
            "--device-class",
            "desktop-browser",
        ])
        .output()
        .expect("wrapper binary must execute");
    assert_eq!(
        output.status.code(),
        Some(RealDeviceWrapperExit::EvidenceValidationFailure.code())
    );
    assert!(output.stdout.is_empty());
    let _ = fs::remove_dir_all(isolated_root);
}

#[test]
fn wrapper_binary_reports_platform_command_unavailable_for_missing_adb() {
    let empty_home = std::env::temp_dir().join(format!(
        "arcrtc-real-device-wrapper-empty-home-{}",
        std::process::id()
    ));
    let empty_path = empty_home.join("bin");
    fs::create_dir_all(&empty_path).expect("empty PATH dir must be created");
    let isolated_root = isolated_distro_root();

    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-distro-real-device-tests"))
        .current_dir(&isolated_root)
        .env_remove("ANDROID_HOME")
        .env_remove("ANDROID_SDK_ROOT")
        .env("HOME", &empty_home)
        .env("PATH", &empty_path)
        .args([
            "android",
            "--profile",
            "reference-local",
            "--device-class",
            "android-physical",
        ])
        .output()
        .expect("wrapper binary must execute without adb");
    let _ = fs::remove_dir_all(isolated_root);

    assert_eq!(
        output.status.code(),
        Some(RealDeviceWrapperExit::PlatformCommandUnavailable.code())
    );
    let json = stdout_json(&output);
    assert_eq!(
        json["exit_status"],
        RealDeviceWrapperExit::PlatformCommandUnavailable.code()
    );
    assert!(json["redacted_device_identifier"].is_null());
    assert_eq!(
        json["distro_reason"],
        serde_json::to_value(DistroEvidenceReason::RuntimeExecutorError)
            .expect("reason must serialize")
    );
}
