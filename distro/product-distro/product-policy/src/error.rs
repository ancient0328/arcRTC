//! product policy のtyped errorです。

/// product policy error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductPolicyError {
    /// fixture identity が不正です。
    InvalidFixtureIdentity,
    /// security reason mapping に失敗しました。
    SecurityReasonMappingFailed,
    /// readiness は採用されていません。
    ReadinessNotAdmitted,
}

impl ProductPolicyError {
    /// distro evidence reasonへ変換します。
    pub const fn distro_reason(&self) -> arcrtc_distro_evidence::DistroEvidenceReason {
        match self {
            Self::InvalidFixtureIdentity => {
                arcrtc_distro_evidence::DistroEvidenceReason::FixtureIdentityInvalid
            }
            Self::SecurityReasonMappingFailed => {
                arcrtc_distro_evidence::DistroEvidenceReason::StateBoundaryViolation
            }
            Self::ReadinessNotAdmitted => {
                arcrtc_distro_evidence::DistroEvidenceReason::ReadinessNotAdmitted
            }
        }
    }
}
