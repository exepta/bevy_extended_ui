use std::collections::HashMap;
use std::fs;
use bevy::prelude::*;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::traits::ToCss;
use crate::styling::paint::Colored;
use crate::styling::{Background, FontVal, Radius, Style};

pub fn load_css(path: &str) -> Result<HashMap<String, Style>, String> {
    let mut internal_path = "assets/internal.css";
    if !path.is_empty() {
        internal_path = path;
    }
    let css = match fs::read_to_string(internal_path) {
        Ok(content) => content,
        Err(err) => {
            error!("Failed to load default style: {}: {}", internal_path, err);
            return Ok(HashMap::new());
        }
    };

    let stylesheet = match StyleSheet::parse(&css, ParserOptions::default()) {
        Ok(stylesheet) => stylesheet,
        Err(err) => {
            error!("Css Parsing failed: {:?}", err);
            return Ok(HashMap::new());
        }
    };
    
    let mut style_map = HashMap::new();
    for rule in &stylesheet.rules.0 {
        if let CssRule::Style(style_rule) = rule {
            let selector = match style_rule.selectors.to_css_string(PrinterOptions::default()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let mut style = Style::default();
            for property in &style_rule.declarations.declarations {
                let property_id = property.property_id();
                let name = property_id.name();
                let value = match property.value_to_css_string(PrinterOptions::default()) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                
                match name {
                    "width" => {
                        style.width = convert_to_val(value.clone());
                    },
                    "min_width" => {
                        style.min_width = convert_to_val(value.clone());
                    },
                    "max_width" => {
                        style.max_width = convert_to_val(value.clone());
                    },
                    "height" => {
                        style.height = convert_to_val(value.clone());
                    },
                    "min_height" => {
                        style.min_height = convert_to_val(value.clone());
                    },
                    "max_height" => {
                        style.max_height = convert_to_val(value.clone());
                    },
                    "padding" => {
                        style.padding = convert_to_ui_rect(value.clone());
                    },
                    "padding-left" => {
                        let correct_string = format!("{} 0 0 0", value.clone()).to_string();
                        style.padding = convert_to_ui_rect(correct_string);
                    },
                    "padding-right" => {
                        let correct_string = format!("0 {} 0 0", value.clone()).to_string();
                        style.padding = convert_to_ui_rect(correct_string);
                    },
                    "padding-top" => {
                        let correct_string = format!("0 0 {} 0", value.clone()).to_string();
                        style.padding = convert_to_ui_rect(correct_string);
                    },
                    "padding-bottom" => {
                        let correct_string = format!("0 0 0 {}", value.clone()).to_string();
                        style.padding = convert_to_ui_rect(correct_string);
                    },
                    "margin" => {
                        style.margin = convert_to_ui_rect(value.clone());
                    },
                    "margin-left" => {
                        let correct_string = format!("{} 0 0 0", value.clone()).to_string();
                        style.margin = convert_to_ui_rect(correct_string);
                    },
                    "margin-right" => {
                        let correct_string = format!("0 {} 0 0", value.clone()).to_string();
                        style.margin = convert_to_ui_rect(correct_string);
                    },
                    "margin-top" => {
                        let correct_string = format!("0 0 {} 0", value.clone()).to_string();
                        style.margin = convert_to_ui_rect(correct_string);
                    },
                    "margin-bottom" => {
                        let correct_string = format!("0 0 0 {}", value.clone()).to_string();
                        style.margin = convert_to_ui_rect(correct_string);
                    },
                    "color" => {
                        style.color = convert_to_color(value.clone());
                    },
                    "left" => {
                        style.left = convert_to_val(value.clone());
                    },
                    "right" => {
                        style.right = convert_to_val(value.clone());
                    },
                    "top" => {
                        style.top = convert_to_val(value.clone());
                    },
                    "bottom" => {
                        style.bottom = convert_to_val(value.clone());
                    },
                    "display" => {
                        style.display = convert_to_display(value.clone());
                    },
                    "position" => {
                        style.position_type = convert_to_position(value.clone());
                    },
                    "gap" => {
                        style.gap = convert_to_val(value.clone());
                    },
                    "justify-content" => {
                        style.justify_content = convert_to_bevy_justify_content(value.clone());
                    },
                    "align-items" => {
                        style.align_items = convert_to_bevy_align_items(value.clone());
                    },
                    "flex-direction" => {
                        style.flex_direction = convert_to_bevy_flex_direction(value.clone());
                    },
                    "background" | "background-color" => {
                        style.background = Some(Background {
                            color: convert_to_color(value.clone()).unwrap_or_else(|| Color::WHITE),
                            ..default()
                        });
                    },
                    "font-size" => {
                        style.font_size = convert_to_font_size(value.clone());
                    },
                    "border" => {
                        if let Some((rect, color)) = convert_css_border(value.clone()) {
                            style.border = Some(rect);
                            style.border_color = Some(color);
                        }
                    },
                    "border-left" => {
                        let val = convert_to_val(value.clone()).unwrap_or(Val::Px(0.));
                        let mut border = style.border.unwrap_or_default();
                        border.left = val;
                        style.border = Some(border);
                    },
                    "border-right" => {
                        let val = convert_to_val(value.clone()).unwrap_or(Val::Px(0.));
                        let mut border = style.border.unwrap_or_default();
                        border.right = val;
                        style.border = Some(border);
                    },
                    "border-top" => {
                        let val = convert_to_val(value.clone()).unwrap_or(Val::Px(0.));
                        let mut border = style.border.unwrap_or_default();
                        border.top = val;
                        style.border = Some(border);
                    },
                    "border-bottom" => {
                        let val = convert_to_val(value.clone()).unwrap_or(Val::Px(0.));
                        let mut border = style.border.unwrap_or_default();
                        border.bottom = val;
                        style.border = Some(border);
                    },
                    "border-radius" => {
                        style.border_radius = convert_to_radius(value.clone());
                    },
                    "border-color" => {
                        style.border_color = convert_to_color(value.clone());
                    },
                    "border-width" => {
                        style.border = convert_to_ui_rect(value.clone());
                    },
                    "box-shadow" => {
                        style.box_shadow = convert_to_bevy_box_shadow(value.clone());
                    },
                    "overflow" => {
                        style.overflow = convert_overflow(value.clone(), "all");
                    },
                    "overflow-y" => {
                        let val = convert_overflow(value.clone(), "y");
                        let mut overflow = style.overflow.unwrap_or_default();
                        overflow.y = val.unwrap_or(Overflow::default()).y;
                        style.overflow = Some(overflow);
                    },
                    "overflow-x" => {
                        let val = convert_overflow(value.clone(), "x");
                        let mut overflow = style.overflow.unwrap_or_default();
                        overflow.x = val.unwrap_or(Overflow::default()).x;
                        style.overflow = Some(overflow);
                    },
                    "text-wrap" => {
                        style.text_wrap = convert_to_bevy_line_break(value.clone());
                    }
                    _ => {}
                }
            }
            
            style_map.insert(selector, style);
        }
    }

    Ok(style_map)
}

pub fn convert_to_val(value: String) -> Option<Val> {
    let mut val = None;
    let trimmed = value.trim();
    if trimmed.ends_with("px") {
        let count = trimmed.replace("px", "").parse::<f32>().ok()?;
        val = Some(Val::Px(count));
    } else if trimmed.ends_with("%") {
        let count = trimmed.replace("%", "").parse::<f32>().ok()?;
        val = Some(Val::Percent(count));
    }
    val
}

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

pub fn convert_to_color(value: String) -> Option<Color> {
    let mut color = None;
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("transparent") {
        return Some(Colored::TRANSPARENT);
    }

    if trimmed.starts_with("#") {
        color = Some(Colored::hex_to_color(trimmed));
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
    
    color
}

pub fn convert_to_display(value: String) -> Option<Display> {
    let trimmed = value.trim();
    match trimmed {
        "flex" => { Some(Display::Flex) },
        "grid" => { Some(Display::Grid) },
        "block" => { Some(Display::Block) },
        "none" => { Some(Display::None) },
        _ => { Some(Display::Block) },
    }
}

pub fn convert_to_position(value: String) -> Option<PositionType> {
    let trimmed = value.trim();
    match trimmed {
        "relative" => Some(PositionType::Relative),
        "absolute" => Some(PositionType::Absolute),
        _ => { Some(PositionType::Relative) },
    }
}

pub fn convert_to_radius(value: String) -> Option<Radius> {
    let vals = parse_radius_values(&value)?;

    let (top_left, top_right, bottom_right, bottom_left) = match vals.len() {
        1 => (vals[0].clone(), vals[0].clone(), vals[0].clone(), vals[0].clone()),
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
            vals[2].clone()
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

pub fn convert_to_ui_rect(value: String) -> Option<UiRect> {
    let vals = parse_radius_values(&value)?;

    let (left, right, top, bottom) = match vals.len() {
        1 => (vals[0].clone(), vals[0].clone(), vals[0].clone(), vals[0].clone()),
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
            Val::Px(0.0)
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
        } else if trimmed.starts_with("#") || trimmed.starts_with("rgb(") || trimmed.starts_with("rgba(") {
            color = convert_to_color(trimmed.to_string())?;
        }
    }

    let (x, y, blur, spread) = match vals.len() {
        1 => (vals[0].clone(), vals[0].clone(), vals[0].clone(), vals[0].clone()),
        2 => (
            vals[0].clone(), // x
            vals[1].clone(), // y
            Val::Px(0.), // blur
            Val::Px(0.), // spread
        ),
        3 => (
            vals[0].clone(),
            vals[1].clone(),
            vals[2].clone(),
            Val::Px(0.0)
        ),
        4 => (
            vals[0].clone(),
            vals[1].clone(),
            vals[2].clone(),
            vals[3].clone(),
        ),
        _ => return None,
    };
    
    Some(BoxShadow::new(
        color,
        x,
        y,
        spread,
        blur,
    ))
}

pub fn convert_css_border(value: String) -> Option<(UiRect, Color)> {
    fn parse_val(input: &str) -> Option<Val> {
        if input.ends_with("px") {
            input.trim_end_matches("px").parse::<f32>().ok().map(Val::Px)
        } else if input.ends_with('%') {
            input.trim_end_matches('%').parse::<f32>().ok().map(Val::Percent)
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

pub fn convert_to_bevy_justify_content(value: String) -> Option<JustifyContent> {
    let trimmed = value.trim();
    match trimmed { 
        "start" => { Some(JustifyContent::Start) },
        "flex-start" => { Some(JustifyContent::FlexStart) },
        "end" => { Some(JustifyContent::End) },
        "flex-end" => { Some(JustifyContent::FlexStart) },
        "center" => { Some(JustifyContent::Center) },
        "space-between" => { Some(JustifyContent::SpaceBetween) },
        "space-around" => { Some(JustifyContent::SpaceAround) },
        "space-evenly" => { Some(JustifyContent::SpaceEvenly) },
        "stretch" => { Some(JustifyContent::Stretch) },
        _ => { Some(JustifyContent::default()) }
    }
}

pub fn convert_to_bevy_align_items(value: String) -> Option<AlignItems> {
    let trimmed = value.trim();
    match trimmed {
        "start" => { Some(AlignItems::Start) },
        "flex-start" => { Some(AlignItems::FlexStart) },
        "end" => { Some(AlignItems::End) },
        "flex-end" => { Some(AlignItems::FlexStart) },
        "center" => { Some(AlignItems::Center) },
        "baseline" => { Some(AlignItems::Baseline) },
        "stretch" => { Some(AlignItems::Stretch) },
        _ => { Some(AlignItems::default()) }
    }
}

pub fn convert_to_bevy_flex_direction(value: String) -> Option<FlexDirection> {
    let trimmed = value.trim();
    match trimmed {
        "row" => { Some(FlexDirection::Row) },
        "column" => { Some(FlexDirection::Column) },
        "row-reverse" => { Some(FlexDirection::RowReverse) },
        "column-reverse" => { Some(FlexDirection::ColumnReverse) },
        _ => { Some(FlexDirection::default()) }
    }
}

pub fn convert_to_bevy_line_break(value: String) -> Option<LineBreak> {
    let trimmed = value.trim();
    match trimmed {
        "wrap" | "stable" => { Some(LineBreak::AnyCharacter) },
        "nowrap" => { Some(LineBreak::NoWrap) },
        "pretty" | "balance" => { Some(LineBreak::WordBoundary) },
        "unset" => { Some(LineBreak::WordOrCharacter) },
        _=> { Some(LineBreak::default()) }
    }
}

pub fn convert_overflow(value: String, which: &str) -> Option<Overflow> {
    let trimmed = value.trim();
    let overflow_axis = match trimmed {
        "hidden" => { OverflowAxis::Hidden },
        "scroll" => { OverflowAxis::Scroll },
        "clip" => { OverflowAxis::Clip },
        "visible" => { OverflowAxis::Visible },
        _ => { OverflowAxis::default() }
    };
    
    if which == "*" || which == "all" || which == "both" {
        Some(Overflow { x: overflow_axis, y: overflow_axis })
    } else if which == "y" {
        Some(Overflow { y: overflow_axis, ..default() })
    } else if which == "x" {
        Some(Overflow { x: overflow_axis, ..default() })
    } else {
        return None;
    }
}

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