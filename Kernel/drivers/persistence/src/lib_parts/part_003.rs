/// artifact audit shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportBackupArtifactAuditShapeError {
    /// required audit field が不足しています。
    RequiredAuditFieldMissing,
}

impl ExportBackupArtifactAuditShape {
    /// `export_backup_artifact_decision` event に必要な field を確認します。
    pub const fn try_new(
        event_type_recorded: bool,
        artifact_class_recorded: bool,
        source_scope_recorded: bool,
        redaction_class_recorded: bool,
        integrity_reference_or_non_digest_reason_recorded: bool,
        retention_class_recorded: bool,
        correlation_id_recorded: bool,
        storage_or_tool_owner_recorded: bool,
    ) -> Result<Self, ExportBackupArtifactAuditShapeError> {
        if !event_type_recorded
            || !artifact_class_recorded
            || !source_scope_recorded
            || !redaction_class_recorded
            || !integrity_reference_or_non_digest_reason_recorded
            || !retention_class_recorded
            || !correlation_id_recorded
            || !storage_or_tool_owner_recorded
        {
            return Err(ExportBackupArtifactAuditShapeError::RequiredAuditFieldMissing);
        }

        Ok(Self {
            event_type_recorded,
            artifact_class_recorded,
            source_scope_recorded,
            redaction_class_recorded,
            integrity_reference_or_non_digest_reason_recorded,
            retention_class_recorded,
            correlation_id_recorded,
            storage_or_tool_owner_recorded,
        })
    }
}

/// export / backup artifact 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedExportBackupArtifactBehavior {
    /// database dump shape becomes public API.
    DatabaseDumpShapeAsPublicApi,
    /// backup file existence is treated as restore success.
    BackupExistenceAsRestoreSuccess,
    /// storage checksum is treated as audit hash-chain proof.
    StorageChecksumAsAuditHashChainProof,
    /// raw sensitive material is exported into general evidence.
    RawSensitiveMaterialInGeneralEvidence,
    /// artifact path or bucket key becomes core domain identity.
    ArtifactPathAsCoreDomainIdentity,
    /// export format is treated as canonical serialization without explicit Canonical.
    ExportFormatAsCanonicalSerialization,
    /// release/distribution evidence is hidden under backup evidence.
    ReleaseDistributionEvidenceHiddenAsBackup,
}
