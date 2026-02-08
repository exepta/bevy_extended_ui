# CSS Usage (Parser Reference)

This document mirrors what `src/styles/parser.rs` actually parses and applies. Unsupported
properties or values are ignored silently.

## Variables

- Define CSS variables only in `:root` using `--name: value`.
- Use them as `var(--name)` with no fallback support.
- The parser only resolves the exact `var(...)` form; nested or chained vars are not resolved.

## Supported Properties

### Box Model / Sizing

- `width`, `min-width`, `max-width`
- `height`, `min-height`, `max-height`
- `padding`, `padding-left`, `padding-right`, `padding-top`, `padding-bottom`
- `margin`, `margin-left`, `margin-right`, `margin-top`, `margin-bottom`
- `border`, `border-left`, `border-right`, `border-top`, `border-bottom`
- `border-width`
- `border-radius`
- `box-sizing` (`border-box`, `content-box`; defaults to `border-box`)

### Layout / Positioning

- `display` (`flex`, `grid`, `block`, `none`; defaults to `block`)
- `position` (`relative`, `absolute`; defaults to `relative`)
- `left`, `right`, `top`, `bottom`
- `gap`
- `overflow`, `overflow-x`, `overflow-y`

### Flex

- `justify-content`
- `align-items`
- `flex-direction`
- `flex-grow`
- `flex-shrink`
- `flex-basis`
- `flex-wrap`

### Grid

- `grid-row`, `grid-column`
- `grid-auto-flow`
- `grid-template-rows`, `grid-template-columns`
- `grid-auto-rows`, `grid-auto-columns`

### Typography

- `font-size`
- `font-family`
- `font-weight`
- `text-wrap`

### Visuals

- `color`
- `background-color`
- `background`
- `background-image`
- `border-color`
- `box-shadow`

### Interaction / Misc

- `pointer-events`
- `z-index`
- `scroll-width`

### Transforms

- `transform`

### Animations / Transitions

- `transition`
- `animation`
- `animation-name`
- `animation-duration`
- `animation-delay`
- `animation-timing-function`
- `animation-iteration-count`
- `animation-direction`

## Value Rules and Parsing Details

### Lengths (`Val`)

Used by width/height, padding/margin, border widths, translation, etc.

- Supported units: `px`, `%`
- Also supports `0` / `0.0` (treated as `0px`)
- Anything else is ignored

### Font Size (`font-size`)

- `px` -> `FontVal::Px`
- `rem` -> `FontVal::Rem`

### Colors

- Named colors (via `Colored::named`)
- Hex (`#rrggbb`, `#rrggbbaa`)
- `rgb(r,g,b)`
- `rgba(r,g,b,a)` where all parts are `0..255`
- `transparent` or `none` -> transparent

### Background

- `background`: supports `url("...")` or a color
- `background-image`: supports only `url("...")`
- `background-color`: color only

### Border

- `border`: expects `WIDTH [COLOR]` (style keywords are ignored)
- `border-left/right/top/bottom`: width only (e.g. `2px`)
- `border-width`: shorthand like `padding` (1-4 values)
- `border-color`: color only
- `border-radius`: 1-4 values in `px` or `%`

### Padding / Margin Shorthand

Accepted forms are 1-4 values (px, %, or 0). Mapping is:

- 1 value: all sides
- 2 values: top/bottom = first, left/right = second
- 3 values: left = first, right = second, top = third, bottom = 0
- 4 values: left, right, top, bottom

Note: The 3-value form is non-standard and sets bottom to `0`.

### Border Radius Shorthand

Accepted forms are 1-4 values (px, %, or 0). Mapping is:

- 1 value: all corners
- 2 values: top-left/top-right = first, bottom-right/bottom-left = second
- 3 values: top-left = first, top-right = second, bottom-left = third, bottom-right = 0
- 4 values: top-left, top-right, bottom-right, bottom-left

Note: The 3-value form is non-standard and sets bottom-right to `0`.

### Box Shadow

Single shadow only. Syntax: up to 4 size values plus optional color.

- Sizes support `px`, `%`, or `0`
- 1 value: x, y, blur, spread all same
- 2 values: x, y; blur/spread = 0
- 3 values: x, y, blur; spread = 0
- 4 values: x, y, blur, spread
- Color can be `#`, `rgb(...)`, or `rgba(...)`
- `inset` is not supported

### Transform

`transform` parses a list of functions. Supported functions:

- `translate(x y)` or `translation(x y)`
- `translateX(x)`
- `translateY(y)`
- `scale(s)` or `scale(x y)`
- `scaleX(s)`
- `scaleY(s)`
- `rotate(10deg)` / `rotate(0.5rad)` (numeric values are treated as degrees)

Translations use `px`, `%`, or `0`. Scale is unitless `f32`.

### Overflow

`overflow`, `overflow-x`, `overflow-y` support:

- `hidden`, `scroll` (or `auto`), `clip`, `visible`

### Pointer Events

- `pointer-events: none` disables picking
- any other value uses default pick behavior

### Text Wrap

`text-wrap` maps to Bevy `LineBreak`:

- `wrap`, `stable` -> word/character break
- `nowrap` -> no wrap
- `pretty`, `balance` -> word boundary
- `unset` -> any character

### Z-Index

- Integer only (e.g. `0`, `10`, `-1`)

### Scroll Width

- `scroll-width` expects a float (e.g. `12`, `8.5`)

## Transitions

`transition` supports:

- Properties: `all`, `color`, `background` / `background-color`, `transform`
- Timing functions: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`
- Durations: `ms` or `s` (first time value is duration, second is delay)

Example:

```css
.card {
  transition: transform 0.25s ease-in-out 50ms;
}
```

Only the properties above are respected during transitions.

## Animations

`@keyframes` declarations are parsed with the same property support as regular styles.

Shorthand `animation` supports:

- name, duration, delay, timing function, iteration count, direction
- directions: `normal`, `reverse`, `alternate`, `alternate-reverse`
- iteration count: `infinite` or a number

Notes:

- Only the first comma-separated animation is parsed.
- `animation-name: none` clears the animation.

Example:

```css
@keyframes pulse {
  0% { transform: scale(1); }
  100% { transform: scale(1.05); }
}

.btn {
  animation: pulse 1.2s ease-in-out infinite alternate;
}
```

## Grid Details

`grid-row` / `grid-column` accept:

- `span N`
- `start/end` (e.g. `1 / 3`)
- a single start index (e.g. `2`)

`grid-template-rows` / `grid-template-columns` accept:

- `repeat(count, track)` (single track only)
- or a list of tracks separated by spaces

Tracks support:

- `auto`, `min-content`, `max-content`
- `minmax(min, max)`
- `px`, `%`, `fr`

`grid-auto-rows` / `grid-auto-columns` accept a list of tracks (space-separated).

## Limitations

- Unsupported properties and values are ignored silently.
- Many CSS shorthands and keywords are not implemented.
- Only a single `box-shadow` is supported.
- `border` ignores style keywords (only width + optional color).
- `animation` only reads the first comma-separated segment.
