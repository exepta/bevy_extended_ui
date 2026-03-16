---
title: RadioButton
---

# RadioButton

### Überblick

Einzelauswahl-Widget, das typischerweise innerhalb einer FieldSet-Gruppe verwendet wird.

- Rust-Komponente: RadioButton
- HTML-Tag: radio
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- value trägt den semantischen Auswahlwert.
- selected markiert den initial aktiven Eintrag.
- Typischerweise in einem fieldset verwaltet.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### RadioButton
<iframe
  id="radiobutton"
  title="RadioButton"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<fieldset mode="single" allow-none="false">
  <radio value="black">Black</radio>
  <radio value="green">Green</radio>
  <radio value="white">White</radio>
</fieldset>
```

#### Rust Example

```rust
fn spawn_radiobutton_widget(mut commands: Commands) {
    commands
        .spawn((
            FieldSet {
                field_mode: FieldMode::Single,
                allow_none: false,
                ..default()
            },
            Node::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                RadioButton {
                    label: "Black".to_string(),
                    value: "black".to_string(),
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                RadioButton {
                    label: "Green".to_string(),
                    value: "green".to_string(),
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                RadioButton {
                    label: "White".to_string(),
                    value: "white".to_string(),
                    ..default()
                },
                Node::default(),
            ));
        });
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
