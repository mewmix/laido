use bevy::prelude::*;
use bevy_iaido::*;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(IaidoSettings::default())
        .add_plugins((
            DefaultPlugins.set(WindowPlugin { primary_window: Some(Window { title: "IAIDO".into(), resolution: (720.0, 1280.0).into(), resizable: false, ..default() }), ..default() }),
            IaidoPlugin,
        ))
        .add_systems(Update, (on_go, on_slash, on_clash))
        .run();
}

fn on_go(mut ev: EventReader<GoCue>) {
    for _ in ev.read() {
        info!("GO");
    }
}

fn on_slash(mut ev: EventReader<SlashCue>) {
    for e in ev.read() {
        info!("SLASH by {:?}", e.actor);
    }
}

fn on_clash(mut ev: EventReader<ClashCue>) {
    for _ in ev.read() { info!("CLASH"); }
}
