---
title: Base System
---

# How-to: Base System mit `UiRegistry`

Dieser Guide beschreibt das klassische System mit `UiRegistry` (ohne `extended-framework`).

## Wann dieses System nutzen?

Nutze das Base-System, wenn du:

- HTML-Dateien gezielt per Namen laden/schalten willst
- mehrere UI-Screens dynamisch austauschen willst
- bewusst beim Legacy-Flow bleiben möchtest

Wichtig: `UiRegistry` funktioniert nicht mit aktivem `extended-framework`.

## 1) Abhängigkeiten

```toml
[dependencies]
bevy_extended_ui = "x.x.x"
bevy_extended_ui_macros = "x.x.x"
```

Kein `extended-framework` Feature aktivieren.

## 2) Eine UI laden und aktivieren

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

## 3) Zwischen UIs umschalten

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

## 4) Mehrere UIs gleichzeitig aktivieren

Wenn mehrere Layouts parallel angezeigt werden sollen:

```rust
fn activate_overlay_and_hud(mut reg: ResMut<UiRegistry>) {
    reg.use_uis(vec!["hud-ui".to_string(), "overlay-ui".to_string()]);
}
```

## 5) Registrieren mehrerer Quellen

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

## 6) Form/Click Handler mit `#[html_fn]`

HTML:

```html
<form action="save_profile">
  <input name="username" />
  <button type="submit">Speichern</button>
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
    info!("Speichere Profil für: {}", username);
}

#[html_fn("generic_click")]
fn generic_click(In(event): In<HtmlClick>) {
    info!("Klick auf Entity {:?}", event.entity);
}
```

## 7) Typische Stolperfallen

1. `UiRegistry` wird mit `extended-framework` zur Laufzeit blockiert (Panic).
2. `use_ui("name")` funktioniert nur für zuvor registrierte Namen.
3. Bei `HtmlSubmit` kommen Daten als `HashMap<String, String>` und müssen ggf. geparst werden.
4. Handlername in HTML und `#[html_fn("...")]` muss exakt gleich sein.

