---
title: Paragraph
---

# Paragraph

## Overview

Basic text block widget for descriptive content and inline placeholder rendering.

- Rust component: Paragraph
- HTML tag: p
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- Plain text content widget using p tag.
- Works with reactive placeholders in inner content.
- Useful for status and descriptive text blocks.

## HTML Example

    <p oninit="log_paragraph">Welcome {{player.name}}</p>

## Bevy Example

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::Paragraph;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/paragraph.html");
        reg.add_and_use("paragraph-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_paragraph")]
    fn log_paragraph(In(event): In<HtmlEvent>, query: Query<&Paragraph>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("Paragraph event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy WASM Preview (Placeholder)

<iframe
  title="Bevy WASM Preview - Paragraph"
  src="https://example.com/bevy-extended-ui/wasm/paragraph"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Replace the src URL with your deployed WASM preview endpoint.

## Notes

- Keep the HTML tag spelling exact (p) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
