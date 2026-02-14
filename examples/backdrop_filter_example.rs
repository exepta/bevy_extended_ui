use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;

/// Runs the backdrop-filter showcase example.
fn main() {
    let mut app = make_app("Backdrop Filter Example");
    app.add_systems(Startup, setup);
    app.run();
}

/// Loads and registers the backdrop-filter example UI.
fn setup(mut registry: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/backdrop_filter_example.html");
    registry.add_and_use(
        "backdrop_filter_example".to_string(),
        HtmlSource::from_handle(handle),
    );
}
