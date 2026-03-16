---
title: Scrollbar
---

# Scrollbar

### Überblick

Scroll-Helfer-Widget aus dem scroll-Tag für vertikales oder horizontales Scrollen.

- Rust-Komponente: Scrollbar
- HTML-Tag: scroll
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- Tag ist scroll, nicht scrollbar.
- alignment schaltet vertikal/horizontal.
- Nützlich für benutzerdefinierte Scroll-Bereiche.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### Scrollbar Vertical
<iframe
  id="scrollbar-vertical"
  title="Scrollbar"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<div style="width: 200px; height: 200px; overflow-y: scroll; display: flex; flex-direction: column; gap: 10px;">
  <p>Line 1</p>
  <p>Line 2</p>
  <p>Line 3</p>
  <p>Line 4</p>
  <p>Line 5</p>
  <p>Line 6</p>
  <p>Line 7</p>
  <p>Line 8</p>
  <p>Line 9</p>
  <p>Line 10</p>
</div>
```

#### Rust Example

```rust
fn spawn_scrollbar_widget(mut commands: Commands) {
    commands.spawn((
        Scrollbar {
            vertical: true,
            min: 0.0,
            max: 1000.0,
            value: 0.0,
            step: 10.0,
            ..default()
        },
        Node::default(),
    ));
}
```

### Scrollbar Horizontal
<iframe
id="scrollbar-horizontal"
title="Scrollbar"
src="{base.url}/examples/base"
width="100%"
height="420"
loading="lazy">
</iframe>

#### Html Example

```html
<div style="width: 200px; height: 100px; overflow-y: scroll; display: flex; flex-direction: row; gap: 10px;">
  <p>Line 1</p>
  <p>Line 2</p>
  <p>Line 3</p>
  <p>Line 4</p>
  <p>Line 5</p>
  <p>Line 6</p>
  <p>Line 7</p>
  <p>Line 8</p>
</div>
```

#### Rust Example

```rust
fn spawn_scrollbar_widget(mut commands: Commands) {
    commands.spawn((
        Scrollbar {
            vertical: false,
            min: 0.0,
            max: 1000.0,
            value: 0.0,
            step: 10.0,
            ..default()
        },
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
