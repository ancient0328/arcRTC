// Roadmap の assertion 名をそのまま残すため、このテストファイルだけ許可します。
#![allow(non_snake_case)]

use std::io::BufRead;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority, StartupRunId};
use arcrtc_core_runtime::{
    decide_runtime_task_lifecycle, BackpressureDecision, CancellationPropagationRule,
    RuntimeFailureKind, RuntimeTaskClass, RuntimeTaskInputReferenceClass,
    RuntimeTaskLifecycleDecision, RuntimeTaskLifecycleInput, RuntimeTaskLifecycleOutcome,
    RuntimeTaskLifecyclePolicy, RuntimeTaskOutputObservation, RuntimeTaskOwningLayer,
    RuntimeWorkerBound, ShutdownDrainDecision, SupervisionScope, SupervisorRestartObservation,
};

const COOKIE: [u8; 4] = [0x21, 0x12, 0xA4, 0x42];
const TX: [u8; 12] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
];

struct ServerProc {
    child: Child,
}

impl ServerProc {
    fn spawn() -> (Self, String) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_arcrtc-turn-server"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("turn server must spawn");
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

        (Self { child }, addr)
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

fn attribute(attribute_type: u16, value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + ((value.len() + 3) & !3));
    out.extend_from_slice(&attribute_type.to_be_bytes());
    out.extend_from_slice(&(value.len() as u16).to_be_bytes());
    out.extend_from_slice(value);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    out
}

fn datagram(method: u16, attributes: Vec<Vec<u8>>) -> Vec<u8> {
    let body = attributes.concat();
    let mut out = Vec::with_capacity(20 + body.len());
    out.extend_from_slice(&method.to_be_bytes());
    out.extend_from_slice(&(body.len() as u16).to_be_bytes());
    out.extend_from_slice(&COOKIE);
    out.extend_from_slice(&TX);
    out.extend_from_slice(&body);
    out
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
        OpaqueReference::accept("startup:runtime:turn", ReferenceAuthority::CorePolicy)
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
        Some("task:turn-runtime"),
        worker_bound,
        shutdown_drain,
        BackpressureDecision::WithinBound,
        supervisor_restart,
    ))
    .expect("runtime decision must be closed")
}

#[test]
fn assert_t_runtime_01__concurrency() {
    let (mut server, addr) = ServerProc::spawn();
    let addr = Arc::new(addr);
    let allocate = Arc::new(datagram(0x0003, vec![attribute(0x0006, b"credential")]));
    let mut handles = Vec::new();

    for _ in 0..4 {
        let addr = Arc::clone(&addr);
        let allocate = Arc::clone(&allocate);
        handles.push(std::thread::spawn(move || exchange(&addr, &allocate)));
    }

    for handle in handles {
        assert_eq!(handle.join().expect("client thread must join"), vec![0x00]);
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
