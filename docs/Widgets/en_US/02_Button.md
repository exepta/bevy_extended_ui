---
title: Button
---

# Button

## Overview

Clickable action widget with optional icon and optional type behavior for form integration.

- Rust component: Button
- HTML tag: button
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- type=button|submit|reset controls form behavior.
- Supports icon child nodes.
- Typical events: onclick, onfocus, onmouseover, onmouseout, oninit.

## HTML Example

```html
<button onclick="log_button" type="button">
  Save
  <icon src="extended_ui/icons/check-mark.png"></icon>
</button>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Button;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/button.html");
    reg.add_and_use("button-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_button")]
fn log_button(In(event): In<HtmlEvent>, query: Query<&Button>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Button event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Example

<iframe
  title="Bevy WASM Preview - Button"
  src="{base.url}/examples/button"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (button) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
