---
title: Scrollbar (Bildlaufleiste)
---

# Scrollbar (Bildlaufleiste)

## Überblick

Scroll-Helfer-Widget aus dem scroll-Tag für vertikales oder horizontales Scrollen.

- Rust-Komponente: Scrollbar
- HTML-Tag: scroll
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Tag ist scroll, nicht scrollbar.
- alignment schaltet vertikal/horizontal.
- Nützlich für benutzerdefinierte Scroll-Bereiche.

## HTML-Beispiel

    <scroll alignment="vertical" onscroll="log_scrollbar"></scroll>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::Scrollbar;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/scrollbar.html");
        reg.add_and_use("scrollbar-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_scrollbar")]
    fn log_scrollbar(In(event): In<HtmlEvent>, query: Query<&Scrollbar>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("Scrollbar event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - Scrollbar"
  src="https://example.com/bevy-extended-ui/wasm/scrollbar"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (scroll), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
