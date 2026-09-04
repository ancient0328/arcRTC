/// operational probe surface の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationalProbeClass {
    /// health probe surface.
    Health,
    /// readiness probe surface.
    Readiness,
    /// liveness probe surface.
    Liveness,
    /// admin probe surface.
    Admin,
}

/// health probe と readiness probe の relation observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HealthReadinessRelation {
    /// health probe の opaque reference です。
    pub health_probe_ref: &'static str,
    /// readiness probe の opaque reference です。
    pub readiness_probe_ref: &'static str,
}

impl HealthReadinessRelation {
    /// health/readiness relation を束ねます。
    pub const fn new(health_probe_ref: &'static str, readiness_probe_ref: &'static str) -> Self {
        Self {
            health_probe_ref,
            readiness_probe_ref,
        }
    }
}

/// operator authorization の observation reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperatorAuthorizationObservationRef {
    /// operator authorization observation の opaque value です。
    pub value: &'static str,
}

impl OperatorAuthorizationObservationRef {
    /// operator authorization observation ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

/// admin maintenance command の surface model です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdminMaintenanceCommand {
    /// maintenance command reference です。
    pub command_ref: &'static str,
    /// operator reference です。
    pub operator_ref: OperatorAuthorizationObservationRef,
}

impl AdminMaintenanceCommand {
    /// command ref と operator observation ref を束ねます。
    pub const fn new(
        command_ref: &'static str,
        operator_ref: OperatorAuthorizationObservationRef,
    ) -> Self {
        Self {
            command_ref,
            operator_ref,
        }
    }
}

/// admin probe surface construction input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdminProbeSurfaceInput {
    /// probe class です。
    pub probe_class: OperationalProbeClass,
    /// health/readiness relation です。
    pub health_readiness_relation: HealthReadinessRelation,
    /// maintenance command surface です。
    pub maintenance_command: AdminMaintenanceCommand,
}

impl AdminProbeSurfaceInput {
    /// admin probe surface input を束ねます。
    pub const fn new(
        probe_class: OperationalProbeClass,
        health_readiness_relation: HealthReadinessRelation,
        maintenance_command: AdminMaintenanceCommand,
    ) -> Self {
        Self {
            probe_class,
            health_readiness_relation,
            maintenance_command,
        }
    }
}

/// admin probe surface observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdminProbeSurface {
    /// probe class です。
    pub probe_class: OperationalProbeClass,
    /// health/readiness relation です。
    pub health_readiness_relation: HealthReadinessRelation,
    /// maintenance command surface です。
    pub maintenance_command: AdminMaintenanceCommand,
}

/// admin probe surface を構成します。
///
/// authorization decision、domain success、managed-runtime readiness authority は生成しません。
pub const fn build_admin_probe_surface(input: AdminProbeSurfaceInput) -> AdminProbeSurface {
    AdminProbeSurface {
        probe_class: input.probe_class,
        health_readiness_relation: input.health_readiness_relation,
        maintenance_command: input.maintenance_command,
    }
}
