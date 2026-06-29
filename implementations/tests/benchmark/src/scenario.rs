//! benchmark scenario の閉集合です。

use arcrtc_implementation_evidence::{ImplementationLayer, ImplementationPlane};

/// benchmark scenario 定義です。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BenchmarkScenario {
    /// scenario idです。
    pub id: &'static str,
    /// scenario nameです。
    pub name: &'static str,
    /// implementation layerです。
    pub layer: ImplementationLayer,
    /// target planeです。
    pub plane: ImplementationPlane,
    /// workload summaryです。
    pub workload_summary: &'static str,
}

/// Canonical に固定された benchmark scenario set です。
pub const BENCHMARK_SCENARIOS: &[BenchmarkScenario] = &[
    scenario(
        "BENCH-001",
        "signaling_join_room_single",
        ImplementationLayer::Reference,
        ImplementationPlane::Signaling,
        "one room, one participant, one `JoinRoom`",
    ),
    scenario(
        "BENCH-002",
        "signaling_join_room_batch",
        ImplementationLayer::Reference,
        ImplementationPlane::Signaling,
        "8 rooms, 6 participants per room, `JoinRoom` for all participants",
    ),
    scenario(
        "BENCH-003",
        "signaling_offer_answer_candidate",
        ImplementationLayer::Reference,
        ImplementationPlane::Signaling,
        "joined pair, offer, answer, 4 candidates",
    ),
    scenario(
        "BENCH-004",
        "signaling_turn_credential_request",
        ImplementationLayer::Reference,
        ImplementationPlane::Signaling,
        "joined participant, one `RequestTurnCredential`",
    ),
    scenario(
        "BENCH-005",
        "turn_allocate_single",
        ImplementationLayer::Reference,
        ImplementationPlane::Turn,
        "one fixture credential, one `Allocate`",
    ),
    scenario(
        "BENCH-006",
        "turn_permission_batch",
        ImplementationLayer::Reference,
        ImplementationPlane::Turn,
        "8 allocations, 16 permissions",
    ),
    scenario(
        "BENCH-007",
        "turn_channel_bind_batch",
        ImplementationLayer::Reference,
        ImplementationPlane::Turn,
        "8 allocations, 16 permissions, 16 channel binds",
    ),
    scenario(
        "BENCH-008",
        "turn_relay_data_path",
        ImplementationLayer::Reference,
        ImplementationPlane::Turn,
        "active allocation / permission, 1200 byte packet id fixture",
    ),
    scenario(
        "BENCH-009",
        "sfu_contract_item_create",
        ImplementationLayer::Reference,
        ImplementationPlane::Sfu,
        "one `SfuContractItem` per route",
    ),
    scenario(
        "BENCH-010",
        "sfu_borrowed_packet_view",
        ImplementationLayer::Reference,
        ImplementationPlane::Sfu,
        "borrowed 1200 byte packet / payload slices",
    ),
    scenario(
        "BENCH-011",
        "sfu_route_select_batch",
        ImplementationLayer::Reference,
        ImplementationPlane::Sfu,
        "24 routes selected across 8 sessions",
    ),
    scenario(
        "BENCH-012",
        "sfu_packet_fanout_intent",
        ImplementationLayer::Reference,
        ImplementationPlane::Sfu,
        "one source packet, 6 target endpoint references",
    ),
    scenario(
        "BENCH-013",
        "composition_signaling_turn_binding",
        ImplementationLayer::Reference,
        ImplementationPlane::Composition,
        "8 joined rooms and 8 active allocations",
    ),
    scenario(
        "BENCH-014",
        "composition_signaling_sfu_binding",
        ImplementationLayer::Reference,
        ImplementationPlane::Composition,
        "8 joined rooms and 24 SFU routes",
    ),
    scenario(
        "BENCH-015",
        "runtime_start_shutdown",
        ImplementationLayer::Reference,
        ImplementationPlane::Ops,
        "start reference runtime then graceful shutdown",
    ),
    scenario(
        "BENCH-016",
        "evidence_json_record_write",
        ImplementationLayer::Reference,
        ImplementationPlane::Ops,
        "build one `ImplementationEvidenceRecord` and serialize JSON",
    ),
    scenario(
        "BENCH-017",
        "product_policy_evaluate",
        ImplementationLayer::Product,
        ImplementationPlane::Policy,
        "one product policy input per plane",
    ),
    scenario(
        "BENCH-018",
        "product_projection_mapping",
        ImplementationLayer::Product,
        ImplementationPlane::Persistence,
        "one projection mapping per record class",
    ),
    scenario(
        "BENCH-019",
        "product_runtime_select",
        ImplementationLayer::Product,
        ImplementationPlane::Deployment,
        "select local product runtime profile",
    ),
    scenario(
        "BENCH-020",
        "product_drain_plan_create",
        ImplementationLayer::Product,
        ImplementationPlane::Rollback,
        "create drain plan for signaling / turn / sfu",
    ),
];

const fn scenario(
    id: &'static str,
    name: &'static str,
    layer: ImplementationLayer,
    plane: ImplementationPlane,
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
            ImplementationLayer::Product,
            ImplementationPlane::Policy,
            "unit workload summary",
        );

        assert_eq!(constructed.id, "BENCH-UNIT");
        assert_eq!(constructed.name, "unit_runtime_constructor");
        assert_eq!(constructed.layer, ImplementationLayer::Product);
        assert_eq!(constructed.plane, ImplementationPlane::Policy);
        assert_eq!(constructed.workload_summary, "unit workload summary");
    }
}
