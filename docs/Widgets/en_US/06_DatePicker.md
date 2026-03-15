---
title: DatePicker
---

# DatePicker

## Overview

Calendar picker widget that can be standalone or bound to an input via the for attribute.

- Rust component: DatePicker
- HTML tag: date-picker
- Recommended source reference: src/widgets/mod.rs

## Attributes

- Supports for, name, label, placeholder, value, min, max, format.
- Works standalone or bound to an input.
- Useful with onchange handlers for date-driven UI.

## Html Example

```html
<input id="birthday" type="date" />
<date-picker for="birthday" min="1990-01-01" max="2030-12-31" format="dmy" onchange="log_datepicker"></date-picker>
```

## Rust Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::DatePicker;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/datepicker.html");
    reg.add_and_use("datepicker-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_datepicker")]
fn log_datepicker(In(event): In<HtmlEvent>, query: Query<&DatePicker>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("DatePicker event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Preview

<iframe
  id="datepicker"
  title="Bevy WASM Preview - DatePicker"
  src="{base.url}/examples/datepicker"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (date-picker) so the converter maps to the correct widget.
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
