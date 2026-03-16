---
title: Form
---

# Form

### Überblick

Formular-Container, der Kinder validiert und Submit-Aktionen mit gesammelten Daten auslöst.

- Rust-Komponente: Form
- HTML-Tag: form
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- action-Handler erhält gesammelte Submit-Daten.
- validate-Modus: Always, Send, Interact.
- Submit-Button triggert Validierung und HtmlSubmit-Event.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### Form
<iframe
  id="form"
  title="Form"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<form action="log_form" validate="Send" class="con-column">
  <input name="email" type="email" required />
  <button type="submit">Send</button>
</form>
```

#### Rust Example

```rust
fn spawn_form_widget(mut commands: Commands) {
    commands
        .spawn((
            Form {
                action: Some("log_form".to_string()),
                validate_mode: FormValidationMode::Send,
                ..default()
            },
            Node::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                InputField {
                    name: "email".to_string(),
                    input_type: InputType::Email,
                    ..default()
                },
                ValidationRules {
                    required: true,
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                Button {
                    text: "Send".to_string(),
                    button_type: ButtonType::Submit,
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
