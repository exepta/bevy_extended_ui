---
title: Component System
---

> Supported by version `1.6.0` and above.

# How-to: Component System (`extended-framework`)

This guide covers the recommended flow for the new component system with `extended-framework`.

## When should you use this?

Use `extended-framework` when you want:

- a fixed `index.html` entrypoint
- reusable HTML component tags
- automatic injection of component-specific styles

Important: the legacy `UiRegistry` system is disabled in this mode.

## 1) Enable the feature

```toml
[dependencies]
bevy_extended_ui = { version = "x.x.x", features = ["extended-framework"] }
bevy_extended_ui_macros = "x.x.x"
```

## 2) Folder layout

Example layout:

```text
assets/
  index.html
  components/
    main.component.rs
    main.component.html
    main.component.css
```

## 3) `index.html` entrypoint

`extended-framework` requires a real `index.html` entry file.

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Framework UI</title>
  </head>
  <body>
    <app-main />
  </body>
</html>
```

## 4) Component definition (`*.component.rs`)

File: `src/packages/main.component.rs`

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlInit;
use bevy_extended_ui_macros::{html_fn, ui_component};

#[ui_component]
pub struct MainComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const MAIN_COMPONENT: MainComponent = MainComponent {
    template_name: "app-main",
    template_file: "main.component.html",
    styles: &["main.component.css"],
};

#[html_fn("init_main")]
pub fn init_main(In(_): In<HtmlInit>) {
    info!("app-main initialized");
}
```

Matching template file: `assets/components/main.component.html`

```html
<div>
  <h1 onclick="init_main">Hello from app-main</h1>
</div>
```

## 5) Include component file in your build

Since `*.component.rs` is not a regular Rust module name, include it via `#[path]`:

```rust
#[path = "packages/main.component.rs"]
mod main_component_mod;
```

Without this, `#[html_fn]` handlers inside that file are not compiled/registered.

## 6) App setup

```rust
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy_extended_ui::framework::ExtendedFrameworkConfiguration;
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};

#[path = "packages/main.component.rs"]
mod main_component_mod;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "assets".to_string(),
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, configure_ui)
        .run();
}

fn configure_ui(
    mut ui_config: ResMut<ExtendedUiConfiguration>,
    mut fw_config: ResMut<ExtendedFrameworkConfiguration>,
) {
    ui_config.camera = ExtendedCam::Default;
    ui_config.framework_components_path = "components".to_string();

    fw_config.asset_root_fs_path = "assets".to_string();
    fw_config.assets_component_root = "components".to_string();
    fw_config.rust_component_root = "src/packages".to_string();
    fw_config.index_html_file = "index.html".to_string();
}
```

## 7) Contracts and common failures

1. `index_html_file` must be `index.html`.
2. `template_file` must match the component filename:
   - `main.component.rs` -> `main.component.html`
3. Every component requires `#[ui_component]`.
4. `template_name` must be unique.
5. Referenced style/template files must exist.

## 8) Event handlers with `#[html_fn]`

`#[html_fn("name")]` links HTML attributes to Rust handlers:

- HTML: `onclick="save_settings"`
- Rust: `#[html_fn("save_settings")]`

Example:

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui_macros::html_fn;

#[html_fn("save_settings")]
fn save_settings(In(event): In<HtmlClick>) {
    info!("Click on entity {:?}", event.entity);
}
```

