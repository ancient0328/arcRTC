use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const FRAME: &[u8] = &[0x00, 0x00, 0x02, 0x72, 0x31, 0x63, 0x31];
const EXPECTED_RESPONSE: &[u8] = &[0x00];

struct ServerProc {
    child: Child,
}

impl ServerProc {
    fn spawn() -> (Self, String) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_arcrtc-signaling-server"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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

fn roundtrip(addr: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(addr).expect("client must connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
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

fn assert_stopped_endpoint_returns_no_bytes(addr: &str) {
    match TcpStream::connect(addr) {
        Err(error) => {
            println!("stopped_branch=connect_error error={error:?}");
        }
        Ok(mut stream) => {
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("read timeout must be set");
            if let Err(error) = stream.write_all(FRAME) {
                println!("stopped_branch=write_error error={error:?}");
                return;
            }
            let _ = stream.shutdown(std::net::Shutdown::Write);
            let mut response = Vec::new();
            let read_result = stream.read_to_end(&mut response);
            println!("stopped_branch=connected read_result={read_result:?} response={response:?}");
            assert!(response.is_empty());
        }
    }
}

#[test]
fn stopping_one_process_does_not_stop_the_other_process() {
    let (mut server_a, addr_a) = ServerProc::spawn();
    let (mut server_b, addr_b) = ServerProc::spawn();

    let before_a = roundtrip(&addr_a);
    let before_b = roundtrip(&addr_b);
    println!("addr_a={addr_a} before_a={before_a:?}");
    println!("addr_b={addr_b} before_b={before_b:?}");
    assert_eq!(before_a, EXPECTED_RESPONSE);
    assert_eq!(before_b, EXPECTED_RESPONSE);

    server_a.kill_and_wait();

    let after_b = roundtrip(&addr_b);
    println!("after_b={after_b:?}");
    assert_eq!(after_b, EXPECTED_RESPONSE);
    assert_stopped_endpoint_returns_no_bytes(&addr_a);

    server_b.kill_and_wait();
}
