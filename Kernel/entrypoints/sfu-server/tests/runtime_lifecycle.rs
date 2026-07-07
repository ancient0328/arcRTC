// Roadmap の assertion 名をそのまま残すため、このテストファイルだけ許可します。
#![allow(non_snake_case)]

use std::io::BufRead;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority, StartupRunId};
use arcrtc_core_runtime::{
    decide_runtime_task_lifecycle, BackpressureDecision, CancellationPropagationRule,
    RuntimeFailureKind, RuntimeTaskClass, RuntimeTaskInputReferenceClass,
    RuntimeTaskLifecycleDecision, RuntimeTaskLifecycleInput, RuntimeTaskLifecycleOutcome,
    RuntimeTaskLifecyclePolicy, RuntimeTaskOutputObservation, RuntimeTaskOwningLayer,
    RuntimeWorkerBound, ShutdownDrainDecision, SupervisionScope, SupervisorRestartObservation,
};

struct ServerProc {
    child: Child,
}

impl ServerProc {
    fn spawn() -> (Self, String, mpsc::Receiver<std::io::Result<String>>) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_arcrtc-sfu-server"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("sfu server must spawn");
        let stdout = child.stdout.take().expect("stdout must be piped");
        let mut reader = std::io::BufReader::new(stdout);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("listening line must be readable");
        let addr = line
            .strip_prefix("listening=")
            .expect("first stdout line must expose listening address")
            .trim()
            .to_owned();
        let (stdout_tx, stdout_rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in reader.lines() {
                let _ = stdout_tx.send(line);
            }
        });

        (Self { child }, addr, stdout_rx)
    }

    fn kill_and_wait(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ServerProc {
    fn drop(&mut self) {
        self.kill_and_wait();
    }
}

fn exchange(addr: &str, input: &[u8]) -> Vec<u8> {
    let client = UdpSocket::bind("127.0.0.1:0").expect("client UDP socket must bind");
    client
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout must be set");
    client
        .send_to(input, addr)
        .expect("request datagram must be sent");
    let mut buf = [0u8; 2048];
    let (n, _peer) = client
        .recv_from(&mut buf)
        .expect("response datagram must be received");
    buf[..n].to_vec()
}

fn startup_run_id() -> StartupRunId {
    StartupRunId::new(
        OpaqueReference::accept("startup:runtime:sfu", ReferenceAuthority::CorePolicy)
            .expect("startup reference is fixed"),
    )
}

fn driver_policy() -> RuntimeTaskLifecyclePolicy {
    RuntimeTaskLifecyclePolicy::try_new(
        RuntimeTaskClass::DriverIoWorker,
        Some(SupervisionScope::EntrypointStartupRun),
        RuntimeTaskOwningLayer::Driver,
        RuntimeTaskInputReferenceClass::OpaqueTaskReference,
        RuntimeTaskOutputObservation::SpawnObserved,
        CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
        true,
        true,
    )
    .expect("runtime lifecycle policy is fixed")
}

fn runtime_decision(
    worker_bound: RuntimeWorkerBound,
    shutdown_drain: ShutdownDrainDecision,
    supervisor_restart: SupervisorRestartObservation,
) -> RuntimeTaskLifecycleDecision {
    decide_runtime_task_lifecycle(RuntimeTaskLifecycleInput::new(
        startup_run_id(),
        None,
        driver_policy(),
        Some("task:sfu-runtime"),
        worker_bound,
        shutdown_drain,
        BackpressureDecision::WithinBound,
        supervisor_restart,
    ))
    .expect("runtime decision must be closed")
}

#[test]
fn assert_t_runtime_01__concurrency() {
    let (mut server, addr, stdout_rx) = ServerProc::spawn();
    let addr = Arc::new(addr);
    let mut handles = Vec::new();

    for _ in 0..4 {
        let addr = Arc::clone(&addr);
        handles.push(std::thread::spawn(move || {
            exchange(&addr, b"ARCRTC-SFU/ICE/CONNECTED")
        }));
    }

    for handle in handles {
        assert_eq!(handle.join().expect("client thread must join"), vec![0x20]);
    }
    for _ in 0..4 {
        let line = stdout_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("SFU outcome line must be emitted")
            .expect("SFU outcome line must be readable");
        assert!(line.starts_with("outcome=ok secure=ice_connected "));
    }
    server.kill_and_wait();
}

#[test]
fn assert_t_runtime_01__bounded_workers() {
    let decision = runtime_decision(
        RuntimeWorkerBound::new(false, true, true),
        ShutdownDrainDecision::Continue,
        SupervisorRestartObservation::NotObserved,
    );

    assert_eq!(decision.outcome(), RuntimeTaskLifecycleOutcome::Rejected);
    assert_eq!(
        decision.reason(),
        Some(RuntimeFailureKind::RuntimeTaskQueueBoundExceeded)
    );
}

#[test]
fn assert_t_runtime_01__shutdown_drain() {
    let decision = runtime_decision(
        RuntimeWorkerBound::new(true, true, true),
        ShutdownDrainDecision::Draining,
        SupervisorRestartObservation::NotObserved,
    );

    assert_eq!(decision.outcome(), RuntimeTaskLifecycleOutcome::Cancelled);
    assert_eq!(decision.reason(), None);
}

#[test]
fn assert_t_runtime_01__supervisor_observation() {
    let decision = runtime_decision(
        RuntimeWorkerBound::new(true, true, true),
        ShutdownDrainDecision::Continue,
        SupervisorRestartObservation::Failed(RuntimeFailureKind::RuntimeTaskJoinFailed),
    );

    assert_eq!(decision.outcome(), RuntimeTaskLifecycleOutcome::Failed);
    assert_eq!(
        decision.reason(),
        Some(RuntimeFailureKind::RuntimeTaskJoinFailed)
    );
}
