---
title: Div
---

# Div
## Überblick

Generischer Layout-Container zum Gruppieren und Strukturieren verschachtelter Widgets.

- Rust-Komponente: Div
- HTML-Tag: div
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

- Generischer Gruppen-Container.
- Ideal für CSS-Klassen und Layout-Komposition.
- Unterstützt verschachtelte Widgets und Event-Attribute.

## Html Beispiel

```html
<div id="card" class="panel" oninit="log_div">
  <p>Panel content</p>
</div>
```

## Rust Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Div;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/div.html");
    reg.add_and_use("div-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_div")]
fn log_div(In(event): In<HtmlEvent>, query: Query<&Div>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Div event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Vorschau

<iframe
  id="div"
  title="Bevy WASM Vorschau - Div"
  src="{base.url}/examples/div"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (div), damit der Converter korrekt mappt.
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
