use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use bevy_iaido::{load_log, replay_match, IaidoPlugin, IaidoSettings};

fn main() {
    // Simple CLI: --replay <path>
    let mut args = std::env::args().skip(1);
    if let Some(cmd) = args.next() {
        if cmd == "--replay" {
            if let Some(path) = args.next() {
                match load_log(&path) {
                    Some(log) => {
                        let ok = replay_match(&log);
                        println!("Replay {} for seed {}", if ok {"OK"} else {"FAILED"}, log.seed);
                    }
                    None => eprintln!("Failed to load replay: {}", path),
                }
                return;
            }
        }
    }

    App::new()
        .insert_resource(IaidoSettings::default())
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "IAIDO MVP".into(),
                        resolution: (1280., 720.).into(),
                        resizable: true,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
            IaidoPlugin,
        ))
        .run();
}
