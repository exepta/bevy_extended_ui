---
title: Badge
---

# Badge

## Overview

Small notification indicator widget, usually attached to another target widget.

- Rust component: Badge
- HTML tag: badge
- Recommended source reference: src/widgets/mod.rs

## Attributes

- Supports `for`, `value` (alias: `count`), `max`, `anchor`.
- If `value > max`, the label is rendered as `+{max}` (for example `+99`).
- Binding target resolution:
- `for="id"` binds to the widget with that id.
- Without `for`, the nearest widget parent is used.
- If no target can be resolved, the badge stays hidden.
- Anchor values: `top right` (default), `top left`, `bottom right`, `bottom left`.

## Html Example

```html
<button id="inbox-button">Inbox</button>
<badge for="inbox-button" value="112" max="99" anchor="top right"></badge>
```

## Rust Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Badge;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .add_systems(Update, tick_badges)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/widgets_overview.html");
    reg.add_and_use("badge-demo".to_string(), HtmlSource::from_handle(handle));
}

fn tick_badges(mut query: Query<&mut Badge>) {
    for mut badge in &mut query {
        badge.value = (badge.value + 1) % 130;
    }
}
```

## WASM Preview

<iframe
  id="badge"
  title="Bevy WASM Preview - Badge"
  src="{base.url}/examples/badge"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (`badge`) so the converter maps to the correct widget.
- Use `for` when the badge should be attached outside of parent-child structure.
- Style with `badge` and `.badge-text` CSS selectors for shape and label appearance.

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
