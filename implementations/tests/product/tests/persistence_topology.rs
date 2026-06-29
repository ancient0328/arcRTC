//! product persistence topology 境界を検査します。

use std::{fs, path::PathBuf};

use arcrtc_implementation_evidence::{ImplementationEvidenceReason, ImplementationPlane};
use arcrtc_product_persistence_topology::{
    build_persistence_topology, map_product_projection, ProductPersistenceMode,
    ProductPersistenceRecordClass, ProductPersistenceTopologyError,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

#[test]
fn persistence_topology_keeps_provider_deferred_boundary() {
    let topology = build_persistence_topology(ProductPersistenceMode::InMemoryProjectionOnly)
        .expect("in-memory projection topology must be accepted");
    assert_eq!(
        topology.mode,
        ProductPersistenceMode::InMemoryProjectionOnly
    );
    assert!(topology
        .record_classes
        .contains(&ProductPersistenceRecordClass::SessionProjection));
    assert!(topology
        .record_classes
        .contains(&ProductPersistenceRecordClass::AllocationProjection));
    assert!(topology
        .record_classes
        .contains(&ProductPersistenceRecordClass::RouteProjection));
    assert!(topology
        .record_classes
        .contains(&ProductPersistenceRecordClass::EvidenceProjection));

    assert_eq!(
        build_persistence_topology(ProductPersistenceMode::ProviderDeferred),
        Err(ProductPersistenceTopologyError::ProviderNotAdmitted)
    );
    assert_eq!(
        build_persistence_topology(ProductPersistenceMode::NotAdmitted),
        Err(ProductPersistenceTopologyError::ProviderNotAdmitted)
    );
}

#[test]
fn persistence_projection_mapper_rejects_cross_plane_shape_mismatch() {
    let session_mapping = map_product_projection(
        ImplementationPlane::Signaling,
        ProductPersistenceRecordClass::SessionProjection,
    )
    .expect("signaling session projection must map");
    assert_eq!(
        session_mapping.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );

    assert_eq!(
        map_product_projection(
            ImplementationPlane::Signaling,
            ProductPersistenceRecordClass::RouteProjection,
        ),
        Err(ProductPersistenceTopologyError::ProjectionMappingViolation)
    );
}

#[test]
fn kpi_product_persistence_executes_without_kernel_state_ownership() {
    for path in [
        "product-implementation/persistence-topology/src/topology.rs",
        "product-implementation/persistence-topology/src/mapper.rs",
    ] {
        let source = fs::read_to_string(root().join(path))
            .expect("product persistence source must be readable");
        for forbidden in [
            "arcrtc_core_",
            "ReferenceSignalingOutcome",
            "ReferenceTurnOutcome",
            "ReferenceSfuOutcome",
            "ReferenceCompositionOutcome",
            "ProviderConnection",
            "DynamoDB",
            "Postgres",
            "Redis",
            "KernelState",
        ] {
            assert!(
                !source.contains(forbidden),
                "product persistence must not own non-projection state: {path} {forbidden}"
            );
        }
    }

    let accepted_pairs = [
        (
            ImplementationPlane::Signaling,
            ProductPersistenceRecordClass::SessionProjection,
        ),
        (
            ImplementationPlane::Turn,
            ProductPersistenceRecordClass::AllocationProjection,
        ),
        (
            ImplementationPlane::Sfu,
            ProductPersistenceRecordClass::RouteProjection,
        ),
        (
            ImplementationPlane::Monitoring,
            ProductPersistenceRecordClass::EvidenceProjection,
        ),
    ];
    for (plane, record_class) in accepted_pairs {
        let mapping = map_product_projection(plane, record_class)
            .expect("allowed product projection mapping must be accepted");
        assert_eq!(mapping.source_plane, plane);
        assert_eq!(mapping.record_class, record_class);
        assert_eq!(
            mapping.implementation_reason,
            ImplementationEvidenceReason::ImplementationOk
        );
    }
}
