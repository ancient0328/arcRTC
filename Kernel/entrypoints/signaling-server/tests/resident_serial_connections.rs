use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const FRAME: &[u8] = &[0x00, 0x00, 0x02, 0x72, 0x31, 0x63, 0x31];

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
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

fn exchange(addr: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(addr).expect("client must connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout must be set");
    stream.write_all(FRAME).expect("frame must be written");
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("write side must close");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("response must be readable");
    response
}

#[test]
fn same_process_handles_two_serial_connections() {
    let (mut server, addr) = ChildGuard::spawn();
    let first = exchange(&addr);
    let second = exchange(&addr);
    println!("addr={addr} first={first:?} second={second:?}");
    assert!(!first.is_empty());
    assert!(!second.is_empty());
    assert_eq!(first, second);
    assert!(
        server.try_wait().expect("try_wait must succeed").is_none(),
        "server must still be resident after two connections"
    );
}
