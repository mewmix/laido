use bevy::prelude::*;
use bevy_tweening::*;
use bevy_tweening::lens::*;
use std::time::Duration;

use crate::{Actor, ClashCue, GoCue, SlashCue, InputDetected};
use crate::types::Direction as GameDirection;

pub struct VisualsPlugin;

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
                despawn_expired,
                reset_character_frames,
            ));
    }
}

fn setup_characters(mut commands: Commands, assets: Res<CharacterAssets>) {
    spawn_character(&mut commands, Actor::Human, Vec2::new(-300.0, -100.0), &assets);
    spawn_character(&mut commands, Actor::Ai, Vec2::new(300.0, -100.0), &assets);
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct Lifetime(Timer);

#[derive(Component)]
struct ResetFrame(Timer);

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
struct CharacterAssets {
    layout: Handle<TextureAtlasLayout>,
    texture: Handle<Image>,
}

#[derive(Component)]
pub struct OriginalTransform(pub Vec3);

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Assets
    let texture = asset_server.load("atlas/swordsman_laido_atlas.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(501, 501), 4, 4, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);
    commands.insert_resource(CharacterAssets {
        layout: layout_handle,
        texture,
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
    assets: &CharacterAssets,
) {
    let color = match actor {
        Actor::Human => Color::srgb(0.2, 0.6, 1.0), // Blueish tint
        Actor::Ai => Color::srgb(1.0, 0.3, 0.3),    // Reddish tint
    };

    let base_scale = Vec3::splat(0.4); // Scale down 501x501

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

    commands.spawn((
        SpriteBundle {
            texture: assets.texture.clone(),
            sprite: Sprite {
                color,
                flip_x,
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0).with_scale(base_scale),
            ..default()
        },
        TextureAtlas {
            layout: assets.layout.clone(),
            index: 0,
        },
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
    mut char_q: Query<(Entity, &Character, &OriginalTransform, &mut Animator<Transform>, &mut TextureAtlas)>,
) {
    for ev in slash_rx.read() {
        for (entity, character, original, mut animator, mut atlas) in char_q.iter_mut() {
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
                atlas.index = 1;
                commands.entity(entity).insert(ResetFrame(Timer::from_seconds(0.5, TimerMode::Once)));
            }
        }
    }
}

fn reset_character_frames(
    mut commands: Commands,
    time: Res<Time>,
    mut char_q: Query<(Entity, &mut TextureAtlas, &mut ResetFrame), With<Character>>,
) {
    for (entity, mut atlas, mut reset) in char_q.iter_mut() {
        reset.0.tick(time.delta());
        if reset.0.finished() {
            atlas.index = 0;
            commands.entity(entity).remove::<ResetFrame>();
        }
    }
}

fn handle_clash_cue(
    mut clash_rx: EventReader<ClashCue>,
    mut camera_q: Query<&mut CameraShake, With<MainCamera>>,
    mut commands: Commands,
) {
    for _ in clash_rx.read() {
        if let Ok(mut shake) = camera_q.get_single_mut() {
            shake.strength = 4.0; // Violent shake
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