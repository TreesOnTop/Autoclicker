use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

pub const MIN_INTERVAL_MS: i32 = 2;
pub const MAX_CPS: i32 = 500;

pub const SPEED_MODE_DELAY: i32 = 0;
pub const SPEED_MODE_RATE: i32 = 1;
pub const MAX_DELAY_HOURS: u16 = 999;

pub const RATE_UNIT_SECOND: i32 = 0;
pub const RATE_UNIT_MINUTE: i32 = 1;
pub const RATE_UNIT_HOUR: i32 = 2;
pub const RATE_UNIT_DAY: i32 = 3;

pub const DEFAULT_RATE_COUNT: i32 = 10;
pub const DEFAULT_RATE_UNIT: i32 = RATE_UNIT_SECOND;

#[derive(Clone)]
pub struct SpeedState {
    pub interval_ms: Arc<AtomicI32>,
    pub mode: Arc<AtomicI32>,
    pub delay_h: Arc<AtomicI32>,
    pub delay_m: Arc<AtomicI32>,
    pub delay_s: Arc<AtomicI32>,
    pub delay_ms: Arc<AtomicI32>,
    pub rate_count: Arc<AtomicI32>,
    pub rate_unit: Arc<AtomicI32>,
}

impl SpeedState {
    pub fn new(
        mode: i32,
        delay_h: u16,
        delay_m: u8,
        delay_s: u8,
        delay_ms: u16,
        rate_count: i32,
        rate_unit: i32,
    ) -> Self {
        let mode = normalize_speed_mode(mode);
        let delay_h = delay_h.min(MAX_DELAY_HOURS) as i32;
        let delay_m = delay_m.min(59) as i32;
        let delay_s = delay_s.min(59) as i32;
        let delay_ms = delay_ms.min(999) as i32;
        let rate_unit = normalize_rate_unit(rate_unit);
        let rate_count = clamp_rate_count(rate_count, rate_unit);
        let interval_ms = compute_interval_ms(
            mode,
            delay_h as u16,
            delay_m as u8,
            delay_s as u8,
            delay_ms as u16,
            rate_count,
            rate_unit,
        );
        Self {
            interval_ms: Arc::new(AtomicI32::new(interval_ms)),
            mode: Arc::new(AtomicI32::new(mode)),
            delay_h: Arc::new(AtomicI32::new(delay_h)),
            delay_m: Arc::new(AtomicI32::new(delay_m)),
            delay_s: Arc::new(AtomicI32::new(delay_s)),
            delay_ms: Arc::new(AtomicI32::new(delay_ms)),
            rate_count: Arc::new(AtomicI32::new(rate_count)),
            rate_unit: Arc::new(AtomicI32::new(rate_unit)),
        }
    }

    pub fn sync_interval(&self) -> i32 {
        let interval = compute_interval_ms(
            self.mode.load(Ordering::Relaxed),
            self.delay_h
                .load(Ordering::Relaxed)
                .clamp(0, MAX_DELAY_HOURS as i32) as u16,
            self.delay_m.load(Ordering::Relaxed).clamp(0, 59) as u8,
            self.delay_s.load(Ordering::Relaxed).clamp(0, 59) as u8,
            self.delay_ms.load(Ordering::Relaxed).clamp(0, 999) as u16,
            self.rate_count.load(Ordering::Relaxed),
            self.rate_unit.load(Ordering::Relaxed),
        );
        self.interval_ms.store(interval, Ordering::Relaxed);
        interval
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DelayParts {
    pub hours: u16,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
}

impl DelayParts {
    pub fn from_interval_ms(interval_ms: i32) -> Self {
        let ms = interval_ms.max(0) as u64;
        let hours = (ms / 3_600_000).min(MAX_DELAY_HOURS as u64) as u16;
        let rem = ms % 3_600_000;
        let minutes = (rem / 60_000) as u8;
        let rem = rem % 60_000;
        let seconds = (rem / 1000) as u8;
        let milliseconds = (rem % 1000) as u16;
        Self {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }
}

pub fn clamp_interval_ms(ms: i64) -> i32 {
    if ms < MIN_INTERVAL_MS as i64 {
        MIN_INTERVAL_MS
    } else if ms > i32::MAX as i64 {
        i32::MAX
    } else {
        ms as i32
    }
}

pub fn interval_from_delay(hours: u16, minutes: u8, seconds: u8, milliseconds: u16) -> i32 {
    let total = (hours.min(MAX_DELAY_HOURS) as i64) * 3_600_000
        + (minutes as i64) * 60_000
        + (seconds as i64) * 1000
        + milliseconds as i64;
    clamp_interval_ms(total)
}

pub fn unit_ms(rate_unit: i32) -> i64 {
    match rate_unit {
        RATE_UNIT_MINUTE => 60_000,
        RATE_UNIT_HOUR => 3_600_000,
        RATE_UNIT_DAY => 86_400_000,
        _ => 1000,
    }
}

pub fn clamp_rate_count(count: i32, _rate_unit: i32) -> i32 {
    count.clamp(1, MAX_CPS)
}

pub fn interval_from_rate(count: i32, rate_unit: i32) -> i32 {
    let count = clamp_rate_count(count, rate_unit) as i64;
    let ms = unit_ms(rate_unit) / count;
    clamp_interval_ms(ms)
}

pub fn compute_interval_ms(
    speed_mode: i32,
    hours: u16,
    minutes: u8,
    seconds: u8,
    milliseconds: u16,
    rate_count: i32,
    rate_unit: i32,
) -> i32 {
    if speed_mode == SPEED_MODE_RATE {
        interval_from_rate(rate_count, rate_unit)
    } else {
        interval_from_delay(hours, minutes, seconds, milliseconds)
    }
}

pub fn normalize_rate_unit(unit: i32) -> i32 {
    match unit {
        RATE_UNIT_MINUTE | RATE_UNIT_HOUR | RATE_UNIT_DAY => unit,
        _ => RATE_UNIT_SECOND,
    }
}

pub fn normalize_speed_mode(mode: i32) -> i32 {
    if mode == SPEED_MODE_RATE {
        SPEED_MODE_RATE
    } else {
        SPEED_MODE_DELAY
    }
}

