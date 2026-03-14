---
title: FieldSet
---

# FieldSet

## Overview

Grouping widget for selectable children such as radio and toggle items with selection rules.

- Rust component: FieldSet
- HTML tag: fieldset
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- mode supports single, multi, count.
- allow-none controls empty selection behavior.
- Groups radio/toggle children with shared selection state.

## HTML Example

```html
<fieldset mode="single" allow-none="false" onchange="log_fieldset">
  <radio value="easy">Easy</radio>
  <radio value="hard" selected>Hard</radio>
</fieldset>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::FieldSet;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/fieldset.html");
    reg.add_and_use("fieldset-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_fieldset")]
fn log_fieldset(In(event): In<HtmlEvent>, query: Query<&FieldSet>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("FieldSet event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Example

<iframe
  title="Bevy WASM Preview - FieldSet"
  src="{base.url}/examples/fieldset"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (fieldset) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
