---
title: Headline
---

# Headline
## Überblick

Überschriften-Widget aus h1 bis h6 für Kapitelstruktur und semantische Titel.

- Rust-Komponente: Headline
- HTML-Tag: h1-h6
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

- h1 bis h6 werden auf eine Headline-Komponente gemappt.
- Speichert Text und Überschriftenstufe.
- Kann dynamisch per System umgestylt werden.

## Html Beispiel

```html
<h2 oninit="log_headline">Settings</h2>
```

## Rust Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Headline;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/headline.html");
    reg.add_and_use("headline-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_headline")]
fn log_headline(In(event): In<HtmlEvent>, query: Query<&Headline>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Headline event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Vorschau

<iframe
  id="headline"
  title="Bevy WASM Vorschau - Headline"
  src="{base.url}/examples/headline"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (h1-h6), damit der Converter korrekt mappt.
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
