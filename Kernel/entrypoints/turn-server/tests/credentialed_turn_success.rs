#![allow(non_snake_case)]

use std::io::BufRead;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const ALLOCATION_SUCCESS: u8 = 0x00;
const REFRESH_SUCCESS: u8 = 0x01;
const PERMISSION_SUCCESS: u8 = 0x02;
const CHANNEL_BIND_SUCCESS: u8 = 0x03;
const RELAY_DATA_FORWARDING: u8 = 0x04;
const COOKIE: [u8; 4] = [0x21, 0x12, 0xA4, 0x42];
const TX: [u8; 12] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
];

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

fn exchange(client: &UdpSocket, addr: &str, input: &[u8]) -> Vec<u8> {
    client
        .send_to(input, addr)
        .expect("request datagram must be sent");
    let mut buf = [0u8; 2048];
    let (n, _peer) = client
        .recv_from(&mut buf)
        .expect("response datagram must be received");
    buf[..n].to_vec()
}

#[test]
fn t_turn_01_credentialed_success_path() {
    let (mut server, addr) = ServerProc::spawn();
    let client = UdpSocket::bind("127.0.0.1:0").expect("client socket must bind");
    client
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout must be set");

    assert_t_turn_01__allocate(&client, &addr);
    assert_t_turn_01__refresh(&client, &addr);
    assert_t_turn_01__create_permission(&client, &addr);
    assert_t_turn_01__channel_bind(&client, &addr);
    assert_t_turn_01__relay_data(&client, &addr);
    server.kill_and_wait();
}

fn assert_t_turn_01__allocate(client: &UdpSocket, addr: &str) {
    let response = exchange(
        client,
        addr,
        &datagram(0x0003, vec![attribute(0x0006, b"credential")]),
    );
    assert_eq!(response, vec![ALLOCATION_SUCCESS]);
}

fn assert_t_turn_01__refresh(client: &UdpSocket, addr: &str) {
    let response = exchange(
        client,
        addr,
        &datagram(0x0004, vec![attribute(0x0006, b"credential")]),
    );
    assert_eq!(response, vec![REFRESH_SUCCESS]);
}

fn assert_t_turn_01__create_permission(client: &UdpSocket, addr: &str) {
    let response = exchange(
        client,
        addr,
        &datagram(0x0008, vec![attribute(0x000c, b"peer")]),
    );
    assert_eq!(response, vec![PERMISSION_SUCCESS]);
}

fn assert_t_turn_01__channel_bind(client: &UdpSocket, addr: &str) {
    let response = exchange(
        client,
        addr,
        &datagram(
            0x0009,
            vec![attribute(0x000c, b"peer"), attribute(0x0012, b"channel")],
        ),
    );
    assert_eq!(response, vec![CHANNEL_BIND_SUCCESS]);
}

fn assert_t_turn_01__relay_data(client: &UdpSocket, addr: &str) {
    let response = exchange(
        client,
        addr,
        &datagram(
            0x0016,
            vec![attribute(0x000c, b"peer"), attribute(0x0013, b"packet")],
        ),
    );
    assert_eq!(response, vec![RELAY_DATA_FORWARDING]);
}
