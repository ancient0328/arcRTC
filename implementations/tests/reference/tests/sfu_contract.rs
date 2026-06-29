//! reference SFU と Kernel contract builder の境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

#[test]
fn sfu_builder_uses_kernel_contract_item_and_borrowed_packet_view() {
    let source =
        fs::read_to_string(root().join("reference-implementation/sfu/src/kernel_contract.rs"))
            .expect("sfu kernel contract source must be readable");
    assert!(
        source.contains("SfuContractItem::new(input.model_kind, input.references, input.payload)")
    );
    assert!(source.contains("SfuPacketView::new("));
    assert!(source.contains("raw_packet: &'packet [u8]"));
    assert!(source.contains("payload: &'packet [u8]"));
}
