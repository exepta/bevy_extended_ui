---
title: Button
---

# Button

## Overview

`Button` is an interactive action widget for click flows and form integration. The widget text is read from tag content, an optional `<icon src="...">` child is detected automatically, and the `type` value (`button`, `submit`, `reset`) controls how the widget behaves in forms.

- Rust component: Button
- HTML tag: button
- Recommended source reference: src/widgets/mod.rs

## Attributes

Important widget-specific attributes (detailed):

- `type`: Controls the button behavior mode.
  Allowed values:
  `button` (normal click action), `submit` (triggers form submit), `reset` (resets form values).
  If omitted, the internal default mode `Auto` is used.

Supported global HTML attributes:

- `id`: Unique id for selectors, event mapping, and future widget references.
- `class`: Assigns CSS classes used by style rules and state-based styling.
- `style`: Inline declarations are parsed into `HtmlStyle` and merged in the style pipeline.
- `hidden`: Makes the button initially invisible.
- `disabled`: Disables interactions and blocks active click/focus behavior.
- `readonly`: Carried into widget state handling to preserve consistent interaction logic.
- Event attributes such as `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, and `oninit`: Bind function names into the event binding system.

Also relevant:

- `<icon src="..."></icon>`: This is not an attribute but a child element. It is parsed and placed before or after text depending on content order.

## WASM Previews

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

## Rust Example

```rust
fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/button.html");
    reg.add_and_use("button-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("on_save_click")]
fn on_save_click(In(event): In<HtmlClick>, query: Query<&Button>) {
    if let Ok(widget) = query.get(event.entity) {
        info!(
            "Button clicked: text='{}' type={:?} icon={:?}",
            widget.text,
            widget.button_type,
            widget.icon_path
        );
    }
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

## Rust Example

```rust
fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/button.html");
    reg.add_and_use("button-demo".to_string(), HtmlSource::from_handle(handle));
}
```

## Widget Creator

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
    <a href="https://github.com/exepta" style="margin-top: 10px;">Link to GitHub</a>
  </div>
</div>
