use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{ColorPicker, Headline};
use bevy_extended_ui_macros::html_fn;

/// Runs the color picker example app.
fn main() {
    let mut app = make_app("Debug Html UI - color picker");

    app.add_systems(
        Startup,
        |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
            let handle: Handle<HtmlAsset> = asset_server.load("examples/color_picker.html");
            reg.add_and_use(
                "color_picker_test".to_string(),
                HtmlSource::from_handle(handle),
            );
        },
    );

    app.run();
}

/// Updates the headline with the selected color.
#[html_fn("update_color")]
fn update_color(
    In(event): In<HtmlEvent>,
    picker_q: Query<&ColorPicker>,
    mut headline_q: Query<(&CssID, &mut Headline), With<Headline>>,
) {
    let Ok(picker) = picker_q.get(event.entity) else {
        return;
    };

    let Some((_id, mut headline)) = headline_q.iter_mut().find(|(id, _)| id.0 == "picked-color")
    else {
        return;
    };

    headline.text = format!("{} | {}", picker.hex(), picker.rgba_string());
}
