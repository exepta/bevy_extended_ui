use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;

fn main() {
    let mut app = make_app("Wildcard Selector Test");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut registry: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/wildcard.html");
    // Register the wildcard test UI
    registry.add_and_use("wildcard_test".to_string(), HtmlSource::from_handle(handle));
}
