---
title: ToolTip
---

# ToolTip
## Überblick

Kontext-Hinweis-Widget, das dem Cursor folgt oder auf ein Ziel zeigt.

- Rust-Komponente: ToolTip
- HTML-Tag: tool-tip
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

- Unterstützt for, variant, prio, alignment, trigger.
- Modi: follow (Cursor) und point (verankert).
- Bindung implizit über Parent oder explizit per for=id.

## Html Beispiel

```html
<button id="help">?</button>
<tool-tip for="help" variant="point" prio="right" alignment="horizontal" trigger="hover | click" oninit="log_tooltip">
  More information
</tool-tip>
```

## Rust Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::ToolTip;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/tooltip.html");
    reg.add_and_use("tooltip-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_tooltip")]
fn log_tooltip(In(event): In<HtmlEvent>, query: Query<&ToolTip>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("ToolTip event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## WASM Vorschau

<iframe
  id="tooltip"
  title="Bevy WASM Vorschau - ToolTip"
  src="{base.url}/examples/tooltip"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (tool-tip), damit der Converter korrekt mappt.
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
