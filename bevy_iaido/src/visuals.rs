use bevy::prelude::*;
use bevy_tweening::*;
use bevy_tweening::lens::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{Actor, AttackCue, ClashCue, GoCue, SlashCue, InputDetected, DebugInputCue};
use crate::types::Direction as GameDirection;
use crate::plugin::{DuelRuntime, DebugState, AnimationEditMode};
use crate::combat::correct_direction_for;
#[cfg(feature = "bevy")]
use crate::touch::VirtualKey;
use bevy::window::PrimaryWindow;

pub struct VisualsPlugin;

const HUMAN_FRAMES_DIR: &str = "atlas/white_samurai";
const AI_FRAMES_DIR: &str = "atlas/red_samurai";
const AI_IDLE_FRAME: &str = "red_samurai__tile_3.png";
const AI_ATTACK_FRAMES: [&str; 3] = [
    "red_samurai__tile_0.png",
    "red_samurai__tile_1.png",
    "red_samurai__tile_2.png",
];
const AI_BLOCK_CHANCE: f32 = 0.15;
const AI_ATTACK_RANGE: f32 = 140.0;
const AI_ATTACK_COOLDOWN: f32 = 1.6;
const AI_APPROACH_SPEED: f32 = 120.0;
const AI_STOP_DISTANCE: f32 = 120.0;
const AI_DEATH_FRAMES: [&str; 4] = [
    "red_death_tile_0.png",
    "red_death_tile_1.png",
    "red_death_tile_2.png",
    "red_death_tile_3.png",
];
const AI_PARRY_FRAME: &str = "red_parry__tile_1.png";
const AI_HITS_TO_DEATH: u8 = 2;
const DEATH_FADE_SECONDS: f32 = 0.25;
const RESPAWN_FADE_SECONDS: f32 = 0.25;
const HIT_RANGE: f32 = 320.0;
const BLOCK_WINDOW_MS: u64 = 150;
const STAGGER_DISTANCE: f32 = 80.0;
const MIN_SEPARATION: f32 = 10.0;
const SEQUENCE_FRAME_TIME: f32 = 0.2;
const DASH_DISTANCE: f32 = 220.0;
const RUN_FRAME_TIME: f32 = 0.12;
const RUN_FRAMES: [&str; 4] = ["run_0.png", "run_1.png", "run_2.png", "run_3.png"];
const STAGE_LEFT_X: f32 = -520.0;
const STAGE_RIGHT_X: f32 = 520.0;
const BG_IMAGE_SIZE: Vec2 = Vec2::new(3168.0, 1344.0);
const GROUND_OFFSET_Y: f32 = 120.0;
const Z_PRESS_FRAME: &str = "up_attack_seq_1.png";
const Z_RELEASE_FRAME: &str = "up_attack_seq_2.png";
const X_PRESS_FRAME: &str = "up_attack_extended_seq_1.png";
const X_RELEASE_FRAME: &str = "up_attack_extended_seq_2.png";
const X_FOLLOW_FRAME: &str = "up_attack_extended_seq_3.png";
const IDLE_FRAME: &str = "forward-idle.png";
const BLOCK_PRESS_FRAME: &str = "block_forward.png";
const BLOCK_HOLD_FRAME: &str = "block_forward_2.png";
const BLOCK_HOLD_THRESHOLD: f32 = 0.2;
const BLOCK_DOWN_FRAME: &str = "block_down.png";
const BLOCK_HIT_FRAME: &str = "block_hit.png";
const BACK_HEAVY_FRAME: &str = "back_heavy_stance.png";
const S_PRESS_FRAME: &str = "duel.png";
const S_RELEASE_FRAMES: [&str; 2] = ["fast-attack-forward.png", "fast_attack_forward1.png"];
const S_DOUBLE_FRAMES: [&str; 2] = ["heavy_spin.png", "heavy_spin_2.png"];
const S_DOUBLE_RETURN: &str = "back_fast_stance.png";
const S_DOUBLE_WINDOW_MS: u64 = 250;
const SX_FRAMES: [&str; 2] = ["top_slash_heavy_1.png", "top_slash_heavy.png"];
const PARRY_FRAME: &str = "parry_1.png";
const PARRY_FRAME_ALT: &str = "parry_2.png";
const PARRY_COUNTER_FRAME: &str = "parry_counter_attack_z.png";
const PARRY_READY_SECONDS: f32 = 0.6;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TweeningPlugin)
            .add_systems(Startup, setup_scene)
            .add_systems(PostStartup, setup_characters)
            .add_systems(Update, (
                handle_go_cue,
                handle_slash_cue,
                handle_clash_cue,
                apply_camera_shake,
                handle_input_detected,
                handle_debug_input_cue,
                despawn_expired,
                reset_character_frames,
                update_character_stance,
                update_frame_sequences,
            ))
            .add_systems(Update, (
                update_block_hold,
                update_walk_input,
                update_run_animation,
                update_ai_approach,
                update_background_layout,
                update_ai_proximity,
                update_ai_idle,
                handle_hit_resolution,
                update_hit_flash,
                animation_tester,
            ));
        app.add_systems(Update, (update_death_respawn, update_respawn_fade_in));
        app.add_systems(Update, update_parry_state);
    }
}

fn get_sprite_index_from_dir(dir: GameDirection) -> usize {
    match dir {
        GameDirection::Up => 0,
        GameDirection::Down => 1,
        GameDirection::Left => 2,
        GameDirection::Right => 3,
        GameDirection::UpLeft => 0,
        GameDirection::UpRight => 0,
        GameDirection::DownLeft => 1,
        GameDirection::DownRight => 1,
        GameDirection::UpDown => 0,
        GameDirection::LeftRight => 2,
    }
}

fn update_character_stance(
    rt: Res<DuelRuntime>,
    mut char_q: Query<(&Character, &mut FrameIndex, &mut Handle<Image>), Without<ResetFrame>>,
    debug_state: Res<DebugState>,
    time: Res<Time>,
    frames: Res<FrameLibrary>,
) {
    if matches!(*debug_state, DebugState::Animation) { return; }

    for (character, mut frame_idx, mut texture) in char_q.iter_mut() {
        let frames = frames_for_actor(character.actor, &frames);
        let (dir, _is_combo) = match character.actor {
            Actor::Human => {
                if let Some(swipe) = &rt.machine.human_swipe {
                    (swipe.dir, true)
                } else {
                    (correct_direction_for(rt.machine.ai_opening), false)
                }
            }
            Actor::Ai => {
                if let Some(swipe) = &rt.machine.ai_swipe {
                    (swipe.dir, true)
                } else {
                    (correct_direction_for(rt.machine.human_opening), false)
                }
            }
        };

        if dir == GameDirection::UpDown {
            frame_idx.index = if (time.elapsed_seconds() * 5.0).sin() > 0.0 { 0 } else { 1 };
        } else if dir == GameDirection::LeftRight {
            frame_idx.index = if (time.elapsed_seconds() * 5.0).sin() > 0.0 { 2 } else { 3 };
        } else {
            frame_idx.index = get_sprite_index_from_dir(dir);
        }
        apply_frame(&frames, &mut frame_idx, &mut texture);
    }
}

fn get_pose_name(index: usize) -> &'static str {
    match index {
        0 => "Up",
        1 => "Down",
        2 => "Left",
        3 => "Right",
        _ => "Unknown",
    }
}

use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
struct AnimationEvents<'w> {
    slash: EventWriter<'w, SlashCue>,
    clash: EventWriter<'w, ClashCue>,
    attack: EventWriter<'w, AttackCue>,
    debug_input: EventWriter<'w, DebugInputCue>,
}

fn animation_tester(
    mut char_q: Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    mut move_q: Query<(&Character, &mut Transform, &mut OriginalTransform)>,
    keys: Res<ButtonInput<KeyCode>>,
    vkeys: Res<ButtonInput<VirtualKey>>,
    debug_state: Res<DebugState>,
    events: AnimationEvents,
    mut controller_state: ResMut<CharacterControllerState>,
    frames: Res<FrameLibrary>,
    mut commands: Commands,
    time: Res<Time>,
    edit_mode: Res<AnimationEditMode>,
    mut block_state: ResMut<BlockState>,
    mut stance_lock: ResMut<StanceLock>,
    mut parry_state: ResMut<ParryState>,
) {
    let AnimationEvents { slash: mut slash_tx, clash: mut clash_tx, attack: mut attack_tx, debug_input: mut debug_input_tx } = events;

    if !matches!(*debug_state, DebugState::Animation) { return; }

    if edit_mode.0 {
        let mut delta = 0;
        if keys.just_pressed(KeyCode::ArrowLeft) || vkeys.just_pressed(VirtualKey::Left) { delta = -1; }
        if keys.just_pressed(KeyCode::ArrowRight) || vkeys.just_pressed(VirtualKey::Right) { delta = 1; }

        if delta != 0 {
            for (_entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
                if matches!(character.actor, Actor::Human) {
                    let frames = &frames.human;
                    let new_idx = (frame_idx.index as i32 + delta)
                        .rem_euclid(frames.count() as i32) as usize;
                    frame_idx.index = new_idx;
                    apply_frame(&frames, &mut frame_idx, &mut texture);
                    println!("Human Atlas Index: {} - {}", new_idx, get_pose_name(new_idx));
                }
            }
        }
    }

    if edit_mode.0 {
        if keys.just_pressed(KeyCode::Space) || vkeys.just_pressed(VirtualKey::Space) {
            if let Some(idx) = current_human_index(&mut char_q) {
                controller_state.controller.slash_index = idx;
            }
            slash_tx.send(SlashCue { actor: Actor::Human });
        }
        if keys.just_pressed(KeyCode::Enter) || vkeys.just_pressed(VirtualKey::Enter) {
            if let Some(idx) = current_human_index(&mut char_q) {
                controller_state.controller.clash_index = idx;
            }
            clash_tx.send(ClashCue);
        }
    } else if keys.just_pressed(KeyCode::Space) || vkeys.just_pressed(VirtualKey::Space) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "SPACE DASH".to_string() });
        dash_forward(&mut move_q);
    }
    if (keys.just_pressed(KeyCode::KeyX) || vkeys.just_pressed(VirtualKey::X)) && (keys.pressed(KeyCode::KeyS) || vkeys.pressed(VirtualKey::S)) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "S+X DOWN".to_string() });
        if let Some(seq) = frames.human.sequence_indices(&SX_FRAMES) {
            if let Some(return_idx) = current_human_index(&mut char_q) {
                play_sequence_with_return_index(
                    Actor::Human,
                    seq,
                    return_idx,
                    &frames,
                    &mut char_q,
                    &mut commands,
                );
                maybe_ai_block(&frames, &mut char_q, &mut commands, &mut block_state, &time, &mut debug_input_tx);
                attack_tx.send(AttackCue { actor: Actor::Human });
            }
        } else {
            println!("Missing one or more top slash heavy frames.");
        }
        controller_state.x_armed = false;
    } else if keys.just_pressed(KeyCode::KeyX) || vkeys.just_pressed(VirtualKey::X) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "X DOWN".to_string() });
        if let Some(idx) = frames.human.index_for_name(X_PRESS_FRAME) {
            play_frame(Actor::Human, idx, &frames, &mut char_q, &mut commands);
            controller_state.x_armed = true;
            maybe_ai_block(&frames, &mut char_q, &mut commands, &mut block_state, &time, &mut debug_input_tx);
        } else {
            println!("Missing frame: {}", X_PRESS_FRAME);
        }
    }
    if (keys.just_pressed(KeyCode::KeyZ) || vkeys.just_pressed(VirtualKey::Z)) && parry_state.ready {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "PARRY COUNTER".to_string() });
        if let Some(idx) = frames.human.index_for_name(PARRY_COUNTER_FRAME) {
            play_frame(Actor::Human, idx, &frames, &mut char_q, &mut commands);
            attack_tx.send(AttackCue { actor: Actor::Human });
        } else {
            println!("Missing frame: {}", PARRY_COUNTER_FRAME);
        }
        parry_state.ready = false;
        controller_state.z_up_armed = false;
    } else if keys.just_pressed(KeyCode::KeyZ) || vkeys.just_pressed(VirtualKey::Z) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "Z DOWN".to_string() });
        if let Some(idx) = frames.human.index_for_name(Z_PRESS_FRAME) {
            play_frame(Actor::Human, idx, &frames, &mut char_q, &mut commands);
            controller_state.z_up_armed = true;
            maybe_ai_block(&frames, &mut char_q, &mut commands, &mut block_state, &time, &mut debug_input_tx);
        } else {
            println!("Missing frame: {}", Z_PRESS_FRAME);
        }
    }
    if (keys.just_released(KeyCode::KeyZ) || vkeys.just_released(VirtualKey::Z)) && controller_state.z_up_armed {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "Z UP".to_string() });
        if let Some(idx) = frames.human.index_for_name(Z_RELEASE_FRAME) {
            play_frame(Actor::Human, idx, &frames, &mut char_q, &mut commands);
            attack_tx.send(AttackCue { actor: Actor::Human });
        } else {
            println!("Missing frame: {}", Z_RELEASE_FRAME);
        }
        controller_state.z_up_armed = false;
    }
    if (keys.just_released(KeyCode::KeyX) || vkeys.just_released(VirtualKey::X)) && controller_state.x_armed {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "X UP".to_string() });
        if let Some(seq) = frames.human.sequence_indices(&[X_RELEASE_FRAME, X_FOLLOW_FRAME]) {
            play_sequence(Actor::Human, seq, &frames, &mut char_q, &mut commands);
            maybe_ai_block(&frames, &mut char_q, &mut commands, &mut block_state, &time, &mut debug_input_tx);
            attack_tx.send(AttackCue { actor: Actor::Human });
        } else {
            println!("Missing one or more extended release frames.");
        }
        controller_state.x_armed = false;
    }
    if keys.just_pressed(KeyCode::KeyS) || vkeys.just_pressed(VirtualKey::S) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "S DOWN".to_string() });
        let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
        let is_double = now_ms.saturating_sub(controller_state.s_last_press_ms) <= S_DOUBLE_WINDOW_MS;
        controller_state.s_last_press_ms = now_ms;
        controller_state.s_waiting_release = !is_double;
        controller_state.s_double_active = is_double;
        if is_double {
            debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "S DOUBLE".to_string() });
            if let Some(seq) = frames.human.sequence_indices(&S_DOUBLE_FRAMES) {
                if let Some(return_idx) = frames.human.index_for_name(S_DOUBLE_RETURN) {
                    stance_lock.index = Some(return_idx);
                    play_sequence_with_return_index(
                        Actor::Human,
                        seq,
                        return_idx,
                        &frames,
                        &mut char_q,
                        &mut commands,
                    );
                    maybe_ai_block(&frames, &mut char_q, &mut commands, &mut block_state, &time, &mut debug_input_tx);
                    attack_tx.send(AttackCue { actor: Actor::Human });
                } else {
                    println!("Missing frame: {}", S_DOUBLE_RETURN);
                }
            } else {
                println!("Missing one or more heavy spin frames.");
            }
        } else if let Some(idx) = frames.human.index_for_name(S_PRESS_FRAME) {
            stance_lock.index = Some(idx);
            play_frame_with_return_index(
                Actor::Human,
                idx,
                0.6,
                idx,
                &frames,
                &mut char_q,
                &mut commands,
            );
            maybe_ai_block(&frames, &mut char_q, &mut commands, &mut block_state, &time, &mut debug_input_tx);
        } else {
            println!("Missing frame: {}", S_PRESS_FRAME);
        }
    }
    if (keys.just_released(KeyCode::KeyS) || vkeys.just_released(VirtualKey::S)) && controller_state.s_waiting_release && !controller_state.s_double_active {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "S UP".to_string() });
        if let Some(seq) = frames.human.sequence_indices(&S_RELEASE_FRAMES) {
            if let Some(return_idx) = frames.human.index_for_name(S_PRESS_FRAME) {
                play_sequence_with_return_index(
                    Actor::Human,
                    seq,
                    return_idx,
                    &frames,
                    &mut char_q,
                    &mut commands,
                );
                maybe_ai_block(&frames, &mut char_q, &mut commands, &mut block_state, &time, &mut debug_input_tx);
                attack_tx.send(AttackCue { actor: Actor::Human });
            } else {
                println!("Missing frame: {}", S_PRESS_FRAME);
            }
        } else {
            println!("Missing one or more S release frames.");
        }
        controller_state.s_waiting_release = false;
    }
    if keys.just_pressed(KeyCode::KeyC) || vkeys.just_pressed(VirtualKey::C) {
        let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
        block_state.human_last_ms = now_ms;
        if keys.pressed(KeyCode::ArrowLeft) || vkeys.pressed(VirtualKey::Left) {
            debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "C+LEFT".to_string() });
            if let Some(idx) = frames.human.index_for_name(BACK_HEAVY_FRAME) {
                play_frame_with_duration(
                    Actor::Human,
                    idx,
                    0.4,
                    &frames,
                    &mut char_q,
                    &mut commands,
                );
            } else {
                println!("Missing frame: {}", BACK_HEAVY_FRAME);
            }
            controller_state.block_hold_active = false;
        } else if keys.pressed(KeyCode::ArrowDown) || vkeys.pressed(VirtualKey::Down) {
            debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "C+DOWN".to_string() });
            if let Some(idx) = frames.human.index_for_name(BLOCK_DOWN_FRAME) {
                play_frame_with_duration(
                    Actor::Human,
                    idx,
                    0.4,
                    &frames,
                    &mut char_q,
                    &mut commands,
                );
            } else {
                println!("Missing frame: {}", BLOCK_DOWN_FRAME);
            }
            controller_state.block_hold_active = false;
        } else {
            debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "C DOWN".to_string() });
            if let Some(idx) = frames.human.index_for_name(BLOCK_PRESS_FRAME) {
                play_frame_with_duration(
                    Actor::Human,
                    idx,
                    BLOCK_HOLD_THRESHOLD + 0.1,
                    &frames,
                    &mut char_q,
                    &mut commands,
                );
                controller_state.block_hold_active = true;
                controller_state.block_hold_elapsed = 0.0;
                controller_state.block_hold_second = false;
            } else {
                println!("Missing frame: {}", BLOCK_PRESS_FRAME);
            }
        }
    }
    if keys.just_released(KeyCode::KeyC) || vkeys.just_released(VirtualKey::C) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "C UP".to_string() });
        controller_state.block_hold_active = false;
    }
    if keys.just_pressed(KeyCode::KeyP) || vkeys.just_pressed(VirtualKey::P) {
        if let Err(err) = save_controller(&controller_state.controller_path, &controller_state.controller) {
            println!("Failed to save controller: {}", err);
        } else {
            println!("Saved controller: {}", controller_state.controller_path.display());
        }
    }
}

fn setup_characters(mut commands: Commands, frames: Res<FrameLibrary>) {
    let start_y = -180.0;
    spawn_character(&mut commands, Actor::Human, Vec2::new(-300.0, start_y), &frames.human, IDLE_FRAME, 1.0);
    spawn_character(&mut commands, Actor::Ai, Vec2::new(300.0, start_y), &frames.ai, AI_IDLE_FRAME, 1.0);
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct Lifetime(Timer);

#[derive(Component)]
struct ResetFrame {
    timer: Timer,
    return_index: usize,
}

#[derive(Component)]
struct HitFlash {
    timer: Timer,
}

#[derive(Component)]
pub(crate) struct DeathRespawn {
    stage: DeathStage,
    timer: Timer,
    respawn_x: f32,
}

#[derive(Component)]
pub(crate) struct RespawnFadeIn {
    timer: Timer,
}

#[derive(Clone, Copy)]
enum DeathStage {
    Sequence,
    FadeOut,
}

#[derive(Component)]
pub struct CameraShake {
    pub strength: f32,
    pub decay: f32,
}

#[derive(Component)]
pub struct Character {
    pub actor: Actor,
}

#[derive(Component)]
struct Grounded;

#[derive(Component)]
struct BackgroundSprite {
    size: Vec2,
}

#[derive(Resource)]
pub(crate) struct CharacterFrames {
    handles: Vec<Handle<Image>>,
    name_to_index: HashMap<String, usize>,
    names: Vec<String>,
}

#[derive(Resource)]
pub(crate) struct FrameLibrary {
    pub(crate) human: CharacterFrames,
    pub(crate) ai: CharacterFrames,
}

#[derive(Resource, Default)]
struct AiDemoState {
    cooldown: f32,
}

#[derive(Resource, Default)]
struct BlockState {
    human_last_ms: u64,
    ai_last_ms: u64,
}

#[derive(Resource)]
pub(crate) struct AiHealth {
    pub(crate) hits_remaining: u8,
}

#[derive(Resource)]
pub(crate) struct ParryState {
    pub(crate) ready: bool,
    timer: Timer,
    pub(crate) ai_ready: bool,
    ai_timer: Timer,
    human_parry_alt: bool,
}

#[derive(Resource, Default)]
struct MoveIntent {
    dir: f32,
}

#[derive(Resource, Default)]
struct RunAnimationFrames {
    human: Vec<usize>,
}

#[derive(Resource, Default)]
struct StanceLock {
    index: Option<usize>,
}

#[derive(Resource, Default)]
struct GroundY(f32);

#[derive(Resource, Clone)]
pub(crate) struct CharacterControllerState {
    pub(crate) controller: CharacterController,
    pub(crate) controller_path: PathBuf,
    pub(crate) controller_name: String,
    z_up_armed: bool,
    x_armed: bool,
    block_hold_active: bool,
    block_hold_elapsed: f32,
    block_hold_second: bool,
    s_last_press_ms: u64,
    s_waiting_release: bool,
    s_double_active: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CharacterController {
    pub(crate) slash_index: usize,
    pub(crate) clash_index: usize,
    pub(crate) fast_index: usize,
    pub(crate) heavy_index: usize,
    pub(crate) heavy_up_ready_index: usize,
    pub(crate) heavy_up_release_index: usize,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            slash_index: 2,
            clash_index: 0,
            fast_index: 0,
            heavy_index: 1,
            heavy_up_ready_index: 2,
            heavy_up_release_index: 3,
        }
    }
}

#[derive(Component)]
pub struct OriginalTransform(pub Vec3);

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Assets
    let human_frames = load_frames_for_folder(HUMAN_FRAMES_DIR, &asset_server);
    let ai_frames = load_frames_for_folder(AI_FRAMES_DIR, &asset_server);
    let run_frames = human_frames
        .sequence_indices(&RUN_FRAMES)
        .unwrap_or_default();
    commands.insert_resource(FrameLibrary {
        human: human_frames,
        ai: ai_frames,
    });
    commands.insert_resource(RunAnimationFrames { human: run_frames });
    commands.insert_resource(MoveIntent::default());
    commands.insert_resource(StanceLock::default());
    commands.insert_resource(GroundY::default());
    commands.insert_resource(AiHealth { hits_remaining: AI_HITS_TO_DEATH });
    commands.insert_resource(ParryState {
        ready: false,
        timer: Timer::from_seconds(PARRY_READY_SECONDS, TimerMode::Once),
        ai_ready: false,
        ai_timer: Timer::from_seconds(PARRY_READY_SECONDS, TimerMode::Once),
        human_parry_alt: false,
    });

    let controller_path = controller_path_for_folder(HUMAN_FRAMES_DIR);
    let controller = load_controller(&controller_path);
    let controller_name = controller_name_from_folder(HUMAN_FRAMES_DIR);
    commands.insert_resource(CharacterControllerState {
        controller,
        controller_path,
        controller_name,
        z_up_armed: false,
        x_armed: false,
        block_hold_active: false,
        block_hold_elapsed: 0.0,
        block_hold_second: false,
        s_last_press_ms: 0,
        s_waiting_release: false,
        s_double_active: false,
    });
    commands.insert_resource(AiDemoState::default());
    commands.insert_resource(BlockState::default());

    // Camera
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
        CameraShake { strength: 0.0, decay: 3.0 },
    ));

    // Background - Burning Village (static)
    let bg_texture = asset_server.load("background/burning_village_0.png");
    commands.spawn((
        SpriteBundle {
            texture: bg_texture,
            sprite: Sprite {
                custom_size: Some(BG_IMAGE_SIZE),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -10.0),
            ..default()
        },
        BackgroundSprite { size: BG_IMAGE_SIZE },
    ));

}

fn spawn_character(
    commands: &mut Commands,
    actor: Actor,
    pos: Vec2,
    frames: &CharacterFrames,
    idle_name: &str,
    initial_alpha: f32,
) -> Entity {
    let base_scale = Vec3::splat(0.4); // Scale down 512x512

    let idle_tween = Tween::new(
        EaseMethod::EaseFunction(EaseFunction::SineInOut),
        Duration::from_millis(1500),
        TransformScaleLens {
            start: base_scale,
            end: base_scale * Vec3::new(1.0, 1.05, 1.0),
        },
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

    let flip_x = matches!(actor, Actor::Ai);

    let idle_idx = if idle_name.is_empty() {
        0
    } else {
        frames.index_for_name(idle_name).unwrap_or(0)
    };
    let idle_texture = frames.get(idle_idx).unwrap_or_default();
    commands.spawn((
        SpriteBundle {
            texture: idle_texture,
            sprite: Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, initial_alpha),
                flip_x,
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, if matches!(actor, Actor::Human) { 1.0 } else { 0.0 }).with_scale(base_scale),
            ..default()
        },
        FrameIndex { index: idle_idx },
        Character { actor },
        Grounded,
        OriginalTransform(Vec3::new(pos.x, pos.y, if matches!(actor, Actor::Human) { 1.0 } else { 0.0 })),
        Animator::new(idle_tween),
    )).id()
}

fn handle_go_cue(
    mut go_rx: EventReader<GoCue>,
    mut camera_q: Query<&mut CameraShake, With<MainCamera>>,
) {
    for _ in go_rx.read() {
        if let Ok(mut shake) = camera_q.get_single_mut() {
            shake.strength = 1.0; // Subtle shake on GO
        }
    }
}

fn handle_slash_cue(
    mut commands: Commands,
    mut slash_rx: EventReader<SlashCue>,
    mut char_q: Query<(Entity, &Character, &OriginalTransform, &Transform, &mut Animator<Transform>, &mut FrameIndex, &mut Handle<Image>, Option<&DeathRespawn>, Option<&RespawnFadeIn>)>,
    controller_state: Res<CharacterControllerState>,
    frames: Res<FrameLibrary>,
    pos_q: Query<(&Character, &Transform)>,
) {
    for ev in slash_rx.read() {
        for (entity, character, original, transform, mut animator, mut frame_idx, mut texture, death, fade) in char_q.iter_mut() {
            if character.actor == ev.actor {
                if matches!(character.actor, Actor::Ai) && (death.is_some() || fade.is_some()) {
                    continue;
                }
                let start_pos = original.0;
                let mut lunge_dist = if matches!(character.actor, Actor::Human) { 250.0 } else { -250.0 };
                let mut opponent_x = None;
                for (c, t) in pos_q.iter() {
                    if c.actor != character.actor {
                        opponent_x = Some(t.translation.x);
                        break;
                    }
                }
                if let Some(ox) = opponent_x {
                    let current = transform.translation.x;
                    let desired = current + lunge_dist;
                    let clamped = if matches!(character.actor, Actor::Human) {
                        clamp_human_x(desired, ox)
                    } else {
                        clamp_ai_x(desired, ox)
                    };
                    lunge_dist = clamped - current;
                }
                let end_pos = start_pos + Vec3::new(lunge_dist, 0.0, 0.0);

                let lunge = Tween::new(
                    EaseMethod::EaseFunction(EaseFunction::ExponentialOut),
                    Duration::from_millis(150),
                    TransformPositionLens {
                        start: start_pos,
                        end: end_pos,
                    },
                );

                let ret = Tween::new(
                    EaseMethod::EaseFunction(EaseFunction::QuadraticInOut),
                    Duration::from_millis(300),
                    TransformPositionLens {
                        start: end_pos,
                        end: start_pos,
                    },
                );

                let attack_seq = lunge.then(ret);
                animator.set_tweenable(attack_seq);

                let actor_frames = frames_for_actor(character.actor, &frames);
                if matches!(character.actor, Actor::Ai) {
                    if let Some(seq) = ai_attack_sequence(&frames) {
                        let total = SEQUENCE_FRAME_TIME * (seq.len() as f32);
                        frame_idx.index = seq[0];
                        apply_frame(actor_frames, &mut frame_idx, &mut texture);
                        commands.entity(entity).remove::<FrameSequence>();
                        commands.entity(entity).remove::<ResetFrame>();
                        commands.entity(entity).insert(FrameSequence {
                            frames: seq,
                            next_index: 1,
                            timer: Timer::from_seconds(SEQUENCE_FRAME_TIME, TimerMode::Repeating),
                        });
                        commands.entity(entity).insert(ResetFrame {
                            timer: Timer::from_seconds(total, TimerMode::Once),
                            return_index: ai_idle_index(&frames),
                        });
                    } else {
                        frame_idx.index = ai_idle_index(&frames);
                        apply_frame(actor_frames, &mut frame_idx, &mut texture);
                    }
                } else {
                    frame_idx.index = controller_state.controller.slash_index;
                    apply_frame(actor_frames, &mut frame_idx, &mut texture);
                    commands.entity(entity).insert(ResetFrame {
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                        return_index: 0,
                    });
                }
            }
        }
    }
}

fn reset_character_frames(
    mut commands: Commands,
    time: Res<Time>,
    mut char_q: Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>, &mut ResetFrame)>,
    frames: Res<FrameLibrary>,
) {
    for (entity, character, mut frame_idx, mut texture, mut reset) in char_q.iter_mut() {
        reset.timer.tick(time.delta());
        if reset.timer.finished() {
            frame_idx.index = reset.return_index;
            apply_frame(frames_for_actor(character.actor, &frames), &mut frame_idx, &mut texture);
            commands.entity(entity).remove::<ResetFrame>();
        }
    }
}

fn handle_clash_cue(
    mut clash_rx: EventReader<ClashCue>,
    mut camera_q: Query<&mut CameraShake, With<MainCamera>>,
    mut commands: Commands,
    mut char_q: Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>, Option<&DeathRespawn>, Option<&RespawnFadeIn>)>,
    controller_state: Res<CharacterControllerState>,
    frames: Res<FrameLibrary>,
) {
    for _ in clash_rx.read() {
        if let Ok(mut shake) = camera_q.get_single_mut() {
            shake.strength = 4.0; // Violent shake
        }
        for (entity, character, mut frame_idx, mut texture, death, fade) in char_q.iter_mut() {
            let actor_frames = frames_for_actor(character.actor, &frames);
            if matches!(character.actor, Actor::Ai) && (death.is_some() || fade.is_some()) {
                continue;
            }
            if matches!(character.actor, Actor::Ai) {
                if let Some(seq) = ai_attack_sequence(&frames) {
                    let total = SEQUENCE_FRAME_TIME * (seq.len() as f32);
                    frame_idx.index = seq[0];
                    apply_frame(actor_frames, &mut frame_idx, &mut texture);
                    commands.entity(entity).remove::<FrameSequence>();
                    commands.entity(entity).remove::<ResetFrame>();
                    commands.entity(entity).insert(FrameSequence {
                        frames: seq,
                        next_index: 1,
                        timer: Timer::from_seconds(SEQUENCE_FRAME_TIME, TimerMode::Repeating),
                    });
                    commands.entity(entity).insert(ResetFrame {
                        timer: Timer::from_seconds(total, TimerMode::Once),
                        return_index: ai_idle_index(&frames),
                    });
                } else {
                    frame_idx.index = ai_idle_index(&frames);
                    apply_frame(actor_frames, &mut frame_idx, &mut texture);
                    commands.entity(entity).insert(ResetFrame {
                        timer: Timer::from_seconds(0.2, TimerMode::Once),
                        return_index: ai_idle_index(&frames),
                    });
                }
            } else {
                frame_idx.index = controller_state.controller.clash_index;
                apply_frame(actor_frames, &mut frame_idx, &mut texture);
                commands.entity(entity).insert(ResetFrame {
                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                    return_index: 0,
                });
            }
        }
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 1.0, 0.5),
                    custom_size: Some(Vec2::new(150.0, 150.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 50.0, 10.0)
                    .with_rotation(Quat::from_rotation_z(0.78)),
                ..default()
            },
            Animator::new(Tween::new(
                EaseMethod::EaseFunction(EaseFunction::QuadraticOut),
                Duration::from_millis(200),
                SpriteColorLens {
                    start: Color::srgb(1.0, 1.0, 0.5),
                    end: Color::srgba(1.0, 1.0, 0.5, 0.0),
                },
            )),
        ));
    }
}

fn apply_camera_shake(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CameraShake), With<MainCamera>>,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for (mut transform, mut shake) in query.iter_mut() {
        if shake.strength > 0.0 {
            let offset_x = rng.gen_range(-shake.strength..shake.strength);
            let offset_y = rng.gen_range(-shake.strength..shake.strength);
            transform.translation.x = offset_x;
            transform.translation.y = offset_y;

            shake.strength -= shake.decay * time.delta_seconds() * 60.0;
            if shake.strength < 0.0 {
                shake.strength = 0.0;
                transform.translation = Vec3::ZERO;
            }
        } else {
            transform.translation = Vec3::ZERO;
        }
    }
}

fn handle_input_detected(
    mut commands: Commands,
    mut input_rx: EventReader<InputDetected>,
    asset_server: Res<AssetServer>,
    char_q: Query<(&Character, &Transform)>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    for ev in input_rx.read() {
        for (character, transform) in char_q.iter() {
            if character.actor == ev.actor {
                let text = match ev.dir {
                    GameDirection::Up => "UP",
                    GameDirection::UpRight => "UP+RIGHT",
                    GameDirection::Right => "RIGHT",
                    GameDirection::DownRight => "DOWN+RIGHT",
                    GameDirection::Down => "DOWN",
                    GameDirection::DownLeft => "DOWN+LEFT",
                    GameDirection::Left => "LEFT",
                    GameDirection::UpLeft => "UP+LEFT",
                    GameDirection::UpDown => "UP+DOWN",
                    GameDirection::LeftRight => "LEFT+RIGHT",
                };

                let start_pos = transform.translation + Vec3::new(0.0, 150.0, 10.0);
                let end_pos = start_pos + Vec3::new(0.0, 200.0, 10.0);
                let color = if matches!(character.actor, Actor::Human) { Color::srgb(0.0, 1.0, 1.0) } else { Color::srgb(1.0, 0.65, 0.0) };

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(text, TextStyle {
                            font: font.clone(),
                            font_size: 40.0,
                            color,
                        }),
                        transform: Transform::from_translation(start_pos),
                        ..default()
                    },
                    Lifetime(Timer::from_seconds(0.5, TimerMode::Once)),
                    Animator::new(Tween::new(
                        EaseMethod::EaseFunction(EaseFunction::QuadraticOut),
                        Duration::from_millis(500),
                        TransformPositionLens { start: start_pos, end: end_pos }
                    )),
                ));
            }
        }
    }
}

fn update_background_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ground: ResMut<GroundY>,
    mut q: ParamSet<(
        Query<(&BackgroundSprite, &mut Sprite, &mut Transform)>,
        Query<(&mut Transform, &mut OriginalTransform), With<Grounded>>,
    )>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let width = window.resolution.width();
    for (bg, mut sprite, mut transform) in q.p0().iter_mut() {
        let scale = width / bg.size.x;
        let height = bg.size.y * scale;
        sprite.custom_size = Some(Vec2::new(width, height));
        transform.translation.y = 0.0;
        ground.0 = -height * 0.5 + GROUND_OFFSET_Y;
    }
    for (mut transform, mut original) in q.p1().iter_mut() {
        transform.translation.y = ground.0;
        original.0.y = ground.0;
    }
}

fn handle_debug_input_cue(
    mut commands: Commands,
    mut input_rx: EventReader<DebugInputCue>,
    asset_server: Res<AssetServer>,
    char_q: Query<(&Character, &Transform)>,
    debug_state: Res<DebugState>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    for ev in input_rx.read() {
        for (character, transform) in char_q.iter() {
            if character.actor == ev.actor {
                let start_pos = transform.translation + Vec3::new(0.0, 150.0, 10.0);
                let end_pos = start_pos + Vec3::new(0.0, 200.0, 10.0);
                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(ev.label.clone(), TextStyle {
                            font: font.clone(),
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                        }),
                        transform: Transform::from_translation(start_pos),
                        ..default()
                    },
                    Lifetime(Timer::from_seconds(0.5, TimerMode::Once)),
                    Animator::new(Tween::new(
                        EaseMethod::EaseFunction(EaseFunction::QuadraticOut),
                        Duration::from_millis(500),
                        TransformPositionLens { start: start_pos, end: end_pos }
                    )),
                ));
            }
        }
    }
}

fn despawn_expired(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn controller_name_from_folder(folder_path: &str) -> String {
    Path::new(folder_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("character")
        .to_string()
}

fn controller_path_for_folder(folder_path: &str) -> PathBuf {
    let name = controller_name_from_folder(folder_path);
    let rel_parent = Path::new(folder_path);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(rel_parent)
        .join(format!("{}_controller.json", name))
}

fn load_controller(path: &Path) -> CharacterController {
    if path.exists() {
        match fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()) {
            Some(controller) => controller,
            None => {
                println!("Failed to parse controller at {}, using defaults.", path.display());
                CharacterController::default()
            }
        }
    } else {
        CharacterController::default()
    }
}

fn save_controller(path: &Path, controller: &CharacterController) -> Result<(), String> {
    let json = serde_json::to_string_pretty(controller).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn current_human_index(
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
) -> Option<usize> {
    for (_entity, character, frame_idx, _) in char_q.iter_mut() {
        if matches!(character.actor, Actor::Human) {
            return Some(frame_idx.index);
        }
    }
    None
}

fn discover_frame_paths(folder: &str) -> Vec<String> {
    let rel_parent = Path::new(folder);
    let atlas_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(rel_parent);
    let mut paths: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&atlas_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                let rel = rel_parent.join(file_name);
                paths.push(rel.to_string_lossy().to_string());
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn load_frames_for_folder(folder: &str, asset_server: &AssetServer) -> CharacterFrames {
    let frame_paths = discover_frame_paths(folder);
    let mut name_to_index = HashMap::new();
    let mut names = Vec::with_capacity(frame_paths.len());
    let handles = frame_paths
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            if let Some(name) = Path::new(p).file_name().and_then(|s| s.to_str()) {
                name_to_index.insert(name.to_string(), idx);
                names.push(name.to_string());
            }
            asset_server.load(p.clone())
        })
        .collect::<Vec<_>>();
    CharacterFrames {
        handles,
        name_to_index,
        names,
    }
}

#[derive(Component)]
pub struct FrameIndex {
    pub index: usize,
}

impl CharacterFrames {
    fn count(&self) -> usize { self.handles.len().max(1) }
    #[allow(dead_code)]
    fn primary(&self) -> Option<Handle<Image>> { self.handles.first().cloned() }
    fn get(&self, index: usize) -> Option<Handle<Image>> {
        if self.handles.is_empty() { None } else { Some(self.handles[index % self.handles.len()].clone()) }
    }
    fn index_for_name(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }
    fn sequence_indices(&self, names: &[&str]) -> Option<Vec<usize>> {
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            out.push(self.index_for_name(name)?);
        }
        Some(out)
    }
    pub(crate) fn name_for_index(&self, index: usize) -> Option<&str> {
        self.names.get(index).map(|s| s.as_str())
    }
}

fn apply_frame(frames: &CharacterFrames, frame_idx: &mut FrameIndex, texture: &mut Handle<Image>) {
    if let Some(handle) = frames.get(frame_idx.index) {
        *texture = handle;
    }
}

fn frames_for_actor<'a>(actor: Actor, frames: &'a FrameLibrary) -> &'a CharacterFrames {
    match actor {
        Actor::Human => &frames.human,
        Actor::Ai => &frames.ai,
    }
}

fn ai_idle_index(frames: &FrameLibrary) -> usize {
    frames.ai.index_for_name(AI_IDLE_FRAME).unwrap_or(0)
}

fn ai_attack_sequence(frames: &FrameLibrary) -> Option<Vec<usize>> {
    frames.ai.sequence_indices(&AI_ATTACK_FRAMES)
}

#[derive(Component)]
struct FrameSequence {
    frames: Vec<usize>,
    next_index: usize,
    timer: Timer,
}

#[derive(Component)]
struct RunCycle {
    next_index: usize,
    timer: Timer,
}

fn update_frame_sequences(
    time: Res<Time>,
    debug_state: Res<DebugState>,
    frames: Res<FrameLibrary>,
    mut commands: Commands,
    mut q: Query<(Entity, &Character, &mut FrameSequence, &mut FrameIndex, &mut Handle<Image>)>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    for (entity, character, mut seq, mut frame_idx, mut texture) in q.iter_mut() {
        seq.timer.tick(time.delta());
        if seq.timer.finished() {
            if seq.next_index >= seq.frames.len() {
                commands.entity(entity).remove::<FrameSequence>();
                continue;
            }
            frame_idx.index = seq.frames[seq.next_index];
            apply_frame(frames_for_actor(character.actor, &frames), &mut frame_idx, &mut texture);
            seq.next_index += 1;
        }
    }
}

fn play_frame(
    actor: Actor,
    index: usize,
    frames: &FrameLibrary,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    play_frame_with_duration(actor, index, 0.4, frames, char_q, commands);
}

fn play_frame_with_duration(
    actor: Actor,
    index: usize,
    duration: f32,
    frames: &FrameLibrary,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            let return_index = frame_idx.index;
            frame_idx.index = index;
            apply_frame(frames_for_actor(actor, frames), &mut frame_idx, &mut texture);
            commands.entity(entity).remove::<FrameSequence>();
            commands.entity(entity).remove::<RunCycle>();
            commands.entity(entity).insert(ResetFrame {
                timer: Timer::from_seconds(duration, TimerMode::Once),
                return_index,
            });
        }
    }
}

fn play_frame_with_return_index(
    actor: Actor,
    index: usize,
    duration: f32,
    return_index: usize,
    frames: &FrameLibrary,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            frame_idx.index = index;
            apply_frame(frames_for_actor(actor, frames), &mut frame_idx, &mut texture);
            commands.entity(entity).remove::<FrameSequence>();
            commands.entity(entity).remove::<RunCycle>();
            commands.entity(entity).insert(ResetFrame {
                timer: Timer::from_seconds(duration, TimerMode::Once),
                return_index,
            });
        }
    }
}

fn play_sequence(
    actor: Actor,
    frames_seq: Vec<usize>,
    frames: &FrameLibrary,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    if frames_seq.is_empty() {
        return;
    }
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            let return_index = frame_idx.index;
            let total = SEQUENCE_FRAME_TIME * (frames_seq.len() as f32);
            frame_idx.index = frames_seq[0];
            apply_frame(frames_for_actor(actor, frames), &mut frame_idx, &mut texture);
            commands.entity(entity).insert(FrameSequence {
                frames: frames_seq.clone(),
                next_index: 1,
                timer: Timer::from_seconds(SEQUENCE_FRAME_TIME, TimerMode::Repeating),
            });
            commands.entity(entity).remove::<RunCycle>();
            commands.entity(entity).insert(ResetFrame {
                timer: Timer::from_seconds(total, TimerMode::Once),
                return_index,
            });
        }
    }
}

fn play_sequence_with_return_index(
    actor: Actor,
    frames_seq: Vec<usize>,
    return_index: usize,
    frames: &FrameLibrary,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    if frames_seq.is_empty() {
        return;
    }
    let total = SEQUENCE_FRAME_TIME * (frames_seq.len() as f32);
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            frame_idx.index = frames_seq[0];
            apply_frame(frames_for_actor(actor, frames), &mut frame_idx, &mut texture);
            commands.entity(entity).insert(FrameSequence {
                frames: frames_seq.clone(),
                next_index: 1,
                timer: Timer::from_seconds(SEQUENCE_FRAME_TIME, TimerMode::Repeating),
            });
            commands.entity(entity).remove::<RunCycle>();
            commands.entity(entity).insert(ResetFrame {
                timer: Timer::from_seconds(total, TimerMode::Once),
                return_index,
            });
        }
    }
}

fn play_sequence_no_return(
    actor: Actor,
    frames_seq: Vec<usize>,
    frames: &FrameLibrary,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    if frames_seq.is_empty() {
        return;
    }
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            frame_idx.index = frames_seq[0];
            apply_frame(frames_for_actor(actor, frames), &mut frame_idx, &mut texture);
            commands.entity(entity).insert(FrameSequence {
                frames: frames_seq.clone(),
                next_index: 1,
                timer: Timer::from_seconds(SEQUENCE_FRAME_TIME, TimerMode::Repeating),
            });
        }
    }
}

fn update_block_hold(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    debug_state: Res<DebugState>,
    mut controller_state: ResMut<CharacterControllerState>,
    frames: Res<FrameLibrary>,
    mut char_q: Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    mut commands: Commands,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    if !controller_state.block_hold_active {
        return;
    }
    if !keys.pressed(KeyCode::KeyC) {
        controller_state.block_hold_active = false;
        return;
    }
    controller_state.block_hold_elapsed += time.delta_seconds();
    if controller_state.block_hold_second || controller_state.block_hold_elapsed < BLOCK_HOLD_THRESHOLD {
        return;
    }
    if let Some(idx) = frames.human.index_for_name(BLOCK_HOLD_FRAME) {
        play_frame_with_duration(
            Actor::Human,
            idx,
            0.4,
            &frames,
            &mut char_q,
            &mut commands,
        );
        controller_state.block_hold_second = true;
    } else {
        println!("Missing frame: {}", BLOCK_HOLD_FRAME);
        controller_state.block_hold_second = true;
    }
}

fn update_ai_proximity(
    time: Res<Time>,
    debug_state: Res<DebugState>,
    mut ai_state: ResMut<AiDemoState>,
    mut slash_tx: EventWriter<SlashCue>,
    mut debug_input_tx: EventWriter<DebugInputCue>,
    mut parry_state: ResMut<ParryState>,
    char_q: Query<(
        &Character,
        &Transform,
        Option<&FrameSequence>,
        Option<&ResetFrame>,
        Option<&DeathRespawn>,
        Option<&RespawnFadeIn>,
    )>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    ai_state.cooldown = (ai_state.cooldown - time.delta_seconds()).max(0.0);
    let mut human = None;
    let mut ai = None;
    let mut ai_attacking = false;
    for (character, transform, seq, reset, death, fade) in char_q.iter() {
        if matches!(character.actor, Actor::Human) {
            human = Some(transform.translation);
        } else {
            if death.is_none() && fade.is_none() {
                ai = Some(transform.translation);
                if seq.is_some() || reset.is_some() {
                    ai_attacking = true;
                }
            }
        }
    }
    let (Some(h), Some(a)) = (human, ai) else { return; };
    if ai_attacking { return; }
    if parry_state.ai_ready && (h.x - a.x).abs() <= AI_ATTACK_RANGE {
        slash_tx.send(SlashCue { actor: Actor::Ai });
        ai_state.cooldown = AI_ATTACK_COOLDOWN;
        parry_state.ai_ready = false;
        debug_input_tx.send(DebugInputCue { actor: Actor::Ai, label: "AI PARRY COUNTER".to_string() });
        return;
    }
    if ai_state.cooldown > 0.0 { return; }
    if (h.x - a.x).abs() <= AI_ATTACK_RANGE {
        slash_tx.send(SlashCue { actor: Actor::Ai });
        ai_state.cooldown = AI_ATTACK_COOLDOWN;
        debug_input_tx.send(DebugInputCue { actor: Actor::Ai, label: "AI ATTACK".to_string() });
    }
}

fn update_ai_approach(
    time: Res<Time>,
    debug_state: Res<DebugState>,
    edit_mode: Res<AnimationEditMode>,
    mut char_q: Query<(&Character, &mut Transform, &mut OriginalTransform, Option<&FrameSequence>, Option<&ResetFrame>, Option<&DeathRespawn>, Option<&RespawnFadeIn>)>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    if edit_mode.0 { return; }
    let mut human_x = None;
    let mut ai_x = None;
    for (character, transform, _original, _seq, _reset, death, fade) in char_q.iter() {
        if matches!(character.actor, Actor::Human) {
            human_x = Some(transform.translation.x);
        } else if death.is_none() && fade.is_none() {
            ai_x = Some(transform.translation.x);
        }
    }
    let (Some(hx), Some(ax)) = (human_x, ai_x) else { return; };
    let dist = (hx - ax).abs();
    if dist <= AI_STOP_DISTANCE {
        return;
    }
    let dir = if hx > ax { 1.0 } else { -1.0 };
    let delta = dir * AI_APPROACH_SPEED * time.delta_seconds();
    let desired = ax + delta;
    let clamped = clamp_ai_x(desired, hx);
    for (character, mut transform, mut original, seq, reset, death, fade) in char_q.iter_mut() {
        if !matches!(character.actor, Actor::Ai) { continue; }
        if death.is_some() || fade.is_some() { continue; }
        if seq.is_some() || reset.is_some() { continue; }
        transform.translation.x = clamped;
        original.0.x = clamped;
    }
}

fn update_ai_idle(
    debug_state: Res<DebugState>,
    frames: Res<FrameLibrary>,
    mut char_q: Query<(&Character, &mut FrameIndex, &mut Handle<Image>, Option<&FrameSequence>, Option<&ResetFrame>, Option<&DeathRespawn>, Option<&RespawnFadeIn>)>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    for (character, mut frame_idx, mut texture, seq, reset, death, fade) in char_q.iter_mut() {
        if !matches!(character.actor, Actor::Ai) {
            continue;
        }
        if seq.is_some() || reset.is_some() || death.is_some() || fade.is_some() {
            continue;
        }
        let idle_idx = ai_idle_index(&frames);
        if frame_idx.index != idle_idx {
            frame_idx.index = idle_idx;
            apply_frame(&frames.ai, &mut frame_idx, &mut texture);
        }
    }
}

fn handle_hit_resolution(
    mut slash_rx: EventReader<SlashCue>,
    mut attack_rx: EventReader<AttackCue>,
    time: Res<Time>,
    block_state: Res<BlockState>,
    frames: Res<FrameLibrary>,
    mut frame_q: Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    mut ai_health: ResMut<AiHealth>,
    mut parry_state: ResMut<ParryState>,
    mut q: ParamSet<(
        Query<(&Character, &Transform)>,
        Query<(Entity, &Character, &mut Transform, &mut OriginalTransform, &mut Sprite, Option<&DeathRespawn>)>,
    )>,
    mut commands: Commands,
    mut debug_input_tx: EventWriter<DebugInputCue>,
) {
    let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
    let mut events: Vec<Actor> = Vec::new();
    events.extend(slash_rx.read().map(|ev| ev.actor));
    events.extend(attack_rx.read().map(|ev| ev.actor));

    for actor in events {
        let target = match actor {
            Actor::Human => Actor::Ai,
            Actor::Ai => Actor::Human,
        };
        let mut attacker_x = None;
        let mut target_x = None;
        for (character, transform) in q.p0().iter() {
            if character.actor == actor {
                attacker_x = Some(transform.translation.x);
            } else if character.actor == target {
                target_x = Some(transform.translation.x);
            }
        }
        let (Some(ax), Some(tx)) = (attacker_x, target_x) else { continue; };
        if (ax - tx).abs() > HIT_RANGE {
            continue;
        }
        let last_block = match target {
            Actor::Human => block_state.human_last_ms,
            Actor::Ai => block_state.ai_last_ms,
        };
        if now_ms.saturating_sub(last_block) <= BLOCK_WINDOW_MS {
            if matches!(target, Actor::Human) && matches!(actor, Actor::Ai) {
                parry_state.ready = true;
                parry_state.timer.reset();
                debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "PARRY READY".to_string() });
                let parry_frame = if parry_state.human_parry_alt { PARRY_FRAME_ALT } else { PARRY_FRAME };
                if let Some(idx) = frames.human.index_for_name(parry_frame) {
                    play_frame(Actor::Human, idx, &frames, &mut frame_q, &mut commands);
                } else {
                    println!("Missing frame: {}", parry_frame);
                }
                parry_state.human_parry_alt = !parry_state.human_parry_alt;
                if let Some(idx) = frames.human.index_for_name(BLOCK_HIT_FRAME) {
                    play_frame(Actor::Human, idx, &frames, &mut frame_q, &mut commands);
                } else {
                    println!("Missing frame: {}", BLOCK_HIT_FRAME);
                }
            }
            if matches!(target, Actor::Ai) && matches!(actor, Actor::Human) {
                parry_state.ai_ready = true;
                parry_state.ai_timer.reset();
                debug_input_tx.send(DebugInputCue { actor: Actor::Ai, label: "AI PARRY READY".to_string() });
                if let Some(idx) = frames.ai.index_for_name(AI_PARRY_FRAME) {
                    play_frame(Actor::Ai, idx, &frames, &mut frame_q, &mut commands);
                } else {
                    println!("Missing frame: {}", AI_PARRY_FRAME);
                }
            }
            continue;
        }
        for (entity, character, mut transform, mut original, mut sprite, death) in q.p1().iter_mut() {
            if character.actor == target {
                if death.is_some() {
                    continue;
                }
                if matches!(target, Actor::Ai) {
                    if ai_health.hits_remaining > 0 {
                        ai_health.hits_remaining = ai_health.hits_remaining.saturating_sub(1);
                    }
                    if ai_health.hits_remaining == 0 {
                        sprite.color = Color::srgb(1.0, 0.2, 0.2);
                        if let Some(seq) = frames.ai.sequence_indices(&AI_DEATH_FRAMES) {
                            let total = SEQUENCE_FRAME_TIME * (seq.len() as f32);
                            play_sequence_no_return(
                                Actor::Ai,
                                seq,
                                &frames,
                                &mut frame_q,
                                &mut commands,
                            );
                            commands.entity(entity).insert(DeathRespawn {
                                stage: DeathStage::Sequence,
                                timer: Timer::from_seconds(total, TimerMode::Once),
                                respawn_x: original.0.x,
                            });
                        }
                        ai_health.hits_remaining = AI_HITS_TO_DEATH;
                    } else {
                        sprite.color = Color::srgb(1.0, 0.2, 0.2);
                        commands.entity(entity).insert(HitFlash { timer: Timer::from_seconds(0.12, TimerMode::Once) });
                    }
                } else {
                    sprite.color = Color::srgb(1.0, 0.2, 0.2);
                    commands.entity(entity).insert(HitFlash { timer: Timer::from_seconds(0.12, TimerMode::Once) });
                }
                stagger_target(character.actor, ax, &mut transform, &mut original);
            }
        }
    }
}

fn update_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Sprite, &mut HitFlash)>,
) {
    for (entity, mut sprite, mut flash) in q.iter_mut() {
        flash.timer.tick(time.delta());
        if flash.timer.finished() {
            sprite.color = Color::srgb(1.0, 1.0, 1.0);
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

fn update_death_respawn(
    time: Res<Time>,
    frames: Res<FrameLibrary>,
    ground: Res<GroundY>,
    mut ai_state: ResMut<AiDemoState>,
    mut ai_health: ResMut<AiHealth>,
    mut block_state: ResMut<BlockState>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Sprite, &mut DeathRespawn)>,
) {
    for (entity, mut sprite, mut death) in q.iter_mut() {
        death.timer.tick(time.delta());
        match death.stage {
            DeathStage::Sequence => {
                if death.timer.finished() {
                    death.stage = DeathStage::FadeOut;
                    death.timer = Timer::from_seconds(DEATH_FADE_SECONDS, TimerMode::Once);
                }
            }
            DeathStage::FadeOut => {
                let alpha = 1.0 - (death.timer.elapsed_secs() / DEATH_FADE_SECONDS).min(1.0);
                sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
                if death.timer.finished() {
                    let respawn_pos = Vec2::new(death.respawn_x, ground.0);
                    commands.entity(entity).despawn();
                    let new_entity = spawn_character(
                        &mut commands,
                        Actor::Ai,
                        respawn_pos,
                        &frames.ai,
                        AI_IDLE_FRAME,
                        0.0,
                    );
                    ai_state.cooldown = 0.0;
                    ai_health.hits_remaining = AI_HITS_TO_DEATH;
                    block_state.ai_last_ms = 0;
                    commands.entity(new_entity).insert(RespawnFadeIn {
                        timer: Timer::from_seconds(RESPAWN_FADE_SECONDS, TimerMode::Once),
                    });
                }
            }
        }
    }
}

fn update_respawn_fade_in(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Sprite, &mut RespawnFadeIn)>,
) {
    for (entity, mut sprite, mut fade) in q.iter_mut() {
        fade.timer.tick(time.delta());
        let alpha = (fade.timer.elapsed_secs() / RESPAWN_FADE_SECONDS).min(1.0);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
        if fade.timer.finished() {
            commands.entity(entity).remove::<RespawnFadeIn>();
        }
    }
}

fn update_parry_state(
    time: Res<Time>,
    mut parry_state: ResMut<ParryState>,
) {
    if parry_state.ready {
        parry_state.timer.tick(time.delta());
        if parry_state.timer.finished() {
            parry_state.ready = false;
        }
    }
    if parry_state.ai_ready {
        parry_state.ai_timer.tick(time.delta());
        if parry_state.ai_timer.finished() {
            parry_state.ai_ready = false;
        }
    }
}

fn maybe_ai_block(
    frames: &FrameLibrary,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
    block_state: &mut BlockState,
    time: &Time,
    debug_input_tx: &mut EventWriter<DebugInputCue>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    if rng.gen::<f32>() > AI_BLOCK_CHANCE {
        return;
    }
    block_state.ai_last_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
    debug_input_tx.send(DebugInputCue { actor: Actor::Ai, label: "AI BLOCK".to_string() });
    if let Some(seq) = ai_attack_sequence(frames) {
        play_sequence_with_return_index(
            Actor::Ai,
            seq,
            ai_idle_index(frames),
            frames,
            char_q,
            commands,
        );
    }
}

#[allow(dead_code)]
fn stagger_actor(
    actor: Actor,
    other_x: f32,
    char_q: &mut Query<(Entity, &Character, &mut Transform, &mut OriginalTransform, &mut Sprite)>,
) {
    for (_entity, character, mut transform, mut original, _sprite) in char_q.iter_mut() {
        if character.actor == actor {
            let dir = if transform.translation.x < other_x { -1.0 } else { 1.0 };
            let desired = transform.translation.x + (dir * STAGGER_DISTANCE);
            let clamped = if matches!(actor, Actor::Human) {
                clamp_human_x(desired, other_x)
            } else {
                clamp_ai_x(desired, other_x)
            };
            transform.translation.x = clamped;
            original.0.x = clamped;
        }
    }
}

fn stagger_target(
    actor: Actor,
    other_x: f32,
    transform: &mut Transform,
    original: &mut OriginalTransform,
) {
    let dir = if transform.translation.x < other_x { -1.0 } else { 1.0 };
    let desired = transform.translation.x + (dir * STAGGER_DISTANCE);
    let clamped = if matches!(actor, Actor::Human) {
        clamp_human_x(desired, other_x)
    } else {
        clamp_ai_x(desired, other_x)
    };
    transform.translation.x = clamped;
    original.0.x = clamped;
}

fn update_walk_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    vkeys: Res<ButtonInput<VirtualKey>>,
    debug_state: Res<DebugState>,
    edit_mode: Res<AnimationEditMode>,
    mut move_intent: ResMut<MoveIntent>,
    mut char_q: Query<(&Character, &mut Transform, &mut OriginalTransform)>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    if edit_mode.0 {
        move_intent.dir = 0.0;
        return;
    }
    let mut dir: f32 = 0.0;
    if keys.pressed(KeyCode::ArrowRight) || vkeys.pressed(VirtualKey::Right) { dir += 1.0; }
    if keys.pressed(KeyCode::ArrowLeft) || vkeys.pressed(VirtualKey::Left) { dir -= 1.0; }
    move_intent.dir = dir;
    if dir.abs() < f32::EPSILON { return; }
    let speed = 240.0;
    let delta = dir * speed * time.delta_seconds();
    let mut human_x = None;
    let mut ai_x = None;
    for (character, transform, _original) in char_q.iter() {
        if matches!(character.actor, Actor::Human) {
            human_x = Some(transform.translation.x);
        } else {
            ai_x = Some(transform.translation.x);
        }
    }
    let Some(hx) = human_x else { return; };
    let Some(ax) = ai_x else { return; };
    let clamped = clamp_human_x(hx + delta, ax);
    for (character, mut transform, mut original) in char_q.iter_mut() {
        if matches!(character.actor, Actor::Human) {
            transform.translation.x = clamped;
            original.0.x = transform.translation.x;
        }
    }
}

fn update_run_animation(
    time: Res<Time>,
    debug_state: Res<DebugState>,
    edit_mode: Res<AnimationEditMode>,
    move_intent: Res<MoveIntent>,
    frames: Res<FrameLibrary>,
    run_frames: Res<RunAnimationFrames>,
    stance_lock: Res<StanceLock>,
    mut commands: Commands,
    mut char_q: Query<
        (
            Entity,
            &Character,
            &mut FrameIndex,
            &mut Handle<Image>,
            &mut Sprite,
            Option<&mut RunCycle>,
        ),
        (Without<FrameSequence>, Without<ResetFrame>),
    >,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    if edit_mode.0 { return; }
    let dir = move_intent.dir;
    for (entity, character, mut frame_idx, mut texture, mut sprite, run_cycle) in char_q.iter_mut() {
        if !matches!(character.actor, Actor::Human) {
            continue;
        }
        if dir.abs() < f32::EPSILON {
            sprite.flip_x = false;
            if run_cycle.is_some() {
                commands.entity(entity).remove::<RunCycle>();
            }
            if let Some(locked_idx) = stance_lock.index {
                frame_idx.index = locked_idx;
                apply_frame(&frames.human, &mut frame_idx, &mut texture);
            } else if let Some(idx) = frames.human.index_for_name(IDLE_FRAME) {
                frame_idx.index = idx;
                apply_frame(&frames.human, &mut frame_idx, &mut texture);
            }
            continue;
        }
        if run_frames.human.is_empty() {
            continue;
        }
        sprite.flip_x = dir < 0.0;
        let mut cycle = match run_cycle {
            Some(cycle) => cycle,
            None => {
                frame_idx.index = run_frames.human[0];
                apply_frame(&frames.human, &mut frame_idx, &mut texture);
                commands.entity(entity).insert(RunCycle {
                    next_index: 1,
                    timer: Timer::from_seconds(RUN_FRAME_TIME, TimerMode::Repeating),
                });
                continue;
            }
        };
        cycle.timer.tick(time.delta());
        if cycle.timer.finished() {
            let idx = cycle.next_index % run_frames.human.len();
            frame_idx.index = run_frames.human[idx];
            apply_frame(&frames.human, &mut frame_idx, &mut texture);
            cycle.next_index = (cycle.next_index + 1) % run_frames.human.len();
        }
    }
}

fn dash_forward(
    move_q: &mut Query<(&Character, &mut Transform, &mut OriginalTransform)>,
) {
    let mut human_x = None;
    let mut ai_x = None;
    for (character, transform, _original) in move_q.iter() {
        if matches!(character.actor, Actor::Human) {
            human_x = Some(transform.translation.x);
        } else {
            ai_x = Some(transform.translation.x);
        }
    }
    let (Some(hx), Some(ax)) = (human_x, ai_x) else { return; };
    let clamped = clamp_human_x(hx + DASH_DISTANCE, ax);
    for (character, mut transform, mut original) in move_q.iter_mut() {
        if matches!(character.actor, Actor::Human) {
            transform.translation.x = clamped;
            original.0.x = transform.translation.x;
        }
    }
}

fn clamp_human_x(desired: f32, ai_x: f32) -> f32 {
    let max_x = ai_x - MIN_SEPARATION;
    let clamped = if desired > max_x { max_x } else { desired };
    if clamped < STAGE_LEFT_X { STAGE_LEFT_X } else { clamped }
}

fn clamp_ai_x(desired: f32, human_x: f32) -> f32 {
    let min_x = human_x + MIN_SEPARATION;
    let clamped = if desired < min_x { min_x } else { desired };
    if clamped > STAGE_RIGHT_X { STAGE_RIGHT_X } else { clamped }
}
