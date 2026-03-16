---
title: InputField
---

# InputField

### Overview

Text-based input widget for text, email, date, number, password and file fields.

- Rust component: InputField
- HTML tag: input
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- Supports id, name, type, value, placeholder, icon, maxlength, format, folder, extensions, show-size, max-size.
- Input types: text/email/date/password/number/file.
- Notes for `type="file"`:
  - `folder="true|false"` (default `false`)
  - `extensions="json"` or `extensions="[json, css, yaml, png]"` (ignored for `folder="true"`)
  - `show-size="true|false"` (default `false`)
  - `max-size="1KB|1MB|1GB"` rejects files larger than the limit
- Validation attributes required and validation are supported.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### InputField Example
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
<input id="username" name="username" type="text" maxlength="32" placeholder="Your name" onchange="log_inputfield" />
```

#### Rust Example

```rust
fn spawn_inputfield_widget(mut commands: Commands) {
    commands.spawn((
        InputField::default(),
        Node::default(),
    ));
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
