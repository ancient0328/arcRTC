//! reference TURN と Kernel contract builder の境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

#[test]
fn turn_builder_delegates_to_kernel_try_new_and_maps_failure() {
    let source =
        fs::read_to_string(root().join("reference-implementation/turn/src/kernel_contract.rs"))
            .expect("turn kernel contract source must be readable");
    assert!(source.contains("TurnCommand::try_new("));
    assert!(source.contains(".map_err(|_| ReferenceTurnError::KernelContractMismatch)"));
}
