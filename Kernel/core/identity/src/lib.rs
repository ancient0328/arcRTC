//! core/identity は opaque reference と correlation ID の core surface です。
//!
//! 外部 platform identity を直接採用せず、driver 境界で変換された
//! core-owned reference だけを扱います。

/// core identity package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreIdentitySurface;

/// core が所有する opaque reference の種類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityKind {
    /// request / event trace の参照です。
    CorrelationId,
    /// Signaling room の opaque reference です。
    RoomId,
    /// communication session の opaque reference です。
    SessionId,
    /// room membership の opaque reference です。
    ParticipantId,
    /// SFU endpoint の opaque reference です。
    EndpointId,
    /// media stream の opaque reference です。
    StreamId,
    /// packet decision correlation の参照です。
    PacketId,
    /// SFU routing decision の参照です。
    RouteId,
    /// TURN allocation の参照です。
    AllocationId,
    /// TURN permission の参照です。
    PermissionId,
    /// TURN channel binding の参照です。
    ChannelBindId,
    /// audit event の参照です。
    AuditEventId,
    /// raw secret を含まない credential reference です。
    CredentialRef,
    /// startup / wiring attempt の参照です。
    StartupRunId,
    /// configuration validation scope の参照です。
    ConfigurationScopeRef,
}

impl IdentityKind {
    /// reference が stable であるべき lifecycle です。
    pub const fn lifetime(self) -> IdentityLifetime {
        match self {
            Self::CorrelationId => IdentityLifetime::SingleCommandEventChain,
            Self::RoomId => IdentityLifetime::RoomLifecycle,
            Self::SessionId => IdentityLifetime::TransportSessionLifecycle,
            Self::ParticipantId => IdentityLifetime::RoomMembershipLifecycle,
            Self::EndpointId => IdentityLifetime::SfuEndpointLifecycle,
            Self::StreamId => IdentityLifetime::MediaStreamLifecycle,
            Self::PacketId => IdentityLifetime::PacketLifecycle,
            Self::RouteId => IdentityLifetime::RouteDecisionLifecycle,
            Self::AllocationId => IdentityLifetime::TurnAllocationLifecycle,
            Self::PermissionId => IdentityLifetime::TurnPermissionLifecycle,
            Self::ChannelBindId => IdentityLifetime::TurnChannelBindLifecycle,
            Self::AuditEventId => IdentityLifetime::AuditEventLifecycle,
            Self::CredentialRef => IdentityLifetime::CredentialVerificationLifecycle,
            Self::StartupRunId => IdentityLifetime::SingleStartupWiringAttempt,
            Self::ConfigurationScopeRef => IdentityLifetime::ConfigurationValidationLifecycle,
        }
    }

    /// public contract へ露出できる範囲です。
    pub const fn external_exposure(self) -> ExternalExposure {
        match self {
            Self::CorrelationId | Self::RoomId | Self::SessionId | Self::ParticipantId => {
                ExternalExposure::PublicOpaque
            }
            Self::EndpointId | Self::StreamId => ExternalExposure::ControlledExternal,
            Self::PacketId
            | Self::RouteId
            | Self::AllocationId
            | Self::PermissionId
            | Self::ChannelBindId => ExternalExposure::InternalByDefault,
            Self::AuditEventId | Self::StartupRunId | Self::ConfigurationScopeRef => {
                ExternalExposure::AuditOnly
            }
            Self::CredentialRef => ExternalExposure::NoRawSecretExposure,
        }
    }
}

/// reference の stability lifetime です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityLifetime {
    /// single command / event chain の中だけで stable です。
    SingleCommandEventChain,
    /// room lifecycle の中だけで stable です。
    RoomLifecycle,
    /// transport session lifecycle の中だけで stable です。
    TransportSessionLifecycle,
    /// room membership lifecycle の中だけで stable です。
    RoomMembershipLifecycle,
    /// SFU endpoint lifecycle の中だけで stable です。
    SfuEndpointLifecycle,
    /// media stream lifecycle の中だけで stable です。
    MediaStreamLifecycle,
    /// packet lifecycle の中だけで stable です。
    PacketLifecycle,
    /// route decision lifecycle の中だけで stable です。
    RouteDecisionLifecycle,
    /// TURN allocation lifecycle の中だけで stable です。
    TurnAllocationLifecycle,
    /// TURN permission lifecycle の中だけで stable です。
    TurnPermissionLifecycle,
    /// TURN channel bind lifecycle の中だけで stable です。
    TurnChannelBindLifecycle,
    /// audit event lifecycle の中だけで stable です。
    AuditEventLifecycle,
    /// credential verification lifecycle の中だけで stable です。
    CredentialVerificationLifecycle,
    /// single startup / wiring attempt の中だけで stable です。
    SingleStartupWiringAttempt,
    /// configuration validation lifecycle の中だけで stable です。
    ConfigurationValidationLifecycle,
}

/// opaque reference の外部露出範囲です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalExposure {
    /// public contract に opaque value として露出できます。
    PublicOpaque,
    /// 明示 contract がある場合に限り制御された形で露出できます。
    ControlledExternal,
    /// default では public に出しません。
    InternalByDefault,
    /// audit surface に限って露出できます。
    AuditOnly,
    /// raw secret として露出してはいけません。
    NoRawSecretExposure,
}

/// accepted core reference がどの authority から成立したかを示します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceAuthority {
    /// core policy が発行した reference です。
    CorePolicy,
    /// untrusted input を core validation 後に受理した reference です。
    CoreValidatedUntrustedInput,
    /// external credential から driver conversion された credential reference です。
    DriverCredentialConversion,
    /// entrypoints 由来の startup reference を core validation 後に受理した reference です。
    CoreValidatedStartupInput,
    /// entrypoints 由来の configuration scope を core validation 後に受理した reference です。
    CoreValidatedConfigurationInput,
}

/// driver / entrypoints / caller から入る、まだ trusted core identity ではない参照です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UntrustedReference {
    value: String,
}

impl UntrustedReference {
    /// 外部由来の文字列を trusted identity にせず保持します。
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// core validation 前の値です。
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// core validation または core policy を通過した opaque reference value です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpaqueReference {
    value: String,
    authority: ReferenceAuthority,
}

impl OpaqueReference {
    /// accepted core reference を生成します。
    pub fn accept(
        value: impl Into<String>,
        authority: ReferenceAuthority,
    ) -> Result<Self, OpaqueReferenceError> {
        let value = value.into();
        if value.is_empty() {
            return Err(OpaqueReferenceError::Empty);
        }
        if value.chars().any(char::is_control) {
            return Err(OpaqueReferenceError::ControlCharacter);
        }

        Ok(Self { value, authority })
    }

    /// core validation 後に外部由来 reference を accepted reference へ変換します。
    pub fn accept_untrusted(
        reference: UntrustedReference,
        authority: ReferenceAuthority,
    ) -> Result<Self, OpaqueReferenceError> {
        Self::accept(reference.value, authority)
    }

    /// opaque value です。ここから user identity や role を推測してはいけません。
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// accepted reference の成立 authority です。
    pub const fn authority(&self) -> ReferenceAuthority {
        self.authority
    }
}

/// opaque reference として採用できない入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpaqueReferenceError {
    /// 空文字列は stable reference として扱えません。
    Empty,
    /// 制御文字を含む値は opaque reference として扱えません。
    ControlCharacter,
}

/// core identity と同一視してはならない external identity source です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalIdentitySource {
    /// application user identity です。
    ApplicationUser,
    /// token subject です。
    TokenSubject,
    /// regulated subject identity です。
    RegulatedSubject,
    /// tenant / business role identity です。
    TenantRole,
    /// medical role identity です。
    MedicalRole,
    /// facility identity です。
    FacilityIdentity,
}

impl ExternalIdentitySource {
    /// external identity source は core transport identity の発行 authority ではありません。
    pub const fn issues_core_identity(self) -> bool {
        match self {
            Self::ApplicationUser
            | Self::TokenSubject
            | Self::RegulatedSubject
            | Self::TenantRole
            | Self::MedicalRole
            | Self::FacilityIdentity => false,
        }
    }
}

macro_rules! define_reference {
    ($name:ident, $kind:expr) => {
        #[doc = concat!(stringify!($name), " の core-owned opaque reference です。")]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(OpaqueReference);

        impl $name {
            /// reference kind です。
            pub const fn kind() -> IdentityKind {
                $kind
            }

            /// accepted opaque reference から typed reference を生成します。
            pub fn new(reference: OpaqueReference) -> Self {
                Self(reference)
            }

            /// opaque value です。外部 identity と同一視してはいけません。
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// accepted reference の成立 authority です。
            pub const fn authority(&self) -> ReferenceAuthority {
                self.0.authority()
            }
        }
    };
}

define_reference!(CorrelationId, IdentityKind::CorrelationId);
define_reference!(RoomId, IdentityKind::RoomId);
define_reference!(SessionId, IdentityKind::SessionId);
define_reference!(ParticipantId, IdentityKind::ParticipantId);
define_reference!(EndpointId, IdentityKind::EndpointId);
define_reference!(StreamId, IdentityKind::StreamId);
define_reference!(PacketId, IdentityKind::PacketId);
define_reference!(RouteId, IdentityKind::RouteId);
define_reference!(AllocationId, IdentityKind::AllocationId);
define_reference!(PermissionId, IdentityKind::PermissionId);
define_reference!(ChannelBindId, IdentityKind::ChannelBindId);
define_reference!(AuditEventId, IdentityKind::AuditEventId);
define_reference!(CredentialRef, IdentityKind::CredentialRef);
define_reference!(StartupRunId, IdentityKind::StartupRunId);
define_reference!(ConfigurationScopeRef, IdentityKind::ConfigurationScopeRef);
