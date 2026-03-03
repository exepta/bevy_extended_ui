---
title: Slider
---

# Slider

## Overview

Numeric drag control widget with min, max, value and step properties.

- Rust component: Slider
- HTML tag: slider
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- min, max, value, step control numeric interaction.
- Works with onchange/oninput style handlers.
- Current value is stored in Slider component.

## HTML Example

    <slider min="0" max="100" value="25" step="1" onchange="log_slider"></slider>

## Bevy Example

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::Slider;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/slider.html");
        reg.add_and_use("slider-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_slider")]
    fn log_slider(In(event): In<HtmlEvent>, query: Query<&Slider>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("Slider event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy WASM Preview (Placeholder)

<iframe
  title="Bevy WASM Preview - Slider"
  src="https://example.com/bevy-extended-ui/wasm/slider"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Replace the src URL with your deployed WASM preview endpoint.

## Notes

- Keep the HTML tag spelling exact (slider) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
