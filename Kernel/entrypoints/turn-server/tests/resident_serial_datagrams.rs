use std::io::BufRead;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const BINDING_REQUEST: &[u8] = &[
    0x00, 0x01, 0x00, 0x00, 0x21, 0x12, 0xA4, 0x42, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0A, 0x0B,
];

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
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

    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn exchange(client: &UdpSocket, addr: &str) -> Vec<u8> {
    client
        .send_to(BINDING_REQUEST, addr)
        .expect("request datagram must be sent");
    let mut buf = [0u8; 2048];
    let (n, _peer) = client
        .recv_from(&mut buf)
        .expect("response datagram must be received");
    buf[..n].to_vec()
}

#[test]
fn same_process_handles_two_serial_datagrams() {
    let (mut server, addr) = ChildGuard::spawn();
    let client = UdpSocket::bind("127.0.0.1:0").expect("client socket must bind");
    client
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout must be set");

    let first = exchange(&client, &addr);
    let second = exchange(&client, &addr);
    println!("addr={addr} input={BINDING_REQUEST:?} first={first:?} second={second:?}");
    assert!(!first.is_empty());
    assert!(!second.is_empty());
    assert!(
        server.try_wait().expect("try_wait must succeed").is_none(),
        "server must still be resident after two datagrams"
    );
}
