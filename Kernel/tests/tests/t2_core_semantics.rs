use arcrtc_roadmap_tests::{
    assert_impl_file_contains, assert_impl_rust_source_set_contains, assert_not_contains,
    files_named, impl_relative, implementation_root, read_impl_rust_source_set,
};

#[test]
fn core_unit_asset_surfaces_exist_for_all_roadmap_groups() {
    let assets = [
        (
            "tests/core/vocabulary-protocol/CORE_VOCABULARY_PROTOCOL_UNIT_ASSET.md",
            "T2.1",
        ),
        ("tests/core/signaling/SIGNALING_UNIT_ASSET.md", "T2.2"),
        ("tests/core/sfu/SFU_UNIT_ASSET.md", "T2.3"),
        ("tests/core/turn/TURN_UNIT_ASSET.md", "T2.4"),
        (
            "tests/core/transport/TRANSPORT_SECURE_MEDIA_UNIT_ASSET.md",
            "T2.5",
        ),
        (
            "tests/core/operation-runtime-quality/OPERATION_RUNTIME_QUALITY_UNIT_ASSET.md",
            "T2.6",
        ),
    ];

    for (path, task) in assets {
        assert_impl_file_contains(
            path,
            &[
                task,
                "unit-test asset",
                "owned by core",
                "forbidden driver ownership",
            ],
        );
    }
}

#[test]
fn core_source_does_not_pull_driver_or_external_io_ownership() {
    let root = implementation_root().join("core");
    for source in files_named(&root, "lib.rs") {
        let relative = impl_relative(&source);
        let crate_dir = relative
            .strip_suffix("/src/lib.rs")
            .expect("core source path must be a crate lib.rs");
        let content = read_impl_rust_source_set(&format!("{crate_dir}/src"));
        let code = rust_code_without_line_comments(&content);
        assert_not_contains(
            crate_dir,
            &code,
            &[
                "arcrtc_driver",
                "arcrtc-driver",
                "std::net",
                "std::fs",
                "tokio::",
                "hyper::",
                "reqwest::",
                "sqlx::",
                "str0m",
            ],
        );
    }
}

fn rust_code_without_line_comments(content: &str) -> String {
    content
        .lines()
        // 境界説明コメントは設計意図の証跡であり、実依存の証跡ではないため検査対象から外します。
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn core_plane_surfaces_expose_closed_decision_or_failure_vocabulary() {
    for (path, markers) in [
        (
            "core/signaling/src",
            &[
                "SignalingCommandKind",
                "SignalingEventKind",
                "SignalingFailureKind",
                "WrongTargetSurface",
            ][..],
        ),
        (
            "core/sfu/src",
            &[
                "SfuDecisionKind",
                "SfuFailureKind",
                "BorrowedPacket",
                "WrongTargetSurface",
            ][..],
        ),
        (
            "core/turn/src",
            &[
                "TurnDecisionKind",
                "TurnFailureKind",
                "TurnRequestedLifetimeSeconds",
                "TurnContractError",
            ][..],
        ),
    ] {
        assert_impl_rust_source_set_contains(path, markers);
    }
}
