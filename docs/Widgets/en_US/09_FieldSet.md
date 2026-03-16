---
title: FieldSet
---

# FieldSet

### Overview

Group widget for selectable children such as radio and toggle elements with selection rules.

- Rust component: FieldSet
- HTML tag: fieldset
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- mode supports single, multi, count.
- allow-none controls empty selection states.
- Groups radio/toggle children with common selection status.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### FieldSet Example
<iframe
  id="fieldset"
  title="FieldSet"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<fieldset mode="single" allow-none="false" onchange="log_fieldset">
  <radio value="easy">Easy</radio>
  <radio value="hard" selected>Hard</radio>
</fieldset>
```

#### Rust Example

```rust
fn spawn_fieldset_widget(mut commands: Commands) {
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
                    label: "Easy".to_string(),
                    value: "easy".to_string(),
                    selected: false,
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                RadioButton {
                    label: "Hard".to_string(),
                    value: "hard".to_string(),
                    selected: true,
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
