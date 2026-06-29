// arcrtc owner-layer: sdk
// arcrtc package-role: android signaling-only public sdk surface
// arcrtc allowed-dependency-direction: sdk must not depend on regulated or driver internals
// arcrtc dependency-class: sdk_public_dependency

plugins {
  id("com.android.library") version "8.13.2" apply false
  id("org.jetbrains.kotlin.android") version "2.3.21" apply false
}
