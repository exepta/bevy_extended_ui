use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Debug Html UI".to_string(),
                resolution: WindowResolution::new(1270, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
        )
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_main_html)
        .run();
}

fn load_main_html(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/test.html");
    reg.add_and_use("test".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("test_fn")]
fn test_fn(mut _commands: Commands) {
    info!("Test fn called!");
}

#[html_fn("test_fn2")]
fn test_fn2(mut _commands: Commands) {
    info!("Test fn over!");
}

#[html_fn("test_fn3")]
fn test_fn3(mut _commands: Commands) {
    info!("Test fn out!");
}