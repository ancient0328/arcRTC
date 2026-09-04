//! Kernel runtime の benchmark command surface です。
//!
//! `arcrtc-benchmarks` package の workload registry と bench target を実体化し、
//! test support へ依存せず Kernel production crates の公開 contract を測定します。

mod measurement_window;

pub use measurement_window::{
    benchmark_lane_window, BenchmarkLaneWindow, BenchmarkMeasurementWindow, BenchmarkWindowError,
};

use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CredentialRef, OpaqueReference, PacketId, PermissionId,
    ReferenceAuthority,
};
use arcrtc_core_sfu::{apply_resident_sfu_input, ResidentSfuInputClass, ResidentSfuOutcome};
use arcrtc_core_signaling::{
    apply_one_shot_signaling_command, SignalingCommandKind, INITIAL_SIGNALING_PARTICIPANT_STATE,
    INITIAL_SIGNALING_ROOM_STATE,
};
use arcrtc_core_turn::{
    apply_resident_turn_command, CorePeerAddress, ResidentTurnState, ResidentTurnSuccessKind,
    TurnCommand, TurnCommandKind, TurnReferenceSet, TurnRequestedLifetimeSeconds,
    TurnTransactionId,
};

const BENCHMARK_CASES: [BenchmarkCriterionCase; 16] = [
    BenchmarkCriterionCase {
        scenario_id: "BENCH-001",
        label: "signaling membership transition load",
        workload_class: "signaling_membership_transition_load",
        source_class: "kernel_core_signaling",
        workload: signaling_membership_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-002",
        label: "resident TURN allocation load",
        workload_class: "turn_allocation_lifecycle_load",
        source_class: "kernel_core_turn",
        workload: turn_lifecycle_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-003",
        label: "resident SFU secure media load",
        workload_class: "sfu_secure_media_load",
        source_class: "kernel_core_sfu",
        workload: sfu_secure_media_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-004",
        label: "mixed resident kernel load",
        workload_class: "mixed_resident_kernel_load",
        source_class: "kernel_core_resident_paths",
        workload: mixed_resident_kernel_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-005",
        label: "signaling rejection load",
        workload_class: "signaling_rejection_load",
        source_class: "kernel_core_signaling",
        workload: signaling_rejection_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-006",
        label: "TURN relay load",
        workload_class: "turn_relay_load",
        source_class: "kernel_core_turn",
        workload: turn_lifecycle_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-007",
        label: "signaling membership soak",
        workload_class: "signaling_membership_soak",
        source_class: "kernel_core_signaling",
        workload: signaling_soak_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-008",
        label: "TURN lifecycle soak",
        workload_class: "turn_lifecycle_soak",
        source_class: "kernel_core_turn",
        workload: turn_soak_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-009",
        label: "SFU secure media soak",
        workload_class: "sfu_secure_media_soak",
        source_class: "kernel_core_sfu",
        workload: sfu_soak_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-010",
        label: "mixed resident kernel soak",
        workload_class: "mixed_resident_kernel_soak",
        source_class: "kernel_core_resident_paths",
        workload: mixed_soak_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-011",
        label: "closed failure mapping soak",
        workload_class: "closed_failure_mapping_soak",
        source_class: "kernel_core_closed_reasons",
        workload: closed_reason_soak_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-012",
        label: "signaling concurrency-shaped loop",
        workload_class: "signaling_concurrency_shape",
        source_class: "kernel_core_signaling",
        workload: signaling_concurrency_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-013",
        label: "TURN concurrency-shaped loop",
        workload_class: "turn_concurrency_shape",
        source_class: "kernel_core_turn",
        workload: turn_concurrency_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-014",
        label: "SFU concurrency-shaped loop",
        workload_class: "sfu_concurrency_shape",
        source_class: "kernel_core_sfu",
        workload: sfu_concurrency_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-015",
        label: "mixed resident concurrency-shaped loop",
        workload_class: "mixed_resident_kernel_concurrency_shape",
        source_class: "kernel_core_resident_paths",
        workload: mixed_concurrency_workload,
    },
    BenchmarkCriterionCase {
        scenario_id: "BENCH-016",
        label: "closed reason concurrency-shaped loop",
        workload_class: "closed_reason_concurrency_shape",
        source_class: "kernel_core_closed_reasons",
        workload: closed_reason_concurrency_workload,
    },
];

/// Criterion が反復実行する Kernel runtime benchmark case です。
#[derive(Clone, Copy)]
pub struct BenchmarkCriterionCase {
    /// `BENCH-001` のような scenario ID です。
    pub scenario_id: &'static str,
    /// scenario matrix の表示名です。
    pub label: &'static str,
    /// benchmark registry 内での workload class です。
    pub workload_class: &'static str,
    /// workload が直接参照する Kernel source class です。
    pub source_class: &'static str,
    /// Criterion が反復実行する対象 workload です。
    pub workload: fn() -> u64,
}

/// v0.2 BENCH scenario を Criterion harness へ渡すための case list です。
pub fn criterion_benchmark_cases() -> Vec<BenchmarkCriterionCase> {
    BENCHMARK_CASES.to_vec()
}

/// load / soak / concurrency workload class に対応する slice 指定です。
///
/// BENCH-001..016 の順序はこのcrateのexecutable registryが所有します。
pub fn cases_for_lane(lane: BenchmarkLane) -> Vec<BenchmarkCriterionCase> {
    let cases = criterion_benchmark_cases();
    match lane {
        BenchmarkLane::Load => cases.into_iter().take(6).collect(),
        BenchmarkLane::Soak => cases.into_iter().skip(6).take(5).collect(),
        BenchmarkLane::Concurrency => cases.into_iter().skip(11).collect(),
    }
}

/// CI command ID と bench target の対応を明示する閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkLane {
    /// Load benchmark lane です。
    Load,
    /// Soak benchmark lane です。
    Soak,
    /// Concurrency benchmark lane です。
    Concurrency,
}

impl BenchmarkLane {
    /// Executable benchmark lane の閉集合です。
    pub const ALL: [Self; 3] = [Self::Load, Self::Soak, Self::Concurrency];

    /// Technical output に使用する安定した lane 名です。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Soak => "soak",
            Self::Concurrency => "concurrency",
        }
    }
}

fn signaling_membership_workload() -> u64 {
    let commands = [
        SignalingCommandKind::JoinRoom,
        SignalingCommandKind::SendOffer,
        SignalingCommandKind::SendAnswer,
        SignalingCommandKind::SendIceCandidate,
        SignalingCommandKind::LeaveRoom,
    ];
    signaling_loop(&commands, 64)
}

fn signaling_rejection_workload() -> u64 {
    let commands = [
        SignalingCommandKind::SendOffer,
        SignalingCommandKind::SendAnswer,
        SignalingCommandKind::RequestTurnCredential,
    ];
    signaling_loop(&commands, 96)
}

fn signaling_soak_workload() -> u64 {
    signaling_membership_workload().wrapping_add(signaling_rejection_workload())
}

fn signaling_concurrency_workload() -> u64 {
    signaling_loop(
        &[
            SignalingCommandKind::JoinRoom,
            SignalingCommandKind::AcknowledgeForward,
            SignalingCommandKind::LeaveRoom,
        ],
        128,
    )
}

fn signaling_loop(commands: &[SignalingCommandKind], iterations: u64) -> u64 {
    let mut digest = 0u64;
    for iteration in 0..iterations {
        for command in commands {
            let observation = apply_one_shot_signaling_command(
                *command,
                INITIAL_SIGNALING_ROOM_STATE,
                INITIAL_SIGNALING_PARTICIPANT_STATE,
            );
            digest = digest.wrapping_add(iteration);
            digest = match observation {
                Ok(outcome) => digest
                    .wrapping_add(outcome.next_room.map_or(0, room_state_digest))
                    .wrapping_add(outcome.next_participant.map_or(0, participant_state_digest)),
                Err(reason) => digest.wrapping_add(reason.reason_code().len() as u64),
            };
        }
    }
    digest
}

fn room_state_digest(state: arcrtc_core_signaling::RoomState) -> u64 {
    match state {
        arcrtc_core_signaling::RoomState::RoomAbsent => 1,
        arcrtc_core_signaling::RoomState::RoomOpen => 2,
        arcrtc_core_signaling::RoomState::RoomDraining => 3,
        arcrtc_core_signaling::RoomState::RoomClosed => 4,
    }
}

fn participant_state_digest(state: arcrtc_core_signaling::ParticipantState) -> u64 {
    match state {
        arcrtc_core_signaling::ParticipantState::ParticipantNew => 1,
        arcrtc_core_signaling::ParticipantState::ParticipantVerifying => 2,
        arcrtc_core_signaling::ParticipantState::ParticipantJoined => 3,
        arcrtc_core_signaling::ParticipantState::ParticipantLeaving => 4,
        arcrtc_core_signaling::ParticipantState::ParticipantLeft => 5,
        arcrtc_core_signaling::ParticipantState::ParticipantRejected => 6,
    }
}

fn turn_lifecycle_workload() -> u64 {
    let mut state = ResidentTurnState::default();
    let commands = [
        turn_command(TurnCommandKind::Allocate),
        turn_command(TurnCommandKind::Refresh),
        turn_command(TurnCommandKind::CreatePermission),
        turn_command(TurnCommandKind::ChannelBind),
        turn_command(TurnCommandKind::RelayData),
    ];

    let mut digest = 0u64;
    for command in commands {
        digest = digest.wrapping_add(
            match apply_resident_turn_command(&command, true, &mut state) {
                Ok(success) => turn_success_digest(success),
                Err(reason) => reason.reason_code().len() as u64,
            },
        );
    }
    digest
}

fn turn_soak_workload() -> u64 {
    (0..32).fold(0u64, |digest, iteration| {
        digest
            .wrapping_add(turn_lifecycle_workload())
            .wrapping_add(iteration)
    })
}

fn turn_concurrency_workload() -> u64 {
    (0..64).fold(0u64, |digest, iteration| {
        digest
            .wrapping_add(turn_lifecycle_workload())
            .rotate_left((iteration % 31) as u32)
    })
}

fn turn_command(kind: TurnCommandKind) -> TurnCommand {
    let allocation = AllocationId::new(accepted_reference("allocation:bench"));
    let permission = PermissionId::new(accepted_reference("permission:bench"));
    let channel = ChannelBindId::new(accepted_reference("channel:bench"));
    let credential = CredentialRef::new(accepted_reference("credential:bench"));
    let references = TurnReferenceSet::new(
        Some(allocation),
        Some(permission),
        Some(channel),
        Some(credential),
    );
    let peer = CorePeerAddress::new("198.51.100.10:3478").expect("benchmark peer is fixed");
    let packet = PacketId::new(accepted_reference("packet:bench"));
    let lifetime =
        TurnRequestedLifetimeSeconds::try_new(600).expect("benchmark lifetime is non-zero");

    TurnCommand::try_new(
        kind,
        TurnTransactionId::new(accepted_reference("transaction:bench")),
        references,
        Some(peer),
        Some(lifetime),
        Some(packet),
    )
    .expect("benchmark TURN command is fixed")
}

fn turn_success_digest(success: ResidentTurnSuccessKind) -> u64 {
    match success {
        ResidentTurnSuccessKind::AllocationSuccessResponse => 1,
        ResidentTurnSuccessKind::RefreshSuccessResponse => 2,
        ResidentTurnSuccessKind::PermissionSuccessResponse => 3,
        ResidentTurnSuccessKind::ChannelBindSuccessResponse => 4,
        ResidentTurnSuccessKind::RelayDataForwarding => 5,
    }
}

fn sfu_secure_media_workload() -> u64 {
    let mut state = arcrtc_core_sfu::ResidentSfuState::default();
    let inputs = [
        ResidentSfuInputClass::IceConnectivityObservation,
        ResidentSfuInputClass::DtlsHandshakeObservation,
        ResidentSfuInputClass::SrtpProtectionObservation,
        ResidentSfuInputClass::ProtectedMediaPacket,
    ];
    let mut digest = 0u64;
    for input in inputs {
        digest = digest.wrapping_add(match apply_resident_sfu_input(input, &mut state) {
            ResidentSfuOutcome::Accepted {
                digest, transmits, ..
            } => digest.wrapping_add(transmits as u64),
            ResidentSfuOutcome::Rejected { reason_code } => reason_code.len() as u64,
        });
    }
    digest
}

fn sfu_soak_workload() -> u64 {
    (0..32).fold(0u64, |digest, iteration| {
        digest
            .wrapping_add(sfu_secure_media_workload())
            .wrapping_add(iteration)
    })
}

fn sfu_concurrency_workload() -> u64 {
    (0..64).fold(0u64, |digest, iteration| {
        digest
            .wrapping_add(sfu_secure_media_workload())
            .rotate_right((iteration % 29) as u32)
    })
}

fn mixed_resident_kernel_workload() -> u64 {
    signaling_membership_workload()
        .wrapping_add(turn_lifecycle_workload())
        .wrapping_add(sfu_secure_media_workload())
}

fn mixed_soak_workload() -> u64 {
    signaling_soak_workload()
        .wrapping_add(turn_soak_workload())
        .wrapping_add(sfu_soak_workload())
}

fn mixed_concurrency_workload() -> u64 {
    signaling_concurrency_workload()
        .wrapping_add(turn_concurrency_workload())
        .wrapping_add(sfu_concurrency_workload())
}

fn closed_reason_soak_workload() -> u64 {
    signaling_rejection_workload().wrapping_add(turn_soak_workload())
}

fn closed_reason_concurrency_workload() -> u64 {
    signaling_rejection_workload()
        .wrapping_add(turn_concurrency_workload())
        .wrapping_add(sfu_concurrency_workload())
}

fn accepted_reference(value: &'static str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy)
        .expect("benchmark reference is fixed")
}
