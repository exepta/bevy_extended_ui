---
title: Divider
---

# Divider

## Overview

Visual separator widget with horizontal or vertical orientation.

- Rust component: Divider
- HTML tag: divider
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- alignment accepts horizontal or vertical.
- Short aliases like h and v are parsed.
- Intended as pure visual separator.

## HTML Example

```html
<divider alignment="horizontal" oninit="log_divider"></divider>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Divider;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/divider.html");
    reg.add_and_use("divider-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_divider")]
fn log_divider(In(event): In<HtmlEvent>, query: Query<&Divider>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Divider event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Example

<iframe
  title="Bevy WASM Preview - Divider"
  src="{base.url}/examples/divider"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (divider) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
