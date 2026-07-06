use std::process::Command;

fn assert_case(args: &[&str], stdout: &str, stderr: &str, code: i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-entrypoint-configuration"))
        .args(args)
        .output()
        .expect("configuration entrypoint must execute");
    let actual_stdout = String::from_utf8(output.stdout).expect("stdout must be UTF-8");
    let actual_stderr = String::from_utf8(output.stderr).expect("stderr must be UTF-8");
    println!(
        "args={args:?} stdout={actual_stdout:?} stderr={actual_stderr:?} code={:?}",
        output.status.code()
    );
    assert_eq!(actual_stdout.trim(), stdout);
    assert_eq!(actual_stderr.trim(), stderr);
    assert_eq!(output.status.code(), Some(code));
}

#[test]
fn configuration_bin_startup_cases_match_fixed_surface() {
    assert_case(
        &["test-deterministic"],
        "profile_class=test-deterministic adoption_rule=TestEvidenceOnly",
        "",
        0,
    );
    assert_case(&[], "", "runtime_config_missing", 2);
    assert_case(&["bogus"], "", "runtime_config_invalid", 2);
}
