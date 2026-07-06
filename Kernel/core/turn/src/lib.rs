//! core/turn は TURN allocation、permission、channel-bind の意味論を所有する surface です。
//!
//! UDP/TCP socket、STUN/TURN wire decode、HMAC backend は driver に置き、
//! core には lifecycle と許可判断の語彙だけを配置します。

use arcrtc_core_command::{TargetSurface, UseCaseDecision};
use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CredentialRef, OpaqueReference, PacketId, PermissionId,
};
use arcrtc_core_security::{MessageIntegrityPolicyRef, VerifiedCredentialRef};

/// core TURN package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreTurnSurface;

include!("lib_parts/part_002.rs");
include!("lib_parts/part_003.rs");
include!("lib_parts/part_004.rs");

/// TURN message semantic class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnMessageClass {
    /// request semantic model です。
    Request,
    /// response semantic model です。
    Response,
    /// indication semantic model です。
    Indication,
    /// error semantic model です。
    Error,
}

/// TURN core model の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnModelKind {
    /// transaction ID です。
    TransactionId,
    /// request / response / indication です。
    Message(TurnMessageClass),
    /// allocation です。
    Allocation,
    /// permission です。
    Permission,
    /// channel binding です。
    ChannelBinding,
    /// channel binding reference です。
    ChannelBindingReference,
    /// core-owned peer address です。
    PeerAddress,
    /// relay decision です。
    RelayDecision,
    /// credential verification outcome です。
    CredentialVerificationOutcome,
    /// lifetime / expiry です。
    LifetimeExpiry,
    /// closed error reason です。
    ClosedErrorReason,
}

/// TURN decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnDecisionKind {
    /// allocation accepted / rejected です。
    Allocation,
    /// allocation released / expired です。
    AllocationLifecycle,
    /// refresh accepted / rejected / expired です。
    Refresh,
    /// permission accepted / rejected / revoked / expired です。
    Permission,
    /// channel bind accepted / rejected / expired です。
    ChannelBind,
    /// relay allowed / denied です。
    Relay,
    /// malformed message classification です。
    MalformedMessage,
    /// expired credential classification です。
    ExpiredCredential,
    /// unauthorized request classification です。
    UnauthorizedRequest,
}

/// TURN peer address の core-owned opaque representation です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorePeerAddress(String);

impl CorePeerAddress {
    /// driver 変換後の peer address semantic reference を保持します。
    pub fn new(value: impl Into<String>) -> Result<Self, TurnContractError> {
        let value = value.into();
        if value.is_empty() || value.chars().any(char::is_control) {
            return Err(TurnContractError::InvalidPeerAddress);
        }
        Ok(Self(value))
    }

    /// opaque peer address value です。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// TURN transaction ID の core-owned opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TurnTransactionId(OpaqueReference);

impl TurnTransactionId {
    /// driver で wire transaction ID を検査した後の opaque reference を保持します。
    pub const fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque transaction value です。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// TURN requested lifetime です。wire integer そのものではなく core-owned duration request です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnRequestedLifetimeSeconds(u32);

impl TurnRequestedLifetimeSeconds {
    /// requested lifetime を作ります。0 は release / no-lifetime 指示と混同するため拒否します。
    pub const fn try_new(value: u32) -> Result<Self, TurnContractError> {
        if value == 0 {
            return Err(TurnContractError::InvalidRequestedLifetime);
        }
        Ok(Self(value))
    }

    /// requested lifetime seconds です。
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// TURN contract references です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnReferenceSet {
    allocation_id: Option<AllocationId>,
    permission_id: Option<PermissionId>,
    channel_bind_id: Option<ChannelBindId>,
    credential_ref: Option<CredentialRef>,
}

impl TurnReferenceSet {
    /// TURN references を作ります。
    pub const fn new(
        allocation_id: Option<AllocationId>,
        permission_id: Option<PermissionId>,
        channel_bind_id: Option<ChannelBindId>,
        credential_ref: Option<CredentialRef>,
    ) -> Self {
        Self {
            allocation_id,
            permission_id,
            channel_bind_id,
            credential_ref,
        }
    }
}

/// wire driver から core TURN boundary に渡せる command kind です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnCommandKind {
    /// Allocate request.
    Allocate,
    /// Refresh request.
    Refresh,
    /// CreatePermission request.
    CreatePermission,
    /// ChannelBind request.
    ChannelBind,
    /// Send/Data indication の relay data intent.
    RelayData,
}

impl TurnCommandKind {
    /// command kind に対応する decision kind です。
    pub const fn decision_kind(self) -> TurnDecisionKind {
        match self {
            Self::Allocate => TurnDecisionKind::Allocation,
            Self::Refresh => TurnDecisionKind::Refresh,
            Self::CreatePermission => TurnDecisionKind::Permission,
            Self::ChannelBind => TurnDecisionKind::ChannelBind,
            Self::RelayData => TurnDecisionKind::Relay,
        }
    }
}

/// TURN wire driver が core に渡す semantic command です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnCommand {
    kind: TurnCommandKind,
    transaction_id: TurnTransactionId,
    references: TurnReferenceSet,
    peer_address: Option<CorePeerAddress>,
    requested_lifetime: Option<TurnRequestedLifetimeSeconds>,
    relay_packet_id: Option<PacketId>,
}

impl TurnCommand {
    /// wire decode 後の semantic command を作ります。raw attribute object は保持しません。
    pub fn try_new(
        kind: TurnCommandKind,
        transaction_id: TurnTransactionId,
        references: TurnReferenceSet,
        peer_address: Option<CorePeerAddress>,
        requested_lifetime: Option<TurnRequestedLifetimeSeconds>,
        relay_packet_id: Option<PacketId>,
    ) -> Result<Self, TurnContractError> {
        match kind {
            TurnCommandKind::CreatePermission if peer_address.is_none() => {
                return Err(TurnContractError::MissingPeerAddress);
            }
            TurnCommandKind::ChannelBind
                if peer_address.is_none() || references.channel_bind_id.is_none() =>
            {
                return Err(TurnContractError::MissingChannelBindReference);
            }
            TurnCommandKind::RelayData if peer_address.is_none() || relay_packet_id.is_none() => {
                return Err(TurnContractError::MissingRelayPacketReference);
            }
            _ => {}
        }

        Ok(Self {
            kind,
            transaction_id,
            references,
            peer_address,
            requested_lifetime,
            relay_packet_id,
        })
    }

    /// TURN command kind です。
    pub const fn kind(&self) -> TurnCommandKind {
        self.kind
    }

    /// TURN transaction reference です。
    pub const fn transaction_id(&self) -> &TurnTransactionId {
        &self.transaction_id
    }

    /// allocation / permission / channel / credential references です。
    pub const fn references(&self) -> &TurnReferenceSet {
        &self.references
    }
}

/// TURN decision contract です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnDecision<Reason> {
    kind: TurnDecisionKind,
    decision: UseCaseDecision<Reason>,
}

impl<Reason> TurnDecision<Reason> {
    /// TURN decision を core command decision から作ります。
    pub fn new(
        kind: TurnDecisionKind,
        decision: UseCaseDecision<Reason>,
    ) -> Result<Self, TurnContractError> {
        if decision.target_surface() != TargetSurface::Turn {
            return Err(TurnContractError::WrongTargetSurface);
        }
        Ok(Self { kind, decision })
    }
}

/// TURN fail-closed failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnFailureKind {
    /// malformed STUN/TURN message or invalid transaction ID.
    MalformedTurnMessage,
    /// missing credential proof.
    CredentialMissing,
    /// invalid credential proof.
    CredentialInvalid,
    /// expired credential.
    CredentialExpired,
    /// secret generation not accepted.
    SecretGenerationNotAccepted,
    /// credential key or generation revoked.
    SecretKeyRevoked,
    /// prior-generation overlap window expired.
    SecretOverlapWindowExpired,
    /// rotation state source unavailable.
    SecretRotationStateUnavailable,
    /// allocation capacity exceeded.
    AllocationCapacityExceeded,
    /// permission capacity exceeded.
    PermissionCapacityExceeded,
    /// unauthorized peer.
    PeerNotAllowed,
    /// missing or non-active permission.
    PermissionNotFound,
    /// relay denied.
    RelayDenied,
    /// allocation missing or non-active.
    AllocationNotFound,
    /// unsupported method.
    UnsupportedTurnMethod,
    /// unsupported TURN contract version.
    UnsupportedTurnContractVersion,
    /// lifetime violation.
    TurnLifetimeViolation,
    /// allocation lifetime cap exceeded.
    AllocationLifetimeExceeded,
    /// permission lifetime expired.
    PermissionLifetimeExceeded,
    /// channel bind lifetime expired.
    ChannelBindLifetimeExceeded,
    /// refresh count or cumulative refresh cap exceeded.
    RefreshLimitExceeded,
    /// relay queue capacity or wait limit exceeded.
    TurnRelayQueueBoundExceeded,
}

impl TurnFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::MalformedTurnMessage => "malformed_turn_message",
            Self::CredentialMissing => "credential_missing",
            Self::CredentialInvalid => "credential_invalid",
            Self::CredentialExpired => "credential_expired",
            Self::SecretGenerationNotAccepted => "secret_generation_not_accepted",
            Self::SecretKeyRevoked => "secret_key_revoked",
            Self::SecretOverlapWindowExpired => "secret_overlap_window_expired",
            Self::SecretRotationStateUnavailable => "secret_rotation_state_unavailable",
            Self::AllocationCapacityExceeded => "allocation_capacity_exceeded",
            Self::PermissionCapacityExceeded => "permission_capacity_exceeded",
            Self::PeerNotAllowed => "peer_not_allowed",
            Self::PermissionNotFound => "permission_not_found",
            Self::RelayDenied => "relay_denied",
            Self::AllocationNotFound => "allocation_not_found",
            Self::UnsupportedTurnMethod => "unsupported_turn_method",
            Self::UnsupportedTurnContractVersion => "unsupported_turn_contract_version",
            Self::TurnLifetimeViolation => "turn_lifetime_violation",
            Self::AllocationLifetimeExceeded => "allocation_lifetime_exceeded",
            Self::PermissionLifetimeExceeded => "permission_lifetime_exceeded",
            Self::ChannelBindLifetimeExceeded => "channel_bind_lifetime_exceeded",
            Self::RefreshLimitExceeded => "refresh_limit_exceeded",
            Self::TurnRelayQueueBoundExceeded => "turn_relay_queue_bound_exceeded",
        }
    }
}

/// TURN contract shape rule 違反です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnContractError {
    /// decision target surface が TURN ではありません。
    WrongTargetSurface,
    /// peer address semantic reference が空または制御文字を含みます。
    InvalidPeerAddress,
    /// requested lifetime が 0 です。
    InvalidRequestedLifetime,
    /// peer address attribute が必要な command で欠落しています。
    MissingPeerAddress,
    /// channel bind reference が必要な command で欠落しています。
    MissingChannelBindReference,
    /// relay packet reference が必要な command で欠落しています。
    MissingRelayPacketReference,
}

/// TURN allocation state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllocationState {
    /// no allocation exists.
    Absent,
    /// request is being evaluated.
    Requested,
    /// allocation can relay when permission allows.
    Active,
    /// refresh is being evaluated.
    Refreshing,
    /// lifetime ended.
    Expired,
    /// allocation intentionally released.
    Released,
    /// request rejected.
    Rejected,
}

/// TURN permission state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionState {
    /// no permission for peer.
    Absent,
    /// permission request is being evaluated.
    Requested,
    /// peer relay is allowed.
    Active,
    /// permission lifetime ended.
    Expired,
    /// permission is no longer allowed.
    Revoked,
    /// request rejected.
    Rejected,
}

/// TURN channel bind state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelBindState {
    /// no channel binding exists.
    Unbound,
    /// bind request is being evaluated.
    Requested,
    /// channel can be used.
    Bound,
    /// binding lifetime ended.
    Expired,
    /// bind request rejected.
    Rejected,
}

/// TURN lifecycle row です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnLifecycleRule {
    event: &'static str,
    allowed_pre_state: &'static str,
    success_state: &'static str,
    failure_state: &'static str,
    reason_codes: &'static [TurnFailureKind],
}

impl TurnLifecycleRule {
    /// lifecycle event 名です。
    pub const fn event(&self) -> &'static str {
        self.event
    }

    /// Canonical の pre-state tuple notation です。
    pub const fn allowed_pre_state(&self) -> &'static str {
        self.allowed_pre_state
    }

    /// success state notation です。
    pub const fn success_state(&self) -> &'static str {
        self.success_state
    }

    /// failure state notation です。
    pub const fn failure_state(&self) -> &'static str {
        self.failure_state
    }

    /// decision reason candidates です。
    pub const fn reason_codes(&self) -> &'static [TurnFailureKind] {
        self.reason_codes
    }
}

const CREDENTIAL_ALLOCATION_REJECTS: &[TurnFailureKind] = &[
    TurnFailureKind::CredentialMissing,
    TurnFailureKind::CredentialInvalid,
    TurnFailureKind::CredentialExpired,
    TurnFailureKind::AllocationCapacityExceeded,
];
const CREDENTIAL_REJECTS: &[TurnFailureKind] = &[
    TurnFailureKind::CredentialMissing,
    TurnFailureKind::CredentialInvalid,
    TurnFailureKind::CredentialExpired,
];
const ALLOCATION_NOT_FOUND: &[TurnFailureKind] = &[TurnFailureKind::AllocationNotFound];
const PERMISSION_REJECTS: &[TurnFailureKind] = &[
    TurnFailureKind::CredentialMissing,
    TurnFailureKind::CredentialInvalid,
    TurnFailureKind::CredentialExpired,
    TurnFailureKind::AllocationNotFound,
    TurnFailureKind::PeerNotAllowed,
    TurnFailureKind::PermissionCapacityExceeded,
    TurnFailureKind::TurnLifetimeViolation,
];
const CHANNEL_REJECTS: &[TurnFailureKind] = &[
    TurnFailureKind::CredentialMissing,
    TurnFailureKind::CredentialInvalid,
    TurnFailureKind::CredentialExpired,
    TurnFailureKind::AllocationNotFound,
    TurnFailureKind::PermissionNotFound,
    TurnFailureKind::RelayDenied,
    TurnFailureKind::TurnLifetimeViolation,
];
const RELAY_REJECTS: &[TurnFailureKind] = &[
    TurnFailureKind::CredentialMissing,
    TurnFailureKind::CredentialInvalid,
    TurnFailureKind::CredentialExpired,
    TurnFailureKind::RelayDenied,
];
const PERMISSION_NOT_FOUND: &[TurnFailureKind] = &[TurnFailureKind::PermissionNotFound];
const SUCCESS_ONLY: &[TurnFailureKind] = &[];

/// canonical 由来の TURN lifecycle table です。
pub const TURN_LIFECYCLE_RULES: &[TurnLifecycleRule] = &[
    TurnLifecycleRule { event: "Allocate", allowed_pre_state: "allocation=allocation_absent", success_state: "allocation_requested", failure_state: "allocation_rejected", reason_codes: CREDENTIAL_ALLOCATION_REJECTS },
    TurnLifecycleRule { event: "AcceptAllocation", allowed_pre_state: "allocation=allocation_requested", success_state: "allocation_active", failure_state: "allocation_rejected", reason_codes: &[TurnFailureKind::CredentialInvalid, TurnFailureKind::CredentialExpired, TurnFailureKind::AllocationCapacityExceeded] },
    TurnLifecycleRule { event: "RejectAllocation", allowed_pre_state: "allocation=allocation_requested", success_state: "allocation_rejected", failure_state: "allocation_rejected", reason_codes: CREDENTIAL_ALLOCATION_REJECTS },
    TurnLifecycleRule { event: "Refresh", allowed_pre_state: "allocation=allocation_active", success_state: "allocation_refreshing", failure_state: "allocation_active", reason_codes: CREDENTIAL_REJECTS },
    TurnLifecycleRule { event: "RejectRefreshLifetimeViolation", allowed_pre_state: "allocation=allocation_active", success_state: "allocation_active", failure_state: "allocation_active", reason_codes: &[TurnFailureKind::TurnLifetimeViolation] },
    TurnLifecycleRule { event: "AcceptRefresh", allowed_pre_state: "allocation=allocation_refreshing", success_state: "allocation_active", failure_state: "allocation_active", reason_codes: CREDENTIAL_REJECTS },
    TurnLifecycleRule { event: "RejectRefresh", allowed_pre_state: "allocation=allocation_refreshing", success_state: "allocation_active", failure_state: "allocation_active", reason_codes: CREDENTIAL_REJECTS },
    TurnLifecycleRule { event: "RejectRefreshAbsent", allowed_pre_state: "allocation=allocation_absent", success_state: "allocation_absent", failure_state: "allocation_absent", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "RejectRefreshRequested", allowed_pre_state: "allocation=allocation_requested", success_state: "allocation_requested", failure_state: "allocation_requested", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "RejectRefreshExpired", allowed_pre_state: "allocation=allocation_expired", success_state: "allocation_expired", failure_state: "allocation_expired", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "RejectRefreshReleased", allowed_pre_state: "allocation=allocation_released", success_state: "allocation_released", failure_state: "allocation_released", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "RejectRefreshRejected", allowed_pre_state: "allocation=allocation_rejected", success_state: "allocation_rejected", failure_state: "allocation_rejected", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "Release", allowed_pre_state: "allocation={allocation_active, allocation_refreshing}", success_state: "allocation_released", failure_state: "success-only", reason_codes: SUCCESS_ONLY },
    TurnLifecycleRule { event: "RejectReleaseAbsent", allowed_pre_state: "allocation=allocation_absent", success_state: "allocation_absent", failure_state: "allocation_absent", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "RejectReleaseRequested", allowed_pre_state: "allocation=allocation_requested", success_state: "allocation_requested", failure_state: "allocation_requested", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "RejectReleaseExpired", allowed_pre_state: "allocation=allocation_expired", success_state: "allocation_expired", failure_state: "allocation_expired", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "RejectReleaseReleased", allowed_pre_state: "allocation=allocation_released", success_state: "allocation_released", failure_state: "allocation_released", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "RejectReleaseRejected", allowed_pre_state: "allocation=allocation_rejected", success_state: "allocation_rejected", failure_state: "allocation_rejected", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "ExpireAllocationLifetime", allowed_pre_state: "allocation={allocation_active, allocation_refreshing}", success_state: "allocation_expired", failure_state: "allocation_expired", reason_codes: &[TurnFailureKind::AllocationLifetimeExceeded] },
    TurnLifecycleRule { event: "ExpireRefreshCap", allowed_pre_state: "allocation={allocation_active, allocation_refreshing}", success_state: "allocation_expired", failure_state: "allocation_expired", reason_codes: &[TurnFailureKind::RefreshLimitExceeded] },
    TurnLifecycleRule { event: "CreatePermission", allowed_pre_state: "allocation=allocation_active; permission=permission_absent", success_state: "permission_requested", failure_state: "permission_rejected", reason_codes: PERMISSION_REJECTS },
    TurnLifecycleRule { event: "AcceptPermission", allowed_pre_state: "allocation=allocation_active; permission=permission_requested", success_state: "permission_active", failure_state: "permission_rejected", reason_codes: PERMISSION_REJECTS },
    TurnLifecycleRule { event: "RejectPermission", allowed_pre_state: "permission=permission_requested", success_state: "permission_rejected", failure_state: "permission_rejected", reason_codes: PERMISSION_REJECTS },
    TurnLifecycleRule { event: "ExpirePermission", allowed_pre_state: "permission=permission_active", success_state: "permission_expired", failure_state: "permission_expired", reason_codes: &[TurnFailureKind::PermissionLifetimeExceeded] },
    TurnLifecycleRule { event: "RevokePermission", allowed_pre_state: "permission=permission_active", success_state: "permission_revoked", failure_state: "permission_revoked", reason_codes: &[TurnFailureKind::PeerNotAllowed] },
    TurnLifecycleRule { event: "ChannelBind", allowed_pre_state: "allocation=allocation_active; permission=permission_active; channel=channel_unbound", success_state: "channel_bind_requested", failure_state: "channel_rejected", reason_codes: CHANNEL_REJECTS },
    TurnLifecycleRule { event: "AcceptChannelBind", allowed_pre_state: "allocation=allocation_active; permission=permission_active; channel=channel_bind_requested", success_state: "channel_bound", failure_state: "channel_rejected", reason_codes: CHANNEL_REJECTS },
    TurnLifecycleRule { event: "RejectChannelBind", allowed_pre_state: "channel=channel_bind_requested", success_state: "channel_rejected", failure_state: "channel_rejected", reason_codes: CHANNEL_REJECTS },
    TurnLifecycleRule { event: "ExpireChannelBind", allowed_pre_state: "channel=channel_bound", success_state: "channel_expired", failure_state: "channel_expired", reason_codes: &[TurnFailureKind::ChannelBindLifetimeExceeded] },
    TurnLifecycleRule { event: "RelayData", allowed_pre_state: "allocation=allocation_active; permission=permission_active", success_state: "allocation_active, permission_active", failure_state: "allocation_active, permission_active", reason_codes: RELAY_REJECTS },
    TurnLifecycleRule { event: "DenyRelayDataAllocationAbsent", allowed_pre_state: "allocation=allocation_absent", success_state: "allocation_absent", failure_state: "allocation_absent", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataAllocationRequested", allowed_pre_state: "allocation=allocation_requested", success_state: "allocation_requested", failure_state: "allocation_requested", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataAllocationExpired", allowed_pre_state: "allocation=allocation_expired", success_state: "allocation_expired", failure_state: "allocation_expired", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataAllocationReleased", allowed_pre_state: "allocation=allocation_released", success_state: "allocation_released", failure_state: "allocation_released", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataAllocationRejected", allowed_pre_state: "allocation=allocation_rejected", success_state: "allocation_rejected", failure_state: "allocation_rejected", reason_codes: ALLOCATION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataPermissionAbsent", allowed_pre_state: "allocation=allocation_active; permission=permission_absent", success_state: "allocation_active, permission_absent", failure_state: "allocation_active, permission_absent", reason_codes: PERMISSION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataPermissionRequested", allowed_pre_state: "allocation=allocation_active; permission=permission_requested", success_state: "allocation_active, permission_requested", failure_state: "allocation_active, permission_requested", reason_codes: PERMISSION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataPermissionExpired", allowed_pre_state: "allocation=allocation_active; permission=permission_expired", success_state: "allocation_active, permission_expired", failure_state: "allocation_active, permission_expired", reason_codes: PERMISSION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataPermissionRevoked", allowed_pre_state: "allocation=allocation_active; permission=permission_revoked", success_state: "allocation_active, permission_revoked", failure_state: "allocation_active, permission_revoked", reason_codes: PERMISSION_NOT_FOUND },
    TurnLifecycleRule { event: "DenyRelayDataPermissionRejected", allowed_pre_state: "allocation=allocation_active; permission=permission_rejected", success_state: "allocation_active, permission_rejected", failure_state: "allocation_active, permission_rejected", reason_codes: PERMISSION_NOT_FOUND },
];

/// TURN one-shot 評価における allocation 初期状態です。
pub const INITIAL_TURN_ALLOCATION_STATE: AllocationState = AllocationState::Absent;

/// TURN one-shot 評価における permission 初期状態です。
pub const INITIAL_TURN_PERMISSION_STATE: PermissionState = PermissionState::Absent;

/// TURN one-shot 評価における channel-bind 初期状態です。
pub const INITIAL_TURN_CHANNEL_BIND_STATE: ChannelBindState = ChannelBindState::Unbound;

/// 初期状態に対する 1 datagram の TURN command を fail-closed reason へ写像します。
pub fn apply_initial_turn_command(command: &TurnCommand) -> TurnFailureKind {
    match command.kind() {
        TurnCommandKind::Allocate => {
            let rule = TURN_LIFECYCLE_RULES
                .iter()
                .find(|rule| rule.event() == "Allocate")
                .expect("TURN lifecycle table must contain Allocate row");
            if command.references().credential_ref.is_none() {
                rule.reason_codes()[0]
            } else {
                rule.reason_codes()[1]
            }
        }
        TurnCommandKind::Refresh => {
            let rule = TURN_LIFECYCLE_RULES
                .iter()
                .find(|rule| rule.event() == "RejectRefreshAbsent")
                .expect("TURN lifecycle table must contain RejectRefreshAbsent row");
            rule.reason_codes()[0]
        }
        TurnCommandKind::CreatePermission => {
            let rule = TURN_LIFECYCLE_RULES
                .iter()
                .find(|rule| rule.event() == "CreatePermission")
                .expect("TURN lifecycle table must contain CreatePermission row");
            rule.reason_codes()
                .iter()
                .copied()
                .find(|kind| *kind == TurnFailureKind::AllocationNotFound)
                .expect("CreatePermission row must include AllocationNotFound reason")
        }
        TurnCommandKind::ChannelBind => {
            let rule = TURN_LIFECYCLE_RULES
                .iter()
                .find(|rule| rule.event() == "ChannelBind")
                .expect("TURN lifecycle table must contain ChannelBind row");
            rule.reason_codes()
                .iter()
                .copied()
                .find(|kind| *kind == TurnFailureKind::AllocationNotFound)
                .expect("ChannelBind row must include AllocationNotFound reason")
        }
        TurnCommandKind::RelayData => {
            let rule = TURN_LIFECYCLE_RULES
                .iter()
                .find(|rule| rule.event() == "DenyRelayDataAllocationAbsent")
                .expect("TURN lifecycle table must contain DenyRelayDataAllocationAbsent row");
            rule.reason_codes()[0]
        }
    }
}
