#![allow(non_snake_case)]

use std::io::BufRead;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use arcrtc_core_identity::{OpaqueReference, PacketId, ReferenceAuthority};
use arcrtc_core_security::{CredentialPolicyReferenceState, MessageIntegrityPolicyRef};
use arcrtc_core_turn::{
    classify_turn_security, AmplificationBoundDecision, TurnFailureKind, TurnNonceEvaluationState,
    TurnNoncePolicy, TurnNonceRef, TurnReplayDecision, TurnSecurityClassificationDecision,
    TurnSecurityClassificationInput,
};

const ERROR_RESPONSE: u8 = 0x05;

// Test Roadmap の named assertion rule に合わせ、二重アンダースコア名を維持します。

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

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy)
        .expect("test fixture uses accepted references")
}

fn classification_input(
    nonce_state: TurnNonceEvaluationState,
    replay_window_declared: bool,
    amplification_bound_declared: bool,
    amplification_within_bound: bool,
) -> TurnSecurityClassificationInput {
    TurnSecurityClassificationInput::new(
        MessageIntegrityPolicyRef::new(
            reference("integrity-policy:turn-02"),
            CredentialPolicyReferenceState::Present,
        ),
        PacketId::new(reference("packet:turn-02")),
        TurnNonceRef::new(
            reference("nonce:turn-02"),
            nonce_state,
            TurnNoncePolicy::new(
                replay_window_declared,
                amplification_bound_declared,
                amplification_within_bound,
            ),
        ),
    )
}

fn exchange(input: &[u8]) -> Vec<u8> {
    let (mut server, addr) = ServerProc::spawn();
    let client = UdpSocket::bind("127.0.0.1:0").expect("client socket must bind");
    client
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout must be set");
    client
        .send_to(input, &addr)
        .expect("request datagram must be sent");
    let mut buf = [0u8; 4096];
    let (n, _peer) = client
        .recv_from(&mut buf)
        .expect("response datagram must be received");
    server.kill_and_wait();
    buf[..n].to_vec()
}

fn expected_error(reason: &str) -> Vec<u8> {
    let mut expected = vec![ERROR_RESPONSE];
    expected.extend_from_slice(reason.as_bytes());
    expected
}

#[test]
fn t_turn_02_security_abuse_fail_closed() {
    assert_t_turn_02__nonce();
    assert_t_turn_02__integrity();
    assert_t_turn_02__replay();
    assert_t_turn_02__malformed();
    assert_t_turn_02__amplification_resource_bound();
}

fn assert_t_turn_02__nonce() {
    assert_eq!(
        classify_turn_security(classification_input(
            TurnNonceEvaluationState::Present,
            true,
            true,
            true,
        )),
        TurnSecurityClassificationDecision::new(
            TurnReplayDecision::Accepted,
            AmplificationBoundDecision::WithinBound,
            None,
        )
    );
}

fn assert_t_turn_02__integrity() {
    let input = classification_input(TurnNonceEvaluationState::Present, true, true, true);
    assert_eq!(
        input.credential_integrity_policy_ref().as_str(),
        "integrity-policy:turn-02"
    );
}

fn assert_t_turn_02__replay() {
    assert_eq!(
        classify_turn_security(classification_input(
            TurnNonceEvaluationState::Replayed,
            true,
            true,
            true,
        )),
        TurnSecurityClassificationDecision::new(
            TurnReplayDecision::Rejected(TurnFailureKind::CredentialInvalid),
            AmplificationBoundDecision::WithinBound,
            Some(TurnFailureKind::CredentialInvalid),
        )
    );
}

fn assert_t_turn_02__malformed() {
    assert_eq!(
        exchange(&[0x00, 0x03, 0x00]),
        expected_error("malformed_turn_message")
    );
}

fn assert_t_turn_02__amplification_resource_bound() {
    assert_eq!(
        classify_turn_security(classification_input(
            TurnNonceEvaluationState::Present,
            true,
            true,
            false,
        )),
        TurnSecurityClassificationDecision::new(
            TurnReplayDecision::Accepted,
            AmplificationBoundDecision::Rejected(TurnFailureKind::TurnRelayQueueBoundExceeded),
            Some(TurnFailureKind::TurnRelayQueueBoundExceeded),
        )
    );
    assert_eq!(
        exchange(&vec![0x00; 2049]),
        expected_error("frame_size_bound_exceeded")
    );
}
