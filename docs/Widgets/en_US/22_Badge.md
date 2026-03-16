---
title: Badge
---

# Badge

### Overview

Small notification widget that typically binds to another target widget.

- Rust component: Badge
- HTML tag: badge
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- Supports `for`, `value` (Alias: `count`), `max`, `anchor`.
- If `value > max`, the label will be rendered as `+{max}` (e.g. `+99`).
- Target widget resolution:
- `for="id"` binds to the widget with this id.
- Without `for`, the next widget parent is used.
- If no target is found, the badge remains hidden.
- Anchor values: `top right` (default), `top left`, `bottom right`, `bottom left`.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### Badge Example
<iframe
  id="badge"
  title="Badge"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<button id="inbox-button">Inbox</button>
<badge for="inbox-button" value="112" max="99" anchor="top right"></badge>
```

#### Rust Example

```rust
fn spawn_badge_widget(mut commands: Commands) {
    commands.spawn((
        Badge::default(),
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
