use arcrtc_benchmarks::{
    benchmark_lane_window, cases_for_lane, BenchmarkLane, BenchmarkMeasurementWindow,
    BenchmarkWindowError,
};

const POLICY_SOURCE: &str = include_str!("../../tools/benchmark/benchmark-scenario-policy.toml");

#[test]
fn embedded_policy_and_registry_derive_every_lane_window() {
    let measurement_window = BenchmarkMeasurementWindow::parse(POLICY_SOURCE).unwrap();

    for lane in BenchmarkLane::ALL {
        let case_count = cases_for_lane(lane).len();
        let derived = measurement_window.allocate(lane, case_count).unwrap();
        let executable = benchmark_lane_window(lane).unwrap();

        assert_eq!(executable, derived);
        assert_eq!(executable.case_count(), case_count);
        assert_eq!(
            executable.warmup_per_case_seconds() * case_count as u64,
            measurement_window.warmup_seconds()
        );
        assert_eq!(
            executable.measurement_per_case_seconds() * case_count as u64,
            measurement_window.sample_seconds()
        );
        assert_eq!(
            executable.cooldown_seconds(),
            measurement_window.cooldown_seconds()
        );
    }
}

#[test]
fn malformed_or_unowned_policy_is_rejected_without_fallback() {
    assert_policy_error("", BenchmarkWindowError::MeasurementWindowSectionMissing);
    assert_policy_error(
        "[measurement_window]\nowner",
        BenchmarkWindowError::MalformedPolicyEntry,
    );
    assert_policy_error(
        "[measurement_window]\nowner = \"tools/benchmark\"\nwarmup_seconds = 1\nsample_seconds = 1\ncooldown_seconds = 1\n[measurement_window]\n",
        BenchmarkWindowError::MeasurementWindowSectionDuplicated,
    );
    assert_policy_error(
        "[measurement_window]\nwarmup_seconds = 1\nsample_seconds = 1\ncooldown_seconds = 1\n",
        BenchmarkWindowError::OwnerMissing,
    );
    assert_policy_error(
        "[measurement_window]\nowner = \"tools/benchmark\"\nowner = \"tools/benchmark\"\nwarmup_seconds = 1\nsample_seconds = 1\ncooldown_seconds = 1\n",
        BenchmarkWindowError::OwnerDuplicated,
    );
    assert_policy_error(
        &policy_source("\"another-owner\"", "1", "1", "1", ""),
        BenchmarkWindowError::OwnerMismatch,
    );
    assert_policy_error(
        "[measurement_window]\nowner = \"tools/benchmark\"\nsample_seconds = 1\ncooldown_seconds = 1\n",
        BenchmarkWindowError::WarmupMissing,
    );
    assert_policy_error(
        "[measurement_window]\nowner = \"tools/benchmark\"\nwarmup_seconds = 1\nwarmup_seconds = 1\nsample_seconds = 1\ncooldown_seconds = 1\n",
        BenchmarkWindowError::WarmupDuplicated,
    );
    assert_policy_error(
        &policy_source("\"tools/benchmark\"", "not-an-integer", "1", "1", ""),
        BenchmarkWindowError::WarmupInvalid,
    );
    assert_policy_error(
        &policy_source("\"tools/benchmark\"", "0", "1", "1", ""),
        BenchmarkWindowError::WarmupZero,
    );
    assert_policy_error(
        "[measurement_window]\nowner = \"tools/benchmark\"\nwarmup_seconds = 1\ncooldown_seconds = 1\n",
        BenchmarkWindowError::SampleMissing,
    );
    assert_policy_error(
        "[measurement_window]\nowner = \"tools/benchmark\"\nwarmup_seconds = 1\nsample_seconds = 1\nsample_seconds = 1\ncooldown_seconds = 1\n",
        BenchmarkWindowError::SampleDuplicated,
    );
    assert_policy_error(
        &policy_source("\"tools/benchmark\"", "1", "not-an-integer", "1", ""),
        BenchmarkWindowError::SampleInvalid,
    );
    assert_policy_error(
        &policy_source("\"tools/benchmark\"", "1", "0", "1", ""),
        BenchmarkWindowError::SampleZero,
    );
    assert_policy_error(
        "[measurement_window]\nowner = \"tools/benchmark\"\nwarmup_seconds = 1\nsample_seconds = 1\n",
        BenchmarkWindowError::CooldownMissing,
    );
    assert_policy_error(
        "[measurement_window]\nowner = \"tools/benchmark\"\nwarmup_seconds = 1\nsample_seconds = 1\ncooldown_seconds = 1\ncooldown_seconds = 1\n",
        BenchmarkWindowError::CooldownDuplicated,
    );
    assert_policy_error(
        &policy_source("\"tools/benchmark\"", "1", "1", "not-an-integer", ""),
        BenchmarkWindowError::CooldownInvalid,
    );
    assert_policy_error(
        &policy_source("\"tools/benchmark\"", "1", "1", "0", ""),
        BenchmarkWindowError::CooldownZero,
    );
    assert_policy_error(
        &policy_source("\"tools/benchmark\"", "1", "1", "1", "unexpected = 1"),
        BenchmarkWindowError::UnexpectedMeasurementWindowField,
    );
}

#[test]
fn empty_or_non_divisible_lane_allocation_is_rejected() {
    let lane = BenchmarkLane::Load;
    let case_count = cases_for_lane(lane).len() + 1;
    let measurement_window = BenchmarkMeasurementWindow::parse(&policy_source(
        "\"tools/benchmark\"",
        &(case_count as u64 + 1).to_string(),
        &(case_count as u64 * 2).to_string(),
        "1",
        "",
    ))
    .unwrap();

    assert_eq!(
        measurement_window.allocate(lane, 0),
        Err(BenchmarkWindowError::EmptyLane(lane))
    );
    assert_eq!(
        measurement_window.allocate(lane, case_count),
        Err(BenchmarkWindowError::WarmupNotDivisible {
            total_seconds: case_count as u64 + 1,
            case_count,
        })
    );

    let sample_non_divisible = BenchmarkMeasurementWindow::parse(&policy_source(
        "\"tools/benchmark\"",
        &(case_count as u64 * 2).to_string(),
        &(case_count as u64 + 1).to_string(),
        "1",
        "",
    ))
    .unwrap();
    assert_eq!(
        sample_non_divisible.allocate(lane, case_count),
        Err(BenchmarkWindowError::SampleNotDivisible {
            total_seconds: case_count as u64 + 1,
            case_count,
        })
    );
}

fn assert_policy_error(source: &str, expected: BenchmarkWindowError) {
    assert_eq!(BenchmarkMeasurementWindow::parse(source), Err(expected));
}

fn policy_source(owner: &str, warmup: &str, sample: &str, cooldown: &str, extra: &str) -> String {
    format!(
        "[measurement_window]\nowner = {owner}\nwarmup_seconds = {warmup}\nsample_seconds = {sample}\ncooldown_seconds = {cooldown}\n{extra}\n"
    )
}
