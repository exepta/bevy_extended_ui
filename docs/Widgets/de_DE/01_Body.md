---
title: Body
---

# Body

## Überblick

`Body` ist das zentrale Wurzel-Widget der UI-Struktur und repräsentiert den HTML-Tag `<body>` innerhalb des Parsers. Alle untergeordneten Widgets werden unter diesem Knoten aufgebaut, wodurch `Body` die Grundlage für Layout, Scroll-Verhalten, Event-Weiterleitung und die Verknüpfung zur geladenen UI-Quelle (`html_key`) bildet.

- Rust-Komponente: Body
- HTML-Tag: body
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

Keine extra attributes!

Unterstützte globale HTML-Attribute (ausführlich):

- `id`: Vergibt eine eindeutige Element-ID und ermöglicht gezieltes Ansprechen über Selektoren, Skripte und Widget-Bindings.
- `class`: Übergibt mehrere CSS-Klassen für Zustands- und Theme-Styling; die Klassen werden als Liste in die Bevy-Style-Pipeline übernommen.
- `style`: Übergibt Inline-CSS; die Werte werden in `HtmlStyle` geparsed und mit anderen Styles zusammengeführt.
- `hidden`: Markiert das Widget initial als versteckt, damit es nicht sichtbar gerendert wird.
- `disabled`: Setzt den interaktiven Zustand auf deaktiviert und blockiert entsprechende Nutzerinteraktionen.
- `readonly`: Markiert das Widget als nur lesbar; relevant für konsistentes Zustands-Handling in der Widget-State-Logik.
- Event-Attribute wie `onclick`, `oninit`, `onmouseover`, `onmouseout`, `onfocus`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Binden Handler-Namen direkt an das Event-System und werden als `HtmlEventBindings` registriert.

## WASM Vorschau

<iframe
  id="body"
  title="Bevy WASM Vorschau - Body"
  src="{base.url}/examples/body"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Html Beispiel

```html
<body id="main-body" class="screen-root app-layout" oninit="on_body_init">
  <div class="content">...</div>
</body>
```

## Rust Beispiel

```rust
fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/body.html");
    reg.add_and_use("body-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("on_body_init")]
fn on_body_init(In(event): In<HtmlInit>, query: Query<&Body>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Body geladen: entry={} html_key={:?}", widget.entry, widget.html_key);
    }
}
```

## Ersteller vom Widget

<div style="display: flex; align-items: center; justify-content: flex-start; padding: 15px; border: 1px solid #5658db; border-radius: 10px; gap: 15px; width: 300px;">
  <img
    src="https://avatars.githubusercontent.com/u/84874606?v=4"
    alt="exepta avatar"
    width="64"
    height="64"
    style="width: 64px; height: 64px; border-radius: 50%; object-fit: cover;"
  />
  <div style="display: flex; flex-direction: column; align-items: flex-start; justify-content: center;">
    <strong>exepta</strong>
    <a href="https://github.com/exepta" style="margin-top: 10px; color: #5658db;">Link to GitHub</a>
  </div>
</div>
