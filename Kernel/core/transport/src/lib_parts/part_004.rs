/// ICE connectivity の secure media 前提判定です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IceConnectivityDecision {
    /// ICE relation は接続済みです。
    Connected(TransportIceRelationRef),
    /// ICE relation は policy/contract 上拒否されました。
    Rejected(IceFailureKind),
    /// ICE relation は実行結果として失敗しました。
    Failed(IceFailureKind),
}

/// DTLS handshake の secure media 前提判定です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DtlsHandshakeDecision {
    /// DTLS handshake は成立済みです。
    Established,
    /// DTLS handshake は policy/contract 上拒否されました。
    Rejected(SecureMediaFailureKind),
    /// DTLS handshake は実行結果として失敗しました。
    Failed(SecureMediaFailureKind),
}

/// SRTP protection の secure media 前提判定です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SrtpProtectionDecision {
    /// SRTP protection は有効です。
    Active,
    /// SRTP protection は policy/contract 上拒否されました。
    Rejected(SecureMediaFailureKind),
    /// SRTP protection は実行結果として失敗しました。
    Failed(SecureMediaFailureKind),
}

/// secure media session failure の閉じた分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecureMediaSessionFailureReason {
    /// ICE connectivity が secure media 前提を満たしません。
    Ice(IceFailureKind),
    /// DTLS handshake が secure media 前提を満たしません。
    Dtls(SecureMediaFailureKind),
    /// SRTP protection が secure media 前提を満たしません。
    Srtp(SecureMediaFailureKind),
}

/// secure media session 判定の入力です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureMediaSessionInput {
    ice_connectivity: IceConnectivityDecision,
    dtls_handshake: DtlsHandshakeDecision,
    srtp_protection: SrtpProtectionDecision,
}

impl SecureMediaSessionInput {
    /// ICE/DTLS/SRTP の前提判定を束ねます。
    pub fn new(
        ice_connectivity: IceConnectivityDecision,
        dtls_handshake: DtlsHandshakeDecision,
        srtp_protection: SrtpProtectionDecision,
    ) -> Self {
        Self {
            ice_connectivity,
            dtls_handshake,
            srtp_protection,
        }
    }

    /// ICE connectivity 判定です。
    pub const fn ice_connectivity(&self) -> &IceConnectivityDecision {
        &self.ice_connectivity
    }

    /// DTLS handshake 判定です。
    pub const fn dtls_handshake(&self) -> DtlsHandshakeDecision {
        self.dtls_handshake
    }

    /// SRTP protection 判定です。
    pub const fn srtp_protection(&self) -> SrtpProtectionDecision {
        self.srtp_protection
    }
}

/// secure media session outcome の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecureMediaSessionDecision {
    /// secure media session は保護済みとして成立します。
    Protected,
    /// secure media session は policy/contract 上拒否されました。
    Rejected(SecureMediaSessionFailureReason),
    /// secure media session は実行結果として失敗しました。
    Failed(SecureMediaSessionFailureReason),
}

/// ICE/DTLS/SRTP の前提判定だけから secure media session を決めます。
///
/// この contract は media protection の成立だけを扱い、参加許可や通信権限は判断しません。
pub fn decide_secure_media_session(
    input: SecureMediaSessionInput,
) -> SecureMediaSessionDecision {
    match input.ice_connectivity {
        IceConnectivityDecision::Connected(_) => {}
        IceConnectivityDecision::Rejected(reason) => {
            return SecureMediaSessionDecision::Rejected(
                SecureMediaSessionFailureReason::Ice(reason),
            );
        }
        IceConnectivityDecision::Failed(reason) => {
            return SecureMediaSessionDecision::Failed(
                SecureMediaSessionFailureReason::Ice(reason),
            );
        }
    }

    match input.dtls_handshake {
        DtlsHandshakeDecision::Established => {}
        DtlsHandshakeDecision::Rejected(reason) => {
            return SecureMediaSessionDecision::Rejected(
                SecureMediaSessionFailureReason::Dtls(reason),
            );
        }
        DtlsHandshakeDecision::Failed(reason) => {
            return SecureMediaSessionDecision::Failed(
                SecureMediaSessionFailureReason::Dtls(reason),
            );
        }
    }

    match input.srtp_protection {
        SrtpProtectionDecision::Active => SecureMediaSessionDecision::Protected,
        SrtpProtectionDecision::Rejected(reason) => SecureMediaSessionDecision::Rejected(
            SecureMediaSessionFailureReason::Srtp(reason),
        ),
        SrtpProtectionDecision::Failed(reason) => SecureMediaSessionDecision::Failed(
            SecureMediaSessionFailureReason::Srtp(reason),
        ),
    }
}
