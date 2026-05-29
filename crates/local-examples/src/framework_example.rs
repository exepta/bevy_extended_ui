#[cfg(feature = "extended-framework")]
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
#[cfg(feature = "extended-framework")]
use bevy_extended_ui::framework::ExtendedFrameworkConfiguration;
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};

#[cfg(feature = "extended-framework")]
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, configure_ui)
        .run();
}

#[cfg(feature = "extended-framework")]
fn configure_ui(
    mut config: ResMut<ExtendedUiConfiguration>,
    mut framework_config: ResMut<ExtendedFrameworkConfiguration>,
) {
    config.camera = ExtendedCam::Default;
    config.framework_components_path = "components".to_string();
    framework_config.asset_root_fs_path = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    framework_config.index_html_file = "index.html".to_string();
}

#[cfg(not(feature = "extended-framework"))]
pub fn run() {
    eprintln!(
        "This example requires `extended-framework`.\nRun: cargo run --manifest-path crates/local-examples/Cargo.toml --features extended-framework -- framework"
    );
}
