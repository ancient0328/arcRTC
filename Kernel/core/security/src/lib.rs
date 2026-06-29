//! core/security は authorization context と token verification 境界型を所有する surface です。
//!
//! 鍵取得、JWT library、secret rotation の具象処理は drivers/security に置き、
//! core には検証結果と通信許可判断に必要な語彙だけを配置します。

use arcrtc_core_command::{DecisionReason, TargetSurface, UseCaseOutcome};
use arcrtc_core_identity::{CorrelationId, CredentialRef};

/// core security package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreSecuritySurface;

/// v0.2 initial architecture で許可された authorization context class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorizationContextClass {
    /// token/key verification result が存在することを示します。
    VerifiedCredentialContext,
    /// Signaling room membership の join policy input です。
    ParticipantJoinContext,
    /// SFU publication authorization input です。
    PublicationPolicyContext,
    /// SFU subscription authorization input です。
    SubscriptionPolicyContext,
    /// TURN allocation/permission/relay authorization input です。
    TurnRelayPolicyContext,
    /// rate/quota/admission grouping input です。
    AdmissionPolicyContext,
    /// regulated 側だけで扱う policy input です。
    RegulatedAuthorizationContext,
}

/// authorization mapping の source class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorizationMappingSource {
    /// verified credential から mapping します。
    VerifiedCredential,
    /// external application context から mapping します。
    ExternalApplicationContext,
    /// edge trust policy が許可した proxy-derived context から mapping します。
    EdgeTrustedContext,
    /// regulated 側の context です。generic core requirement にはしません。
    RegulatedContext,
}

/// communication authorization action です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommunicationAction {
    /// Signaling join action です。
    Join,
    /// SFU publication action です。
    Publish,
    /// SFU subscription action です。
    Subscribe,
    /// TURN allocation action です。
    TurnAllocate,
    /// TURN permission action です。
    TurnPermission,
    /// TURN relay action です。
    TurnRelay,
    /// rate/quota/admission action です。
    Admission,
}

/// authorization context lifetime です。具体的な時計は core/time が所有します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorizationLifetime {
    /// single target decision に限って有効です。
    SingleDecision,
    /// credential expiry と同じ境界で有効です。
    UntilCredentialExpiry,
    /// 明示された policy label の lifetime です。
    Bounded(&'static str),
}

/// raw claims や sensitive payload の redaction rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorizationRedactionRule {
    /// raw token claims を core/audit/log/report に入れません。
    RawClaimsExcluded,
    /// sensitive authorization payload を入れません。
    SensitivePayloadExcluded,
}

/// authorization mapping declaration です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationMapping {
    source: AuthorizationMappingSource,
    context_class: AuthorizationContextClass,
    allowed_target_surfaces: Vec<TargetSurface>,
    lifetime: AuthorizationLifetime,
    redaction_rule: AuthorizationRedactionRule,
}

impl AuthorizationMapping {
    /// mapping source、context class、target surface、lifetime、redaction を固定します。
    pub fn new(
        source: AuthorizationMappingSource,
        context_class: AuthorizationContextClass,
        allowed_target_surfaces: Vec<TargetSurface>,
        lifetime: AuthorizationLifetime,
        redaction_rule: AuthorizationRedactionRule,
    ) -> Self {
        Self {
            source,
            context_class,
            allowed_target_surfaces,
            lifetime,
            redaction_rule,
        }
    }

    /// mapped authorization context class です。
    pub const fn context_class(&self) -> AuthorizationContextClass {
        self.context_class
    }

    /// authorization context lifetime です。
    pub const fn lifetime(&self) -> AuthorizationLifetime {
        self.lifetime
    }

    /// 許可された target surface 群です。
    pub fn allowed_target_surfaces(&self) -> &[TargetSurface] {
        &self.allowed_target_surfaces
    }
}

/// core policy が受け取る typed communication authorization input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationPolicyInput<ScopeRef> {
    correlation_id: CorrelationId,
    credential_ref: Option<CredentialRef>,
    context_class: AuthorizationContextClass,
    target_surface: TargetSurface,
    action: CommunicationAction,
    scope_ref: ScopeRef,
}

impl<ScopeRef> AuthorizationPolicyInput<ScopeRef> {
    /// target domain decision の前に渡す typed policy input を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        credential_ref: Option<CredentialRef>,
        context_class: AuthorizationContextClass,
        target_surface: TargetSurface,
        action: CommunicationAction,
        scope_ref: ScopeRef,
    ) -> Self {
        Self {
            correlation_id,
            credential_ref,
            context_class,
            target_surface,
            action,
            scope_ref,
        }
    }

    /// target surface です。
    pub const fn target_surface(&self) -> TargetSurface {
        self.target_surface
    }

    /// communication action です。
    pub const fn action(&self) -> CommunicationAction {
        self.action
    }
}

/// communication authorization policy output です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationPolicyDecision<Reason> {
    outcome: UseCaseOutcome,
    reason: DecisionReason<Reason>,
}

impl<Reason> AuthorizationPolicyDecision<Reason> {
    /// policy output を outcome/reason rule に従って作ります。
    pub fn new(
        outcome: UseCaseOutcome,
        reason: DecisionReason<Reason>,
    ) -> Result<Self, AuthorizationPolicyError> {
        match (outcome.requires_reason(), &reason) {
            (false, DecisionReason::Cataloged(_)) => {
                return Err(AuthorizationPolicyError::SuccessMustNotCarryReason);
            }
            (true, DecisionReason::Absent) => {
                return Err(AuthorizationPolicyError::NonSuccessRequiresReason);
            }
            _ => {}
        }
        Ok(Self { outcome, reason })
    }

    /// policy outcome です。
    pub const fn outcome(&self) -> UseCaseOutcome {
        self.outcome
    }

    /// policy reason slot です。
    pub const fn reason(&self) -> &DecisionReason<Reason> {
        &self.reason
    }
}

/// authorization context failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorizationFailureKind {
    /// required authorization context missing です。
    AuthorizationContextMissing,
    /// authorization context cannot be mapped です。
    AuthorizationContextInvalid,
    /// authorization context expired です。
    AuthorizationContextExpired,
    /// policy denies requested action です。
    AuthorizationPolicyDenied,
    /// requested authorization scope not allowed です。
    AuthorizationScopeNotAllowed,
    /// edge/proxy metadata is not trustworthy です。
    ForwardedHeaderUntrusted,
    /// client address is not trustworthy です。
    ClientAddressUntrusted,
    /// token verification failed です。
    TokenVerificationFailed,
    /// required authorization configuration missing です。
    RuntimeConfigMissing,
    /// required authorization configuration invalid です。
    RuntimeConfigInvalid,
}

impl AuthorizationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::AuthorizationContextMissing => "authorization_context_missing",
            Self::AuthorizationContextInvalid => "authorization_context_invalid",
            Self::AuthorizationContextExpired => "authorization_context_expired",
            Self::AuthorizationPolicyDenied => "authorization_policy_denied",
            Self::AuthorizationScopeNotAllowed => "authorization_scope_not_allowed",
            Self::ForwardedHeaderUntrusted => "forwarded_header_untrusted",
            Self::ClientAddressUntrusted => "client_address_untrusted",
            Self::TokenVerificationFailed => "token_verification_failed",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
        }
    }
}

/// authorization policy output shape rule 違反です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorizationPolicyError {
    /// success outcome が fake reason を持っています。
    SuccessMustNotCarryReason,
    /// non-success outcome に cataloged reason がありません。
    NonSuccessRequiresReason,
}

/// token verification が要求できる claim vocabulary です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequiredTokenClaim {
    /// issuer claim です。
    Issuer,
    /// audience claim です。
    Audience,
    /// subject claim です。protocol identity と同一視しません。
    Subject,
    /// expiry claim です。
    Expiration,
    /// not-before claim です。
    NotBefore,
    /// issued-at claim です。
    IssuedAt,
    /// key id claim/header です。
    KeyId,
}

/// issuer policy の抽象表現です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuerPolicy {
    accepted_issuers: Vec<&'static str>,
}

impl IssuerPolicy {
    /// accepted issuer set を固定します。
    pub fn new(accepted_issuers: Vec<&'static str>) -> Self {
        Self { accepted_issuers }
    }

    /// accepted issuer set です。
    pub fn accepted_issuers(&self) -> &[&'static str] {
        &self.accepted_issuers
    }
}

/// audience policy の抽象表現です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudiencePolicy {
    accepted_audiences: Vec<&'static str>,
}

impl AudiencePolicy {
    /// accepted audience set を固定します。
    pub fn new(accepted_audiences: Vec<&'static str>) -> Self {
        Self { accepted_audiences }
    }

    /// accepted audience set です。
    pub fn accepted_audiences(&self) -> &[&'static str] {
        &self.accepted_audiences
    }
}

/// accepted token algorithm policy です。cryptographic backend は driver が所有します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenAlgorithmPolicy {
    accepted_algorithms: Vec<&'static str>,
}

impl TokenAlgorithmPolicy {
    /// accepted algorithm set を固定します。
    pub fn new(accepted_algorithms: Vec<&'static str>) -> Self {
        Self {
            accepted_algorithms,
        }
    }

    /// accepted algorithm set です。
    pub fn accepted_algorithms(&self) -> &[&'static str] {
        &self.accepted_algorithms
    }
}

/// core-owned token verification request です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenVerificationRequest {
    correlation_id: CorrelationId,
    credential_ref: CredentialRef,
    required_claims: Vec<RequiredTokenClaim>,
    issuer_policy: IssuerPolicy,
    audience_policy: AudiencePolicy,
    algorithm_policy: TokenAlgorithmPolicy,
}

impl TokenVerificationRequest {
    /// token verification request を作ります。
    pub fn new(
        correlation_id: CorrelationId,
        credential_ref: CredentialRef,
        required_claims: Vec<RequiredTokenClaim>,
        issuer_policy: IssuerPolicy,
        audience_policy: AudiencePolicy,
        algorithm_policy: TokenAlgorithmPolicy,
    ) -> Self {
        Self {
            correlation_id,
            credential_ref,
            required_claims,
            issuer_policy,
            audience_policy,
            algorithm_policy,
        }
    }

    /// credential reference です。raw token secret ではありません。
    pub const fn credential_ref(&self) -> &CredentialRef {
        &self.credential_ref
    }

    /// required claim set です。
    pub fn required_claims(&self) -> &[RequiredTokenClaim] {
        &self.required_claims
    }
}

/// token expiry / not-before decision です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenTemporalDecision {
    /// token temporal claims are valid for the policy input.
    Valid,
    /// token is expired.
    Expired,
    /// token is not yet valid.
    NotYetValid,
}

/// verified credential result です。authorization success ではありません。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedCredential {
    correlation_id: CorrelationId,
    credential_ref: CredentialRef,
    temporal_decision: TokenTemporalDecision,
}

impl VerifiedCredential {
    /// verified credential を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        credential_ref: CredentialRef,
        temporal_decision: TokenTemporalDecision,
    ) -> Self {
        Self {
            correlation_id,
            credential_ref,
            temporal_decision,
        }
    }

    /// credential reference です。
    pub const fn credential_ref(&self) -> &CredentialRef {
        &self.credential_ref
    }

    /// temporal decision です。
    pub const fn temporal_decision(&self) -> TokenTemporalDecision {
        self.temporal_decision
    }
}

/// token verification result です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenVerificationResult {
    /// verification accepted. communication authorization は別 decision です。
    Accepted(VerifiedCredential),
    /// verification rejected with concrete token reason.
    Rejected(TokenVerificationFailureKind),
}

/// token verification fail-closed reason mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenVerificationFailureKind {
    /// token missing です。
    TokenMissing,
    /// token malformed です。
    TokenMalformed,
    /// signature invalid です。
    TokenSignatureInvalid,
    /// key unavailable です。
    TokenKeyUnavailable,
    /// issuer mismatch です。
    TokenIssuerMismatch,
    /// audience mismatch です。
    TokenAudienceMismatch,
    /// expired です。
    TokenExpired,
    /// not yet valid です。
    TokenNotYetValid,
    /// required claim missing です。
    TokenRequiredClaimMissing,
    /// unsupported algorithm です。
    TokenUnsupportedAlgorithm,
}

impl TokenVerificationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::TokenMissing => "token_missing",
            Self::TokenMalformed => "token_malformed",
            Self::TokenSignatureInvalid => "token_signature_invalid",
            Self::TokenKeyUnavailable => "token_key_unavailable",
            Self::TokenIssuerMismatch => "token_issuer_mismatch",
            Self::TokenAudienceMismatch => "token_audience_mismatch",
            Self::TokenExpired => "token_expired",
            Self::TokenNotYetValid => "token_not_yet_valid",
            Self::TokenRequiredClaimMissing => "token_required_claim_missing",
            Self::TokenUnsupportedAlgorithm => "token_unsupported_algorithm",
        }
    }
}
