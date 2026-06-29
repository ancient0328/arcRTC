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

/// migration report に必要な shape です。実測 report 本体は evidence record 側が所有します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaMigrationReportShape {
    has_correlation_or_startup_ref: bool,
    has_target_driver_and_store_class: bool,
    has_migration_class: bool,
    has_source_and_target_version: bool,
    has_canonical_format_version_when_claimed: bool,
    has_reproducible_procedure: bool,
    has_result_and_reason_for_non_success: bool,
    has_rollback_status: bool,
    has_sensitive_data_redaction_statement: bool,
}

/// migration report shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaMigrationReportShapeError {
    /// correlation ID or startup run ID がありません。
    CorrelationOrStartupRefMissing,
    /// target driver / store class がありません。
    TargetDriverStoreMissing,
    /// migration class がありません。
    MigrationClassMissing,
    /// source / target version がありません。
    VersionMissing,
    /// digest / compatibility evidence claim に必要な canonical format/version がありません。
    CanonicalFormatVersionMissing,
    /// reproducible procedure がありません。
    ReproducibleProcedureMissing,
    /// result / non-success reason がありません。
    ResultOrReasonMissing,
    /// rollback status がありません。
    RollbackStatusMissing,
    /// sensitive-data redaction statement がありません。
    RedactionStatementMissing,
}

impl SchemaMigrationReportShape {
    /// migration report を evidence として採用できる最小 shape を確認します。
    pub const fn try_new(
        has_correlation_or_startup_ref: bool,
        has_target_driver_and_store_class: bool,
        has_migration_class: bool,
        has_source_and_target_version: bool,
        has_canonical_format_version_when_claimed: bool,
        digest_or_compatibility_evidence_claimed: bool,
        has_reproducible_procedure: bool,
        has_result_and_reason_for_non_success: bool,
        has_rollback_status: bool,
        has_sensitive_data_redaction_statement: bool,
    ) -> Result<Self, SchemaMigrationReportShapeError> {
        if !has_correlation_or_startup_ref {
            return Err(SchemaMigrationReportShapeError::CorrelationOrStartupRefMissing);
        }
        if !has_target_driver_and_store_class {
            return Err(SchemaMigrationReportShapeError::TargetDriverStoreMissing);
        }
        if !has_migration_class {
            return Err(SchemaMigrationReportShapeError::MigrationClassMissing);
        }
        if !has_source_and_target_version {
            return Err(SchemaMigrationReportShapeError::VersionMissing);
        }
        if digest_or_compatibility_evidence_claimed && !has_canonical_format_version_when_claimed {
            return Err(SchemaMigrationReportShapeError::CanonicalFormatVersionMissing);
        }
        if !has_reproducible_procedure {
            return Err(SchemaMigrationReportShapeError::ReproducibleProcedureMissing);
        }
        if !has_result_and_reason_for_non_success {
            return Err(SchemaMigrationReportShapeError::ResultOrReasonMissing);
        }
        if !has_rollback_status {
            return Err(SchemaMigrationReportShapeError::RollbackStatusMissing);
        }
        if !has_sensitive_data_redaction_statement {
            return Err(SchemaMigrationReportShapeError::RedactionStatementMissing);
        }

        Ok(Self {
            has_correlation_or_startup_ref,
            has_target_driver_and_store_class,
            has_migration_class,
            has_source_and_target_version,
            has_canonical_format_version_when_claimed,
            has_reproducible_procedure,
            has_result_and_reason_for_non_success,
            has_rollback_status,
            has_sensitive_data_redaction_statement,
        })
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
    /// migration snapshot is adopted as backup/restore evidence without artifact classification.
    MigrationSnapshotAsBackupRestoreEvidence,
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
    /// rerunnable report-support material.
    EvidenceBundle,
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

/// export / backup artifact の integrity evidence class です。
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
    /// restore input 候補だが owning Canonical admission が必要です。
    RequiresRestoreCanonicalAdmission,
    /// replay verification input 候補だが owning Canonical admission が必要です。
    RequiresReplayCanonicalAdmission,
}

/// export / backup artifact admission shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExportBackupArtifactAdmission {
    artifact_class: ExportBackupArtifactClass,
    redaction_class: ArtifactRedactionClass,
    retention_class: ArtifactRetentionClass,
    integrity_class: ArtifactIntegrityClass,
    restore_replay_applicability: RestoreReplayApplicability,
    integrity_reference_or_non_digest_reason_recorded: bool,
    schema_or_format_version_applicable: bool,
    schema_or_format_version_recorded: bool,
    generation_procedure_recorded: bool,
    working_directory_or_owner_recorded: bool,
    source_scope_and_time_phase_recorded: bool,
    correlation_id_recorded: bool,
    concrete_storage_owner_recorded: bool,
    rerun_condition_recorded: bool,
}

/// artifact admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportBackupArtifactAdmissionError {
    /// generation command / procedure がありません。
    GenerationProcedureMissing,
    /// working directory or driver/tool owner がありません。
    WorkingDirectoryOrOwnerMissing,
    /// source scope / time phase がありません。
    SourceScopeTimePhaseMissing,
    /// correlation ID がありません。
    CorrelationIdMissing,
    /// concrete storage location owner がありません。
    ConcreteStorageOwnerMissing,
    /// rerun / regeneration condition がありません。
    RerunConditionMissing,
    /// integrity reference or explicit non-digest reason がありません。
    IntegrityEvidenceMissing,
    /// applicable schema / format version がありません。
    SchemaFormatVersionMissing,
}

impl ExportBackupArtifactAdmission {
    /// artifact を evidence 候補として扱うための最小 admission shape を確認します。
    pub const fn try_new(
        artifact_class: ExportBackupArtifactClass,
        redaction_class: ArtifactRedactionClass,
        retention_class: ArtifactRetentionClass,
        integrity_class: ArtifactIntegrityClass,
        restore_replay_applicability: RestoreReplayApplicability,
        integrity_reference_or_non_digest_reason_recorded: bool,
        schema_or_format_version_applicable: bool,
        schema_or_format_version_recorded: bool,
        generation_procedure_recorded: bool,
        working_directory_or_owner_recorded: bool,
        source_scope_and_time_phase_recorded: bool,
        correlation_id_recorded: bool,
        concrete_storage_owner_recorded: bool,
        rerun_condition_recorded: bool,
    ) -> Result<Self, ExportBackupArtifactAdmissionError> {
        if !generation_procedure_recorded {
            return Err(ExportBackupArtifactAdmissionError::GenerationProcedureMissing);
        }
        if !working_directory_or_owner_recorded {
            return Err(ExportBackupArtifactAdmissionError::WorkingDirectoryOrOwnerMissing);
        }
        if !source_scope_and_time_phase_recorded {
            return Err(ExportBackupArtifactAdmissionError::SourceScopeTimePhaseMissing);
        }
        if !correlation_id_recorded {
            return Err(ExportBackupArtifactAdmissionError::CorrelationIdMissing);
        }
        if !concrete_storage_owner_recorded {
            return Err(ExportBackupArtifactAdmissionError::ConcreteStorageOwnerMissing);
        }
        if !rerun_condition_recorded {
            return Err(ExportBackupArtifactAdmissionError::RerunConditionMissing);
        }
        if !integrity_reference_or_non_digest_reason_recorded {
            return Err(ExportBackupArtifactAdmissionError::IntegrityEvidenceMissing);
        }
        if schema_or_format_version_applicable && !schema_or_format_version_recorded {
            return Err(ExportBackupArtifactAdmissionError::SchemaFormatVersionMissing);
        }

        Ok(Self {
            artifact_class,
            redaction_class,
            retention_class,
            integrity_class,
            restore_replay_applicability,
            integrity_reference_or_non_digest_reason_recorded,
            schema_or_format_version_applicable,
            schema_or_format_version_recorded,
            generation_procedure_recorded,
            working_directory_or_owner_recorded,
            source_scope_and_time_phase_recorded,
            correlation_id_recorded,
            concrete_storage_owner_recorded,
            rerun_condition_recorded,
        })
    }
}

/// sensitive data を一般 export / backup evidence へ入れないための guard です。
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

/// integrity evidence class の混同を防ぐ guard です。
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
    owning_canonical_admission_recorded: bool,
    restore_or_replay_evidence_recorded: bool,
}

/// restore/import guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactRestoreImportGuardError {
    /// restore/import/replay admission がありません。
    RestoreImportAdmissionMissing,
    /// restore/replay evidence がありません。
    RestoreReplayEvidenceMissing,
}

impl ArtifactRestoreImportGuard {
    /// artifact presence と restore/import/replay admission を分離します。
    pub const fn try_new(
        applicability: RestoreReplayApplicability,
        owning_canonical_admission_recorded: bool,
        restore_or_replay_evidence_recorded: bool,
    ) -> Result<Self, ArtifactRestoreImportGuardError> {
        match applicability {
            RestoreReplayApplicability::NotIntendedForRestoreReplay => {}
            RestoreReplayApplicability::RequiresRestoreCanonicalAdmission
            | RestoreReplayApplicability::RequiresReplayCanonicalAdmission => {
                if !owning_canonical_admission_recorded {
                    return Err(ArtifactRestoreImportGuardError::RestoreImportAdmissionMissing);
                }
                if !restore_or_replay_evidence_recorded {
                    return Err(ArtifactRestoreImportGuardError::RestoreReplayEvidenceMissing);
                }
            }
        }

        Ok(Self {
            applicability,
            owning_canonical_admission_recorded,
            restore_or_replay_evidence_recorded,
        })
    }
}

/// export / backup artifact failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportBackupArtifactFailureKind {
    /// export or backup surface is not admitted.
    ExportSurfaceNotAllowed,
    /// artifact requires redaction before adoption.
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

