---
title: Body
---

# Body

## Overview

Root container widget that represents the HTML page body and anchors one parsed UI tree.

- Rust component: Body
- HTML tag: body
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- Root parent for parsed child widgets.
- Stores html_key for active UI entry.
- Accepts global id/class/style/events attributes.

## HTML Example

```html
<body oninit="log_body">
  <div class="screen-root">...</div>
</body>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Body;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/body.html");
    reg.add_and_use("body-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_body")]
fn log_body(In(event): In<HtmlEvent>, query: Query<&Body>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Body event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Notes

- Keep the HTML tag spelling exact (body) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
