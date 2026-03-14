---
title: FieldSet (Feldgruppe)
---

# FieldSet (Feldgruppe)

## Überblick

Gruppen-Widget für auswählbare Kinder wie Radio- und Toggle-Elemente mit Selektionsregeln.

- Rust-Komponente: FieldSet
- HTML-Tag: fieldset
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- mode unterstützt single, multi, count.
- allow-none steuert leere Auswahlzustände.
- Gruppiert radio/toggle Kinder mit gemeinsamem Selektionszustand.

## HTML-Beispiel

```html
<fieldset mode="single" allow-none="false" onchange="log_fieldset">
  <radio value="easy">Easy</radio>
  <radio value="hard" selected>Hard</radio>
</fieldset>
```

## Bevy-Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::FieldSet;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/fieldset.html");
    reg.add_and_use("fieldset-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_fieldset")]
fn log_fieldset(In(event): In<HtmlEvent>, query: Query<&FieldSet>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("FieldSet event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Beispiel

<iframe
  title="Bevy WASM Vorschau - FieldSet"
  src="{base.url}/examples/fieldset"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (fieldset), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
