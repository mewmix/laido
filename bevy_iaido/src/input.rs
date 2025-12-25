use crate::config::{mm_to_px, MIN_SWIPE_MM, DIRECTION_LOCK_MS};
use crate::types::Direction;

#[derive(Copy, Clone, Debug, Default)]
pub struct SwipeSample {
    pub dt_ms: u64,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct SwipeConfig {
    pub dpi: f32,
}

impl SwipeConfig {
    pub fn min_distance_px(&self) -> f32 { mm_to_px(MIN_SWIPE_MM, self.dpi) }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SwipeState { Idle, Moving, Locked }

#[derive(Copy, Clone, Debug)]
pub struct SwipeDetector {
    pub state: SwipeState,
    pub lock_dir: Option<Direction>,
    pub elapsed_ms: u64,
    pub accum_dx: f32,
    pub accum_dy: f32,
    pub committed: bool,
}

impl SwipeDetector {
    pub fn new() -> Self {
        Self { state: SwipeState::Idle, lock_dir: None, elapsed_ms: 0, accum_dx: 0.0, accum_dy: 0.0, committed: false }
    }

    pub fn reset(&mut self) {
        self.state = SwipeState::Idle;
        self.lock_dir = None;
        self.elapsed_ms = 0;
        self.accum_dx = 0.0;
        self.accum_dy = 0.0;
        self.committed = false;
    }

    pub fn update(&mut self, cfg: &SwipeConfig, sample: SwipeSample) -> Option<Direction> {
        match self.state {
            SwipeState::Idle => {
                if sample.dx != 0.0 || sample.dy != 0.0 {
                    self.state = SwipeState::Moving;
                }
            }
            SwipeState::Moving => {
                self.elapsed_ms += sample.dt_ms;
                self.accum_dx += sample.dx;
                self.accum_dy += sample.dy;

                if !self.committed && self.elapsed_ms >= DIRECTION_LOCK_MS {
                    self.lock_dir = Some(primary_direction(self.accum_dx, self.accum_dy));
                    self.committed = true;
                }
                if self.committed {
                    let dist2 = self.accum_dx * self.accum_dx + self.accum_dy * self.accum_dy;
                    let min = cfg.min_distance_px();
                    if dist2 >= min * min {
                        self.state = SwipeState::Locked;
                        return self.lock_dir;
                    }
                }
            }
            SwipeState::Locked => {}
        }
        None
    }
}

pub fn primary_direction(dx: f32, dy: f32) -> Direction {
    let adx = dx.abs();
    let ady = dy.abs();
    let max = adx.max(ady);
    if max == 0.0 { return Direction::Up; } // Fallback

    // Check for diagonal: if the smaller component is at least 40% of the larger
    if adx > 0.4 * max && ady > 0.4 * max {
        if dx > 0.0 {
            return if dy > 0.0 { Direction::UpRight } else { Direction::DownRight };
        } else {
            return if dy > 0.0 { Direction::UpLeft } else { Direction::DownLeft };
        }
    }

    if adx > ady {
        if dx > 0.0 { Direction::Right } else { Direction::Left }
    } else {
        if dy > 0.0 { Direction::Up } else { Direction::Down }
    }
}
