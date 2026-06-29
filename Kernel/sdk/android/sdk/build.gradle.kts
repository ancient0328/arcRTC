import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.gradle.testing.jacoco.tasks.JacocoReport

// arcrtc owner-layer: sdk
// arcrtc package-role: android signaling-only public sdk module
// arcrtc allowed-dependency-direction: sdk must not depend on regulated or driver internals
// arcrtc dependency-class: sdk_public_dependency

plugins {
  id("com.android.library")
  id("org.jetbrains.kotlin.android")
  jacoco
}

android {
  namespace = "dev.arcrtc.sdk"
  compileSdk = 36

  defaultConfig {
    minSdk = 23
  }

  buildTypes {
    debug {
      enableUnitTestCoverage = true
    }
  }
}

kotlin {
  compilerOptions {
    jvmTarget.set(JvmTarget.JVM_1_8)
  }
}

dependencies {
  testImplementation(kotlin("test"))
}

tasks.register<JacocoReport>("jacocoDebugUnitTestReport") {
  group = "verification"
  description = "Generates line, branch, and method coverage for the Android SDK debug unit tests."
  dependsOn("testDebugUnitTest")

  val excludedClassFiles = listOf(
    "**/R.class",
    "**/R$*.class",
    "**/BuildConfig.*",
    "**/Manifest*.*",
  )

  classDirectories.setFrom(
    fileTree(layout.buildDirectory.dir("tmp/kotlin-classes/debug")) {
      exclude(excludedClassFiles)
    },
  )
  sourceDirectories.setFrom(files("src/main/kotlin", "src/main/java"))
  executionData.setFrom(
    fileTree(layout.buildDirectory) {
      include(
        "outputs/unit_test_code_coverage/debugUnitTest/testDebugUnitTest.exec",
        "jacoco/testDebugUnitTest.exec",
      )
    },
  )

  reports {
    xml.required.set(true)
    csv.required.set(true)
    html.required.set(true)
    xml.outputLocation.set(layout.buildDirectory.file("reports/jacoco/debugUnitTest/jacoco.xml"))
    csv.outputLocation.set(layout.buildDirectory.file("reports/jacoco/debugUnitTest/jacoco.csv"))
    html.outputLocation.set(layout.buildDirectory.dir("reports/jacoco/debugUnitTest/html"))
  }
}
