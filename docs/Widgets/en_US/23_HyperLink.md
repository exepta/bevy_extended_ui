---
title: HyperLink
---

# HyperLink

### Overview

Clickable text link widget that maps to the HTML tag `<a>`.

- Rust component: HyperLink
- HTML tag: a
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- `href` defines the target URL.
- `browsers` is optional (default: `system`).
- Supported values for `browsers`: `system`, single browser (`firefox`, `chrome`, `brave`, ...) or list (`[firefox, brave, chrome]`).
- `open-modal` is optional (default: `false`).
- If `open-modal="true"` is set and the configured browser is not installed, the Bevy app will display a modal asking if the install command should be opened in the terminal.
- Native browser detection is implemented for Linux, macOS and Windows.
- Under `wasm32`, `browsers` and `open-modal` are ignored.
- There is no system browser fallback if explicit `browsers` are configured and none is installed.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### HyperLink Example
<iframe
  id="hyperlink"
  title="HyperLink"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<a href="https://bevy.org">Open with system browser</a>
<a href="https://bevy.org" browsers="[chrome]" open-modal="true">Open with configured browser</a>
```

#### Rust Example

```rust
fn spawn_hyperlink_widget(mut commands: Commands) {
    commands.spawn((
        HyperLink {
            text: "Open with system browser".to_string(),
            href: "https://bevy.org".to_string(),
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        HyperLink {
            text: "Open with configured browser".to_string(),
            href: "https://bevy.org".to_string(),
            browsers: HyperLinkBrowsers::Custom(vec!["chrome".to_string()]),
            open_modal: true,
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
