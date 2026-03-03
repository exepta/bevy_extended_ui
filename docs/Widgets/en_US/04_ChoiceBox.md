---
title: ChoiceBox
---

# ChoiceBox

## Overview

Dropdown selection widget based on select/option entries with one active value.

- Rust component: ChoiceBox
- HTML tag: select
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- Uses select with nested option entries.
- selected on option sets initial active value.
- Optional icon attribute per option.

## HTML Example

```html
<select onchange="log_choicebox" id="quality">
  <option value="low">Low</option>
  <option selected value="high">High</option>
</select>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::ChoiceBox;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/choicebox.html");
    reg.add_and_use("choicebox-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_choicebox")]
fn log_choicebox(In(event): In<HtmlEvent>, query: Query<&ChoiceBox>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("ChoiceBox event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Example

<iframe
  title="Bevy WASM Preview - ChoiceBox"
  src="{base.url}/examples/choicebox"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (select) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
