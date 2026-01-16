use bevy_extended_ui::html::HtmlEvent;
use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{Headline, UIWidgetState};
use bevy_extended_ui_macros::html_fn;

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let switch_button_test_handle: Handle<HtmlAsset> = asset_server.load("examples/switch_button.html");
        let overlay_test_handle: Handle<HtmlAsset> = asset_server.load("examples/overlay_ui.html");

        reg.add("switch_button_test".to_string(), HtmlSource::from_handle(switch_button_test_handle));
        reg.add("overlay_ui".to_string(), HtmlSource::from_handle(overlay_test_handle));
        reg.use_uis(vec!["switch_button_test".to_string(), "overlay_ui".to_string()]);
    });

    app.run();
}

#[html_fn("text_click")]
fn text_click(
    In(event): In<HtmlEvent>,
    query: Query<&UIWidgetState>,
    mut text_query: Query<(&CssID, &mut Headline), With<Headline>>,
) {
    let Some((_text_id, mut headline)) = text_query
        .iter_mut()
        .find(|(id, _)| id.0 == "check-text")
    else {
        return;
    };

    if let Ok(state) = query.get(event.entity) {
        headline.text = format!("Value: {}", state.checked);
    }
}