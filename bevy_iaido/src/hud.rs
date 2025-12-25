#![cfg(feature = "bevy")]
use bevy::prelude::*;
use crate::plugin::DuelRuntime;
use crate::types::{DuelPhase, MatchState, Outcome};

pub fn systems() -> impl Plugin {
    HudPlugin
}

struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, (
                update_onboarding,
                update_round_indicators,
                handle_restart_input,
            ));
    }
}

#[derive(Component)]
struct OnboardingText;

#[derive(Component)]
struct RoundIndicator {
    index: usize,
}

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Onboarding Text: "Swipe when you hear the sound."
    // Centered, Z=5 (Global Z via Style is implied on top, but we can set ZIndex)
    commands.spawn((
        TextBundle::from_section(
            "Swipe when you hear the sound.",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"), // Assuming default bevy font or placeholder
                font_size: 40.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            left: Val::Auto,
            right: Val::Auto,
            top: Val::Percent(20.0), // Slightly above center
            ..default()
        })
        .with_text_justify(JustifyText::Center),
        OnboardingText,
        ZIndex::Global(5),
    ));

    // Between-round overlay: 3 small bars.
    // Using Sprites as requested for "small bars" with specific Z=5.
    // Positioned centrally.
    let bar_width = 80.0;
    let bar_height = 20.0;
    let gap = 10.0;
    let start_x = -(bar_width + gap); // Centering 3 bars: -1, 0, +1

    for i in 0..3 {
        let x = start_x + (i as f32 * (bar_width + gap));
        commands.spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5), // Gray unplayed
                custom_size: Some(Vec2::new(bar_width, bar_height)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, 5.0), // Z=5
            RoundIndicator { index: i },
            Visibility::Hidden,
        ));
    }
}

fn update_onboarding(
    mut query: Query<&mut Visibility, With<OnboardingText>>,
    rt: Res<DuelRuntime>,
) {
    let mut vis = query.single_mut();
    // Visible only during first round (0) until a valid swipe occurs.
    // Assuming "valid swipe" implies we've moved past the input window or results are in.
    // If round_results is empty, we are in round 0.
    if rt.machine.round_results.is_empty() {
        // Show only during "quiet" phases before resolution
        match rt.machine.phase {
            DuelPhase::Standoff | DuelPhase::RandomDelay | DuelPhase::GoSignal | DuelPhase::InputWindow => {
                *vis = Visibility::Visible;
            }
            _ => {
                // Resolution, ResultFlash, etc. -> Hide
                *vis = Visibility::Hidden;
            }
        }
    } else {
        *vis = Visibility::Hidden;
    }
}

fn update_round_indicators(
    mut query: Query<(&RoundIndicator, &mut Sprite, &mut Visibility)>,
    rt: Res<DuelRuntime>,
) {
    // Visible only in ResultFlash / NextRound
    let show = matches!(rt.machine.phase, DuelPhase::ResultFlash | DuelPhase::NextRound);

    for (indicator, mut sprite, mut vis) in query.iter_mut() {
        if show {
            *vis = Visibility::Visible;
            // Color based on outcome
            if let Some(result) = rt.machine.round_results.get(indicator.index) {
                sprite.color = match result.outcome {
                    Outcome::HumanWin | Outcome::EarlyAi | Outcome::WrongAi => Color::srgb(1.0, 0.0, 0.0), // Human (Red)
                    Outcome::AiWin | Outcome::EarlyHuman | Outcome::WrongHuman => Color::srgb(0.0, 0.0, 1.0), // AI (Blue)
                    Outcome::Clash => Color::srgb(1.0, 1.0, 0.0), // Clash
                };
            } else {
                sprite.color = Color::srgb(0.5, 0.5, 0.5); // Gray unplayed
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
) {
    // Restart on match end: tap/click
    if matches!(rt.machine.match_state, MatchState::HumanWon | MatchState::AiWon) {
        let tap = mouse.just_pressed(MouseButton::Left) || touches.any_just_pressed();
        if tap {
            let now_ms = (time.elapsed_seconds_f64() * 1000.0) as u64;
            rt.machine.reset_match(now_ms);
        }
    }
}
