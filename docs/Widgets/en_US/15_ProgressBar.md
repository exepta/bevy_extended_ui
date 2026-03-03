---
title: ProgressBar
---

# ProgressBar

## Overview

Range-based display widget for showing numeric progress between min and max.

- Rust component: ProgressBar
- HTML tag: progressbar
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- Range values via min, max, value.
- Best updated by app/game state systems.
- Good for XP, loading, health and cooldown UIs.

## HTML Example

    <progressbar id="xp" min="0" max="100" value="42" oninit="log_progressbar"></progressbar>

## Bevy Example

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::ProgressBar;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/progressbar.html");
        reg.add_and_use("progressbar-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_progressbar")]
    fn log_progressbar(In(event): In<HtmlEvent>, query: Query<&ProgressBar>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("ProgressBar event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy WASM Preview (Placeholder)

<iframe
  title="Bevy WASM Preview - ProgressBar"
  src="https://example.com/bevy-extended-ui/wasm/progressbar"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Replace the src URL with your deployed WASM preview endpoint.

## Notes

- Keep the HTML tag spelling exact (progressbar) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
