use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
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

const JOIN_FRAME: &[u8] = &[0x00, 0x00, 0x02, 0x72, 0x31, 0x63, 0x31];

struct ServerProc {
    child: Child,
}

impl ServerProc {
    fn spawn() -> (Self, String) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_arcrtc-signaling-server"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("signaling server must spawn");
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

fn exchange(addr: &str, input: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(addr).expect("client must connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout must be set");
    stream.write_all(input).expect("input frame must be sent");
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("client write side must close");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("response must be readable");
    response
}

fn startup_run_id() -> StartupRunId {
    StartupRunId::new(
        OpaqueReference::accept("startup:runtime:signaling", ReferenceAuthority::CorePolicy)
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
        Some("task:signaling-runtime"),
        worker_bound,
        shutdown_drain,
        BackpressureDecision::WithinBound,
        supervisor_restart,
    ))
    .expect("runtime decision must be closed")
}

#[test]
fn assert_concurrency() {
    let (mut server, addr) = ServerProc::spawn();
    let addr = Arc::new(addr);
    let mut handles = Vec::new();

    for _ in 0..4 {
        let addr = Arc::clone(&addr);
        handles.push(std::thread::spawn(move || exchange(&addr, JOIN_FRAME)));
    }

    for handle in handles {
        assert_eq!(handle.join().expect("client thread must join"), vec![0x00]);
    }
    server.kill_and_wait();
}

#[test]
fn assert_bounded_workers() {
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
fn assert_shutdown_drain() {
    let decision = runtime_decision(
        RuntimeWorkerBound::new(true, true, true),
        ShutdownDrainDecision::Draining,
        SupervisorRestartObservation::NotObserved,
    );

    assert_eq!(decision.outcome(), RuntimeTaskLifecycleOutcome::Cancelled);
    assert_eq!(decision.reason(), None);
}

#[test]
fn assert_supervisor_observation() {
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
