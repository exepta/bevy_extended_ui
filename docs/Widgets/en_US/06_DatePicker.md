---
title: DatePicker
---

# DatePicker

## Overview

Calendar picker widget that can be standalone or bound to an input via the for attribute.

- Rust component: DatePicker
- HTML tag: date-picker
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- Supports for, name, label, placeholder, value, min, max, format.
- Works standalone or bound to an input.
- Useful with onchange handlers for date-driven UI.

## HTML Example

    <input id="birthday" type="date" />
    <date-picker for="birthday" min="1990-01-01" max="2030-12-31" format="dmy" onchange="log_datepicker"></date-picker>

## Bevy Example

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::DatePicker;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/datepicker.html");
        reg.add_and_use("datepicker-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_datepicker")]
    fn log_datepicker(In(event): In<HtmlEvent>, query: Query<&DatePicker>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("DatePicker event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy WASM Preview (Placeholder)

<iframe
  title="Bevy WASM Preview - DatePicker"
  src="https://example.com/bevy-extended-ui/wasm/datepicker"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Replace the src URL with your deployed WASM preview endpoint.

## Notes

- Keep the HTML tag spelling exact (date-picker) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
