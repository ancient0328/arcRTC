use arcrtc_entrypoint_admin::{AdminProbeClass, HealthAdminFailureKind};

fn main() {
    let Some(token) = std::env::args().nth(1) else {
        eprintln!(
            "{}",
            HealthAdminFailureKind::RuntimeConfigMissing.reason_code()
        );
        std::process::exit(2);
    };

    // Admin probe CLI は typed decode に閉じ、readiness 実体の成立は主張しません。
    let _probe_class = match token.as_str() {
        "process-liveness" => AdminProbeClass::ProcessLiveness,
        "driver-dependency-readiness" => AdminProbeClass::DriverDependencyReadiness,
        "core-policy-readiness" => AdminProbeClass::CorePolicyReadiness,
        "composition-readiness" => AdminProbeClass::CompositionReadiness,
        "maintenance-status" => AdminProbeClass::MaintenanceStatus,
        "operator-action-result" => AdminProbeClass::OperatorActionResult,
        _ => {
            eprintln!(
                "{}",
                HealthAdminFailureKind::RuntimeConfigInvalid.reason_code()
            );
            std::process::exit(2);
        }
    };

    println!("probe_class={token}");
}
