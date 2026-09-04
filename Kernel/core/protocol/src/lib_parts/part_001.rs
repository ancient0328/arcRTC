// core/protocol は deterministic encoding、protocol versioning、compatibility/deprecation の core surface です。
//
// wire codec や external envelope は driver に置き、ここでは core が判断する
// canonical serialization と version/capability 語彙を後続 task で配置します。

use arcrtc_core_command::{TargetSurface, UseCaseOutcome};
use arcrtc_core_identity::{CorrelationId, OpaqueReference};
use arcrtc_core_reason::CatalogedReasonRef;

/// protocol boundary の所有 package が確定していることを示す marker です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreProtocolSurface;

/// canonical encoding の対象 data class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalDataClass {
    /// audit event hash input です。
    AuditEventHashInput,
    /// core reason category/code です。
    CoreReason,
    /// core opaque reference です。
    CoreReference,
    /// evidence timestamp です。
    EvidenceTimestamp,
    /// normalized duration です。
    Duration,
    /// admitted digest/reference only の binary payload です。
    BinaryPayloadDigest,
}

/// canonical rule が定義済みかどうかです。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalRuleStatus {
    /// source contract で指定済みです。
    Specified,
    /// source contract で未指定です。
    Unspecified,
}

/// unknown field handling です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnknownFieldHandling {
    /// compatibility policy が明示した場合だけ ignored です。
    IgnoredOnlyWhenCompatibilityAllows,
    /// 既定では reject します。
    Reject,
}

/// canonical field rule set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalEncodingRuleSet {
    field_set: CanonicalRuleStatus,
    field_order: CanonicalRuleStatus,
    absent_null_handling: CanonicalRuleStatus,
    enum_representation: CanonicalRuleStatus,
    integer_representation: CanonicalRuleStatus,
    decimal_precision: CanonicalRuleStatus,
    string_normalization: CanonicalRuleStatus,
    binary_digest_representation: CanonicalRuleStatus,
    timestamp_representation: CanonicalRuleStatus,
    unknown_field_handling: UnknownFieldHandling,
    version_field_inclusion: CanonicalRuleStatus,
    redaction_before_digest: CanonicalRuleStatus,
}

impl CanonicalEncodingRuleSet {
    /// canonical encoding に必要な rule status を束ねます。
    pub const fn new(
        field_set: CanonicalRuleStatus,
        field_order: CanonicalRuleStatus,
        absent_null_handling: CanonicalRuleStatus,
        enum_representation: CanonicalRuleStatus,
        integer_representation: CanonicalRuleStatus,
        decimal_precision: CanonicalRuleStatus,
        string_normalization: CanonicalRuleStatus,
        binary_digest_representation: CanonicalRuleStatus,
        timestamp_representation: CanonicalRuleStatus,
        unknown_field_handling: UnknownFieldHandling,
        version_field_inclusion: CanonicalRuleStatus,
        redaction_before_digest: CanonicalRuleStatus,
    ) -> Self {
        Self {
            field_set,
            field_order,
            absent_null_handling,
            enum_representation,
            integer_representation,
            decimal_precision,
            string_normalization,
            binary_digest_representation,
            timestamp_representation,
            unknown_field_handling,
            version_field_inclusion,
            redaction_before_digest,
        }
    }

    /// canonical encoding に必要な source rule がすべて指定済みかです。
    pub const fn is_complete_for_canonical_encoding(&self) -> bool {
        matches!(self.field_set, CanonicalRuleStatus::Specified)
            && matches!(self.field_order, CanonicalRuleStatus::Specified)
            && matches!(self.absent_null_handling, CanonicalRuleStatus::Specified)
            && matches!(self.enum_representation, CanonicalRuleStatus::Specified)
            && matches!(self.integer_representation, CanonicalRuleStatus::Specified)
            && matches!(self.decimal_precision, CanonicalRuleStatus::Specified)
            && matches!(self.string_normalization, CanonicalRuleStatus::Specified)
            && matches!(
                self.binary_digest_representation,
                CanonicalRuleStatus::Specified
            )
            && matches!(self.timestamp_representation, CanonicalRuleStatus::Specified)
            && matches!(self.version_field_inclusion, CanonicalRuleStatus::Specified)
            && matches!(self.redaction_before_digest, CanonicalRuleStatus::Specified)
    }
}

/// canonical format/version です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalFormatVersion {
    format: &'static str,
    version: &'static str,
}

impl CanonicalFormatVersion {
    /// canonical format と version を明示します。
    pub const fn new(format: &'static str, version: &'static str) -> Self {
        Self { format, version }
    }

    /// canonical format 名です。
    pub const fn format(&self) -> &'static str {
        self.format
    }

    /// canonical format version です。
    pub const fn version(&self) -> &'static str {
        self.version
    }
}

/// canonical digest value です。raw sensitive payload は持ちません。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalDigest {
    format_version: CanonicalFormatVersion,
    algorithm: &'static str,
    digest: Vec<u8>,
}

impl CanonicalDigest {
    /// canonical digest を保持します。
    pub fn new(
        format_version: CanonicalFormatVersion,
        algorithm: &'static str,
        digest: Vec<u8>,
    ) -> Result<Self, CanonicalEncodingError> {
        if algorithm.is_empty() || digest.is_empty() {
            return Err(CanonicalEncodingError::EmptyDigestMaterial);
        }
        Ok(Self {
            format_version,
            algorithm,
            digest,
        })
    }

    /// canonical format/version です。
    pub const fn format_version(&self) -> CanonicalFormatVersion {
        self.format_version
    }

    /// digest algorithm です。
    pub const fn algorithm(&self) -> &'static str {
        self.algorithm
    }

    /// digest bytes です。
    pub fn digest(&self) -> &[u8] {
        &self.digest
    }
}

/// canonical encoding failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalEncodingFailureKind {
    /// canonical encoding cannot be produced.
    CanonicalSerializationFailed,
    /// canonical verification mismatch.
    CanonicalSerializationMismatch,
    /// external payload cannot decode to core type.
    ExternalDecodeFailed,
    /// external response encoding failed.
    ExternalEncodeFailed,
    /// required canonical field missing.
    MissingRequiredWireField,
    /// unsupported canonical version.
    UnsupportedCanonicalVersion,
}

impl CanonicalEncodingFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::CanonicalSerializationFailed => "canonical_serialization_failed",
            Self::CanonicalSerializationMismatch => "canonical_serialization_mismatch",
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::ExternalEncodeFailed => "external_encode_failed",
            Self::MissingRequiredWireField => "missing_required_wire_field",
            Self::UnsupportedCanonicalVersion => "unsupported_version",
        }
    }
}

/// canonical digest construction error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalEncodingError {
    /// algorithm または digest が空です。
    EmptyDigestMaterial,
}

/// version を持つ surface の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VersionedSurface {
    /// Signaling command / event schema です。
    SignalingContract,
    /// TURN request / response semantic model です。
    TurnContract,
    /// SFU endpoint / stream / route / decision model です。
    SfuContract,
    /// SDK public API mapped to Signaling contract です。
    SdkPublicContract,
    /// audit event schema です。
    AuditSchema,
    /// driver wire encoding です。
    DriverWireEncoding,
}

/// version semantics owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VersionOwner {
    /// core が contract version semantics を所有します。
    Core,
    /// driver が external encoding version を所有します。
    Driver,
    /// sdk が public API version を所有します。
    Sdk,
}

/// semantic contract version です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl ContractVersion {
    /// semantic contract version を作ります。
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// major version です。
    pub const fn major(&self) -> u16 {
        self.major
    }
}

/// driver-owned wire encoding version です。core contract version ではありません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireEncodingVersion {
    code: &'static str,
}

impl WireEncodingVersion {
    /// wire encoding version code を保持します。
    pub const fn new(code: &'static str) -> Self {
        Self { code }
    }

    /// wire encoding version code です。
    pub const fn code(&self) -> &'static str {
        self.code
    }
}

/// version negotiation result の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VersionNegotiationOutcome {
    /// exact version accepted です。
    AcceptedExact(ContractVersion),
    /// compatible version accepted です。
    AcceptedCompatible {
        /// requested version です。
        requested: ContractVersion,
        /// accepted version です。
        accepted: ContractVersion,
    },
    /// unsupported version rejected です。
    RejectedUnsupported(ContractVersion),
}

/// capability 名です。version の代替ではありません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability {
    name: &'static str,
}

impl Capability {
    /// capability 名を作ります。
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    /// capability name です。
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

/// capability の意味境界です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityRule {
    /// accepted version 内の optional behavior だけを表します。
    OptionalWithinAcceptedVersion,
    /// required state transition semantics を変更してはいけません。
    MustNotAlterRequiredStateTransition,
}

/// versioned surface と owner の対応です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VersionedSurfaceOwnership {
    surface: VersionedSurface,
    owner: VersionOwner,
}

impl VersionedSurfaceOwnership {
    /// versioned surface と owner を固定します。
    pub const fn new(surface: VersionedSurface, owner: VersionOwner) -> Self {
        Self { surface, owner }
    }

    /// versioned surface です。
    pub const fn surface(&self) -> VersionedSurface {
        self.surface
    }

    /// version semantics owner です。
    pub const fn owner(&self) -> VersionOwner {
        self.owner
    }
}

/// compatibility change の分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompatibilityChange {
    /// optional field with default behavior の追加です。
    AddOptionalFieldWithDefault,
    /// required field の追加です。
    AddRequiredField,
    /// field removal です。
    RemoveField,
    /// reason code semantics の変更です。
    ChangeReasonCodeSemantics,
    /// reason code の追加です。
    AddReasonCode,
    /// state transition の変更です。
    ChangeStateTransition,
    /// driver encoding の追加です。
    AddDriverEncoding,
}

/// compatibility classification です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompatibilityClassification {
    /// compatible change です。
    Compatible,
    /// breaking change です。
    Breaking,
    /// catalog update 等の条件付き compatible change です。
    Conditional,
    /// prohibited change です。
    Prohibited,
}

impl CompatibilityChange {
    /// canonical の compatibility classification です。
    pub const fn classification(self) -> CompatibilityClassification {
        match self {
            Self::AddOptionalFieldWithDefault => CompatibilityClassification::Compatible,
            Self::AddRequiredField => CompatibilityClassification::Breaking,
            Self::RemoveField => CompatibilityClassification::Breaking,
            Self::ChangeReasonCodeSemantics => CompatibilityClassification::Prohibited,
            Self::AddReasonCode => CompatibilityClassification::Conditional,
            Self::ChangeStateTransition => CompatibilityClassification::Breaking,
            Self::AddDriverEncoding => CompatibilityClassification::Compatible,
        }
    }
}

/// accepted compatible version range です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompatibilityVersionRange {
    surface: VersionedSurface,
    min_inclusive: ContractVersion,
    max_inclusive: ContractVersion,
}

impl CompatibilityVersionRange {
    /// surface と accepted compatible range を明示します。
    pub const fn new(
        surface: VersionedSurface,
        min_inclusive: ContractVersion,
        max_inclusive: ContractVersion,
    ) -> Self {
        Self {
            surface,
            min_inclusive,
            max_inclusive,
        }
    }

    /// compatibility surface です。
    pub const fn surface(&self) -> VersionedSurface {
        self.surface
    }
}

/// deprecation lifecycle step の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeprecationLifecycleStep {
    /// affected surface and version を特定します。
    IdentifyAffectedSurfaceAndVersion,
    /// unsupported-version behavior と cataloged reason を定義します。
    DefineUnsupportedVersionBehavior,
    /// SDK parity / driver mapping rules を更新します。
    UpdateSdkParityAndDriverMapping,
    /// compatibility window の終了後に support を除去します。
    RemoveAfterCompatibilityWindow,
}

/// deprecation decision の source contract です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeprecationDecision {
    affected_surface: VersionedSurface,
    affected_version: ContractVersion,
    owner: VersionOwner,
    compatibility_window: CompatibilityVersionRange,
    lifecycle_steps: Vec<DeprecationLifecycleStep>,
}

impl DeprecationDecision {
    /// deprecation の affected surface/version/window を記録します。
    pub fn new(
        affected_surface: VersionedSurface,
        affected_version: ContractVersion,
        owner: VersionOwner,
        compatibility_window: CompatibilityVersionRange,
        lifecycle_steps: Vec<DeprecationLifecycleStep>,
    ) -> Self {
        Self {
            affected_surface,
            affected_version,
            owner,
            compatibility_window,
            lifecycle_steps,
        }
    }

    /// deprecation lifecycle steps です。
    pub fn lifecycle_steps(&self) -> &[DeprecationLifecycleStep] {
        &self.lifecycle_steps
    }
}

/// unsupported version / compatibility mapping failure の reason mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompatibilityFailureKind {
    /// Signaling command/event version unsupported です。
    UnsupportedCommandVersion,
    /// media-facing SFU/transport contract unsupported です。
    UnsupportedMediaContractVersion,
    /// TURN contract version unsupported です。
    UnsupportedTurnContractVersion,
    /// driver wire encoding unsupported です。
    UnsupportedDriverWireVersion,
    /// required wire field missing after compatibility mapping です。
    MissingRequiredWireField,
    /// external enum cannot map after compatibility mapping です。
    ExternalEnumUnmapped,
}

impl CompatibilityFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::UnsupportedCommandVersion => "unsupported_command_version",
            Self::UnsupportedMediaContractVersion => "unsupported_media_contract_version",
            Self::UnsupportedTurnContractVersion => "unsupported_turn_contract_version",
            Self::UnsupportedDriverWireVersion => "unsupported_driver_wire_version",
            Self::MissingRequiredWireField => "missing_required_wire_field",
            Self::ExternalEnumUnmapped => "external_enum_unmapped",
        }
    }
}
