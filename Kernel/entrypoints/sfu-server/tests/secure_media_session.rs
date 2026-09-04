use std::io::BufRead;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

struct ServerProc {
    child: Child,
}

impl ServerProc {
    fn kill_and_wait(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct SfuHarness {
    server: ServerProc,
    client: UdpSocket,
    addr: String,
    stdout_rx: Receiver<std::io::Result<String>>,
}

impl SfuHarness {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_arcrtc-sfu-server"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("sfu server must spawn");
        let stdout = child.stdout.take().expect("stdout must be piped");
        let mut reader = std::io::BufReader::new(stdout);
        let mut first_line = String::new();
        reader
            .read_line(&mut first_line)
            .expect("listening line must be readable");
        let addr = first_line
            .strip_prefix("listening=")
            .expect("first stdout line must expose listening address")
            .trim()
            .to_owned();
        let client = UdpSocket::bind("127.0.0.1:0").expect("client UDP socket must bind");
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("client UDP timeout must be set");
        let (stdout_tx, stdout_rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in reader.lines() {
                let _ = stdout_tx.send(line);
            }
        });

        Self {
            server: ServerProc { child },
            client,
            addr,
            stdout_rx,
        }
    }

    fn exchange(&mut self, input: &[u8]) -> (Vec<u8>, String) {
        self.client
            .send_to(input, &self.addr)
            .expect("input datagram must be sent");
        let mut response = [0u8; 256];
        let (len, _peer) = self
            .client
            .recv_from(&mut response)
            .expect("response datagram must be received");
        // stdout は response と同じ runtime event の外部観測なので、timeout で bounded に読む。
        let outcome = match self.stdout_rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(line)) => line,
            Ok(Err(error)) => panic!("stdout line must be readable: {error}"),
            Err(error) => panic!("stdout collection timed out: {error}"),
        };

        (response[..len].to_vec(), outcome)
    }
}

impl Drop for SfuHarness {
    fn drop(&mut self) {
        self.server.kill_and_wait();
    }
}

#[test]
fn assert_ice() {
    let mut server = SfuHarness::start();
    let (response, outcome) = server.exchange(b"ARCRTC-SFU/ICE/CONNECTED");

    assert_eq!(response, vec![0x20]);
    assert!(outcome.starts_with("outcome=ok secure=ice_connected "));
}

#[test]
fn assert_dtls() {
    let mut server = SfuHarness::start();
    let _ = server.exchange(b"ARCRTC-SFU/ICE/CONNECTED");
    let (response, outcome) = server.exchange(b"ARCRTC-SFU/DTLS/ESTABLISHED");

    assert_eq!(response, vec![0x21]);
    assert!(outcome.starts_with("outcome=ok secure=dtls_established "));
}

#[test]
fn assert_srtp_protection() {
    let mut server = SfuHarness::start();
    let _ = server.exchange(b"ARCRTC-SFU/ICE/CONNECTED");
    let _ = server.exchange(b"ARCRTC-SFU/DTLS/ESTABLISHED");
    let (response, outcome) = server.exchange(b"ARCRTC-SFU/SRTP/ACTIVE");

    assert_eq!(response, vec![0x22]);
    assert!(outcome.starts_with("outcome=ok secure=srtp_active "));
}

#[test]
fn assert_secure_media_session_decision() {
    let mut server = SfuHarness::start();
    let _ = server.exchange(b"ARCRTC-SFU/ICE/CONNECTED");
    let _ = server.exchange(b"ARCRTC-SFU/DTLS/ESTABLISHED");
    let _ = server.exchange(b"ARCRTC-SFU/SRTP/ACTIVE");
    let (response, outcome) = server.exchange(b"ARCRTC-SFU/RTP/FORWARD");

    assert_eq!(response, vec![0x23]);
    assert!(outcome.starts_with("outcome=ok secure=protected "));
    assert!(outcome.contains("transmits=1"));
    assert!(outcome.contains("dropped=0"));
}

#[test]
fn assert_fail_closed_without_secure_media() {
    let mut server = SfuHarness::start();
    let (response, outcome) = server.exchange(b"ARCRTC-SFU/RTP/FORWARD");

    assert_eq!(response[0], 0x15);
    assert_eq!(outcome, "outcome=rejected reason=command_order_violation");
}
