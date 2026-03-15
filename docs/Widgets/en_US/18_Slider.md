---
title: Slider
---

# Slider

## Overview

Numeric drag control widget with min, max, value and step properties.
Supports `default` (single thumb) and `range` (two thumbs).

- Rust component: Slider
- HTML tag: slider
- Recommended source reference: src/widgets/mod.rs

## Attributes

- `min`, `max`, `step`: numeric bounds and step size.
- `type`: `default | range` (default: `default`).
- `value`:
  - `default`: single numeric value (for example `35`)
  - `range`: `start - end` format (for example `20 - 40`)
- `dots`: number of segments/ticks. Example `dots="5"` for `0..100` creates ticks at `0, 20, 40, 60, 80, 100`.
  - If `dots` is present and `<= 1` (including `0` or negative), only min/max ticks are used (same as `dots="1"`).
  - Internal safeguard: minimum value gap between neighboring ticks is 10 units.
- `show-labels`: `true | false` (default: `false`). Shows tick labels.
- `dot-anchor`: `top | bottom` (default: `top`). Places tick labels above or below the track.
- `tip`: `true | false` (default: `true`). Enables/disables thumb tooltips.
- Track reacts to click and drag.
- In `range` mode, tooltip text is per-thumb (hovered thumb value).
- Works with onchange/oninput style handlers.
- Current value is stored in Slider component.

## Html Example

```html
<slider min="0" max="100" value="25" step="1" onchange="log_slider"></slider>
```

```html
<slider
  type="range"
  min="0"
  max="100"
  value="20 - 40"
  step="1"
  dots="5"
  show-labels="true"
  dot-anchor="top"
  tip="true">
</slider>
```

## Rust Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Slider;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/slider.html");
    reg.add_and_use("slider-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_slider")]
fn log_slider(In(event): In<HtmlEvent>, query: Query<&Slider>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Slider event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Preview

<iframe
  id="slider"
  title="Bevy WASM Preview - Slider"
  src="{base.url}/examples/slider"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (slider) so the converter maps to the correct widget.
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
