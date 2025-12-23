use bevy::prelude::*;
use rand::prelude::*;

use crate::ai::{AIAgent, AIProfile};
use crate::combat::{resolve};
use crate::config::{DeviceMetrics, TimingConfig};
use crate::input::{poll_swipe, SwipeState};
use crate::logging::{DuelLogger, RoundLog};
use crate::types::{DuelPhase, Opening, RoundOutcome, SwipeDir};
use crate::events::{GoEvent, EarlyInputEvent, ResolveEvent, RoundTransitionEvent};

#[derive(Resource)]
pub struct MatchCfg { pub best_of: u32 }

#[derive(Resource, Default)]
pub struct MatchState {
    pub player_wins: u32,
    pub ai_wins: u32,
    pub round_index: u32,
    pub in_clash: bool,
    pub match_seed: u64,
}

#[derive(Resource, Default)]
pub struct RoundState {
    pub phase: DuelPhase,
    pub opening: Option<Opening>,
    pub delay_end: f64,
    pub go_ts: f64,
    pub window_end: f64,
    pub player_dir: Option<(SwipeDir, f64)>,
    pub ai_dir: Option<(SwipeDir, f64)>,
}

#[derive(Resource)]
pub struct AIState { pub agent: AIAgent }

#[derive(Resource)]
pub struct RngState { pub rng: StdRng }

pub struct DuelPlugin;

impl Plugin for DuelPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MatchCfg { best_of: 3 })
            .insert_resource(MatchState::default())
            .insert_resource(RoundState::default())
            .insert_resource(DuelLogger::new(random()))
            .insert_resource(DeviceMetrics::default())
            .add_event::<GoEvent>()
            .add_event::<EarlyInputEvent>()
            .add_event::<ResolveEvent>()
            .add_event::<RoundTransitionEvent>()
            .add_systems(Startup, setup_ai)
            .add_systems(Update, duel_update);
    }
}

fn setup_ai(mut commands: Commands, cfg: Res<TimingConfig>, mut logger: ResMut<DuelLogger>, mut m: ResMut<MatchState>) {
    let seed: u64 = random();
    m.match_seed = seed;
    *logger = DuelLogger::new(seed);
    let agent = AIAgent::new(
        seed ^ 0xA11CE,
        AIProfile::Skilled,
        (
            cfg.novice_mean_ms, cfg.novice_wrong_pct,
            cfg.skilled_mean_ms, cfg.skilled_wrong_pct,
            cfg.master_mean_ms, cfg.master_wrong_pct,
        ),
    );
    commands.insert_resource(AIState { agent });
    commands.insert_resource(RngState { rng: StdRng::seed_from_u64(seed ^ 0xC0FFEE) });
}

fn now(time: &Res<Time>) -> f64 { time.elapsed_seconds_f64() }

fn duel_update(
    time: Res<Time>,
    cfg: Res<TimingConfig>,
    match_cfg: Res<MatchCfg>,
    mut m: ResMut<MatchState>,
    mut r: ResMut<RoundState>,
    mut swipe: ResMut<SwipeState>,
    mut ai_state: ResMut<AIState>,
    metrics: Res<DeviceMetrics>,
    mut logger: ResMut<DuelLogger>,
    windows: Query<&Window>,
    touches: Res<Touches>,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut rng_state: ResMut<RngState>,
    mut go_events: EventWriter<GoEvent>,
    mut early_events: EventWriter<EarlyInputEvent>,
    mut resolve_events: EventWriter<ResolveEvent>,
    mut round_events: EventWriter<RoundTransitionEvent>,
) {
    let t = now(&time);

    // Initialize
    if matches!(r.phase, DuelPhase::Reset | DuelPhase::MatchEnd) {
        m.player_wins = 0; m.ai_wins = 0; m.round_index = 0; m.in_clash = false;
        r.phase = DuelPhase::Standoff;
        swipe.reset();
        return;
    }

    // Match end check
    let needed = (match_cfg.best_of / 2) + 1;
    if m.player_wins >= needed || m.ai_wins >= needed || m.round_index >= match_cfg.best_of {
        r.phase = DuelPhase::MatchEnd;
        logger.flush_to_disk();
        // simple restart gesture
        if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Return) || touches.any_just_pressed() || mouse.just_pressed(MouseButton::Left) {
            r.phase = DuelPhase::Reset;
        }
        return;
    }

    match r.phase {
        DuelPhase::Standoff => {
            // Setup round
            m.round_index += 1;
            // deterministic opening & delay using seeded RNG
            let roll: u8 = rng_state.rng.gen();
            r.opening = Some(match roll % 4 { 0 => Opening::HighGuard, 1 => Opening::LowGuard, 2 => Opening::LeftGuard, _ => Opening::RightGuard });
            // Random delay
            let (min_ms, max_ms) = if m.in_clash { (cfg.clash_delay_min_ms, cfg.clash_delay_max_ms) } else { (cfg.delay_min_ms, cfg.delay_max_ms) };
            let delay_ms = if max_ms > min_ms { rng_state.rng.gen_range(min_ms..max_ms) } else { min_ms };
            r.delay_end = t + (delay_ms as f64 / 1000.0);
            r.phase = DuelPhase::RandomDelay;
        }
        DuelPhase::RandomDelay => {
            // Early input = auto-loss
            // Any motion before GO: we approximate by detecting any touch move or mouse drag length>0
            // For simplicity, check touches delta via just presence (UI: simplify). If touch exists, consider early.
            if (touches.any_just_pressed() || mouse.just_pressed(MouseButton::Left)) && !swipe.tracking {
                apply_and_log(&mut m, &mut r, &mut logger, RoundOutcome::EarlyPlayerLoss, SwipeDir::None, 0.0, SwipeDir::None, 0.0);
                early_events.send(EarlyInputEvent);
                m.ai_wins += 1;
                r.phase = DuelPhase::ResultFlash; r.delay_end = t + (cfg.result_flash_ms as f64 / 1000.0); m.in_clash = false; return;
            }
            if t >= r.delay_end { r.phase = DuelPhase::GoSignal; }
        }
        DuelPhase::GoSignal => {
            r.go_ts = t;
            swipe.begin(pointer_pos(&windows), t);
            let window_ms = if m.in_clash { cfg.clash_input_window_ms } else { cfg.input_window_ms };
            r.window_end = r.go_ts + (window_ms as f64 / 1000.0);
            if let Some(op) = r.opening { go_events.send(GoEvent { opening: op }); }

            // Precompute AI decision/time
            let ai_rt_ms = ai_state.agent.sample_reaction_ms();
            let ai_ts = r.go_ts + (ai_rt_ms as f64 / 1000.0);
            if ai_ts <= r.window_end {
                let ai_dir = ai_state.agent.decide_direction(r.opening.unwrap());
                r.ai_dir = Some((ai_dir, ai_ts));
            } else {
                r.ai_dir = None;
            }
            r.phase = DuelPhase::InputWindow;
        }
        DuelPhase::InputWindow => {
            if r.player_dir.is_none() {
                let dir = poll_swipe(&windows, &touches, &metrics, cfg.direction_lock_ms, cfg.min_swipe_distance_mm, t, &mut swipe);
                if dir != SwipeDir::None { r.player_dir = Some((dir, t)); }
            }
            if t >= r.window_end || r.player_dir.is_some() && r.ai_dir.is_some() {
                r.phase = DuelPhase::Resolution;
            }
        }
        DuelPhase::Resolution => {
            // Determine outcome
            let mut outcome = RoundOutcome::Timeout;
            let mut clash = false;
            let (p_dir, p_ts) = r.player_dir.unwrap_or((SwipeDir::None, 0.0));
            let (a_dir, a_ts) = r.ai_dir.unwrap_or((SwipeDir::None, 0.0));

            if r.player_dir.is_none() && r.ai_dir.is_some() { outcome = RoundOutcome::AIWin; }
            else if r.player_dir.is_some() && r.ai_dir.is_none() { outcome = RoundOutcome::PlayerWin; }
            else if r.player_dir.is_none() && r.ai_dir.is_none() { outcome = RoundOutcome::Timeout; }
            else {
                let prt = ((p_ts - r.go_ts) * 1000.0) as i32;
                let art = ((a_ts - r.go_ts) * 1000.0) as i32;
                let res = resolve(r.opening.unwrap(), p_dir, a_dir, prt, art, cfg.equal_tolerance_ms);
                outcome = res.outcome; clash = res.is_clash;
            }

            // Apply
            match outcome {
                RoundOutcome::PlayerWin => m.player_wins += 1,
                RoundOutcome::AIWin => m.ai_wins += 1,
                _ => {}
            }
            // Log
            logger.append(RoundLog { round_index: m.round_index, opening: r.opening.unwrap(), go_timestamp: r.go_ts,
                player_dir: p_dir, player_input_ts: p_ts, ai_dir: a_dir, ai_input_ts: a_ts, outcome, clash, seed: m.match_seed });
            resolve_events.send(ResolveEvent { outcome, clash });

            // Setup next phase
            r.phase = DuelPhase::ResultFlash;
            r.delay_end = t + (cfg.result_flash_ms as f64 / 1000.0);
            m.in_clash = clash;
        }
        DuelPhase::ResultFlash => {
            if t >= r.delay_end {
                r.phase = DuelPhase::NextRound;
                round_events.send(RoundTransitionEvent);
            }
        }
        DuelPhase::NextRound => {
            // Between-round delay
            r.phase = DuelPhase::Standoff;
            swipe.reset();
            r.opening = None; r.player_dir = None; r.ai_dir = None;
            // extra delay between rounds
            r.delay_end = t + (cfg.next_round_ms as f64 / 1000.0);
            // wait out next_round_ms in RandomDelay
            r.phase = DuelPhase::RandomDelay;
        }
        DuelPhase::MatchEnd => {}
        DuelPhase::Reset => {}
    }
}

fn apply_and_log(
    m: &mut MatchState,
    r: &mut RoundState,
    logger: &mut DuelLogger,
    outcome: RoundOutcome,
    p_dir: SwipeDir,
    p_ts: f64,
    a_dir: SwipeDir,
    a_ts: f64,
) {
    match outcome { RoundOutcome::PlayerWin => m.player_wins += 1, RoundOutcome::AIWin => m.ai_wins += 1, _ => {} }
    logger.append(RoundLog { round_index: m.round_index, opening: r.opening.unwrap_or(Opening::HighGuard), go_timestamp: r.go_ts,
        player_dir: p_dir, player_input_ts: p_ts, ai_dir: a_dir, ai_input_ts: a_ts, outcome, clash: outcome == RoundOutcome::Clash, seed: 0 });
}

fn pointer_pos(windows: &Query<&Window>) -> Vec2 {
    if let Ok(win) = windows.get_single() { if let Some(p) = win.cursor_position() { return p; } }
    Vec2::ZERO
}
