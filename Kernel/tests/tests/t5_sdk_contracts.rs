use arcrtc_roadmap_tests::{assert_impl_file_contains, assert_not_contains, read_impl};

#[test]
fn sdk_surfaces_remain_signaling_only_and_do_not_reference_driver_or_regulated_internals() {
    for (path, markers) in [
        (
            "sdk/typescript/package.json",
            &["ownerLayer", "sdk", "signaling-only"][..],
        ),
        (
            "sdk/android/build.gradle.kts",
            &["owner-layer: sdk", "signaling-only"][..],
        ),
        (
            "sdk/ios/Package.swift",
            &["owner-layer: sdk", "signaling-only"][..],
        ),
    ] {
        assert_impl_file_contains(path, markers);
        assert_not_contains(
            path,
            &read_impl(path),
            &["../drivers", "../../drivers", "arcrtc-driver"],
        );
    }
}
