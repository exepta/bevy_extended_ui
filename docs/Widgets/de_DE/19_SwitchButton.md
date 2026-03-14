---
title: SwitchButton (Schalter)
---

# SwitchButton (Schalter)

## Überblick

Schalterartiges Binär-Widget mit Textlabel und optionalem Icon.

- Rust-Komponente: SwitchButton
- HTML-Tag: switch
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Label-Text kommt aus dem InnerContent.
- Optionales icon-Attribut wird unterstützt.
- Checked-ähnlicher Zustand über UIWidgetState lesbar.

## HTML-Beispiel

```html
<switch icon="extended_ui/icons/drop-arrow.png" onclick="log_switchbutton">Dark mode</switch>
```

## Bevy-Beispiel

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::SwitchButton;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/switchbutton.html");
    reg.add_and_use("switchbutton-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_switchbutton")]
fn log_switchbutton(In(event): In<HtmlEvent>, query: Query<&SwitchButton>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("SwitchButton event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Beispiel

<iframe
  title="Bevy WASM Vorschau - SwitchButton"
  src="{base.url}/examples/switchbutton"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Hinweise

- Schreibe den HTML-Tag exakt (switch), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
