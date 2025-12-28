use bevy_extended_ui::html::HtmlEvent;
use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{Headline, Slider};
use bevy_extended_ui_macros::html_fn;

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/slider.html");
        reg.add_and_use("slider_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.run();
}

#[html_fn("update_value")]
fn update_value(
    In(event): In<HtmlEvent>,
    query: Query<&Slider>,
    mut text_query: Query<(&CssID, &mut Headline), With<Headline>>,
) {
    let Some((_text_id, mut headline)) = text_query
        .iter_mut()
        .find(|(id, _)| id.0 == "cur-value")
    else {
        return;
    };

    if let Ok(slider) = query.get(event.entity) {
        headline.text = format!("Value: {}", slider.value);
    }
}