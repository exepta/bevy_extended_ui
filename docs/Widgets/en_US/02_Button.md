---
title: Button
---

# Button

### Overview

`Button` is an interactive action widget for click and form flows. The widget text is read from inside the tag, an optional `<icon src="...">` can be automatically placed to the left or right of the text, and the widget directly controls the form behavior via the type (`button`, `submit`, `reset`).

- Rust component: Button
- HTML tag: button
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- `type`: Controls the behavior mode of the button.
  Allowed values:
  `button` (normal click action without submit), `submit` (triggers form submit), `reset` (resets form values).
  If not specified, the internal standard `Auto` is used.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### Button states
<iframe
id="button"
title="Button States"
src="{base.url}/examples/base"
width="50%"
height="250px"
loading="lazy">
</iframe>

#### Html Example

```html
<button id="save-btn" class="cta" type="submit" onclick="on_save_click">
  Save
  <icon src="extended_ui/icons/check-mark.png"></icon>
</button>
```

#### Rust Example

```rust
fn spawn_button_widget(mut commands: Commands) {
    commands.spawn((
        Button {
            text: "Save".to_string(),
            icon_path: Some("extended_ui/icons/check-mark.png".to_string()),
            icon_place: IconPlace::Right,
            button_type: ButtonType::Submit,
            ..default()
        },
        Node::default(),
    ));
}
```

### Icon Button

<iframe
id="button-icon-only"
title="Icon Button"
src="{base.url}/examples/base"
width="50%"
height="250px"
loading="lazy">
</iframe>

#### Html Example

```html
<button style="width: 50px; height: 50px; border-radius: 50%;">
  <icon src="icons/check-mark.png"></icon>
</button>
```

#### Rust Example

```rust
fn spawn_button_widget(mut commands: Commands) {
    commands.spawn((
        Button {
            text: String::new(),
            icon_path: Some("icons/check-mark.png".to_string()),
            icon_place: IconPlace::Left,
            button_type: ButtonType::Button,
            ..default()
        },
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
