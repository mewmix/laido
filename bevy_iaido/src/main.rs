mod types;
mod config;
mod combat;
mod ai;
mod logging;
mod input;
mod duel;
mod events;

use bevy::prelude::*;
use config::TimingConfig;
use duel::DuelPlugin;
use input::InputPlugin;
use logging::{load_log, replay_match};

fn main() {
    // Simple CLI: --replay <path>
    let mut args = std::env::args().skip(1);
    if let Some(cmd) = args.next() {
        if cmd == "--replay" {
            if let Some(path) = args.next() {
                match load_log(&path) {
                    Some(log) => {
                        let ok = replay_match(&log);
                        println!("Replay {} for seed {}", if ok {"OK"} else {"FAILED"}, log.match_seed);
                    }
                    None => eprintln!("Failed to load replay: {}", path),
                }
                return;
            }
        }
    }

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(TimingConfig::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "IAIDO MVP".into(),
                resolution: (720., 1280.).into(), // portrait-ish
                ..default()
            }),
            ..default()
        }))
        .add_plugins((InputPlugin, DuelPlugin))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
