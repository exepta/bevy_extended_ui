---
title: Base System
---

# How-to: Base System with `UiRegistry`

This guide explains the classic `UiRegistry` flow (without `extended-framework`).

## When should you use this?

Use the base system when you want:

- explicit named HTML source loading/switching
- dynamic runtime switching between multiple UI screens
- the legacy integration style

Important: `UiRegistry` does not work when `extended-framework` is enabled.

## 1) Dependencies

```toml
[dependencies]
bevy_extended_ui = "x.x.x"
bevy_extended_ui_macros = "x.x.x"
```

Do not enable the `extended-framework` feature.

## 2) Load and activate one UI

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::old::registry::UiRegistry;
use bevy_extended_ui::ExtendedUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_main_ui)
        .run();
}

fn load_main_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/main.html");
    reg.add_and_use("main-ui".to_string(), HtmlSource::from_handle(handle));
}
```

## 3) Switch UI screens at runtime

```rust
use bevy::prelude::*;
use bevy_extended_ui::old::registry::UiRegistry;

#[derive(Resource, Default)]
struct UseSettings(bool);

fn toggle_ui(input: Res<ButtonInput<KeyCode>>, mut reg: ResMut<UiRegistry>, mut flag: ResMut<UseSettings>) {
    if !input.just_pressed(KeyCode::F1) {
        return;
    }

    flag.0 = !flag.0;
    if flag.0 {
        reg.use_ui("settings-ui");
    } else {
        reg.use_ui("main-ui");
    }
}
```

## 4) Activate multiple UIs at once

If you want layered layouts in parallel:

```rust
fn activate_overlay_and_hud(mut reg: ResMut<UiRegistry>) {
    reg.use_uis(vec!["hud-ui".to_string(), "overlay-ui".to_string()]);
}
```

## 5) Register multiple sources

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::old::registry::UiRegistry;

fn register_all(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    reg.add_and_use_multiple(vec![
        (
            "main-ui".to_string(),
            HtmlSource::from_handle(asset_server.load("ui/main.html")),
        ),
        (
            "settings-ui".to_string(),
            HtmlSource::from_handle(asset_server.load("ui/settings.html")),
        ),
    ]);
}
```

## 6) Form/click handlers with `#[html_fn]`

HTML:

```html
<form action="save_profile">
  <input name="username" />
  <button type="submit">Save</button>
</form>
```

Rust:

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlClick, HtmlSubmit};
use bevy_extended_ui_macros::html_fn;

#[html_fn("save_profile")]
fn save_profile(In(event): In<HtmlSubmit>) {
    let username = event.data.get("username").cloned().unwrap_or_default();
    info!("Saving profile for: {}", username);
}

#[html_fn("generic_click")]
fn generic_click(In(event): In<HtmlClick>) {
    info!("Click on entity {:?}", event.entity);
}
```

## 7) Common pitfalls

1. `UiRegistry` is blocked (panic) when `extended-framework` is active.
2. `use_ui("name")` only works for names that were registered before.
3. `HtmlSubmit` provides `HashMap<String, String>` values; parse to typed values yourself.
4. HTML handler name and `#[html_fn("...")]` name must match exactly.

