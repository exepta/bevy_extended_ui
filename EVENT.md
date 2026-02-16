# HTML Events

This document describes the HTML event bindings provided by `bevy_extended_ui` and the payloads you receive in Rust.

## Binding attributes

Use these attributes in your HTML:

- `onclick`
- `onchange`
- `action` (on `<form>`, submit handler name)
- `oninit`
- `onmouseover`
- `onmouseout`
- `onfoucs` (alias: `onfocus`)
- `onscroll`
- `onkeydown`
- `onkeyup`
- `ondragstart`
- `ondrag`
- `ondragstop` (alias: `ondragend`)

## Using handlers

Use the `#[html_fn("name")]` macro to register a handler. You can accept either the generic `HtmlEvent`
(only the target entity) or a concrete typed event.

Generic (target only):

```rust
#[html_fn("log_target")]
fn log_target(In(event): In<HtmlEvent>) {
    info!("Target: {:?}", event.target());
}
```

Typed event example:

```rust
#[html_fn("on_key_up")]
fn on_key_up(In(event): In<HtmlKeyUp>) {
    info!("Key up: {:?}", event.key);
}
```

## Event payloads

All events include `target` (the Bevy `Entity` that fired the event).

### HtmlChange

- `target`
- `action`: `State`, `Style`, or `Unknown`

### HtmlFocus

- `target`
- `state`: `Gained` or `Lost`

### HtmlSubmit

- `target`: submitted form entity
- `submitter`: clicked submit button entity
- `action`: form action handler name
- `data`: collected input map (`name -> value`)

### HtmlScroll

- `target`
- `delta`: `Vec2` scroll delta since the last scroll event
- `x`: current scroll position on X
- `y`: current scroll position on Y

### HtmlClick

- `target`
- `position`: screen-space pointer position
- `inner_position`: pointer position inside the target (best effort; falls back to `position`)

### HtmlKeyDown / HtmlKeyUp

- `target`
- `key`: `KeyCode`

### HtmlDragStart / HtmlDrag / HtmlDragStop

- `target`
- `position`: screen-space pointer position
