---
title: Beispiele
---

# Praxisbeispiele

Diese Seite zeigt einen vollständigen Ablauf für:

- Werte aus Widgets auslesen (`InputField`, `DatePicker`, `CheckBox`, `ChoiceBox`, `Slider`, `ColorPicker`)
- Selektionen aus `FieldSet` lesen (`Radio` und `Toggle`)
- Form-Submit-Daten lesen (`HtmlSubmit`)
- Event-Handler mit `#[html_fn]` korrekt binden

## 1. HTML mit Event-Bindings

```html
<body>
  <input id="email-input" name="email" type="email" onchange="on_any_change" />
  <date-picker id="birthday-input" name="birthday" onchange="on_any_change"></date-picker>
  <checkbox id="tos-checkbox" onchange="on_any_change">AGB akzeptieren</checkbox>

  <select id="country-select" onchange="on_any_change">
    <option value="de">Deutschland</option>
    <option value="us">USA</option>
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
    <button type="submit">Speichern</button>
  </form>
</body>
```

## 2. Rust-Handler: Werte aus Widgets lesen

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

## 3. FieldSet auslesen (Single/Radio)

Wichtig: Bei `onchange` von `fieldset` ist `event.entity` das `FieldSet`-Entity.  
Die eigentliche Auswahl liest du über `FieldSelectionSingle`.

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
        info!("FieldSet: keine Auswahl");
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

## 4. FieldSet auslesen (Multi/Toggle)

Bei Multi-Mode enthält `FieldSelectionMulti` alle aktuell selektierten Toggle-Entities.

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

## 5. Form-Submit-Daten lesen

`HtmlSubmit` liefert alle Form-Felder als `HashMap<String, String>` in `event.data`.

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

## 6. `#[html_fn]` richtig verwenden

`#[html_fn("name")]` registriert eine Rust-Funktion unter genau diesem Namen im HTML-Event-System.

Regeln und Praxis:

1. Der String in `#[html_fn("...")]` muss exakt zum HTML-Attribut passen (z. B. `onclick="save_user"`).
2. Nutze typisierte Events, wenn du Event-Daten brauchst:
   - `HtmlClick`, `HtmlChange`, `HtmlSubmit`, `HtmlKeyDown`, ...
3. Nutze `HtmlEvent` nur, wenn dir das Ziel-Entity reicht.
4. Handler sollten robust sein:
   - `Query::get(event.entity)` mit `if let Ok(...)` / `let Ok(...) = ... else { return; }`
5. Bei `FieldSet` immer über `FieldSelectionSingle` oder `FieldSelectionMulti` lesen, nicht direkt nur über das Child-Widget.

Kleines Minimalbeispiel:

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui_macros::html_fn;

#[html_fn("save_user")]
fn save_user(In(event): In<HtmlClick>) {
    info!("Klick auf Entity {:?}", event.entity);
}
```
