#![cfg(feature = "bevy")]
use bevy::prelude::*;
use bevy_tweening::*;
use bevy_tweening::lens::*;
use std::time::Duration;

use crate::combat::correct_direction_for;
use crate::plugin::{DuelRuntime, GoCue, DebugState, AnimationEditMode};
use crate::types::{DuelPhase, MatchState, Outcome, Actor};
use crate::visuals::{AI_ATTACK_RANGE, AI_DODGE_DISTANCE, AI_STOP_DISTANCE, HIT_RANGE, MIN_SEPARATION, AiHealth, Character, CharacterControllerState, DeathRespawn, FrameIndex, FrameLibrary, ParryState, RespawnFadeIn};

pub fn systems() -> impl Plugin {
    HudPlugin
}

struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, (
                update_onboarding,
                update_swipe_cues,
                handle_go_event,
                update_round_indicators,
                handle_restart_input,
                update_debug_text,
            ));
    }
}

#[derive(Component)]
struct OnboardingText;

#[derive(Component)]
struct DebugText;

#[derive(Component)]
struct RoundIndicator {
    index: usize,
}

#[derive(Component)]
struct SwipeCueText;

#[derive(Component)]
struct GoText;

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = {
        let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/fonts/FiraSans-Bold.ttf");
        if font_path.exists() {
            asset_server.load("fonts/FiraSans-Bold.ttf")
        } else {
            Handle::default()
        }
    };

    // Debug Stats Text (Top Left)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Debug Stats",
                TextStyle {
                    font: font_handle.clone(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                ..default()
            },
            z_index: ZIndex::Global(20),
            ..default()
        },
        DebugText,
    ));

    // Onboarding Text
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Swipe when you see GO.",
                TextStyle {
                    font: font_handle.clone(),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            )
            .with_justify(JustifyText::Center),
            style: Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Percent(20.0),
                ..default()
            },
            z_index: ZIndex::Global(5),
            ..default()
        },
        OnboardingText,
    ));

    // GO Text (Hidden by default or Scale 0)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "GO!",
                TextStyle {
                    font: font_handle.clone(),
                    font_size: 150.0,
                    color: Color::srgb(0.2, 1.0, 0.4),
                },
            )
            .with_justify(JustifyText::Center),
            style: Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                ..default()
            },
            z_index: ZIndex::Global(10),
            visibility: Visibility::Hidden,
            ..default()
        },
        GoText,
    ));

    // Swipe Cue Text (Center Top)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "READY",
                TextStyle {
                    font: font_handle.clone(),
                    font_size: 60.0,
                    color: Color::srgb(1.0, 1.0, 0.0),
                },
            )
            .with_justify(JustifyText::Center),
            style: Style {
                position_type: PositionType::Absolute,
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                top: Val::Px(150.0),
                ..default()
            },
            z_index: ZIndex::Global(5),
            visibility: Visibility::Hidden,
            ..default()
        },
        SwipeCueText,
    ));

    // Between-round overlay
    let bar_width = 80.0;
    let bar_height = 20.0;
    let gap = 10.0;
    let start_x = -(bar_width + gap);

    for i in 0..3 {
        let x = start_x + (i as f32 * (bar_width + gap));
        commands.spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5), 
                custom_size: Some(Vec2::new(bar_width, bar_height)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, 5.0),
            RoundIndicator { index: i },
            Visibility::Hidden,
        ));
    }
}

fn update_onboarding(
    mut query: Query<&mut Visibility, With<OnboardingText>>,
    rt: Res<DuelRuntime>,
    debug_state: Res<DebugState>,
) {
    if matches!(*debug_state, DebugState::Animation) {
        *query.single_mut() = Visibility::Hidden;
        return;
    }
    let mut vis = query.single_mut();
    if rt.machine.round_results.is_empty() {
        match rt.machine.phase {
            DuelPhase::Standoff | DuelPhase::RandomDelay | DuelPhase::GoSignal | DuelPhase::InputWindow => {
                *vis = Visibility::Visible;
            }
            _ => {
                *vis = Visibility::Hidden;
            }
        }
    } else {
        *vis = Visibility::Hidden;
    }
}

fn update_swipe_cues(
    mut query: Query<(&mut Text, &mut Visibility), With<SwipeCueText>>,
    rt: Res<DuelRuntime>,
    debug_state: Res<DebugState>,
) {
    if matches!(*debug_state, DebugState::Animation) {
        if let Ok((_text, mut vis)) = query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }
    let show = matches!(
        rt.machine.phase,
        DuelPhase::Standoff | DuelPhase::RandomDelay | DuelPhase::GoSignal | DuelPhase::InputWindow
    );

    if let Ok((mut text, mut vis)) = query.get_single_mut() {
        if show {
            *vis = Visibility::Visible;
            let dir = correct_direction_for(rt.machine.human_opening);
            text.sections[0].value = format!("{}", dir);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn handle_go_event(
    mut commands: Commands,
    mut go_rx: EventReader<GoCue>,
    query: Query<Entity, With<GoText>>,
    debug_state: Res<DebugState>,
) {
    if matches!(*debug_state, DebugState::Animation) {
        return;
    }
    for _ in go_rx.read() {
        if let Ok(entity) = query.get_single() {
            // Pop in
            commands.entity(entity)
                .insert(Visibility::Visible)
                .insert(Transform::from_scale(Vec3::ZERO))
                .insert(Animator::new(Tween::new(
                    EaseMethod::EaseFunction(EaseFunction::ElasticOut),
                    Duration::from_millis(600),
                    TransformScaleLens {
                        start: Vec3::ZERO,
                        end: Vec3::ONE,
                    }
                ).with_completed_event(0)));
            
            commands.entity(entity).insert(Animator::new(
                Tween::new(
                    EaseMethod::EaseFunction(EaseFunction::ElasticOut),
                    Duration::from_millis(500),
                    TransformScaleLens { start: Vec3::ZERO, end: Vec3::ONE }
                ).then(
                    Tween::new(
                        EaseMethod::EaseFunction(EaseFunction::QuadraticIn),
                        Duration::from_millis(300),
                        TransformScaleLens { start: Vec3::ONE, end: Vec3::ZERO }
                    )
                )
            ));
        }
    }
}

fn update_round_indicators(
    mut query: Query<(&RoundIndicator, &mut Sprite, &mut Visibility)>,
    rt: Res<DuelRuntime>,
    debug_state: Res<DebugState>,
) {
    if matches!(*debug_state, DebugState::Animation) {
        for (_indicator, _sprite, mut vis) in query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }
    let show = matches!(rt.machine.phase, DuelPhase::ResultFlash | DuelPhase::NextRound);

    for (indicator, mut sprite, mut vis) in query.iter_mut() {
        if show {
            *vis = Visibility::Visible;
            if let Some(result) = rt.machine.round_results.get(indicator.index) {
                sprite.color = match result.outcome {
                    Outcome::HumanWin | Outcome::EarlyAi | Outcome::WrongAi => Color::srgb(0.2, 0.6, 1.0), // Blue (Human)
                    Outcome::AiWin | Outcome::EarlyHuman | Outcome::WrongHuman => Color::srgb(1.0, 0.3, 0.3), // Red (AI)
                    Outcome::Clash => Color::srgb(1.0, 1.0, 0.0),
                };
            } else {
                sprite.color = Color::srgb(0.5, 0.5, 0.5);
            }
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn handle_restart_input(
    mut rt: ResMut<DuelRuntime>,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    debug_state: Res<DebugState>,
) {
    if matches!(*debug_state, DebugState::Animation) {
        return;
    }
    if matches!(rt.machine.match_state, MatchState::HumanWon | MatchState::AiWon) {
        let tap = mouse.just_pressed(MouseButton::Left) || touches.any_just_pressed();
        if tap {
            let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
            rt.machine.reset_match(now_ms);
        }
    }
}

fn update_debug_text(
    mut query: Query<(&mut Text, &mut Visibility), With<DebugText>>,
    rt: Res<DuelRuntime>,
    debug_state: Res<DebugState>,
    char_q: Query<(&Character, &FrameIndex, Option<&DeathRespawn>, Option<&RespawnFadeIn>)>,
    char_pos_q: Query<(&Character, &Transform)>,
    controller_state: Res<CharacterControllerState>,
    frames: Res<FrameLibrary>,
    edit_mode: Res<AnimationEditMode>,
    ai_health: Res<AiHealth>,
    parry_state: Res<ParryState>,
) {
    if let Ok((mut text, mut vis)) = query.get_single_mut() {
        match *debug_state {
            DebugState::Off => {
                *vis = Visibility::Hidden;
                return;
            }
            DebugState::Animation => {
                *vis = Visibility::Visible;
                let mut human_idx = 0;
                let mut ai_idx = None;
                let mut ai_state = "Alive";
                for (c, frame_idx, death, fade) in char_q.iter() {
                    if matches!(c.actor, Actor::Human) {
                        human_idx = frame_idx.index;
                    }
                    if matches!(c.actor, Actor::Ai) {
                        ai_idx = Some(frame_idx.index);
                        if death.is_some() {
                            ai_state = "Dying";
                        } else if fade.is_some() {
                            ai_state = "Respawn";
                        }
                    }
                }
                let human_name = frames.human.name_for_index(human_idx).unwrap_or("unknown");
                let ai_name = ai_idx.and_then(|idx| frames.ai.name_for_index(idx)).unwrap_or("missing");
                let mut human_pos = None;
                let mut ai_pos = None;
                for (character, transform) in char_pos_q.iter() {
                    if matches!(character.actor, Actor::Human) {
                        human_pos = Some(transform.translation);
                    } else if matches!(character.actor, Actor::Ai) {
                        ai_pos = Some(transform.translation);
                    }
                }
                let pos_line = if let (Some(h), Some(a)) = (human_pos, ai_pos) {
                    format!(
                        "Pos H({:.1},{:.1}) AI({:.1},{:.1}) | dx {:.1} | dist {:.1}",
                        h.x,
                        h.y,
                        a.x,
                        a.y,
                        (a.x - h.x).abs(),
                        h.distance(a)
                    )
                } else {
                    "Pos: missing".to_string()
                };
                let range_line = format!(
                    "Range hit {} | ai_attack {} | ai_stop {:.1} | dodge {} | min_sep {}",
                    HIT_RANGE,
                    AI_ATTACK_RANGE,
                    AI_STOP_DISTANCE,
                    AI_DODGE_DISTANCE,
                    MIN_SEPARATION,
                );
                text.sections[0].value = format!(
                    "ANIMATION PLAYGROUND\nFolder: {}\nHuman: {} ({})\nAI: {} ({})\nAI State: {} | HP: {} | Parry: {}\n{}\n{}\nEdit: {}\nSlash: {}\nClash: {}\n[Left/Right] Cycle Frame (Edit Mode)\n[Space] Set Slash + Play\n[Enter] Set Clash + Play\n[Z] Up Attack: seq_1 press / seq_2 release\n[X] Extended: seq_1 press / seq_2+seq_3 release\n[C] Block: tap = frame1, hold = frame1+frame2\n[S] Duel: press=duel, release=fast, double=spin\n[D] Toggle Edit Mode\n[P] Save controller",
                    controller_state.controller_name,
                    human_idx,
                    human_name,
                    ai_idx.unwrap_or(0),
                    ai_name,
                    ai_state,
                    ai_health.hits_remaining,
                    if parry_state.ready { "READY" } else { "OFF" },
                    pos_line,
                    range_line,
                    if edit_mode.0 { "ON" } else { "OFF" },
                    controller_state.controller.slash_index,
                    controller_state.controller.clash_index,
                );
                return;
            }
            DebugState::Stats => {
                *vis = Visibility::Visible;
                // fall through to stats logic
            }
        }

        let m = &rt.machine;
        
        let last_outcome = if let Some(res) = m.round_results.last() {
            format!("{:?}", res.outcome)
        } else {
            "None".to_string()
        };

        let p_swipe = if let Some(s) = &m.human_swipe {
            format!("{:?} @ {}ms", s.dir, s.ts_ms.saturating_sub(m.go_ts_ms.unwrap_or(0)))
        } else {
            "Waiting...".to_string()
        };

        let state_str = format!("{:?}", m.match_state);
        let valid_dir = correct_direction_for(m.human_opening);

        let mut human_pos = None;
        let mut ai_pos = None;
        for (character, transform) in char_pos_q.iter() {
            if matches!(character.actor, Actor::Human) {
                human_pos = Some(transform.translation);
            } else if matches!(character.actor, Actor::Ai) {
                ai_pos = Some(transform.translation);
            }
        }
        let pos_line = if let (Some(h), Some(a)) = (human_pos, ai_pos) {
            format!(
                "Pos H({:.1},{:.1}) AI({:.1},{:.1}) | dx {:.1} | dist {:.1}",
                h.x,
                h.y,
                a.x,
                a.y,
                (a.x - h.x).abs(),
                h.distance(a)
            )
        } else {
            "Pos: missing".to_string()
        };
        let range_line = format!(
            "Range hit {} | ai_attack {} | ai_stop {:.1} | dodge {} | min_sep {}",
            HIT_RANGE,
            AI_ATTACK_RANGE,
            AI_STOP_DISTANCE,
            AI_DODGE_DISTANCE,
            MIN_SEPARATION,
        );
        let info = format!(
            "P1: {} | AI: {}\nRound: {}\nState: {}\nLast Outcome: {}\nP1 Swipe: {}\nInput Window: {}ms\nValid: {}\n{}\n{}",
            m.human_score,
            m.ai_score,
            m.round_results.len() + 1,
            state_str,
            last_outcome,
            p_swipe,
            m.input_window_ms,
            valid_dir,
            pos_line,
            range_line,
        );
        
        text.sections[0].value = info;
    }
}
