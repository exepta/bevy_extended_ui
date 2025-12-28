use bevy_extended_ui::html::{HtmlEvent, HtmlEventObject};
use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::components::UiStyle;
use bevy_extended_ui::styles::{CssID, FontVal};
use bevy_extended_ui::styles::paint::Colored;
use bevy_extended_ui::widgets::{FieldSelectionMulti, Headline, ToggleButton};
use bevy_extended_ui_macros::html_fn;

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/toggle_button.html");
        reg.add_and_use("toggle_button_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.run();
}

#[html_fn("text_format")]
fn text_color_from_set(
    In(event): In<HtmlEvent>,
    text_query: Query<(&CssID, &mut UiStyle), With<Headline>>,
    set_q: Query<&FieldSelectionMulti>,
    toggle_q: Query<&ToggleButton>,
) {
    match event.object {
        HtmlEventObject::Change(_) | HtmlEventObject::Init(_) => {
            apply_selected_radio_color_to_text(text_query, set_q, toggle_q, event.entity);
        }
        _ => {}
    }
}

fn apply_selected_radio_color_to_text(
    mut text_query: Query<(&CssID, &mut UiStyle), With<Headline>>,
    set_q: Query<&FieldSelectionMulti>,
    toggle_q: Query<&ToggleButton>,
    set_entity: Entity,
) {
    let Ok(selections) = set_q.get(set_entity) else { return };

    let Some((_text_id, mut text_style)) = text_query
        .iter_mut()
        .find(|(id, _)| id.0 == "my-text")
    else { return };

    for pair in text_style.styles.values_mut() {
        pair.normal.color = Some(Colored::hex_to_color("#e8e8fd"));
        pair.normal.font_size = Some(FontVal::Px(32.0));
    }

    for &entity in selections.0.iter() {
        let Ok(toggle) = toggle_q.get(entity) else { continue };

        match toggle.value.as_str() {
            "text_color" => {
                if let Some(color) = Colored::named("yellow") {
                    for pair in text_style.styles.values_mut() {
                        pair.normal.color = Some(color);
                    }
                }
            }
            "text_size" => {
                for pair in text_style.styles.values_mut() {
                    pair.normal.font_size = Some(FontVal::Px(12.0));
                }
            }
            "test_bold" => {
                info!("Bigger!");
            }
            _ => {}
        }
    }
}