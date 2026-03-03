---
title: CheckBox (Kontrollkästchen)
---

# CheckBox (Kontrollkästchen)

## Überblick

Boolesches Eingabe-Widget mit Label und optionalem Icon für Ja/Nein-Zustände.

- Rust-Komponente: CheckBox
- HTML-Tag: checkbox
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Haupt-Tag ist checkbox mit Label-Text.
- Optionales icon-Attribut für das Häkchen.
- Checked-Zustand ist über Laufzeit-State auslesbar.

## HTML-Beispiel

    <checkbox icon="extended_ui/icons/check-mark.png" onclick="log_checkbox">
      Enable music
    </checkbox>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::CheckBox;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/checkbox.html");
        reg.add_and_use("checkbox-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_checkbox")]
    fn log_checkbox(In(event): In<HtmlEvent>, query: Query<&CheckBox>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("CheckBox event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - CheckBox"
  src="https://example.com/bevy-extended-ui/wasm/checkbox"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (checkbox), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
