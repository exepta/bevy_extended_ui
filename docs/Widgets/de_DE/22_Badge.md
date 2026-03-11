---
title: Badge (Benachrichtigung)
---

# Badge (Benachrichtigung)

## Überblick

Kleines Benachrichtigungs-Widget, das in der Regel an ein anderes Ziel-Widget gebunden wird.

- Rust-Komponente: Badge
- HTML-Tag: badge
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Unterstützt `for`, `value` (Alias: `count`), `max`, `anchor`.
- Wenn `value > max`, wird das Label als `+{max}` gerendert (z. B. `+99`).
- Auflösung des Ziel-Widgets:
- `for="id"` bindet an das Widget mit dieser id.
- Ohne `for` wird das nächste Widget-Parent verwendet.
- Wenn kein Ziel gefunden wird, bleibt das Badge verborgen.
- Anchor-Werte: `top right` (Standard), `top left`, `bottom right`, `bottom left`.

## HTML-Beispiel

```html
<button id="inbox-button">Inbox</button>
<badge for="inbox-button" value="112" max="99" anchor="top right"></badge>
```

## Bevy-Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Badge;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .add_systems(Update, tick_badges)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/widgets_overview.html");
    reg.add_and_use("badge-demo".to_string(), HtmlSource::from_handle(handle));
}

fn tick_badges(mut query: Query<&mut Badge>) {
    for mut badge in &mut query {
        badge.value = (badge.value + 1) % 130;
    }
}
```

## Beispiel

<iframe
  title="Bevy WASM Vorschau - Badge"
  src="{base.url}/examples/badge"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (`badge`), damit der Converter korrekt mappt.
- Nutze `for`, wenn das Badge außerhalb einer Parent-Child-Struktur gebunden werden soll.
- Für das Styling verwende die CSS-Selektoren `badge` und `.badge-text`.
