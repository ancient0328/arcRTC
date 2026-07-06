//! benchmark scenario の閉集合です。

use arcrtc_distro_evidence::{DistroLayer, DistroPlane};

/// benchmark scenario 定義です。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BenchmarkScenario {
    /// scenario idです。
    pub id: &'static str,
    /// scenario nameです。
    pub name: &'static str,
    /// distro layerです。
    pub layer: DistroLayer,
    /// target planeです。
    pub plane: DistroPlane,
    /// workload summaryです。
    pub workload_summary: &'static str,
}

/// Canonical に固定された benchmark scenario set です。
pub const BENCHMARK_SCENARIOS: &[BenchmarkScenario] = &[
    scenario(
        "BENCH-001",
        "signaling_join_room_single",
        DistroLayer::Reference,
        DistroPlane::Signaling,
        "one room, one participant, one `JoinRoom`",
    ),
    scenario(
        "BENCH-002",
        "signaling_join_room_batch",
        DistroLayer::Reference,
        DistroPlane::Signaling,
        "8 rooms, 6 participants per room, `JoinRoom` for all participants",
    ),
    scenario(
        "BENCH-003",
        "signaling_offer_answer_candidate",
        DistroLayer::Reference,
        DistroPlane::Signaling,
        "joined pair, offer, answer, 4 candidates",
    ),
    scenario(
        "BENCH-004",
        "signaling_turn_credential_request",
        DistroLayer::Reference,
        DistroPlane::Signaling,
        "joined participant, one `RequestTurnCredential`",
    ),
    scenario(
        "BENCH-005",
        "turn_allocate_single",
        DistroLayer::Reference,
        DistroPlane::Turn,
        "one fixture credential, one `Allocate`",
    ),
    scenario(
        "BENCH-006",
        "turn_permission_batch",
        DistroLayer::Reference,
        DistroPlane::Turn,
        "8 allocations, 16 permissions",
    ),
    scenario(
        "BENCH-007",
        "turn_channel_bind_batch",
        DistroLayer::Reference,
        DistroPlane::Turn,
        "8 allocations, 16 permissions, 16 channel binds",
    ),
    scenario(
        "BENCH-008",
        "turn_relay_data_path",
        DistroLayer::Reference,
        DistroPlane::Turn,
        "active allocation / permission, 1200 byte packet id fixture",
    ),
    scenario(
        "BENCH-009",
        "sfu_contract_item_create",
        DistroLayer::Reference,
        DistroPlane::Sfu,
        "one `SfuContractItem` per route",
    ),
    scenario(
        "BENCH-010",
        "sfu_borrowed_packet_view",
        DistroLayer::Reference,
        DistroPlane::Sfu,
        "borrowed 1200 byte packet / payload slices",
    ),
    scenario(
        "BENCH-011",
        "sfu_route_select_batch",
        DistroLayer::Reference,
        DistroPlane::Sfu,
        "24 routes selected across 8 sessions",
    ),
    scenario(
        "BENCH-012",
        "sfu_packet_fanout_intent",
        DistroLayer::Reference,
        DistroPlane::Sfu,
        "one source packet, 6 target endpoint references",
    ),
    scenario(
        "BENCH-013",
        "composition_signaling_turn_binding",
        DistroLayer::Reference,
        DistroPlane::Composition,
        "8 joined rooms and 8 active allocations",
    ),
    scenario(
        "BENCH-014",
        "composition_signaling_sfu_binding",
        DistroLayer::Reference,
        DistroPlane::Composition,
        "8 joined rooms and 24 SFU routes",
    ),
    scenario(
        "BENCH-015",
        "runtime_start_shutdown",
        DistroLayer::Reference,
        DistroPlane::Ops,
        "start reference runtime then graceful shutdown",
    ),
    scenario(
        "BENCH-016",
        "evidence_json_record_write",
        DistroLayer::Reference,
        DistroPlane::Ops,
        "build one `DistroEvidenceRecord` and serialize JSON",
    ),
    scenario(
        "BENCH-017",
        "product_policy_evaluate",
        DistroLayer::Product,
        DistroPlane::Policy,
        "one product policy input per plane",
    ),
    scenario(
        "BENCH-018",
        "product_projection_mapping",
        DistroLayer::Product,
        DistroPlane::Persistence,
        "one projection mapping per record class",
    ),
    scenario(
        "BENCH-019",
        "product_runtime_select",
        DistroLayer::Product,
        DistroPlane::Deployment,
        "select local product runtime profile",
    ),
    scenario(
        "BENCH-020",
        "product_drain_plan_create",
        DistroLayer::Product,
        DistroPlane::Rollback,
        "create drain plan for signaling / turn / sfu",
    ),
];

const fn scenario(
    id: &'static str,
    name: &'static str,
    layer: DistroLayer,
    plane: DistroPlane,
    workload_summary: &'static str,
) -> BenchmarkScenario {
    BenchmarkScenario {
        id,
        name,
        layer,
        plane,
        workload_summary,
    }
}

/// scenario id から定義を返します。
pub fn scenario_by_id(id: &str) -> Option<&'static BenchmarkScenario> {
    BENCHMARK_SCENARIOS
        .iter()
        .find(|scenario| scenario.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_unit_covers_runtime_constructor_path() {
        let constructed = scenario(
            "BENCH-UNIT",
            "unit_runtime_constructor",
            DistroLayer::Product,
            DistroPlane::Policy,
            "unit workload summary",
        );

        assert_eq!(constructed.id, "BENCH-UNIT");
        assert_eq!(constructed.name, "unit_runtime_constructor");
        assert_eq!(constructed.layer, DistroLayer::Product);
        assert_eq!(constructed.plane, DistroPlane::Policy);
        assert_eq!(constructed.workload_summary, "unit workload summary");
    }
}
