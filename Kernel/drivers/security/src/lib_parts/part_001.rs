// drivers/security は key source、verifier driver、secret rotation の driver surface です。
//
// token 発行や domain authorization semantics は所有せず、core/security が
// 定義する境界への具象接続だけを後続 task で配置します。

use arcrtc_core_ports::{CorePort, PortFamily, TokenVerifierPort};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_security::{
    TokenVerificationFailureKind, TokenVerificationRequest, TokenVerificationResult,
};

/// security driver package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityDriverSurface;

/// v0.2 initial architecture で許可する key source type です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityKeySourceType {
    /// entrypoints が供給する typed static key material configuration.
    StaticTypedKeyMaterial,
    /// driver が bounded HTTP/cache policy で扱う JWKS endpoint.
    JwksEndpoint,
    /// entrypoints が供給する file/secret-manager opaque reference.
    FileOrSecretManagerReference,
}

/// key source configuration の供給元です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeySourceConfigurationOrigin {
    /// entrypoints composition root から typed config として供給されます。
    EntrypointsTypedConfiguration,
    /// driver が環境変数を直接読む経路です。禁止します。
    DriverEnvironmentRead,
    /// driver が process args を直接読む経路です。禁止します。
    DriverProcessArgsRead,
    /// driver が暗黙の file path discovery を行う経路です。禁止します。
    DriverImplicitFileDiscovery,
}

/// key source configuration の境界 guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeySourceConfigurationGuard {
    source_type: SecurityKeySourceType,
    origin: KeySourceConfigurationOrigin,
    typed_configuration_present: bool,
    core_sees_opaque_reference_only: bool,
    raw_secret_absent_from_core: bool,
    driver_direct_env_file_args_absent: bool,
}

/// key source configuration guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeySourceConfigurationError {
    /// entrypoints typed configuration 以外から key source を読んでいます。
    ConfigurationOriginInvalid,
    /// typed verifier configuration がありません。
    TypedConfigurationMissing,
    /// core に opaque reference 以外を渡しています。
    RawKeyReferenceCrossesToCore,
    /// driver が env/file/process args を直接読んでいます。
    DriverDirectConfigurationRead,
}

impl KeySourceConfigurationGuard {
    /// key source は entrypoints wiring 済み typed config からだけ受け取ります。
    pub const fn try_new(
        source_type: SecurityKeySourceType,
        origin: KeySourceConfigurationOrigin,
        typed_configuration_present: bool,
        core_sees_opaque_reference_only: bool,
        raw_secret_absent_from_core: bool,
        driver_direct_env_file_args_absent: bool,
    ) -> Result<Self, KeySourceConfigurationError> {
        if !matches!(origin, KeySourceConfigurationOrigin::EntrypointsTypedConfiguration) {
            return Err(KeySourceConfigurationError::ConfigurationOriginInvalid);
        }
        if !typed_configuration_present {
            return Err(KeySourceConfigurationError::TypedConfigurationMissing);
        }
        if !core_sees_opaque_reference_only || !raw_secret_absent_from_core {
            return Err(KeySourceConfigurationError::RawKeyReferenceCrossesToCore);
        }
        if !driver_direct_env_file_args_absent {
            return Err(KeySourceConfigurationError::DriverDirectConfigurationRead);
        }

        Ok(Self {
            source_type,
            origin,
            typed_configuration_present,
            core_sees_opaque_reference_only,
            raw_secret_absent_from_core,
            driver_direct_env_file_args_absent,
        })
    }
}

/// key cache / refresh の必須 bound です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCacheRefreshBounds {
    maximum_keys: usize,
    maximum_key_material_bytes: usize,
    maximum_cache_age_millis: u64,
    maximum_failed_refresh_attempts: usize,
    maximum_refresh_wait_millis: u64,
}

/// key cache / refresh bound の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCacheRefreshBoundsError {
    /// required bound が 0 または未指定です。
    RequiredBoundMissing,
}

impl KeyCacheRefreshBounds {
    /// source policy が要求する 5 つの bound をすべて固定します。
    pub const fn try_new(
        maximum_keys: usize,
        maximum_key_material_bytes: usize,
        maximum_cache_age_millis: u64,
        maximum_failed_refresh_attempts: usize,
        maximum_refresh_wait_millis: u64,
    ) -> Result<Self, KeyCacheRefreshBoundsError> {
        if maximum_keys == 0
            || maximum_key_material_bytes == 0
            || maximum_cache_age_millis == 0
            || maximum_failed_refresh_attempts == 0
            || maximum_refresh_wait_millis == 0
        {
            return Err(KeyCacheRefreshBoundsError::RequiredBoundMissing);
        }

        Ok(Self {
            maximum_keys,
            maximum_key_material_bytes,
            maximum_cache_age_millis,
            maximum_failed_refresh_attempts,
            maximum_refresh_wait_millis,
        })
    }
}

/// key lookup / refresh の状態分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyLookupRefreshState {
    /// bounded cache hit.
    CacheHitFresh,
    /// cache miss で refresh が必要です。
    CacheMissRefreshRequired,
    /// stale key で refresh が必要です。
    StaleKeyRefreshRequired,
    /// refresh failure.
    RefreshFailed,
    /// key source unavailable.
    KeySourceUnavailable,
}

/// refresh 失敗時に token を fail-open で受理しないための guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyLookupRefreshGuard {
    state: KeyLookupRefreshState,
    bounds_respected: bool,
    refresh_failure_accepts_token: bool,
    key_unavailable_maps_to_token_key_unavailable: bool,
}

/// key lookup / refresh guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyLookupRefreshError {
    /// cache/refresh bound を超えています。
    RefreshBoundExceeded,
    /// refresh failure 後に token を受理しようとしています。
    RefreshFailureWouldAcceptToken,
    /// key unavailable が token_key_unavailable に写像されません。
    KeyUnavailableReasonMissing,
}

impl KeyLookupRefreshGuard {
    /// cache miss/stale/refresh failure の fail-closed 条件を確認します。
    pub const fn try_new(
        state: KeyLookupRefreshState,
        bounds_respected: bool,
        refresh_failure_accepts_token: bool,
        key_unavailable_maps_to_token_key_unavailable: bool,
    ) -> Result<Self, KeyLookupRefreshError> {
        if !bounds_respected {
            return Err(KeyLookupRefreshError::RefreshBoundExceeded);
        }
        if matches!(
            state,
            KeyLookupRefreshState::RefreshFailed | KeyLookupRefreshState::KeySourceUnavailable
        ) && refresh_failure_accepts_token
        {
            return Err(KeyLookupRefreshError::RefreshFailureWouldAcceptToken);
        }
        if matches!(
            state,
            KeyLookupRefreshState::RefreshFailed | KeyLookupRefreshState::KeySourceUnavailable
        ) && !key_unavailable_maps_to_token_key_unavailable
        {
            return Err(KeyLookupRefreshError::KeyUnavailableReasonMissing);
        }

        Ok(Self {
            state,
            bounds_respected,
            refresh_failure_accepts_token,
            key_unavailable_maps_to_token_key_unavailable,
        })
    }
}

/// verifier driver の concrete backend error class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerifierBackendFailureClass {
    /// token absent.
    TokenAbsent,
    /// parser/decode failure.
    TokenDecodeFailed,
    /// signature invalid.
    SignatureInvalid,
    /// key source unavailable or bounded lookup failed.
    KeyUnavailable,
    /// issuer mismatch.
    IssuerMismatch,
    /// audience mismatch.
    AudienceMismatch,
    /// token expired.
    TokenExpired,
    /// token not yet valid.
    TokenNotYetValid,
    /// required claim absent.
    RequiredClaimMissing,
    /// unsupported algorithm.
    UnsupportedAlgorithm,
    /// required config missing during startup/runtime wiring.
    RequiredConfigMissing,
    /// secret/key source unavailable.
    SecretUnavailable,
    /// key generation rejected by the secret-rotation policy.
    RotationPolicyRejected,
}

impl VerifierBackendFailureClass {
    /// core-owned token verification failure へ写像できる場合の分類です。
    pub const fn token_failure_kind(self) -> Option<TokenVerificationFailureKind> {
        match self {
            Self::TokenAbsent => Some(TokenVerificationFailureKind::TokenMissing),
            Self::TokenDecodeFailed => Some(TokenVerificationFailureKind::TokenMalformed),
            Self::SignatureInvalid => Some(TokenVerificationFailureKind::TokenSignatureInvalid),
            Self::KeyUnavailable => Some(TokenVerificationFailureKind::TokenKeyUnavailable),
            Self::IssuerMismatch => Some(TokenVerificationFailureKind::TokenIssuerMismatch),
            Self::AudienceMismatch => Some(TokenVerificationFailureKind::TokenAudienceMismatch),
            Self::TokenExpired => Some(TokenVerificationFailureKind::TokenExpired),
            Self::TokenNotYetValid => Some(TokenVerificationFailureKind::TokenNotYetValid),
            Self::RequiredClaimMissing => {
                Some(TokenVerificationFailureKind::TokenRequiredClaimMissing)
            }
            Self::UnsupportedAlgorithm => {
                Some(TokenVerificationFailureKind::TokenUnsupportedAlgorithm)
            }
            Self::RequiredConfigMissing
            | Self::SecretUnavailable
            | Self::RotationPolicyRejected => None,
        }
    }

    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self.token_failure_kind() {
            Some(kind) => kind.reason_code(),
            None => match self {
                Self::RequiredConfigMissing => "runtime_config_missing",
                Self::SecretUnavailable => "secret_unavailable",
                Self::RotationPolicyRejected => "secret_rotation_required",
                _ => "token_key_unavailable",
            },
        }
    }
}

/// verifier driver failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VerifierDriverFailure {
    backend_failure: VerifierBackendFailureClass,
    token_failure: Option<TokenVerificationFailureKind>,
    reason: CatalogedReasonRef,
}

impl VerifierDriverFailure {
    /// driver-local backend failure を closed reason に変換します。
    pub fn from_backend_failure(backend_failure: VerifierBackendFailureClass) -> Self {
        let token_failure = backend_failure.token_failure_kind();
        let reason = CatalogedReasonRef::from_code(backend_failure.reason_code())
            .expect("security verifier driver reason code must be registered");

        Self {
            backend_failure,
            token_failure,
            reason,
        }
    }
}

/// verifier driver が core/security semantics を奪わないことを確認する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenVerifierDriverBoundaryGuard {
    concrete_crypto_backend_driver_owned: bool,
    key_fetch_cache_refresh_driver_owned: bool,
    core_policy_left_to_core: bool,
    entrypoint_specific_role_authorization_absent: bool,
    token_issuance_absent: bool,
    external_error_mapped_to_closed_reason: bool,
    raw_material_absent_from_audit_log_metric_report: bool,
    credential_reference_only_to_core: bool,
}

/// verifier driver boundary guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenVerifierDriverBoundaryError {
    /// concrete crypto/key source ownership が driver に閉じていません。
    ConcreteImplementationEscapesDriver,
    /// core policy semantics を driver が所有しています。
    CorePolicyTakenByDriver,
    /// entrypoint-specific role authorization を混入しています。
    EntrypointSpecificAuthorizationMixed,
    /// token issuance を arcRTC responsibility として扱っています。
    TokenIssuanceMixed,
    /// external error が closed reason に写像されません。
    ExternalErrorNotMapped,
    /// raw token/key/backend detail が外部 audit/observation surface に出ます。
    RawSecurityMaterialExposed,
    /// core に raw token/key を渡しています。
    RawCredentialCrossesToCore,
}

impl TokenVerifierDriverBoundaryGuard {
    /// token verification driver の所有範囲と禁止動作を検査します。
    pub const fn try_new(
        concrete_crypto_backend_driver_owned: bool,
        key_fetch_cache_refresh_driver_owned: bool,
        core_policy_left_to_core: bool,
        entrypoint_specific_role_authorization_absent: bool,
        token_issuance_absent: bool,
        external_error_mapped_to_closed_reason: bool,
        raw_material_absent_from_audit_log_metric_report: bool,
        credential_reference_only_to_core: bool,
    ) -> Result<Self, TokenVerifierDriverBoundaryError> {
        if !concrete_crypto_backend_driver_owned || !key_fetch_cache_refresh_driver_owned {
            return Err(TokenVerifierDriverBoundaryError::ConcreteImplementationEscapesDriver);
        }
        if !core_policy_left_to_core {
            return Err(TokenVerifierDriverBoundaryError::CorePolicyTakenByDriver);
        }
        if !entrypoint_specific_role_authorization_absent {
            return Err(TokenVerifierDriverBoundaryError::EntrypointSpecificAuthorizationMixed);
        }
        if !token_issuance_absent {
            return Err(TokenVerifierDriverBoundaryError::TokenIssuanceMixed);
        }
        if !external_error_mapped_to_closed_reason {
            return Err(TokenVerifierDriverBoundaryError::ExternalErrorNotMapped);
        }
        if !raw_material_absent_from_audit_log_metric_report {
            return Err(TokenVerifierDriverBoundaryError::RawSecurityMaterialExposed);
        }
        if !credential_reference_only_to_core {
            return Err(TokenVerifierDriverBoundaryError::RawCredentialCrossesToCore);
        }

        Ok(Self {
            concrete_crypto_backend_driver_owned,
            key_fetch_cache_refresh_driver_owned,
            core_policy_left_to_core,
            entrypoint_specific_role_authorization_absent,
            token_issuance_absent,
            external_error_mapped_to_closed_reason,
            raw_material_absent_from_audit_log_metric_report,
            credential_reference_only_to_core,
        })
    }
}

/// drivers/security が core-owned TokenVerifierPort を実装する marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecurityTokenVerifierDriverPort;

impl CorePort for SecurityTokenVerifierDriverPort {
    const FAMILY: PortFamily = PortFamily::TokenVerifier;
    type Input = TokenVerificationRequest;
    type Output = TokenVerificationResult;
    type Error = TokenVerificationFailureKind;
}

impl TokenVerifierPort for SecurityTokenVerifierDriverPort {}

/// security verifier driver 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedSecurityVerifierDriverBehavior {
    /// core performs JWKS/file/secret fetch.
    CoreFetchesKeyMaterial,
    /// token is accepted when key refresh fails.
    RefreshFailureAcceptsToken,
    /// key cache has no bound.
    UnboundedKeyCache,
    /// raw token/key material is persisted or logged in an audit/observation surface.
    RawTokenOrKeyMaterialPersistedOrLogged,
    /// entrypoint-specific role authorization is mixed into verification.
    EntrypointSpecificRoleAuthorizationInVerifier,
    /// token issuance is treated as arcRTC responsibility.
    TokenIssuanceOwnedByArcRtc,
    /// driver error remains open-ended string in core decision.
    OpenEndedDriverErrorInCoreDecision,
    /// token verification success is treated as communication authorization success.
    VerificationSuccessAsAuthorizationSuccess,
}

/// secret rotation lifecycle で扱う secret class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretRotationClass {
    /// JWT/JWKS verification key material.
    JwtVerificationKey,
    /// TURN credential derivation / HMAC shared secret.
    TurnSharedSecret,
    /// TLS/mTLS/DTLS/SRTP private material classification.
    TransportCertificateKey,
    /// external secret manager/file/env source reference.
    OpaqueSecretReference,
    /// driver-owned ephemeral backend session material.
    EphemeralSessionKeyMaterial,
}

/// secret generation state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretGenerationState {
    /// new verification/issuance use where applicable is accepted.
    CurrentGeneration,
    /// accepted only for bounded verification overlap.
    PreviousGenerationOverlap,
    /// loaded but not accepted for decision.
    PendingGeneration,
    /// must not be accepted.
    RevokedGeneration,
    /// lifetime ended.
    ExpiredGeneration,
}

/// rotation policy に必須の宣言を固定する guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecretRotationPolicyGuard {
    secret_class: SecretRotationClass,
    generation_reference_format_declared: bool,
    maximum_overlap_window_millis: u64,
    revocation_behavior_declared: bool,
    active_credential_session_relation_declared: bool,
    failure_reason_declared: bool,
    audit_relation_declared: bool,
    redaction_rule_declared: bool,
}
