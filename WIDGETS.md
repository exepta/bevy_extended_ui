# Widgets

This document describes all widgets in the project, their corresponding Rust structs, and how they appear in HTML source.

---

## Shared Inner Content Attributes

All spawned HTML widgets now receive a `HtmlInnerContent` component with:

- `innerText`: raw text content of the element.
- `innerHtml`: serialized inner HTML.
- `innerBindings`: detected placeholders like `{{user.name}}`.

All three fields can be overridden at runtime via setters:

- `set_inner_text(...)`
- `set_inner_html(...)`
- `set_inner_bindings(...)`

---

## Body (`Body`)

**Struct purpose:** Root container for an HTML structure. Holds an internal `entry` ID and an optional `html_key` (the `<meta name="...">` key of the HTML file).

**HTML tag:**
```html
<body>
  <!-- child widgets -->
</body>
```

---

## Div (`Div`)

**Struct purpose:** Generic container for layout and grouping; only stores an internal ID.

**HTML tag:**
```html
<div>
  <!-- child widgets -->
</div>
```

---

## Form (`Form`)

**Struct purpose:** Container for form controls. Supports `action="handler_name"` to trigger a submit handler via `#[html_fn("handler_name")]`.

Validation is only active inside `<form>`.
Use `validate="Allways|Always|Send|Interact"` on the form (default: `Send`):
- `Allways`/`Always`: validates continuously (state/input changes).
- `Send`: validates only on submit click.
- `Interact`: validates on input interaction (e.g. typing).

When a child `<button type="submit">` is clicked, the form:
- validates descendants with `validation`/`required` rules,
- collects input data into a submit payload (`data` map),
- calls the `action` handler only if validation passes.

**HTML tag:**
```html
<form action="login_action" validate="Send">
  <input name="username" required />
  <input name="email" type="email" required />
  <button type="submit">Login</button>
</form>
```

---

## Button (`Button`)

**Struct purpose:** Clickable button with text plus an optional icon and its placement. Supports `type="button|submit|reset"` (`Auto` when omitted).

Inside a `<form>`, use `type="submit"` to submit the parent form without `onclick`.

**HTML tag:**
```html
<button>
  Text
  <icon src="path/to/icon.png"></icon>
</button>
```

---

## CheckBox (`CheckBox`)

**Struct purpose:** Checkbox with label, optional icon, and a `checked` state.

**HTML tag:**
```html
<checkbox icon="extended_ui/icons/check-mark.png">Label</checkbox>
```

---

## ChoiceBox / Select (`ChoiceBox`)

**Struct purpose:** Dropdown selection with a label, current value (`value`), and a list of options.

**HTML tag:**
```html
<select>
  <option value="a" selected>Option A</option>
  <option value="b">Option B</option>
</select>
```

---

## Divider (`Divider`)

**Struct purpose:** Separator line whose alignment can be vertical or horizontal.

**HTML tag:**
```html
<divider alignment="horizontal"></divider>
```

---

## FieldSet (`FieldSet`)

**Struct purpose:** Group container for selection widgets (e.g., `<radio>` or `<toggle>`). Controls the selection mode (single/multi) and whether “no selection” is allowed.

**HTML tag:**
```html
<fieldset mode="single" allow-none="false">
  <radio value="a" selected>Option A</radio>
  <radio value="b">Option B</radio>
</fieldset>
```

---

## Headline (`Headline`)

**Struct purpose:** Heading with text and type (H1–H6).

**HTML tag:**
```html
<h1>Headline</h1>
```

---

## Image (`Img`)

**Struct purpose:** Image widget with `src` and `alt` text.

**HTML tag:**
```html
<img src="path/to/image.png" alt="Description" />
```

---

## InputField (`InputField`)

**Struct purpose:** Text input with `name`, label, placeholder, icon, type (text/email/date/password/number), and length limit.

**HTML tag:**
```html
<label for="name">Name</label>
<input id="name" name="name" type="text" placeholder="Your name" maxlength="32" />
```

---

## Paragraph (`Paragraph`)

**Struct purpose:** Paragraph with free-form text.

**HTML tag:**
```html
<p>This is a paragraph.</p>
```

---

## ProgressBar (`ProgressBar`)

**Struct purpose:** Progress indicator with `min`, `max`, and `value`.

**HTML tag:**
```html
<progressbar min="0" max="100" value="42"></progressbar>
```

---

## RadioButton (`RadioButton`)

**Struct purpose:** Single radio button with label, `value`, and `selected` state. Can be used directly or inside a `<fieldset>`.

**HTML tag:**
```html
<radio value="choice" selected>Choice</radio>
```

---

## Scrollbar (`Scrollbar`)

**Struct purpose:** Scrollbar for vertical or horizontal scrolling with min/max/step and current value.

**HTML tag:**
```html
<scroll alignment="vertical"></scroll>
```

---

## Slider (`Slider`)

**Struct purpose:** Slider for numeric input with `min`, `max`, `value`, and `step`.

**HTML tag:**
```html
<slider min="0" max="100" value="50" step="5"></slider>
```

---

## ColorPicker (`ColorPicker`)

**Struct purpose:** Canvas-based color selection widget with live `HEX`, `RGB`, and `RGBA` output.
Supports optional initial `value` (`#hex`, `rgb(...)`, `rgba(...)`) and optional `alpha`.

**HTML tag:**
```html
<colorpicker value="#4285f4" alpha="255" onchange="on_color_change"></colorpicker>
```

---

## SwitchButton (`SwitchButton`)

**Struct purpose:** Switch widget with a label and optional icon.

**HTML tag:**
```html
<switch icon="path/to/icon.png">On/Off</switch>
```

---

## ToggleButton (`ToggleButton`)

**Struct purpose:** Toggleable button with label, `value`, icon, and `selected` state.

**HTML tag:**
```html
<toggle value="flag" selected>
  Toggle Text
  <icon src="path/to/icon.png"></icon>
</toggle>
```
