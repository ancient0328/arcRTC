use std::process::Command;

fn assert_case(args: &[&str], stdout: &str, stderr: &str, code: i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-entrypoint-endpoints"))
        .args(args)
        .output()
        .expect("endpoints entrypoint must execute");
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
fn endpoints_bin_startup_cases_match_fixed_surface() {
    assert_case(
        &["signaling-public"],
        "endpoint_class=signaling-public public_surface=true test_only=false private_control_or_admin=false",
        "",
        0,
    );
    assert_case(
        &["admin-private"],
        "endpoint_class=admin-private public_surface=false test_only=false private_control_or_admin=true",
        "",
        0,
    );
    assert_case(&["bogus"], "", "public_endpoint_not_allowed", 2);
}
