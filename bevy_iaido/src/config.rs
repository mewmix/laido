use core::time::Duration;

// Timing constants (monotonic, deterministic)
pub const START_DELAY_MS: u64 = 3000;
pub const RANDOM_DELAY_MIN_MS: u64 = 600;
pub const RANDOM_DELAY_MAX_MS: u64 = 1400;
pub const INPUT_WINDOW_MS: u64 = 120;

pub const CLASH_DELAY_MIN_MS: u64 = 300;
pub const CLASH_DELAY_MAX_MS: u64 = 600;
pub const CLASH_INPUT_WINDOW_MS: u64 = 80;

pub const DIRECTION_LOCK_MS: u64 = 20; // lock after ~20ms of motion
pub const TIE_WINDOW_MS: u64 = 5; // ±5ms considered equal

// Match config
pub const ROUNDS_TO_WIN: u8 = 2; // best of 3

// Input thresholds
// Minimum swipe distance in millimeters; scale by device DPI
pub const MIN_SWIPE_MM: f32 = 7.0; // between 6–8 mm

// Utility to convert mm to pixels given DPI (dots per inch)
// 1 inch = 25.4 mm
pub fn mm_to_px(mm: f32, dpi: f32) -> f32 {
    let inches = mm / 25.4;
    inches * dpi
}

pub fn ms(ms: u64) -> Duration { Duration::from_millis(ms) }
