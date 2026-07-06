//! real-device platform command の実行結果を閉じた型へ変換する境界です。
#![allow(dead_code)]

use arcrtc_distro_evidence::DistroEvidenceReason;

/// wrapper が採用する real-device command exit の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealDeviceCommandExitStatus {
    /// 必須 device class を観測できた状態です。
    Success,
    /// command は実行できたが、必須 device class を観測できない状態です。
    RequiredDeviceNotObserved,
    /// platform command が実行不能または失敗した状態です。
    PlatformCommandUnavailable,
    /// evidence field を構築できない状態です。
    EvidenceFieldsIncomplete,
    /// wrapper command scope が一致しない状態です。
    CommandScopeMismatch,
}

impl RealDeviceCommandExitStatus {
    /// wrapper process exit code へ変換します。
    pub const fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::RequiredDeviceNotObserved => 2,
            Self::PlatformCommandUnavailable => 3,
            Self::EvidenceFieldsIncomplete => 4,
            Self::CommandScopeMismatch => 5,
        }
    }

    /// evidence reason へ変換します。
    pub const fn distro_reason(self) -> DistroEvidenceReason {
        match self {
            Self::Success => DistroEvidenceReason::DistroOk,
            Self::RequiredDeviceNotObserved => {
                DistroEvidenceReason::RealDeviceScopeMismatch
            }
            Self::PlatformCommandUnavailable => DistroEvidenceReason::RuntimeExecutorError,
            Self::EvidenceFieldsIncomplete => {
                DistroEvidenceReason::EvidenceFieldsIncomplete
            }
            Self::CommandScopeMismatch => DistroEvidenceReason::CommandScopeMismatch,
        }
    }

    /// success exit であるかを返します。
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }
}

/// platform command unavailable の閉じた分類です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealDeviceCommandUnavailable {
    /// command program が存在しない状態です。
    MissingProgram,
    /// command spawn が失敗した状態です。
    SpawnFailed,
    /// command は開始したが non-zero exit で終了した状態です。
    NonZeroExit,
    /// timeout として扱う状態です。
    TimedOut,
    /// canonical closed set にない command です。
    CommandNotAdmitted,
}

/// `std::process::Command` の出力を test-side evidence 用に保持する型です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealDeviceCommandOutput {
    /// wrapper が採用する exit status です。
    pub exit_status: RealDeviceCommandExitStatus,
    /// platform command の raw exit status です。
    pub raw_exit_status: Option<i32>,
    /// stdout summary です。
    pub stdout_summary: String,
    /// stderr summary です。
    pub stderr_summary: String,
    /// toolchain / runtime version summary です。
    pub toolchain_runtime_version: String,
    /// unavailable reason です。
    pub unavailable: Option<RealDeviceCommandUnavailable>,
}

impl RealDeviceCommandOutput {
    /// 成功出力を構築します。
    pub fn success(
        stdout: impl Into<String>,
        stderr: impl Into<String>,
        runtime: impl Into<String>,
    ) -> Self {
        Self {
            exit_status: RealDeviceCommandExitStatus::Success,
            raw_exit_status: Some(0),
            stdout_summary: normalize_summary(stdout.into()),
            stderr_summary: normalize_summary(stderr.into()),
            toolchain_runtime_version: normalize_summary(runtime.into()),
            unavailable: None,
        }
    }

    /// 必須 device class 未観測の出力へ変換します。
    pub fn required_device_not_observed(mut self) -> Self {
        self.exit_status = RealDeviceCommandExitStatus::RequiredDeviceNotObserved;
        self
    }

    /// platform command unavailable 出力を構築します。
    pub fn unavailable(
        reason: RealDeviceCommandUnavailable,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
        raw_exit_status: Option<i32>,
        runtime: impl Into<String>,
    ) -> Self {
        Self {
            exit_status: RealDeviceCommandExitStatus::PlatformCommandUnavailable,
            raw_exit_status,
            stdout_summary: normalize_summary(stdout.into()),
            stderr_summary: normalize_summary(stderr.into()),
            toolchain_runtime_version: normalize_summary(runtime.into()),
            unavailable: Some(reason),
        }
    }

    /// evidence field incomplete 出力を構築します。
    pub fn evidence_fields_incomplete(runtime: impl Into<String>) -> Self {
        Self {
            exit_status: RealDeviceCommandExitStatus::EvidenceFieldsIncomplete,
            raw_exit_status: None,
            stdout_summary: String::new(),
            stderr_summary: String::new(),
            toolchain_runtime_version: normalize_summary(runtime.into()),
            unavailable: None,
        }
    }
}

fn normalize_summary(value: String) -> String {
    let single_line = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    if single_line.chars().count() > 512 {
        let truncated = single_line.chars().take(512).collect::<String>();
        format!("{truncated}...")
    } else {
        single_line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_output_unit_covers_summary_truncation_branch() {
        let output = RealDeviceCommandOutput::success(
            "x".repeat(513),
            " first stderr line \n\n second stderr line ",
            " runtime line ",
        );

        assert!(output.stdout_summary.ends_with("..."));
        assert_eq!(output.stdout_summary.chars().count(), 515);
        assert_eq!(
            output.stderr_summary,
            "first stderr line | second stderr line"
        );
        assert_eq!(output.toolchain_runtime_version, "runtime line");
    }
}
