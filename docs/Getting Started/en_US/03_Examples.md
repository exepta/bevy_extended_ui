---
title: Examples
---

# Practical Examples

This page shows an end-to-end flow for:

- Reading values from widgets (`InputField`, `DatePicker`, `CheckBox`, `ChoiceBox`, `Slider`, `ColorPicker`)
- Reading `FieldSet` selections (`Radio` and `Toggle`)
- Reading form submit payloads (`HtmlSubmit`)
- Correct usage of `#[html_fn]`

## 1. HTML with event bindings

```html
<body>
  <input id="email-input" name="email" type="email" onchange="on_any_change" />
  <date-picker id="birthday-input" name="birthday" onchange="on_any_change"></date-picker>
  <checkbox id="tos-checkbox" onchange="on_any_change">Accept terms</checkbox>

  <select id="country-select" onchange="on_any_change">
    <option value="de">Germany</option>
    <option value="us">United States</option>
  </select>

  <slider id="volume-slider" min="0" max="100" step="1" onchange="on_any_change"></slider>
  <colorpicker id="accent-color" onchange="on_any_change"></colorpicker>

  <fieldset id="language-group" mode="single" onchange="on_fieldset_change">
    <radio value="de">Deutsch</radio>
    <radio value="en">English</radio>
  </fieldset>

  <fieldset id="style-group" mode="multi" onchange="on_fieldset_multi_change">
    <toggle value="bold">Bold</toggle>
    <toggle value="italic">Italic</toggle>
    <toggle value="underline">Underline</toggle>
  </fieldset>

  <form id="profile-form" action="on_profile_submit" validate="Send">
    <input name="username" type="text" required />
    <input name="age" type="number" />
    <button type="submit">Save</button>
  </form>
</body>
```

## 2. Rust handlers: read values from widgets

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlChange, HtmlSubmit};
use bevy_extended_ui::styles::CssID;
use bevy_extended_ui::widgets::{
    CheckBox, ChoiceBox, ColorPicker, DatePicker, FieldSelectionMulti, FieldSelectionSingle,
    InputField, InputValue, RadioButton, Slider, SliderType, ToggleButton,
};
use bevy_extended_ui_macros::html_fn;

#[html_fn("on_any_change")]
fn on_any_change(
    In(event): In<HtmlChange>,
    input_q: Query<(&InputField, &InputValue, Option<&CssID>)>,
    date_q: Query<(&DatePicker, &InputValue, Option<&CssID>)>,
    checkbox_q: Query<(&CheckBox, Option<&CssID>)>,
    choice_q: Query<(&ChoiceBox, Option<&CssID>)>,
    slider_q: Query<(&Slider, Option<&CssID>)>,
    color_q: Query<(&ColorPicker, Option<&CssID>)>,
) {
    if let Ok((input, value, css_id)) = input_q.get(event.entity) {
        info!(
            "InputField [{}] name={} text='{}' value='{}'",
            css_id.map(|v| v.0.as_str()).unwrap_or("-"),
            input.name,
            input.text,
            value.0
        );
        return;
    }

    if let Ok((_date, value, css_id)) = date_q.get(event.entity) {
        info!(
            "DatePicker [{}] value='{}'",
            css_id.map(|v| v.0.as_str()).unwrap_or("-"),
            value.0
        );
        return;
    }

    if let Ok((checkbox, css_id)) = checkbox_q.get(event.entity) {
        info!(
            "CheckBox [{}] checked={} label='{}'",
            css_id.map(|v| v.0.as_str()).unwrap_or("-"),
            checkbox.checked,
            checkbox.label
        );
        return;
    }

    if let Ok((choice, css_id)) = choice_q.get(event.entity) {
        info!(
            "ChoiceBox [{}] selected_text='{}' selected_value='{}'",
            css_id.map(|v| v.0.as_str()).unwrap_or("-"),
            choice.value.text,
            choice.value.value_as_str().unwrap_or("<non-string>")
        );
        return;
    }

    if let Ok((slider, css_id)) = slider_q.get(event.entity) {
        match slider.slider_type {
            SliderType::Default => {
                info!(
                    "Slider [{}] value={} ({}..{})",
                    css_id.map(|v| v.0.as_str()).unwrap_or("-"),
                    slider.value,
                    slider.min,
                    slider.max
                );
            }
            SliderType::Range => {
                info!(
                    "Slider [{}] range_start={} range_end={} ({}..{})",
                    css_id.map(|v| v.0.as_str()).unwrap_or("-"),
                    slider.range_start,
                    slider.range_end,
                    slider.min,
                    slider.max
                );
            }
        }
        return;
    }

    if let Ok((color, css_id)) = color_q.get(event.entity) {
        info!(
            "ColorPicker [{}] rgb=({}, {}, {}) hex={}",
            css_id.map(|v| v.0.as_str()).unwrap_or("-"),
            color.red,
            color.green,
            color.blue,
            color.hex()
        );
    }
}
```

## 3. Read `FieldSet` (single/radio)

Important: for `fieldset onchange`, `event.entity` is the `FieldSet` entity.  
Read the selected child via `FieldSelectionSingle`.

```rust
#[html_fn("on_fieldset_change")]
fn on_fieldset_change(
    In(event): In<HtmlChange>,
    fieldset_q: Query<&FieldSelectionSingle>,
    radio_q: Query<&RadioButton>,
) {
    let Ok(selection) = fieldset_q.get(event.entity) else {
        return;
    };

    let Some(selected_radio_entity) = selection.0 else {
        info!("FieldSet: no selection");
        return;
    };

    let Ok(radio) = radio_q.get(selected_radio_entity) else {
        return;
    };

    info!(
        "FieldSet Single: label='{}' value='{}'",
        radio.label,
        radio.value_as_str().unwrap_or("<non-string>")
    );
}
```

## 4. Read `FieldSet` (multi/toggle)

In multi mode, `FieldSelectionMulti` contains all selected toggle entities.

```rust
#[html_fn("on_fieldset_multi_change")]
fn on_fieldset_multi_change(
    In(event): In<HtmlChange>,
    fieldset_q: Query<&FieldSelectionMulti>,
    toggle_q: Query<&ToggleButton>,
) {
    let Ok(selection) = fieldset_q.get(event.entity) else {
        return;
    };

    let mut values = Vec::new();
    for entity in &selection.0 {
        if let Ok(toggle) = toggle_q.get(*entity) {
            values.push(
                toggle
                    .value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| toggle.label.clone()),
            );
        }
    }

    info!("FieldSet Multi: {:?}", values);
}
```

## 5. Read form submit data

`HtmlSubmit` includes all form fields in `event.data` as `HashMap<String, String>`.

```rust
#[html_fn("on_profile_submit")]
fn on_profile_submit(In(event): In<HtmlSubmit>) {
    let username = event.data.get("username").cloned().unwrap_or_default();
    let age = event
        .data
        .get("age")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    info!(
        "Submit action='{}' username='{}' age={}",
        event.action, username, age
    );
}
```

## 6. How to use `#[html_fn]` correctly

`#[html_fn("name")]` registers a Rust function under exactly that HTML handler name.

Rules and practical notes:

1. The string in `#[html_fn("...")]` must exactly match the HTML attribute value (for example `onclick="save_user"`).
2. Use typed events when you need event payload:
   - `HtmlClick`, `HtmlChange`, `HtmlSubmit`, `HtmlKeyDown`, ...
3. Use `HtmlEvent` only when target entity is enough.
4. Handlers should be resilient:
   - `Query::get(event.entity)` with `if let Ok(...)` / `let Ok(...) = ... else { return; }`
5. For `FieldSet`, always read selection via `FieldSelectionSingle` or `FieldSelectionMulti`.

Minimal example:

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui_macros::html_fn;

#[html_fn("save_user")]
fn save_user(In(event): In<HtmlClick>) {
    info!("Click on entity {:?}", event.entity);
}
```
