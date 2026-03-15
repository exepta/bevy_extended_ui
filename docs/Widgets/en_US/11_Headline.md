---
title: Headline
---

# Headline

## Overview

Heading widget generated from h1 to h6 tags for section titles and semantic structure.

- Rust component: Headline
- HTML tag: h1-h6
- Recommended source reference: src/widgets/mod.rs

## Attributes

- h1 to h6 map to one Headline component type.
- Stores text and heading level metadata.
- Can be restyled dynamically via systems.

## Html Example

```html
<h2 oninit="log_headline">Settings</h2>
```

## Rust Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Headline;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/headline.html");
    reg.add_and_use("headline-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_headline")]
fn log_headline(In(event): In<HtmlEvent>, query: Query<&Headline>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Headline event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Preview

<iframe
  id="headline"
  title="Bevy WASM Preview - Headline"
  src="{base.url}/examples/headline"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (h1-h6) so the converter maps to the correct widget.
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
