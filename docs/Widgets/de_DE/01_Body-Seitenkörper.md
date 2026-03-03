---
title: Body (Seitenkörper)
---

# Body (Seitenkörper)

## Überblick

Wurzel-Widget für den HTML-Seitenkörper, das einen kompletten geparsten UI-Baum verankert.

- Rust-Komponente: Body
- HTML-Tag: body
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Wichtige Attribute und Verhalten

- Wurzel-Elternteil für geparste Kinder-Widgets.
- Speichert html_key für den aktiven UI-Eintrag.
- Akzeptiert globale id/class/style/event Attribute.

## HTML-Beispiel

    <body oninit="log_body">
      <div class="screen-root">...</div>
    </body>

## Bevy-Beispiel

    use bevy::prelude::*;
    use bevy_extended_ui::ExtendedUiPlugin;
    use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
    use bevy_extended_ui::io::HtmlAsset;
    use bevy_extended_ui::registry::UiRegistry;
    use bevy_extended_ui::widgets::Body;
    use bevy_extended_ui_macros::html_fn;
    
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(ExtendedUiPlugin)
            .add_systems(Startup, load_ui)
            .run();
    }
    
    fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
        let handle: Handle<HtmlAsset> = asset_server.load("ui/body.html");
        reg.add_and_use("body-demo".to_string(), HtmlSource::from_handle(handle));
    }
    
    #[html_fn("log_body")]
    fn log_body(In(event): In<HtmlEvent>, query: Query<&Body>) {
        if let Ok(widget) = query.get(event.entity) {
            info!("Body event entity={:?} data={:?}", event.entity, widget);
        }
    }

## Bevy-WASM-Vorschau (Platzhalter)

<iframe
  title="Bevy WASM Vorschau - Body"
  src="https://example.com/bevy-extended-ui/wasm/body"
  width="100%"
  height="420"
  loading="lazy"
></iframe>

Ersetze die src-URL durch deinen deployten WASM-Preview-Endpunkt.

## Hinweise

- Schreibe den HTML-Tag exakt (body), damit der Converter korrekt mappt.
- Registriere Handler-Namen mit html_fn exakt wie im HTML-Attribut.
- Verlinke diese Seite später auf einen echten Demo-Build.
