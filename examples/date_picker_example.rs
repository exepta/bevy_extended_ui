use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlEvent;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{Headline, InputValue};
use bevy_extended_ui_macros::html_fn;

/// Runs the date picker example app.
fn main() {
    let mut app = make_app("Date Picker Example");

    app.add_systems(
        Startup,
        |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
            let handle: Handle<HtmlAsset> = asset_server.load("examples/date_picker.html");
            reg.add_and_use(
                "date_picker_test".to_string(),
                HtmlSource::from_handle(handle),
            );
        },
    );

    app.run();
}

/// Updates the headline when the date value changes.
#[html_fn("on_date_change")]
fn on_date_change(
    In(event): In<HtmlEvent>,
    query: Query<&InputValue>,
    mut text_query: Query<(&CssID, &mut Headline), With<Headline>>,
) {
    let Ok(value) = query.get(event.entity) else {
        return;
    };

    let Some((_id, mut headline)) = text_query
        .iter_mut()
        .find(|(id, _)| id.0 == "selected-date")
    else {
        return;
    };

    headline.text = if value.0.is_empty() {
        "Selected: -".to_string()
    } else {
        format!("Selected: {}", value.0)
    };
}
