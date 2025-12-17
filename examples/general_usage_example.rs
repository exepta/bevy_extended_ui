use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    let _ = App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Debug Html UI".to_string(),
                resolution: WindowResolution::new(1270, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
        )
        .add_plugins(ExtendedUiPlugin)
        .run();
}
