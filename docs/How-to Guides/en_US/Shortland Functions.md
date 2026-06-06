---
title: Shortland Functions
---

# How-to: Shortland Functions (`$set`, `$add`, `$min`, `$event`)

Shortland functions are inline HTML event actions for small state updates.
Use them when an event only needs to write a value, add to a number, or subtract from a number.
They avoid a dedicated Rust `#[html_fn]` handler for simple changes.

```html
<input value="{{ player.name }}" onchange="$set(player.name, $event.value)">
<button onclick="$add(info.value, 1)">+</button>
<button onclick="$min(info.value, 1)">-</button>
```

## When to use them

Use shortland functions for:

- assigning input values into a `BeuStore` field
- incrementing or decrementing numeric store values
- copying simple widget state into reactive template data
- replacing trivial handlers like `set_name`, `set_value`, or `increase_value`

Keep `#[html_fn]` for:

- routing (`router.navigate(...)`)
- spawning/despawning entities
- updating multiple resources with custom logic
- validation flows or side effects
- complex conversions such as filtering date ranges or mapping selected fieldset values

## Requirements

Shortland functions are reactive through the extended framework store.
The target path should point to a `#[derive(BeuStore)]` value or a value stored in `UiBindingStore`.

```rust
use bevy_extended_ui::BeuStore;
use serde::Serialize;

#[derive(BeuStore, Clone, PartialEq, Serialize)]
pub struct Player {
    pub name: String,
    pub state: bool,
}

#[derive(BeuStore, Clone, PartialEq, Serialize)]
pub struct Info {
    pub value: f32,
    pub color: String,
}
```

The template alias is the Rust type name converted to snake case:

- `Player` -> `player`
- `Info` -> `info`
- `PlayerProfile` -> `player_profile`

So `player.name` writes to the `Player` store value, and `info.value` writes to `Info`.

## Functions

### `$set(target, value)`

Sets `target` to `value`.

```html
<input value="{{ player.name }}" onchange="$set(player.name, $event.value)">
<colorpicker value="{{ info.color }}" onchange="$set(info.color, $event.hex)"></colorpicker>
<input type="file" onchange="$set(info.image_file, $event.value)">
```

### `$add(target, value)`

Adds a numeric value to `target`.

```html
<button onclick="$add(info.value, 1)">Increase</button>
<slider value="{{ info.value }}" onchange="$add(info.value, 0.5)"></slider>
```

`$add` accepts numeric literals and numeric strings.

### `$min(target, value)`

Subtracts a numeric value from `target`.

```html
<button onclick="$min(info.value, 1)">Decrease</button>
```

`$min(info.value, 1)` means `info.value = info.value - 1`.

## `$event`

`$event` reads values from the widget that emitted the event.
The most common form is `$event.value`.

```html
<input onchange="$set(player.name, $event.value)">
<slider onchange="$set(info.value, $event.value)"></slider>
```

Supported `$event` fields:

- `$event.value`: primary widget value
- `$event.checked`: checked/selected boolean for checkbox, radio, toggle, and switch widgets
- `$event.selected`: alias-style selected boolean for selectable widgets
- `$event.text`: label/text for selected option-like widgets
- `$event.hex`: color picker value as `#RRGGBB`
- `$event.rgb`: color picker value as `rgb(r, g, b)`
- `$event.rgba`: color picker value as `rgba(r, g, b, a)`
- `$event.red`, `$event.green`, `$event.blue`, `$event.alpha`: color channel numbers

## Value Types

Direct store keys keep their existing Rust primitive type when possible:

- `bool`
- `String`
- unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- signed integers: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- floats: `f32`, `f64`
- `serde_json::Value`

Nested paths such as `info.value` update the template-visible JSON projection of the store value.
This is what makes placeholders like `{{ info.value }}` reactive after the inline event runs.

## Literals

You can pass booleans, numbers, strings, arrays, objects, and `null`.

```html
<button onclick="$set(player.state, true)">Enable</button>
<button onclick="$set(player.name, 'Ada')">Use Ada</button>
<button onclick="$set(info.value, 42)">Set 42</button>
<button onclick="$set(info.tags, ['ui', 'debug'])">Set Tags</button>
```

Both single and double quoted strings are accepted.

## Multiple Actions

Multiple calls can be separated with semicolons.

```html
<button onclick="$set(player.name, 'Ada'); $add(info.value, 1)">
  Apply preset
</button>
```

Use this only for simple changes. If the action starts to encode business logic, move it to `#[html_fn]`.

## Complete Example

```html
@use "crate::data_structs::*";

<div>
  <p>Name: {{ player.name }}</p>
  <input value="{{ player.name }}" onchange="$set(player.name, $event.value)">

  <p>Value: {{ info.value }}</p>
  <button onclick="$min(info.value, 1)">-</button>
  <button onclick="$add(info.value, 1)">+</button>
  <slider value="{{ info.value }}" onchange="$set(info.value, $event.value)"></slider>

  <p>Color: {{ info.color }}</p>
  <colorpicker value="{{ info.color }}" onchange="$set(info.color, $event.hex)"></colorpicker>
</div>
```

## Common Pitfalls

- Do not omit the `$` prefix: `set(...)` is treated as a normal handler name.
- Target aliases use snake case: `PlayerProfile` becomes `player_profile`, not `PlayerProfile` in templates.
- `$add` and `$min` require numeric current and input values.
- Text inputs may trigger reactive rebuilds frequently. For heavy templates, prefer a normal `#[html_fn]` or a commit/blur based flow until debouncing is available.
- Keep complex logic in Rust. Shortland functions are for fast state wiring, not business logic.
