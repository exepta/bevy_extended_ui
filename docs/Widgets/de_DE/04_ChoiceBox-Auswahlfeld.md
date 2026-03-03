---
title: ChoiceBox (Auswahlfeld)
---

# ChoiceBox (Auswahlfeld)

## Überblick

Dropdown-Auswahl-Widget auf Basis von select/option mit genau einem aktiven Wert.

- Rust-Komponente: ChoiceBox
- HTML-Tag: select
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Nutzt select mit verschachtelten option-Einträgen.
- selected auf option setzt den initialen Wert.
- Optionales icon pro Option möglich.

## HTML-Beispiel

    <select onchange="log_choicebox" id="quality">
      <option value="low">Low</option>
      <option selected value="high">High</option>
    </select>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::ChoiceBox;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/choicebox.html");
        reg.add_and_use("choicebox-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_choicebox")]
    fn log_choicebox(In(event): In<HtmlEvent>, query: Query<&ChoiceBox>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("ChoiceBox event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - ChoiceBox"
  src="https://example.com/bevy-extended-ui/wasm/choicebox"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (select), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
