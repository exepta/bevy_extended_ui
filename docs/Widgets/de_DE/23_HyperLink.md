---
title: HyperLink
---

# HyperLink

### Überblick

Klickbares Text-Link-Widget, das auf den HTML-Tag `<a>` gemappt wird.

- Rust-Komponente: HyperLink
- HTML-Tag: a
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- `href` definiert die Ziel-URL.
- `browsers` ist optional (Standard: `system`).
- Unterstützte Werte für `browsers`: `system`, einzelner Browser (`firefox`, `chrome`, `brave`, ...) oder Liste (`[firefox, brave, chrome]`).
- `open-modal` ist optional (Standard: `false`).
- Wenn `open-modal="true"` gesetzt ist und der konfigurierte Browser nicht installiert ist, zeigt die Bevy-App ein Modal an und fragt, ob der Installationsbefehl im Terminal geöffnet werden soll.
- Native Browser-Erkennung ist für Linux, macOS und Windows umgesetzt.
- Unter `wasm32` werden `browsers` und `open-modal` ignoriert.
- Es gibt keinen System-Browser-Fallback, wenn explizite `browsers` konfiguriert sind und keiner installiert ist.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### HyperLink Example
<iframe
  id="hyperlink"
  title="HyperLink"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<a href="https://bevy.org">Mit System-Browser öffnen</a>
<a href="https://bevy.org" browsers="[chrome]" open-modal="true">Mit konfiguriertem Browser öffnen</a>
```

#### Rust Example

```rust
fn spawn_hyperlink_widget(mut commands: Commands) {
    commands.spawn((
        HyperLink::default(),
        Node::default(),
    ));
}
```

### Ersteller vom Widget

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
