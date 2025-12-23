use crate::*;

#[cfg(feature = "bevy")]
use bevy::prelude::*;

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
            .add_systems(Startup, setup)
            .add_systems(Update, (update_time, read_input, drive_ai, advance_duel, react_outcomes));
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

#[cfg(feature = "bevy")]
fn update_time(mut rt: ResMut<DuelRuntime>, time: Res<Time>) {
    let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
    rt.machine.tick(now_ms);
}

#[cfg(feature = "bevy")]
fn read_input(
    mut rt: ResMut<DuelRuntime>,
    mut touches: EventReader<TouchInput>,
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

