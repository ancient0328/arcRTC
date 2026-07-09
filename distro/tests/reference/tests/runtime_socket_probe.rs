//! reference runtime socket probe の実loopback bindを検査します。

use std::net::SocketAddr;

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_reference_composition::ReferenceCompositionState;
use arcrtc_reference_ops::{DistroRuntimeState, DistroShutdownMode, ReferenceRuntime};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

#[tokio::test]
async fn reference_runtime_socket_probe_binds_loopback_while_running() {
    let bind_addr: SocketAddr = "127.0.0.1:0"
        .parse()
        .expect("loopback bind address must parse");
    let mut runtime = ReferenceRuntime::new(ReferenceCompositionState::default());

    let (started, probe) = runtime
        .start_with_socket_probe(cid("reference-runtime-socket-probe"), bind_addr)
        .await
        .expect("socket probe start must succeed");
    println!(
        "reference runtime socket probe input={bind_addr} local_addr={} state={:?}",
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
    let stopped = runtime
        .shutdown(
            cid("reference-runtime-socket-probe-stop"),
            DistroShutdownMode::GracefulLocal,
        )
        .expect("socket probe shutdown must succeed");
    println!(
        "reference runtime socket probe shutdown local_addr={} state={:?}",
        probe.local_addr(),
        stopped.state
    );
    assert_eq!(stopped.state, DistroRuntimeState::Stopped);
    assert_eq!(runtime.state(), DistroRuntimeState::Stopped);

    // shutdown 後に同一addressへ再bindできることを、listener解放の直接証跡にします。
    let rebound = tokio::net::TcpListener::bind(released_addr)
        .await
        .expect("released reference socket address must be reusable");
    assert_eq!(
        rebound
            .local_addr()
            .expect("rebound reference socket must expose local address"),
        released_addr
    );
}
