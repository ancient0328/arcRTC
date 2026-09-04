use std::io::BufRead;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const ERROR_RESPONSE: u8 = 0x05;
const STUN_BINDING_REQUEST: &[u8] = &[
    0x00, 0x01, 0x00, 0x00, 0x21, 0x12, 0xA4, 0x42, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0A, 0x0B,
];
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

fn frame(prefix: &[u8], body: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::from(prefix);
    bytes.extend_from_slice(body);
    bytes
}

fn exchange(input: &[u8]) -> Vec<u8> {
    let (mut server, addr) = ServerProc::spawn();
    let client = UdpSocket::bind("127.0.0.1:0").expect("client UDP socket must bind");
    client
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("read timeout must be set");
    client
        .send_to(input, &addr)
        .expect("input datagram must be sent");
    let mut buf = [0u8; 2048];
    let (n, _peer) = client
        .recv_from(&mut buf)
        .expect("response datagram must be received");
    let response = buf[..n].to_vec();
    server.kill_and_wait();
    response
}

fn expected_error(reason: &str) -> Vec<u8> {
    let mut expected = vec![ERROR_RESPONSE];
    expected.extend_from_slice(reason.as_bytes());
    expected
}

fn assert_error_response(label: &str, input: &[u8], reason: &str) {
    let response = exchange(input);
    println!("case={label} input={input:?} response={response:?}");
    assert_eq!(
        response,
        expected_error(reason),
        "{label} response mismatch"
    );
}

#[test]
fn well_formed_binding_request_returns_unsupported_method() {
    assert_error_response(
        "binding-request",
        STUN_BINDING_REQUEST,
        "unsupported_turn_method",
    );
}

#[test]
fn adversarial_datagrams_fail_closed() {
    let mut malformed_cookie = vec![0x00, 0x03, 0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
    malformed_cookie.extend_from_slice(&TX);

    let mut length_conflict = vec![0x00, 0x03, 0x00, 0x04];
    length_conflict.extend_from_slice(&COOKIE);
    length_conflict.extend_from_slice(&TX);

    let mut non_multiple_length = vec![0x00, 0x03, 0x00, 0x02];
    non_multiple_length.extend_from_slice(&COOKIE);
    non_multiple_length.extend_from_slice(&TX);
    non_multiple_length.extend_from_slice(&[0xFF, 0xFF]);

    let non_stun_type = frame(
        &[0xC0, 0x01, 0x00, 0x00],
        &[COOKIE.as_slice(), TX.as_slice()].concat(),
    );
    let unknown_type = frame(
        &[0x00, 0x0F, 0x00, 0x00],
        &[COOKIE.as_slice(), TX.as_slice()].concat(),
    );
    let create_permission = frame(
        &[0x00, 0x08, 0x00, 0x00],
        &[COOKIE.as_slice(), TX.as_slice()].concat(),
    );
    let allocate_without_credential = frame(
        &[0x00, 0x03, 0x00, 0x00],
        &[COOKIE.as_slice(), TX.as_slice()].concat(),
    );
    let refresh_without_allocation = frame(
        &[0x00, 0x04, 0x00, 0x00],
        &[COOKIE.as_slice(), TX.as_slice()].concat(),
    );
    let over_bound = vec![0x00; 2049];

    let cases: Vec<(&str, Vec<u8>, &str)> = vec![
        (
            "malformed-cookie",
            malformed_cookie,
            "malformed_turn_message",
        ),
        (
            "short-header",
            vec![0x00, 0x03, 0x00],
            "malformed_turn_message",
        ),
        ("length-conflict", length_conflict, "malformed_turn_message"),
        (
            "non-multiple-length",
            non_multiple_length,
            "malformed_turn_message",
        ),
        ("non-stun-type", non_stun_type, "malformed_turn_message"),
        ("unknown-type", unknown_type, "unsupported_turn_method"),
        ("over-bound", over_bound, "frame_size_bound_exceeded"),
        (
            "create-permission-missing-attribute",
            create_permission,
            "malformed_turn_message",
        ),
        (
            "allocate-without-credential",
            allocate_without_credential,
            "credential_missing",
        ),
        (
            "refresh-without-allocation",
            refresh_without_allocation,
            "allocation_not_found",
        ),
    ];

    for (label, input, reason) in cases {
        assert_error_response(label, &input, reason);
    }
}
