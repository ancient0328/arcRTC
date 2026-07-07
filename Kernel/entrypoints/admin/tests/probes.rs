use arcrtc_entrypoint_admin::{
    build_admin_probe_surface, AdminMaintenanceCommand, AdminProbeSurfaceInput,
    HealthReadinessRelation, OperationalProbeClass, OperatorAuthorizationObservationRef,
};

#[test]
fn admin_probe_surface_preserves_health_readiness_relation() {
    let relation = HealthReadinessRelation::new("health-probe", "readiness-probe");
    let maintenance = AdminMaintenanceCommand::new(
        "shutdown-drain",
        OperatorAuthorizationObservationRef::new("operator-observation"),
    );
    let surface = build_admin_probe_surface(AdminProbeSurfaceInput::new(
        OperationalProbeClass::Readiness,
        relation,
        maintenance,
    ));

    // admin entrypoint は probe surface を構成するだけで、authorization decision は所有しません。
    assert_eq!(surface.probe_class, OperationalProbeClass::Readiness);
    assert_eq!(
        surface.health_readiness_relation.health_probe_ref,
        "health-probe"
    );
    assert_eq!(
        surface.health_readiness_relation.readiness_probe_ref,
        "readiness-probe"
    );
    assert_eq!(surface.maintenance_command.command_ref, "shutdown-drain");
    assert_eq!(
        surface.maintenance_command.operator_ref.value,
        "operator-observation"
    );
}

#[test]
fn admin_probe_surface_covers_all_probe_classes() {
    let relation = HealthReadinessRelation::new("health-probe", "readiness-probe");
    let maintenance = AdminMaintenanceCommand::new(
        "rollback",
        OperatorAuthorizationObservationRef::new("operator-observation"),
    );
    let classes = [
        OperationalProbeClass::Health,
        OperationalProbeClass::Readiness,
        OperationalProbeClass::Liveness,
        OperationalProbeClass::Admin,
    ];

    for probe_class in classes {
        let surface = build_admin_probe_surface(AdminProbeSurfaceInput::new(
            probe_class,
            relation,
            maintenance,
        ));
        assert_eq!(surface.probe_class, probe_class);
        assert_eq!(surface.maintenance_command.command_ref, "rollback");
    }
}
