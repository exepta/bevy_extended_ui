---
title: Badge
---

# Badge

### Überblick

Kleines Benachrichtigungs-Widget, das in der Regel an ein anderes Ziel-Widget gebunden wird.

- Rust-Komponente: Badge
- HTML-Tag: badge
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- Unterstützt `for`, `value` (Alias: `count`), `max`, `anchor`.
- Wenn `value > max`, wird das Label als `+{max}` gerendert (z. B. `+99`).
- Auflösung des Ziel-Widgets:
- `for="id"` bindet an das Widget mit dieser id.
- Ohne `for` wird das nächste Widget-Parent verwendet.
- Wenn kein Ziel gefunden wird, bleibt das Badge verborgen.
- Anchor-Werte: `top right` (Standard), `top left`, `bottom right`, `bottom left`.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### Badge Example
<iframe
  id="badge"
  title="Badge"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<button id="inbox-button">Inbox</button>
<badge for="inbox-button" value="112" max="99" anchor="top right"></badge>
```

#### Rust Example

```rust
fn spawn_badge_widget(mut commands: Commands) {
    commands.spawn((
        Badge::default(),
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
