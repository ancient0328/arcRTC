use std::process::Command;

fn assert_case(args: &[&str], stdout: &str, stderr: &str, code: i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc"))
        .args(args)
        .output()
        .expect("cli must execute");
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
fn cli_startup_cases_match_fixed_surface() {
    assert_case(&["signaling"], "command_class=signaling", "", 0);
    assert_case(&["admin-maintenance"], "", "authorization_policy_denied", 3);
    assert_case(&["bogus"], "", "external_decode_failed", 2);
    assert_case(&[], "", "runtime_config_missing", 2);
}
