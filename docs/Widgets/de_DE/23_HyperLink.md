---
title: HyperLink
---

# HyperLink
## Überblick

Klickbares Text-Link-Widget, das auf den HTML-Tag `<a>` gemappt wird.

- Rust-Komponente: HyperLink
- HTML-Tag: a
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

- `href` definiert die Ziel-URL.
- `browsers` ist optional (Standard: `system`).
- Unterstützte Werte für `browsers`: `system`, einzelner Browser (`firefox`, `chrome`, `brave`, ...) oder Liste (`[firefox, brave, chrome]`).
- `open-modal` ist optional (Standard: `false`).
- Wenn `open-modal="true"` gesetzt ist und der konfigurierte Browser nicht installiert ist, zeigt die Bevy-App ein Modal an und fragt, ob der Installationsbefehl im Terminal geöffnet werden soll.
- Native Browser-Erkennung ist für Linux, macOS und Windows umgesetzt.
- Unter `wasm32` werden `browsers` und `open-modal` ignoriert.
- Es gibt keinen System-Browser-Fallback, wenn explizite `browsers` konfiguriert sind und keiner installiert ist.

## Html Beispiel

```html
<a href="https://bevy.org">Mit System-Browser öffnen</a>
<a href="https://bevy.org" browsers="[chrome]" open-modal="true">Mit konfiguriertem Browser öffnen</a>
```

## Rust Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::HyperLink;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .add_systems(Update, update_link_target)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("examples/widgets_overview.html");
    reg.add_and_use("hyperlink-demo".to_string(), HtmlSource::from_handle(handle));
}

fn update_link_target(mut query: Query<&mut HyperLink>) {
    for mut link in &mut query {
        if link.href.is_empty() {
            link.href = "https://bevyengine.org".to_string();
        }
    }
}
```

## WASM Vorschau

<iframe
  id="hyperlink"
  title="Bevy WASM Vorschau - HyperLink"
  src="{base.url}/examples/widgets-overview"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (`a`), damit der Converter korrekt mappt.
- Nutze `open-modal="true"` nur dann, wenn du bei fehlenden konfigurierten Browsern den Install-Dialogfluss möchtest.
- Style Links über den CSS-Selektor `a`.

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
