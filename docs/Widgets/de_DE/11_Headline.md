---
title: Headline
---

# Headline

### Überblick

Überschriften-Widget aus h1 bis h6 für Kapitelstruktur und semantische Titel.

- Rust-Komponente: Headline
- HTML-Tag: h1-h6
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- h1 bis h6 werden auf eine Headline-Komponente gemappt.
- Speichert Text und Überschriftenstufe.
- Kann dynamisch per System umgestylt werden.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### Headline
<iframe
  id="headline"
  title="Headline"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<h1>Ueberschrift</h1>
<h2>Ueberschrift</h2>
<h3>Ueberschrift</h3>
<h4>Ueberschrift</h4>
<h5>Ueberschrift</h5>
<h6>Ueberschrift</h6>
```

#### Rust Example

```rust
fn spawn_headline_widget(mut commands: Commands) {
    commands.spawn((
        Headline {
            text: "Ueberschrift".to_string(),
            h_type: HeadlineType::H1,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        Headline {
            text: "Ueberschrift".to_string(),
            h_type: HeadlineType::H2,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        Headline {
            text: "Ueberschrift".to_string(),
            h_type: HeadlineType::H3,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        Headline {
            text: "Ueberschrift".to_string(),
            h_type: HeadlineType::H4,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        Headline {
            text: "Ueberschrift".to_string(),
            h_type: HeadlineType::H5,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        Headline {
            text: "Ueberschrift".to_string(),
            h_type: HeadlineType::H6,
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
