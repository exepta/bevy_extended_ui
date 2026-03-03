---
title: Div (Container)
---

# Div (Container)

## Überblick

Generischer Layout-Container zum Gruppieren und Strukturieren verschachtelter Widgets.

- Rust-Komponente: Div
- HTML-Tag: div
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Generischer Gruppen-Container.
- Ideal für CSS-Klassen und Layout-Komposition.
- Unterstützt verschachtelte Widgets und Event-Attribute.

## HTML-Beispiel

    <div id="card" class="panel" oninit="log_div">
      <p>Panel content</p>
    </div>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::Div;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/div.html");
        reg.add_and_use("div-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_div")]
    fn log_div(In(event): In<HtmlEvent>, query: Query<&Div>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("Div event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - Div"
  src="https://example.com/bevy-extended-ui/wasm/div"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (div), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
