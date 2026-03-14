---
title: Button (Schaltfläche)
---

# Button (Schaltfläche)

## Überblick

Klickbares Aktions-Widget mit optionalem Icon und optionalem Typ-Verhalten für Formulare.

- Rust-Komponente: Button
- HTML-Tag: button
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- type=button|submit|reset steuert Formularverhalten.
- Unterstützt Icon-Kinderknoten.
- Typische Events: onclick, onfocus, onmouseover, onmouseout, oninit.

## HTML-Beispiel

```html
<button onclick="log_button" type="button">
  Save
  <icon src="extended_ui/icons/check-mark.png"></icon>
</button>
```

## Bevy-Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Button;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/button.html");
    reg.add_and_use("button-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_button")]
fn log_button(In(event): In<HtmlEvent>, query: Query<&Button>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Button event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Beispiel

<iframe
  title="Bevy WASM Vorschau - Button"
  src="{base.url}/examples/button"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (button), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
