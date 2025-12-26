use crate::*;

#[cfg(feature = "bevy")]
use bevy::prelude::*;
#[cfg(feature = "bevy")]
use bevy::asset::AssetServer;

#[cfg(feature = "bevy")]
use bevy_kira_audio::prelude::{AudioSource as KiraAudioSource, *};
#[cfg(feature = "bevy")]
use bevy::input::mouse::MouseMotion;

#[cfg(feature = "bevy")]
#[derive(Debug, Clone, Copy, Resource)]
pub struct IaidoSettings {
    pub seed: u32,
    pub dpi: f32,
    pub ai: bool,
}

#[cfg(feature = "bevy")]
impl Default for IaidoSettings {
    fn default() -> Self { Self { seed: 0xA1D0_5EED, dpi: 320.0, ai: true } }
}

#[cfg(feature = "bevy")]
#[derive(Resource)]
pub struct DuelRuntime {
    pub machine: DuelMachine,
    pub swipe: SwipeDetector,
    cfg: SwipeConfig,
    ai_rng: XorShift32,
    ai_plan: Option<AiPlan>,
    ai_profile: AiProfile,
}

#[cfg(feature = "bevy")]
#[derive(Event)]
pub struct GoCue;

#[cfg(feature = "bevy")]
#[derive(Event)]
pub struct SlashCue { pub actor: Actor }

#[cfg(feature = "bevy")]
#[derive(Event)]
pub struct ClashCue;

use crate::types::Direction as GameDirection;

#[cfg(feature = "bevy")]
#[derive(Event)]
pub struct InputDetected {
    pub actor: Actor,
    pub dir: GameDirection,
}

#[cfg(feature = "bevy")]
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum DebugState {
    #[default]
    Off,
    Stats,
    Animation,
}

#[cfg(feature = "bevy")]
pub struct IaidoPlugin;

#[cfg(feature = "bevy")]
impl Plugin for IaidoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IaidoSettings>()
            .init_resource::<DebugState>()
            .insert_resource(ClearColor(Color::BLACK))
            .init_resource::<TouchTracker>()
            .add_event::<GoCue>()
            .add_event::<SlashCue>()
            .add_event::<ClashCue>()
            .add_event::<InputDetected>()
            .add_plugins(bevy_kira_audio::AudioPlugin)
            .add_plugins(hud::systems())
            .add_plugins(visuals::VisualsPlugin)
            .add_systems(Startup, (setup, setup_audio))
            .add_systems(Update, (
                update_time,
                read_input,
                drive_ai,
                advance_duel,
                react_outcomes,
                react_audio,
                toggle_debug_state,
            ));
    }
}

#[cfg(feature = "bevy")]
fn toggle_debug_state(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugState>,
) {
    if keys.just_pressed(KeyCode::KeyD) {
        *state = match *state {
            DebugState::Off => DebugState::Stats,
            DebugState::Stats => DebugState::Animation,
            DebugState::Animation => DebugState::Off,
        };
        println!("Debug State: {:?}", *state);
    }
}

#[cfg(feature = "bevy")]
fn setup(mut commands: Commands, settings: Res<IaidoSettings>) {
    let now_ms = 0;
    let machine = DuelMachine::new(DuelConfig { seed: settings.seed, clash: true }, now_ms);
    let swipe = SwipeDetector::new();
    let cfg = SwipeConfig { dpi: settings.dpi };
    let ai_rng = XorShift32::new(settings.seed ^ 0xDEADBEEF);
    let ai_profile = DUMB;
    commands.insert_resource(DuelRuntime { machine, swipe, cfg, ai_rng, ai_plan: None, ai_profile });
}

// Audio setup and reactions
#[cfg(feature = "bevy")]
#[derive(Resource, Default)]
struct AudioHandles {
    _wind: Option<Handle<KiraAudioSource>>,
    go: Option<Handle<KiraAudioSource>>,
    draw: Option<Handle<KiraAudioSource>>,
    hit: Option<Handle<KiraAudioSource>>,
    clash: Option<Handle<KiraAudioSource>>,
}

#[cfg(feature = "bevy")]
fn setup_audio(mut commands: Commands, assets: Res<AssetServer>, audio: Res<Audio>) {
    // Attempt to load if present; missing assets are acceptable (silent fallback)
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let load_if_exists = |rel: &str| -> Option<Handle<KiraAudioSource>> {
        if base.join(rel).exists() {
            Some(assets.load::<KiraAudioSource>(rel.to_string()))
        } else {
            None
        }
    };
    let wind = load_if_exists("audio/wind.ogg");
    let go = load_if_exists("audio/go.ogg");
    let draw = load_if_exists("audio/draw.ogg");
    let hit = load_if_exists("audio/hit.ogg");
    let clash = load_if_exists("audio/clash.ogg");
    commands.insert_resource(AudioHandles { _wind: wind.clone(), go, draw, hit, clash });
    // Start wind loop quietly if available
    if let Some(wind) = wind {
        audio.play(wind).with_volume(0.2).looped();
    }
}

#[cfg(feature = "bevy")]
fn update_time(mut rt: ResMut<DuelRuntime>, time: Res<Time>) {
    let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
    rt.machine.tick(now_ms);
}

// Track previous touch positions to calculate delta
#[cfg(feature = "bevy")]
#[derive(Resource, Default)]
struct TouchTracker {
    last_pos: std::collections::HashMap<u64, Vec2>,
}

#[cfg(feature = "bevy")]
fn read_input(
    mut rt: ResMut<DuelRuntime>,
    mut touches: EventReader<TouchInput>,
    mut tracker: ResMut<TouchTracker>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    mut input_tx: EventWriter<InputDetected>,
    time: Res<Time>,
) {
    let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;

    // Keys: Check combinations first
    let up = keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW);
    let down = keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS);
    let left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    let right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);

    let any_just = keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) ||
                   keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) ||
                   keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) ||
                   keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD);

    let mut key_dir = None;
    if any_just {
        if up && down { key_dir = Some(GameDirection::UpDown); }
        else if left && right { key_dir = Some(GameDirection::LeftRight); }
        else if up && right { key_dir = Some(GameDirection::UpRight); }
        else if up && left { key_dir = Some(GameDirection::UpLeft); }
        else if down && right { key_dir = Some(GameDirection::DownRight); }
        else if down && left { key_dir = Some(GameDirection::DownLeft); }
        else if up { key_dir = Some(GameDirection::Up); }
        else if down { key_dir = Some(GameDirection::Down); }
        else if left { key_dir = Some(GameDirection::Left); }
        else if right { key_dir = Some(GameDirection::Right); }
    }

    if let Some(dir) = key_dir {
        rt.machine.on_swipe(Actor::Human, dir, now_ms);
        input_tx.send(InputDetected { actor: Actor::Human, dir });
    }

    let dt_ms = (time.delta_seconds_f64() * 1000.0) as u64;
    for ev in touches.read() {
        match ev.phase {
            bevy::input::touch::TouchPhase::Started => {
                tracker.last_pos.insert(ev.id, ev.position);
            }
            bevy::input::touch::TouchPhase::Moved => {
                if let Some(last) = tracker.last_pos.get(&ev.id) {
                    let dx = ev.position.x - last.x;
                    let dy = ev.position.y - last.y;
                    let sample = SwipeSample { dt_ms, dx, dy };
                    let cfg = rt.cfg.clone();
                    if let Some(dir) = rt.swipe.update(&cfg, sample) {
                        rt.machine.on_swipe(Actor::Human, dir, now_ms);
                        input_tx.send(InputDetected { actor: Actor::Human, dir });
                    }
                }
                tracker.last_pos.insert(ev.id, ev.position);
            }
            bevy::input::touch::TouchPhase::Ended | bevy::input::touch::TouchPhase::Canceled => {
                tracker.last_pos.remove(&ev.id);
            }
        }
    }

    // Desktop mouse-drag adapter: hold left button and move to generate swipe deltas
    if mouse_buttons.pressed(MouseButton::Left) {
        for m in mouse_motion.read() {
            let sample = SwipeSample { dt_ms, dx: m.delta.x, dy: m.delta.y };
            let cfg = rt.cfg.clone();
            if let Some(dir) = rt.swipe.update(&cfg, sample) {
                rt.machine.on_swipe(Actor::Human, dir, now_ms);
                input_tx.send(InputDetected { actor: Actor::Human, dir });
            }
        }
    } else {
        // Reset detector when button released to avoid stale state
        rt.swipe.reset();
    }
}

#[cfg(feature = "bevy")]
fn drive_ai(
    mut rt: ResMut<DuelRuntime>,
    mut input_tx: EventWriter<InputDetected>,
    time: Res<Time>,
) {
    // If AI has already swiped for this round, do nothing
    if rt.machine.ai_swipe.is_some() {
        return;
    }

    let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
    // Plan AI on GO
    if let Some(go) = rt.machine.go_ts_ms {
        if rt.ai_plan.is_none() {
            rt.ai_plan = Some(plan_for_go(rt.ai_profile, &mut rt.ai_rng));
        }
        if let Some(plan) = rt.ai_plan.clone() {
            if now_ms >= go + plan.reaction_ms {
                let dir = plan.decide_dir(rt.machine.ai_opening, rt.ai_rng);
                rt.machine.on_swipe(Actor::Ai, dir, now_ms);
                input_tx.send(InputDetected { actor: Actor::Ai, dir });
                rt.ai_plan = None;
            }
        }
    } else {
        rt.ai_plan = None;
    }
}

#[cfg(feature = "bevy")]
fn advance_duel(mut rt: ResMut<DuelRuntime>, mut go_tx: EventWriter<GoCue>, time: Res<Time>) {
    if matches!(rt.machine.phase, DuelPhase::GoSignal) {
        go_tx.send(GoCue);
        let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
        rt.machine.tick(now_ms); // advance into input window immediately
    }
}

#[cfg(feature = "bevy")]
fn react_outcomes(
    rt: Res<DuelRuntime>,
    mut clash_tx: EventWriter<ClashCue>,
    mut slash_tx: EventWriter<SlashCue>,
) {
    if let Some(last) = rt.machine.round_results.last() {
        match last.outcome {
            Outcome::Clash => { clash_tx.send(ClashCue); },
            Outcome::HumanWin | Outcome::WrongAi | Outcome::EarlyAi => { slash_tx.send(SlashCue { actor: Actor::Human }); },
            Outcome::AiWin | Outcome::WrongHuman | Outcome::EarlyHuman => { slash_tx.send(SlashCue { actor: Actor::Ai }); },
        }
    }
}

// overlay moved to hud.rs via Gemini-generated plugin

#[cfg(feature = "bevy")]
fn react_audio(
    mut go_rx: EventReader<GoCue>,
    mut slash_rx: EventReader<SlashCue>,
    mut clash_rx: EventReader<ClashCue>,
    handles: Res<AudioHandles>,
    audio: Res<Audio>,
) {
    for _ in go_rx.read() {
        if let Some(h) = &handles.go { audio.play(h.clone()); }
    }
    for _ in slash_rx.read() {
        // Draw + hit sequence
        if let Some(d) = &handles.draw { audio.play(d.clone()); }
        if let Some(h) = &handles.hit { audio.play(h.clone()); }
    }
    for _ in clash_rx.read() {
        if let Some(c) = &handles.clash { audio.play(c.clone()); }
    }
}
