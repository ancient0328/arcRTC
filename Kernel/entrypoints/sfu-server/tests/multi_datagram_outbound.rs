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
        // response datagram と stdout outcome の双方で同一 event の外部投影を観測します。
        let outcome = match self.stdout_rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(line)) => line,
            Ok(Err(error)) => panic!("stdout line must be readable: {error}"),
            Err(error) => panic!("stdout collection timed out: {error}"),
        };

        (response[..len].to_vec(), outcome)
    }

    fn establish_secure_media(&mut self) {
        let _ = self.exchange(b"ARCRTC-SFU/ICE/CONNECTED");
        let _ = self.exchange(b"ARCRTC-SFU/DTLS/ESTABLISHED");
        let _ = self.exchange(b"ARCRTC-SFU/SRTP/ACTIVE");
    }
}

impl Drop for SfuHarness {
    fn drop(&mut self) {
        self.server.kill_and_wait();
    }
}

fn digest(outcome: &str) -> u64 {
    outcome
        .split_whitespace()
        .find_map(|part| part.strip_prefix("digest="))
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_else(|| panic!("digest must be present in {outcome}"))
}

#[test]
fn assert_outbound_response_bytes() {
    let mut server = SfuHarness::start();
    server.establish_secure_media();
    let (response, outcome) = server.exchange(b"ARCRTC-SFU/RTP/FORWARD");

    assert_eq!(response, vec![0x23]);
    assert!(outcome.starts_with("outcome=ok secure=protected "));
}

#[test]
fn assert_client_recv() {
    let mut server = SfuHarness::start();
    server.establish_secure_media();
    let (first_response, _first_outcome) = server.exchange(b"ARCRTC-SFU/RTP/FORWARD");
    let (second_response, _second_outcome) = server.exchange(&[
        0x80, 0x60, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03,
    ]);

    assert_eq!(first_response, vec![0x23]);
    assert_eq!(second_response, vec![0x23]);
}

#[test]
fn assert_transmit_digest() {
    let mut server = SfuHarness::start();
    server.establish_secure_media();
    let (_first_response, first_outcome) = server.exchange(b"ARCRTC-SFU/RTP/FORWARD");
    let (_second_response, second_outcome) = server.exchange(b"ARCRTC-SFU/RTP/FORWARD");

    let first_digest = digest(&first_outcome);
    let second_digest = digest(&second_outcome);
    assert_ne!(first_digest, 0);
    assert_ne!(second_digest, 0);
    assert_ne!(first_digest, second_digest);
}
