// arcrtc owner-layer: sdk
// arcrtc package-role: android signaling-only public sdk surface
// arcrtc allowed-dependency-direction: sdk must not depend on regulated or driver internals
// arcrtc dependency-class: sdk_public_dependency

pluginManagement {
  repositories {
    google()
    mavenCentral()
    gradlePluginPortal()
  }
}

dependencyResolutionManagement {
  repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
  repositories {
    google()
    mavenCentral()
  }
}

rootProject.name = "arcrtc-sdk-android"
include(":sdk")
