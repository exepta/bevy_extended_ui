---
title: DatePicker
---

# DatePicker

### Overview

Calendar widget that can be bound to an input alone or via the for attribute.

- Rust component: DatePicker
- HTML tag: date-picker
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- Supports for, name, label, placeholder, value, min, max, format.
- Works alone or tied to an input.
- Handy with onchange handlers for date dependent UIs.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### DatePicker
<iframe
  id="datepicker"
  title="DatePicker"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<input id="birthday" type="date" />
<date-picker for="birthday" min="1990-01-01" max="2030-12-31" format="dmy"></date-picker>
```

#### Rust Example

```rust
fn spawn_datepicker_widget(mut commands: Commands) {
    commands.spawn((
        DatePicker::default(),
        Node::default(),
    ));
}
```

### DatePicker Range
<iframe
id="datepicker-range"
title="DatePicker"
src="{base.url}/examples/base"
width="100%"
height="420"
loading="lazy">
</iframe>

#### Html Example

```html
<input id="birthday" type="range" />
<date-picker for="birthday" min="1990-01-01" max="2030-12-31" format="dmy"></date-picker>
```

#### Rust Example

```rust
fn spawn_datepicker_widget(mut commands: Commands) {
    commands.spawn((
        DatePicker::default(),
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
