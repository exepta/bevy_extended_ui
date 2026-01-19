use bevy::app::Startup;
use bevy::asset::{AssetServer, Handle};
use bevy::prelude::{Res, ResMut};
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/key_frame.html");
        reg.add_and_use("key_frame_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.run();
}