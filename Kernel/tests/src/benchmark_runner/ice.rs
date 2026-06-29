use std::hint::black_box;

use arcrtc_core_signaling::SignalingCommandKind;

use super::support::mix_bytes;

#[derive(Debug, Clone)]
struct CandidateTemplate {
    foundation: String,
    component: u8,
    protocol: &'static str,
    priority: u32,
    address: String,
    port: u16,
    candidate_type: &'static str,
}

pub(super) fn pooled_candidate_generation_workload() -> u64 {
    let pool: Vec<CandidateTemplate> = (0..128).map(candidate_template).collect();
    let mut checksum = 0u64;
    for index in 0..1_000 {
        checksum = mix_bytes(
            checksum,
            &render_candidate(&pool[index % pool.len()], index),
        );
    }
    black_box(SignalingCommandKind::SendIceCandidate);
    checksum
}

pub(super) fn ondemand_candidate_generation_workload() -> u64 {
    let mut checksum = 0u64;
    for index in 0..1_000 {
        checksum = mix_bytes(
            checksum,
            &render_candidate(&candidate_template(index), index),
        );
    }
    black_box(SignalingCommandKind::SendIceCandidate);
    checksum
}

fn candidate_template(index: usize) -> CandidateTemplate {
    CandidateTemplate {
        foundation: format!("f{}", index % 16),
        component: if index.is_multiple_of(2) { 1 } else { 2 },
        protocol: "udp",
        priority: 2_130_706_431u32.saturating_sub((index % 2048) as u32),
        address: format!("192.0.2.{}", (index % 250) + 1),
        port: 10_000 + (index % 20_000) as u16,
        candidate_type: if index.is_multiple_of(5) {
            "srflx"
        } else {
            "host"
        },
    }
}

fn render_candidate(template: &CandidateTemplate, generation: usize) -> Vec<u8> {
    format!(
        "candidate:{} {} {} {} {} {} typ {} generation {}",
        template.foundation,
        template.component,
        template.protocol,
        template.priority,
        template.address,
        template.port,
        template.candidate_type,
        generation % 32
    )
    .into_bytes()
}
