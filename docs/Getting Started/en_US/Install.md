---
title: Install
---

# Install Bevy Extended UI

## Requirements

- Rust toolchain (`stable` recommended by default in this repository)
- A Bevy project
- Asset folder for your HTML/CSS files (usually `assets/`)

## 1. Add dependencies

Use Cargo to add both crates:

```bash
cargo add bevy_extended_ui
cargo add bevy_extended_ui_macros
```

If you prefer manual setup, add them in `Cargo.toml` with matching versions from crates.io.

## 2. Register the plugin

Add the plugin to your app:

```rust
use bevy::prelude::*;
use bevy_extended_ui::ExtendedUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ExtendedUiPlugin)
        .run();
}
```

## 3. Load your first HTML UI

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/main.html");
    reg.add_and_use("main-ui".to_string(), HtmlSource::from_handle(handle));
}
```

Then register that system, for example at startup:

```rust
app.add_systems(Startup, load_ui);
```

## 4. Optional feature flags

You can enable additional behavior via Cargo features:

- `css-breakpoints`: desktop breakpoint handling
- `wasm-breakpoints`: browser viewport breakpoints for WASM
- `wasm-default`: preset for common WASM setup
- `fluent`: Fluent language backend
- `properties-lang`: Java properties language backend
- `clipboard-wasm`: clipboard support in WASM

Example:

```toml
[dependencies]
bevy_extended_ui = { version = "x.y.z", features = ["wasm-default", "fluent"] }
bevy_extended_ui_macros = "x.y.z"
```

## 5. Verify setup

- Place your HTML and CSS under `assets/`
- Run your app
- Confirm that the UI is visible and event handlers are called

For concept and architecture context, see [Overview](./Overview.md).
