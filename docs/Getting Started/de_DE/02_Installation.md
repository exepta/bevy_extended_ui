---
title: Installation
---

# Bevy Extended UI installieren

## Voraussetzungen

- Rust-Toolchain (`stable` ist hier der Standard)
- Ein Bevy-Projekt
- Ein Asset-Ordner für HTML/CSS (üblich: `assets/`)

## 1. Abhängigkeiten hinzufügen

Füge beide Crates per Cargo hinzu:

```bash
cargo add bevy_extended_ui
cargo add bevy_extended_ui_macros
```

Alternativ kannst du die Einträge manuell in `Cargo.toml` setzen und passende Versionen von crates.io verwenden.

## 2. Plugin registrieren

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .run();
}
```

## 3. Erste HTML-Oberfläche laden

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;

fn lade_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/main.html");
    reg.add_and_use("haupt-ui".to_string(), HtmlSource::from_handle(handle));
}
```

System danach z. B. beim Start einhängen:

```rust
app.add_systems(Startup, lade_ui);
```

## 4. Optionale Feature-Flags

Nützliche Features:

- `css-breakpoints`: Breakpoints für Desktop-Fenster
- `wasm-breakpoints`: Breakpoints für Browser-Viewport (WASM)
- `wasm-default`: Vorkonfiguration für typische WASM-Setups
- `fluent`: Fluent-Sprachbackend
- `properties-lang`: Java-Properties-Sprachbackend
- `clipboard-wasm`: Clipboard-Unterstützung im Browser

Beispiel:

```toml
[dependencies]
bevy_extended_ui = { version = "x.y.z", features = ["wasm-default", "fluent"] }
bevy_extended_ui_macros = "x.y.z"
```

## 5. Installation prüfen

- HTML/CSS-Dateien unter `assets/` ablegen
- Anwendung starten
- Sichtbarkeit der UI und Event-Reaktionen überprüfen

Für den fachlichen Einstieg siehe [Überblick](01_Überblick.md).
