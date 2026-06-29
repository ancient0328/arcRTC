//! Test Roadmap / CI matrix / benchmark / CE evidence の閉集合です。
//!
//! 正典・Roadmap 由来の義務を test-side manifest として固定し、
//! テストが単なるファイル名や文字列列挙に退化しないようにします。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequiredItem {
    pub id: &'static str,
    pub label: &'static str,
}

pub const T_SERIES_TASKS: &[RequiredItem] = &[
    RequiredItem {
        id: "T0.1",
        label: "evidence intake shape",
    },
    RequiredItem {
        id: "T0.2",
        label: "fake/test-double register",
    },
    RequiredItem {
        id: "T0.3",
        label: "fixture/scenario register",
    },
    RequiredItem {
        id: "T1.1",
        label: "architecture dependency static assets",
    },
    RequiredItem {
        id: "T1.2",
        label: "owner and target-surface static assets",
    },
    RequiredItem {
        id: "T1.3",
        label: "command executability matrix",
    },
    RequiredItem {
        id: "T1.4",
        label: "CI quality gate metadata",
    },
    RequiredItem {
        id: "T2.1",
        label: "core vocabulary and protocol unit assets",
    },
    RequiredItem {
        id: "T2.2",
        label: "signaling unit assets",
    },
    RequiredItem {
        id: "T2.3",
        label: "SFU unit assets",
    },
    RequiredItem {
        id: "T2.4",
        label: "TURN unit assets",
    },
    RequiredItem {
        id: "T2.5",
        label: "core transport and secure media unit assets",
    },
    RequiredItem {
        id: "T2.6",
        label: "cross-plane runtime state recovery quality unit assets",
    },
    RequiredItem {
        id: "T3.1",
        label: "network conversion test assets",
    },
    RequiredItem {
        id: "T3.2",
        label: "TURN wire driver test assets",
    },
    RequiredItem {
        id: "T3.3",
        label: "WebRTC str0m driver test assets",
    },
    RequiredItem {
        id: "T3.4",
        label: "platform driver test assets",
    },
    RequiredItem {
        id: "T4.1",
        label: "server composition test assets",
    },
    RequiredItem {
        id: "T4.2",
        label: "operational entrypoint surface test assets",
    },
    RequiredItem {
        id: "T5.1",
        label: "TypeScript SDK contract test assets",
    },
    RequiredItem {
        id: "T5.2",
        label: "Android SDK contract test assets",
    },
    RequiredItem {
        id: "T5.3",
        label: "iOS SDK contract test assets",
    },
    RequiredItem {
        id: "T6.1",
        label: "regulated optional boundary test assets",
    },
    RequiredItem {
        id: "T7.1",
        label: "signaling controlled integration assets",
    },
    RequiredItem {
        id: "T7.2",
        label: "TURN relay-gate integration assets",
    },
    RequiredItem {
        id: "T7.3",
        label: "TURN blocked-network observation assets",
    },
    RequiredItem {
        id: "T7.4",
        label: "SFU three-party routing integration assets",
    },
    RequiredItem {
        id: "T7.5",
        label: "cross-plane binding integration assets",
    },
    RequiredItem {
        id: "T8.1",
        label: "benchmark scope and evidence shape",
    },
    RequiredItem {
        id: "T8.2",
        label: "benchmark scenario matrix",
    },
    RequiredItem {
        id: "T8.3",
        label: "benchmark command and report classification",
    },
    RequiredItem {
        id: "T9.1",
        label: "evidence adoption check assets",
    },
    RequiredItem {
        id: "T9.2",
        label: "test-result Closed Gate assets",
    },
    RequiredItem {
        id: "T9.3",
        label: "benchmark report non-correctness assets",
    },
    RequiredItem {
        id: "T9.4",
        label: "v0.2 completion evidence closure assets",
    },
];

pub const CE_REPORTS: &[RequiredItem] = &[
    RequiredItem {
        id: "CE0",
        label: "evidence fake fixture integrity",
    },
    RequiredItem {
        id: "CE1",
        label: "architecture source-shape closure",
    },
    RequiredItem {
        id: "CE2",
        label: "core semantic closure",
    },
    RequiredItem {
        id: "CE3",
        label: "driver contract closure",
    },
    RequiredItem {
        id: "CE4",
        label: "entrypoint composition closure",
    },
    RequiredItem {
        id: "CE5",
        label: "SDK contract closure",
    },
    RequiredItem {
        id: "CE6",
        label: "regulated optional boundary closure",
    },
    RequiredItem {
        id: "CE7",
        label: "controlled integration closure",
    },
    RequiredItem {
        id: "CE8",
        label: "benchmark measurement closure",
    },
    RequiredItem {
        id: "CE9",
        label: "evidence adoption closure",
    },
    RequiredItem {
        id: "CE10",
        label: "v0.2 completion Closed Gate",
    },
];

pub const CI_GATES: &[RequiredItem] = &[
    RequiredItem {
        id: "CI-001",
        label: "docs structure",
    },
    RequiredItem {
        id: "CI-002",
        label: "architecture dependency",
    },
    RequiredItem {
        id: "CI-003",
        label: "Rust format/lint",
    },
    RequiredItem {
        id: "CI-004",
        label: "Rust unit tests",
    },
    RequiredItem {
        id: "CI-005",
        label: "SDK TypeScript tests",
    },
    RequiredItem {
        id: "CI-006",
        label: "SDK Android unit/build/lint command",
    },
    RequiredItem {
        id: "CI-007",
        label: "SDK iOS tests",
    },
    RequiredItem {
        id: "CI-008",
        label: "SDK public API projection tests",
    },
    RequiredItem {
        id: "CI-009",
        label: "internal control-plane contract tests",
    },
    RequiredItem {
        id: "CI-010",
        label: "ICE candidate/connectivity contract tests",
    },
    RequiredItem {
        id: "CI-011",
        label: "secure media session contract tests",
    },
    RequiredItem {
        id: "CI-012",
        label: "operator/admin authorization tests",
    },
    RequiredItem {
        id: "CI-013",
        label: "out-of-scope feature rejection tests",
    },
    RequiredItem {
        id: "CI-014",
        label: "public endpoint/connection lifecycle tests",
    },
    RequiredItem {
        id: "CI-015",
        label: "export/backup artifact tests",
    },
    RequiredItem {
        id: "CI-016",
        label: "release artifact/provenance tests",
    },
    RequiredItem {
        id: "CI-017",
        label: "time synchronization/clock skew tests",
    },
    RequiredItem {
        id: "CI-018",
        label: "edge/proxy trust tests",
    },
    RequiredItem {
        id: "CI-019",
        label: "runtime reconfiguration tests",
    },
    RequiredItem {
        id: "CI-020",
        label: "packet rewrite/media transform tests",
    },
    RequiredItem {
        id: "CI-021",
        label: "service discovery/endpoint resolution tests",
    },
    RequiredItem {
        id: "CI-022",
        label: "distributed state/failover tests",
    },
    RequiredItem {
        id: "CI-023",
        label: "runtime task/worker lifecycle tests",
    },
    RequiredItem {
        id: "CI-024",
        label: "internal service identity/trust tests",
    },
    RequiredItem {
        id: "CI-025",
        label: "cross-plane identity/session binding tests",
    },
    RequiredItem {
        id: "CI-026",
        label: "fake driver contract tests",
    },
    RequiredItem {
        id: "CI-027",
        label: "supply-chain gate",
    },
    RequiredItem {
        id: "CI-028",
        label: "integration tests",
    },
    RequiredItem {
        id: "CI-029",
        label: "benchmark smoke",
    },
    RequiredItem {
        id: "CI-030",
        label: "v0.2 completion evidence check",
    },
    RequiredItem {
        id: "CI-031",
        label: "Closed Gate report check",
    },
];

pub const BENCHMARK_SCENARIOS: &[RequiredItem] = &[
    RequiredItem {
        id: "BENCH-001",
        label: "core signaling decision latency",
    },
    RequiredItem {
        id: "BENCH-002",
        label: "core SFU routing decision latency",
    },
    RequiredItem {
        id: "BENCH-003",
        label: "packet semantic view cost",
    },
    RequiredItem {
        id: "BENCH-004",
        label: "packet rewrite / transform cost",
    },
    RequiredItem {
        id: "BENCH-005",
        label: "driver conversion cost",
    },
    RequiredItem {
        id: "BENCH-006",
        label: "service discovery resolution cost",
    },
    RequiredItem {
        id: "BENCH-007",
        label: "distributed owner/failover diagnostic cost",
    },
    RequiredItem {
        id: "BENCH-008",
        label: "runtime task lifecycle cost",
    },
    RequiredItem {
        id: "BENCH-009",
        label: "internal service trust cost",
    },
    RequiredItem {
        id: "BENCH-010",
        label: "cross-plane binding cost",
    },
    RequiredItem {
        id: "BENCH-011",
        label: "TURN wire decode/encode cost",
    },
    RequiredItem {
        id: "BENCH-012",
        label: "resource bound saturation",
    },
    RequiredItem {
        id: "BENCH-013",
        label: "audit hash-chain append/verify cost",
    },
    RequiredItem {
        id: "BENCH-014",
        label: "canonical serialization cost",
    },
    RequiredItem {
        id: "BENCH-015",
        label: "persistence/export cost",
    },
    RequiredItem {
        id: "BENCH-016",
        label: "SDK signaling roundtrip",
    },
];

pub fn ids_are_unique(items: &[RequiredItem]) -> bool {
    for (index, item) in items.iter().enumerate() {
        if items[index + 1..]
            .iter()
            .any(|candidate| candidate.id == item.id)
        {
            return false;
        }
    }
    true
}

pub fn contains_id(items: &[RequiredItem], id: &str) -> bool {
    items.iter().any(|item| item.id == id)
}
