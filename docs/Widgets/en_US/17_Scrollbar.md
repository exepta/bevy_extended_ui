---
title: Scrollbar
---

# Scrollbar

### Overview

Scroll helper widget from the scroll tag for vertical or horizontal scrolling.

- Rust component: Scrollbar
- HTML tag: scroll
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- Tag is scroll, not scrollable.
- Alignment switches vertically/horizontally.
- Useful for custom scroll areas.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### Scrollbar Example
<iframe
  id="scrollbar"
  title="Scrollbar"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<scroll alignment="vertical" onscroll="log_scrollbar"></scroll>
```

#### Rust Example

```rust
fn spawn_scrollbar_widget(mut commands: Commands) {
    commands.spawn((
        Scrollbar {
            vertical: true,
            min: 0.0,
            max: 1000.0,
            value: 0.0,
            step: 10.0,
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
