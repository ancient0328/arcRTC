use arcrtc_entrypoint_topology::{DeploymentTopologyClass, DeploymentTopologyFailureKind};

fn main() {
    let Some(token) = std::env::args().nth(1) else {
        eprintln!(
            "{}",
            DeploymentTopologyFailureKind::DeploymentTopologyUnsupported.reason_code()
        );
        std::process::exit(2);
    };

    // source-owned token 表を topology surface の既存 enum へ写像します。
    let topology_class = match token.as_str() {
        "single-process-local" => DeploymentTopologyClass::SingleProcessLocal,
        "split-plane-same-host" => DeploymentTopologyClass::SplitPlaneSameHost,
        "split-plane-networked" => DeploymentTopologyClass::SplitPlaneNetworked,
        "multi-node-experimental" => DeploymentTopologyClass::MultiNodeExperimental,
        "external-managed-dependency" => DeploymentTopologyClass::ExternalManagedDependency,
        _ => {
            eprintln!(
                "{}",
                DeploymentTopologyFailureKind::DeploymentTopologyUnsupported.reason_code()
            );
            std::process::exit(2);
        }
    };

    println!(
        "topology_class={} requires_explicit_service_endpoint_wiring={} requires_networked_internal_service_relation={} requires_explicit_experimental_enablement={} requires_external_dependency_contract={}",
        token,
        topology_class.requires_explicit_service_endpoint_wiring(),
        topology_class.requires_networked_internal_service_relation(),
        topology_class.requires_explicit_experimental_enablement(),
        topology_class.requires_external_dependency_contract()
    );
}
