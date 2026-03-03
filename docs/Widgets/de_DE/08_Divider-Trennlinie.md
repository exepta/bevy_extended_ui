---
title: Divider (Trennlinie)
---

# Divider (Trennlinie)

## Überblick

Visuelles Trenn-Widget mit horizontaler oder vertikaler Ausrichtung.

- Rust-Komponente: Divider
- HTML-Tag: divider
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- alignment akzeptiert horizontal oder vertical.
- Kurze Aliase wie h und v werden geparst.
- Gedacht als reine visuelle Trennung.

## HTML-Beispiel

    <divider alignment="horizontal" oninit="log_divider"></divider>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::Divider;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/divider.html");
        reg.add_and_use("divider-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_divider")]
    fn log_divider(In(event): In<HtmlEvent>, query: Query<&Divider>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("Divider event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - Divider"
  src="https://example.com/bevy-extended-ui/wasm/divider"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (divider), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
