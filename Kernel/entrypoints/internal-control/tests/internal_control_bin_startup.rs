use std::process::Command;

fn assert_case(args: &[&str], stdout: &str, stderr: &str, code: i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-entrypoint-internal-control"))
        .args(args)
        .output()
        .expect("internal-control entrypoint must execute");
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
fn internal_control_bin_startup_cases_match_fixed_surface() {
    assert_case(
        &["mtls-peer-identity"],
        "trust_class=mtls-peer-identity credential_bearing=true test_only=false rejected_request_class=false",
        "",
        0,
    );
    assert_case(
        &["unauthenticated-internal-service-requested"],
        "trust_class=unauthenticated-internal-service-requested credential_bearing=false test_only=false rejected_request_class=true",
        "",
        0,
    );
    assert_case(&["bogus"], "", "internal_control_message_invalid", 2);
}
