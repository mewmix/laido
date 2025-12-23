use crate::*;

#[cfg(feature = "bevy")]
use bevy::prelude::*;
#[cfg(feature = "bevy")]
use bevy::asset::AssetServer;

#[cfg(feature = "bevy")]
use bevy_kira_audio::prelude::*;

#[cfg(feature = "bevy")]
mod hud;

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
struct DuelRuntime {
    machine: DuelMachine,
    swipe: SwipeDetector,
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

#[cfg(feature = "bevy")]
pub struct IaidoPlugin;

#[cfg(feature = "bevy")]
impl Plugin for IaidoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IaidoSettings>()
            .insert_resource(ClearColor(Color::BLACK))
            .add_event::<GoCue>()
            .add_event::<SlashCue>()
            .add_event::<ClashCue>()
            .add_plugins(bevy_kira_audio::AudioPlugin)
            .add_plugins(hud::systems())
            .add_systems(Startup, (setup, setup_visuals, setup_audio))
            .add_systems(Update, (
                update_time,
                read_input,
                drive_ai,
                advance_duel,
                react_outcomes,
                update_visuals,
                react_audio,
            ));
    }
}

#[cfg(feature = "bevy")]
fn setup(mut commands: Commands, settings: Res<IaidoSettings>) {
    commands.spawn(Camera2d);
    let now_ms = 0;
    let machine = DuelMachine::new(DuelConfig { seed: settings.seed, clash: true }, now_ms);
    let swipe = SwipeDetector::new();
    let cfg = SwipeConfig { dpi: settings.dpi };
    let ai_rng = XorShift32::new(settings.seed ^ 0xDEADBEEF);
    let ai_profile = SKILLED;
    commands.insert_resource(DuelRuntime { machine, swipe, cfg, ai_rng, ai_plan: None, ai_profile });
}

// Minimal silhouettes and hit flash state
#[cfg(feature = "bevy")]
#[derive(Component)]
struct HumanSilhouette;

#[cfg(feature = "bevy")]
#[derive(Component)]
struct AiSilhouette;

#[cfg(feature = "bevy")]
#[derive(Resource, Default)]
struct VisualFlash { human_ms: u64, ai_ms: u64, clash_ms: u64 }

#[cfg(feature = "bevy")]
fn setup_visuals(mut commands: Commands) {
    // Human on left, AI on right; simple rectangles as silhouettes
    commands.spawn((Sprite::from_color(Color::GRAY), Transform::from_xyz(-150.0, 0.0, 0.0), HumanSilhouette));
    commands.spawn((Sprite::from_color(Color::GRAY), Transform::from_xyz(150.0, 0.0, 0.0), AiSilhouette));
    commands.insert_resource(VisualFlash::default());
}

// Audio setup and reactions
#[cfg(feature = "bevy")]
#[derive(Resource, Default)]
struct AudioHandles {
    wind: Option<Handle<AudioSource>>,
    go: Option<Handle<AudioSource>>,
    draw: Option<Handle<AudioSource>>,
    hit: Option<Handle<AudioSource>>,
    clash: Option<Handle<AudioSource>>,
}

#[cfg(feature = "bevy")]
fn setup_audio(mut commands: Commands, assets: Res<AssetServer>, audio: Res<Audio>) {
    // Attempt to load; missing assets are acceptable (silent fallback)
    let wind = assets.load::<AudioSource>("audio/wind.ogg");
    let go = assets.load::<AudioSource>("audio/go.ogg");
    let draw = assets.load::<AudioSource>("audio/draw.ogg");
    let hit = assets.load::<AudioSource>("audio/hit.ogg");
    let clash = assets.load::<AudioSource>("audio/clash.ogg");
    commands.insert_resource(AudioHandles { wind: Some(wind.clone()), go: Some(go), draw: Some(draw), hit: Some(hit), clash: Some(clash) });
    // Start wind loop quietly
    audio.play(wind).with_volume(0.2).looped();
}

#[cfg(feature = "bevy")]
fn update_time(mut rt: ResMut<DuelRuntime>, time: Res<Time>) {
    let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
    rt.machine.tick(now_ms);
}

#[cfg(feature = "bevy")]
fn read_input(
    mut rt: ResMut<DuelRuntime>,
    mut touches: EventReader<TouchInput>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let dt_ms = (time.delta_seconds_f64() * 1000.0) as u64;
    for ev in touches.read() {
        match ev.phase {
            bevy::input::touch::TouchPhase::Moved => {
                // Using delta is not directly provided; in production track previous positions per touch ID.
                // Here we approximate with zero because detailed input is out of scope.
                let sample = SwipeSample { dt_ms, dx: ev.delta.x, dy: ev.delta.y };
                if let Some(dir) = rt.swipe.update(&rt.cfg, sample) {
                    let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
                    rt.machine.on_swipe(Actor::Human, dir, now_ms);
                }
            }
            _ => {}
        }
    }

    // Desktop mouse-drag adapter: hold left button and move to generate swipe deltas
    if mouse_buttons.pressed(MouseButton::Left) {
        for m in mouse_motion.read() {
            let sample = SwipeSample { dt_ms, dx: m.delta.x, dy: m.delta.y };
            if let Some(dir) = rt.swipe.update(&rt.cfg, sample) {
                let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
                rt.machine.on_swipe(Actor::Human, dir, now_ms);
            }
        }
    } else {
        // Reset detector when button released to avoid stale state
        rt.swipe.reset();
    }
}

#[cfg(feature = "bevy")]
fn drive_ai(mut rt: ResMut<DuelRuntime>, time: Res<Time>) {
    let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
    // Plan AI on GO
    if let Some(go) = rt.machine.go_ts_ms {
        if rt.ai_plan.is_none() {
            rt.ai_plan = Some(plan_for_go(rt.ai_profile, &mut rt.ai_rng));
        }
        if let Some(plan) = rt.ai_plan.clone() {
            if now_ms >= go + plan.reaction_ms {
                let dir = plan.decide_dir(rt.machine.opening, rt.ai_rng);
                rt.machine.on_swipe(Actor::Ai, dir, now_ms);
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
            Outcome::Clash => clash_tx.send(ClashCue),
            Outcome::HumanWin | Outcome::WrongAi | Outcome::EarlyAi => slash_tx.send(SlashCue { actor: Actor::Human }),
            Outcome::AiWin | Outcome::WrongHuman | Outcome::EarlyHuman => slash_tx.send(SlashCue { actor: Actor::Ai }),
        }
    }
}

#[cfg(feature = "bevy")]
fn update_visuals(
    time: Res<Time>,
    mut vf: ResMut<VisualFlash>,
    mut humans: Query<&mut Sprite, With<HumanSilhouette>>,
    mut ais: Query<&mut Sprite, (With<AiSilhouette>, Without<HumanSilhouette>)>,
    mut slashes: EventReader<SlashCue>,
    mut clashes: EventReader<ClashCue>,
) {
    let dt_ms = (time.delta_seconds_f64() * 1000.0) as u64;
    for e in slashes.read() {
        match e.actor { Actor::Human => vf.ai_ms = 200, Actor::Ai => vf.human_ms = 200 }
    }
    for _ in clashes.read() { vf.clash_ms = 100; }

    vf.human_ms = vf.human_ms.saturating_sub(dt_ms);
    vf.ai_ms = vf.ai_ms.saturating_sub(dt_ms);
    vf.clash_ms = vf.clash_ms.saturating_sub(dt_ms);

    let base = if vf.clash_ms > 0 { Color::YELLOW } else { Color::GRAY };
    if let Ok(mut s) = humans.get_single_mut() {
        s.color = if vf.human_ms > 0 { Color::RED } else { base };
    }
    if let Ok(mut s) = ais.get_single_mut() {
        s.color = if vf.ai_ms > 0 { Color::RED } else { base };
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
    for e in slash_rx.read() {
        // Draw + hit sequence
        if let Some(d) = &handles.draw { audio.play(d.clone()); }
        if let Some(h) = &handles.hit { audio.play(h.clone()); }
    }
    for _ in clash_rx.read() {
        if let Some(c) = &handles.clash { audio.play(c.clone()); }
    }
}
