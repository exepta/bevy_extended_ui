---
title: ToolTip
---

# ToolTip

## Overview

Context hint widget that can follow the cursor or point to a target element.

- Rust component: ToolTip
- HTML tag: tool-tip
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- Supports for, variant, prio, alignment, trigger.
- Modes: follow (cursor) and point (anchored).
- Can be attached implicitly (parent) or explicitly (for=id).

## HTML Example

    <button id="help">?</button>
    <tool-tip for="help" variant="point" prio="right" alignment="horizontal" trigger="hover | click" oninit="log_tooltip">
      More information
    </tool-tip>

## Bevy Example

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::ToolTip;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/tooltip.html");
        reg.add_and_use("tooltip-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_tooltip")]
    fn log_tooltip(In(event): In<HtmlEvent>, query: Query<&ToolTip>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("ToolTip event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy WASM Preview (Placeholder)

<iframe
  title="Bevy WASM Preview - ToolTip"
  src="https://example.com/bevy-extended-ui/wasm/tooltip"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Replace the src URL with your deployed WASM preview endpoint.

## Notes

- Keep the HTML tag spelling exact (tool-tip) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
