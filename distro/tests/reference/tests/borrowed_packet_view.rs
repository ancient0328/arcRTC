//! borrowed packet view の zero-copy 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn borrowed_packet_view_keeps_packet_and_payload_as_borrowed_slices() {
    let source =
        fs::read_to_string(root().join("reference-distro/sfu/src/kernel_contract.rs"))
            .expect("sfu kernel contract source must be readable");
    assert!(source.contains("raw_packet: &'packet [u8]"));
    assert!(source.contains("payload: &'packet [u8]"));
    assert!(source.contains("SfuPacketView::new("));
    assert!(!source.contains("raw_packet.to_vec()"));
    assert!(!source.contains("payload.to_vec()"));
}
