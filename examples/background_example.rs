use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;

/// Runs the background image example app.
fn main() {
    let mut app = make_app("Background Image Example");
    app.add_systems(Startup, setup);
    app.run();
}

/// Loads and registers the background example UI.
fn setup(mut registry: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/background_examples.html");
    registry.add_and_use("background_examples".to_string(), HtmlSource::from_handle(handle));
}
