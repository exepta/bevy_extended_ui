---
title: Headline (Überschrift)
---

# Headline (Überschrift)

## Überblick

Überschriften-Widget aus h1 bis h6 für Kapitelstruktur und semantische Titel.

- Rust-Komponente: Headline
- HTML-Tag: h1-h6
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- h1 bis h6 werden auf eine Headline-Komponente gemappt.
- Speichert Text und Überschriftenstufe.
- Kann dynamisch per System umgestylt werden.

## HTML-Beispiel

```html
<h2 oninit="log_headline">Settings</h2>
```

## Bevy-Beispiel

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

## Beispiel

<iframe
  title="Bevy WASM Vorschau - Headline"
  src="{base.url}/examples/headline"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (h1-h6), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
