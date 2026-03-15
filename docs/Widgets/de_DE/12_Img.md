---
title: Img
---

# Img
## Überblick

Bild-Widget für eine Asset-Quelle inklusive Alternativtext.

- Rust-Komponente: Img
- HTML-Tag: img
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

- src wird relativ zum HTML-Asset-Pfad aufgelöst.
- alt speichert Fallback-/Metadaten-Text.
- preview kann die Bildquelle an die id eines `input type="file"` binden (z. B. `preview="avatar-file"`).
- Leichtgewichtiges Bild-Widget für Inhalte.

## Html Beispiel

```html
<img src="ui/logo.png" alt="Project logo" oninit="log_img" />
```

```html
<input id="avatar-file" type="file" extensions="[jpg, jpeg, png]" show-size="true" />
<img preview="avatar-file" alt="Avatar Vorschau" />
```

## Rust Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Img;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/img.html");
    reg.add_and_use("img-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_img")]
fn log_img(In(event): In<HtmlEvent>, query: Query<&Img>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Img event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Vorschau

<iframe
  id="img"
  title="Bevy WASM Vorschau - Img"
  src="{base.url}/examples/img"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (img), damit der Converter korrekt mappt.
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
