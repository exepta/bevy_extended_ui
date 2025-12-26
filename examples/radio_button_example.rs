use bevy::prelude::*;
use bevy_extended_ui::example_utils::make_app;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::styles::components::UiStyle;
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::styles::paint::Colored;
use bevy_extended_ui::widgets::{FieldSelectionSingle, Headline, RadioButton};

fn main() {
    let mut app = make_app("Debug Html UI - test");

    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("examples/radio_button.html");
        reg.add_and_use("radio_test".to_string(), HtmlSource::from_handle(handle));
    });

    app.add_systems(Update, update_text);

    app.run();
}

fn update_text(
    check_query: Query<(&CssID, &FieldSelectionSingle), Changed<FieldSelectionSingle>>,
    mut text_query: Query<(&CssID, &mut UiStyle), With<Headline>>,
    radio_q: Query<&RadioButton>,
) {
    let Some((_text_id, mut text_style)) = text_query
        .iter_mut()
        .find(|(id, _)| id.0 == "my-text")
    else {
        return;
    };

    for (id, selection) in &check_query {
        if id.0 != "set" {
            continue;
        }

        let Some(sel) = selection.0 else { continue };
        let Ok(radio) = radio_q.get(sel) else { continue };
        let Some(color) = parse_color(radio.value.as_str()) else { continue };

        for pair in text_style.styles.values_mut() {
            pair.normal.color = Some(color);
        }

        text_style.active_style = None;

        break;
    }
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::srgb_u8(r, g, b));
        }
    }

    Colored::named(s)
}