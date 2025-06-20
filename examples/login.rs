mod controller;

use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::registry::UiRegistry;
use crate::controller::ControllerPlugin;

fn main() {
    let _ = App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Debug Html UI".to_string(),
                    resolution: WindowResolution::new(1270.0, 720.0),
                    ..default()
                }),
                ..default()
            }
        ))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)))
        .add_plugins((ExtendedUiPlugin, ControllerPlugin))
        .add_systems(Startup, test_html)
        .run();
}


fn test_html(mut ui_registry: ResMut<UiRegistry>) {
    ui_registry.add_and_use(String::from("login-example"), HtmlSource { source: String::from("examples/html/login-ui.html"), ..default() });
}