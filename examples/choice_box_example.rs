use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{ChoiceBox, Headline};
use bevy_extended_ui_macros::html_fn;

/// Runs the choice box example app.
fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/choice_box.html");
        reg.add_and_use("choice_box_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.run();
}

/// Updates headline text when a choice box selection changes.
#[html_fn("on_select_change")]
fn on_select_change(
    In(event): In<HtmlEvent>,
    query: Query<&ChoiceBox>,
    mut text_query: Query<(&CssID, &mut Headline), With<Headline>>,
) {
    let Some((_text_id, mut headline)) = text_query
        .iter_mut()
        .find(|(id, _)| id.0 == "sel-text")
    else {
        return;
    };

    if let Ok(choice_box) = query.get(event.entity) {
        headline.text = choice_box.value.text.clone();
    }
}
