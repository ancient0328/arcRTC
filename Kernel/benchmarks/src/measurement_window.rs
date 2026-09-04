//! Benchmark measurement policy を executable lane window へ変換します。

use std::time::Duration;

use crate::{cases_for_lane, BenchmarkLane};

const BENCHMARK_SCENARIO_POLICY: &str =
    include_str!("../../tools/benchmark/benchmark-scenario-policy.toml");
const EXPECTED_POLICY_OWNER: &str = "\"tools/benchmark\"";

/// Policy source が固定する lane 全体の measurement window です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkMeasurementWindow {
    warmup_seconds: u64,
    sample_seconds: u64,
    cooldown_seconds: u64,
}

impl BenchmarkMeasurementWindow {
    /// Policy source の `[measurement_window]` を fallback なしで解析します。
    pub fn parse(policy_source: &str) -> Result<Self, BenchmarkWindowError> {
        let mut section_count = 0usize;
        let mut in_measurement_window = false;
        let mut owner = None;
        let mut warmup_seconds = None;
        let mut sample_seconds = None;
        let mut cooldown_seconds = None;

        for raw_line in policy_source.lines() {
            let line = raw_line.split('#').next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') {
                if !line.ends_with(']') {
                    return Err(BenchmarkWindowError::MalformedPolicyEntry);
                }
                in_measurement_window = line == "[measurement_window]";
                if in_measurement_window {
                    section_count += 1;
                    if section_count > 1 {
                        return Err(BenchmarkWindowError::MeasurementWindowSectionDuplicated);
                    }
                }
                continue;
            }

            if !in_measurement_window {
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or(BenchmarkWindowError::MalformedPolicyEntry)?;
            let key = key.trim();
            let value = value.trim();
            match key {
                "owner" => set_once(&mut owner, value, BenchmarkWindowError::OwnerDuplicated)?,
                "warmup_seconds" => set_once(
                    &mut warmup_seconds,
                    parse_unsigned(value, BenchmarkWindowError::WarmupInvalid)?,
                    BenchmarkWindowError::WarmupDuplicated,
                )?,
                "sample_seconds" => set_once(
                    &mut sample_seconds,
                    parse_unsigned(value, BenchmarkWindowError::SampleInvalid)?,
                    BenchmarkWindowError::SampleDuplicated,
                )?,
                "cooldown_seconds" => set_once(
                    &mut cooldown_seconds,
                    parse_unsigned(value, BenchmarkWindowError::CooldownInvalid)?,
                    BenchmarkWindowError::CooldownDuplicated,
                )?,
                _ => return Err(BenchmarkWindowError::UnexpectedMeasurementWindowField),
            }
        }

        if section_count == 0 {
            return Err(BenchmarkWindowError::MeasurementWindowSectionMissing);
        }
        let owner = owner.ok_or(BenchmarkWindowError::OwnerMissing)?;
        if owner != EXPECTED_POLICY_OWNER {
            return Err(BenchmarkWindowError::OwnerMismatch);
        }
        let warmup_seconds = warmup_seconds.ok_or(BenchmarkWindowError::WarmupMissing)?;
        let sample_seconds = sample_seconds.ok_or(BenchmarkWindowError::SampleMissing)?;
        let cooldown_seconds = cooldown_seconds.ok_or(BenchmarkWindowError::CooldownMissing)?;
        if warmup_seconds == 0 {
            return Err(BenchmarkWindowError::WarmupZero);
        }
        if sample_seconds == 0 {
            return Err(BenchmarkWindowError::SampleZero);
        }
        if cooldown_seconds == 0 {
            return Err(BenchmarkWindowError::CooldownZero);
        }

        Ok(Self {
            warmup_seconds,
            sample_seconds,
            cooldown_seconds,
        })
    }

    /// Lane case 数に対して total window を丸めずに等分します。
    pub fn allocate(
        self,
        lane: BenchmarkLane,
        case_count: usize,
    ) -> Result<BenchmarkLaneWindow, BenchmarkWindowError> {
        if case_count == 0 {
            return Err(BenchmarkWindowError::EmptyLane(lane));
        }
        let case_count_seconds = case_count as u64;
        if !self.warmup_seconds.is_multiple_of(case_count_seconds) {
            return Err(BenchmarkWindowError::WarmupNotDivisible {
                total_seconds: self.warmup_seconds,
                case_count,
            });
        }
        if !self.sample_seconds.is_multiple_of(case_count_seconds) {
            return Err(BenchmarkWindowError::SampleNotDivisible {
                total_seconds: self.sample_seconds,
                case_count,
            });
        }

        Ok(BenchmarkLaneWindow {
            lane,
            case_count,
            total_warmup_seconds: self.warmup_seconds,
            total_sample_seconds: self.sample_seconds,
            warmup_per_case_seconds: self.warmup_seconds / case_count_seconds,
            measurement_per_case_seconds: self.sample_seconds / case_count_seconds,
            cooldown_seconds: self.cooldown_seconds,
        })
    }

    /// Lane 全体の warmup target 秒数です。
    pub const fn warmup_seconds(self) -> u64 {
        self.warmup_seconds
    }

    /// Lane 全体の sample target 秒数です。
    pub const fn sample_seconds(self) -> u64 {
        self.sample_seconds
    }

    /// Lane 全体で一度だけ適用する cooldown 秒数です。
    pub const fn cooldown_seconds(self) -> u64 {
        self.cooldown_seconds
    }
}

/// Registry-derived case 数へ割り当てた executable lane window です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkLaneWindow {
    lane: BenchmarkLane,
    case_count: usize,
    total_warmup_seconds: u64,
    total_sample_seconds: u64,
    warmup_per_case_seconds: u64,
    measurement_per_case_seconds: u64,
    cooldown_seconds: u64,
}

impl BenchmarkLaneWindow {
    /// この window が対象とする lane です。
    pub const fn lane(self) -> BenchmarkLane {
        self.lane
    }

    /// Executable registry から得た lane case 数です。
    pub const fn case_count(self) -> usize {
        self.case_count
    }

    /// Lane 全体の warmup target 秒数です。
    pub const fn total_warmup_seconds(self) -> u64 {
        self.total_warmup_seconds
    }

    /// Lane 全体の sample target 秒数です。
    pub const fn total_sample_seconds(self) -> u64 {
        self.total_sample_seconds
    }

    /// Criterion が各 case に適用する warmup 秒数です。
    pub const fn warmup_per_case_seconds(self) -> u64 {
        self.warmup_per_case_seconds
    }

    /// Criterion が各 case に適用する measurement 秒数です。
    pub const fn measurement_per_case_seconds(self) -> u64 {
        self.measurement_per_case_seconds
    }

    /// Lane 終了後に一度だけ適用する cooldown 秒数です。
    pub const fn cooldown_seconds(self) -> u64 {
        self.cooldown_seconds
    }

    /// Criterion が各 case に適用する warmup duration です。
    pub const fn warmup_per_case_duration(self) -> Duration {
        Duration::from_secs(self.warmup_per_case_seconds)
    }

    /// Criterion が各 case に適用する measurement duration です。
    pub const fn measurement_per_case_duration(self) -> Duration {
        Duration::from_secs(self.measurement_per_case_seconds)
    }

    /// Lane 終了後に一度だけ適用する cooldown duration です。
    pub const fn cooldown_duration(self) -> Duration {
        Duration::from_secs(self.cooldown_seconds)
    }
}

/// Measurement policy または registry allocation の closed failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkWindowError {
    /// `[measurement_window]` が存在しません。
    MeasurementWindowSectionMissing,
    /// `[measurement_window]` が重複しています。
    MeasurementWindowSectionDuplicated,
    /// Measurement window 内の entry が `key = value` 形式ではありません。
    MalformedPolicyEntry,
    /// Measurement window に許可されない field があります。
    UnexpectedMeasurementWindowField,
    /// `owner` が存在しません。
    OwnerMissing,
    /// `owner` が重複しています。
    OwnerDuplicated,
    /// `owner` が benchmark policy owner と一致しません。
    OwnerMismatch,
    /// `warmup_seconds` が存在しません。
    WarmupMissing,
    /// `warmup_seconds` が重複しています。
    WarmupDuplicated,
    /// `warmup_seconds` が符号なし整数ではありません。
    WarmupInvalid,
    /// `warmup_seconds` が0です。
    WarmupZero,
    /// `sample_seconds` が存在しません。
    SampleMissing,
    /// `sample_seconds` が重複しています。
    SampleDuplicated,
    /// `sample_seconds` が符号なし整数ではありません。
    SampleInvalid,
    /// `sample_seconds` が0です。
    SampleZero,
    /// `cooldown_seconds` が存在しません。
    CooldownMissing,
    /// `cooldown_seconds` が重複しています。
    CooldownDuplicated,
    /// `cooldown_seconds` が符号なし整数ではありません。
    CooldownInvalid,
    /// `cooldown_seconds` が0です。
    CooldownZero,
    /// Lane に executable case がありません。
    EmptyLane(BenchmarkLane),
    /// Warmup total が lane case 数で割り切れません。
    WarmupNotDivisible {
        /// Policy が固定した lane total 秒数です。
        total_seconds: u64,
        /// Registry から得た lane case 数です。
        case_count: usize,
    },
    /// Sample total が lane case 数で割り切れません。
    SampleNotDivisible {
        /// Policy が固定した lane total 秒数です。
        total_seconds: u64,
        /// Registry から得た lane case 数です。
        case_count: usize,
    },
}

/// Embedded policy と executable registry から lane window を構築します。
pub fn benchmark_lane_window(
    lane: BenchmarkLane,
) -> Result<BenchmarkLaneWindow, BenchmarkWindowError> {
    let measurement_window = BenchmarkMeasurementWindow::parse(BENCHMARK_SCENARIO_POLICY)?;
    measurement_window.allocate(lane, cases_for_lane(lane).len())
}

fn parse_unsigned(value: &str, error: BenchmarkWindowError) -> Result<u64, BenchmarkWindowError> {
    value.parse::<u64>().map_err(|_| error)
}

fn set_once<T: Copy>(
    slot: &mut Option<T>,
    value: T,
    duplicate_error: BenchmarkWindowError,
) -> Result<(), BenchmarkWindowError> {
    if slot.replace(value).is_some() {
        return Err(duplicate_error);
    }
    Ok(())
}
