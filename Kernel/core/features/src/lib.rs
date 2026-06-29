//! core/features は feature の許可・拒否・out-of-scope 判定を core 側で扱う surface です。
//!
//! runtime feature flag system や rollout mechanism はここでは実装せず、
//! core が受理できる feature 語彙と拒否理由を後続 task で配置します。

/// features boundary の所有 package が確定していることを示す marker です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreFeaturesSurface;

use arcrtc_core_identity::CorrelationId;

/// v0.2 initial architecture で excluded として固定された feature class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExcludedFeatureClass {
    /// user/application chat content or workflow です。
    ChatApplicationSemantics,
    /// media recording, storage, retrieval, retention workflow です。
    RecordingWorkflow,
    /// capture/share workflow beyond media routing です。
    ScreenShareWorkflow,
    /// SCTP/DataChannel entrypoint protocol semantics です。
    DataChannelApplicationSemantics,
    /// user-facing UI workflow です。
    UiEndUserWorkflow,
    /// camera/microphone/screen capture permission UX です。
    MediaCaptureWorkflow,
    /// medical/regulated domain workflow です。
    RegulatedDomainWorkflow,
}

/// feature request が現れた surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureRequestedSurface {
    /// generic core surface です。
    Core,
    /// Signaling surface です。
    Signaling,
    /// SFU surface です。
    Sfu,
    /// TURN surface です。
    Turn,
    /// SDK public surface です。
    Sdk,
    /// driver boundary です。
    Driver,
    /// entrypoint/demo boundary です。
    Entrypoints,
    /// regulated optional support boundary です。
    Regulated,
}

/// future admission に必須の declaration です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FutureAdmissionRequirement {
    /// feature class の明示です。
    FeatureClass,
    /// owner package/layer の明示です。
    OwnerPackageLayer,
    /// generic communication core との関係です。
    GenericCoreRelation,
    /// public SDK/API surface です。
    PublicSdkApiSurface,
    /// driver/runtime dependency boundary です。
    DriverRuntimeDependencyBoundary,
    /// security/privacy/redaction boundary です。
    SecurityPrivacyRedactionBoundary,
    /// reason catalog additions です。
    ReasonCatalogAdditions,
    /// audit event relation です。
    AuditEventRelation,
    /// evidence class です。
    EvidenceClass,
    /// migration/deprecation relation です。
    MigrationDeprecationRelation,
}

/// excluded feature request に対する decision class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureAdmissionDecisionClass {
    /// fail-closed rejection です。
    Rejected,
    /// current target の close claim に採用しない扱いです。
    CloseNotClaimed,
    /// separate ADR/Canonical によって admission された扱いです。
    AdmittedByCanonical,
}

/// out-of-scope feature decision です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureAdmissionDecision {
    correlation_id: Option<CorrelationId>,
    feature_class: ExcludedFeatureClass,
    requested_surface: FeatureRequestedSurface,
    decision_class: FeatureAdmissionDecisionClass,
    failure_kind: Option<FeatureAdmissionFailureKind>,
}

impl FeatureAdmissionDecision {
    /// feature admission / exclusion decision を作ります。
    pub const fn new(
        correlation_id: Option<CorrelationId>,
        feature_class: ExcludedFeatureClass,
        requested_surface: FeatureRequestedSurface,
        decision_class: FeatureAdmissionDecisionClass,
        failure_kind: Option<FeatureAdmissionFailureKind>,
    ) -> Self {
        Self {
            correlation_id,
            feature_class,
            requested_surface,
            decision_class,
            failure_kind,
        }
    }

    /// feature class です。
    pub const fn feature_class(&self) -> ExcludedFeatureClass {
        self.feature_class
    }

    /// requested surface です。
    pub const fn requested_surface(&self) -> FeatureRequestedSurface {
        self.requested_surface
    }

    /// decision class です。
    pub const fn decision_class(&self) -> FeatureAdmissionDecisionClass {
        self.decision_class
    }
}

/// out-of-scope feature failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureAdmissionFailureKind {
    /// requested feature is outside v0.2 scope.
    FeatureOutOfScope,
    /// feature admission lacks ADR/Canonical.
    FeatureAdmissionNotDocumented,
    /// chat semantics requested.
    ChatNotSupported,
    /// recording workflow requested.
    RecordingNotSupported,
    /// screen share workflow requested.
    ScreenShareNotSupported,
    /// DataChannel application semantics requested.
    DataChannelNotSupported,
    /// UI/end-user workflow requested.
    UiWorkflowNotSupported,
    /// media capture workflow requested as core/server feature.
    MediaCaptureNotSupported,
    /// regulated workflow requested as generic core feature.
    RegulatedWorkflowNotSupported,
}

impl FeatureAdmissionFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::FeatureOutOfScope => "feature_out_of_scope",
            Self::FeatureAdmissionNotDocumented => "feature_admission_not_documented",
            Self::ChatNotSupported => "chat_not_supported",
            Self::RecordingNotSupported => "recording_not_supported",
            Self::ScreenShareNotSupported => "screen_share_not_supported",
            Self::DataChannelNotSupported => "datachannel_not_supported",
            Self::UiWorkflowNotSupported => "ui_workflow_not_supported",
            Self::MediaCaptureNotSupported => "media_capture_not_supported",
            Self::RegulatedWorkflowNotSupported => "regulated_workflow_not_supported",
        }
    }
}
