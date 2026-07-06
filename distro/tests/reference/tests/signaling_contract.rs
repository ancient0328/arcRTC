//! reference Signaling と Kernel contract builder の境界を検査します。

use std::{fs, path::PathBuf};

fn source() -> String {
    fs::read_to_string(root().join("reference-distro/signaling/src/kernel_contract.rs"))
        .expect("signaling kernel contract source must be readable")
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn signaling_builder_uses_fixed_kernel_constructor_chain() {
    let source = source();
    for expected in [
        "SignalingSubject::new(input.room_id.clone(), input.participant_id.clone())",
        "CommandEnvelope::new(",
        "SignalingCommandKind::JoinRoom => CommandType::new(\"reference.signaling.join_room\")",
        "SignalingCommandKind::LeaveRoom => CommandType::new(\"reference.signaling.leave_room\")",
        "SignalingCommandKind::SendOffer => CommandType::new(\"reference.signaling.send_offer\")",
        "SignalingCommandKind::SendAnswer => CommandType::new(\"reference.signaling.send_answer\")",
        "CommandType::new(\"reference.signaling.send_ice_candidate\")",
        "CommandType::new(\"reference.signaling.request_turn_credential\")",
        "CommandType::new(\"reference.signaling.acknowledge_forward\")",
        "CommandVersion::new(1)",
        "TargetSurface::Signaling",
        "SignalingCommand::new(",
    ] {
        assert!(source.contains(expected), "missing {expected}");
    }
}
