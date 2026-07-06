use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const SENTINEL: &str = "ARCRTC-P2-SENTINEL-0001";

struct ServerProc {
    child: Child,
    stdout_prefix: Vec<u8>,
    stdout: std::io::BufReader<std::process::ChildStdout>,
    stderr: std::process::ChildStderr,
}

impl ServerProc {
    fn spawn() -> (Self, String) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_arcrtc-signaling-server"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("signaling server must spawn");
        let stdout = child.stdout.take().expect("stdout must be piped");
        let stderr = child.stderr.take().expect("stderr must be piped");
        let mut stdout = std::io::BufReader::new(stdout);
        let mut line = String::new();
        stdout
            .read_line(&mut line)
            .expect("listening line must be readable");
        let addr = line
            .strip_prefix("listening=")
            .expect("first stdout line must expose listening address")
            .trim()
            .to_owned();
        (
            Self {
                child,
                stdout_prefix: line.into_bytes(),
                stdout,
                stderr,
            },
            addr,
        )
    }

    fn kill_wait_and_collect(mut self) -> (Vec<u8>, Vec<u8>) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let mut stdout = std::mem::take(&mut self.stdout_prefix);
        let mut stderr = Vec::new();
        self.stdout
            .read_to_end(&mut stdout)
            .expect("stdout must be collectable");
        self.stderr
            .read_to_end(&mut stderr)
            .expect("stderr must be collectable");
        (stdout, stderr)
    }
}

impl Drop for ServerProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn sentinel_frame() -> Vec<u8> {
    let value = SENTINEL.as_bytes();
    let len = u16::try_from(value.len()).expect("sentinel length must fit u16");
    let mut frame = Vec::with_capacity(3 + value.len() * 2);
    frame.push(0x00);
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(value);
    frame.extend_from_slice(value);
    frame
}

#[test]
fn sentinel_bytes_do_not_appear_on_socket_or_process_output() {
    let frame = sentinel_frame();
    let (server, addr) = ServerProc::spawn();
    let mut stream = TcpStream::connect(&addr).expect("client must connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout must be set");
    stream.write_all(&frame).expect("frame must be written");
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("write side must close");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("response must be readable");
    assert_eq!(response.first().copied(), Some(0x01));
    assert_eq!(
        std::str::from_utf8(&response[1..]).expect("reason must be UTF-8"),
        "token_verification_failed"
    );

    let (stdout, stderr) = server.kill_wait_and_collect();
    println!(
        "addr={addr} frame={frame:?} response={response:?} stdout_len={} stderr_len={}",
        stdout.len(),
        stderr.len()
    );
    let sentinel = SENTINEL.as_bytes();
    assert!(!response
        .windows(sentinel.len())
        .any(|window| window == sentinel));
    assert!(!stdout
        .windows(sentinel.len())
        .any(|window| window == sentinel));
    assert!(!stderr
        .windows(sentinel.len())
        .any(|window| window == sentinel));
}
