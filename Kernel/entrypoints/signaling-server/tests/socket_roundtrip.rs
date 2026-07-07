use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const JOIN_FRAME: &[u8] = &[0x00, 0x00, 0x02, 0x72, 0x31, 0x63, 0x31];
const JOINED: u8 = 0x00;
const REJECTED: u8 = 0x01;

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
    let mut stream = TcpStream::connect(addr).expect("client must connect to signaling server");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout must be set");
    stream
        .write_all(input)
        .expect("input frame must be written");
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("client write side must close");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("response frame must be readable");
    response
}

fn assert_rejected_reason(label: &str, input: &[u8], expected_reason: &str) {
    let (mut server, addr) = ServerProc::spawn();
    let response = exchange(&addr, input);
    println!("case={label} input={input:?} response={response:?}");
    assert_eq!(
        response.first().copied(),
        Some(REJECTED),
        "{label} must reject"
    );
    assert_eq!(
        std::str::from_utf8(&response[1..]).expect("reason must be UTF-8"),
        expected_reason
    );
    server.kill_and_wait();
}

#[test]
fn well_formed_join_is_accepted() {
    let (mut server, addr) = ServerProc::spawn();
    let response = exchange(&addr, JOIN_FRAME);
    println!("case=T1 input={JOIN_FRAME:?} response={response:?}");
    assert_eq!(response.first().copied(), Some(JOINED), "T1 must accept");
    assert_eq!(response.len(), 1, "accepted join must not expose payload");
    server.kill_and_wait();
}

#[test]
fn adversarial_inputs_fail_closed() {
    let cases: &[(&str, &[u8], &str)] = &[
        (
            "T2-unknown-discriminant",
            &[0x07, 0x00, 0x01, 0x72, 0x63],
            "external_enum_unmapped",
        ),
        ("T2-short-header", &[0x00, 0x00], "external_decode_failed"),
        (
            "T2-room-len-overflow",
            &[0x00, 0xFF, 0xFF],
            "external_decode_failed",
        ),
        (
            "T2-empty-room",
            &[0x00, 0x00, 0x00, 0x63],
            "missing_required_wire_field",
        ),
        (
            "T2-missing-correlation",
            &[0x00, 0x00, 0x01, 0x72],
            "missing_correlation_id",
        ),
    ];

    for (label, input, reason) in cases {
        assert_rejected_reason(label, input, reason);
    }
}
