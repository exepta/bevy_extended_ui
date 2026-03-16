---
title: CheckBox
---

# CheckBox

### Overview

Boolean input widget with label and optional icon for yes/no states.

- Rust component: CheckBox
- HTML tag: checkbox
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- Main tag is checkbox with label text.
- Optional icon attribute for the check mark.
- Checked state can be read out via runtime state.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

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
        CheckBox {
            label: "Checkbox".to_string(),
            checked: false,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        CheckBox {
            label: "Checked".to_string(),
            checked: true,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        CheckBox {
            label: "Disabled".to_string(),
            ..default()
        },
        UIWidgetState {
            disabled: true,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        CheckBox {
            label: String::new(),
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        CheckBox {
            label: String::new(),
            checked: true,
            ..default()
        },
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
        CheckBox {
            label: "Checkbox".to_string(),
            icon_path: Some("examples/icons/custom.png".to_string()),
            checked: false,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        CheckBox {
            label: "Checked".to_string(),
            icon_path: Some("examples/icons/custom.png".to_string()),
            checked: true,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        CheckBox {
            label: "Disabled".to_string(),
            icon_path: Some("examples/icons/custom.png".to_string()),
            ..default()
        },
        UIWidgetState {
            disabled: true,
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
