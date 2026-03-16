use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use bevy_extended_ui::html::{HtmlClick, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::providers::ThemeProvider;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};
use bevy_extended_ui_macros::html_fn;

const UI_KEY: &str = "theme-provider-single";

pub fn run() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Extended UI WASM Theme Provider Demo".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Extended UI Theme Provider Demo (Native Fallback)".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: resolve_asset_root(),
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        );
    }

    app.add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, (spawn_camera, configure_ui, load_ui))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn configure_ui(mut config: ResMut<ExtendedUiConfiguration>) {
    config.camera = ExtendedCam::Default;

    #[cfg(target_arch = "wasm32")]
    {
        // wasm32 has no filesystem theme discovery; provide explicit names.
        config.theme_names = vec!["dark".to_string(), "light".to_string()];
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        config.themes_path = format!("{}/assets/themes", env!("CARGO_MANIFEST_DIR"));
    }
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/theme_provider_single.html");
    reg.add_and_use(UI_KEY.to_string(), HtmlSource::from_handle(handle));
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_asset_root() -> String {
    format!("{}/assets", env!("CARGO_MANIFEST_DIR"))
}

#[html_fn("toggle_theme")]
fn toggle_theme(In(_event): In<HtmlClick>) {
    ThemeProvider::switch_next_theme();
}
