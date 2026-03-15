---
title: HyperLink
---

# HyperLink

## Overview

Clickable text link widget mapped from the HTML `<a>` tag.

- Rust component: HyperLink
- HTML tag: a
- Recommended source reference: src/widgets/mod.rs

## Attributes

- `href` defines the target URL.
- `browsers` is optional (default: `system`).
- Supported values for `browsers`: `system`, single browser (`firefox`, `chrome`, `brave`, ...), or list (`[firefox, brave, chrome]`).
- `open-modal` is optional (default: `false`).
- If `open-modal="true"` and the configured browser is not installed, a Bevy-app modal asks whether the install command should be opened in a terminal.
- Native platform browser detection is implemented for Linux, macOS, and Windows.
- On `wasm32`, `browsers` and `open-modal` are ignored.
- No system-browser fallback is used when explicit `browsers` are configured and none are installed.

## Html Example

```html
<a href="https://bevy.org">Open with system browser</a>
<a href="https://bevy.org" browsers="[chrome]" open-modal="true">Open with configured browser</a>
```

## Rust Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::HyperLink;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .add_systems(Update, update_link_target)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/widgets_overview.html");
    reg.add_and_use("hyperlink-demo".to_string(), HtmlSource::from_handle(handle));
}

fn update_link_target(mut query: Query<&mut HyperLink>) {
    for mut link in &mut query {
        if link.href.is_empty() {
            link.href = "https://bevyengine.org".to_string();
        }
    }
}
```

## WASM Preview

<iframe
  id="hyperlink"
  title="Bevy WASM Preview - HyperLink"
  src="{base.url}/examples/widgets-overview"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (`a`) so the converter maps to the correct widget.
- Use `open-modal="true"` only when you want an install prompt flow for missing configured browsers.
- Style links with the `a` selector in CSS.

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
