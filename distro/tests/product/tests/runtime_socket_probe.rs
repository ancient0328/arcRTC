//! product runtime socket probe の実loopback bindを検査します。

use std::net::SocketAddr;

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_distro_evidence::DistroPlane;
use arcrtc_product_deployment::{
    build_product_runtime_profile, runtime::DistroRuntimeState, ProductHostClass, ProductRuntime,
};
use arcrtc_product_rollback::{plan_drain, ProductDrainMode};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

#[tokio::test]
async fn product_runtime_socket_probe_binds_loopback_while_running() {
    let bind_addr: SocketAddr = "127.0.0.1:0"
        .parse()
        .expect("loopback bind address must parse");
    let mut runtime = ProductRuntime::new(build_product_runtime_profile(
        ProductHostClass::LocalSingleHost,
    ));

    let (started, probe) = runtime
        .start_with_socket_probe(cid("product-runtime-socket-probe"), bind_addr)
        .await
        .expect("socket probe start must succeed");
    println!(
        "product runtime socket probe input={bind_addr} local_addr={} state={:?}",
        probe.local_addr(),
        started.state
    );
    assert_eq!(started.state, DistroRuntimeState::Running);
    assert!(probe.local_addr().ip().is_loopback());
    assert_ne!(probe.local_addr().port(), 0);

    // 同一addressへの再bind失敗を、runtimeがlistenerを保持している直接証跡にします。
    assert!(tokio::net::TcpListener::bind(probe.local_addr())
        .await
        .is_err());

    let released_addr = probe.local_addr();
    let drain = plan_drain(
        cid("product-runtime-socket-probe-stop"),
        vec![DistroPlane::Signaling],
        ProductDrainMode::ReferenceLocal,
    );
    let stopped = runtime
        .drain(drain)
        .expect("socket probe drain must succeed");
    println!(
        "product runtime socket probe drain local_addr={} state={:?}",
        probe.local_addr(),
        stopped.state
    );
    assert_eq!(stopped.state, DistroRuntimeState::Stopped);

    // drain 後に同一addressへ再bindできることを、listener解放の直接証跡にします。
    let rebound = tokio::net::TcpListener::bind(released_addr)
        .await
        .expect("released product socket address must be reusable");
    assert_eq!(
        rebound
            .local_addr()
            .expect("rebound product socket must expose local address"),
        released_addr
    );
}
