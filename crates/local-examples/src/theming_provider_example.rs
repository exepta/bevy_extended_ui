use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlClick, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::providers::ThemeProvider;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};
use bevy_extended_ui_macros::html_fn;

const UI_KEY: &str = "theme-provider-single";

pub fn run() {
    let asset_root = resolve_asset_root();

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: asset_root,
            // keep startup resilient when .meta files are missing in local examples
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, (spawn_camera, configure_ui, load_ui))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn configure_ui(mut config: ResMut<ExtendedUiConfiguration>) {
    config.camera = ExtendedCam::Default;
    config.themes_path = format!("{}/assets/themes", env!("CARGO_MANIFEST_DIR"));
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/theme_provider_single.html");
    reg.add_and_use(UI_KEY.to_string(), HtmlSource::from_handle(handle));
}

fn resolve_asset_root() -> String {
    format!("{}/assets", env!("CARGO_MANIFEST_DIR"))
}

#[html_fn("toggle_theme")]
fn toggle_theme(In(_event): In<HtmlClick>) {
    ThemeProvider::switch_next_theme();
}
