use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const JOIN_FRAME: &[u8] = &[0x00, 0x00, 0x02, 0x72, 0x31, 0x63, 0x31];
const JOINED: u8 = 0x00;

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

#[test]
fn accepted_join_socket_roundtrip() {
    assert_accepted_join();
}

fn assert_accepted_join() {
    let (mut server, addr) = ServerProc::spawn();
    let response = exchange(&addr, JOIN_FRAME);

    assert_eq!(
        response.first().copied(),
        Some(JOINED),
        "well-formed join must produce Joined frame"
    );
    assert_eq!(response.len(), 1, "accepted join carries no reason payload");
    server.kill_and_wait();
}
