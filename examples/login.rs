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

#[derive(Resource)]
struct HtmlTestTimer {
    timer: Timer,
    state: u8,
}

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
        .add_systems(Startup, test_html_setup)
        .add_systems(Update, test_html_update)
        .run();
}


fn test_html_setup(mut commands: Commands, mut ui_registry: ResMut<UiRegistry>) {
    ui_registry.add_and_use(
        "login-example".to_string(),
        HtmlSource::from_file_path("examples/html/login-ui.html"),
    );

    commands.insert_resource(HtmlTestTimer {
        timer: Timer::from_seconds(10.0, TimerMode::Once),
        state: 0,
    });
}

fn test_html_update(
    time: Res<Time>,
    mut ui_registry: ResMut<UiRegistry>,
    mut html_timer: ResMut<HtmlTestTimer>,
) {
    if html_timer.timer.tick(time.delta()).just_finished() {
        match html_timer.state {
            0 => {
                ui_registry.add_and_use(
                    "grid-example".to_string(),
                    HtmlSource::from_file_path("examples/html/grid-ui.html"),
                );
                html_timer.state = 1;
                html_timer.timer.reset();
            }
            1 => {
                ui_registry.use_ui(
                    "login-example"
                );
                html_timer.state = 2;
                html_timer.timer.reset();
            }
            _ => {
            }
        }
    }
}