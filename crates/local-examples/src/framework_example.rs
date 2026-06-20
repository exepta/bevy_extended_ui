#[cfg(feature = "extended-framework")]
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
#[cfg(feature = "extended-framework")]
use bevy_extended_ui::framework::ExtendedFrameworkConfiguration;
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};

#[cfg(feature = "extended-framework")]
pub fn run() {
    let asset_root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    let mut ui_config = ExtendedUiConfiguration::default();
    ui_config.camera = ExtendedCam::Default;
    ui_config.framework_components_path = "components".to_string();
    ui_config.assets_path = format!("{asset_root}/extended_ui/");
    ui_config.themes_path = format!("{asset_root}/themes");

    let mut framework_config = ExtendedFrameworkConfiguration::default();
    framework_config.asset_root_fs_path = asset_root.clone();
    framework_config.assets_component_root = "components".to_string();
    framework_config.index_html_file = "index.html".to_string();

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: asset_root,
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .insert_resource(ui_config)
        .insert_resource(framework_config)
        .add_plugins(ExtendedUiPlugin)
        .run();
}

#[cfg(not(feature = "extended-framework"))]
pub fn run() {
    eprintln!(
        "This example requires `extended-framework`.\nRun: cargo run --manifest-path crates/local-examples/Cargo.toml --features extended-framework -- framework"
    );
}
