---
title: ToolTip (Kurzinfo)
---

# ToolTip (Kurzinfo)

## Überblick

Kontext-Hinweis-Widget, das dem Cursor folgt oder auf ein Ziel zeigt.

- Rust-Komponente: ToolTip
- HTML-Tag: tool-tip
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Unterstützt for, variant, prio, alignment, trigger.
- Modi: follow (Cursor) und point (verankert).
- Bindung implizit über Parent oder explizit per for=id.

## HTML-Beispiel

```html
<button id="help">?</button>
<tool-tip for="help" variant="point" prio="right" alignment="horizontal" trigger="hover | click" oninit="log_tooltip">
  More information
</tool-tip>
```

## Bevy-Beispiel

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

## Beispiel

<iframe
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
