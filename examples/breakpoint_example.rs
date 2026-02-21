use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::Headline;

/// Runs the breakpoint example app.
fn main() {
    let mut app = make_app("Debug Html UI - breakpoints");

    app.add_systems(Startup, load_ui);
    app.add_systems(Update, update_window_size_label);

    app.run();
}

/// Loads and registers the breakpoint example UI.
fn load_ui(mut registry: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/breakpoint.html");
    registry.add_and_use(
        "breakpoint_test".to_string(),
        HtmlSource::from_handle(handle),
    );
}

/// Displays the current primary window size in the top-left label.
fn update_window_size_label(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut label_query: Query<(&CssID, &mut Headline), With<Headline>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let width = window.resolution.width().round() as u32;
    let height = window.resolution.height().round() as u32;
    let size_label = format!("{width}px x {height}px");

    for (id, mut headline) in &mut label_query {
        if id.0 == "window-size" && headline.text != size_label {
            headline.text = size_label.clone();
        }
    }
}
