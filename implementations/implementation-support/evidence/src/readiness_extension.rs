//! readiness evidence extension の型境界です。

use crate::{
    record::{ImplementationCommandClass, ImplementationEvidenceRecord},
    validation::{reject_secret_like, validate_evidence_record, EvidenceValidationError},
};

/// readiness claim の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessClaim {
    /// production readiness claim です。
    ProductionReadiness,
    /// live readiness claim です。
    LiveReadiness,
}

/// readiness authority の採用状態です。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessAdmissionState {
    /// authority が採用済みです。
    Admitted,
    /// authority が存在しません。
    Absent,
}

/// readiness extension validation のcontextです。
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReadinessValidationContext {
    /// auth provider authority の採用状態です。
    pub auth_provider_authority: ReadinessAdmissionState,
    /// persistence provider authority の採用状態です。
    pub persistence_provider_authority: ReadinessAdmissionState,
    /// live endpoint authority の採用状態です。
    pub live_endpoint_authority: ReadinessAdmissionState,
    /// production readiness report の採用状態です。
    pub production_readiness_report: ReadinessAdmissionState,
    /// public traversal authority の採用状態です。
    pub public_traversal_authority: ReadinessAdmissionState,
}

/// readiness extension を持つevidence recordです。
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReadinessEvidenceRecord {
    /// base evidence recordです。
    #[serde(flatten)]
    pub base: ImplementationEvidenceRecord,
    /// readiness gate idです。
    pub readiness_gate_id: String,
    /// readiness claim 種別です。
    pub readiness_claim: ReadinessClaim,
    /// readiness ADR referenceです。
    pub readiness_adr_ref: String,
    /// readiness Canonical referenceです。
    pub readiness_canonical_ref: String,
    /// build evidence referenceです。
    pub build_evidence_ref: Option<String>,
    /// behavior test evidence referenceです。
    pub behavior_test_evidence_ref: Option<String>,
    /// auth provider admission referenceです。
    pub auth_provider_admission_ref: Option<String>,
    /// persistence provider admission referenceです。
    pub persistence_provider_admission_ref: Option<String>,
    /// deployment profile referenceです。
    pub deployment_profile_ref: Option<String>,
    /// monitoring probe referenceです。
    pub monitoring_probe_ref: Option<String>,
    /// rollback plan referenceです。
    pub rollback_plan_ref: Option<String>,
    /// security scan referenceです。
    pub security_scan_ref: Option<String>,
    /// production readiness report referenceです。
    pub production_readiness_report_ref: Option<String>,
    /// live endpoint evidence referenceです。
    pub live_endpoint_evidence_ref: Option<String>,
    /// public traversal evidence referenceです。
    pub public_traversal_evidence_ref: Option<String>,
    /// rollback drain execution referenceです。
    pub rollback_drain_execution_ref: Option<String>,
    /// shutdown drain evidence referenceです。
    pub shutdown_drain_evidence_ref: Option<String>,
    /// restore evidence referenceです。
    pub restore_evidence_ref: Option<String>,
    /// Closed Gate Report referenceです。
    pub closed_gate_report_ref: Option<String>,
}

/// readiness evidence validation error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadinessEvidenceValidationError {
    /// base record validation errorです。
    Base(EvidenceValidationError),
    /// command class と readiness claim が一致しません。
    CommandClassClaimMismatch,
    /// readiness gate と claim が一致しません。
    ReadinessGateClaimMismatch,
    /// gate id が一覧にありません。
    GateIdNotListed,
    /// readiness authority reference が一致しません。
    ReadinessAuthorityRefMismatch,
    /// required extension reference がありません。
    MissingRequiredExtensionRef,
    /// required extension reference が空です。
    EmptyRequiredExtensionRef,
    /// 予期しない extension reference が設定されています。
    UnexpectedExtensionRefPopulated,
    /// auth provider authority が未採用です。
    AuthProviderAuthorityNotAdmitted,
    /// persistence provider authority が未採用です。
    PersistenceProviderAuthorityNotAdmitted,
    /// live endpoint authority が未採用です。
    LiveEndpointAuthorityNotAdmitted,
    /// production readiness report が未採用です。
    ProductionReadinessReportNotAdmitted,
    /// public traversal authority が未採用です。
    PublicTraversalAuthorityNotAdmitted,
    /// secret-like extension reference が含まれています。
    SecretLikeExtensionRef,
}

/// readiness extension record を検証します。
pub fn validate_readiness_evidence_record(
    record: &ReadinessEvidenceRecord,
    context: &ReadinessValidationContext,
) -> Result<(), ReadinessEvidenceValidationError> {
    validate_evidence_record(&record.base).map_err(ReadinessEvidenceValidationError::Base)?;
    validate_claim_and_command_class(record)?;
    validate_gate_listing_and_claim_prefix(record)?;
    validate_common_readiness_refs(record)?;
    validate_gate_id(record, context)?;
    Ok(())
}

fn validate_claim_and_command_class(
    record: &ReadinessEvidenceRecord,
) -> Result<(), ReadinessEvidenceValidationError> {
    match (record.readiness_claim, record.base.command_class) {
        (ReadinessClaim::ProductionReadiness, ImplementationCommandClass::ProductionReadiness)
        | (ReadinessClaim::LiveReadiness, ImplementationCommandClass::LiveReadiness) => Ok(()),
        _ => Err(ReadinessEvidenceValidationError::CommandClassClaimMismatch),
    }
}

fn validate_common_readiness_refs(
    record: &ReadinessEvidenceRecord,
) -> Result<(), ReadinessEvidenceValidationError> {
    if record.readiness_adr_ref.trim().is_empty()
        || record.readiness_canonical_ref.trim().is_empty()
    {
        return Err(ReadinessEvidenceValidationError::EmptyRequiredExtensionRef);
    }
    if record.readiness_adr_ref
        != "READINESS_CLAIM_BOUNDARY"
        || record.readiness_canonical_ref
            != "READINESS_MATRIX"
    {
        return Err(ReadinessEvidenceValidationError::ReadinessAuthorityRefMismatch);
    }

    reject_secret_like(&record.readiness_adr_ref)
        .map_err(|_| ReadinessEvidenceValidationError::SecretLikeExtensionRef)?;
    reject_secret_like(&record.readiness_canonical_ref)
        .map_err(|_| ReadinessEvidenceValidationError::SecretLikeExtensionRef)?;
    for value in optional_refs(record).into_iter().flatten() {
        if value.trim().is_empty() {
            return Err(ReadinessEvidenceValidationError::EmptyRequiredExtensionRef);
        }
        reject_secret_like(value)
            .map_err(|_| ReadinessEvidenceValidationError::SecretLikeExtensionRef)?;
    }
    Ok(())
}

fn validate_gate_id(
    record: &ReadinessEvidenceRecord,
    context: &ReadinessValidationContext,
) -> Result<(), ReadinessEvidenceValidationError> {
    match record.readiness_gate_id.as_str() {
        "PRD-001" => require_only(record, "build_evidence_ref"),
        "PRD-002" => require_only(record, "behavior_test_evidence_ref"),
        "PRD-003" => {
            if context.auth_provider_authority != ReadinessAdmissionState::Admitted {
                return Err(ReadinessEvidenceValidationError::AuthProviderAuthorityNotAdmitted);
            }
            require_only(record, "auth_provider_admission_ref")
        }
        "PRD-004" => {
            if context.persistence_provider_authority != ReadinessAdmissionState::Admitted {
                return Err(
                    ReadinessEvidenceValidationError::PersistenceProviderAuthorityNotAdmitted,
                );
            }
            require_only(record, "persistence_provider_admission_ref")
        }
        "PRD-005" => require_only(record, "deployment_profile_ref"),
        "PRD-006" => require_only(record, "monitoring_probe_ref"),
        "PRD-007" => require_only(record, "rollback_plan_ref"),
        "PRD-008" => require_only(record, "security_scan_ref"),
        "PRD-009" => require_only(record, "closed_gate_report_ref"),
        "LIVE-001" => {
            validate_live_authority(context)?;
            if context.production_readiness_report != ReadinessAdmissionState::Admitted {
                return Err(ReadinessEvidenceValidationError::ProductionReadinessReportNotAdmitted);
            }
            require_only(record, "production_readiness_report_ref")
        }
        "LIVE-002" => {
            validate_live_authority(context)?;
            require_only(record, "live_endpoint_evidence_ref")
        }
        "LIVE-003" => {
            validate_live_authority(context)?;
            if context.public_traversal_authority != ReadinessAdmissionState::Admitted {
                return Err(ReadinessEvidenceValidationError::PublicTraversalAuthorityNotAdmitted);
            }
            require_only(record, "public_traversal_evidence_ref")
        }
        "LIVE-004" => {
            validate_live_authority(context)?;
            require_only(record, "monitoring_probe_ref")
        }
        "LIVE-005" => {
            validate_live_authority(context)?;
            require_only(record, "rollback_drain_execution_ref")
        }
        "LIVE-006" => {
            validate_live_authority(context)?;
            require_only(record, "shutdown_drain_evidence_ref")
        }
        "LIVE-007" => {
            validate_live_authority(context)?;
            require_only(record, "restore_evidence_ref")
        }
        "LIVE-008" => {
            validate_live_authority(context)?;
            require_only(record, "closed_gate_report_ref")
        }
        _ => Err(ReadinessEvidenceValidationError::GateIdNotListed),
    }
}

fn validate_gate_listing_and_claim_prefix(
    record: &ReadinessEvidenceRecord,
) -> Result<(), ReadinessEvidenceValidationError> {
    let is_production_gate = matches!(
        record.readiness_gate_id.as_str(),
        "PRD-001"
            | "PRD-002"
            | "PRD-003"
            | "PRD-004"
            | "PRD-005"
            | "PRD-006"
            | "PRD-007"
            | "PRD-008"
            | "PRD-009"
    );
    let is_live_gate = matches!(
        record.readiness_gate_id.as_str(),
        "LIVE-001"
            | "LIVE-002"
            | "LIVE-003"
            | "LIVE-004"
            | "LIVE-005"
            | "LIVE-006"
            | "LIVE-007"
            | "LIVE-008"
    );

    if !is_production_gate && !is_live_gate {
        return Err(ReadinessEvidenceValidationError::GateIdNotListed);
    }
    match (record.readiness_claim, is_production_gate, is_live_gate) {
        (ReadinessClaim::ProductionReadiness, true, false)
        | (ReadinessClaim::LiveReadiness, false, true) => Ok(()),
        _ => Err(ReadinessEvidenceValidationError::ReadinessGateClaimMismatch),
    }
}

fn validate_live_authority(
    context: &ReadinessValidationContext,
) -> Result<(), ReadinessEvidenceValidationError> {
    if context.live_endpoint_authority != ReadinessAdmissionState::Admitted {
        return Err(ReadinessEvidenceValidationError::LiveEndpointAuthorityNotAdmitted);
    }
    Ok(())
}

fn require_only(
    record: &ReadinessEvidenceRecord,
    required: &str,
) -> Result<(), ReadinessEvidenceValidationError> {
    if required_ref(record, required).is_none() {
        return Err(ReadinessEvidenceValidationError::MissingRequiredExtensionRef);
    }
    for (name, value) in named_optional_refs(record) {
        if name != required && value.is_some() {
            return Err(ReadinessEvidenceValidationError::UnexpectedExtensionRefPopulated);
        }
    }
    Ok(())
}

fn required_ref<'a>(record: &'a ReadinessEvidenceRecord, name: &str) -> Option<&'a str> {
    named_optional_refs(record)
        .into_iter()
        .find_map(|(field_name, value)| (field_name == name).then_some(value).flatten())
}

fn optional_refs(record: &ReadinessEvidenceRecord) -> [Option<&str>; 16] {
    [
        record.build_evidence_ref.as_deref(),
        record.behavior_test_evidence_ref.as_deref(),
        record.auth_provider_admission_ref.as_deref(),
        record.persistence_provider_admission_ref.as_deref(),
        record.deployment_profile_ref.as_deref(),
        record.monitoring_probe_ref.as_deref(),
        record.rollback_plan_ref.as_deref(),
        record.security_scan_ref.as_deref(),
        record.production_readiness_report_ref.as_deref(),
        record.live_endpoint_evidence_ref.as_deref(),
        record.public_traversal_evidence_ref.as_deref(),
        record.rollback_drain_execution_ref.as_deref(),
        record.shutdown_drain_evidence_ref.as_deref(),
        record.restore_evidence_ref.as_deref(),
        record.closed_gate_report_ref.as_deref(),
        None,
    ]
}

fn named_optional_refs(record: &ReadinessEvidenceRecord) -> [(&'static str, Option<&str>); 15] {
    [
        ("build_evidence_ref", record.build_evidence_ref.as_deref()),
        (
            "behavior_test_evidence_ref",
            record.behavior_test_evidence_ref.as_deref(),
        ),
        (
            "auth_provider_admission_ref",
            record.auth_provider_admission_ref.as_deref(),
        ),
        (
            "persistence_provider_admission_ref",
            record.persistence_provider_admission_ref.as_deref(),
        ),
        (
            "deployment_profile_ref",
            record.deployment_profile_ref.as_deref(),
        ),
        (
            "monitoring_probe_ref",
            record.monitoring_probe_ref.as_deref(),
        ),
        ("rollback_plan_ref", record.rollback_plan_ref.as_deref()),
        ("security_scan_ref", record.security_scan_ref.as_deref()),
        (
            "production_readiness_report_ref",
            record.production_readiness_report_ref.as_deref(),
        ),
        (
            "live_endpoint_evidence_ref",
            record.live_endpoint_evidence_ref.as_deref(),
        ),
        (
            "public_traversal_evidence_ref",
            record.public_traversal_evidence_ref.as_deref(),
        ),
        (
            "rollback_drain_execution_ref",
            record.rollback_drain_execution_ref.as_deref(),
        ),
        (
            "shutdown_drain_evidence_ref",
            record.shutdown_drain_evidence_ref.as_deref(),
        ),
        (
            "restore_evidence_ref",
            record.restore_evidence_ref.as_deref(),
        ),
        (
            "closed_gate_report_ref",
            record.closed_gate_report_ref.as_deref(),
        ),
    ]
}
