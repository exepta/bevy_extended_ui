---
title: ToggleButton
---

# ToggleButton

### Overview

Selectable button widget for single or multiple selection, often in the FieldSet.

- Rust component: ToggleButton
- HTML tag: toggle
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- Supports value and selected attributes.
- Optional icon child for toolbar-like UIs.
- Commonly used in fieldset mode=multi.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### ToggleButton Example
<iframe
  id="togglebutton"
  title="ToggleButton"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<toggle value="bold" selected onclick="log_togglebutton">
  <icon src="extended_ui/icons/bold.png"></icon>
</toggle>
```

#### Rust Example

```rust
fn spawn_togglebutton_widget(mut commands: Commands) {
    commands.spawn((
        ToggleButton {
            label: String::new(),
            value: "bold".to_string(),
            icon_path: Some("extended_ui/icons/bold.png".to_string()),
            icon_place: IconPlace::Left,
            selected: true,
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
