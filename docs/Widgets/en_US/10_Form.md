---
title: Form
---

# Form

### Overview

Form container that validates children and triggers submit actions with collected data.

- Rust component: Form
- HTML tag: form
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- action handler receives collected submit data.
- validate mode: Always, Send, Interact.
- Submit button triggers validation and HtmlSubmit event.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### Form Example
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
<form action="log_form" validate="Send">
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

### Widget Creator

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
