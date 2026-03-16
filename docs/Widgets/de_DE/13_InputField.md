---
title: InputField
---

# InputField

### Überblick

Textbasiertes Eingabe-Widget für Text-, E-Mail-, Datum-, Zahl-, Passwort- und Datei-Felder.

- Rust-Komponente: InputField
- HTML-Tag: input
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- Unterstützt id, name, type, value, placeholder, icon, maxlength, format, folder, extensions, show-size, max-size.
- Eingabetypen: text/email/date/password/number/file.
- Hinweise für `type="file"`:
  - `folder="true|false"` (Standard `false`)
  - `extensions="json"` oder `extensions="[json, css, yaml, png]"` (wird bei `folder="true"` ignoriert)
  - `show-size="true|false"` (Standard `false`)
  - `max-size="1KB|1MB|1GB"` weist Dateien zurück, die größer als das Limit sind
- Validierungsattribute required und validation werden unterstützt.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### InputField Standard
<iframe
  id="inputfield"
  title="InputField"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<label for="username">Text</label>
<input id="username" name="username" type="text" maxlength="32" placeholder="A Placeholder" />
<label for="password">Password</label>
<input id="password" name="password" type="password" maxlength="16" />
<label for="email">Email</label>
<input id="email" name="email" type="email" />
<label for="number">Number</label>
<input id="number" name="number" type="number" maxlength="16" />
```

#### Rust Example

```rust
fn spawn_inputfield_widget(mut commands: Commands) {
    commands.spawn((
        InputField {
            name: "username".to_string(),
            input_type: InputType::Text,
            cap_text_at: InputCap::CapAt(32),
            placeholder: "A Placeholder".to_string(),
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        InputField {
            name: "password".to_string(),
            input_type: InputType::Password,
            cap_text_at: InputCap::CapAt(16),
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        InputField {
            name: "email".to_string(),
            input_type: InputType::Email,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        InputField {
            name: "number".to_string(),
            input_type: InputType::Number,
            cap_text_at: InputCap::CapAt(16),
            ..default()
        },
        Node::default(),
    ));
}
```

### InputField File
<iframe
id="inputfield-file"
title="InputField"
src="{base.url}/examples/base"
width="100%"
height="420"
loading="lazy">
</iframe>

#### Html Example

```html
<label for="file">File</label>
<input id="file" name="file" type="file" />
<label for="file-size">File With Size</label>
<input id="file-size" name="file-size" type="file" max-size="2MB" extensions="[jpg, jpeg, png]" show-size="true" />
```

#### Rust Example

```rust
fn spawn_inputfield_widget(mut commands: Commands) {
    commands.spawn((
        InputField {
            name: "file".to_string(),
            input_type: InputType::File,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        InputField {
            name: "file-size".to_string(),
            input_type: InputType::File,
            max_size_bytes: Some(2 * 1024 * 1024),
            extensions: vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string()],
            show_size: true,
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
