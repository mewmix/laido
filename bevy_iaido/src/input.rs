use bevy::input::mouse::MouseButtonInput;
use bevy::input::touch::Touches;
use bevy::prelude::*;

use crate::config::DeviceMetrics;
use crate::types::SwipeDir;

#[derive(Resource, Default, Debug, Clone)]
pub struct SwipeState {
    pub tracking: bool,
    pub committed: bool,
    pub committed_dir: SwipeDir,
    pub start_pos: Vec2,
    pub start_time: f64,
}

impl SwipeState {
    pub fn reset(&mut self) {
        self.tracking = false;
        self.committed = false;
        self.committed_dir = SwipeDir::None;
    }
    pub fn begin(&mut self, start_pos: Vec2, time: f64) {
        self.tracking = true;
        self.committed = false;
        self.committed_dir = SwipeDir::None;
        self.start_pos = start_pos;
        self.start_time = time;
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SwipeState::default());
    }
}

pub fn current_pointer_pos(windows: &Query<&Window>) -> Vec2 {
    if let Ok(win) = windows.get_single() {
        if let Some(pos) = win.cursor_position() {
            return pos;
        }
    }
    Vec2::ZERO
}

pub fn classify(delta: Vec2) -> SwipeDir {
    if delta.length_squared() == 0.0 { return SwipeDir::None; }
    if delta.x.abs() > delta.y.abs() {
        if delta.x > 0.0 { SwipeDir::Right } else { SwipeDir::Left }
    } else {
        if delta.y > 0.0 { SwipeDir::Up } else { SwipeDir::Down }
    }
}

pub fn poll_swipe(
    windows: &Query<&Window>,
    touches: &Res<Touches>,
    metrics: &Res<DeviceMetrics>,
    direction_lock_ms: u64,
    min_swipe_mm: f32,
    now: f64,
    swipe: &mut ResMut<SwipeState>,
) -> SwipeDir {
    if !swipe.tracking { return SwipeDir::None; }

    // Determine current pointer position: prefer touch
    let mut current = None;
    for t in touches.iter() {
        current = Some(t.position());
        break;
    }
    let current = current.unwrap_or_else(|| current_pointer_pos(windows));
    let delta = current - swipe.start_pos;

    if !swipe.committed {
        let dt_ms = ((now - swipe.start_time) * 1000.0) as u64;
        let threshold_px = metrics.mm_to_px(min_swipe_mm);
        if dt_ms >= direction_lock_ms || delta.length() >= threshold_px {
            swipe.committed_dir = classify(delta);
            swipe.committed = true;
        }
    }
    if swipe.committed { swipe.committed_dir } else { SwipeDir::None }
}

