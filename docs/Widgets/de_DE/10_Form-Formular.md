---
title: Form (Formular)
---

# Form (Formular)

## Überblick

Formular-Container, der Kinder validiert und Submit-Aktionen mit gesammelten Daten auslöst.

- Rust-Komponente: Form
- HTML-Tag: form
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- action-Handler erhält gesammelte Submit-Daten.
- validate-Modus: Always, Send, Interact.
- Submit-Button triggert Validierung und HtmlSubmit-Event.

## HTML-Beispiel

```html
<form action="log_form" validate="Send">
  <input name="email" type="email" required />
  <button type="submit">Send</button>
</form>
```

## Bevy-Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Form;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/form.html");
    reg.add_and_use("form-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_form")]
fn log_form(In(event): In<HtmlEvent>, query: Query<&Form>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Form event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Beispiel

<iframe
  title="Bevy WASM Vorschau - Form"
  src="{base.url}/examples/form"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.


## Hinweise

- Schreibe den HTML-Tag exakt (form), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
