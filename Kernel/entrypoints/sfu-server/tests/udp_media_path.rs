use std::io::BufRead;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
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

impl Drop for ServerProc {
    fn drop(&mut self) {
        self.kill_and_wait();
    }
}

fn run_sequence(label: &str, inputs: &[&[u8]]) -> Vec<String> {
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
    let mut server = ServerProc { child };

    let client = UdpSocket::bind("127.0.0.1:0").expect("client UDP socket must bind");
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in reader.lines() {
            let _ = tx.send(line);
        }
    });

    let mut outcomes = Vec::with_capacity(inputs.len());
    for input in inputs {
        client
            .send_to(input, &addr)
            .expect("input datagram must be sent");
        // outcome 読み取りは timeout で bounded にし、失敗時も guard で子 process を回収します。
        let outcome = match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(line)) => line,
            Ok(Err(error)) => panic!("stdout line must be readable: {error}"),
            Err(error) => {
                server.kill_and_wait();
                panic!("stdout collection timed out: {error}");
            }
        };
        assert!(
            outcome.starts_with("outcome="),
            "outcome line must be present, got {outcome}"
        );
        println!("case={label} input={input:?} outcome={outcome}");
        outcomes.push(outcome);
    }

    server.kill_and_wait();
    outcomes
}

fn run_case(label: &str, input: &[u8]) -> String {
    match run_sequence(label, &[input]).pop() {
        Some(outcome) => outcome,
        None => panic!("single outcome must be present"),
    }
}

fn run_secure_media_sequence(label: &str, input: &[u8]) -> String {
    let outcomes = run_sequence(
        label,
        &[
            b"ARCRTC-SFU/ICE/CONNECTED",
            b"ARCRTC-SFU/DTLS/ESTABLISHED",
            b"ARCRTC-SFU/SRTP/ACTIVE",
            input,
        ],
    );
    outcomes
        .last()
        .expect("final secure media outcome must be present")
        .to_owned()
}

fn parse_counter(line: &str, key: &str) -> usize {
    line.split_whitespace()
        .find_map(|part| {
            let value = part.strip_prefix(key)?;
            value.parse::<usize>().ok()
        })
        .unwrap_or_else(|| panic!("{key} counter must be present in {line}"))
}

#[test]
fn accepted_media_datagrams_emit_ok_outcome() {
    let cases: &[(&str, &[u8])] = &[
        (
            "MS1-rtp",
            &[
                0x80, 0x60, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02,
            ],
        ),
        ("MS1-dtls", &[0x16, 0x00, 0x00, 0x00]),
    ];

    for (label, input) in cases {
        let outcome = run_secure_media_sequence(label, input);
        assert!(
            outcome.starts_with("outcome=ok "),
            "{label} must be accepted, got {outcome}"
        );
        let transmits = parse_counter(&outcome, "transmits=");
        let dropped = parse_counter(&outcome, "dropped=");
        println!("case={label} transmits={transmits} dropped={dropped}");
    }
}

#[test]
fn adversarial_media_datagrams_fail_closed() {
    let zeroes = [0u8; 20];
    let cases: &[(&str, &[u8])] = &[
        ("MS2-empty", &[]),
        (
            "MS2-garbage",
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ),
        ("MS2-short-stun", &[0x00, 0x01, 0x02]),
        ("MS2-classification-gap", &[0x40, 0x40, 0x40, 0x40]),
        ("MS2-short-rtp", &[0x80, 0x60]),
        ("MS2-zero-stun", &zeroes),
    ];

    for (label, input) in cases {
        let outcome = run_case(label, input);
        assert_eq!(outcome, "outcome=rejected reason=external_decode_failed");
    }
}
