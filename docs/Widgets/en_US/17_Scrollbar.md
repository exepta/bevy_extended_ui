---
title: Scrollbar
---

# Scrollbar

## Overview

Scroll helper widget rendered from the scroll tag for vertical or horizontal scrolling.

- Rust component: Scrollbar
- HTML tag: scroll
- Recommended source reference: src/widgets/mod.rs

## Attributes

- Tag is scroll, not scrollbar.
- alignment toggles vertical/horizontal behavior.
- Useful in custom scroll interaction regions.

## Html Example

```html
<scroll alignment="vertical" onscroll="log_scrollbar"></scroll>
```

## Rust Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Scrollbar;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/scrollbar.html");
    reg.add_and_use("scrollbar-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_scrollbar")]
fn log_scrollbar(In(event): In<HtmlEvent>, query: Query<&Scrollbar>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Scrollbar event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Preview

<iframe
  id="scrollbar"
  title="Bevy WASM Preview - Scrollbar"
  src="{base.url}/examples/scrollbar"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (scroll) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.

## Widget Creator

<div style="display: flex; align-items: center; justify-content: flex-start; padding: 15px; border: 1px solid #5658db; border-radius: 10px; gap: 15px; width: 300px;">
  <img
    src="https://avatars.githubusercontent.com/u/84874606?v=4"
    alt="exepta avatar"
    width="64"
    height="64"
    style="width: 64px; height: 64px; border-radius: 50%; object-fit: cover;"
  />
  <div style="display: flex; flex-direction: column; align-items: flex-start; justify-content: center;">
    <strong>exepta</strong>
    <a href="https://github.com/exepta" style="margin-top: 10px;">Link to GitHub</a>
  </div>
</div>
