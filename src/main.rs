use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::{Commands, KeyCode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::widgets::containers::DivContainer;

fn main() {
    let _ = App::new()
        .add_plugins((DefaultPlugins, ExtendedUiPlugin))
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)))
        .add_systems(Startup, example_widget)
        .run();
}

fn example_widget(mut commands: Commands) {
    commands.spawn(DivContainer);
}