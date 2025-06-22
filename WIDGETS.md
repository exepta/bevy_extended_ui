# Widget Documentation

This document describes all UI widget components used in the extended UI system. Each widget is represented as a Bevy component and is designed to be parsed from HTML and rendered dynamically. All widgets use internal ID pools to ensure unique instance IDs and share common dependencies, such as visibility, transform, and event bindings.

---

## 🧱 HtmlBody

The root container for HTML-rendered UI elements.

**Fields:**
- `w_count`: Unique widget ID.
- `bind_to_html`: Optional HTML ID to bind this widget to.
- `source`: Optional `HtmlSource` for origin tracking.

---

## 🧱 Div

A generic container element, representing a `<div>` in HTML.

**Fields:**
- `0`: Unique widget ID (newtype).

---

## 📝 Headline

Used for rendering headline elements (`<h1>` to `<h6>`).

**Fields:**
- `w_count`: Unique ID.
- `text`: Text to display.
- `h_type`: Headline level (H1–H6).

**Enum:** `HeadlineType`
- Variants: `H1`, `H2`, `H3`, `H4`, `H5`, `H6`

---

## 📄 Paragraph

Represents a simple paragraph of text (`<p>`).

**Fields:**
- `w_count`: Unique ID.
- `text`: Paragraph content.

---

## 🔘 Button

Represents a clickable button element.

**Fields:**
- `w_count`: Unique ID.
- `text`: Display text.
- `icon_place`: Icon placement relative to text.
- `icon_path`: Optional icon asset path.

---

## ☑️ CheckBox

Checkbox input with an optional label and icon.

**Fields:**
- `w_count`: Unique ID.
- `label`: Display label.
- `icon_path`: Icon path for checked state (default provided).

---

## 🎚️ Slider

Numeric input using a slider UI element.

**Fields:**
- `w_count`: Unique ID.
- `value`: Current value.
- `step`: Step size.
- `min`, `max`: Range bounds.

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

---

## 🖼️ Img

Represents an image element (`<img>`).

**Fields:**
- `w_count`: Unique ID.
- `src`: Image path.
- `alt`: Alt text for accessibility.

---

## 📊 ProgressBar

Visualizes progress of a task or value.

**Fields:**
- `w_count`: Unique ID.
- `value`: Current progress value.
- `max`, `min`: Value bounds.

---

All widgets implement `Default`, generate a unique internal `w_count`, and are expected to work with Bevy’s ECS-based UI system. Event bindings, visibility, and transforms are handled automatically via required component dependencies.

