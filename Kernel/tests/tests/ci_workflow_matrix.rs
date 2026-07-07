const WORKFLOW: &str = include_str!("../../../.github/workflows/kernel-ci.yml");
const MATRIX: &str = include_str!("../../tools/ci/kernel-gate-matrix.toml");

#[derive(Debug, Clone, Copy)]
struct ExpectedCommand {
    job_id: &'static str,
    command_family: &'static str,
    command_id: &'static str,
    gate_class: &'static str,
    working_directory: &'static str,
    command_line: &'static str,
}

const EXPECTED_COMMANDS: &[ExpectedCommand] = &[
    ExpectedCommand {
        job_id: "kernel-rust",
        command_family: "cargo-fmt-check",
        command_id: "cargo-fmt-check",
        gate_class: "build",
        working_directory: ".",
        command_line: "cargo fmt --all -- --check",
    },
    ExpectedCommand {
        job_id: "kernel-rust",
        command_family: "cargo-clippy-workspace",
        command_id: "cargo-clippy-workspace",
        gate_class: "build",
        working_directory: ".",
        command_line: "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    },
    ExpectedCommand {
        job_id: "kernel-rust",
        command_family: "cargo-test-workspace",
        command_id: "cargo-test-workspace",
        gate_class: "test",
        working_directory: ".",
        command_line: "cargo test --workspace",
    },
    ExpectedCommand {
        job_id: "sdk-typescript",
        command_family: "typescript-sdk-test",
        command_id: "typescript-sdk-test",
        gate_class: "test",
        working_directory: "sdk/typescript",
        command_line: "pnpm test",
    },
    ExpectedCommand {
        job_id: "sdk-android",
        command_family: "android-sdk-test",
        command_id: "android-sdk-test",
        gate_class: "test",
        working_directory: "sdk/android",
        command_line: "./gradlew :sdk:testDebugUnitTest",
    },
    ExpectedCommand {
        job_id: "sdk-ios",
        command_family: "ios-sdk-test",
        command_id: "ios-sdk-test",
        gate_class: "test",
        working_directory: "sdk/ios",
        command_line: "swift test",
    },
    ExpectedCommand {
        job_id: "supply-chain",
        command_family: "supply-chain-policy-check",
        command_id: "supply-chain-policy-check",
        gate_class: "source-shape",
        working_directory: ".",
        command_line: "cargo test -p arcrtc-roadmap-tests --test release_supply_chain_policy",
    },
    ExpectedCommand {
        job_id: "coverage",
        command_family: "coverage-denominator-check",
        command_id: "coverage-denominator-check",
        gate_class: "test",
        working_directory: ".",
        command_line: "cargo test -p arcrtc-roadmap-tests --test coverage_denominator_policy",
    },
    ExpectedCommand {
        job_id: "benchmark",
        command_family: "benchmark-scenario-check",
        command_id: "benchmark-load-check",
        gate_class: "benchmark",
        working_directory: ".",
        command_line: "cargo bench -p arcrtc-benchmarks --bench kernel_load",
    },
    ExpectedCommand {
        job_id: "benchmark",
        command_family: "benchmark-scenario-check",
        command_id: "benchmark-soak-check",
        gate_class: "benchmark",
        working_directory: ".",
        command_line: "cargo bench -p arcrtc-benchmarks --bench kernel_soak",
    },
    ExpectedCommand {
        job_id: "benchmark",
        command_family: "benchmark-scenario-check",
        command_id: "benchmark-concurrency-check",
        gate_class: "benchmark",
        working_directory: ".",
        command_line: "cargo bench -p arcrtc-benchmarks --bench kernel_concurrency",
    },
    ExpectedCommand {
        job_id: "security",
        command_family: "security-adversarial-check",
        command_id: "security-adversarial-check",
        gate_class: "runtime-in-test",
        working_directory: ".",
        command_line:
            "cargo test --manifest-path tests/Cargo.toml --test security_adversarial_scan",
    },
];

fn matrix_command_blocks() -> Vec<&'static str> {
    MATRIX.split("[[commands]]").skip(1).collect()
}

fn matrix_has_command(expected: ExpectedCommand) -> bool {
    matrix_command_blocks().iter().any(|block| {
        block.contains(&format!("job_id = \"{}\"", expected.job_id))
            && block.contains(&format!("command_family = \"{}\"", expected.command_family))
            && block.contains(&format!("command_id = \"{}\"", expected.command_id))
            && block.contains(&format!("gate_class = \"{}\"", expected.gate_class))
            && block.contains(&format!(
                "working_directory = \"{}\"",
                expected.working_directory
            ))
            && block.contains(&format!("command_line = \"{}\"", expected.command_line))
    })
}

#[test]
fn ci_workflow_declares_expected_job_ids() {
    for job_id in [
        "kernel-rust",
        "sdk-typescript",
        "sdk-android",
        "sdk-ios",
        "supply-chain",
        "coverage",
        "benchmark",
        "security",
    ] {
        assert!(WORKFLOW.contains(&format!("\n  {job_id}:")), "{job_id}");
        assert!(MATRIX.contains(&format!("\"{job_id}\"")), "{job_id}");
    }
}

#[test]
fn kernel_gate_matrix_contains_every_ci_command_line_assertion() {
    assert_eq!(matrix_command_blocks().len(), EXPECTED_COMMANDS.len());

    for expected in EXPECTED_COMMANDS {
        assert!(matrix_has_command(*expected), "{expected:?}");
    }
}

#[test]
fn workflow_resolves_every_matrix_command_id() {
    assert!(WORKFLOW.contains("KERNEL_GATE_MATRIX_SOURCE: Kernel/tools/ci/kernel-gate-matrix.toml"));
    assert_eq!(
        WORKFLOW.matches("Resolve matrix source reference").count(),
        8
    );

    for expected in EXPECTED_COMMANDS {
        let command_id_env = format!("MATRIX_COMMAND_ID: {}", expected.command_id);
        let command_id_matrix = format!("command_id: {}", expected.command_id);
        assert!(
            WORKFLOW.contains(&command_id_env) || WORKFLOW.contains(&command_id_matrix),
            "{}",
            expected.command_id
        );
    }
}
