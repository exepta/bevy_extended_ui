---
title: ColorPicker (Farbwähler)
---

# ColorPicker (Farbwähler)

## Überblick

Interaktives Farb-Widget für RGB/RGBA/HEX-Workflows inklusive Änderungs-Events.

- Rust-Komponente: ColorPicker
- HTML-Tag: colorpicker
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- value definiert die Startfarbe; alpha ist konfigurierbar.
- Change-Events erlauben Live-Updates in der UI.
- Komponente liefert rgb/hsv/hex Hilfswerte.

## HTML-Beispiel

    <colorpicker value="#4285f4" alpha="255" onchange="log_colorpicker"></colorpicker>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::ColorPicker;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/colorpicker.html");
        reg.add_and_use("colorpicker-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_colorpicker")]
    fn log_colorpicker(In(event): In<HtmlEvent>, query: Query<&ColorPicker>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("ColorPicker event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - ColorPicker"
  src="https://example.com/bevy-extended-ui/wasm/colorpicker"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (colorpicker), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
