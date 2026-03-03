---
title: Div
---

# Div

## Overview

Generic layout container for grouping and structuring nested widgets.

- Rust component: Div
- HTML tag: div
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- Generic grouping container.
- Ideal for CSS classes and layout composition.
- Supports nested widgets and event attributes.

## HTML Example

```html
<div id="card" class="panel" oninit="log_div">
  <p>Panel content</p>
</div>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Div;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/div.html");
    reg.add_and_use("div-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_div")]
fn log_div(In(event): In<HtmlEvent>, query: Query<&Div>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Div event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Notes

- Keep the HTML tag spelling exact (div) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
