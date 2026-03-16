---
title: Body
---

# Body

### Overview

`Body` is the central root widget of the UI structure and represents the HTML tag `<body>` within the parser. All child widgets are built under this node, making `Body` the basis for layout, scrolling behavior, event routing, and linking to the loaded UI source (`html_key`).

- Rust component: Body
- HTML tag: body
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

No extra attributes!

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

#### Html Example

```html
<body id="main-body" class="screen-root app-layout" oninit="on_body_init">
  <div class="content">...</div>
</body>
```

#### Rust Example

```rust
fn spawn_body_widget(mut commands: Commands) {
    commands
        .spawn((
            Body {
                html_key: Some("main-body".to_string()),
                ..default()
            },
            Node::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Div::default(),
                Node::default(),
                Paragraph {
                    text: "...".to_string(),
                    ..default()
                },
            ));
        });
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
