---
title: Component System
---

> Erst verfügbar ab Version 1.6.0!

# How-to: Component System (`extended-framework`)

Dieser Guide zeigt den empfohlenen Ablauf für das neue Component-System mit `extended-framework`.

## Wann dieses System nutzen?

Nutze `extended-framework`, wenn du:

- einen festen Einstieg über `index.html` möchtest
- UI in wiederverwendbare HTML-Komponenten aufteilen willst
- komponentenspezifische Styles automatisch injizieren willst

Wichtig: In diesem Modus ist das alte `UiRegistry`-System deaktiviert.

## 1) Feature aktivieren

```toml
[dependencies]
bevy_extended_ui = { version = "x.x.x", features = ["extended-framework"] }
bevy_extended_ui_macros = "x.x.x"
```

## 2) Ordnerstruktur

Beispielstruktur:

```text
assets/
  index.html
  components/
    main.component.rs
    main.component.html
    main.component.css
```

## 3) `index.html` als Einstieg

`extended-framework` erwartet eine echte `index.html` als Entry-Datei.

```html
<!DOCTYPE html>
<html lang="de">
  <head>
    <meta charset="utf-8" />
    <title>Framework UI</title>
  </head>
  <body>
    <app-main />
  </body>
</html>
```

## 4) Component-Definition (`*.component.rs`)

Datei: `src/packages/main.component.rs`

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlInit;
use bevy_extended_ui_macros::{html_fn, ui_component};

#[ui_component]
pub struct MainComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const MAIN_COMPONENT: MainComponent = MainComponent {
    template_name: "app-main",
    template_file: "main.component.html",
    styles: &["main.component.css"],
};

#[html_fn("init_main")]
pub fn init_main(In(_): In<HtmlInit>) {
    info!("app-main initialisiert");
}
```

Passende Template-Datei: `assets/components/main.component.html`

```html
<div>
  <h1 onclick="init_main">Hallo aus app-main</h1>
</div>
```

## 5) Komponentendatei in den Build einbinden

Da `*.component.rs` keinen normalen Rust-Modulnamen hat, binde die Datei über `#[path]` ein:

```rust
#[path = "packages/main.component.rs"]
mod main_component_mod;
```

Ohne diese Einbindung werden `#[html_fn]`-Handler aus der Datei nicht kompiliert/registriert.

## 6) App-Konfiguration

```rust
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy_extended_ui::framework::ExtendedFrameworkConfiguration;
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};

#[path = "packages/main.component.rs"]
mod main_component_mod;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "assets".to_string(),
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, configure_ui)
        .run();
}

fn configure_ui(
    mut ui_config: ResMut<ExtendedUiConfiguration>,
    mut fw_config: ResMut<ExtendedFrameworkConfiguration>,
) {
    ui_config.camera = ExtendedCam::Default;
    ui_config.framework_components_path = "components".to_string();

    fw_config.asset_root_fs_path = "assets".to_string();
    fw_config.assets_component_root = "components".to_string();
    fw_config.rust_component_root = "src/packages".to_string();
    fw_config.index_html_file = "index.html".to_string();
}
```

## 7) Kontrakte und Fehlerquellen

1. `index_html_file` muss `index.html` sein.
2. `template_file` muss zum Dateinamen passen:
   - `main.component.rs` -> `main.component.html`
3. Jede Komponente braucht `#[ui_component]`.
4. `template_name` muss eindeutig sein.
5. Referenzierte Styles/Template-Dateien müssen existieren.

## 8) Event-Handler mit `#[html_fn]`

Mit `#[html_fn("name")]` verknüpfst du HTML-Attribut und Rust-System:

- HTML: `onclick="save_settings"`
- Rust: `#[html_fn("save_settings")]`

Beispiel:

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui_macros::html_fn;

#[html_fn("save_settings")]
fn save_settings(In(event): In<HtmlClick>) {
    info!("Klick auf Entity {:?}", event.entity);
}
```

