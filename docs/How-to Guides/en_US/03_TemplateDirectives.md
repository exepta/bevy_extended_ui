---
title: Template Directives
---

# How-to: Template Directives (`@use`, `@if`, `@else`, `@for`)

This guide explains the template directives that are evaluated before HTML parsing.
Use them to import shared Rust data into templates, evaluate conditions, and render lists.

## When should you use template directives?

Use template directives when you want to:

- show or hide UI blocks based on data
- render lists from JSON/Rust data
- use `#[html_shared]` or `#[html_use]` values in HTML
- keep imports explicit and readable in the template

Important: directives run before DOM parsing. They produce regular HTML, which is then processed by the converter.

## 1) Provide data to templates

Simple runtime variables can be set through `UiLangVariables`. Values are parsed as JSON when possible.

```rust
use bevy::prelude::*;
use bevy_extended_ui::UiLangVariables;

fn setup_vars(mut vars: ResMut<UiLangVariables>) {
    vars.set("enabled", "true");
    vars.set("data", r#"{"users":[{"name":"Ada"},{"name":"Bob"}]}"#);
}
```

For typed Rust data, mark resources with `#[html_shared]` and `Serialize`.

```rust
use bevy::prelude::*;
use bevy_extended_ui_macros::html_shared;
use serde::Serialize;

#[html_shared]
#[derive(Resource, Serialize)]
pub struct DataPack {
    pub version: String,
    pub used: bool,
}

#[html_shared]
#[derive(Resource, Serialize)]
pub enum DataState {
    Active,
    Inactive,
}
```

When the resource exists in the Bevy world, it becomes available to templates.

## 2) Use `@use`

`@use` imports shared values into the template context.
Every `@use` line needs a semicolon.

### Import one type

```html
@use "crate::data_structs::DataPack";

<p>Version: {{ data_pack.version }}</p>
```

Without an explicit alias, the type name is converted into a template alias automatically:

- `DataPack` -> `data_pack`
- `DataState` -> `data_state`
- `PlayerProfile` -> `player_profile`

### Add an alias

```html
@use "crate::data_structs::DataState" as state;

@if(state == DataState::Inactive) {
  <p>Inactive</p>
}
```

### Import multiple types with grouped syntax

```html
@use "crate::data_structs::{DataState, DataPack}";

<p>{{ data_pack.version }}</p>
<p>{{ data_state }}</p>
```

### Add an alias inside a group

```html
@use "crate::data_structs::{DataState as current_state, DataPack}";

@if(current_state == DataState::Inactive) {
  <p>Inactive</p>
}
```

### Import all shared types from a path

```html
@use "crate::data_structs::*";

<p>{{ data_pack.version }}</p>
<p>{{ data_state }}</p>
```

With `path::*`, every matching type is imported with its default alias.
`as` is not used in this form.

Do not use:

```html
@use "crate::data_structs::*" as data;
```

### Import fields from one struct value directly

`as *` is different from `path::*`: it imports fields from one struct/object value directly into the context.

```html
@use "Player" as *;

<p>{{ name }}</p>
@if(state) {
  <p>Online</p>
}
```

Use `as *` sparingly, because field names can easily collide with other variables.

## 3) Use `@if` and `@else`

`@if` renders a block only when the condition is true.
`@else` is optional.

```html
@if(enabled) {
  <button>Start</button>
} @else {
  <p>Disabled</p>
}
```

You can use object paths, negation, comparisons, and logical operators.

```html
@use "crate::data_structs::*";

@if(!data_pack.used && data_state == DataState::Inactive) {
  <p>Ready</p>
} @else {
  <p>Already used or active</p>
}
```

Supported expressions:

- boolean values: `enabled`, `!enabled`
- object paths: `data.user.name`, `data_pack.version`
- equality: `state == "active"`, `data_state == DataState::Inactive`
- logic: `&&`, `||`
- parentheses: `(a || b) && c`
- methods on strings/lists: `startsWidth`, `startsWith`, `endsWidth`, `endsWith`, `contains`

Example:

```html
@if(data.username.startsWith("Net") && enabled) {
  <p>Allowed</p>
}
```

## 4) Use `@for`

`@for` renders one block for each item in a list.

```html
@for(user in data.users) {
  <p>{{ user.name }}</p>
}
```

With an index:

```html
@for(user, index in data.users) {
  <p>{{ index }}: {{ user.name }}</p>
}
```

`@for` can be combined with `@if`.

```html
@for(user, index in data.users) {
  @if(user.active || index == 0) {
    <p>{{ index }} - {{ user.name }}</p>
  }
}
```

Zero-argument getter methods work for serialized fields when they follow the `get_<field_name>` pattern.

```html
@use "crate::data_structs::DataPack";

@for(entry, index in data_pack.get_data()) {
  <p>Byte {{ index }}: {{ entry }}</p>
}
```

## 5) Complete example

```html
@use "crate::data_structs::*";

<div id="main">
  @if(data_state == DataState::Inactive && !data_pack.used) {
    <h3>Data Pack {{ data_pack.version }}</h3>
  } @else {
    <h3>Data unavailable</h3>
  }

  @for(entry, index in data_pack.get_data()) {
    <p>Byte {{ index }}: {{ entry }}</p>
  }
</div>
```

## 6) Common pitfalls

1. Every `@use` line needs a semicolon.
2. `@use "path::*"` imports types with default aliases; `as` is not meaningful there.
3. `@use "Type" as *` imports fields from one object value, not all types from a module.
4. Use aliases in the template, for example `data_state`, not `DataState`.
5. Enum variants are compared as literals, for example `DataState::Inactive`.
6. Directives must produce valid HTML after they are evaluated.
