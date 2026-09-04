// drivers/persistence は storage、schema migration、export/backup の driver surface です。
//
// core-owned state policy を変更せず、永続化先や schema の具象処理を
// 後続 task でここに隔離します。

use arcrtc_core_ports::{
    CorePort, PersistencePort, PersistencePortFailure, PersistencePortFailureKind,
    PersistencePortInput, PersistencePortOutput, PortFamily,
};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_state::{PersistenceRule, StateClass};

/// persistence driver package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistenceDriverSurface;

/// driver が所有する concrete storage backend class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceBackendClass {
    /// PostgreSQL / SQL database.
    SqlDatabase,
    /// Redis / key-value store.
    KeyValueStore,
    /// object store.
    ObjectStore,
    /// filesystem.
    FileSystem,
    /// in-memory store.
    Memory,
}

/// storage shape が core API に漏れないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersistenceStorageShapeGuard {
    backend_class: PersistenceBackendClass,
    schema_layout_driver_owned: bool,
    storage_key_not_core_api: bool,
    migration_file_not_domain_model: bool,
    replication_not_domain_failover_proof: bool,
}

/// storage shape guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceStorageShapeError {
    /// schema/table/key/object/file layout が core API に露出しています。
    StorageLayoutExposedAsCoreApi,
    /// migration file を domain model source として扱っています。
    MigrationFileAsDomainModel,
    /// storage replication を domain failover proof として扱っています。
    StorageReplicationAsDomainFailoverProof,
}

impl PersistenceStorageShapeGuard {
    /// concrete storage shape が driver 内部に閉じていることを確認します。
    pub const fn try_new(
        backend_class: PersistenceBackendClass,
        schema_layout_driver_owned: bool,
        storage_key_not_core_api: bool,
        migration_file_not_domain_model: bool,
        replication_not_domain_failover_proof: bool,
    ) -> Result<Self, PersistenceStorageShapeError> {
        if !schema_layout_driver_owned || !storage_key_not_core_api {
            return Err(PersistenceStorageShapeError::StorageLayoutExposedAsCoreApi);
        }
        if !migration_file_not_domain_model {
            return Err(PersistenceStorageShapeError::MigrationFileAsDomainModel);
        }
        if !replication_not_domain_failover_proof {
            return Err(PersistenceStorageShapeError::StorageReplicationAsDomainFailoverProof);
        }

        Ok(Self {
            backend_class,
            schema_layout_driver_owned,
            storage_key_not_core_api,
            migration_file_not_domain_model,
            replication_not_domain_failover_proof,
        })
    }
}

/// persistence intent を driver execution に入れる直前の admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersistenceDriverAdmissionGuard {
    state_class: StateClass,
    persistence_rule: PersistenceRule,
    driver_does_not_decide_domain_semantics: bool,
    driver_transaction_not_domain_commit: bool,
}

/// persistence admission guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceDriverAdmissionError {
    /// driver が domain transition / command accept/reject を判断しています。
    DriverOwnsDomainSemantics,
    /// driver DB transaction を aggregate commit として扱っています。
    DriverTransactionAsDomainCommit,
}

impl PersistenceDriverAdmissionGuard {
    /// core state policy を保存先の都合で上書きしないことを確認します。
    pub const fn try_new(
        state_class: StateClass,
        driver_does_not_decide_domain_semantics: bool,
        driver_transaction_not_domain_commit: bool,
    ) -> Result<Self, PersistenceDriverAdmissionError> {
        if !driver_does_not_decide_domain_semantics {
            return Err(PersistenceDriverAdmissionError::DriverOwnsDomainSemantics);
        }
        if !driver_transaction_not_domain_commit {
            return Err(PersistenceDriverAdmissionError::DriverTransactionAsDomainCommit);
        }
        Ok(Self {
            state_class,
            persistence_rule: state_class.persistence_rule(),
            driver_does_not_decide_domain_semantics,
            driver_transaction_not_domain_commit,
        })
    }
}

/// driver-local retry store bound guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersistenceRetryBoundGuard {
    entry_count_bounded: bool,
    byte_size_bounded: bool,
    retry_count_bounded: bool,
    retry_duration_bounded: bool,
}

/// retry bound guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceRetryBoundError {
    /// retry store に必要な bound がありません。
    UnboundedRetryStore,
}

impl PersistenceRetryBoundGuard {
    /// retry store が count / byte / retry-count / duration の bound を持つことを確認します。
    pub const fn try_new(
        entry_count_bounded: bool,
        byte_size_bounded: bool,
        retry_count_bounded: bool,
        retry_duration_bounded: bool,
    ) -> Result<Self, PersistenceRetryBoundError> {
        if !entry_count_bounded
            || !byte_size_bounded
            || !retry_count_bounded
            || !retry_duration_bounded
        {
            return Err(PersistenceRetryBoundError::UnboundedRetryStore);
        }

        Ok(Self {
            entry_count_bounded,
            byte_size_bounded,
            retry_count_bounded,
            retry_duration_bounded,
        })
    }

    /// retry count/byte/count bound 超過時の core-owned failure です。
    pub fn bound_exceeded_failure(&self) -> PersistencePortFailure {
        PersistencePortFailure::from_kind(PersistencePortFailureKind::PersistenceRetryBoundExceeded)
    }

    /// retry duration 超過時の core-owned failure です。
    pub fn duration_exceeded_failure(&self) -> PersistencePortFailure {
        PersistencePortFailure::from_kind(
            PersistencePortFailureKind::PersistenceRetryDurationExceeded,
        )
    }
}

/// concrete persistence failure source です。公開 reason は core catalog に寄せます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceDriverFailureSource {
    /// concrete DB/object/file store unavailable.
    ConcreteStoreUnavailable,
    /// retry store count / byte / retry-count bound exceeded.
    RetryBoundExceeded,
    /// retry duration exceeded.
    RetryDurationExceeded,
    /// audit backlog exceeded while persistence participates.
    AuditBacklogExceeded,
    /// driver shutdown.
    DriverShutdown,
}

impl PersistenceDriverFailureSource {
    /// driver-local failure source を core-owned PersistencePort failure へ変換します。
    pub fn to_port_failure(self) -> PersistencePortFailure {
        let kind = match self {
            Self::ConcreteStoreUnavailable => PersistencePortFailureKind::PersistenceUnavailable,
            Self::RetryBoundExceeded => PersistencePortFailureKind::PersistenceRetryBoundExceeded,
            Self::RetryDurationExceeded => {
                PersistencePortFailureKind::PersistenceRetryDurationExceeded
            }
            Self::AuditBacklogExceeded => PersistencePortFailureKind::AuditBacklogBoundExceeded,
            Self::DriverShutdown => PersistencePortFailureKind::DriverShutdown,
        };
        PersistencePortFailure::from_kind(kind)
    }
}

/// drivers/persistence が core-owned PersistencePort を実装する marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersistenceDriverPort;

impl CorePort for PersistenceDriverPort {
    const FAMILY: PortFamily = PortFamily::Persistence;
    type Input = PersistencePortInput;
    type Output = PersistencePortOutput;
    type Error = PersistencePortFailure;
}

impl PersistencePort for PersistenceDriverPort {}

/// persistence driver 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedPersistenceDriverBehavior {
    /// storage schema becomes core domain API.
    StorageSchemaAsCoreDomainApi,
    /// driver persistence owns state transition semantics.
    DriverOwnsStateTransitionSemantics,
    /// persistence retry queue is unbounded or unaudited.
    UnboundedOrUnauditedRetryQueue,
    /// audit hash-chain meaning delegated to storage implementation.
    StorageOwnsAuditHashChainMeaning,
    /// failed persistence output is treated as a successful result.
    FailedPersistenceOutputAsSuccess,
    /// storage transaction becomes aggregate commit authority.
    StorageTransactionAsAggregateCommitAuthority,
    /// persisted artifact leaves boundary without export/backup classification.
    ArtifactLeavesWithoutExportBackupClassification,
    /// backend replication is treated as domain state ownership proof.
    BackendReplicationAsDomainOwnershipProof,
}

/// v0.2 で許可する schema / migration class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaMigrationClass {
    /// new store initialization.
    DriverSchemaInit,
    /// concrete storage shape の forward migration.
    DriverSchemaForward,
    /// concrete storage shape の rollback.
    DriverSchemaRollback,
    /// persisted representation decode compatibility.
    DriverEncodingCompat,
    /// schema unchanged.
    NoMigrationRequired,
}

/// persisted representation の unknown field handling です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnknownPersistedFieldHandling {
    /// unknown field は cataloged reason 付きで拒否します。
    RejectWithCatalogedReason,
    /// driver detail として保持するが core state には採用しません。
    PreserveAsDriverDetailOnly,
    /// 明示 rule に従って無視します。
    IgnoreByExplicitRule,
}

/// persisted representation compatibility rule です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationCompatibilityRule {
    accepted_versions: Vec<&'static str>,
    rejected_versions: Vec<&'static str>,
    canonical_format_version: Option<&'static str>,
    required_fields_declared: bool,
    optional_field_defaults_declared: bool,
    unknown_field_handling: UnknownPersistedFieldHandling,
    rollback_condition_declared: bool,
    data_loss_condition_declared: bool,
}

/// compatibility rule の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MigrationCompatibilityRuleError {
    /// accepted / rejected version が宣言されていません。
    VersionSetMissing,
    /// digest / compatibility verification に使う canonical format version が未宣言です。
    CanonicalFormatVersionMissing,
    /// required fields が未宣言です。
    RequiredFieldsMissing,
    /// optional field default behavior が未宣言です。
    OptionalFieldDefaultsMissing,
    /// rollback condition が未宣言です。
    RollbackConditionMissing,
    /// data loss condition が未宣言です。
    DataLossConditionMissing,
}

impl MigrationCompatibilityRule {
    /// compatibility verification に必要な version / field / rollback 条件を固定します。
    pub fn try_new(
        accepted_versions: Vec<&'static str>,
        rejected_versions: Vec<&'static str>,
        canonical_format_version: Option<&'static str>,
        required_fields_declared: bool,
        optional_field_defaults_declared: bool,
        unknown_field_handling: UnknownPersistedFieldHandling,
        rollback_condition_declared: bool,
        data_loss_condition_declared: bool,
        digest_or_compatibility_verification_requested: bool,
    ) -> Result<Self, MigrationCompatibilityRuleError> {
        if accepted_versions.is_empty() || rejected_versions.is_empty() {
            return Err(MigrationCompatibilityRuleError::VersionSetMissing);
        }
        if digest_or_compatibility_verification_requested && canonical_format_version.is_none() {
            return Err(MigrationCompatibilityRuleError::CanonicalFormatVersionMissing);
        }
        if !required_fields_declared {
            return Err(MigrationCompatibilityRuleError::RequiredFieldsMissing);
        }
        if !optional_field_defaults_declared {
            return Err(MigrationCompatibilityRuleError::OptionalFieldDefaultsMissing);
        }
        if !rollback_condition_declared {
            return Err(MigrationCompatibilityRuleError::RollbackConditionMissing);
        }
        if !data_loss_condition_declared {
            return Err(MigrationCompatibilityRuleError::DataLossConditionMissing);
        }

        Ok(Self {
            accepted_versions,
            rejected_versions,
            canonical_format_version,
            required_fields_declared,
            optional_field_defaults_declared,
            unknown_field_handling,
            rollback_condition_declared,
            data_loss_condition_declared,
        })
    }
}

/// migration mode selection の owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MigrationModeSelectionOwner {
    /// entrypoints startup / wiring typed configuration.
    EntrypointsTypedConfiguration,
    /// driver selected migration mode by itself; forbidden.
    DriverSelfSelection,
}

/// migration execution 前の boundary guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaMigrationExecutionGuard {
    migration_class: SchemaMigrationClass,
    mode_owner: MigrationModeSelectionOwner,
    dry_run_or_validation_available: bool,
    migration_result_not_treated_as_restore_success: bool,
}

/// migration execution guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaMigrationExecutionGuardError {
    /// migration mode が entrypoints typed configuration から選択されていません。
    MigrationModeNotSelectedByEntrypoints,
    /// dry-run / validation が可能なのに扱われていません。
    DryRunOrValidationMissing,
    /// migration success を restore / replay success として扱っています。
    MigrationSuccessAsRestoreSuccess,
}

impl SchemaMigrationExecutionGuard {
    /// schema migration が domain recovery proof に昇格しないことを確認します。
    pub const fn try_new(
        migration_class: SchemaMigrationClass,
        mode_owner: MigrationModeSelectionOwner,
        dry_run_or_validation_available: bool,
        migration_result_not_treated_as_restore_success: bool,
    ) -> Result<Self, SchemaMigrationExecutionGuardError> {
        if !matches!(
            mode_owner,
            MigrationModeSelectionOwner::EntrypointsTypedConfiguration
        ) {
            return Err(SchemaMigrationExecutionGuardError::MigrationModeNotSelectedByEntrypoints);
        }
        if !dry_run_or_validation_available
            && !matches!(migration_class, SchemaMigrationClass::NoMigrationRequired)
        {
            return Err(SchemaMigrationExecutionGuardError::DryRunOrValidationMissing);
        }
        if !migration_result_not_treated_as_restore_success {
            return Err(SchemaMigrationExecutionGuardError::MigrationSuccessAsRestoreSuccess);
        }

        Ok(Self {
            migration_class,
            mode_owner,
            dry_run_or_validation_available,
            migration_result_not_treated_as_restore_success,
        })
    }
}

/// schema migration failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaMigrationFailureKind {
    /// persistence store unavailable during migration.
    PersistenceUnavailable,
    /// runtime migration configuration missing.
    RuntimeConfigMissing,
    /// runtime migration configuration invalid.
    RuntimeConfigInvalid,
    /// persisted representation cannot decode.
    ExternalDecodeFailed,
    /// persisted representation version unsupported.
    UnsupportedDriverWireVersion,
    /// required persisted field absent.
    MissingRequiredWireField,
    /// persisted enum cannot map to core reference.
    ExternalEnumUnmapped,
    /// retry bound exceeded during migration / retry.
    PersistenceRetryBoundExceeded,
    /// retry duration exceeded.
    PersistenceRetryDurationExceeded,
    /// driver shutdown.
    DriverShutdown,
}

impl SchemaMigrationFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::UnsupportedDriverWireVersion => "unsupported_driver_wire_version",
            Self::MissingRequiredWireField => "missing_required_wire_field",
            Self::ExternalEnumUnmapped => "external_enum_unmapped",
            Self::PersistenceRetryBoundExceeded => "persistence_retry_bound_exceeded",
            Self::PersistenceRetryDurationExceeded => "persistence_retry_duration_exceeded",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// schema migration failure です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaMigrationFailure {
    kind: SchemaMigrationFailureKind,
    reason: CatalogedReasonRef,
    persistence_failure: Option<PersistencePortFailure>,
}
