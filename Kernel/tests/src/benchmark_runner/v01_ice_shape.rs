//! v0.1.2 `ice_pool_bench` と同じ Criterion surface を v0.2 側で観測する比較専用 module です。
//!
//! production core へ旧 ICE pool の責務を移植せず、test-side benchmark の分母だけを揃えます。
//! ここで得る値は比較観測であり、v0.2 の完成・readiness・performance threshold には採用しません。

use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const V01_ICE_SHAPE_BENCH_FUNCTIONS: &[&str] = &[
    "pool_generate_candidate",
    "ondemand_generate_candidate",
    "bulk_generate_candidates",
];
pub const V01_ICE_SHAPE_GROUP: &str = "ice_candidate_generation";
pub const V01_ICE_SHAPE_ITERATION_COUNTS: &[u32] = &[100, 1000, 10000];

thread_local! {
    static COMPARABLE_GENERATOR: ComparableFastIdGenerator = ComparableFastIdGenerator::new();
}

/// v0.1.2 の fast-id 生成コストを比較対象へ含めるための test-side mirror です。
pub struct ComparableFastIdGenerator {
    state: RefCell<u64>,
    collision_detector: RefCell<HashSet<u64>>,
    generation_count: AtomicU64,
}

impl ComparableFastIdGenerator {
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|error| error.duration())
            .as_nanos() as u64;

        Self {
            state: RefCell::new(seed),
            collision_detector: RefCell::new(HashSet::with_capacity(10000)),
            generation_count: AtomicU64::new(0),
        }
    }

    pub fn generate_ice_candidate_id(&self) -> String {
        let mut state = self.state.borrow_mut();
        let mut detector = self.collision_detector.borrow_mut();

        loop {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;

            if detector.contains(&*state) {
                continue;
            }

            detector.insert(*state);
            self.generation_count.fetch_add(1, Ordering::Relaxed);

            if detector.len() > 10000 {
                detector.clear();
                detector.insert(*state);
            }

            let state_value = *state;
            return format!("ice_{state_value:016x}");
        }
    }
}

impl Default for ComparableFastIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_comparable_fast_ice_id() -> String {
    COMPARABLE_GENERATOR.with(|generator| generator.generate_ice_candidate_id())
}

/// v0.1.2 `IceCandidateTemplate` の candidate 生成 shape を v0.2 test-side で再現します。
#[derive(Debug, Clone)]
pub struct ComparableIceCandidateTemplate {
    pub candidate_id: String,
    pub priority_base: u32,
    pub protocol: String,
    pub candidate_type: String,
    pub foundation: String,
    pub created_at: Instant,
}

impl ComparableIceCandidateTemplate {
    pub fn new(template_id: u32) -> Self {
        let candidate_id = generate_comparable_fast_ice_id();
        let foundation = format!("foundation_{template_id}");

        let candidate_types = ["host", "srflx", "relay"];
        let candidate_type =
            candidate_types[template_id as usize % candidate_types.len()].to_string();

        let protocols = ["UDP", "TCP"];
        let protocol = protocols[template_id as usize % protocols.len()].to_string();

        let priority_base = match candidate_type.as_str() {
            "host" => 126 << 24,
            "srflx" => 100 << 24,
            "relay" => 0 << 24,
            _ => 50 << 24,
        };

        Self {
            candidate_id,
            priority_base,
            protocol,
            candidate_type,
            foundation,
            created_at: Instant::now(),
        }
    }

    pub fn generate_candidate(&self, ip: &str, port: u16) -> ComparableIceCandidate {
        let priority = self.calculate_priority(ip);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|error| error.duration())
            .as_nanos()
            % 1000000;

        let ip_formatted = ip.replace('.', "_");
        let unique_candidate_id = format!(
            "{}_{}_{}_{}",
            self.candidate_id, ip_formatted, port, timestamp
        );

        ComparableIceCandidate {
            candidate_id: unique_candidate_id,
            foundation: self.foundation.clone(),
            protocol: self.protocol.clone(),
            priority,
            ip: ip.to_string(),
            port,
            candidate_type: self.candidate_type.clone(),
            related_address: None,
            related_port: None,
            created_at: Instant::now(),
        }
    }

    fn calculate_priority(&self, ip: &str) -> u32 {
        let type_preference = match self.candidate_type.as_str() {
            "host" => 126,
            "srflx" => 100,
            "prflx" => 110,
            "relay" => 0,
            _ => 50,
        };

        let local_preference = if ip.contains(':') { 50 } else { 65535 };
        let component_id = 1u32;

        (type_preference << 24) | (local_preference << 8) | (256 - component_id)
    }
}

/// v0.1.2 `IceCandidate` と同じ測定対象フィールドを持つ比較専用 candidate です。
#[derive(Debug, Clone)]
pub struct ComparableIceCandidate {
    pub candidate_id: String,
    pub foundation: String,
    pub protocol: String,
    pub priority: u32,
    pub ip: String,
    pub port: u16,
    pub candidate_type: String,
    pub related_address: Option<String>,
    pub related_port: Option<u16>,
    pub created_at: Instant,
}

impl ComparableIceCandidate {
    pub fn to_sdp_string(&self) -> String {
        let related = if let (Some(addr), Some(port)) = (&self.related_address, &self.related_port)
        {
            format!(" raddr {addr} rport {port}")
        } else {
            String::new()
        };

        let foundation = &self.foundation;
        let protocol = self.protocol.to_uppercase();
        let priority = self.priority;
        let ip = &self.ip;
        let port = self.port;
        let candidate_type = &self.candidate_type;
        format!(
            "candidate:{foundation} 1 {protocol} {priority} {ip} {port} typ {candidate_type}{related}"
        )
    }
}

/// v0.1.2 `IceCandidatePool` の pool/ondemand 比較 shape を保持する test-side pool です。
pub struct ComparableIceCandidatePool {
    templates: Vec<ComparableIceCandidateTemplate>,
    next_index: AtomicUsize,
    stats: Arc<Mutex<ComparablePoolStatistics>>,
}

impl ComparableIceCandidatePool {
    pub fn new() -> Self {
        let start_time = Instant::now();
        let templates: Vec<_> = (0..100).map(ComparableIceCandidateTemplate::new).collect();
        let generation_time = start_time.elapsed();

        let stats = ComparablePoolStatistics {
            total_templates: 100,
            pool_hits: 0,
            pool_misses: 0,
            on_demand_generations: 0,
            total_candidates_generated: 0,
            average_generation_time_ns: generation_time.as_nanos() / 100,
            created_at: start_time,
        };

        Self {
            templates,
            next_index: AtomicUsize::new(0),
            stats: Arc::new(Mutex::new(stats)),
        }
    }

    pub fn generate_candidate(&self, ip: &str, port: u16) -> ComparableIceCandidate {
        let start_time = Instant::now();

        let candidate = if self.templates.is_empty() {
            self.generate_on_demand(ip, port)
        } else {
            self.generate_from_pool(ip, port)
        };

        let generation_time = start_time.elapsed();
        self.update_statistics(generation_time, !self.templates.is_empty());

        candidate
    }

    pub fn generate_candidates(&self, interfaces: &[(String, u16)]) -> Vec<ComparableIceCandidate> {
        interfaces
            .iter()
            .map(|(ip, port)| self.generate_candidate(ip, *port))
            .collect()
    }

    pub fn get_statistics(&self) -> ComparablePoolStatistics {
        self.stats
            .lock()
            .expect("comparable pool statistics lock must not be poisoned")
            .clone()
    }

    fn generate_from_pool(&self, ip: &str, port: u16) -> ComparableIceCandidate {
        let index = self.next_index.fetch_add(1, Ordering::Relaxed) % self.templates.len();
        let template = &self.templates[index];
        template.generate_candidate(ip, port)
    }

    fn generate_on_demand(&self, ip: &str, port: u16) -> ComparableIceCandidate {
        let template = ComparableIceCandidateTemplate::new(0);
        template.generate_candidate(ip, port)
    }

    fn update_statistics(&self, generation_time: Duration, from_pool: bool) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_candidates_generated += 1;

            if from_pool {
                stats.pool_hits += 1;
            } else {
                stats.pool_misses += 1;
                stats.on_demand_generations += 1;
            }

            let total_generated = stats.total_candidates_generated as u128;
            let current_average = stats.average_generation_time_ns;
            let new_time_ns = generation_time.as_nanos();

            stats.average_generation_time_ns =
                (current_average * (total_generated - 1) + new_time_ns) / total_generated;
        }
    }
}

impl Default for ComparableIceCandidatePool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ComparablePoolStatistics {
    pub total_templates: usize,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub on_demand_generations: u64,
    pub total_candidates_generated: u64,
    pub average_generation_time_ns: u128,
    pub created_at: Instant,
}
