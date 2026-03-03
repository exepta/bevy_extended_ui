---
title: Slider (Schieberegler)
---

# Slider (Schieberegler)

## Überblick

Numerisches Zieh-Widget mit Min-, Max-, Wert- und Schritt-Eigenschaften.

- Rust-Komponente: Slider
- HTML-Tag: slider
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- min, max, value, step steuern die Zahleninteraktion.
- Funktioniert mit onchange/oninput-Handlern.
- Aktueller Wert liegt in der Slider-Komponente.

## HTML-Beispiel

```html
<slider min="0" max="100" value="25" step="1" onchange="log_slider"></slider>
```

## Bevy-Beispiel

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

## Beispiel

<iframe
  title="Bevy WASM Vorschau - Slider"
  src="{base.url}/examples/slider"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (slider), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
