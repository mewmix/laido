use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum VirtualKey {
    Up, Down, Left, Right, Z, X, S, C, Space, Enter, P
}

#[derive(Component)]
pub struct VirtualKeyBtn(pub VirtualKey);

pub struct TouchControlsPlugin;

impl Plugin for TouchControlsPlugin {
    fn build(&self, app: &mut App) {
        // Register the custom input type
        // Bevy's InputPlugin doesn't automatically add it for custom types unless we add InputPlugin::<VirtualKey>::default() which doesn't exist?
        // Actually Bevy's InputPlugin is for KeyCode, MouseButton, GamepadButton.
        // We can just init the resource Input<VirtualKey>.
        app.init_resource::<ButtonInput<VirtualKey>>() // Bevy 0.14 uses ButtonInput instead of Input
           .add_systems(Startup, setup_touch_ui)
           .add_systems(PreUpdate, update_virtual_keys);
    }
}

fn setup_touch_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let button_style = Style {
        width: Val::Px(64.0),
        height: Val::Px(64.0),
        margin: UiRect::all(Val::Px(8.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    };
    
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 24.0,
        color: Color::WHITE,
    };

    // Container for controls
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        z_index: ZIndex::Global(100), // On top of everything
        ..default()
    }).with_children(|parent| {
        // Right side: Actions
        parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                right: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            ..default()
        }).with_children(|col| {
            // Row 1: Z X C
            col.spawn(NodeBundle {
                style: Style { flex_direction: FlexDirection::Row, ..default() },
                ..default()
            }).with_children(|row| {
                 spawn_btn(row, VirtualKey::Z, "Z", button_style.clone(), text_style.clone());
                 spawn_btn(row, VirtualKey::X, "X", button_style.clone(), text_style.clone());
                 spawn_btn(row, VirtualKey::C, "C", button_style.clone(), text_style.clone());
            });
            // Row 2: S Space
            col.spawn(NodeBundle {
                style: Style { flex_direction: FlexDirection::Row, ..default() },
                ..default()
            }).with_children(|row| {
                 spawn_btn(row, VirtualKey::S, "S", button_style.clone(), text_style.clone());
                 spawn_btn(row, VirtualKey::Space, "DASH", Style { width: Val::Px(120.0), ..button_style.clone() }, text_style.clone());
            });
        });
        
        // Save button (P) top right
        parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            },
            ..default()
        }).with_children(|col| {
            spawn_btn(col, VirtualKey::P, "SAVE", button_style.clone(), text_style.clone());
        });

        // Left side: Movement (D-Pad)
        parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).with_children(|col| {
             spawn_btn(col, VirtualKey::Up, "^", button_style.clone(), text_style.clone());
             col.spawn(NodeBundle {
                style: Style { flex_direction: FlexDirection::Row, ..default() },
                ..default()
             }).with_children(|row| {
                 spawn_btn(row, VirtualKey::Left, "<", button_style.clone(), text_style.clone());
                 spawn_btn(row, VirtualKey::Down, "v", button_style.clone(), text_style.clone());
                 spawn_btn(row, VirtualKey::Right, ">", button_style.clone(), text_style.clone());
             });
        });
    });
}

fn spawn_btn(parent: &mut ChildBuilder, key: VirtualKey, label: &str, style: Style, text_style: TextStyle) {
    parent.spawn((
        ButtonBundle {
            style,
            background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
            border_color: BorderColor(Color::WHITE),
            ..default()
        },
        VirtualKeyBtn(key),
    )).with_children(|btn| {
        btn.spawn(TextBundle::from_section(label, text_style));
    });
}

fn update_virtual_keys(
    mut inputs: ResMut<ButtonInput<VirtualKey>>,
    interaction_q: Query<(&Interaction, &VirtualKeyBtn), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interaction_q.iter() {
        match interaction {
            Interaction::Pressed => inputs.press(btn.0),
            Interaction::None | Interaction::Hovered => inputs.release(btn.0),
        }
    }
}