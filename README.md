# bevy_extended_ui
___
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
- [ ] Customizable theme.
- [ ] Animation support.
- [ ] Validation for widgets like required fields.
- [ ] Custom Cursor or system cursor support.

There are many other things, but currently you can use the core (HTML / CSS) features.

### How to use?

Add this to your `Cargo.toml`:
```toml
[dependencies]
bevy_extended_ui = "1.1.0"
bevy_extended_ui_macros = "1.1.0"
```

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
    <meta name="test">
    <meta charset="UTF-8">
    <title>Title</title>
    <!-- You can link your CSS file here. -->
    <link rel="stylesheet" href="base.css">
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
- `onchange` Only for `<select>`, `<fieldset>`, `<input>`, `<checkbox>` and `<slider>` elements.
- `oninit`
- `onmouseover`
- `onmouseout`

### What comes next?

Next, I'd like to build a website that's structured like React documentation.
There you'll always be able to see all the patches and how to apply them!
If anyone has any ideas, I'd be happy to hear them.

### Compatibility

> *Note:* This project is currently under construction and not suitable for large projects!.

| `Bevy` version | `bevy_extended_ui` version |
|----------------|----------------------------|
| 0.17.3         | 1.1.0                      |
| 0.16.1         | 0.1.0 - 0.2.2              |
| 0.16.0         | 0.1.0 - 0.2.2              |

### Important Links

[Link to Widget attributes](WIDGETS.md)