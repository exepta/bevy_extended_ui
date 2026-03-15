---
title: Slider
---

# Slider
## Überblick

Numerisches Zieh-Widget mit Min-, Max-, Wert- und Schritt-Eigenschaften.
Unterstützt `default` (ein Thumb) und `range` (zwei Thumbs).

- Rust-Komponente: Slider
- HTML-Tag: slider
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

- `min`, `max`, `step`: Zahlenbereich und Schrittweite.
- `type`: `default | range` (Standard: `default`).
- `value`:
  - `default`: einzelner Zahlenwert (z. B. `35`)
  - `range`: Bereich im Format `start - end` (z. B. `20 - 40`)
- `dots`: Anzahl der Segmente/Ticks. Beispiel `dots="5"` bei `0..100` erzeugt Ticks bei `0, 20, 40, 60, 80, 100`.
  - Wenn `dots` gesetzt ist und `<= 1` (auch `0` oder negativ), werden nur Min/Max-Ticks genutzt (wie `dots="1"`).
  - Interner Schutz: Mindestabstand zwischen Nachbar-Ticks ist 10 Werteinheiten.
- `show-labels`: `true | false` (Standard: `false`). Zeigt Tick-Labels.
- `dot-anchor`: `top | bottom` (Standard: `top`). Position der Tick-Labels relativ zum Track.
- `tip`: `true | false` (Standard: `true`). Zeigt/verbirgt Thumb-Tooltip.
- Track reagiert auf Klick und Drag.
- Im `range`-Modus zeigt der Tooltip den Wert des jeweils gehoverten Thumbs.
- Funktioniert mit onchange/oninput-Handlern.
- Aktueller Wert liegt in der Slider-Komponente.

## Html Beispiel

```html
<slider min="0" max="100" value="25" step="1" onchange="log_slider"></slider>
```

```html
<slider
  type="range"
  min="0"
  max="100"
  value="20 - 40"
  step="1"
  dots="5"
  show-labels="true"
  dot-anchor="top"
  tip="true">
</slider>
```

## Rust Beispiel

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

## WASM Vorschau

<iframe
  id="slider"
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

## Ersteller vom Widget

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
