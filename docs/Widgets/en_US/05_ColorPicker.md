---
title: ColorPicker
---

# ColorPicker

## Overview

Interactive color widget that supports RGB/RGBA/HEX style workflows and change events.

- Rust component: ColorPicker
- HTML tag: colorpicker
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- value defines start color; alpha can be configured.
- Change events can trigger live UI updates.
- Component exposes rgb/hsv/hex helper values.

## HTML Example

```html
<colorpicker value="#4285f4" alpha="255" onchange="log_colorpicker"></colorpicker>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::ColorPicker;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/colorpicker.html");
    reg.add_and_use("colorpicker-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_colorpicker")]
fn log_colorpicker(In(event): In<HtmlEvent>, query: Query<&ColorPicker>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("ColorPicker event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Example

<iframe
  title="Bevy WASM Preview - ColorPicker"
  src="{base.url}/examples/colorpicker"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (colorpicker) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
