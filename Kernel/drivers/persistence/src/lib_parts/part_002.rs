impl SchemaMigrationFailure {
    /// cataloged reason と必要な persistence failure mapping に接続します。
    pub fn from_kind(kind: SchemaMigrationFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("schema migration failure reason code must be registered");
        let persistence_failure = match kind {
            SchemaMigrationFailureKind::PersistenceUnavailable => {
                Some(PersistencePortFailure::from_kind(
                    PersistencePortFailureKind::PersistenceUnavailable,
                ))
            }
            SchemaMigrationFailureKind::PersistenceRetryBoundExceeded => {
                Some(PersistencePortFailure::from_kind(
                    PersistencePortFailureKind::PersistenceRetryBoundExceeded,
                ))
            }
            SchemaMigrationFailureKind::PersistenceRetryDurationExceeded => {
                Some(PersistencePortFailure::from_kind(
                    PersistencePortFailureKind::PersistenceRetryDurationExceeded,
                ))
            }
            SchemaMigrationFailureKind::DriverShutdown => Some(PersistencePortFailure::from_kind(
                PersistencePortFailureKind::DriverShutdown,
            )),
            SchemaMigrationFailureKind::RuntimeConfigMissing
            | SchemaMigrationFailureKind::RuntimeConfigInvalid
            | SchemaMigrationFailureKind::ExternalDecodeFailed
            | SchemaMigrationFailureKind::UnsupportedDriverWireVersion
            | SchemaMigrationFailureKind::MissingRequiredWireField
            | SchemaMigrationFailureKind::ExternalEnumUnmapped => None,
        };

        Self {
            kind,
            reason,
            persistence_failure,
        }
    }
}

/// schema migration lifecycle 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedSchemaMigrationBehavior {
    /// migration file defines domain invariant.
    MigrationFileDefinesDomainInvariant,
    /// schema existence is treated as restore / replay success.
    SchemaExistenceAsRestoreReplaySuccess,
    /// core imports DB migration library.
    CoreImportsDbMigrationLibrary,
    /// driver migration failure is silently skipped.
    MigrationFailureSilentlySkipped,
    /// incompatible persisted representation is best-effort decoded.
    BestEffortDecodeOfIncompatibleRepresentation,
    /// storage row/object layout is treated as canonical serialization without rule.
    StorageLayoutAsCanonicalSerialization,
    /// rollback plan is omitted for forward migration.
    RollbackPlanOmitted,
    /// migration snapshot is treated as restore proof without artifact classification.
    MigrationSnapshotAsRestoreProof,
}

/// v0.2 で許可する export / backup artifact class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportBackupArtifactClass {
    /// redacted audit event export or chain segment.
    AuditEventExport,
    /// domain checkpoint backup for recovery qualification.
    StateCheckpointBackup,
    /// driver-local DB/object/filesystem snapshot.
    PersistenceSnapshot,
    /// typed configuration/profile/policy bundle export.
    ConfigurationExport,
    /// schema/migration support snapshot.
    SchemaMigrationSnapshot,
}

/// export / backup artifact の redaction class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactRedactionClass {
    /// sensitive material is absent or redacted.
    Redacted,
    /// encrypted but still sensitive.
    EncryptedSensitive,
    /// artifact has no sensitive fields by class.
    NotSensitiveByClass,
}

/// export / backup artifact の retention class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactRetentionClass {
    /// short-lived diagnostic artifact.
    Ephemeral,
    /// bounded retention policy applies.
    BoundedRetention,
    /// governance-controlled retention applies.
    GovernanceControlled,
}

/// export / backup artifact の integrity verification class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactIntegrityClass {
    /// digest / hash over artifact bytes.
    Digest,
    /// audit hash-chain reference.
    AuditHashChainReference,
    /// concrete storage checksum only.
    StorageChecksum,
    /// digest is not applicable and reason is explicit.
    ExplicitNonDigestReason,
}

/// restore / replay applicability の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestoreReplayApplicability {
    /// restore/replay input としては使いません。
    NotIntendedForRestoreReplay,
    /// source restore policy で許可された restore input です。
    RestoreInput,
    /// source replay policy で許可された verification input です。
    ReplayVerificationInput,
}

/// sensitive data を一般 export / backup artifact へ入れないための guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArtifactSensitiveDataGuard {
    raw_secret_absent: bool,
    raw_token_absent: bool,
    raw_packet_payload_absent: bool,
    unredacted_sdp_ice_absent: bool,
    regulated_payload_absent: bool,
    personal_data_absent_or_admitted: bool,
}

/// sensitive data guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactSensitiveDataError {
    /// raw secret / token / packet / SDP / ICE / regulated / personal data が残っています。
    SensitiveMaterialRequiresRedaction,
}

impl ArtifactSensitiveDataGuard {
    /// sensitive material が未分類 artifact へ混入していないことを確認します。
    pub const fn try_new(
        raw_secret_absent: bool,
        raw_token_absent: bool,
        raw_packet_payload_absent: bool,
        unredacted_sdp_ice_absent: bool,
        regulated_payload_absent: bool,
        personal_data_absent_or_admitted: bool,
    ) -> Result<Self, ArtifactSensitiveDataError> {
        if !raw_secret_absent
            || !raw_token_absent
            || !raw_packet_payload_absent
            || !unredacted_sdp_ice_absent
            || !regulated_payload_absent
            || !personal_data_absent_or_admitted
        {
            return Err(ArtifactSensitiveDataError::SensitiveMaterialRequiresRedaction);
        }

        Ok(Self {
            raw_secret_absent,
            raw_token_absent,
            raw_packet_payload_absent,
            unredacted_sdp_ice_absent,
            regulated_payload_absent,
            personal_data_absent_or_admitted,
        })
    }
}

/// integrity verification class の混同を防ぐ guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArtifactIntegrityGuard {
    integrity_class: ArtifactIntegrityClass,
    digest_hashchain_checksum_separated: bool,
    storage_checksum_not_hashchain_proof: bool,
    export_format_not_canonical_without_rule: bool,
}

/// integrity guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactIntegrityGuardError {
    /// digest / hash-chain / storage checksum が混同されています。
    IntegrityClassConflated,
    /// export format を explicit rule なしに canonical serialization と扱っています。
    ExportFormatAsCanonicalSerialization,
}

impl ArtifactIntegrityGuard {
    /// checksum や digest を過剰な意味に昇格させないことを確認します。
    pub const fn try_new(
        integrity_class: ArtifactIntegrityClass,
        digest_hashchain_checksum_separated: bool,
        storage_checksum_not_hashchain_proof: bool,
        export_format_not_canonical_without_rule: bool,
    ) -> Result<Self, ArtifactIntegrityGuardError> {
        if !digest_hashchain_checksum_separated || !storage_checksum_not_hashchain_proof {
            return Err(ArtifactIntegrityGuardError::IntegrityClassConflated);
        }
        if !export_format_not_canonical_without_rule {
            return Err(ArtifactIntegrityGuardError::ExportFormatAsCanonicalSerialization);
        }

        Ok(Self {
            integrity_class,
            digest_hashchain_checksum_separated,
            storage_checksum_not_hashchain_proof,
            export_format_not_canonical_without_rule,
        })
    }
}

/// restore / import / replay への artifact 使用を分離する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArtifactRestoreImportGuard {
    applicability: RestoreReplayApplicability,
    source_policy_allows_use: bool,
    restore_or_replay_verification_completed: bool,
}

/// restore/import guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactRestoreImportGuardError {
    /// source restore/replay policy が使用を許可していません。
    RestoreReplayPolicyDenied,
    /// restore/replay verification が完了していません。
    RestoreReplayVerificationMissing,
}

impl ArtifactRestoreImportGuard {
    /// artifact presence と restore/import/replay admission を分離します。
    pub const fn try_new(
        applicability: RestoreReplayApplicability,
        source_policy_allows_use: bool,
        restore_or_replay_verification_completed: bool,
    ) -> Result<Self, ArtifactRestoreImportGuardError> {
        match applicability {
            RestoreReplayApplicability::NotIntendedForRestoreReplay => {}
            RestoreReplayApplicability::RestoreInput
            | RestoreReplayApplicability::ReplayVerificationInput => {
                if !source_policy_allows_use {
                    return Err(ArtifactRestoreImportGuardError::RestoreReplayPolicyDenied);
                }
                if !restore_or_replay_verification_completed {
                    return Err(
                        ArtifactRestoreImportGuardError::RestoreReplayVerificationMissing,
                    );
                }
            }
        }

        Ok(Self {
            applicability,
            source_policy_allows_use,
            restore_or_replay_verification_completed,
        })
    }
}

/// export / backup artifact failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportBackupArtifactFailureKind {
    /// export or backup surface is not admitted.
    ExportSurfaceNotAllowed,
    /// artifact requires redaction before export.
    ExportRedactionRequired,
    /// artifact cannot be generated or fetched.
    BackupArtifactUnavailable,
    /// artifact integrity check fails.
    ArtifactIntegrityMismatch,
    /// artifact is used as restore/import input without admission.
    ArtifactRestoreNotAllowed,
}

impl ExportBackupArtifactFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExportSurfaceNotAllowed => "export_surface_not_allowed",
            Self::ExportRedactionRequired => "export_redaction_required",
            Self::BackupArtifactUnavailable => "backup_artifact_unavailable",
            Self::ArtifactIntegrityMismatch => "artifact_integrity_mismatch",
            Self::ArtifactRestoreNotAllowed => "artifact_restore_not_allowed",
        }
    }
}

/// export / backup artifact failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExportBackupArtifactFailure {
    kind: ExportBackupArtifactFailureKind,
    reason: CatalogedReasonRef,
}

impl ExportBackupArtifactFailure {
    /// cataloged reason に接続した artifact failure を作ります。
    pub fn from_kind(kind: ExportBackupArtifactFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("export backup artifact failure reason code must be registered");
        Self { kind, reason }
    }
}

/// export / backup artifact audit event shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExportBackupArtifactAuditShape {
    event_type_recorded: bool,
    artifact_class_recorded: bool,
    source_scope_recorded: bool,
    redaction_class_recorded: bool,
    integrity_reference_or_non_digest_reason_recorded: bool,
    retention_class_recorded: bool,
    correlation_id_recorded: bool,
    storage_or_tool_owner_recorded: bool,
}
