use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self { child }
    }

    fn wait(mut self) -> std::process::ExitStatus {
        self.child.wait().expect("live server child should exit")
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn live_public_probe_server_command_plan_covers_probe_surface() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_arcrtc-entrypoint-live-server"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("live server binary should spawn");

    let stdout = child
        .stdout
        .take()
        .expect("live server stdout should be piped");
    let mut stdout = BufReader::new(stdout);
    let mut line = String::new();
    stdout
        .read_line(&mut line)
        .expect("live server should announce base URL");

    let base_addr = assert_live_public_base_url_stdout_line(&line);
    let child = ChildGuard::new(child);

    let health = http_get(&base_addr, "/health");
    assert_health_route(&health);

    let readiness = http_get(&base_addr, "/ready");
    assert_ready_route(&readiness);

    let invalid = http_get(&base_addr, "/not-a-probe");
    assert_invalid_route_rejection(&invalid);

    let shutdown = http_get(&base_addr, "/__arcrtc_shutdown");
    assert_shutdown_request(&shutdown);

    // shutdown request は process surface の終了観測だけで、readiness 証明にはしません。
    let status = child.wait();
    assert!(!status.success());
}

fn assert_live_public_base_url_stdout_line(line: &str) -> String {
    let line = line.trim();
    let base_url = line
        .strip_prefix("LIVE_PUBLIC_BASE_URL=http://")
        .expect("stdout must expose LIVE_PUBLIC_BASE_URL");
    assert!(
        base_url.starts_with("127.0.0.1:"),
        "base URL should bind to localhost with an assigned port"
    );
    base_url.to_string()
}

fn assert_health_route(response: &str) {
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("x-arcrtc-body-digest:"));
    assert!(response.ends_with("arcrtc-health-ok"));
}

fn assert_ready_route(response: &str) {
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("x-arcrtc-body-digest:"));
    assert!(response.ends_with("arcrtc-ready-ok"));
}

fn assert_invalid_route_rejection(response: &str) {
    assert!(response.starts_with("HTTP/1.1 404 OK"));
    assert!(response.ends_with("arcrtc-probe-not-found"));
}

fn assert_shutdown_request(response: &str) {
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.ends_with("arcrtc-shutdown"));
}

fn http_get(base_addr: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(base_addr).expect("live server should accept TCP");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("read timeout should be set");
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .expect("write timeout should be set");

    let request = format!("GET {path} HTTP/1.1\r\nhost: {base_addr}\r\nconnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("request should be written");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("response should be readable");
    response
}
