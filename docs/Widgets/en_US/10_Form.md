---
title: Form
---

# Form

## Overview

Form container that validates child fields and emits submit actions with collected input data.

- Rust component: Form
- HTML tag: form
- Recommended source reference: src/widgets/mod.rs

## Important Attributes and Behavior

- action handler receives collected submit data.
- validate mode: Always, Send, Interact.
- Submit button triggers validation and HtmlSubmit event.

## HTML Example

```html
<form action="log_form" validate="Send">
  <input name="email" type="email" required />
  <button type="submit">Send</button>
</form>
```

## Bevy Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;
use bevy_extended_ui::html::{HtmlEvent, HtmlSource};
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use bevy_extended_ui::widgets::Form;
use bevy_extended_ui_macros::html_fn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/form.html");
    reg.add_and_use("form-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("log_form")]
fn log_form(In(event): In<HtmlEvent>, query: Query<&Form>) {
    if let Ok(widget) = query.get(event.entity) {
        info!("Form event entity={:?} data={:?}", event.entity, widget);
    }
}
```

## Example

<iframe
  title="Bevy WASM Preview - Form"
  src="{base.url}/examples/form"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

## Notes

- Keep the HTML tag spelling exact (form) so the converter maps to the correct widget.
- Register handler names with html_fn exactly as used in HTML attributes.
- Link this page to a real demo build once your WASM preview is deployed.
