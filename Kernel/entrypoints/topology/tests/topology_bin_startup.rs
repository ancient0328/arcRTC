use std::process::Command;

fn assert_case(args: &[&str], stdout: &str, stderr: &str, code: i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_arcrtc-entrypoint-topology"))
        .args(args)
        .output()
        .expect("topology entrypoint must execute");
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
fn topology_bin_startup_cases_match_fixed_surface() {
    assert_case(
        &["split-plane-networked"],
        "topology_class=split-plane-networked requires_explicit_service_endpoint_wiring=true requires_networked_internal_service_relation=true requires_explicit_experimental_enablement=false requires_external_dependency_contract=false",
        "",
        0,
    );
    assert_case(
        &["single-process-local"],
        "topology_class=single-process-local requires_explicit_service_endpoint_wiring=false requires_networked_internal_service_relation=false requires_explicit_experimental_enablement=false requires_external_dependency_contract=false",
        "",
        0,
    );
    assert_case(&["bogus"], "", "deployment_topology_unsupported", 2);
}
