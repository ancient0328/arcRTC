//! entrypoints/composition-root は resident server loop の composition root です。
//!
//! ここでは published contract/ref を接続するだけで、domain rule、protocol semantics、
//! reason vocabulary、policy、ledger write は定義しません。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};

use arcrtc_core_sfu::{apply_resident_sfu_input, ResidentSfuOutcome, ResidentSfuState};
use arcrtc_core_signaling::{apply_resident_signaling_command, ResidentSignalingState};
use arcrtc_core_turn::{apply_resident_turn_command, ResidentTurnState};
use arcrtc_driver_network::{
    build_signaling_command, decode_sfu_media_datagram, decode_signaling_command_frame,
    decode_turn_datagram, encode_resident_turn_success_datagram, encode_sfu_error_datagram,
    encode_sfu_success_datagram, encode_signaling_event_frame, encode_signaling_rejection_frame,
    encode_turn_error_datagram,
};

pub use arcrtc_entrypoint_admin::ShutdownDrainObservationRef;
pub use arcrtc_entrypoint_configuration::RuntimeProfileObservationRef;

/// resident server kind の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentServerKind {
    /// Signaling server.
    Signaling,
    /// TURN server.
    Turn,
    /// SFU server.
    Sfu,
}

/// resident loop が接続する driver binding reference の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentDriverBindingRef {
    /// network driver binding.
    Network,
    /// TURN relay driver binding.
    TurnRelay,
    /// SFU transport driver binding.
    SfuTransport,
    /// persistence driver binding.
    Persistence,
    /// observability driver binding.
    Observability,
    /// security driver binding.
    Security,
}

/// resident loop construction failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentLoopFailureKind {
    /// required config/ref is missing.
    ConfigMissing,
    /// required driver binding ref is missing.
    DriverBindingMissing,
    /// runtime start was rejected by published observation boundary.
    RuntimeStartRejected,
    /// shutdown drain observation rejected resident start.
    ShutdownDrainRejected,
    /// supervision observation is missing or failed.
    SupervisionObservationFailed,
}

/// resident server loop config です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidentServerLoopConfig {
    server_kind: ResidentServerKind,
    runtime_profile_ref: RuntimeProfileObservationRef,
    driver_binding_refs: Vec<ResidentDriverBindingRef>,
    shutdown_ref: ShutdownDrainObservationRef,
    supervision_ref: ShutdownDrainObservationRef,
}

impl ResidentServerLoopConfig {
    /// resident server loop に必要な refs を束ねます。
    pub fn new(
        server_kind: ResidentServerKind,
        runtime_profile_ref: RuntimeProfileObservationRef,
        driver_binding_refs: Vec<ResidentDriverBindingRef>,
        shutdown_ref: ShutdownDrainObservationRef,
        supervision_ref: ShutdownDrainObservationRef,
    ) -> Self {
        Self {
            server_kind,
            runtime_profile_ref,
            driver_binding_refs,
            shutdown_ref,
            supervision_ref,
        }
    }

    /// server kind です。
    pub const fn server_kind(&self) -> ResidentServerKind {
        self.server_kind
    }

    /// driver binding refs です。
    pub fn driver_binding_refs(&self) -> &[ResidentDriverBindingRef] {
        &self.driver_binding_refs
    }
}

/// resident loop delegation observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResidentLoopObservation {
    server_kind: ResidentServerKind,
}

impl ResidentLoopObservation {
    /// resident loop delegation observation を作ります。
    pub const fn new(server_kind: ResidentServerKind) -> Self {
        Self { server_kind }
    }

    /// server kind です。
    pub const fn server_kind(&self) -> ResidentServerKind {
        self.server_kind
    }
}

/// resident loop の published ref 接続を行います。
///
/// 実 runtime start や driver policy selection は行わず、composition root の config/ref 境界だけを検査します。
pub fn run_resident_loop(
    config: ResidentServerLoopConfig,
) -> Result<ResidentLoopObservation, ResidentLoopFailureKind> {
    if config.runtime_profile_ref.as_str().is_empty() {
        return Err(ResidentLoopFailureKind::ConfigMissing);
    }
    if config.driver_binding_refs.is_empty() {
        return Err(ResidentLoopFailureKind::DriverBindingMissing);
    }
    if config.shutdown_ref.as_str().is_empty() {
        return Err(ResidentLoopFailureKind::ShutdownDrainRejected);
    }
    if config.supervision_ref.as_str().is_empty() {
        return Err(ResidentLoopFailureKind::SupervisionObservationFailed);
    }

    Ok(ResidentLoopObservation::new(config.server_kind))
}

/// resident server process を起動します。
///
/// socket I/O は entrypoint composition-root の process boundary として扱い、
/// frame decode/encode は drivers/network、状態判断は core contracts へ委譲します。
pub fn serve_resident_loop(
    config: ResidentServerLoopConfig,
) -> Result<(), ResidentLoopFailureKind> {
    let observation = run_resident_loop(config)?;

    match observation.server_kind() {
        ResidentServerKind::Signaling => serve_signaling_tcp_loop(),
        ResidentServerKind::Turn => serve_turn_udp_loop(),
        ResidentServerKind::Sfu => serve_sfu_udp_loop(),
    }
}

fn serve_signaling_tcp_loop() -> Result<(), ResidentLoopFailureKind> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    let local_addr = listener
        .local_addr()
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    println!("listening={local_addr}");
    std::io::stdout()
        .flush()
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;

    let mut signaling_state = ResidentSignalingState::default();
    for stream in listener.incoming() {
        let stream = stream.map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
        handle_signaling_connection(stream, &mut signaling_state)?;
    }

    Err(ResidentLoopFailureKind::RuntimeStartRejected)
}

fn handle_signaling_connection(
    mut stream: TcpStream,
    state: &mut ResidentSignalingState,
) -> Result<(), ResidentLoopFailureKind> {
    let mut input = Vec::new();
    stream
        .read_to_end(&mut input)
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    let response = resident_signaling_response(&input, state);
    stream
        .write_all(&response)
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)
}

fn resident_signaling_response(input: &[u8], state: &mut ResidentSignalingState) -> Vec<u8> {
    let decoded = match decode_signaling_command_frame(input) {
        Ok(decoded) => decoded,
        Err(failure) => return encode_signaling_rejection_frame(failure.kind()),
    };
    let command = match build_signaling_command(decoded) {
        Ok(command) => command,
        Err(failure) => return encode_signaling_rejection_frame(failure.kind()),
    };
    let response = apply_resident_signaling_command(&command, state);

    encode_signaling_event_frame(response.event(), response.reason_code())
}

fn serve_turn_udp_loop() -> Result<(), ResidentLoopFailureKind> {
    let socket = UdpSocket::bind("127.0.0.1:0")
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    let local_addr = socket
        .local_addr()
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    println!("listening={local_addr}");
    std::io::stdout()
        .flush()
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;

    let mut state = ResidentTurnState::default();
    let mut input = [0u8; 4096];
    loop {
        let (len, peer) = socket
            .recv_from(&mut input)
            .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
        let response = resident_turn_response(&input[..len], &mut state);
        socket
            .send_to(&response, peer)
            .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    }
}

fn resident_turn_response(input: &[u8], state: &mut ResidentTurnState) -> Vec<u8> {
    let has_success_material = input.len() > 20;
    let command = match decode_turn_datagram(input).into_core_turn_command() {
        Ok(command) => command,
        Err(failure) => return encode_turn_error_datagram(failure.kind().reason_code()),
    };

    match apply_resident_turn_command(&command, has_success_material, state) {
        Ok(success) => encode_resident_turn_success_datagram(success),
        Err(failure) => encode_turn_error_datagram(failure.reason_code()),
    }
}

fn serve_sfu_udp_loop() -> Result<(), ResidentLoopFailureKind> {
    let socket = UdpSocket::bind("127.0.0.1:0")
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    let local_addr = socket
        .local_addr()
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    println!("listening={local_addr}");
    std::io::stdout()
        .flush()
        .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;

    let mut state = ResidentSfuState::default();
    let mut input = [0u8; 4096];
    loop {
        let (len, peer) = socket
            .recv_from(&mut input)
            .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
        let response = resident_sfu_response(&input[..len], &mut state);
        socket
            .send_to(&response.datagram, peer)
            .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
        println!("{}", response.outcome_line);
        std::io::stdout()
            .flush()
            .map_err(|_error| ResidentLoopFailureKind::RuntimeStartRejected)?;
    }
}

struct ResidentSfuResponse {
    datagram: Vec<u8>,
    outcome_line: String,
}

fn resident_sfu_response(input: &[u8], state: &mut ResidentSfuState) -> ResidentSfuResponse {
    let decoded = match decode_sfu_media_datagram(input) {
        Ok(decoded) => decoded,
        Err(failure) => return rejected_sfu_response(failure.reason_code()),
    };
    let outcome = apply_resident_sfu_input(decoded.into_resident_sfu_input(), state);
    resident_sfu_outcome_response(outcome)
}

fn resident_sfu_outcome_response(outcome: ResidentSfuOutcome) -> ResidentSfuResponse {
    match outcome {
        ResidentSfuOutcome::Accepted {
            success_kind,
            detail_code,
            transmits,
            dropped,
            digest,
        } => ResidentSfuResponse {
            datagram: encode_sfu_success_datagram(success_kind),
            outcome_line: format!(
                "outcome=ok {detail_code} transmits={transmits} dropped={dropped} digest={digest}"
            ),
        },
        ResidentSfuOutcome::Rejected { reason_code } => rejected_sfu_response(reason_code),
    }
}

fn rejected_sfu_response(reason_code: &str) -> ResidentSfuResponse {
    ResidentSfuResponse {
        datagram: encode_sfu_error_datagram(reason_code),
        outcome_line: format!("outcome=rejected reason={reason_code}"),
    }
}
