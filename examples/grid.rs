use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::registry::UiRegistry;

fn main() {
    let _ = App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Debug Html Grid UI".to_string(),
                    resolution: WindowResolution::new(1270.0, 720.0),
                    ..default()
                }),
                ..default()
            }
        ))
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, test_html)
        .run();
}


fn test_html(mut ui_registry: ResMut<UiRegistry>) {
    ui_registry.add_and_use(String::from("grid-example"), HtmlSource { source: String::from("examples/html/grid-ui.html"), ..default() });
}