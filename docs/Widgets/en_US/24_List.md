---
title: List
---

# List

### Overview

List widget based on `<listbox>` where all options stay visible at the same time.

- Rust component: ListBox
- HTML tag: listbox
- Recommended source reference: src/widgets/mod.rs

### Attributes

Important widget-specific attributes (detailed):

- `multiselect` on `<listbox>` enables multi-selection.
- Without `multiselect`, only one option can be active at a time.
- `<option value="...">` sets the internal option value.
- `<option selected>` marks initially selected entries.
- `<option icon="...">` sets an optional icon for each entry.
- `<option internal-value-type="...">` sets the target type for parsed values.

Supported global HTML attributes:

- `id`: Unique id for CSS selectors, event mapping, and widget references.
- `class`: Passes CSS classes for visual styling and state-dependent rules.
- `style`: Passes inline CSS that is parsed into `HtmlStyle` and applied in the style pipeline.
- `hidden`: Renders the widget initially hidden.
- `disabled`: Disables interactions; clicks and focus changes are blocked.
- `readonly`: Is applied as widget state to keep interaction behavior consistent.
- Event attributes like `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, and `onkeyup`: Bind handler functions directly to the event binding system.

### WASM Previews

### List Example
<iframe
  id="listbox"
  title="List"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<listbox id="difficulty" onchange="log_listbox">
  <option value="easy">Easy</option>
  <option value="normal" selected>Normal</option>
  <option value="hard">Hard</option>
</listbox>
```

#### Rust Example

```rust
fn spawn_list_widget(mut commands: Commands) {
    let easy = ChoiceOption::new("Easy").with_value(String::from("easy"));
    let normal = ChoiceOption::new("Normal").with_value(String::from("normal"));
    let hard = ChoiceOption::new("Hard").with_value(String::from("hard"));

    commands.spawn((
        ListBox {
            options: vec![easy.clone(), normal.clone(), hard.clone()],
            values: vec![normal],
            multiselect: false,
            ..default()
        },
        Node::default(),
    ));
}
```

### Multi List Example

Use `multiselect` when the user should be able to keep multiple entries active at the same time.
In Rust, `ListBox::values` contains every selected `ChoiceOption`.

<iframe
  id="listbox-multi"
  title="Multi List"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<listbox id="music-tags" multiselect onchange="log_listbox_multi">
  <option value="rock" selected>Rock</option>
  <option value="jazz" selected>Jazz</option>
  <option value="pop">Pop</option>
  <option value="electro">Electro</option>
  <option value="blues">Blues</option>
</listbox>
```

#### Rust Example

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlChange;
use bevy_extended_ui::widgets::{ChoiceOption, ListBox};
use bevy_extended_ui_macros::html_fn;

fn spawn_multi_list_widget(mut commands: Commands) {
    let rock = ChoiceOption::new("Rock").with_value(String::from("rock"));
    let jazz = ChoiceOption::new("Jazz").with_value(String::from("jazz"));
    let pop = ChoiceOption::new("Pop").with_value(String::from("pop"));
    let electro = ChoiceOption::new("Electro").with_value(String::from("electro"));
    let blues = ChoiceOption::new("Blues").with_value(String::from("blues"));

    commands.spawn((
        ListBox {
            options: vec![
                rock.clone(),
                jazz.clone(),
                pop,
                electro,
                blues,
            ],
            values: vec![rock, jazz],
            multiselect: true,
            ..default()
        },
        Node::default(),
    ));
}

#[html_fn("log_listbox_multi")]
fn log_listbox_multi(In(event): In<HtmlChange>, list_q: Query<&ListBox>) {
    let Ok(list) = list_q.get(event.entity) else {
        return;
    };

    let values: Vec<String> = list
        .values
        .iter()
        .map(|option| {
            option
                .value_as_str()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| option.text.clone())
        })
        .collect();

    info!("selected list values: {:?}", values);
}
```

### Widget Creator

<div style="display: flex; align-items: center; justify-content: flex-start; padding: 15px; border: 1px solid #5658db; border-radius: 10px; gap: 15px; width: 300px;">
  <img
    src="https://avatars.githubusercontent.com/shadow-wolftousen"
    alt="shadow-wolftousen avatar"
    width="64"
    height="64"
    style="width: 64px; height: 64px; border-radius: 50%; object-fit: cover;"
  />
  <div style="display: flex; flex-direction: column; align-items: flex-start; justify-content: center;">
    <strong>shadow-wolftousen</strong>
    <a href="https://github.com/shadow-wolftousen" style="margin-top: 10px; color: #5658db;">Link to GitHub</a>
  </div>
</div>
