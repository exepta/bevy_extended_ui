use crate::styles::paint::Colored;
use crate::styles::{Background, FontVal, Radius, Style};
use bevy::prelude::*;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::traits::ToCss;
use regex::Regex;
use std::collections::HashMap;

/// Loads a CSS file and parses it into a [`HashMap`] of selectors to [`Style`] structs.
///
/// This function reads a `.css` file from the disk, parses its rules, and converts each supported
/// CSS property into a Bevy-compatible [`Style`] representation. These styles can later be applied
/// to UI nodes.
///
/// # Parameters
/// - `path`: A path to the CSS file. If empty, it falls back to the default path `"assets/internal.css"`.
///
/// # Returns
/// - `Ok(HashMap<selector, Style>)` on success, where each key is a full CSS selector (e.g. `#login:hover`)
/// - `Ok(empty map)` if the file cannot be read or parsed, but no panic occurs
/// - `Err(...)` is reserved for future use (currently always returns `Ok(...)`)
///
/// # Example
/// ```rust
/// use bevy_extended_ui::styling::css::load_css;
/// let styles = load_css("assets/theme.css").unwrap();
/// let button_style = styles.get(".button:hover");
/// ```
///
/// # Notes
/// - This uses [`grass_compiler::StyleSheet`] to parse the file.
/// - Supports standard properties like `width`, `height`, `padding`, `color`, `background`, `font-size`, `z-index`, etc.
/// - Ignores unsupported or malformed declarations silently.
pub fn load_css(css: &str) -> HashMap<String, Style> {
    let stylesheet = match StyleSheet::parse(css, ParserOptions::default()) {
        Ok(stylesheet) => stylesheet,
        Err(err) => {
            error!("Css Parsing failed: {:?}", err);
            return HashMap::new();
        }
    };

    let mut css_vars = HashMap::new();
    let mut style_map = HashMap::new();

    for rule in &stylesheet.rules.0 {
        if let CssRule::Style(style_rule) = rule {
            let selector = match style_rule
                .selectors
                .to_css_string(PrinterOptions::default())
            {
                Ok(s) => s,
                Err(_) => continue,
            };

            if selector.trim() == ":root" {
                for property in &style_rule.declarations.declarations {
                    let property_id = property.property_id();
                    let name = property_id.name();

                    if name.starts_with("--") {
                        if let Ok(value) = property.value_to_css_string(PrinterOptions::default()) {
                            css_vars.insert(name.to_string(), value);
                        }
                    }
                }
                continue;
            }

            let mut style = Style::default();
            for property in &style_rule.declarations.declarations {
                let property_id = property.property_id();
                let name = property_id.name();

                let value = match property.value_to_css_string(PrinterOptions::default()) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let mut resolved = value.clone();
                if let Some(var_name) = value.strip_prefix("var(").and_then(|s| s.strip_suffix(')'))
                {
                    if let Some(var_value) = css_vars.get(var_name.trim()) {
                        resolved = var_value.clone();
                    }
                }

                apply_property_to_style(&mut style, name, &resolved);
            }

            style_map.insert(selector, style);
        }
    }

    style_map
}

/// Applies a single CSS property to a mutable [`Style`] object.
///
/// Converts CSS property names and values into Bevy UI values where possible. Also supports
/// compound properties like `padding`, `background`, `border`, `overflow`, and flex/grid layouts.
///
/// # Parameters
/// - `style`: The [`Style`] object to mutate.
/// - `name`: The CSS property name (e.g. `"width"`, `"background-color"`).
/// - `value`: The CSS value as a string (e.g. `"100px"`, `"red"`, `"center"`).
///
/// # Supported Properties
/// - Box model: `width`, `height`, `padding`, `margin`, `border`, `border-radius`
/// - Colors: `color`, `background-color`, `border-color`
/// - Flex/grid layout: `display`, `position`, `flex-grow`, `flex-direction`, `grid-template-columns`, etc.
/// - Visuals: `background`, `background-image`, `box-shadow`, `z-index`, `overflow`, etc.
/// - Typography: `font-size`, `text-wrap`
///
/// # Behavior
/// - If a property or value is unsupported or invalid, it is silently ignored.
/// - Some shorthand values (e.g. `padding-left`) are expanded into full [`UiRect`]s or `Val`s.
///
/// # Example
/// ```rust
/// use bevy_extended_ui::styling::css::apply_property_to_style;
/// use bevy_extended_ui::styling::Style;
/// let mut style = Style::default();
/// apply_property_to_style(&mut style, "padding", "10px 20px");
/// apply_property_to_style(&mut style, "color", "white");
/// ```
pub fn apply_property_to_style(style: &mut Style, name: &str, value: &str) {
    match name {
        "width" => style.width = convert_to_val(value.to_string()),
        "min-width" => style.min_width = convert_to_val(value.to_string()),
        "max-width" => style.max_width = convert_to_val(value.to_string()),
        "height" => style.height = convert_to_val(value.to_string()),
        "min-height" => style.min_height = convert_to_val(value.to_string()),
        "max-height" => style.max_height = convert_to_val(value.to_string()),

        "padding" => style.padding = convert_to_ui_rect(value.to_string()),
        "padding-left" => style.padding = convert_to_ui_rect(format!("{} 0 0 0", value)),
        "padding-right" => style.padding = convert_to_ui_rect(format!("0 {} 0 0", value)),
        "padding-top" => style.padding = convert_to_ui_rect(format!("0 0 {} 0", value)),
        "padding-bottom" => style.padding = convert_to_ui_rect(format!("0 0 0 {}", value)),

        "margin" => style.margin = convert_to_ui_rect(value.to_string()),
        "margin-left" => style.margin = convert_to_ui_rect(format!("{} 0 0 0", value)),
        "margin-right" => style.margin = convert_to_ui_rect(format!("0 {} 0 0", value)),
        "margin-top" => style.margin = convert_to_ui_rect(format!("0 0 {} 0", value)),
        "margin-bottom" => style.margin = convert_to_ui_rect(format!("0 0 0 {}", value)),

        "color" => style.color = convert_to_color(value.to_string()),

        "left" => style.left = convert_to_val(value.to_string()),
        "right" => style.right = convert_to_val(value.to_string()),
        "top" => style.top = convert_to_val(value.to_string()),
        "bottom" => style.bottom = convert_to_val(value.to_string()),

        "display" => style.display = convert_to_display(value.to_string()),
        "position" => style.position_type = convert_to_position(value.to_string()),
        "box-sizing" => style.box_sizing = convert_to_box_sizing(value.to_string()),
        "scroll-width" => style.scrollbar_width = convert_to_f32(value.to_string()),
        "gap" => style.gap = convert_to_val(value.to_string()),
        "justify-content" => {
            style.justify_content = convert_to_bevy_justify_content(value.to_string())
        }
        "align-items" => style.align_items = convert_to_bevy_align_items(value.to_string()),
        "flex-direction" => {
            style.flex_direction = convert_to_bevy_flex_direction(value.to_string())
        }
        "flex-grow" => style.flex_grow = value.trim().parse::<f32>().ok(),
        "flex-shrink" => style.flex_shrink = value.trim().parse::<f32>().ok(),
        "flex-basis" => style.flex_basis = convert_to_val(value.to_string()),
        "flex-wrap" => {
            style.flex_wrap = convert_to_bevy_flex_wrap(value.to_string());
        }

        "grid-row" => {
            style.grid_row = convert_to_bevy_grid_placement(value.to_string());
        }
        "grid-column" => {
            style.grid_column = convert_to_bevy_grid_placement(value.to_string());
        }
        "grid-auto-flow" => {
            style.grid_auto_flow = convert_to_bevy_grid_flow(value.to_string());
        }
        "grid-template-rows" => {
            style.grid_template_rows = convert_to_bevy_grid_template(value.to_string());
        }
        "grid-template-columns" => {
            style.grid_template_columns = convert_to_bevy_grid_template(value.to_string());
        }
        "grid-auto-rows" => {
            style.grid_auto_rows = convert_to_bevy_grid_track(value.to_string());
        }
        "grid-auto-columns" => {
            style.grid_auto_columns = convert_to_bevy_grid_track(value.to_string());
        }

        "background-color" => {
            style.background = Some(Background {
                color: convert_to_color(value.to_string()).unwrap_or(Color::WHITE),
                ..default()
            });
        }
        "background" => {
            style.background = convert_to_background(value.to_string(), true);
        }
        "background-image" => {
            style.background = convert_to_background(value.to_string(), false);
        }

        "font-size" => style.font_size = convert_to_font_size(value.to_string()),

        "border" => {
            if let Some((rect, color)) = convert_css_border(value.to_string()) {
                style.border = Some(rect);
                style.border_color = Some(color);
            }
        }
        "border-left" => {
            let val = convert_to_val(value.to_string()).unwrap_or(Val::Px(0.));
            let mut border = style.border.unwrap_or_default();
            border.left = val;
            style.border = Some(border);
        }
        "border-right" => {
            let val = convert_to_val(value.to_string()).unwrap_or(Val::Px(0.));
            let mut border = style.border.unwrap_or_default();
            border.right = val;
            style.border = Some(border);
        }
        "border-top" => {
            let val = convert_to_val(value.to_string()).unwrap_or(Val::Px(0.));
            let mut border = style.border.unwrap_or_default();
            border.top = val;
            style.border = Some(border);
        }
        "border-bottom" => {
            let val = convert_to_val(value.to_string()).unwrap_or(Val::Px(0.));
            let mut border = style.border.unwrap_or_default();
            border.bottom = val;
            style.border = Some(border);
        }
        "border-radius" => style.border_radius = convert_to_radius(value.to_string()),
        "border-color" => style.border_color = convert_to_color(value.to_string()),
        "border-width" => style.border = convert_to_ui_rect(value.to_string()),

        "box-shadow" => style.box_shadow = convert_to_bevy_box_shadow(value.to_string()),

        "overflow" => style.overflow = convert_overflow(value.to_string(), "all"),
        "overflow-y" => {
            let val = convert_overflow(value.to_string(), "y");
            let mut overflow = style.overflow.unwrap_or_default();
            overflow.y = val.unwrap_or_default().y;
            style.overflow = Some(overflow);
        }
        "overflow-x" => {
            let val = convert_overflow(value.to_string(), "x");
            let mut overflow = style.overflow.unwrap_or_default();
            overflow.x = val.unwrap_or_default().x;
            style.overflow = Some(overflow);
        }

        "text-wrap" => style.text_wrap = convert_to_bevy_line_break(value.to_string()),
        "z-index" => style.z_index = convert_to_i32(value.to_string()),
        "pointer-events" => style.pointer_events = convert_to_bevy_pick_able(value.to_string()),

        _ => {}
    }
}

/// Converts a string representation of a CSS value into a Bevy [`Val`].
///
/// # Supported Formats
/// - `"100px"` → `Val::Px(100.0)`
/// - `"75%"` → `Val::Percent(75.0)`
///
/// # Parameters
/// - `value`: A [`String`] representing a dimension value (e.g. `"20px"`, `"50%"`).
///
/// # Returns
/// - `Some(Val)` if parsing succeeds.
/// - `None` if the format is invalid or cannot be parsed.
///
/// # Example
/// ```
/// use bevy::prelude::Val;
/// use bevy_extended_ui::styling::css::convert_to_val;
/// assert_eq!(convert_to_val("42px".to_string()), Some(Val::Px(42.0)));
/// assert_eq!(convert_to_val("80%".to_string()), Some(Val::Percent(80.0)));
/// ```
pub fn convert_to_val(value: String) -> Option<Val> {
    let mut val = None;
    let trimmed = value.trim();
    if trimmed.ends_with("px") {
        let count = trimmed.replace("px", "").parse::<f32>().ok()?;
        val = Some(Val::Px(count));
    } else if trimmed.ends_with("%") {
        let count = trimmed.replace("%", "").parse::<f32>().ok()?;
        val = Some(Val::Percent(count));
    } else if trimmed == "0" || trimmed == "0.0" || trimmed == "-0" || trimmed == "-0.0" {
        val = Some(Val::Px(0.0));
    }
    val
}

/// Converts a numeric string into an [`i32`] if the format is valid.
///
/// # Parameters
/// - `value`: A [`String`] containing an integer, optionally negative (e.g. `"42"`, `"-10"`).
///
/// # Returns
/// - `Some(i32)` if parsing succeeds and the string is a valid integer.
/// - `None` if the input is non-numeric or contains invalid characters.
///
/// # Example
/// ```
/// use bevy_extended_ui::styling::css::convert_to_i32;
/// assert_eq!(convert_to_i32("123".to_string()), Some(123));
/// assert_eq!(convert_to_i32("abc".to_string()), None);
/// ```
pub fn convert_to_i32(value: String) -> Option<i32> {
    let trimmed = value.trim();

    let re = Regex::new(r"^-?\d+$").unwrap();

    if re.is_match(trimmed) {
        trimmed.parse::<i32>().ok()
    } else {
        None
    }
}

pub fn convert_to_f32(value: String) -> Option<f32> {
    let trimmed = value.trim();
    trimmed.parse::<f32>().ok()
}

/// Converts a CSS font-size string into a [`FontVal`] (custom type).
///
/// # Supported Units
/// - `"px"` → `FontVal::Px(f32)`
/// - `"rem"` → `FontVal::Rem(f32)`
///
/// # Parameters
/// - `value`: A [`String`] containing a font size (e.g. `"16px"`, `"1.2rem"`).
///
/// # Returns
/// - `Some(FontVal)` if the value can be parsed.
/// - `None` if the value is malformed or unsupported.
///
/// # Example
/// ```
/// use bevy_extended_ui::styling::css::convert_to_font_size;
/// use bevy_extended_ui::styling::FontVal;
/// assert_eq!(convert_to_font_size("14px".to_string()), Some(FontVal::Px(14.0)));
/// assert_eq!(convert_to_font_size("1.5rem".to_string()), Some(FontVal::Rem(1.5)));
/// ```
pub fn convert_to_font_size(value: String) -> Option<FontVal> {
    let mut val = None;
    let trimmed = value.trim();
    if trimmed.ends_with("px") {
        let count = trimmed.replace("px", "").parse::<f32>().ok()?;
        val = Some(FontVal::Px(count));
    } else if trimmed.ends_with("rem") {
        let count = trimmed.replace("rem", "").parse::<f32>().ok()?;
        val = Some(FontVal::Rem(count));
    }
    val
}

/// Converts a CSS color string into a Bevy [`Color`].
///
/// # Supported Formats
/// - Named colors (e.g. `"red"`, `"white"`, `"transparent"`)
/// - Hex colors (e.g. `"#ff00ff"`, `"#00000000"` for transparent)
/// - RGB: `"rgb(255, 0, 0)"`
/// - RGBA: `"rgba(255, 0, 0, 128)"`
///
/// # Parameters
/// - `value`: A [`String`] representing a color in any CSS-compatible format.
///
/// # Returns
/// - `Some(Color)` if parsing succeeds.
/// - `None` if the format is invalid or unsupported.
///
/// # Example
/// ```
/// use bevy::prelude::Color;
/// use bevy_extended_ui::styling::css::convert_to_color;
/// assert_eq!(convert_to_color("red".to_string()), Some(Color::WHITE));
/// assert_eq!(convert_to_color("rgba(0,0,0,0)".to_string()), Some(Color::NONE));
/// assert!(convert_to_color("#123456".to_string()).is_some());
/// ```
pub fn convert_to_color(value: String) -> Option<Color> {
    let mut color = None;
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("transparent") || trimmed.eq_ignore_ascii_case("none") {
        return Some(Color::NONE);
    }

    if trimmed.starts_with("#") {
        if trimmed.eq("#00000000") {
            color = Some(Color::NONE);
        } else {
            color = Some(Colored::hex_to_color(trimmed));
        }
    } else if trimmed.starts_with("rgb(") {
        let correct = trimmed.trim_start_matches("rgb(").trim_end_matches(")");
        let parts: Vec<_> = correct.split(',').map(str::trim).collect();

        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;

            color = Some(Color::srgb_u8(r, g, b));
        }
    } else if trimmed.starts_with("rgba(") {
        if trimmed.eq("rgba(0, 0, 0, 0)") {
            color = Some(Color::NONE);
        } else {
            let correct = trimmed.trim_start_matches("rgba(").trim_end_matches(")");
            let parts: Vec<_> = correct.split(',').map(str::trim).collect();

            if parts.len() == 4 {
                let r = parts[0].parse::<u8>().ok()?;
                let g = parts[1].parse::<u8>().ok()?;
                let b = parts[2].parse::<u8>().ok()?;
                let a = parts[3].parse::<u8>().ok()?;

                color = Some(Color::srgba_u8(r, g, b, a));
            }
        }
    } else {
        color = Colored::named(trimmed);
    }

    color
}

/// Converts a CSS `background` value into a [`Background`] struct.
///
/// Supports both `url(...)` image backgrounds and color values.
/// If `all_types` is true, also attempts to interpret the value as a color (e.g. `"red"`, `"#ffcc00"`).
///
/// # Parameters
/// - `value`: The CSS `background` value as a [`String`] (e.g. `"url(\"image.png\")"` or `"blue"`).
/// - `all_types`: Whether to allow color parsing in addition to `url(...)`.
///
/// # Returns
/// - `Some(Background)` if a valid image URL or color is parsed.
/// - `None` if parsing fails or `all_types` is false and the value is not a `url(...)`.
///
/// # Example
/// ```
/// use bevy_extended_ui::styling::css::convert_to_background;
/// convert_to_background("url(\"icon.png\")".to_string(), false);
/// convert_to_background("red".to_string(), true);
/// ```
pub fn convert_to_background(value: String, all_types: bool) -> Option<Background> {
    let trimmed = value.trim();

    if trimmed.starts_with("url(") {
        let url = trimmed.trim_start_matches("url(").trim_end_matches(")");
        Some(Background {
            image: Some(url.to_string().replace("\"", "")),
            ..default()
        })
    } else {
        if all_types {
            let color = convert_to_color(value.to_string()).unwrap_or_default();
            return Some(Background { color, ..default() });
        }

        None
    }
}

/// Converts a CSS `display` value into a Bevy [`Display`] enum.
///
/// Supported values include:
/// - `"flex"` → `Display::Flex`
/// - `"grid"` → `Display::Grid`
/// - `"block"` → `Display::Block`
/// - `"none"` → `Display::None`
///
/// If the input is unrecognized, defaults to `Display::Block`.
///
/// # Parameters
/// - `value`: A [`String`] containing the CSS `display` value.
///
/// # Returns
/// - `Some(Display)` with a best-effort fallback.
///
/// # Example
/// ```
/// use bevy::prelude::Display;
/// use bevy_extended_ui::styling::css::convert_to_display;
/// assert_eq!(convert_to_display("flex".to_string()), Some(Display::Flex));
/// assert_eq!(convert_to_display("unknown".to_string()), Some(Display::Block));
/// ```
pub fn convert_to_display(value: String) -> Option<Display> {
    let trimmed = value.trim();
    match trimmed {
        "flex" => Some(Display::Flex),
        "grid" => Some(Display::Grid),
        "block" => Some(Display::Block),
        "none" => Some(Display::None),
        _ => Some(Display::Block),
    }
}

/// Converts a CSS `position` value into a Bevy [`PositionType`] enum.
///
/// Supported values:
/// - `"relative"` → `PositionType::Relative`
/// - `"absolute"` → `PositionType::Absolute`
///
/// Any unrecognized value defaults to `PositionType::Relative`.
///
/// # Parameters
/// - `value`: A [`String`] containing the CSS `position` value.
///
/// # Returns
/// - `Some(PositionType)`
///
/// # Example
/// ```
/// use bevy::prelude::PositionType;
/// use bevy_extended_ui::styling::css::convert_to_position;
/// assert_eq!(convert_to_position("absolute".to_string()), Some(PositionType::Absolute));
/// ```
pub fn convert_to_position(value: String) -> Option<PositionType> {
    let trimmed = value.trim();
    match trimmed {
        "relative" => Some(PositionType::Relative),
        "absolute" => Some(PositionType::Absolute),
        _ => Some(PositionType::Relative),
    }
}

pub fn convert_to_box_sizing(value: String) -> Option<BoxSizing> {
    let trimmed = value.trim();
    match trimmed {
        "border-box" => Some(BoxSizing::BorderBox),
        "content-box" => Some(BoxSizing::ContentBox),
        _ => Some(BoxSizing::BorderBox),
    }
}

///
/// Accepts 1–4 values (e.g. `"10px"`, `"10px 20px"`, etc.), similar to CSS shorthand.
/// Uses the same order as CSS:
/// - 1 value → all corners
/// - 2 values → top-left & top-right / bottom-right & bottom-left
/// - 3 values → top-left / top-right / bottom-left (bottom-right = 0)
/// - 4 values → top-left / top-right / bottom-right / bottom-left
///
/// # Parameters
/// - `value`: A [`String`] containing the CSS `border-radius` values.
///
/// # Returns
/// - `Some(Radius)` if parsing succeeds.
/// - `None` if the input format is invalid.
///
/// # Example
/// ```
/// use bevy_extended_ui::styling::css::convert_to_radius;
/// convert_to_radius("10px".to_string());
/// convert_to_radius("10px 20px 30px 40px".to_string());
/// ```
pub fn convert_to_radius(value: String) -> Option<Radius> {
    let vals = parse_radius_values(&value)?;

    let (top_left, top_right, bottom_right, bottom_left) = match vals.len() {
        1 => (
            vals[0].clone(),
            vals[0].clone(),
            vals[0].clone(),
            vals[0].clone(),
        ),
        2 => (
            vals[0].clone(), // top-left
            vals[0].clone(), // top-right
            vals[1].clone(), // bottom-right
            vals[1].clone(), // bottom-left
        ),
        3 => (
            vals[0].clone(),
            vals[1].clone(),
            Val::Px(0.0),
            vals[2].clone(),
        ),
        4 => (
            vals[0].clone(),
            vals[1].clone(),
            vals[2].clone(),
            vals[3].clone(),
        ),
        _ => return None,
    };

    Some(Radius {
        top_left,
        top_right,
        bottom_right,
        bottom_left,
    })
}

/// Converts CSS shorthand (e.g. `margin`, `padding`) into a Bevy [`UiRect`].
///
/// Accepts 1–4 values:
/// - 1 value → all sides
/// - 2 values → top & bottom / left & right
/// - 3 values → left / right / top (bottom = 0)
/// - 4 values → left / right / top / bottom
///
/// # Parameters
/// - `value`: A [`String`] like `"10px"`, `"10px 20px"`, etc.
///
/// # Returns
/// - `Some(UiRect)` if parsing succeeds.
/// - `None` if the value format is invalid.
///
/// # Example
/// ```
/// use bevy_extended_ui::styling::css::convert_to_ui_rect;
/// convert_to_ui_rect("10px".to_string());
/// convert_to_ui_rect("10px 20px 5px 15px".to_string());
/// ```
pub fn convert_to_ui_rect(value: String) -> Option<UiRect> {
    let vals = parse_radius_values(&value)?;

    let (left, right, top, bottom) = match vals.len() {
        1 => (
            vals[0].clone(),
            vals[0].clone(),
            vals[0].clone(),
            vals[0].clone(),
        ),
        2 => (
            vals[1].clone(), // left
            vals[1].clone(), // right
            vals[0].clone(), // top
            vals[0].clone(), // bottom
        ),
        3 => (
            vals[0].clone(),
            vals[1].clone(),
            vals[2].clone(),
            Val::Px(0.0),
        ),
        4 => (
            vals[0].clone(),
            vals[1].clone(),
            vals[2].clone(),
            vals[3].clone(),
        ),
        _ => return None,
    };

    Some(UiRect {
        left,
        right,
        top,
        bottom,
    })
}

/// Converts a CSS-like box-shadow string into a Bevy [`BoxShadow`] struct.
///
/// Parses shadow offset (x, y), blur radius, spread radius, and color from the input string.
/// Supports values in pixels (e.g., `"10px"`), percentages (e.g., `"50%"`), or CSS color formats
/// (`"#rrggbb"`, `"rgb(...)"`, `"rgba(...)"`).
///
/// The number of numeric values determines which parts are set:
/// - 1 value: x, y, blur, and spread all set to the same value.
/// - 2 values: x and y set; blur and spread default to 0.
/// - 3 values: x, y, blur set; spread defaults to 0.
/// - 4 values: x, y, blur, and spread all sets.
///
/// If the input is malformed or missing required parts, returns `None`.
///
/// # Parameters
/// - `value`: The CSS box-shadow string (e.g., `"5px 10px 15px #000000"`).
///
/// # Returns
/// - `Some(BoxShadow)` if parsing succeeds.
/// - `None` on failure.
///
/// # Example
/// ```
/// use bevy_extended_ui::styling::css::convert_to_bevy_box_shadow;
/// convert_to_bevy_box_shadow("5px 10px 15px 3px rgba(0,0,0,0.5)".to_string());
/// ```
pub fn convert_to_bevy_box_shadow(value: String) -> Option<BoxShadow> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut vals = vec![];
    let mut color = Colored::TRANSPARENT;

    for part in parts {
        let trimmed = part.trim();
        if trimmed.ends_with("px") || trimmed.eq_ignore_ascii_case("0") {
            let number = trimmed.trim_end_matches("px").parse::<f32>().ok()?;
            vals.push(Val::Px(number));
        } else if trimmed.ends_with('%') {
            let number = trimmed.trim_end_matches('%').parse::<f32>().ok()?;
            vals.push(Val::Percent(number));
        } else if trimmed.starts_with("#")
            || trimmed.starts_with("rgb(")
            || trimmed.starts_with("rgba(")
        {
            color = convert_to_color(trimmed.to_string())?;
        }
    }

    let (x, y, blur, spread) = match vals.len() {
        1 => (
            vals[0].clone(),
            vals[0].clone(),
            vals[0].clone(),
            vals[0].clone(),
        ),
        2 => (
            vals[0].clone(), // x
            vals[1].clone(), // y
            Val::Px(0.),     // blur
            Val::Px(0.),     // spread
        ),
        3 => (
            vals[0].clone(),
            vals[1].clone(),
            vals[2].clone(),
            Val::Px(0.0),
        ),
        4 => (
            vals[0].clone(),
            vals[1].clone(),
            vals[2].clone(),
            vals[3].clone(),
        ),
        _ => return None,
    };

    Some(BoxShadow::new(color, x, y, spread, blur))
}

/// Parses a CSS border shorthand string into a [`UiRect`] for border widths and a [`Color`].
///
/// The input string is expected to be in the form `"WIDTH COLOR"` where:
/// - WIDTH is a length value (e.g. `"5px"`, `"10%"`, or `"0"`).
/// - COLOR is an optional CSS color string (e.g. `"#ff0000"`, `"rgba(255,0,0,1)"`).
///
/// If the color is missing, defaults to transparent.
///
/// # Parameters
/// - `value`: CSS border shorthand string.
///
/// # Returns
/// - `Some((UiRect, Color))` on successful parsing, where `UiRect` sets all borders to the given width.
/// - `None` if the width is missing or cannot be parsed.
///
/// # Example
/// ```
/// use bevy_extended_ui::styling::css::convert_css_border;
/// convert_css_border("5px #ff0000".to_string());
/// convert_css_border("0".to_string()); // transparent border
/// ```
pub fn convert_css_border(value: String) -> Option<(UiRect, Color)> {
    fn parse_val(input: &str) -> Option<Val> {
        if input.ends_with("px") {
            input
                .trim_end_matches("px")
                .parse::<f32>()
                .ok()
                .map(Val::Px)
        } else if input.ends_with('%') {
            input
                .trim_end_matches('%')
                .parse::<f32>()
                .ok()
                .map(Val::Percent)
        } else if input == "0" {
            Some(Val::Px(0.0))
        } else {
            None
        }
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let rect_val = parse_val(parts[0])?;
    let rect = UiRect::all(rect_val);

    let color = if parts.len() > 1 {
        convert_to_color(parts[1].to_string())?
    } else {
        Colored::TRANSPARENT
    };

    Some((rect, color))
}

/**
 * Converts a string into a `JustifyContent` enum value.
 *
 * Recognized values include: "start", "flex-start", "end", "flex-end", "center",
 * "space-between", "space-around", "space-evenly", and "stretch".
 *
 * @param value The CSS justify-content value as a string slice.
 * @return Some(JustifyContent) if the value is recognized, None otherwise.
 */
pub fn convert_to_bevy_justify_content(value: String) -> Option<JustifyContent> {
    let trimmed = value.trim();
    match trimmed {
        "start" => Some(JustifyContent::Start),
        "flex-start" => Some(JustifyContent::FlexStart),
        "end" => Some(JustifyContent::End),
        "flex-end" => Some(JustifyContent::FlexStart),
        "center" => Some(JustifyContent::Center),
        "space-between" => Some(JustifyContent::SpaceBetween),
        "space-around" => Some(JustifyContent::SpaceAround),
        "space-evenly" => Some(JustifyContent::SpaceEvenly),
        "stretch" => Some(JustifyContent::Stretch),
        _ => Some(JustifyContent::default()),
    }
}

/**
 * Converts a string into an `AlignItems` enum value.
 *
 * Recognized values include: "start", "flex-start", "end", "flex-end", "center",
 * "baseline", and "stretch".
 *
 * @param value The CSS align-item value as a string slice.
 * @return Some(AlignItems) if the value is recognized, None otherwise.
 */
pub fn convert_to_bevy_align_items(value: String) -> Option<AlignItems> {
    let trimmed = value.trim();
    match trimmed {
        "start" => Some(AlignItems::Start),
        "flex-start" => Some(AlignItems::FlexStart),
        "end" => Some(AlignItems::End),
        "flex-end" => Some(AlignItems::FlexStart),
        "center" => Some(AlignItems::Center),
        "baseline" => Some(AlignItems::Baseline),
        "stretch" => Some(AlignItems::Stretch),
        _ => Some(AlignItems::default()),
    }
}

/**
 * Converts a string into a `FlexDirection` enum value.
 *
 * Recognized values include: "row", "column", "row-reverse", and "column-reverse".
 *
 * @param value The CSS flex-direction value as a string slice.
 * @return Some(FlexDirection) if the value is recognized, None otherwise.
 */
pub fn convert_to_bevy_flex_direction(value: String) -> Option<FlexDirection> {
    let trimmed = value.trim();
    match trimmed {
        "row" => Some(FlexDirection::Row),
        "column" => Some(FlexDirection::Column),
        "row-reverse" => Some(FlexDirection::RowReverse),
        "column-reverse" => Some(FlexDirection::ColumnReverse),
        _ => Some(FlexDirection::default()),
    }
}

/**
 * Converts a string into a `FlexWrap` enum value.
 *
 * Recognized values include: "wrap", "nowrap", and "wrap-reverse".
 *
 * @param value The CSS flex-wrap value as a string slice.
 * @return Some(FlexWrap) if the value is recognized, None otherwise.
 */
pub fn convert_to_bevy_flex_wrap(value: String) -> Option<FlexWrap> {
    let trimmed = value.trim();
    match trimmed {
        "wrap" => Some(FlexWrap::Wrap),
        "nowrap" => Some(FlexWrap::NoWrap),
        "wrap-reverse" => Some(FlexWrap::WrapReverse),
        _ => Some(FlexWrap::default()),
    }
}

/**
 * Converts a string into a `LineBreak` enum value.
 *
 * Recognized values include: "wrap", "stable", "nowrap", "pretty", "balance", and "unset".
 *
 * @param value The CSS line-break value as a string slice.
 * @return Some(LineBreak) if the value is recognized, None otherwise.
 */
pub fn convert_to_bevy_line_break(value: String) -> Option<LineBreak> {
    let trimmed = value.trim();
    match trimmed {
        "wrap" | "stable" => Some(LineBreak::WordOrCharacter),
        "nowrap" => Some(LineBreak::NoWrap),
        "pretty" | "balance" => Some(LineBreak::WordBoundary),
        "unset" => Some(LineBreak::AnyCharacter),
        _ => Some(LineBreak::default()),
    }
}

/// Converts a string value into a `Pickable` component used by Bevy UI.
///
/// <p>
/// This function interprets the input string and returns an appropriate `Pickable`
/// configuration. If the value is `"none"`, it returns `Pickable::IGNORE` to disable pointer
/// interactions. For all other values, it returns the default `Pickable` behavior.
/// </p>
///
/// # Parameters
/// - `value`: A `String` containing the desired pickable mode (e.g., `"none"`, `"auto"`, etc.).
///
/// # Returns
/// An `Option<Pickable>`:
/// - `Some(Pickable::IGNORE)` if the value is `"none"`
/// - `Some(Pickable::default())` for any other input
///
/// # Example
/// ```
/// use bevy::prelude::Pickable;
/// use bevy_extended_ui::styling::css::convert_to_bevy_pick_able;
/// let pickable = convert_to_bevy_pick_able("none".to_string());
/// assert_eq!(pickable, Some(Pickable::IGNORE));
/// ```
pub fn convert_to_bevy_pick_able(value: String) -> Option<Pickable> {
    let trimmed = value.trim();
    match trimmed {
        "none" => Some(Pickable::IGNORE),
        _ => Some(Pickable::default()),
    }
}

/**
 * Converts a string into a `GridAutoFlow` enum value.
 *
 * Recognized values include: "row", "column", "row-dense", and "column-dense".
 *
 * @param value The CSS grid-auto-flow value as a string slice.
 * @return Some(GridAutoFlow) if the value is recognized, None otherwise.
 */
pub fn convert_to_bevy_grid_flow(value: String) -> Option<GridAutoFlow> {
    let trimmed = value.trim();
    match trimmed {
        "row" => Some(GridAutoFlow::Row),
        "column" => Some(GridAutoFlow::Column),
        "row-dense" => Some(GridAutoFlow::RowDense),
        "column-dense" => Some(GridAutoFlow::ColumnDense),
        _ => Some(GridAutoFlow::default()),
    }
}

/**
 * Converts a CSS grid placement string into a `GridPlacement` enum.
 *
 * Supports values such as
 * - "span N" (where N is a positive integer),
 * - "start/end" (two positive integers separated by a slash),
 * - or a single positive integer (start).
 *
 * @param value The CSS grid placement string as a string slice.
 * @return Some(GridPlacement) if the value is valid and parsed, None otherwise.
 */
pub fn convert_to_bevy_grid_placement(value: String) -> Option<GridPlacement> {
    let trimmed = value.trim();

    if let Some(span_str) = trimmed.strip_prefix("span ") {
        if let Ok(span) = span_str.trim().parse::<u16>() {
            if span > 0 {
                return Some(GridPlacement::span(span));
            }
        }
    }

    if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.split('/').map(str::trim).collect();
        if parts.len() == 2 {
            let start = parts[0].parse::<i16>().ok()?;
            let end = parts[1].parse::<i16>().ok()?;
            if start > 0 && end > 0 {
                return Some(GridPlacement::start_end(start, end));
            }
        }
    }

    if let Ok(start) = trimmed.parse::<i16>() {
        if start > 0 {
            return Some(GridPlacement::start(start));
        }
    }

    None
}

// ==============================================================================
//                              Only Grid Tracks
// ==============================================================================

/**
 * Converts a whitespace-separated string into a vector of `GridTrack` values.
 *
 * Each part of the input string is parsed individually by `parse_single_grid_track`.
 *
 * @param value The CSS grid track definition as a string.
 * @return Some(Vec<GridTrack>) if all parts are successfully parsed; None otherwise.
 */
pub fn convert_to_bevy_grid_track(value: String) -> Option<Vec<GridTrack>> {
    value
        .split_whitespace()
        .map(|part| parse_single_grid_track(part))
        .collect()
}

// ==============================================================================
//                               Grid Template
// ==============================================================================

/**
 * Converts a CSS grid-template string into a vector of `RepeatedGridTrack`.
 *
 * Supports the `repeat(count, track)` syntax as well as space-separated single tracks.
 * Examples:
 * - "repeat(3, 100px)"
 * - "100px auto min-content"
 *
 * @param value The CSS grid-template string.
 * @return Some(Vec<RepeatedGridTrack>) if parsing succeeds, None otherwise.
 */
pub fn convert_to_bevy_grid_template(value: String) -> Option<Vec<RepeatedGridTrack>> {
    let input = value.trim();
    let mut result = Vec::new();

    if let Some(content) = input
        .strip_prefix("repeat(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let mut parts = content.splitn(2, ',').map(str::trim);
        let count = parts.next()?.parse::<u16>().ok()?;
        let track_def = parts.next()?;

        let track = parse_single_grid_track(track_def)?;
        result.push(RepeatedGridTrack::repeat_many(
            GridTrackRepetition::Count(count),
            vec![track],
        ));
        return Some(result);
    }

    for token in input.split_whitespace() {
        if let Some(track) = parse_single_grid_track(token) {
            result.push(RepeatedGridTrack::repeat_many(
                GridTrackRepetition::Count(1),
                vec![track],
            ));
        } else {
            return None;
        }
    }

    Some(result)
}

/**
 * Parses a single CSS grid track definition into a `GridTrack`.
 *
 * Supports values like:
 * - "auto"
 * - "min-content"
 * - "max-content"
 * - "minmax(min, max)"
 * - fixed sizes with units: "100px", "50%", "1fr"
 *
 * @param input The CSS grid track string.
 * @return Some(GridTrack) if parsing succeeds, None otherwise.
 */
fn parse_single_grid_track(input: &str) -> Option<GridTrack> {
    let input = input.trim();
    match input {
        "auto" => Some(GridTrack::auto()),
        "min-content" => Some(GridTrack::min_content()),
        "max-content" => Some(GridTrack::max_content()),
        _ if input.starts_with("minmax(") && input.ends_with(')') => {
            let inner = &input[7..input.len() - 1];
            let mut parts = inner.split(',').map(str::trim);
            let min = parse_min_sizing(parts.next()?)?;
            let max = parse_max_sizing(parts.next()?)?;
            Some(GridTrack::minmax(min, max))
        }
        _ if input.ends_with("px") => input
            .strip_suffix("px")?
            .parse::<f32>()
            .ok()
            .map(GridTrack::px),
        _ if input.ends_with('%') => input
            .strip_suffix('%')?
            .parse::<f32>()
            .ok()
            .map(GridTrack::percent),
        _ if input.ends_with("fr") => input
            .strip_suffix("fr")?
            .parse::<f32>()
            .ok()
            .map(GridTrack::fr),
        _ => None,
    }
}

/**
 * Parses a CSS min track sizing function from a string.
 *
 * Recognized values:
 * - "auto"
 * - "min-content"
 * - "max-content"
 * - fixed size in px, e.g. "100px"
 *
 * @param input The CSS min track sizing string.
 * @return Some(MinTrackSizingFunction) if parsing succeeds, None otherwise.
 */
fn parse_min_sizing(input: &str) -> Option<MinTrackSizingFunction> {
    match input {
        "auto" => Some(MinTrackSizingFunction::Auto),
        "min-content" => Some(MinTrackSizingFunction::MinContent),
        "max-content" => Some(MinTrackSizingFunction::MaxContent),
        _ if input.ends_with("px") => input
            .strip_suffix("px")?
            .parse::<f32>()
            .ok()
            .map(MinTrackSizingFunction::Px),
        _ => None,
    }
}

/**
 * Parses a CSS max track sizing function from a string.
 *
 * Recognized values:
 * - "auto"
 * - "min-content"
 * - "max-content"
 * - fixed size in px, e.g. "100px"
 * - fractional units, e.g. "1fr"
 *
 * @param input The CSS max track sizing string.
 * @return Some(MaxTrackSizingFunction) if parsing succeeds, None otherwise.
 */
fn parse_max_sizing(input: &str) -> Option<MaxTrackSizingFunction> {
    match input {
        "auto" => Some(MaxTrackSizingFunction::Auto),
        "min-content" => Some(MaxTrackSizingFunction::MinContent),
        "max-content" => Some(MaxTrackSizingFunction::MaxContent),
        _ if input.ends_with("px") => input
            .strip_suffix("px")?
            .parse::<f32>()
            .ok()
            .map(MaxTrackSizingFunction::Px),
        _ if input.ends_with("fr") => input
            .strip_suffix("fr")?
            .parse::<f32>()
            .ok()
            .map(MaxTrackSizingFunction::Fraction),
        _ => None,
    }
}

/**
 * Converts a CSS overflow string into an `Overflow` struct for the given axis.
 *
 * Recognized overflow values:
 * - "hidden"
 * - "scroll"
 * - "clip"
 * - "visible"
 *
 * The `which` parameter controls which axis is affected:
 * - "*" | "all" | "both" applies to both axes.
 * - "x" applies only to the horizontal axis.
 * - "y" applies only to the vertical axis.
 *
 * @param value The CSS overflow value string.
 * @param which The axis specifier ("x", "y", "all", etc.).
 * @return Some(Overflow) if valid input, None otherwise.
 */
pub fn convert_overflow(value: String, which: &str) -> Option<Overflow> {
    let trimmed = value.trim();
    let overflow_axis = match trimmed {
        "hidden" => OverflowAxis::Hidden,
        "scroll" | "auto" => OverflowAxis::Scroll,
        "clip" => OverflowAxis::Clip,
        "visible" => OverflowAxis::Visible,
        _ => OverflowAxis::default(),
    };

    if which == "*" || which == "all" || which == "both" {
        Some(Overflow {
            x: overflow_axis,
            y: overflow_axis,
        })
    } else if which == "y" {
        Some(Overflow {
            y: overflow_axis,
            ..default()
        })
    } else if which == "x" {
        Some(Overflow {
            x: overflow_axis,
            ..default()
        })
    } else {
        return None;
    }
}

/**
 * Parses a string containing 1 to 4 CSS radius values into a vector of `Val`.
 *
 * Supported units:
 * - px (pixels), e.g. "10px"
 * - percent (%), e.g. "50%"
 * - zero ("0") without unit
 *
 * @param value The CSS radius string.
 * @return Some(Vec<Val>) if parsing succeeds, None otherwise.
 */
fn parse_radius_values(value: &str) -> Option<Vec<Val>> {
    let mut vals = Vec::new();
    for part in value.split_whitespace() {
        let trimmed = part.trim();
        if trimmed.ends_with("px") || trimmed.eq_ignore_ascii_case("0") {
            let number = trimmed.trim_end_matches("px").parse::<f32>().ok()?;
            vals.push(Val::Px(number));
        } else if trimmed.ends_with('%') {
            let number = trimmed.trim_end_matches('%').parse::<f32>().ok()?;
            vals.push(Val::Percent(number));
        } else {
            return None;
        }
    }
    Some(vals)
}
