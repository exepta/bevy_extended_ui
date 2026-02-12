use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;

/// Runs the math function example app.
fn main() {
    let mut app = make_app("Math Functions Example");
    app.add_systems(Startup, setup);
    app.run();
}

/// Loads and registers the math example UI.
fn setup(mut registry: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/math_examples.html");
    registry.add_and_use("math_examples".to_string(), HtmlSource::from_handle(handle));
}
