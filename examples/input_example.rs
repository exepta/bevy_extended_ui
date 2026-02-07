use bevy_extended_ui::html::{HtmlEvent, HtmlKeyUp};
use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{Headline, InputValue};
use bevy_extended_ui_macros::html_fn;

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/input.html");
        reg.add_and_use("input_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.run();
}

#[html_fn("on_text_change")]
fn on_text_change(
    In(event): In<HtmlEvent>,
    query: Query<&InputValue>,
    mut text_query: Query<(&CssID, &mut Headline), With<Headline>>,
) {
    if let Ok(input_value) = query.get(event.entity) {
        let Some((_text_id, mut headline)) = text_query
            .iter_mut()
            .find(|(id, _)| id.0 == "in-text")
        else {
            return;
        };

        headline.text = input_value.0.clone();
    }
}