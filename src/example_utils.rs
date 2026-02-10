use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::ExtendedUiPlugin;

/// Creates a Bevy `App` configured for the extended UI examples.
pub fn make_app(title: impl Into<String>) -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            resolution: WindowResolution::new(1270, 720),
            ..default()
        }),
        ..default()
    }))
        .add_plugins(ExtendedUiPlugin);

    app
}
