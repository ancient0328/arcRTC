use std::collections::BTreeSet;

const WORKFLOW: &str = include_str!("../../../.github/workflows/kernel-ci.yml");
const MATRIX: &str = include_str!("../../tools/ci/kernel-gate-matrix.toml");

#[derive(Debug)]
struct MatrixCommand<'a> {
    job_id: &'a str,
    command_family: &'a str,
    command_id: &'a str,
    gate_class: &'a str,
    working_directory: &'a str,
    command_line: &'a str,
}

fn quoted_value<'a>(source: &'a str, key: &str) -> &'a str {
    let prefix = format!("{key} = \"");
    let value = source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("missing matrix field: {key}"));
    value
        .strip_suffix('"')
        .unwrap_or_else(|| panic!("invalid matrix field: {key}"))
}

fn quoted_array(source: &str, key: &str) -> BTreeSet<String> {
    let prefix = format!("{key} = [");
    let start = source
        .find(&prefix)
        .unwrap_or_else(|| panic!("missing matrix closed set: {key}"));
    let remainder = &source[start + prefix.len()..];
    let end = remainder
        .find(']')
        .unwrap_or_else(|| panic!("unterminated matrix closed set: {key}"));

    remainder[..end]
        .split(',')
        .filter_map(|value| {
            let value = value.trim();
            (!value.is_empty()).then(|| {
                value
                    .strip_prefix('"')
                    .and_then(|value| value.strip_suffix('"'))
                    .unwrap_or_else(|| panic!("invalid member in matrix closed set: {key}"))
                    .to_owned()
            })
        })
        .collect()
}

fn matrix_commands() -> Vec<MatrixCommand<'static>> {
    MATRIX
        .split("[[commands]]")
        .skip(1)
        .map(|block| MatrixCommand {
            job_id: quoted_value(block, "job_id"),
            command_family: quoted_value(block, "command_family"),
            command_id: quoted_value(block, "command_id"),
            gate_class: quoted_value(block, "gate_class"),
            working_directory: quoted_value(block, "working_directory"),
            command_line: quoted_value(block, "command_line"),
        })
        .collect()
}

#[test]
fn matrix_commands_use_declared_closed_sets_and_unique_ids() {
    let declared_job_ids = quoted_array(MATRIX, "job_ids");
    let declared_command_families = quoted_array(MATRIX, "command_families");
    let declared_gate_classes = quoted_array(MATRIX, "gate_classes");
    let commands = matrix_commands();
    let mut command_ids = BTreeSet::new();
    let mut used_job_ids = BTreeSet::new();

    assert!(!commands.is_empty(), "the CI matrix must contain commands");
    for command in commands {
        assert!(declared_job_ids.contains(command.job_id), "{command:?}");
        assert!(
            declared_command_families.contains(command.command_family),
            "{command:?}"
        );
        assert!(
            declared_gate_classes.contains(command.gate_class),
            "{command:?}"
        );
        assert!(!command.working_directory.is_empty(), "{command:?}");
        assert!(!command.command_line.is_empty(), "{command:?}");
        assert!(command_ids.insert(command.command_id), "{command:?}");
        used_job_ids.insert(command.job_id.to_owned());
    }

    assert_eq!(used_job_ids, declared_job_ids);
}

#[test]
fn workflow_resolves_every_matrix_command() {
    assert!(WORKFLOW.contains("KERNEL_GATE_MATRIX_SOURCE: Kernel/tools/ci/kernel-gate-matrix.toml"));

    let commands = matrix_commands();
    let job_ids: BTreeSet<_> = commands.iter().map(|command| command.job_id).collect();
    assert_eq!(
        WORKFLOW.matches("Resolve matrix source reference").count(),
        job_ids.len()
    );

    for job_id in job_ids {
        assert!(WORKFLOW.contains(&format!("\n  {job_id}:")), "{job_id}");
        assert!(
            WORKFLOW.contains(&format!("MATRIX_JOB_ID: {job_id}")),
            "{job_id}"
        );
    }

    for command in commands {
        assert!(WORKFLOW.contains(command.command_id), "{command:?}");
        assert!(WORKFLOW.contains(command.command_family), "{command:?}");
    }
}
