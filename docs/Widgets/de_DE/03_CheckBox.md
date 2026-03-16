---
title: CheckBox
---

# CheckBox

### Überblick

Boolesches Eingabe-Widget mit Label und optionalem Icon für Ja/Nein-Zustände.

- Rust-Komponente: CheckBox
- HTML-Tag: checkbox
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- Haupt-Tag ist checkbox mit Label-Text.
- Optionales icon-Attribut für das Häkchen.
- Checked-Zustand ist über Laufzeit-State auslesbar.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### CheckBox
<iframe
  id="checkbox"
  title="CheckBox"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<checkbox>Checkbox</checkbox>
<checkbox checked>Checked</checkbox>
<checkbox disabled>Disabled</checkbox>
<checkbox></checkbox>
<checkbox checked></checkbox>
```

#### Rust Example

```rust
fn spawn_checkbox_widget(mut commands: Commands) {
    commands.spawn((
        CheckBox::default(),
        Node::default(),
    ));
}
```

### CheckBox Icon
<iframe
id="checkbox-custom-icon"
title="CheckBox"
src="{base.url}/examples/base"
width="100%"
height="420"
loading="lazy">
</iframe>

#### Html Example

```html
<checkbox icon="examples/icons/custom.png">Checkbox</checkbox>
<checkbox icon="examples/icons/custom.png" checked>Checked</checkbox>
<checkbox icon="examples/icons/custom.png" disabled>Disabled</checkbox>
```

#### Rust Example

```rust
fn spawn_checkbox_widget(mut commands: Commands) {
    commands.spawn((
        CheckBox::default(),
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
