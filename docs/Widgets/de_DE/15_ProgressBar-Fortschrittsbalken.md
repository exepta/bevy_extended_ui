---
title: ProgressBar (Fortschrittsbalken)
---

# ProgressBar (Fortschrittsbalken)

## Überblick

Bereichsbasiertes Anzeige-Widget für Fortschritt zwischen Min- und Max-Wert.

- Rust-Komponente: ProgressBar
- HTML-Tag: progressbar
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Bereichswerte über min, max, value.
- Wird idealerweise über App-/Spielzustand aktualisiert.
- Gut für XP, Laden, Leben und Cooldowns.

## HTML-Beispiel

```html
<progressbar id="xp" min="0" max="100" value="42" oninit="log_progressbar"></progressbar>
```

## Bevy-Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::ProgressBar;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/progressbar.html");
    reg.add_and_use("progressbar-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_progressbar")]
fn log_progressbar(In(event): In<HtmlEvent>, query: Query<&ProgressBar>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("ProgressBar event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Beispiel

<iframe
  title="Bevy WASM Vorschau - ProgressBar"
  src="{base.url}/examples/progressbar"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.


## Hinweise

- Schreibe den HTML-Tag exakt (progressbar), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
