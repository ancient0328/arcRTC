//! Kernel-owned live public probe server entrypoint.
//!
//! この crate は live/operational evidence の raw material を得るための
//! process I/O surface だけを持ち、domain admission や readiness 証明は所有しません。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use arcrtc_core_reason::CatalogedReasonRef;

/// live public probe server の bind / route 設定です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LivePublicProbeServerConfig {
    /// bind する host 文字列です。
    pub bind_host: String,
    /// bind する port です。`0` の場合、OS が空き port を割り当てます。
    pub bind_port: u16,
    /// health probe の path です。
    pub health_path: String,
    /// readiness probe の path です。
    pub readiness_path: String,
}

impl LivePublicProbeServerConfig {
    /// local probe 用の既定設定を作ります。
    #[must_use]
    pub fn local_default() -> Self {
        Self {
            bind_host: "127.0.0.1".to_string(),
            bind_port: 0,
            health_path: "/health".to_string(),
            readiness_path: "/ready".to_string(),
        }
    }
}

/// public probe server が扱う route の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LivePublicProbeRoute {
    /// health probe route です。
    Health,
    /// readiness probe route です。
    Readiness,
}

/// probe response の観測値です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LivePublicProbeResponse {
    /// 応答した route です。
    pub route: LivePublicProbeRoute,
    /// HTTP status code です。
    pub status_code: u16,
    /// body の安定 digest です。
    pub body_digest: String,
}

/// live public base URL の stdout announcement です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LivePublicBaseUrlAnnouncement {
    /// `LIVE_PUBLIC_BASE_URL=<url>` へ埋め込む base URL です。
    pub base_url: String,
}

/// live public probe server の composition / wiring marker です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LivePublicProbeWiring {
    failure_reason: CatalogedReasonRef,
}

impl LivePublicProbeWiring {
    /// entrypoint failure を core-owned reason catalog へ接続します。
    #[must_use]
    pub fn from_failure_kind(kind: LivePublicProbeFailureKind) -> Self {
        let failure_reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("live public probe failure reason code must be registered");
        Self { failure_reason }
    }

    /// 接続済みの cataloged reason です。
    pub const fn failure_reason(self) -> CatalogedReasonRef {
        self.failure_reason
    }
}

/// live public probe server の失敗理由の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LivePublicProbeFailureKind {
    /// bind に失敗しました。
    BindRejected,
    /// probe path が entrypoint surface として不正です。
    InvalidProbePath,
    /// socket への response write に失敗しました。
    WriteFailed,
    /// shutdown request を受け取りました。
    ShutdownRequested,
}

impl LivePublicProbeFailureKind {
    /// core-owned closed reason catalog の reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::BindRejected | Self::WriteFailed => "health_probe_unavailable",
            Self::InvalidProbePath => "runtime_config_invalid",
            Self::ShutdownRequested => "readiness_not_satisfied",
        }
    }
}

/// live public probe server を起動します。
///
/// stdout に `LIVE_PUBLIC_BASE_URL=<url>` を 1 行出力し、health / readiness
/// probe bytes を std TCP listener で返します。
pub fn serve_live_public_probe_server(
    config: LivePublicProbeServerConfig,
) -> Result<(), LivePublicProbeFailureKind> {
    validate_probe_config(&config)?;

    let bind_addr = format!("{}:{}", config.bind_host, config.bind_port);
    let listener =
        TcpListener::bind(bind_addr).map_err(|_| LivePublicProbeFailureKind::BindRejected)?;
    let local_addr = listener
        .local_addr()
        .map_err(|_| LivePublicProbeFailureKind::BindRejected)?;
    let announcement = LivePublicBaseUrlAnnouncement {
        base_url: format!("http://{}", local_addr),
    };
    println!("LIVE_PUBLIC_BASE_URL={}", announcement.base_url);

    for stream in listener.incoming() {
        let mut stream = stream.map_err(|_| LivePublicProbeFailureKind::WriteFailed)?;
        match serve_one_connection(&mut stream, &config)? {
            ConnectionDisposition::Continue => {}
            ConnectionDisposition::Shutdown => {
                return Err(LivePublicProbeFailureKind::ShutdownRequested);
            }
        }
    }

    Ok(())
}

fn validate_probe_config(
    config: &LivePublicProbeServerConfig,
) -> Result<(), LivePublicProbeFailureKind> {
    if !is_valid_probe_path(&config.health_path) || !is_valid_probe_path(&config.readiness_path) {
        return Err(LivePublicProbeFailureKind::InvalidProbePath);
    }
    if config.health_path == config.readiness_path {
        return Err(LivePublicProbeFailureKind::InvalidProbePath);
    }
    Ok(())
}

fn is_valid_probe_path(path: &str) -> bool {
    path.starts_with('/') && path.len() > 1 && !path.contains(' ')
}

enum ConnectionDisposition {
    Continue,
    Shutdown,
}

fn serve_one_connection(
    stream: &mut TcpStream,
    config: &LivePublicProbeServerConfig,
) -> Result<ConnectionDisposition, LivePublicProbeFailureKind> {
    let mut request = [0_u8; 1024];
    let read_len = stream
        .read(&mut request)
        .map_err(|_| LivePublicProbeFailureKind::WriteFailed)?;
    let path = request_path(&request[..read_len]).unwrap_or("/");

    if path == config.health_path {
        let response = build_probe_response(LivePublicProbeRoute::Health, b"arcrtc-health-ok");
        write_http_response(stream, 200, "arcrtc-health-ok", &response)?;
        return Ok(ConnectionDisposition::Continue);
    }

    if path == config.readiness_path {
        let response = build_probe_response(LivePublicProbeRoute::Readiness, b"arcrtc-ready-ok");
        write_http_response(stream, 200, "arcrtc-ready-ok", &response)?;
        return Ok(ConnectionDisposition::Continue);
    }

    if path == "/__arcrtc_shutdown" {
        write_http_response_without_probe(stream, 200, "arcrtc-shutdown")?;
        return Ok(ConnectionDisposition::Shutdown);
    }

    write_http_response_without_probe(stream, 404, "arcrtc-probe-not-found")?;
    Ok(ConnectionDisposition::Continue)
}

fn request_path(request: &[u8]) -> Option<&str> {
    let request_text = std::str::from_utf8(request).ok()?;
    let first_line = request_text.lines().next()?;
    let mut parts = first_line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    if method == "GET" {
        Some(path)
    } else {
        None
    }
}

fn build_probe_response(route: LivePublicProbeRoute, body: &[u8]) -> LivePublicProbeResponse {
    LivePublicProbeResponse {
        route,
        status_code: 200,
        body_digest: stable_body_digest(body),
    }
}

fn stable_body_digest(body: &[u8]) -> String {
    // evidence raw material で比較しやすいよう、外部 crate に依存しない安定 digest を作ります。
    let digest = body
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |accumulator, byte| {
            accumulator.wrapping_mul(0x0000_0100_0000_01b3) ^ u64::from(*byte)
        });
    format!("{digest:016x}")
}

fn write_http_response(
    stream: &mut TcpStream,
    status_code: u16,
    body: &str,
    response: &LivePublicProbeResponse,
) -> Result<(), LivePublicProbeFailureKind> {
    let header = format!(
        "HTTP/1.1 {status_code} OK\r\ncontent-type: text/plain\r\ncontent-length: {}\r\nx-arcrtc-body-digest: {}\r\nconnection: close\r\n\r\n",
        body.len(),
        response.body_digest
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body.as_bytes()))
        .map_err(|_| LivePublicProbeFailureKind::WriteFailed)
}

fn write_http_response_without_probe(
    stream: &mut TcpStream,
    status_code: u16,
    body: &str,
) -> Result<(), LivePublicProbeFailureKind> {
    let header = format!(
        "HTTP/1.1 {status_code} OK\r\ncontent-type: text/plain\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body.as_bytes()))
        .map_err(|_| LivePublicProbeFailureKind::WriteFailed)
}
