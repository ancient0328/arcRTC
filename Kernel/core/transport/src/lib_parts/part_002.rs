impl IceFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::IceCandidatePolicyViolation => "ice_candidate_policy_violation",
            Self::IceCandidateMappingInvalid => "ice_candidate_mapping_invalid",
            Self::IceCandidateRedactionRequired => "ice_candidate_redaction_required",
            Self::IceGatheringFailed => "ice_gathering_failed",
            Self::IceConnectivityCheckFailed => "ice_connectivity_check_failed",
            Self::IceConsentExpired => "ice_consent_expired",
            Self::IceRestartNotAllowed => "ice_restart_not_allowed",
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::CommandOrderViolation => "command_order_violation",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::NetworkSendFailed => "network_send_failed",
        }
    }
}

/// secure media session class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecureMediaSessionClass {
    /// policy requires protected media path.
    SecureMediaRequired,
    /// driver observed DTLS handshake attempt/result.
    DtlsHandshakeObserved,
    /// driver observed peer verification result.
    PeerVerificationObserved,
    /// driver observed SRTP protection active.
    SrtpProtectionActive,
    /// key generation/rotation requires rekey.
    RekeyRequired,
    /// secure media session ended.
    SessionClosed,
}

/// secure media lifecycle policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecureMediaPolicy {
    contract_version: &'static str,
    profile_class: &'static str,
    peer_verification_required: bool,
    protection_required_before_forwarding: bool,
}

impl SecureMediaPolicy {
    /// secure media requirement を固定します。
    pub const fn new(
        contract_version: &'static str,
        profile_class: &'static str,
        peer_verification_required: bool,
        protection_required_before_forwarding: bool,
    ) -> Self {
        Self {
            contract_version,
            profile_class,
            peer_verification_required,
            protection_required_before_forwarding,
        }
    }
}

/// secure media evidence class の分離です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecureMediaEvidenceClass {
    /// handshake evidence です。
    Handshake,
    /// peer verification evidence です。
    PeerVerification,
    /// protection state evidence です。
    ProtectionState,
    /// packet forwarding claim scope です。
    PacketForwardingScope,
}

/// secure media failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecureMediaFailureKind {
    /// secure media profile/version unsupported.
    SecureMediaProfileNotSupported,
    /// DTLS handshake failed.
    SecureMediaHandshakeFailed,
    /// peer verification failed.
    SecureMediaPeerVerificationFailed,
    /// SRTP protection not active where required.
    SecureMediaProtectionNotActive,
    /// secure media key state invalid.
    SecureMediaKeyStateInvalid,
    /// secure media session expired.
    SecureMediaSessionExpired,
    /// rekey required before continuing.
    SecureMediaRekeyRequired,
    /// required secret source unavailable.
    SecretUnavailable,
    /// rotation state unavailable.
    SecretRotationStateUnavailable,
    /// revoked secret/key generation.
    SecretKeyRevoked,
}

impl SecureMediaFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::SecureMediaProfileNotSupported => "secure_media_profile_not_supported",
            Self::SecureMediaHandshakeFailed => "secure_media_handshake_failed",
            Self::SecureMediaPeerVerificationFailed => "secure_media_peer_verification_failed",
            Self::SecureMediaProtectionNotActive => "secure_media_protection_not_active",
            Self::SecureMediaKeyStateInvalid => "secure_media_key_state_invalid",
            Self::SecureMediaSessionExpired => "secure_media_session_expired",
            Self::SecureMediaRekeyRequired => "secure_media_rekey_required",
            Self::SecretUnavailable => "secret_unavailable",
            Self::SecretRotationStateUnavailable => "secret_rotation_state_unavailable",
            Self::SecretKeyRevoked => "secret_key_revoked",
        }
    }
}
