---
title: CheckBox
---

# CheckBox
## Überblick

Boolesches Eingabe-Widget mit Label und optionalem Icon für Ja/Nein-Zustände.

- Rust-Komponente: CheckBox
- HTML-Tag: checkbox
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

- Haupt-Tag ist checkbox mit Label-Text.
- Optionales icon-Attribut für das Häkchen.
- Checked-Zustand ist über Laufzeit-State auslesbar.

## Html Beispiel

```html
<checkbox icon="extended_ui/icons/check-mark.png" onclick="log_checkbox">
  Enable music
</checkbox>
```

## Rust Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::CheckBox;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/checkbox.html");
    reg.add_and_use("checkbox-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_checkbox")]
fn log_checkbox(In(event): In<HtmlEvent>, query: Query<&CheckBox>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("CheckBox event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Vorschau

<iframe
  id="checkbox"
  title="Bevy WASM Vorschau - CheckBox"
  src="{base.url}/examples/checkbox"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (checkbox), damit der Converter korrekt mappt.
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
