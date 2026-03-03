---
title: InputField (Eingabefeld)
---

# InputField (Eingabefeld)

## Überblick

Textbasiertes Eingabe-Widget für Text-, E-Mail-, Datum-, Zahl- und Passwort-Felder.

- Rust-Komponente: InputField
- HTML-Tag: input
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Unterstützt id, name, type, value, placeholder, icon, maxlength, format.
- Eingabetypen: text/email/date/password/number.
- Validierungsattribute required und validation werden unterstützt.

## HTML-Beispiel

```html
<input id="username" name="username" type="text" maxlength="32" placeholder="Your name" onchange="log_inputfield" />
```

## Bevy-Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::InputField;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/inputfield.html");
    reg.add_and_use("inputfield-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_inputfield")]
fn log_inputfield(In(event): In<HtmlEvent>, query: Query<&InputField>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("InputField event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Beispiel

<iframe
  title="Bevy WASM Vorschau - InputField"
  src="{base.url}/examples/inputfield"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.


## Hinweise

- Schreibe den HTML-Tag exakt (input), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
