// core/reason は closed reason category と reason code の core surface です。
//
// driver や entrypoint が独自 reason vocabulary を作らないよう、core-owned
// reason catalog をこの package に集約します。

/// core reason package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreReasonSurface;

/// closed reason category です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasonCategory {
    /// syntax or shape is invalid.
    MalformedInput,
    /// protocol version is not accepted.
    UnsupportedVersion,
    /// verification failed or required authorization absent.
    Unauthorized,
    /// command is invalid for current state.
    ForbiddenState,
    /// idempotency rule rejects duplicate.
    Duplicate,
    /// command/event order is invalid.
    OrderingViolation,
    /// lifetime or deadline exceeded.
    Expired,
    /// bounded resource limit reached.
    ResourceExhausted,
    /// pressure policy delays, suppresses, drops, degrades, closes, or rejects recovery.
    Backpressure,
    /// quality policy rejects, suppresses, degrades, or rejects recovery.
    QualityViolation,
    /// external implementation failure after conversion.
    DriverFailure,
    /// lifecycle shutdown or cancellation prevents action.
    Shutdown,
}

impl ReasonCategory {
    /// canonical category code です。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MalformedInput => "malformed_input",
            Self::UnsupportedVersion => "unsupported_version",
            Self::Unauthorized => "unauthorized",
            Self::ForbiddenState => "forbidden_state",
            Self::Duplicate => "duplicate",
            Self::OrderingViolation => "ordering_violation",
            Self::Expired => "expired",
            Self::ResourceExhausted => "resource_exhausted",
            Self::Backpressure => "backpressure",
            Self::QualityViolation => "quality_violation",
            Self::DriverFailure => "driver_failure",
            Self::Shutdown => "shutdown",
        }
    }

    /// category default metadata です。code-specific override は `ReasonDefinition::metadata` が適用します。
    pub const fn default_metadata(self) -> ReasonMetadata {
        match self {
            Self::MalformedInput => ReasonMetadata::new(false, true, true),
            Self::UnsupportedVersion => ReasonMetadata::new(false, true, true),
            Self::Unauthorized => ReasonMetadata::new(false, false, true),
            Self::ForbiddenState => ReasonMetadata::new(false, true, true),
            Self::Duplicate => ReasonMetadata::new(false, true, false),
            Self::OrderingViolation => ReasonMetadata::new(false, true, true),
            Self::Expired => ReasonMetadata::new(false, true, true),
            Self::ResourceExhausted => ReasonMetadata::new(true, true, true),
            Self::Backpressure => ReasonMetadata::new(true, true, true),
            Self::QualityViolation => ReasonMetadata::new(true, true, true),
            Self::DriverFailure => ReasonMetadata::new(true, false, true),
            Self::Shutdown => ReasonMetadata::new(false, true, true),
        }
    }
}

/// reason metadata です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReasonMetadata {
    retryable: bool,
    safe_to_expose: bool,
    audit_required: bool,
}

impl ReasonMetadata {
    /// metadata triple を作ります。
    pub const fn new(retryable: bool, safe_to_expose: bool, audit_required: bool) -> Self {
        Self {
            retryable,
            safe_to_expose,
            audit_required,
        }
    }

    /// client / caller が retry 可能かどうかです。
    pub const fn retryable(&self) -> bool {
        self.retryable
    }

    /// external surface へ安全に露出できるかどうかです。
    pub const fn safe_to_expose(&self) -> bool {
        self.safe_to_expose
    }

    /// audit event が必要かどうかです。
    pub const fn audit_required(&self) -> bool {
        self.audit_required
    }
}

/// closed catalog に登録された reason code です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReasonCode(&'static str);

impl ReasonCode {
    /// canonical reason code string です。
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// closed reason catalog の 1 定義です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReasonDefinition {
    code: ReasonCode,
    category: ReasonCategory,
}

impl ReasonDefinition {
    /// reason definition を作ります。
    pub const fn new(code: &'static str, category: ReasonCategory) -> Self {
        Self {
            code: ReasonCode(code),
            category,
        }
    }

    /// closed reason code です。
    pub const fn code(&self) -> ReasonCode {
        self.code
    }

    /// closed reason category です。
    pub const fn category(&self) -> ReasonCategory {
        self.category
    }

    /// category default と code-specific override を反映した metadata です。
    pub fn metadata(&self) -> ReasonMetadata {
        reason_metadata(self.code, self.category)
    }
}

/// decision / audit / SDK mapping が参照する closed reason です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason<Details> {
    definition: &'static ReasonDefinition,
    details: Option<Details>,
}

impl<Details> Reason<Details> {
    /// closed definition と任意の非 authoritative details を束ねます。
    pub const fn new(definition: &'static ReasonDefinition, details: Option<Details>) -> Self {
        Self {
            definition,
            details,
        }
    }

    /// closed reason definition です。
    pub const fn definition(&self) -> &'static ReasonDefinition {
        self.definition
    }

    /// 補足 details です。decision の正として使ってはいけません。
    pub const fn details(&self) -> Option<&Details> {
        self.details.as_ref()
    }
}

/// closed source catalog に登録された reason definition だけを検索します。
pub fn find_reason_definition(code: &str) -> Option<&'static ReasonDefinition> {
    REASON_DEFINITIONS
        .iter()
        .find(|definition| definition.code().as_str() == code)
}

/// closed catalog に登録済みであることを保証した reason reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CatalogedReasonRef {
    definition: &'static ReasonDefinition,
}

/// cataloged reason lookup error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogedReasonLookupError {
    /// reason catalog に存在しない code です。
    NotCataloged,
}

impl CatalogedReasonRef {
    /// reason code から cataloged reason reference を作ります。
    pub fn from_code(code: &str) -> Result<Self, CatalogedReasonLookupError> {
        let definition =
            find_reason_definition(code).ok_or(CatalogedReasonLookupError::NotCataloged)?;
        Ok(Self { definition })
    }

    /// ReasonDefinition から cataloged table 上の definition へ正規化します。
    pub fn from_definition(
        definition: &'static ReasonDefinition,
    ) -> Result<Self, CatalogedReasonLookupError> {
        Self::from_code(definition.code().as_str())
    }

    /// cataloged reason definition です。
    pub const fn definition(self) -> &'static ReasonDefinition {
        self.definition
    }

    /// cataloged reason metadata です。
    pub fn metadata(self) -> ReasonMetadata {
        self.definition.metadata()
    }
}

fn reason_metadata(code: ReasonCode, category: ReasonCategory) -> ReasonMetadata {
    match code.as_str() {
        "token_key_unavailable" => ReasonMetadata::new(true, false, true),
        "external_type_leak_blocked" => ReasonMetadata::new(false, false, true),
        "external_encode_failed" => ReasonMetadata::new(false, false, true),
        "buffer_release_failed" => ReasonMetadata::new(false, false, true),
        "core_policy_config_invalid" => ReasonMetadata::new(false, true, true),
        "runtime_config_missing" => ReasonMetadata::new(false, true, true),
        "runtime_config_invalid" => ReasonMetadata::new(false, true, true),
        "secret_unavailable" => ReasonMetadata::new(true, false, true),
        "operation_cancelled" => ReasonMetadata::new(false, true, true),
        "capability_not_enabled" => ReasonMetadata::new(false, true, true),
        "process_panic_detected" => ReasonMetadata::new(false, true, true),
        "process_crash_detected" => ReasonMetadata::new(false, true, true),
        "canonical_serialization_failed" => ReasonMetadata::new(false, false, true),
        "canonical_serialization_mismatch" => ReasonMetadata::new(false, true, true),
        "authorization_context_invalid" => ReasonMetadata::new(false, false, true),
        "authorization_policy_denied" => ReasonMetadata::new(false, false, true),
        "authorization_scope_not_allowed" => ReasonMetadata::new(false, false, true),
        "response_replay_not_available" => ReasonMetadata::new(false, false, true),
        "secret_rotation_state_unavailable" => ReasonMetadata::new(true, false, true),
        "secret_key_revoked" => ReasonMetadata::new(false, false, true),
        "dependency_policy_violation" => ReasonMetadata::new(false, true, true),
        "vulnerability_gate_failed" => ReasonMetadata::new(false, false, true),
        "dependency_missing" => ReasonMetadata::new(false, true, true),
        "sdk_contract_drift_detected" => ReasonMetadata::new(false, true, true),
        "internal_control_authorization_denied" => ReasonMetadata::new(false, false, true),
        "ice_candidate_redaction_required" => ReasonMetadata::new(false, false, true),
        "secure_media_key_state_invalid" => ReasonMetadata::new(false, false, true),
        "operator_credential_invalid" => ReasonMetadata::new(false, false, true),
        "operator_action_denied" => ReasonMetadata::new(false, false, true),
        "public_endpoint_auth_required" => ReasonMetadata::new(false, false, true),
        "export_redaction_required" => ReasonMetadata::new(false, false, true),
        "backup_artifact_unavailable" => ReasonMetadata::new(true, false, true),
        "artifact_integrity_mismatch" => ReasonMetadata::new(false, true, true),
        "release_artifact_provenance_missing" => ReasonMetadata::new(false, true, true),
        "release_artifact_integrity_failed" => ReasonMetadata::new(false, true, true),
        "time_source_untrusted" => ReasonMetadata::new(false, true, true),
        "time_sync_unavailable" => ReasonMetadata::new(true, false, true),
        "timestamp_order_untrusted" => ReasonMetadata::new(false, true, true),
        "forwarded_header_untrusted" => ReasonMetadata::new(false, false, true),
        "tls_termination_boundary_invalid" => ReasonMetadata::new(false, false, true),
        "public_internal_route_confusion" => ReasonMetadata::new(false, false, true),
        "runtime_reconfiguration_rollback_failed" => ReasonMetadata::new(true, false, true),
        "packet_rewrite_owner_violation" => ReasonMetadata::new(false, false, true),
        "payload_transform_failed" => ReasonMetadata::new(true, false, true),
        "service_endpoint_scope_conflict" => ReasonMetadata::new(false, false, true),
        "state_owner_conflict" => ReasonMetadata::new(false, false, true),
        "split_brain_risk_detected" => ReasonMetadata::new(false, false, true),
        "runtime_task_owner_violation" => ReasonMetadata::new(false, false, true),
        "runtime_task_panic_detected" => ReasonMetadata::new(false, true, true),
        "internal_service_identity_invalid" => ReasonMetadata::new(false, false, true),
        "internal_service_identity_untrusted" => ReasonMetadata::new(false, false, true),
        "internal_service_identity_scope_conflict" => ReasonMetadata::new(false, false, true),
        "cross_plane_binding_invalid" => ReasonMetadata::new(false, false, true),
        "cross_plane_binding_scope_conflict" => ReasonMetadata::new(false, false, true),
        _ => category.default_metadata(),
    }
}
