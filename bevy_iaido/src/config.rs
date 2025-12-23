#[derive(Debug, Clone)]
pub struct TimingConfig {
    pub delay_min_ms: u64,
    pub delay_max_ms: u64,
    pub input_window_ms: u64,
    pub clash_input_window_ms: u64,
    pub clash_delay_min_ms: u64,
    pub clash_delay_max_ms: u64,
    pub result_flash_ms: u64,
    pub next_round_ms: u64,
    pub min_swipe_distance_mm: f32,
    pub direction_lock_ms: u64,
    pub equal_tolerance_ms: i32,

    pub novice_mean_ms: i32,
    pub novice_wrong_pct: f32,
    pub skilled_mean_ms: i32,
    pub skilled_wrong_pct: f32,
    pub master_mean_ms: i32,
    pub master_wrong_pct: f32,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            delay_min_ms: 600,
            delay_max_ms: 1400,
            input_window_ms: 120,
            clash_input_window_ms: 80,
            clash_delay_min_ms: 300,
            clash_delay_max_ms: 600,
            result_flash_ms: 300,
            next_round_ms: 500,
            min_swipe_distance_mm: 7.0,
            direction_lock_ms: 20,
            equal_tolerance_ms: 5,
            novice_mean_ms: 280,
            novice_wrong_pct: 0.15,
            skilled_mean_ms: 190,
            skilled_wrong_pct: 0.05,
            master_mean_ms: 140,
            master_wrong_pct: 0.0,
        }
    }
}

// Device metrics used to convert mm→px
#[derive(Debug, Clone)]
pub struct DeviceMetrics {
    pub ppi: f32, // default 160 PPI if unknown
}

impl Default for DeviceMetrics {
    fn default() -> Self {
        Self { ppi: 160.0 }
    }
}

impl DeviceMetrics {
    pub fn mm_to_px(&self, mm: f32) -> f32 {
        // 1 inch = 25.4 mm
        (mm * self.ppi) / 25.4
    }
}

