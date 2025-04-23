use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::widgets::{DivContainer, Button};

fn main() {
    let _ = App::new()
        .add_plugins((DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Debug UI".to_string(),
                    resolution: WindowResolution::new(1270.0, 720.0),
                    ..default()
                }),
                ..default()
            }
        ), ExtendedUiPlugin))
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)))
        .add_systems(Startup, example_widget)
        .run();
}

fn example_widget(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DivContainer).with_children(| builder | {
        builder.spawn(
            Button::default()
        );

        builder.spawn(
            Button::default()
        );
    });
}