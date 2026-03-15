---
title: InputField
---

# InputField
## Überblick

Textbasiertes Eingabe-Widget für Text-, E-Mail-, Datum-, Zahl-, Passwort- und Datei-Felder.

- Rust-Komponente: InputField
- HTML-Tag: input
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

- Unterstützt id, name, type, value, placeholder, icon, maxlength, format, folder, extensions, show-size, max-size.
- Eingabetypen: text/email/date/password/number/file.
- Hinweise für `type="file"`:
  - `folder="true|false"` (Standard `false`)
  - `extensions="json"` oder `extensions="[json, css, yaml, png]"` (wird bei `folder="true"` ignoriert)
  - `show-size="true|false"` (Standard `false`)
  - `max-size="1KB|1MB|1GB"` weist Dateien zurück, die größer als das Limit sind
- Validierungsattribute required und validation werden unterstützt.

## Html Beispiel

```html
<input id="username" name="username" type="text" maxlength="32" placeholder="Your name" onchange="log_inputfield" />
```

## Rust Beispiel

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

## WASM Vorschau

<iframe
  id="inputfield"
  title="Bevy WASM Vorschau - InputField"
  src="{base.url}/examples/inputfield"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (input), damit der Converter korrekt mappt.
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
