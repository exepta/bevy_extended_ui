---
title: List
---

# List

### Overview

List widget based on `<listbox>` where all options stay visible at the same time.

- Rust component: ListBox
- HTML tag: listbox
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- `multiselect` on `<listbox>` enables multi-selection.
- Without `multiselect`, only one option can be active at a time.
- `<option value="...">` sets the internal option value.
- `<option selected>` marks initially selected entries.
- `<option icon="...">` sets an optional icon for each entry.
- `<option internal-value-type="...">` sets the target type for parsed values.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### List Example
<iframe
  id="listbox"
  title="List"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<listbox id="difficulty" onchange="log_listbox">
  <option value="easy">Easy</option>
  <option value="normal" selected>Normal</option>
  <option value="hard">Hard</option>
</listbox>
```

#### Rust Example

```rust
fn spawn_list_widget(mut commands: Commands) {
    let easy = ChoiceOption::new("Easy").with_value(String::from("easy"));
    let normal = ChoiceOption::new("Normal").with_value(String::from("normal"));
    let hard = ChoiceOption::new("Hard").with_value(String::from("hard"));

    commands.spawn((
        ListBox {
            options: vec![easy.clone(), normal.clone(), hard.clone()],
            values: vec![normal],
            multiselect: false,
            ..default()
        },
        Node::default(),
    ));
}
```

### Widget Creator

<div style="display: flex; align-items: center; justify-content: flex-start; padding: 15px; border: 1px solid #5658db; border-radius: 10px; gap: 15px; width: 300px;">
  <img
    src="https://avatars.githubusercontent.com/shadow-wolftousen"
    alt="shadow-wolftousen avatar"
    width="64"
    height="64"
    style="width: 64px; height: 64px; border-radius: 50%; object-fit: cover;"
  />
  <div style="display: flex; flex-direction: column; align-items: flex-start; justify-content: center;">
    <strong>shadow-wolftousen</strong>
    <a href="https://github.com/shadow-wolftousen" style="margin-top: 10px; color: #5658db;">Link to GitHub</a>
  </div>
</div>
