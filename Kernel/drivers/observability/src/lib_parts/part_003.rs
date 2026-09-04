/// retention bound の接続先です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyRetentionBoundClass {
    /// retained closed field set only.
    ClosedFieldSet,
    /// bound is connected to the source-owned resource bound policy.
    ResourceBoundPolicy,
    /// bound is connected to a specialized source-owned retention policy.
    SpecializedRetentionPolicy,
    /// value is not retained.
    NotRetained,
    /// unbounded retention; prohibited.
    Unbounded,
}

/// retention policy guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrivacyRetentionPolicy {
    target: PrivacyRetentionTarget,
    owner: PrivacyRetentionOwner,
    data_class: PrivacyDataClass,
    bound_class: PrivacyRetentionBoundClass,
    owner_declared: bool,
    bound_declared: bool,
    hash_chain_not_mutable_domain_state: bool,
    driver_cache_bounded_local_only: bool,
}

/// retention policy の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyRetentionPolicyError {
    /// retention owner が target と一致していません。
    RetentionOwnerMismatch,
    /// retention bound が未宣言または unbounded です。
    RetentionBoundMissing,
    /// retain してはいけない data class です。
    DataClassCannotBeRetained,
    /// hash-chain が mutable domain state を保持しています。
    HashChainRetainsMutableState,
    /// packet/key cache が driver-local bounded retention ではありません。
    DriverCacheNotBoundedLocal,
}

impl PrivacyRetentionPolicy {
    /// retention owner/bound と data class の整合を確認します。
    pub fn try_new(
        target: PrivacyRetentionTarget,
        owner: PrivacyRetentionOwner,
        data_class: PrivacyDataClass,
        bound_class: PrivacyRetentionBoundClass,
        owner_declared: bool,
        bound_declared: bool,
        hash_chain_not_mutable_domain_state: bool,
        driver_cache_bounded_local_only: bool,
    ) -> Result<Self, PrivacyRetentionPolicyError> {
        if !owner_declared || owner != target.required_owner() {
            return Err(PrivacyRetentionPolicyError::RetentionOwnerMismatch);
        }
        if !bound_declared || matches!(bound_class, PrivacyRetentionBoundClass::Unbounded) {
            return Err(PrivacyRetentionPolicyError::RetentionBoundMissing);
        }
        if data_class.is_forbidden_retention_for(target) {
            return Err(PrivacyRetentionPolicyError::DataClassCannotBeRetained);
        }
        if matches!(target, PrivacyRetentionTarget::AuditHashChain)
            && !hash_chain_not_mutable_domain_state
        {
            return Err(PrivacyRetentionPolicyError::HashChainRetainsMutableState);
        }
        if matches!(
            target,
            PrivacyRetentionTarget::PacketCache | PrivacyRetentionTarget::KeyCache
        ) && (!driver_cache_bounded_local_only
            || !matches!(
                bound_class,
                PrivacyRetentionBoundClass::ResourceBoundPolicy
                    | PrivacyRetentionBoundClass::SpecializedRetentionPolicy
            ))
        {
            return Err(PrivacyRetentionPolicyError::DriverCacheNotBoundedLocal);
        }

        Ok(Self {
            target,
            owner,
            data_class,
            bound_class,
            owner_declared,
            bound_declared,
            hash_chain_not_mutable_domain_state,
            driver_cache_bounded_local_only,
        })
    }
}

impl PrivacyRetentionTarget {
    /// retention target ごとの owner です。
    pub const fn required_owner(self) -> PrivacyRetentionOwner {
        match self {
            Self::AuditEvent | Self::AuditHashChain => PrivacyRetentionOwner::Core,
            Self::LogTrace | Self::Metrics | Self::PacketCache | Self::KeyCache => {
                PrivacyRetentionOwner::Driver
            }
            Self::SdkClientLocalData => PrivacyRetentionOwner::Sdk,
            Self::RegulatedOnly => PrivacyRetentionOwner::Regulated,
        }
    }
}

impl PrivacyDataClass {
    /// target に retain してはいけない data class です。
    pub const fn is_forbidden_retention_for(self, target: PrivacyRetentionTarget) -> bool {
        match self {
            Self::RawSecret | Self::RawToken => true,
            Self::RawKeyMaterial => !matches!(target, PrivacyRetentionTarget::KeyCache),
            Self::RawPacketPayload => !matches!(target, PrivacyRetentionTarget::PacketCache),
            Self::RegulatedPayload => !matches!(target, PrivacyRetentionTarget::RegulatedOnly),
            Self::CoreReference
            | Self::CatalogReason
            | Self::NonSensitiveTag
            | Self::DiagnosticDetail
            | Self::EdgeProxyMetadata => false,
        }
    }
}

/// metric/log label の privacy guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrivacyLabelGuard {
    data_class: PrivacyDataClass,
    label_allowlisted: bool,
    raw_identifier_absent: bool,
    sensitive_payload_absent: bool,
    cardinality_bounded: bool,
    drop_or_rewrite_before_export: bool,
}

/// metric/log label privacy guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyLabelGuardError {
    /// label が source allowlist 外です。
    LabelNotAllowlisted,
    /// label に sensitive identity/payload が含まれています。
    SensitiveLabelValue,
    /// label cardinality が unbounded です。
    UnboundedLabelCardinality,
    /// export 前の drop/rewrite が未宣言です。
    DropOrRewriteMissing,
}

impl PrivacyLabelGuard {
    /// label が sensitive identity や raw payload を含まないことを確認します。
    pub const fn try_new(
        data_class: PrivacyDataClass,
        label_allowlisted: bool,
        raw_identifier_absent: bool,
        sensitive_payload_absent: bool,
        cardinality_bounded: bool,
        drop_or_rewrite_before_export: bool,
    ) -> Result<Self, PrivacyLabelGuardError> {
        if !label_allowlisted || data_class.is_raw_sensitive() {
            return Err(PrivacyLabelGuardError::LabelNotAllowlisted);
        }
        if !raw_identifier_absent || !sensitive_payload_absent {
            return Err(PrivacyLabelGuardError::SensitiveLabelValue);
        }
        if !cardinality_bounded {
            return Err(PrivacyLabelGuardError::UnboundedLabelCardinality);
        }
        if !drop_or_rewrite_before_export {
            return Err(PrivacyLabelGuardError::DropOrRewriteMissing);
        }

        Ok(Self {
            data_class,
            label_allowlisted,
            raw_identifier_absent,
            sensitive_payload_absent,
            cardinality_bounded,
            drop_or_rewrite_before_export,
        })
    }
}

/// privacy/redaction/retention failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyRedactionRetentionFailureKind {
    /// redaction cannot be applied safely.
    ExportRedactionRequired,
    /// secret source unavailable.
    SecretUnavailable,
    /// verification detail is not safely exposable.
    UnsafeVerificationDetailSuppressed,
    /// packet payload export was rejected.
    PacketPayloadExportRejected,
    /// metric/log label contains sensitive material.
    SensitiveLabelRejected,
    /// sensitive identity would be exposed by signal cardinality.
    SensitiveCardinalityRejected,
    /// retention owner/bound is invalid.
    RetentionPolicyInvalid,
    /// rotation detail export would expose raw secret material.
    RotationDetailExportRejected,
}

impl PrivacyRedactionRetentionFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExportRedactionRequired
            | Self::PacketPayloadExportRejected
            | Self::RotationDetailExportRejected => "export_redaction_required",
            Self::SecretUnavailable => "secret_unavailable",
            Self::UnsafeVerificationDetailSuppressed
            | Self::SensitiveLabelRejected
            | Self::RetentionPolicyInvalid => "observability_export_not_allowed",
            Self::SensitiveCardinalityRejected => "metric_cardinality_exceeded",
        }
    }
}

/// privacy/redaction/retention failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrivacyRedactionRetentionFailure {
    kind: PrivacyRedactionRetentionFailureKind,
    reason: CatalogedReasonRef,
}

impl PrivacyRedactionRetentionFailure {
    /// failure kind を cataloged reason に接続します。
    pub fn from_kind(kind: PrivacyRedactionRetentionFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("privacy redaction retention reason code must be registered");
        Self { kind, reason }
    }
}

/// privacy/redaction/retention 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedPrivacyRedactionRetentionBehavior {
    /// raw secret/token/key is included in core/audit/log/metric export.
    RawCredentialMaterialExported,
    /// packet payload bytes are exported outside the driver packet lifecycle.
    RawPacketPayloadExported,
    /// regulated payload is added to a generic core audit/export path.
    RegulatedPayloadInGenericExport,
    /// redaction is delegated to external sink default behavior.
    ExternalSinkDefaultRedaction,
    /// free-text diagnostic detail becomes authoritative reason.
    DiagnosticDetailAsAuthoritativeReason,
    /// diagnostic export preserves raw sensitive material.
    SensitiveMaterialRetainedForDiagnostics,
    /// raw edge/proxy metadata is logged as identity.
    RawEdgeProxyMetadataAsIdentity,
    /// retention owner or bound is missing.
    UnownedOrUnboundedRetention,
}
