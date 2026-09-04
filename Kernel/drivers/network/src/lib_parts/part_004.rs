impl TurnWireOutputClass {
    /// success outcome として扱える external output class です。
    pub const fn is_success_output(self) -> bool {
        matches!(
            self,
            Self::AllocationSuccessResponse
                | Self::RefreshSuccessResponse
                | Self::PermissionSuccessResponse
                | Self::ChannelBindSuccessResponse
                | Self::RelayDataForwarding
        )
    }
}

/// TURN wire encode input の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnWireEncodeError {
    /// failure response/drop に cataloged reason がありません。
    MissingCatalogedReasonForFailure,
    /// success response に driver-created reason を載せています。
    SuccessReasonMustNotBeInvented,
    /// rejected/denied/failed decision を success output に写しています。
    FailureMappedToSuccessOutput,
    /// accepted decision を error/drop output に写しています。
    SuccessMappedToFailureOutput,
}

/// core TURN decision を external TURN response/indication へ写す前の guard です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnWireEncodeInput<ResponseModel> {
    outcome: UseCaseOutcome,
    reason: Option<CatalogedReasonRef>,
    output_class: TurnWireOutputClass,
    response_model: ResponseModel,
}

impl<ResponseModel> TurnWireEncodeInput<ResponseModel> {
    /// semantic outcome と external output class の矛盾を core response 前に止めます。
    pub fn try_new(
        outcome: UseCaseOutcome,
        reason: Option<CatalogedReasonRef>,
        output_class: TurnWireOutputClass,
        response_model: ResponseModel,
    ) -> Result<Self, TurnWireEncodeError> {
        if outcome.requires_reason() && reason.is_none() {
            return Err(TurnWireEncodeError::MissingCatalogedReasonForFailure);
        }
        if !outcome.requires_reason() && reason.is_some() {
            return Err(TurnWireEncodeError::SuccessReasonMustNotBeInvented);
        }
        if outcome.requires_reason() && output_class.is_success_output() {
            return Err(TurnWireEncodeError::FailureMappedToSuccessOutput);
        }
        if !outcome.requires_reason() && !output_class.is_success_output() {
            return Err(TurnWireEncodeError::SuccessMappedToFailureOutput);
        }

        Ok(Self {
            outcome,
            reason,
            output_class,
            response_model,
        })
    }
}

/// TURN wire driver 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedTurnWireDriverBehavior {
    /// driver parser owns TURN allocation / permission / channel state.
    DriverParserOwnsTurnLifecycleState,
    /// socket loop decides relay authorization.
    SocketLoopDecidesRelayAuthorization,
    /// wire error code replaces cataloged core reason.
    WireErrorCodeReplacesCoreReason,
    /// raw TURN/STUN attribute object crosses into core.
    RawTurnAttributeObjectCrossesCoreBoundary,
    /// malformed message enters TURN lifecycle state machine.
    MalformedMessageEntersTurnLifecycle,
    /// relay queue bound is emitted as TURN relay denial.
    RelayQueueBoundAsRelayDenial,
    /// driver issues TURN credentials.
    DriverIssuesTurnCredentials,
}

/// transport security の required mode です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityMode {
    /// TLS listener / client transport is required.
    RequiredTls,
    /// mutual TLS is required.
    RequiredMtls,
    /// DTLS/SRTP protected media transport is required.
    RequiredDtlsSrtp,
    /// edge termination with declared downstream protection is required.
    EdgeTerminatedWithDownstreamTrust,
    /// non-sensitive development-only composition. secure runtime mode には使いません。
    DevelopmentOnlyNonSensitive,
}

/// accepted protocol/profile policy の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityProfile {
    /// TLS profile.
    Tls,
    /// mTLS profile.
    Mtls,
    /// DTLS/SRTP media profile.
    DtlsSrtp,
    /// internal service identity backed profile.
    InternalServiceIdentity,
    /// edge termination plus downstream protection profile.
    EdgeTerminationDownstreamProtection,
}

/// peer verification requirement の抽象分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportPeerVerificationRequirement {
    /// peer verification must pass before command path.
    Required,
    /// internal service identity mapping must pass separately.
    RequiredWithInternalServiceIdentity,
    /// edge trust mapping must pass separately.
    RequiredWithEdgeTrustMapping,
    /// only explicit non-sensitive dev composition may omit verification.
    DevelopmentOnlyNotRequired,
}

/// concrete transport security backend の分類です。library object 自体は core に渡しません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityBackendClass {
    /// TLS backend.
    TlsBackend,
    /// mTLS backend.
    MtlsBackend,
    /// DTLS backend.
    DtlsBackend,
    /// SRTP protection backend.
    SrtpBackend,
    /// edge/proxy downstream protection driver.
    EdgeDownstreamProtection,
}

/// transport security runtime configuration の admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportSecurityConfigurationGuard {
    mode: TransportSecurityMode,
    profile: TransportSecurityProfile,
    peer_verification_requirement: TransportPeerVerificationRequirement,
    backend_class: TransportSecurityBackendClass,
    certificate_or_key_source_reference_declared: bool,
    trust_anchor_source_reference_declared: bool,
    concrete_backend_driver_owned: bool,
    all_required_backend_components_declared: bool,
    listener_binding_entrypoints_wired: bool,
    driver_timeout_or_bound_declared: bool,
    rotation_source_and_reload_bounds_declared_when_supported: bool,
    driver_runtime_config_does_not_redefine_core_policy: bool,
}

/// transport security configuration guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityConfigurationError {
    /// required certificate/key source reference がありません。
    CertificateOrKeySourceMissing,
    /// trust anchor source reference がありません。
    TrustAnchorSourceMissing,
    /// concrete backend が driver に閉じていません。
    ConcreteBackendEscapesDriver,
    /// required mode と accepted profile が一致しません。
    ModeProfileMismatch,
    /// required mode と peer verification requirement が一致しません。
    ModePeerVerificationMismatch,
    /// required mode と concrete backend class が一致しません。
    ModeBackendMismatch,
    /// listener binding は entrypoints wiring でなく driver policy になっています。
    ListenerBindingOwnerInvalid,
    /// driver-local timeout/bound が未宣言です。
    DriverBoundMissing,
    /// rotation source/reload bound が未宣言です。
    RotationReloadBoundMissing,
    /// driver runtime configuration が core security policy を再定義しています。
    DriverRedefinesCorePolicy,
    /// development-only mode を sensitive/production path に使っています。
    DevelopmentOnlyModeMisused,
}

impl TransportSecurityConfigurationGuard {
    /// entrypoints typed configuration と driver runtime configuration の境界を検査します。
    pub const fn try_new(
        mode: TransportSecurityMode,
        profile: TransportSecurityProfile,
        peer_verification_requirement: TransportPeerVerificationRequirement,
        backend_class: TransportSecurityBackendClass,
        certificate_or_key_source_reference_declared: bool,
        trust_anchor_source_reference_declared: bool,
        concrete_backend_driver_owned: bool,
        all_required_backend_components_declared: bool,
        listener_binding_entrypoints_wired: bool,
        driver_timeout_or_bound_declared: bool,
        rotation_source_and_reload_bounds_declared_when_supported: bool,
        driver_runtime_config_does_not_redefine_core_policy: bool,
    ) -> Result<Self, TransportSecurityConfigurationError> {
        if matches!(mode, TransportSecurityMode::DevelopmentOnlyNonSensitive)
            && !matches!(
                peer_verification_requirement,
                TransportPeerVerificationRequirement::DevelopmentOnlyNotRequired
            )
        {
            return Err(TransportSecurityConfigurationError::DevelopmentOnlyModeMisused);
        }
        if !mode.accepts_profile(profile) {
            return Err(TransportSecurityConfigurationError::ModeProfileMismatch);
        }
        if !mode.accepts_peer_verification(peer_verification_requirement) {
            return Err(TransportSecurityConfigurationError::ModePeerVerificationMismatch);
        }
        if !mode.accepts_backend_class(backend_class) || !all_required_backend_components_declared {
            return Err(TransportSecurityConfigurationError::ModeBackendMismatch);
        }
        if !matches!(mode, TransportSecurityMode::DevelopmentOnlyNonSensitive)
            && !certificate_or_key_source_reference_declared
        {
            return Err(TransportSecurityConfigurationError::CertificateOrKeySourceMissing);
        }
        if matches!(
            profile,
            TransportSecurityProfile::Mtls
                | TransportSecurityProfile::InternalServiceIdentity
                | TransportSecurityProfile::EdgeTerminationDownstreamProtection
        ) && !trust_anchor_source_reference_declared
        {
            return Err(TransportSecurityConfigurationError::TrustAnchorSourceMissing);
        }
        if !concrete_backend_driver_owned {
            return Err(TransportSecurityConfigurationError::ConcreteBackendEscapesDriver);
        }
        if !listener_binding_entrypoints_wired {
            return Err(TransportSecurityConfigurationError::ListenerBindingOwnerInvalid);
        }
        if !driver_timeout_or_bound_declared {
            return Err(TransportSecurityConfigurationError::DriverBoundMissing);
        }
        if !rotation_source_and_reload_bounds_declared_when_supported {
            return Err(TransportSecurityConfigurationError::RotationReloadBoundMissing);
        }
        if !driver_runtime_config_does_not_redefine_core_policy {
            return Err(TransportSecurityConfigurationError::DriverRedefinesCorePolicy);
        }

        Ok(Self {
            mode,
            profile,
            peer_verification_requirement,
            backend_class,
            certificate_or_key_source_reference_declared,
            trust_anchor_source_reference_declared,
            concrete_backend_driver_owned,
            all_required_backend_components_declared,
            listener_binding_entrypoints_wired,
            driver_timeout_or_bound_declared,
            rotation_source_and_reload_bounds_declared_when_supported,
            driver_runtime_config_does_not_redefine_core_policy,
        })
    }
}

impl TransportSecurityMode {
    /// mode と accepted profile の互換行列です。
    pub const fn accepts_profile(self, profile: TransportSecurityProfile) -> bool {
        match self {
            Self::RequiredTls => matches!(profile, TransportSecurityProfile::Tls),
            Self::RequiredMtls => matches!(
                profile,
                TransportSecurityProfile::Mtls | TransportSecurityProfile::InternalServiceIdentity
            ),
            Self::RequiredDtlsSrtp => matches!(profile, TransportSecurityProfile::DtlsSrtp),
            Self::EdgeTerminatedWithDownstreamTrust => matches!(
                profile,
                TransportSecurityProfile::EdgeTerminationDownstreamProtection
            ),
            Self::DevelopmentOnlyNonSensitive => true,
        }
    }

    /// mode と peer verification requirement の互換行列です。
    pub const fn accepts_peer_verification(
        self,
        requirement: TransportPeerVerificationRequirement,
    ) -> bool {
        match self {
            Self::RequiredTls => {
                matches!(requirement, TransportPeerVerificationRequirement::Required)
            }
            Self::RequiredMtls => matches!(
                requirement,
                TransportPeerVerificationRequirement::Required
                    | TransportPeerVerificationRequirement::RequiredWithInternalServiceIdentity
            ),
            Self::RequiredDtlsSrtp => {
                matches!(requirement, TransportPeerVerificationRequirement::Required)
            }
            Self::EdgeTerminatedWithDownstreamTrust => matches!(
                requirement,
                TransportPeerVerificationRequirement::RequiredWithEdgeTrustMapping
            ),
            Self::DevelopmentOnlyNonSensitive => matches!(
                requirement,
                TransportPeerVerificationRequirement::DevelopmentOnlyNotRequired
            ),
        }
    }

    /// mode と concrete backend class の互換行列です。
    pub const fn accepts_backend_class(self, backend_class: TransportSecurityBackendClass) -> bool {
        match self {
            Self::RequiredTls => matches!(backend_class, TransportSecurityBackendClass::TlsBackend),
            Self::RequiredMtls => {
                matches!(backend_class, TransportSecurityBackendClass::MtlsBackend)
            }
            Self::RequiredDtlsSrtp => matches!(
                backend_class,
                TransportSecurityBackendClass::DtlsBackend
                    | TransportSecurityBackendClass::SrtpBackend
            ),
            Self::EdgeTerminatedWithDownstreamTrust => matches!(
                backend_class,
                TransportSecurityBackendClass::EdgeDownstreamProtection
            ),
            Self::DevelopmentOnlyNonSensitive => true,
        }
    }
}

/// raw transport secret を外へ出さないための guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportSecuritySecretHandlingGuard {
    raw_transport_secret_material_absent_from_core_state: bool,
    raw_transport_secret_material_absent_from_audit_log_trace: bool,
    raw_transport_secret_material_absent_from_metric_label: bool,
    raw_transport_secret_material_absent_from_sdk_public_error: bool,
    raw_transport_secret_material_absent_from_diagnostic_export: bool,
    opaque_secret_source_reference_used: bool,
    fingerprint_or_hash_policy_allows_export: bool,
}

/// transport secret handling guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecuritySecretHandlingError {
    /// raw transport secret が core state に入ります。
    RawSecretCrossesToCore,
    /// raw transport secret が audit/log/trace/metric に入ります。
    RawSecretInObservability,
    /// raw transport secret が SDK public error に入ります。
    RawSecretInSdkPublicError,
    /// raw transport secret が diagnostic export に入ります。
    RawSecretInDiagnosticExport,
    /// opaque reference または policy-approved fingerprint/hash ではありません。
    RedactedReferenceMissing,
}

impl TransportSecuritySecretHandlingGuard {
    /// raw certificate/private key/shared secret/token を境界外へ出さないことを確認します。
    pub const fn try_new(
        raw_transport_secret_material_absent_from_core_state: bool,
        raw_transport_secret_material_absent_from_audit_log_trace: bool,
        raw_transport_secret_material_absent_from_metric_label: bool,
        raw_transport_secret_material_absent_from_sdk_public_error: bool,
        raw_transport_secret_material_absent_from_diagnostic_export: bool,
        opaque_secret_source_reference_used: bool,
        fingerprint_or_hash_policy_allows_export: bool,
    ) -> Result<Self, TransportSecuritySecretHandlingError> {
        if !raw_transport_secret_material_absent_from_core_state {
            return Err(TransportSecuritySecretHandlingError::RawSecretCrossesToCore);
        }
        if !raw_transport_secret_material_absent_from_audit_log_trace
            || !raw_transport_secret_material_absent_from_metric_label
        {
            return Err(TransportSecuritySecretHandlingError::RawSecretInObservability);
        }
        if !raw_transport_secret_material_absent_from_sdk_public_error {
            return Err(TransportSecuritySecretHandlingError::RawSecretInSdkPublicError);
        }
        if !raw_transport_secret_material_absent_from_diagnostic_export {
            return Err(TransportSecuritySecretHandlingError::RawSecretInDiagnosticExport);
        }
        if !opaque_secret_source_reference_used || !fingerprint_or_hash_policy_allows_export
        {
            return Err(TransportSecuritySecretHandlingError::RedactedReferenceMissing);
        }

        Ok(Self {
            raw_transport_secret_material_absent_from_core_state,
            raw_transport_secret_material_absent_from_audit_log_trace,
            raw_transport_secret_material_absent_from_metric_label,
            raw_transport_secret_material_absent_from_sdk_public_error,
            raw_transport_secret_material_absent_from_diagnostic_export,
            opaque_secret_source_reference_used,
            fingerprint_or_hash_policy_allows_export,
        })
    }
}

/// transport security startup/path failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityFailureKind {
    /// required runtime security configuration missing.
    RuntimeConfigMissing,
    /// runtime security configuration invalid.
    RuntimeConfigInvalid,
    /// required secret/key source unavailable.
    SecretUnavailable,
    /// required secret rotation state unavailable.
    SecretRotationStateUnavailable,
    /// transport secret/key generation revoked.
    SecretKeyRevoked,
    /// transport/media contract version unsupported.
    UnsupportedMediaContractVersion,
    /// network receive failed after secure setup attempt.
    NetworkReceiveFailed,
    /// network send failed after secure setup attempt.
    NetworkSendFailed,
    /// driver shutdown.
    DriverShutdown,
}

impl TransportSecurityFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::SecretUnavailable => "secret_unavailable",
            Self::SecretRotationStateUnavailable => "secret_rotation_state_unavailable",
            Self::SecretKeyRevoked => "secret_key_revoked",
            Self::UnsupportedMediaContractVersion => "unsupported_media_contract_version",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// transport security failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportSecurityFailure {
    kind: TransportSecurityFailureKind,
    reason: CatalogedReasonRef,
}

impl TransportSecurityFailure {
    /// transport security failure を cataloged reason に接続します。
    pub fn from_kind(kind: TransportSecurityFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("transport security failure reason code must be registered");
        Self { kind, reason }
    }
}

/// secure transport path の fail-closed guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportSecurityPathGuard {
    required_secure_mode: bool,
    secure_transport_setup_attempted: bool,
    setup_or_path_failure: bool,
    insecure_fallback_absent: bool,
    failure_mapped_to_cataloged_reason: bool,
    development_only_non_sensitive_policy_enabled: bool,
    development_transport_not_used_as_secure_runtime: bool,
}

/// secure transport path guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityPathError {
    /// required secure mode で setup attempt がありません。
    SecureSetupMissing,
    /// secure path failure 後に insecure fallback しています。
    InsecureFallback,
    /// secure path failure が cataloged reason に写像されません。
    FailureReasonMissing,
    /// development-only composition の制約がありません。
    DevelopmentOnlyBoundaryMissing,
}
