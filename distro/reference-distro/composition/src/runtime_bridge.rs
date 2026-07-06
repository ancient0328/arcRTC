//! reference composition runtime bridge境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{
    DistroEvidenceReason, DistroNonClaimScope, DistroPlane,
};
use arcrtc_reference_output::{
    ReferenceSfuOutcome, ReferenceSignalingOutcome, ReferenceTurnOutcome,
};

use crate::{
    composition_state::{
        bind_signaling_to_sfu, bind_signaling_to_turn, validate_reference_composition_state,
        ReferenceCompositionState,
    },
    error::ReferenceCompositionError,
    step_input::{ReferenceCompositionStep, ReferenceCompositionStepInput},
};

/// composition step 内のplane outcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceCompositionPlaneOutcome {
    /// Signaling outcomeです。
    Signaling(ReferenceSignalingOutcome),
    /// TURN outcomeです。
    Turn(ReferenceTurnOutcome),
    /// SFU outcomeです。
    Sfu(ReferenceSfuOutcome),
    /// Composition bindingまたはvalidation outcomeです。
    Composition,
}

/// composition step outcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCompositionStepOutcome {
    /// step correlation idです。
    pub correlation_id: CorrelationId,
    /// distro-local evidence reasonです。
    pub distro_reason: DistroEvidenceReason,
    /// 適用されたplaneです。
    pub applied_plane: DistroPlane,
    /// plane outcomeです。
    pub plane_outcome: ReferenceCompositionPlaneOutcome,
    /// このstepが主張しないscopeです。
    pub non_claim_scope: Vec<DistroNonClaimScope>,
}

/// reference composition stepを実行します。
pub fn run_reference_composition_step(
    state: &mut ReferenceCompositionState,
    input: ReferenceCompositionStepInput,
) -> Result<ReferenceCompositionStepOutcome, ReferenceCompositionError> {
    if input.correlation_id.as_str().is_empty() {
        return Err(ReferenceCompositionError::EvidenceFieldsIncomplete);
    }
    let (applied_plane, plane_outcome) = match input.step {
        ReferenceCompositionStep::ApplySignaling(command) => (
            DistroPlane::Signaling,
            ReferenceCompositionPlaneOutcome::Signaling(
                arcrtc_reference_signaling::apply_reference_signaling(
                    &mut state.signaling,
                    &command,
                )
                .map_err(|_| ReferenceCompositionError::PlaneExecutionFailed)?,
            ),
        ),
        ReferenceCompositionStep::ApplyTurn(command) => (
            DistroPlane::Turn,
            ReferenceCompositionPlaneOutcome::Turn(
                arcrtc_reference_turn::apply_reference_turn(&mut state.turn, &command)
                    .map_err(|_| ReferenceCompositionError::PlaneExecutionFailed)?,
            ),
        ),
        ReferenceCompositionStep::ApplySfu(action) => (
            DistroPlane::Sfu,
            ReferenceCompositionPlaneOutcome::Sfu(
                arcrtc_reference_sfu::apply_reference_sfu(&mut state.sfu, &action)
                    .map_err(|_| ReferenceCompositionError::PlaneExecutionFailed)?,
            ),
        ),
        ReferenceCompositionStep::BindSignalingToTurn {
            room_id,
            allocation_id,
        } => {
            bind_signaling_to_turn(state, input.correlation_id.clone(), room_id, allocation_id)?;
            (
                DistroPlane::Composition,
                ReferenceCompositionPlaneOutcome::Composition,
            )
        }
        ReferenceCompositionStep::BindSignalingToSfu {
            room_id,
            session_id,
            route_id,
        } => {
            bind_signaling_to_sfu(
                state,
                input.correlation_id.clone(),
                room_id,
                session_id,
                route_id,
            )?;
            (
                DistroPlane::Composition,
                ReferenceCompositionPlaneOutcome::Composition,
            )
        }
        ReferenceCompositionStep::Validate => {
            validate_reference_composition_state(state)?;
            (
                DistroPlane::Composition,
                ReferenceCompositionPlaneOutcome::Composition,
            )
        }
    };
    Ok(ReferenceCompositionStepOutcome {
        correlation_id: input.correlation_id,
        distro_reason: DistroEvidenceReason::DistroOk,
        applied_plane,
        plane_outcome,
        non_claim_scope: vec![
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
    })
}
