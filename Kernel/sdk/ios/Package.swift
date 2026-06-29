// swift-tools-version: 5.9
// arcrtc owner-layer: sdk
// arcrtc package-role: ios signaling-only public sdk surface
// arcrtc allowed-dependency-direction: sdk must not depend on regulated or driver internals
// arcrtc dependency-class: sdk_public_dependency

import PackageDescription

let package = Package(
    name: "ArcRtcSdkIos",
    platforms: [
        .iOS(.v15)
    ],
    products: [
        .library(name: "ArcRtcSdkIos", targets: ["ArcRtcSdkIos"])
    ],
    targets: [
        .target(name: "ArcRtcSdkIos"),
        .testTarget(
            name: "ArcRtcSdkIosTests",
            dependencies: ["ArcRtcSdkIos"]
        )
    ]
)
