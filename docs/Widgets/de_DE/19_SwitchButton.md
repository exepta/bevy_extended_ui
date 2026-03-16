---
title: SwitchButton
---

# SwitchButton

### Überblick

Schalterartiges Binär-Widget mit Textlabel und optionalem Icon.

- Rust-Komponente: SwitchButton
- HTML-Tag: switch
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- Label-Text kommt aus dem InnerContent.
- Optionales icon-Attribut wird unterstützt.
- Checked-ähnlicher Zustand über UIWidgetState lesbar.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### SwitchButton
<iframe
  id="switchbutton"
  title="SwitchButton"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<switch>Switch</switch>
<switch disabled>Switch Disabled</switch>
<switch icon="{custom.png}" checked>Switch Icon</switch>
```

#### Rust Example

```rust
fn spawn_switchbutton_widget(mut commands: Commands) {
    commands.spawn((
        SwitchButton {
            label: "Switch".to_string(),
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        SwitchButton {
            label: "Switch Disabled".to_string(),
            ..default()
        },
        UIWidgetState {
            disabled: true,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        SwitchButton {
            label: "Switch Icon".to_string(),
            icon: Some("custom.png".to_string()),
            ..default()
        },
        UIWidgetState {
            checked: true,
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
