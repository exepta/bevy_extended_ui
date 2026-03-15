---
title: CheckBox
---

# CheckBox

## Overview

Boolean input widget with label text and optional icon, used for yes/no state toggles.

- Rust component: CheckBox
- HTML tag: checkbox
- Recommended source reference: src/widgets/mod.rs

## Attributes

- Main tag is checkbox with label text.
- Optional icon attribute for check mark style.
- Checked state is readable through runtime widget state.

## Html Example

```html
<checkbox icon="extended_ui/icons/check-mark.png" onclick="log_checkbox">
  Enable music
</checkbox>
```

## Rust Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::CheckBox;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/checkbox.html");
    reg.add_and_use("checkbox-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_checkbox")]
fn log_checkbox(In(event): In<HtmlEvent>, query: Query<&CheckBox>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("CheckBox event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Preview

<iframe
  id="checkbox"
  title="Bevy WASM Preview - CheckBox"
  src="{base.url}/examples/checkbox"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (checkbox) so the converter maps to the correct widget.
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
