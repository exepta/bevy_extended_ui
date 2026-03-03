---
title: RadioButton
---

# RadioButton

## Overview

Single-choice option widget that is usually managed in a fieldset group.

- Rust component: RadioButton
- HTML tag: radio
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- value carries the semantic selection payload.
- selected marks initial active item.
- Typically managed inside a fieldset.

## HTML Example

```html
<radio value="de" selected onchange="log_radiobutton">Deutsch</radio>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::RadioButton;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/radiobutton.html");
    reg.add_and_use("radiobutton-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_radiobutton")]
fn log_radiobutton(In(event): In<HtmlEvent>, query: Query<&RadioButton>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("RadioButton event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Example

<iframe
  title="Bevy WASM Preview - RadioButton"
  src="{base.url}/examples/radiobutton"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (radio) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
