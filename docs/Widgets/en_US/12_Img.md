---
title: Img
---

# Img

## Overview

Image widget that renders a referenced asset path and stores alternative text metadata.

- Rust component: Img
- HTML tag: img
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- src resolves relative to HTML asset path.
- alt stores fallback text metadata.
- preview can bind image source updates to an `input type="file"` id (e.g. `preview="avatar-file"`).
- Works as a lightweight content image widget.

## HTML Example

```html
<img src="ui/logo.png" alt="Project logo" oninit="log_img" />
```

```html
<input id="avatar-file" type="file" extensions="[jpg, jpeg, png]" show-size="true" />
<img preview="avatar-file" alt="Avatar preview" />
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Img;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/img.html");
    reg.add_and_use("img-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_img")]
fn log_img(In(event): In<HtmlEvent>, query: Query<&Img>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Img event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Example

<iframe
  title="Bevy WASM Preview - Img"
  src="{base.url}/examples/img"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (img) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
