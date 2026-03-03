---
title: RadioButton (Optionsfeld)
---

# RadioButton (Optionsfeld)

## Überblick

Einzelauswahl-Widget, das typischerweise innerhalb einer FieldSet-Gruppe verwendet wird.

- Rust-Komponente: RadioButton
- HTML-Tag: radio
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- value trägt den semantischen Auswahlwert.
- selected markiert den initial aktiven Eintrag.
- Typischerweise in einem fieldset verwaltet.

## HTML-Beispiel

    <radio value="de" selected onchange="log_radiobutton">Deutsch</radio>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::RadioButton;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/radiobutton.html");
        reg.add_and_use("radiobutton-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_radiobutton")]
    fn log_radiobutton(In(event): In<HtmlEvent>, query: Query<&RadioButton>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("RadioButton event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - RadioButton"
  src="https://example.com/bevy-extended-ui/wasm/radiobutton"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (radio), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
