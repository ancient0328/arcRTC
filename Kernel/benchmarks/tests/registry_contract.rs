use std::collections::BTreeSet;

use arcrtc_benchmarks::{cases_for_lane, criterion_benchmark_cases, BenchmarkLane};

#[test]
fn executable_registry_is_unique_partitioned_and_runnable() {
    let all_cases = criterion_benchmark_cases();
    assert!(!all_cases.is_empty());

    let mut all_ids = BTreeSet::new();
    for case in &all_cases {
        assert!(all_ids.insert(case.scenario_id), "{}", case.scenario_id);
        assert!(!case.label.is_empty(), "{}", case.scenario_id);
        assert!(!case.workload_class.is_empty(), "{}", case.scenario_id);
        assert!(!case.source_class.is_empty(), "{}", case.scenario_id);
        std::hint::black_box((case.workload)());
    }

    let mut partitioned_ids = BTreeSet::new();
    for lane in BenchmarkLane::ALL {
        let cases = cases_for_lane(lane);
        assert!(!cases.is_empty(), "{}", lane.as_str());
        for case in cases {
            assert!(
                partitioned_ids.insert(case.scenario_id),
                "{}",
                case.scenario_id
            );
        }
    }

    assert_eq!(partitioned_ids, all_ids);
}
