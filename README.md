# bevy_extended_ui

---

[![Crates.io](https://img.shields.io/crates/v/bevy_extended_ui.svg)](https://crates.io/crates/bevy_extended_ui)
[![Downloads](https://img.shields.io/crates/d/bevy_extended_ui.svg)](https://crates.io/crates/bevy_extended_ui)
[![license](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE)
[![Build](https://github.com/exepta/bevy_extended_ui/actions/workflows/build.yml/badge.svg)](https://github.com/exepta/bevy_extended_ui/actions/workflows/build.yml)

Since I've been writing a game in the [_Bevy_](https://bevyengine.org/) engine lately,
I created this crate. In my game,
I need more complex UI elements that Bevy doesn't currently support.
These include sliders, choice boxes, check boxes, radio buttons, and so on.
So I set about implementing these elements using the standard bevy_ui system.
I'd like to make this project available to you so that you can use elements other
than just nodes, buttons, or images. If you're missing a widget and know how
to create it, it would be great if you could add it.
Otherwise, feel free to create a ticket.

### Features

There are many features in this crate. You can see all current supported widgets [here](WIDGETS.md).
Available features:

- [x] Full HTML support.
- [x] CSS support but not all CSS properties.
- [x] Hot reload support.
- [x] HTML Bind support for interacting with the code.
- [x] Font support for family and weight.
- [x] Animation support (`@keyframes`).
- [x] Breakpoint support (`@media` for window size).
- [x] Validation for widgets like required fields.
- [x] CSS `*` support.
- [x] Custom Cursor or system cursor support.
- [x] Form Widget for validation and submission.
- [ ] Customizable theme.

There are many other things, but currently you can use the core (HTML / CSS) features.

### Toolchains

This project supports both stable and nightly Rust.

- Default: `stable` via `rust-toolchain.toml`
- Nightly: `cargo +nightly-2025-08-07 ...` or `rustup override set nightly-2025-08-07`
- Optional: use `rust-toolchain-nightly.toml` as a drop-in replacement if you want nightly by default

### How to use?

Add this to your `Cargo.toml`:

```toml
[dependencies]
bevy_extended_ui = "1.4.0"
bevy_extended_ui_macros = "1.4.0"
```

#### Features
| Feature            | Description                                                                                          |
|--------------------|------------------------------------------------------------------------------------------------------|
| `default`          | Enables `css-breakpoints`.                                                                           |
| `wasm-default`     | Web preset: `wasm-breakpoints` + `clipboard-wasm` with legacy WASM CSS/style pipeline compatibility. |
| `css-breakpoints`  | Desktop breakpoints via primary window.                                                              |
| `wasm-breakpoints` | WASM breakpoints via browser viewport.                                                               |
| `fluent`           | Enables Fluent Language support.                                                                     |
| `properties-lang`  | Enables Java Properties Language support.                                                            |
| `clipboard-wasm`   | Enables WASM clipboard support web.                                                                  |

Then, you add the plugin to your `main.rs` or on any point at a build function:

```rust
fn build(&mut app: App) {
    app.add_plugin(ExtendedUiPlugin);
}
```

Then you create an HTML file:

```html
<html lang="en">
  <head>
    <meta name="test" />
    <meta charset="UTF-8" />
    <title>Title</title>
    <!-- You can link your CSS file here. -->
    <link rel="stylesheet" href="base.css" />
    <!-- <link rel="stylesheet" href="base2.css"> You can add more CSS files.-->
  </head>
  <body>
    <!-- You can use HTML bindings here. like the onclick attribute. -->
    <button onclick="test_click">Button</button>
  </body>
</html>
```

And finally,

```rust
fn build(&mut app: App) {
    app.add_systems(Startup, |mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>| {
        let handle: Handle<HtmlAsset> = asset_server.load("YOUR_ASSETS_LOCATION/test.html");
        reg.add_and_use("test".to_string(), HtmlSource::from_handle(handle));
    });
}

#[html_fn("test_click")]
fn test_click(In(event): In<HtmlEvent>) {
    println!("Button clicked!");
}
```

Note that currently you can use this binding:

- `onclick`
- `onchange` Only for `<select>`, `<fieldset>`, `<input>`, `<date-picker>`, `<checkbox>`, `<slider>` and `<colorpicker>` elements.
- `action` on `<form>` for submit handlers with collected input data
- `oninit`
- `onmouseover`
- `onmouseout`
- `onfoucs`
- `onscroll`
- `onkeydown`
- `onkeyup`
- `ondragstart`
- `ondrag`
- `ondragstop`

### WASM support

Now we have wasm support but still not fully tested! Here is an example to show you
how to use it:

```rust
fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy WASM".into(),
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, load_ui)
        .run();
}

fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("test.html");
    reg.add_and_use("test-ui".to_string(), HtmlSource::from_handle(handle));
}

```

and the index HTML:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Bevy Web</title>
    <style>
      html,
      body {
        margin: 0;
        padding: 0;
        width: 100%;
        height: 100%;
        overflow: hidden;
        background: #333;
      }
      #container {
        width: 100%;
        height: 100%;
      }
      canvas {
        width: 100%;
        height: 100%;
        display: block;
      }
    </style>
    <link data-trunk rel="copy-dir" href="assets" />
  </head>
  <body>
    <div id="container">
      <canvas id="bevy"></canvas>
    </div>
  </body>
</html>
```

It was tested with the crate trunk, which worked well. This trunk.toml file was used:

```toml
[build]
dist = "dist"

[[copy]]
source = "assets"
dest = "assets"

[serve]
no_spa = true
```

### Animation support

Basic `@keyframes` usage example:

```css
@keyframes button-pulse {
  0% {
    transform: scale(1);
    background-color: #4c8bf5;
  }
  50% {
    transform: scale(1.05);
    background-color: #72a1ff;
  }
  100% {
    transform: scale(1);
    background-color: #4c8bf5;
  }
}

.cta-button {
  animation: button-pulse 1.4s ease-in-out infinite alternate;
}
```

You can now use `@keyframes` in your CSS. There is now a limit tested; this means that you can use any CSS property.

### Breakpoint support

Basic `@media` usage for breakpoints:

```css
.desktop-panel {
  display: flex;
}

@media (max-width: 900px) {
  .desktop-panel {
    display: none;
  }
}
```

Breakpoint runtime source is feature-based:

- `css-breakpoints` (default): tracks Bevy `PrimaryWindow` size.
- `wasm-breakpoints`: tracks browser viewport (`window.innerWidth/innerHeight`) for WASM and overrides `css-breakpoints` when enabled.
- `wasm-default`: enables `wasm-breakpoints` and `clipboard-wasm` as a web preset.

### What comes next?

Next, I'd like to build a website that's structured like React documentation.
There you'll always be able to see all the patches and how to apply them!
If anyone has any ideas, I'd be happy to hear them.

### Compatibility

> _Note:_ This project is currently under construction and not suitable for large projects!.

| `Bevy` version | `bevy_extended_ui` version |
|----------------|----------------------------|
| 0.18.0         | 1.2.0 - 1.4.0              |
| 0.17.0         | 1.0.0 - 1.1.0              |
| 0.16.0         | 0.1.0 - 0.2.2              |

> _Note:_ WASM is not correctly supported in version 1.4.0 there is a layout bug. I'm working on this bug, but this needs time!

> _Note:_ Version 0.1.0–0.3.0 are deprecated and will not be supported. If you’re interested in a version for bevy 0.16, then create an issue!

### Important Links

[Link to Widget attributes](WIDGETS.md)
<br>
[Link to Event Rules](EVENT.md)
<br>
[Link to Language Rules](LANGUAGE.md)
<br>
[Link to CSS Properties](CSS_USAGE.md)
<br>
[Link to Patches](PATCH.md)
