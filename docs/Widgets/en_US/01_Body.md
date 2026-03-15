---
title: Body
---

# Body

## Overview

`Body` is the root widget of the parsed UI tree and maps directly to the HTML `<body>` tag. It is the anchor for all child widgets and is responsible for important runtime behavior such as tree ownership, scroll structure setup, event routing, and source-key linking through `html_key`.

- Rust component: Body
- HTML tag: body
- Recommended source reference: src/widgets/mod.rs

## Attributes

Keine extra attributes!

Supported global HTML attributes (detailed):

- `id`: Sets a unique element id for selectors, binding references, and runtime lookup.
- `class`: Adds one or many CSS classes that are forwarded into the Bevy styling pipeline.
- `style`: Inline CSS declarations are parsed into `HtmlStyle` and merged with other style sources.
- `hidden`: Marks the widget as initially hidden so it does not render visibly at start.
- `disabled`: Marks the widget state as disabled and blocks relevant interactions.
- `readonly`: Sets read-only state for consistent state propagation behavior.
- Event attributes such as `onclick`, `oninit`, `onmouseover`, `onmouseout`, `onfocus`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Register handler names via `HtmlEventBindings`.

## WASM Preview

<iframe
  id="body"
  title="Bevy WASM Preview - Body"
  src="{base.url}/examples/body"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Html Example

```html
<body id="main-body" class="screen-root app-layout" oninit="on_body_init">
  <div class="content">...</div>
</body>
```

## Rust Example

```rust
fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/body.html");
    reg.add_and_use("body-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("on_body_init")]
fn on_body_init(In(event): In<HtmlInit>, query: Query<&Body>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Body initialized: entry={} html_key={:?}", widget.entry, widget.html_key);
    }
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
