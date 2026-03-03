---
title: SwitchButton
---

# SwitchButton

## Overview

Switch-like binary control widget with label and optional icon.

- Rust component: SwitchButton
- HTML tag: switch
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- Label text is taken from inner content.
- Optional icon attribute supported.
- Checked-like state can be read via UIWidgetState.

## HTML Example

    <switch icon="extended_ui/icons/drop-arrow.png" onclick="log_switchbutton">Dark mode</switch>

## Bevy Example

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::SwitchButton;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/switchbutton.html");
        reg.add_and_use("switchbutton-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_switchbutton")]
    fn log_switchbutton(In(event): In<HtmlEvent>, query: Query<&SwitchButton>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("SwitchButton event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy WASM Preview (Placeholder)

<iframe
  title="Bevy WASM Preview - SwitchButton"
  src="https://example.com/bevy-extended-ui/wasm/switchbutton"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Replace the src URL with your deployed WASM preview endpoint.

## Notes

- Keep the HTML tag spelling exact (switch) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
