# bevy_extended_ui
___
[![Crates.io](https://img.shields.io/crates/v/bevy_extended_ui.svg)](https://crates.io/crates/bevy_extended_ui)
[![Downloads](https://img.shields.io/crates/d/bevy_extended_ui.svg)](https://crates.io/crates/bevy_extended_ui)
[![license](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE)
[![Build](https://github.com/exepta/bevy_extended_ui/actions/workflows/build.yml/badge.svg)](https://github.com/exepta/bevy_extended_ui/actions/workflows/build.yml)


Since I've been writing a game in the [_Bevy_](https://bevyengine.org/) engine lately, 
I created this crate. In my game, 
I need more complex UI elements that aren't currently supported by Bevy. 
These include sliders, choice boxes, check boxes, radio buttons, and so on. 
So I set about implementing these elements using the standard bevy_ui system. 
I'd like to make this project available to you so that you can use elements other 
than just nodes, buttons, or images. If you're missing a widget and know how 
to create it, it would be great if you could add it. 
Otherwise, feel free to create a ticket.

> *Note:* This project is currently under construction and not suitable for large projects!.

## Example
___

Here I will show you how to use the bevy_extended_ui:


First we need to integrate the plugin into your project.
```rust
fn main() {
  let _ = App::new()
          .add_plugins((DefaultPlugins, ExtendedUiPlugin))
          .run();
}
```


Next, you can get started right away. Currently, there are widgets (Div, Button, Checkbox, InputField, and Slider). Note that these aren't all the widgets! More are coming soon.

Here's a simple example of a button that we spawn
```rust
    commands.spawn((
        Div::default(),
        CssSource(String::from("test.css")),
        CssClass(vec![".div-test".to_string(), ".div-override".to_string()]),
        children![
            Button::default(),
            Button::default()
        ]
    ));
```
In the end it should look like this:

![Result Example](docs/example_readme.png)

HTML works now with bevy_extended_ui. You can show the html from your website in bevy!
But currently these widgets ar supported:
- Button
- Div
- Body
- H1 - H6
- Paragraph
- Slider
- Select
- Input type:`number`, `text` and `password`
```rust
    commands.spawn(HtmlSource(String::from("examples/html/login-ui.html")));
```
The html file needed a `head` element which contains a `<meta name="test">` tag, this is used
for identify the correct ui which you currently need.

For implement css styling use `<link href="css/example.css" ref="text/css">`. At the moment only one css
at the same time is supported!
Here is an example html:
```html
<head>
    <meta name="login-example" />
    <link rel="stylesheet" href="examples/css/login-ui.css" />
    <title>Page Title</title>
</head>

<body>
<!-- Login Page -->
<div id="container">
    <!-- Login headline -->
    <h2>Login</h2>

    <!-- Input Field with type text and an icon -->
    <label for="username">Username</label>
    <input id="username" type="text" icon="icons/user-icon.png" />

    <!-- Input Field with type password and an icon -->
    <label for="password">Password</label>
    <input id="password" type="password" icon="icons/pass-icon.png" />

    <!-- Remember me -->
    <checkbox id="remember-me">Remember me</checkbox>

    <!-- Forgot password -->
    <p>Forgot password?</p>

    <!-- Login container -->
    <div class="button-container">
        <button id="login">
            Login
        </button>
    </div>
</div>
</body>
```

All Widgets support CSS3 and can apply many of the default css rules. Note that the current system working with css but
not perfect yet! Let me explain it:

```css
button {
    background: rgb(255, 255, 255); /* will be white */
    display: flex; /* set node display flex */
}

button:hover {
    background: rgba(200, 200, 200, 200); /* will work correctly */
}

.button-text {
    color: #FFFFFF; /* is white and working */
}

/* THIS WORK IF THE BUTTON IS HOVERED! */
.button-text:hover {
    color: red; /* set red */
}

/* THIS WILL WORK TOO! */
button:hover .button-text {
    color: red; /* set red */
}
```

## Examples

For more examples, look at the example package. If you need help with CSS rules, look at CSS_SUPPORT.md in the same folder!
The crate supports many CSS things, the list below shows the future support:
- CSS variables
- CSS Animations @keyframes
- CSS media queries
- HTML function like javascript
- & ~ > Operators
- SCSS support

| `Bevy` version | `bevy_extended_ui` version |
|----------------|----------------------------|
| 0.16.1         | 0.1.0 - 0.1.2              |
| 0.16.0         | 0.1.0 - 0.1.2              |