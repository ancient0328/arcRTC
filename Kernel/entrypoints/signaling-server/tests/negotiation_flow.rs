use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const JOINED: u8 = 0x00;
const REJECTED: u8 = 0x01;
const PARTICIPANT_LEFT: u8 = 0x03;
const OFFER_RECEIVED: u8 = 0x04;
const ANSWER_RECEIVED: u8 = 0x05;
const ICE_CANDIDATE_RECEIVED: u8 = 0x06;

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

fn frame(kind: u8, correlation: &str) -> Vec<u8> {
    let room = b"r1";
    let mut frame = Vec::with_capacity(3 + room.len() + correlation.len());
    frame.push(kind);
    frame.extend_from_slice(&(room.len() as u16).to_be_bytes());
    frame.extend_from_slice(room);
    frame.extend_from_slice(correlation.as_bytes());
    frame
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

fn joined_server() -> (ServerProc, String) {
    let (server, addr) = ServerProc::spawn();
    assert_eq!(exchange(&addr, &frame(0, "c1")), vec![JOINED]);
    (server, addr)
}

#[test]
fn negotiation_flow_roundtrip() {
    assert_offer();
    assert_answer();
    assert_ice_candidate();
    assert_reconnect();
    assert_leave();
    assert_timeout();
}

fn assert_offer() {
    let (mut server, addr) = joined_server();
    assert_eq!(exchange(&addr, &frame(2, "c2")), vec![OFFER_RECEIVED]);
    server.kill_and_wait();
}

fn assert_answer() {
    let (mut server, addr) = joined_server();
    assert_eq!(exchange(&addr, &frame(3, "c3")), vec![ANSWER_RECEIVED]);
    server.kill_and_wait();
}

fn assert_ice_candidate() {
    let (mut server, addr) = joined_server();
    assert_eq!(
        exchange(&addr, &frame(4, "c4")),
        vec![ICE_CANDIDATE_RECEIVED]
    );
    server.kill_and_wait();
}

fn assert_reconnect() {
    let (mut server, addr) = joined_server();
    assert_eq!(exchange(&addr, &frame(6, "c6")), vec![JOINED]);
    server.kill_and_wait();
}

fn assert_leave() {
    let (mut server, addr) = joined_server();
    assert_eq!(exchange(&addr, &frame(1, "c7")), vec![PARTICIPANT_LEFT]);
    assert_eq!(
        exchange(&addr, &frame(2, "c8")),
        vec![
            REJECTED, b'p', b'a', b'r', b't', b'i', b'c', b'i', b'p', b'a', b'n', b't', b'_', b'n',
            b'o', b't', b'_', b'j', b'o', b'i', b'n', b'e', b'd'
        ]
    );
    server.kill_and_wait();
}

fn assert_timeout() {
    let (mut server, addr) = ServerProc::spawn();
    assert_eq!(
        exchange(&addr, &[]),
        vec![
            REJECTED, b'e', b'x', b't', b'e', b'r', b'n', b'a', b'l', b'_', b'd', b'e', b'c', b'o',
            b'd', b'e', b'_', b'f', b'a', b'i', b'l', b'e', b'd'
        ]
    );
    server.kill_and_wait();
}
