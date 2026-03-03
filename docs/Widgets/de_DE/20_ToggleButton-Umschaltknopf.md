---
title: ToggleButton (Umschaltknopf)
---

# ToggleButton (Umschaltknopf)

## Überblick

Auswählbares Button-Widget für Einzel- oder Mehrfachauswahl, oft im FieldSet.

- Rust-Komponente: ToggleButton
- HTML-Tag: toggle
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Unterstützt value und selected Attribute.
- Optionales Icon-Kind für Toolbar-artige UIs.
- Häufig in fieldset mode=multi eingesetzt.

## HTML-Beispiel

```html
<toggle value="bold" selected onclick="log_togglebutton">
  <icon src="extended_ui/icons/bold.png"></icon>
</toggle>
```

## Bevy-Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::ToggleButton;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/togglebutton.html");
    reg.add_and_use("togglebutton-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_togglebutton")]
fn log_togglebutton(In(event): In<HtmlEvent>, query: Query<&ToggleButton>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("ToggleButton event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Beispiel

<iframe
  title="Bevy WASM Vorschau - ToggleButton"
  src="{base.url}/examples/togglebutton"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.


## Hinweise

- Schreibe den HTML-Tag exakt (toggle), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
