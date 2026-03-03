---
title: DatePicker (Datumswähler)
---

# DatePicker (Datumswähler)

## Überblick

Kalender-Widget, das alleine oder über das Attribut for an ein Input gebunden werden kann.

- Rust-Komponente: DatePicker
- HTML-Tag: date-picker
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Unterstützt for, name, label, placeholder, value, min, max, format.
- Funktioniert allein oder gebunden an ein Input.
- Praktisch mit onchange-Handlern für datumsabhängige UIs.

## HTML-Beispiel

    <input id="birthday" type="date" />
    <date-picker for="birthday" min="1990-01-01" max="2030-12-31" format="dmy" onchange="log_datepicker"></date-picker>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::DatePicker;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/datepicker.html");
        reg.add_and_use("datepicker-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_datepicker")]
    fn log_datepicker(In(event): In<HtmlEvent>, query: Query<&DatePicker>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("DatePicker event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - DatePicker"
  src="https://example.com/bevy-extended-ui/wasm/datepicker"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (date-picker), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
