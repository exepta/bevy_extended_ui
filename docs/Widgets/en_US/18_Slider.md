---
title: Slider
---

# Slider

### Overview

Numeric drag widget with min, max, value and step properties.
Supports `default` (one thumb) and `range` (two thumbs).

- Rust component: Slider
- HTML tag: slider
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- `min`, `max`, `step`: Number range and step size.
- `type`: `default | range` (Default: `default`).
- `value`:
  - `default`: single numerical value (e.g. `35`)
  - `range`: Area in format `start - end` (e.g. `20 - 40`)
- `dots`: Number of segments/ticks. Example `dots="5"` at `0..100` produces ticks at `0, 20, 40, 60, 80, 100`.
  - If `dots` is set and `<= 1` (also `0` or negative), only min/max ticks are used (like `dots="1"`).
  - Internal protection: Minimum distance between neighboring ticks is 10 value units.
- `show-labels`: `true | false` (Default: `false`). Shows tick labels.
- `dot-anchor`: `top | bottom` (Default: `top`). Position of the tick labels relative to the track.
- `tip`: `true | false` (Default: `true`). Shows/hides thumb tooltip.
- Track responds to click and drag.
- In `range` mode, the tooltip shows the value of the hovered thumb.
- Works with onchange/oninput handlers.
- Current value is in the slider component.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### Slider Example
<iframe
  id="slider"
  title="Slider"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<slider min="0" max="100" value="25" step="1" onchange="log_slider"></slider>
```

#### Rust Example

```rust
fn spawn_slider_widget(mut commands: Commands) {
    commands.spawn((
        Slider {
            min: 0.0,
            max: 100.0,
            value: 25.0,
            step: 1.0,
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
