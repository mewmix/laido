use bevy::prelude::*;
use bevy_tweening::*;
use bevy_tweening::lens::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{Actor, ClashCue, GoCue, SlashCue, InputDetected, DebugInputCue};
use crate::types::Direction as GameDirection;
use crate::plugin::{DuelRuntime, DebugState, AnimationEditMode};
use crate::combat::correct_direction_for;

pub struct VisualsPlugin;

const CHARACTER_FRAMES_DIR: &str = "atlas/white_samurai";
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
const BACK_HEAVY_FRAME: &str = "back_heavy_stance.png";
const S_PRESS_FRAME: &str = "duel.png";
const S_RELEASE_FRAME: &str = "fast-attack-forward.png";
const S_DOUBLE_FRAMES: [&str; 2] = ["heavy_spin.png", "heavy_spin_2.png"];
const S_DOUBLE_RETURN: &str = "back_fast_stance.png";
const S_DOUBLE_WINDOW_MS: u64 = 250;

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
                update_block_hold,
                update_walk_input,
                animation_tester,
            ));
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
    frames: Res<CharacterFrames>,
) {
    if matches!(*debug_state, DebugState::Animation) { return; }

    for (character, mut frame_idx, mut texture) in char_q.iter_mut() {
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

        // Handle Pulsing for Combo Stances
        if dir == GameDirection::UpDown {
            // Pulse between Up (0) and Down (1)
            frame_idx.index = if (time.elapsed_seconds() * 5.0).sin() > 0.0 { 0 } else { 1 };
        } else if dir == GameDirection::LeftRight {
            // Pulse between Left (2) and Right (3)
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

fn animation_tester(
    keys: Res<ButtonInput<KeyCode>>,
    mut char_q: Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    debug_state: Res<DebugState>,
    mut slash_tx: EventWriter<SlashCue>,
    mut clash_tx: EventWriter<ClashCue>,
    mut debug_input_tx: EventWriter<DebugInputCue>,
    mut controller_state: ResMut<CharacterControllerState>,
    frames: Res<CharacterFrames>,
    mut commands: Commands,
    time: Res<Time>,
    edit_mode: Res<AnimationEditMode>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    
    if edit_mode.0 {
        let mut delta = 0;
        if keys.just_pressed(KeyCode::ArrowLeft) { delta = -1; }
        if keys.just_pressed(KeyCode::ArrowRight) { delta = 1; }
        
        if delta != 0 {
            for (_entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
                if matches!(character.actor, Actor::Human) {
                    let new_idx = (frame_idx.index as i32 + delta)
                        .rem_euclid(frames.count() as i32) as usize;
                    frame_idx.index = new_idx;
                    apply_frame(&frames, &mut frame_idx, &mut texture);
                    println!("Human Atlas Index: {} - {}", new_idx, get_pose_name(new_idx));
                }
            }
        }
    }

    if keys.just_pressed(KeyCode::Space) {
        if let Some(idx) = current_human_index(&mut char_q) {
            controller_state.controller.slash_index = idx;
        }
        slash_tx.send(SlashCue { actor: Actor::Human });
    }
    if keys.just_pressed(KeyCode::Enter) {
        if let Some(idx) = current_human_index(&mut char_q) {
            controller_state.controller.clash_index = idx;
        }
        clash_tx.send(ClashCue);
    }
    if keys.just_pressed(KeyCode::KeyX) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "X DOWN".to_string() });
        if let Some(idx) = frames.index_for_name(X_PRESS_FRAME) {
            play_frame(Actor::Human, idx, &frames, &mut char_q, &mut commands);
            controller_state.x_armed = true;
        } else {
            println!("Missing frame: {}", X_PRESS_FRAME);
        }
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "Z DOWN".to_string() });
        if let Some(idx) = frames.index_for_name(Z_PRESS_FRAME) {
            play_frame(Actor::Human, idx, &frames, &mut char_q, &mut commands);
            controller_state.z_up_armed = true;
        } else {
            println!("Missing frame: {}", Z_PRESS_FRAME);
        }
    }
    if keys.just_released(KeyCode::KeyZ) && controller_state.z_up_armed {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "Z UP".to_string() });
        if let Some(idx) = frames.index_for_name(Z_RELEASE_FRAME) {
            play_frame(Actor::Human, idx, &frames, &mut char_q, &mut commands);
        } else {
            println!("Missing frame: {}", Z_RELEASE_FRAME);
        }
        controller_state.z_up_armed = false;
    }
    if keys.just_released(KeyCode::KeyX) && controller_state.x_armed {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "X UP".to_string() });
        if let Some(seq) = frames.sequence_indices(&[X_RELEASE_FRAME, X_FOLLOW_FRAME]) {
            play_sequence(Actor::Human, seq, &frames, &mut char_q, &mut commands);
        } else {
            println!("Missing one or more extended release frames.");
        }
        controller_state.x_armed = false;
    }
    if keys.just_pressed(KeyCode::KeyS) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "S DOWN".to_string() });
        let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
        let is_double = now_ms.saturating_sub(controller_state.s_last_press_ms) <= S_DOUBLE_WINDOW_MS;
        controller_state.s_last_press_ms = now_ms;
        controller_state.s_waiting_release = !is_double;
        controller_state.s_double_active = is_double;
        if is_double {
            debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "S DOUBLE".to_string() });
            if let Some(seq) = frames.sequence_indices(&S_DOUBLE_FRAMES) {
                if let Some(return_idx) = frames.index_for_name(S_DOUBLE_RETURN) {
                    play_sequence_with_return_index(
                        Actor::Human,
                        seq,
                        return_idx,
                        &frames,
                        &mut char_q,
                        &mut commands,
                    );
                } else {
                    println!("Missing frame: {}", S_DOUBLE_RETURN);
                }
            } else {
                println!("Missing one or more heavy spin frames.");
            }
        } else if let Some(idx) = frames.index_for_name(S_PRESS_FRAME) {
            play_frame_with_return_index(
                Actor::Human,
                idx,
                0.6,
                idx,
                &frames,
                &mut char_q,
                &mut commands,
            );
        } else {
            println!("Missing frame: {}", S_PRESS_FRAME);
        }
    }
    if keys.just_released(KeyCode::KeyS) && controller_state.s_waiting_release && !controller_state.s_double_active {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "S UP".to_string() });
        if let (Some(idx), Some(return_idx)) = (
            frames.index_for_name(S_RELEASE_FRAME),
            frames.index_for_name(S_PRESS_FRAME),
        ) {
            play_frame_with_return_index(
                Actor::Human,
                idx,
                0.4,
                return_idx,
                &frames,
                &mut char_q,
                &mut commands,
            );
        } else {
            println!("Missing frame: {}", S_RELEASE_FRAME);
        }
        controller_state.s_waiting_release = false;
    }
    if keys.just_pressed(KeyCode::KeyC) {
        if keys.pressed(KeyCode::ArrowLeft) {
            debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "C+LEFT".to_string() });
            if let Some(idx) = frames.index_for_name(BACK_HEAVY_FRAME) {
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
        } else if keys.pressed(KeyCode::ArrowDown) {
            debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "C+DOWN".to_string() });
            if let Some(idx) = frames.index_for_name(BLOCK_DOWN_FRAME) {
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
            if let Some(idx) = frames.index_for_name(BLOCK_PRESS_FRAME) {
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
    if keys.just_released(KeyCode::KeyC) {
        debug_input_tx.send(DebugInputCue { actor: Actor::Human, label: "C UP".to_string() });
        controller_state.block_hold_active = false;
    }
    if keys.just_pressed(KeyCode::KeyS) {
        if let Err(err) = save_controller(&controller_state.controller_path, &controller_state.controller) {
            println!("Failed to save controller: {}", err);
        } else {
            println!("Saved controller: {}", controller_state.controller_path.display());
        }
    }
}

fn setup_characters(mut commands: Commands, frames: Res<CharacterFrames>) {
    spawn_character(&mut commands, Actor::Human, Vec2::new(-300.0, -100.0), &frames);
    spawn_character(&mut commands, Actor::Ai, Vec2::new(300.0, -100.0), &frames);
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
pub struct CameraShake {
    pub strength: f32,
    pub decay: f32,
}

#[derive(Component)]
pub struct Character {
    pub actor: Actor,
}

#[derive(Resource)]
pub(crate) struct CharacterFrames {
    handles: Vec<Handle<Image>>,
    name_to_index: HashMap<String, usize>,
    names: Vec<String>,
}

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
    let frame_paths = discover_frame_paths();
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
    commands.insert_resource(CharacterFrames {
        handles,
        name_to_index,
        names,
    });

    let controller_path = controller_path_for_folder(CHARACTER_FRAMES_DIR);
    let controller = load_controller(&controller_path);
    let controller_name = controller_name_from_folder(CHARACTER_FRAMES_DIR);
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

    // Camera
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
        CameraShake { strength: 0.0, decay: 3.0 },
    ));

    // Background - Dark Atmosphere
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.05, 0.05, 0.1),
            custom_size: Some(Vec2::new(2000.0, 2000.0)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -10.0),
        ..default()
    });

    // Floor / Ground Line
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.1, 0.1, 0.15),
            custom_size: Some(Vec2::new(2000.0, 400.0)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, -300.0, -5.0),
        ..default()
    });
}

// Separate system to spawn once assets are ready or just use the resource in Update
// Actually I can spawn them in a system that runs after Startup or just wait for them.
// Let's create a system `spawn_players` that runs once resource is available.

fn spawn_character(
    commands: &mut Commands,
    actor: Actor,
    pos: Vec2,
    frames: &CharacterFrames,
) {
    let base_scale = Vec3::splat(0.4); // Scale down 512x512

    // Idle Animation: Breathing (Scale Y)
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

    let idle_idx = frames.index_for_name(IDLE_FRAME).unwrap_or(0);
    let idle_texture = frames.get(idle_idx).unwrap_or_default();
    commands.spawn((
        SpriteBundle {
            texture: idle_texture,
            sprite: Sprite {
                color: Color::srgb(1.0, 1.0, 1.0),
                flip_x,
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0).with_scale(base_scale),
            ..default()
        },
        FrameIndex { index: idle_idx },
        Character { actor },
        OriginalTransform(Vec3::new(pos.x, pos.y, 0.0)),
        Animator::new(idle_tween),
    ));
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
    mut char_q: Query<(Entity, &Character, &OriginalTransform, &mut Animator<Transform>, &mut FrameIndex, &mut Handle<Image>)>,
    controller_state: Res<CharacterControllerState>,
    frames: Res<CharacterFrames>,
) {
    for ev in slash_rx.read() {
        for (entity, character, original, mut animator, mut frame_idx, mut texture) in char_q.iter_mut() {
            if character.actor == ev.actor {
                // Attacker lunges
                let start_pos = original.0;
                let lunge_dist = if matches!(character.actor, Actor::Human) { 250.0 } else { -250.0 };
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

                // Change to attack frame
                frame_idx.index = controller_state.controller.slash_index;
                apply_frame(&frames, &mut frame_idx, &mut texture);
                commands.entity(entity).insert(ResetFrame {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                    return_index: 0,
                });
            }
        }
    }
}

fn reset_character_frames(
    mut commands: Commands,
    time: Res<Time>,
    mut char_q: Query<(Entity, &mut FrameIndex, &mut Handle<Image>, &mut ResetFrame), With<Character>>,
    frames: Res<CharacterFrames>,
) {
    for (entity, mut frame_idx, mut texture, mut reset) in char_q.iter_mut() {
        reset.timer.tick(time.delta());
        if reset.timer.finished() {
            frame_idx.index = reset.return_index;
            apply_frame(&frames, &mut frame_idx, &mut texture);
            commands.entity(entity).remove::<ResetFrame>();
        }
    }
}

fn handle_clash_cue(
    mut clash_rx: EventReader<ClashCue>,
    mut camera_q: Query<&mut CameraShake, With<MainCamera>>,
    mut commands: Commands,
    mut char_q: Query<(Entity, &mut FrameIndex, &mut Handle<Image>), With<Character>>,
    controller_state: Res<CharacterControllerState>,
    frames: Res<CharacterFrames>,
) {
    for _ in clash_rx.read() {
        if let Ok(mut shake) = camera_q.get_single_mut() {
            shake.strength = 4.0; // Violent shake
        }
        for (entity, mut frame_idx, mut texture) in char_q.iter_mut() {
            frame_idx.index = controller_state.controller.clash_index;
            apply_frame(&frames, &mut frame_idx, &mut texture);
            commands.entity(entity).insert(ResetFrame {
                timer: Timer::from_seconds(0.2, TimerMode::Once),
                return_index: 0,
            });
        }
        // Spawn Spark
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 1.0, 0.5),
                    custom_size: Some(Vec2::new(150.0, 150.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 50.0, 10.0)
                    .with_rotation(Quat::from_rotation_z(0.78)), // 45 deg
                ..default()
            },
            // Fade out tween
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
                transform.translation = Vec3::ZERO; // Reset
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

fn discover_frame_paths() -> Vec<String> {
    let rel_parent = Path::new(CHARACTER_FRAMES_DIR);
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

#[derive(Component)]
pub struct FrameIndex {
    pub index: usize,
}

impl CharacterFrames {
    fn count(&self) -> usize { self.handles.len().max(1) }
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

#[derive(Component)]
struct FrameSequence {
    frames: Vec<usize>,
    next_index: usize,
    timer: Timer,
}

fn update_frame_sequences(
    time: Res<Time>,
    debug_state: Res<DebugState>,
    frames: Res<CharacterFrames>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut FrameSequence, &mut FrameIndex, &mut Handle<Image>)>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    for (entity, mut seq, mut frame_idx, mut texture) in q.iter_mut() {
        seq.timer.tick(time.delta());
        if seq.timer.finished() {
            if seq.next_index >= seq.frames.len() {
                commands.entity(entity).remove::<FrameSequence>();
                continue;
            }
            frame_idx.index = seq.frames[seq.next_index];
            apply_frame(&frames, &mut frame_idx, &mut texture);
            seq.next_index += 1;
        }
    }
}

fn play_frame(
    actor: Actor,
    index: usize,
    frames: &CharacterFrames,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    play_frame_with_duration(actor, index, 0.4, frames, char_q, commands);
}

fn play_frame_with_duration(
    actor: Actor,
    index: usize,
    duration: f32,
    frames: &CharacterFrames,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            let return_index = frame_idx.index;
            frame_idx.index = index;
            apply_frame(frames, &mut frame_idx, &mut texture);
            commands.entity(entity).remove::<FrameSequence>();
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
    frames: &CharacterFrames,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            frame_idx.index = index;
            apply_frame(frames, &mut frame_idx, &mut texture);
            commands.entity(entity).remove::<FrameSequence>();
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
    frames: &CharacterFrames,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    if frames_seq.is_empty() {
        return;
    }
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            let return_index = frame_idx.index;
            let total = 0.12 * (frames_seq.len() as f32);
            frame_idx.index = frames_seq[0];
            apply_frame(frames, &mut frame_idx, &mut texture);
            commands.entity(entity).insert(FrameSequence {
                frames: frames_seq.clone(),
                next_index: 1,
                timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            });
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
    frames: &CharacterFrames,
    char_q: &mut Query<(Entity, &Character, &mut FrameIndex, &mut Handle<Image>)>,
    commands: &mut Commands,
) {
    if frames_seq.is_empty() {
        return;
    }
    let total = 0.12 * (frames_seq.len() as f32);
    for (entity, character, mut frame_idx, mut texture) in char_q.iter_mut() {
        if character.actor == actor {
            frame_idx.index = frames_seq[0];
            apply_frame(frames, &mut frame_idx, &mut texture);
            commands.entity(entity).insert(FrameSequence {
                frames: frames_seq.clone(),
                next_index: 1,
                timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            });
            commands.entity(entity).insert(ResetFrame {
                timer: Timer::from_seconds(total, TimerMode::Once),
                return_index,
            });
        }
    }
}

fn update_block_hold(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    debug_state: Res<DebugState>,
    mut controller_state: ResMut<CharacterControllerState>,
    frames: Res<CharacterFrames>,
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
    if let Some(idx) = frames.index_for_name(BLOCK_HOLD_FRAME) {
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

fn update_walk_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    debug_state: Res<DebugState>,
    edit_mode: Res<AnimationEditMode>,
    mut char_q: Query<(&Character, &mut Transform, &mut OriginalTransform)>,
) {
    if !matches!(*debug_state, DebugState::Animation) { return; }
    if edit_mode.0 { return; }
    let mut dir = 0.0;
    if keys.pressed(KeyCode::ArrowRight) { dir += 1.0; }
    if keys.pressed(KeyCode::ArrowLeft) { dir -= 1.0; }
    if dir.abs() < f32::EPSILON { return; }
    let speed = 240.0;
    let delta = dir * speed * time.delta_seconds();
    for (character, mut transform, mut original) in char_q.iter_mut() {
        if matches!(character.actor, Actor::Human) {
            transform.translation.x += delta;
            original.0.x = transform.translation.x;
        }
    }
}
