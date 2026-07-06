/// Signaling protocol command class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingCommandClass {
    /// SDP offer command.
    Offer,
    /// SDP answer command.
    Answer,
    /// ICE candidate command.
    IceCandidate,
    /// reconnect command.
    Reconnect,
    /// leave command.
    Leave,
}

/// Signaling command classification input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalingCommandClassificationInput {
    command_class: SignalingCommandClass,
}

impl SignalingCommandClassificationInput {
    /// command class を分類入力として保持します。
    pub const fn new(command_class: SignalingCommandClass) -> Self {
        Self { command_class }
    }
}

/// Signaling command classification result です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalingCommandClassification {
    command_class: SignalingCommandClass,
    transport_relation_ref: TransportIceRelationRef,
}

impl SignalingCommandClassification {
    /// command class と opaque transport relation reference を束ねます。
    pub const fn new(
        command_class: SignalingCommandClass,
        transport_relation_ref: TransportIceRelationRef,
    ) -> Self {
        Self {
            command_class,
            transport_relation_ref,
        }
    }

    /// command class です。
    pub const fn command_class(&self) -> SignalingCommandClass {
        self.command_class
    }

    /// opaque transport relation reference です。connectivity proof ではありません。
    pub const fn transport_relation_ref(&self) -> &TransportIceRelationRef {
        &self.transport_relation_ref
    }
}

/// signaling command と opaque transport relation を分類結果に写像します。
pub fn classify_signaling_command(
    command: SignalingCommandClassificationInput,
    transport_relation_ref: TransportIceRelationRef,
) -> SignalingCommandClassification {
    SignalingCommandClassification::new(command.command_class, transport_relation_ref)
}
