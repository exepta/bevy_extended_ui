# Widget Documentation

This document describes all UI widget components used in the extended UI system. Each widget is represented as a Bevy component and is designed to be parsed from HTML and rendered dynamically. All widgets use internal ID pools to ensure unique instance IDs and share common dependencies, such as visibility, transform, and event bindings.

---

## 🧱 HtmlBody

The root container for HTML-rendered UI elements.

**Fields:**
- `w_count`: Unique widget ID.
- `bind_to_html`: Optional HTML ID to bind this widget to.
- `source`: Optional `HtmlSource` for origin tracking.

**Css:**
- `body`: widget tag for html and css
---

## 🧱 Div

A generic container element, representing a `<div>` in HTML.

**Fields:**
- `0`: Unique widget ID (newtype).

**Css:**
- `div`: widget tag for html and css
---

## 📝 Headline

Used for rendering headline elements (`<h1>` to `<h6>`).

**Fields:**
- `w_count`: Unique ID.
- `text`: Text to display.
- `h_type`: Headline level (H1–H6).

**Enum:** `HeadlineType`
- Variants: `H1`, `H2`, `H3`, `H4`, `H5`, `H6`

**Css:**
- `h1 - h6`: widget tag for html and css
---

## 📄 Paragraph

Represents a simple paragraph of text (`<p>`).

**Fields:**
- `w_count`: Unique ID.
- `text`: Paragraph content.

**Css:**
- `p`: widget tag for html and css
---

## 🔘 Button

Represents a clickable button element.

**Fields:**
- `w_count`: Unique ID.
- `text`: Display text.
- `icon_place`: Icon placement relative to text.
- `icon_path`: Optional icon asset path.

**Css:**
- `button`: widget tag for html and css
- `.button-text`: for the inner text and icon
---

## ☑️ CheckBox

Checkbox input with an optional label and icon.

**Fields:**
- `w_count`: Unique ID.
- `label`: Display label.
- `icon_path`: Icon path for checked state (default provided).

**Css:**
- `checkbox`: widget tag for html and css
- `.mark-box`: container for the `mark` icon
- `.check-text`: the label of checkbox
- `.mark`: the checked icon
---

## 🎚️ Slider

Numeric input using a slider UI element.

**Fields:**
- `w_count`: Unique ID.
- `value`: Current value.
- `step`: Step size.
- `min`, `max`: Range bounds.

**Css:**
- `slider`: widget tag for html and css
- `.track`: the filled slider bar
- `.thumb`: the slider thumb
---

## ✏️ InputField

Text input widget with extended features.

**Fields:**
- `w_count`: Unique ID.
- `text`: Current input.
- `label`: Input label.
- `placeholder`: Placeholder text.
- `cursor_position`: Position of the text cursor.
- `clear_after_focus_lost`: Whether to clear on blur.
- `icon_path`: Optional icon.
- `input_type`: Type of input (`Text`, `Password`, `Number`).
- `cap_text_at`: Input cap behavior.

**Enum:** `InputType`
- Variants: `Text`, `Password`, `Number`

**Enum:** `InputCap`
- Variants:
  - `NoCap`: No character limit.
  - `CapAtNodeSize`: Limit by layout node.
  - `CapAt(usize)`: Explicit character cap.

**Css:**
- `input`: widget tag for html and css
- `.in-icon-container`: the left icon container
- `.in-icon`: icon from the container itself
- `.input-label`: input label which move
- `.in-text-container`: the text holder container
- `.input-cursor`: the text cursor
- `.input-text`: input text which displayed
---

## 🔽 ChoiceBox

Dropdown selection widget.

**Fields:**
- `w_count`: Unique ID.
- `label`: Display label.
- `value`: Currently selected option.
- `options`: List of selectable options.
- `icon_path`: Optional icon for dropdown.

**Struct:** `ChoiceOption`
- `text`: Display text.
- `internal_value`: Backend value.
- `icon_path`: Optional icon.

**Css:**
- `select`: widget tag for html and css
- `.select-label`: the outer label
- `.choice-content-box`: drop down box which the options
- `.choice-option`: the option card
- `.option-icon`: manipulated the icon from the option only
- `.option-text`: manipulated the text and the icon
- `.option-selected`: the current selected (displayed at top) option card
- `.option-sel-text`: manipulated the text
- `.option-drop-box`: icon holder from the dropdown icon
- `.option-drop-icon`: the dropdown icon itself
---

## 🖼️ Img

Represents an image element (`<img>`).

**Fields:**
- `w_count`: Unique ID.
- `src`: Image path.
- `alt`: Alt text for accessibility.

**Css:**
- `img`: widget tag for html and css
---

## 📊 ProgressBar

Visualizes progress of a task or value.

**Fields:**
- `w_count`: Unique ID.
- `value`: Current progress value.
- `max`, `min`: Value bounds.

**Css:**
- `progressbar`: widget tag for html and css
- `.progress`: the current show progress
---

All widgets implement `Default`, generate a unique internal `w_count`, and are expected to work with Bevy’s ECS-based UI system. Event bindings, visibility, and transforms are handled automatically via required component dependencies.

