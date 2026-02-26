use crate::styles::paint::Colored;
use crate::styles::{
    AnimationDirection, AnimationKeyframe, AnimationSpec, BackdropFilter, Background,
    BackgroundAttachment, BackgroundPosition, BackgroundPositionValue, BackgroundSize,
    BackgroundSizeValue, CalcExpr, CalcUnit, CalcValue, CursorStyle, FontFamily, FontVal,
    FontWeight, GradientStop, GradientStopPosition, LinearGradient, MediaQueryCondition, ParsedCss,
    Radius, Style, StylePair, TransformStyle, TransitionProperty, TransitionSpec, TransitionTiming,
};
use bevy::prelude::*;
use bevy::ui::Val2;
use bevy::window::SystemCursorIcon;
use lightningcss::media_query::{
    MediaCondition, MediaFeature, MediaFeatureComparison, MediaFeatureId, MediaFeatureName,
    MediaFeatureValue, MediaList, MediaQuery, MediaType, Operator, Qualifier,
};
use lightningcss::rules::CssRule;
use lightningcss::rules::keyframes::KeyframeSelector;
use lightningcss::stylesheet::{ParserFlags, ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::traits::ToCss;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Loads a CSS file and parses it into a [`ParsedCss`] with selectors and keyframes.
///
/// This function reads a `.css` file from the disk, parses its rules, and converts each supported
/// CSS property into a Bevy-compatible [`Style`] representation. These styles can later be applied
/// to UI nodes.
///
/// # Parameters
/// - `path`: A path to the CSS file. If empty, it falls back to the default path `"assets/internal.css"`.
///
/// # Returns
/// - `ParsedCss` on success, containing styles and keyframes.
/// - Empty maps if the file cannot be parsed, but no panic occurs.
///
/// # Notes
/// - This uses [`grass_compiler::StyleSheet`] to parse the file.
/// - Supports standard properties like `width`, `height`, `padding`, `color`, `background`, `font-size`, `z-index`, etc.
/// - Ignores unsupported or malformed declarations silently.
pub fn load_css(css: &str) -> ParsedCss {
    let mut options = ParserOptions::default();
    options.flags.insert(ParserFlags::NESTING);

    let stylesheet = match StyleSheet::parse(css, options) {
        Ok(stylesheet) => stylesheet,
        Err(err) => {
            error!("Css Parsing failed: {:?}", err);
            return ParsedCss::default();
        }
    };

    let mut css_vars = HashMap::new();
    let mut style_map = HashMap::new();
    let mut keyframes_map: HashMap<String, Vec<AnimationKeyframe>> = HashMap::new();

    collect_css_rules(
        &stylesheet.rules,
        None,
        &mut css_vars,
        &mut style_map,
        &mut keyframes_map,
        None,
    );

    for keyframes in keyframes_map.values_mut() {
        keyframes.sort_by(|a, b| {
            a.progress
                .partial_cmp(&b.progress)
                .unwrap_or(Ordering::Equal)
        });
    }

    ParsedCss {
        styles: style_map,
        keyframes: keyframes_map,
    }
}

fn collect_css_rules(
    rules: &lightningcss::rules::CssRuleList<'_>,
    parent_selectors: Option<&[String]>,
    css_vars: &mut HashMap<String, String>,
    style_map: &mut HashMap<String, StylePair>,
    keyframes_map: &mut HashMap<String, Vec<AnimationKeyframe>>,
    media_condition: Option<MediaQueryCondition>,
) {
    for rule in &rules.0 {
        match rule {
            CssRule::Style(style_rule) => collect_style_rule(
                style_rule,
                parent_selectors,
                css_vars,
                style_map,
                keyframes_map,
                media_condition.clone(),
            ),
            CssRule::Nesting(nesting_rule) => collect_style_rule(
                &nesting_rule.style,
                parent_selectors,
                css_vars,
                style_map,
                keyframes_map,
                media_condition.clone(),
            ),
            CssRule::NestedDeclarations(nested_rule) => {
                if let Some(parent_selectors) = parent_selectors {
                    let mut style = StylePair {
                        origin: 0,
                        media: media_condition.clone(),
                        ..Default::default()
                    };
                    apply_declaration_list(
                        &mut style.normal,
                        &nested_rule.declarations.declarations,
                        css_vars,
                    );
                    apply_declaration_list(
                        &mut style.important,
                        &nested_rule.declarations.important_declarations,
                        css_vars,
                    );
                    for selector in parent_selectors {
                        merge_style_map(style_map, selector, &style);
                    }
                }
            }
            CssRule::Media(media_rule) => {
                let media = media_list_to_condition(&media_rule.query);
                let combined_media = combine_media_conditions(media_condition.clone(), Some(media));
                collect_css_rules(
                    &media_rule.rules,
                    parent_selectors,
                    css_vars,
                    style_map,
                    keyframes_map,
                    combined_media,
                );
            }
            CssRule::Keyframes(keyframes_rule) => {
                collect_keyframes_rule(keyframes_rule, css_vars, keyframes_map);
            }
            _ => {}
        }
    }
}

fn collect_style_rule(
    style_rule: &lightningcss::rules::style::StyleRule<'_>,
    parent_selectors: Option<&[String]>,
    css_vars: &mut HashMap<String, String>,
    style_map: &mut HashMap<String, StylePair>,
    keyframes_map: &mut HashMap<String, Vec<AnimationKeyframe>>,
    media_condition: Option<MediaQueryCondition>,
) {
    let selectors = selector_list_to_strings(&style_rule.selectors);
    if parent_selectors.is_none()
        && media_condition.is_none()
        && selectors.len() == 1
        && selectors[0].trim() == ":root"
    {
        collect_css_vars(&style_rule.declarations, css_vars);
        return;
    }

    let full_selectors = expand_selectors(parent_selectors, selectors);

    let mut style = StylePair {
        origin: 0,
        media: media_condition.clone(),
        ..Default::default()
    };
    let decls = &style_rule.declarations;

    apply_declaration_list(&mut style.normal, &decls.declarations, css_vars);
    apply_declaration_list(
        &mut style.important,
        &decls.important_declarations,
        css_vars,
    );

    for selector in &full_selectors {
        merge_style_map(style_map, selector, &style);
    }

    collect_css_rules(
        &style_rule.rules,
        Some(&full_selectors),
        css_vars,
        style_map,
        keyframes_map,
        media_condition,
    );
}

fn combine_media_conditions(
    left: Option<MediaQueryCondition>,
    right: Option<MediaQueryCondition>,
) -> Option<MediaQueryCondition> {
    match (left, right) {
        (None, None) => None,
        (Some(condition), None) | (None, Some(condition)) => Some(condition),
        (Some(MediaQueryCondition::Always), Some(condition))
        | (Some(condition), Some(MediaQueryCondition::Always)) => Some(condition),
        (Some(MediaQueryCondition::Never), _) | (_, Some(MediaQueryCondition::Never)) => {
            Some(MediaQueryCondition::Never)
        }
        (Some(left), Some(right)) => Some(MediaQueryCondition::And(vec![left, right])),
    }
}

fn media_list_to_condition(media_list: &MediaList<'_>) -> MediaQueryCondition {
    if media_list.media_queries.is_empty() {
        return MediaQueryCondition::Always;
    }

    let mut queries = Vec::with_capacity(media_list.media_queries.len());
    for query in &media_list.media_queries {
        queries.push(media_query_to_condition(query));
    }

    if queries.len() == 1 {
        queries.remove(0)
    } else {
        MediaQueryCondition::Or(queries)
    }
}

fn media_query_to_condition(query: &MediaQuery<'_>) -> MediaQueryCondition {
    let media_type_condition = match query.media_type {
        MediaType::All | MediaType::Screen => MediaQueryCondition::Always,
        _ => MediaQueryCondition::Never,
    };

    let condition = query
        .condition
        .as_ref()
        .map(media_condition_to_condition)
        .unwrap_or(MediaQueryCondition::Always);

    let mut combined = if matches!(media_type_condition, MediaQueryCondition::Always) {
        condition
    } else {
        MediaQueryCondition::And(vec![media_type_condition, condition])
    };

    if matches!(query.qualifier, Some(Qualifier::Not)) {
        combined = MediaQueryCondition::Not(Box::new(combined));
    }

    combined
}

fn media_condition_to_condition(condition: &MediaCondition<'_>) -> MediaQueryCondition {
    match condition {
        MediaCondition::Feature(feature) => media_feature_to_condition(feature),
        MediaCondition::Not(inner) => {
            MediaQueryCondition::Not(Box::new(media_condition_to_condition(inner)))
        }
        MediaCondition::Operation {
            operator,
            conditions,
        } => {
            let mut mapped = Vec::with_capacity(conditions.len());
            for part in conditions {
                mapped.push(media_condition_to_condition(part));
            }

            match operator {
                Operator::And => MediaQueryCondition::And(mapped),
                Operator::Or => MediaQueryCondition::Or(mapped),
            }
        }
        MediaCondition::Unknown(_) => MediaQueryCondition::Never,
    }
}

fn media_feature_to_condition(feature: &MediaFeature<'_>) -> MediaQueryCondition {
    match feature {
        MediaFeature::Plain { name, value } => match_media_feature(name, value, None),
        MediaFeature::Range {
            name,
            operator,
            value,
        } => match_media_feature(name, value, Some(*operator)),
        MediaFeature::Interval {
            name,
            start,
            start_operator,
            end,
            end_operator,
        } => {
            let start =
                match_media_feature(name, start, Some(opposite_comparison(*start_operator)));
            let end = match_media_feature(name, end, Some(*end_operator));
            MediaQueryCondition::And(vec![start, end])
        }
        MediaFeature::Boolean { .. } => MediaQueryCondition::Never,
    }
}

fn opposite_comparison(comparison: MediaFeatureComparison) -> MediaFeatureComparison {
    match comparison {
        MediaFeatureComparison::Equal => MediaFeatureComparison::Equal,
        MediaFeatureComparison::GreaterThan => MediaFeatureComparison::LessThan,
        MediaFeatureComparison::GreaterThanEqual => MediaFeatureComparison::LessThanEqual,
        MediaFeatureComparison::LessThan => MediaFeatureComparison::GreaterThan,
        MediaFeatureComparison::LessThanEqual => MediaFeatureComparison::GreaterThanEqual,
    }
}

fn match_media_feature(
    name: &MediaFeatureName<'_, MediaFeatureId>,
    value: &MediaFeatureValue<'_>,
    comparison: Option<MediaFeatureComparison>,
) -> MediaQueryCondition {
    match name {
        MediaFeatureName::Standard(MediaFeatureId::Width)
        | MediaFeatureName::Standard(MediaFeatureId::DeviceWidth) => {
            map_length_feature(value, comparison, true)
        }
        MediaFeatureName::Standard(MediaFeatureId::Height)
        | MediaFeatureName::Standard(MediaFeatureId::DeviceHeight) => {
            map_length_feature(value, comparison, false)
        }
        MediaFeatureName::Standard(MediaFeatureId::Orientation) => map_orientation_feature(value),
        _ => MediaQueryCondition::Never,
    }
}

fn map_length_feature(
    value: &MediaFeatureValue<'_>,
    comparison: Option<MediaFeatureComparison>,
    is_width: bool,
) -> MediaQueryCondition {
    let MediaFeatureValue::Length(length) = value else {
        return MediaQueryCondition::Never;
    };

    let Some(px) = length.to_px() else {
        return MediaQueryCondition::Never;
    };

    let op = comparison.unwrap_or(MediaFeatureComparison::Equal);
    if is_width {
        match op {
            MediaFeatureComparison::Equal => MediaQueryCondition::Width(px),
            MediaFeatureComparison::GreaterThan => {
                MediaQueryCondition::Not(Box::new(MediaQueryCondition::MaxWidth(px)))
            }
            MediaFeatureComparison::GreaterThanEqual => MediaQueryCondition::MinWidth(px),
            MediaFeatureComparison::LessThan => {
                MediaQueryCondition::Not(Box::new(MediaQueryCondition::MinWidth(px)))
            }
            MediaFeatureComparison::LessThanEqual => MediaQueryCondition::MaxWidth(px),
        }
    } else {
        match op {
            MediaFeatureComparison::Equal => MediaQueryCondition::Height(px),
            MediaFeatureComparison::GreaterThan => {
                MediaQueryCondition::Not(Box::new(MediaQueryCondition::MaxHeight(px)))
            }
            MediaFeatureComparison::GreaterThanEqual => MediaQueryCondition::MinHeight(px),
            MediaFeatureComparison::LessThan => {
                MediaQueryCondition::Not(Box::new(MediaQueryCondition::MinHeight(px)))
            }
            MediaFeatureComparison::LessThanEqual => MediaQueryCondition::MaxHeight(px),
        }
    }
}

fn map_orientation_feature(value: &MediaFeatureValue<'_>) -> MediaQueryCondition {
    let MediaFeatureValue::Ident(ident) = value else {
        return MediaQueryCondition::Never;
    };

    match ident.0.as_ref().to_ascii_lowercase().as_str() {
        "landscape" => MediaQueryCondition::OrientationLandscape,
        "portrait" => MediaQueryCondition::OrientationPortrait,
        _ => MediaQueryCondition::Never,
    }
}

fn collect_keyframes_rule(
    keyframes_rule: &lightningcss::rules::keyframes::KeyframesRule<'_>,
    css_vars: &HashMap<String, String>,
    keyframes_map: &mut HashMap<String, Vec<AnimationKeyframe>>,
) {
    let name = match keyframes_rule.name.to_css_string(PrinterOptions::default()) {
        Ok(name) => name,
        Err(_) => return,
    };

    let entry = keyframes_map.entry(name).or_default();

    for keyframe in &keyframes_rule.keyframes {
        let mut style = Style::default();
        let decls = &keyframe.declarations;

        apply_declaration_list(&mut style, &decls.declarations, css_vars);
        apply_declaration_list(&mut style, &decls.important_declarations, css_vars);

        for selector in &keyframe.selectors {
            if let Some(progress) = keyframe_selector_progress(selector) {
                entry.push(AnimationKeyframe {
                    progress,
                    style: style.clone(),
                });
            }
        }
    }
}

fn selector_list_to_strings(selectors: &lightningcss::selector::SelectorList<'_>) -> Vec<String> {
    let mut out = Vec::new();
    for selector in selectors.0.iter() {
        if let Ok(s) = selector.to_css_string(PrinterOptions::default()) {
            out.push(s);
        }
    }
    out
}

fn expand_selectors(parent_selectors: Option<&[String]>, selectors: Vec<String>) -> Vec<String> {
    let Some(parents) = parent_selectors else {
        return selectors
            .into_iter()
            .map(|s| normalize_selector(&s))
            .collect();
    };

    let mut expanded = Vec::new();
    for parent in parents {
        for selector in &selectors {
            let combined = combine_selector(parent, selector);
            expanded.push(normalize_selector(&combined));
        }
    }
    expanded
}

fn combine_selector(parent: &str, nested: &str) -> String {
    let trimmed = nested.trim();
    if trimmed.contains('&') {
        return trimmed.replace('&', parent);
    }
    if trimmed.starts_with('>') {
        return format!("{parent} {trimmed}");
    }
    format!("{parent} {trimmed}")
}

fn normalize_selector(selector: &str) -> String {
    selector
        .replace('>', " > ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect_css_vars(
    decls: &lightningcss::declaration::DeclarationBlock<'_>,
    css_vars: &mut HashMap<String, String>,
) {
    for property in decls
        .declarations
        .iter()
        .chain(decls.important_declarations.iter())
    {
        let property_id = property.property_id();
        let name = property_id.name();
        if name.starts_with("--") {
            if let Ok(value) = property.value_to_css_string(PrinterOptions::default()) {
                css_vars.insert(name.to_string(), value);
            }
        }
    }
}

fn apply_declaration_list(
    style: &mut Style,
    declarations: &[lightningcss::properties::Property<'_>],
    css_vars: &HashMap<String, String>,
) {
    for property in declarations {
        let property_id = property.property_id();
        let name = property_id.name();

        let value = match property.value_to_css_string(PrinterOptions::default()) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let resolved = resolve_var(&value, css_vars);
        apply_property_to_style(style, name, &resolved);
    }
}

fn merge_style_map(style_map: &mut HashMap<String, StylePair>, selector: &str, style: &StylePair) {
    let key = selector_map_key(selector, style.media.as_ref());

    style_map
        .entry(key)
        .and_modify(|existing| {
            existing.normal.merge(&style.normal);
            existing.important.merge(&style.important);
        })
        .or_insert_with(|| {
            let mut cloned = style.clone();
            cloned.selector = selector.to_string();
            cloned
        });
}

fn selector_map_key(selector: &str, media: Option<&MediaQueryCondition>) -> String {
    match media {
        Some(media) => format!("{selector}@@media({})", media.cache_key()),
        None => selector.to_string(),
    }
}

/// Resolves a `var(...)` reference using the collected CSS variables.
fn resolve_var(value: &str, css_vars: &HashMap<String, String>) -> String {
    resolve_var_inner(value, css_vars, 0)
}

fn resolve_var_inner(value: &str, css_vars: &HashMap<String, String>, depth: u8) -> String {
    if depth >= 8 {
        return value.to_string();
    }

    let trimmed = value.trim();
    if !trimmed.starts_with("var(") || !trimmed.ends_with(')') {
        return value.to_string();
    }

    let inner = &trimmed[4..trimmed.len() - 1];
    let (name, fallback) = split_var_args(inner);
    let name = name.trim();

    if let Some(var_value) = css_vars.get(name) {
        return resolve_var_inner(var_value, css_vars, depth + 1);
    }

    if let Some(fallback) = fallback {
        return resolve_var_inner(fallback.trim(), css_vars, depth + 1);
    }

    value.to_string()
}

fn split_var_args(input: &str) -> (&str, Option<&str>) {
    let mut depth = 0u8;
    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let (name, fallback) = input.split_at(idx);
                return (name, Some(&fallback[1..]));
            }
            _ => {}
        }
    }
    (input, None)
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
pub fn apply_property_to_style(style: &mut Style, name: &str, value: &str) {
    match name {
        "width" => apply_length_property(value, &mut style.width, &mut style.width_calc),
        "min-width" => {
            apply_length_property(value, &mut style.min_width, &mut style.min_width_calc)
        }
        "max-width" => {
            apply_length_property(value, &mut style.max_width, &mut style.max_width_calc)
        }
        "height" => apply_length_property(value, &mut style.height, &mut style.height_calc),
        "min-height" => {
            apply_length_property(value, &mut style.min_height, &mut style.min_height_calc)
        }
        "max-height" => {
            apply_length_property(value, &mut style.max_height, &mut style.max_height_calc)
        }

        "padding" => style.padding = convert_to_ui_rect(value.to_string()),
        "padding-left" => apply_rect_side(value, RectSide::Left, &mut style.padding),
        "padding-right" => apply_rect_side(value, RectSide::Right, &mut style.padding),
        "padding-top" => apply_rect_side(value, RectSide::Top, &mut style.padding),
        "padding-bottom" => apply_rect_side(value, RectSide::Bottom, &mut style.padding),

        "margin" => style.margin = convert_to_ui_rect(value.to_string()),
        "margin-left" => apply_rect_side(value, RectSide::Left, &mut style.margin),
        "margin-right" => apply_rect_side(value, RectSide::Right, &mut style.margin),
        "margin-top" => apply_rect_side(value, RectSide::Top, &mut style.margin),
        "margin-bottom" => apply_rect_side(value, RectSide::Bottom, &mut style.margin),

        "color" => style.color = convert_to_color(value.to_string()),

        "left" => apply_length_property(value, &mut style.left, &mut style.left_calc),
        "right" => apply_length_property(value, &mut style.right, &mut style.right_calc),
        "top" => apply_length_property(value, &mut style.top, &mut style.top_calc),
        "bottom" => apply_length_property(value, &mut style.bottom, &mut style.bottom_calc),

        "display" => style.display = convert_to_display(value.to_string()),
        "position" => style.position_type = convert_to_position(value.to_string()),
        "box-sizing" => style.box_sizing = convert_to_box_sizing(value.to_string()),
        "scroll-width" => style.scrollbar_width = convert_to_f32(value.to_string()),
        "gap" => apply_length_property(value, &mut style.gap, &mut style.gap_calc),
        "row-gap" => apply_length_property(value, &mut style.row_gap, &mut style.row_gap_calc),
        "column-gap" => {
            apply_length_property(value, &mut style.column_gap, &mut style.column_gap_calc)
        }
        "transition" => style.transition = parse_transition(value),
        "transform" => apply_transform_functions(value, &mut style.transform),
        "animation" => style.animation = parse_animation(value),
        "animation-name" => apply_animation_name(style, value),
        "animation-duration" => apply_animation_duration(style, value),
        "animation-delay" => apply_animation_delay(style, value),
        "animation-timing-function" => apply_animation_timing(style, value),
        "animation-iteration-count" => apply_animation_iterations(style, value),
        "animation-direction" => apply_animation_direction(style, value),
        "justify-content" => {
            style.justify_content = convert_to_bevy_justify_content(value.to_string())
        }
        "align-items" => style.align_items = convert_to_bevy_align_items(value.to_string()),
        "flex-direction" => {
            style.flex_direction = convert_to_bevy_flex_direction(value.to_string())
        }
        "flex-grow" => style.flex_grow = value.trim().parse::<f32>().ok(),
        "flex-shrink" => style.flex_shrink = value.trim().parse::<f32>().ok(),
        "flex-basis" => {
            apply_length_property(value, &mut style.flex_basis, &mut style.flex_basis_calc)
        }
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
            apply_background_color(style, value);
        }
        "background" => {
            style.background_position = None;
            style.background_size = None;
            style.background_attachment = None;
            style.background = convert_to_background(value.to_string(), true);
        }
        "backdrop-filter" | "-webkit-backdrop-filter" => {
            style.backdrop_filter = parse_backdrop_filter(value);
        }
        "background-image" => {
            apply_background_image(style, value);
        }
        "background-position" => {
            style.background_position = parse_background_position(value);
        }
        "background-size" => {
            style.background_size = parse_background_size(value);
        }
        "background-attachment" => {
            style.background_attachment = parse_background_attachment(value);
        }

        "font-size" => style.font_size = convert_to_font_size(value.to_string()),
        "font-family" => {
            style.font_family = Some(FontFamily(
                value
                    .trim()
                    .replace(" ", "")
                    .replace("\"", "")
                    .replace("'", "")
                    .to_string(),
            ));
        }
        "font-weight" => {
            style.font_weight = convert_to_font_weight(value.to_string());
        }

        "border" => {
            if let Some((rect, color)) = convert_css_border(value.to_string()) {
                style.border = Some(rect);
                style.border_color = Some(color);
            }
        }
        "border-left" => apply_border_side(
            value,
            RectSide::Left,
            &mut style.border,
            &mut style.border_color,
        ),
        "border-right" => apply_border_side(
            value,
            RectSide::Right,
            &mut style.border,
            &mut style.border_color,
        ),
        "border-top" => apply_border_side(
            value,
            RectSide::Top,
            &mut style.border,
            &mut style.border_color,
        ),
        "border-bottom" => apply_border_side(
            value,
            RectSide::Bottom,
            &mut style.border,
            &mut style.border_color,
        ),
        "border-radius" => style.border_radius = convert_to_radius(value.to_string()),
        "border-color" => style.border_color = convert_to_color(value.to_string()),
        "border-width" => style.border = convert_to_ui_rect(value.to_string()),

        "box-shadow" => style.box_shadow = convert_to_bevy_box_shadow(value.to_string()),

        "overflow" => style.overflow = convert_overflow(value.to_string(), "all"),
        "overflow-y" => apply_overflow_axis(style, value, OverflowAxisSelector::Y),
        "overflow-x" => apply_overflow_axis(style, value, OverflowAxisSelector::X),

        "text-wrap" => style.text_wrap = convert_to_bevy_line_break(value.to_string()),
        "z-index" => style.z_index = convert_to_i32(value.to_string()),
        "pointer-events" => style.pointer_events = convert_to_bevy_pick_able(value.to_string()),
        "cursor" => style.cursor = convert_to_cursor_style(value.to_string()),

        _ => {}
    }
}

#[derive(Clone, Copy)]
enum RectSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
enum OverflowAxisSelector {
    X,
    Y,
}

fn apply_rect_side(value: &str, side: RectSide, target: &mut Option<UiRect>) {
    *target = rect_from_side(value, side);
}

fn apply_border_side(
    value: &str,
    side: RectSide,
    target: &mut Option<UiRect>,
    color_target: &mut Option<Color>,
) {
    let parts: Vec<&str> = value.split_whitespace().collect();

    let val = parts
        .iter()
        .find_map(|token| convert_to_val((*token).to_string()))
        .unwrap_or(Val::Px(0.0));
    let mut rect = target.unwrap_or_default();
    set_rect_side(&mut rect, side, val);
    *target = Some(rect);

    for index in 0..parts.len() {
        let color_candidate = parts[index..].join(" ");
        if let Some(parsed_color) = convert_to_color(color_candidate) {
            *color_target = Some(parsed_color);
            break;
        }
    }
}

fn apply_overflow_axis(style: &mut Style, value: &str, axis: OverflowAxisSelector) {
    let val = match axis {
        OverflowAxisSelector::X => convert_overflow(value.to_string(), "x"),
        OverflowAxisSelector::Y => convert_overflow(value.to_string(), "y"),
    };
    let mut overflow = style.overflow.unwrap_or_default();
    match axis {
        OverflowAxisSelector::X => overflow.x = val.unwrap_or_default().x,
        OverflowAxisSelector::Y => overflow.y = val.unwrap_or_default().y,
    }
    style.overflow = Some(overflow);
}

fn rect_from_side(value: &str, side: RectSide) -> Option<UiRect> {
    let val = parse_single_rect_value(value)?;
    let zero = Val::Px(0.0);
    let mut rect = UiRect {
        left: zero,
        right: zero,
        top: zero,
        bottom: zero,
    };
    set_rect_side(&mut rect, side, val);
    Some(rect)
}

fn parse_single_rect_value(value: &str) -> Option<Val> {
    let vals = parse_radius_values(value)?;
    if vals.len() == 1 {
        vals.into_iter().next()
    } else {
        None
    }
}

fn set_rect_side(rect: &mut UiRect, side: RectSide, value: Val) {
    match side {
        RectSide::Left => rect.left = value,
        RectSide::Right => rect.right = value,
        RectSide::Top => rect.top = value,
        RectSide::Bottom => rect.bottom = value,
    }
}

/// Converts a keyframe selector into a normalized progress value.
fn keyframe_selector_progress(selector: &KeyframeSelector) -> Option<f32> {
    match selector {
        KeyframeSelector::Percentage(p) => Some(p.0.clamp(0.0, 1.0)),
        KeyframeSelector::From => Some(0.0),
        KeyframeSelector::To => Some(1.0),
        KeyframeSelector::TimelineRangePercentage(_) => None,
    }
}

/// Parses a CSS time token into seconds.
fn parse_time_seconds(token: &str) -> Option<f32> {
    let token = token.trim().to_ascii_lowercase();
    if let Some(value) = token.strip_suffix("ms") {
        return value.trim().parse::<f32>().ok().map(|v| v / 1000.0);
    }

    if let Some(value) = token.strip_suffix('s') {
        return value.trim().parse::<f32>().ok();
    }

    None
}

/// Parses CSS transition shorthand into a `TransitionSpec`.
fn parse_transition(value: &str) -> Option<TransitionSpec> {
    let mut spec = TransitionSpec::default();
    let mut has_duration = false;
    let mut has_delay = false;
    let mut has_property = false;

    for token in value.split_whitespace() {
        match token.to_ascii_lowercase().as_str() {
            "all" => {
                spec.properties = vec![TransitionProperty::All];
                has_property = true;
                continue;
            }
            "color" => {
                spec.properties = vec![TransitionProperty::Color];
                has_property = true;
                continue;
            }
            "background" | "background-color" => {
                spec.properties = vec![TransitionProperty::Background];
                has_property = true;
                continue;
            }
            "transform" | "scale" | "translate" | "translation" | "rotate" | "rotation" => {
                spec.properties = vec![TransitionProperty::Transform];
                has_property = true;
                continue;
            }
            _ => {}
        }

        if let Some(time) = parse_time_seconds(token) {
            if !has_duration {
                spec.duration = time;
                has_duration = true;
            } else if !has_delay {
                spec.delay = time;
                has_delay = true;
            }
            continue;
        }

        if let Some(timing) = TransitionTiming::from_name(token) {
            spec.timing = timing;
        }
    }

    if has_duration || has_property {
        Some(spec)
    } else {
        None
    }
}

/// Parses S animation shorthand into an `AnimationSpec`.
fn parse_animation(value: &str) -> Option<AnimationSpec> {
    let mut spec = AnimationSpec::default();
    let mut has_duration = false;
    let mut has_delay = false;
    let mut has_name = false;

    let segment = value.split(',').next().unwrap_or(value);
    for token in segment.split_whitespace() {
        let lower = token.to_ascii_lowercase();

        if let Some(time) = parse_time_seconds(token) {
            if !has_duration {
                spec.duration = time;
                has_duration = true;
            } else if !has_delay {
                spec.delay = time;
                has_delay = true;
            }
            continue;
        }

        if let Some(timing) = TransitionTiming::from_name(token) {
            spec.timing = timing;
            continue;
        }

        if let Some(direction) = AnimationDirection::from_name(token) {
            spec.direction = direction;
            continue;
        }

        if lower == "infinite" {
            spec.iterations = None;
            continue;
        }

        if let Ok(count) = token.parse::<f32>() {
            spec.iterations = Some(count.max(0.0));
            continue;
        }

        if !has_name && lower != "none" {
            spec.name = token.to_string();
            has_name = true;
        }
    }

    if spec.name.is_empty() {
        None
    } else {
        Some(spec)
    }
}

/// Ensures a style has an animation spec and returns a mutable reference to it.
fn ensure_animation_spec(style: &mut Style) -> &mut AnimationSpec {
    style.animation.get_or_insert_with(AnimationSpec::default)
}

#[derive(Clone, Copy)]
enum AnimationTimeField {
    Duration,
    Delay,
}

fn apply_animation_time(style: &mut Style, value: &str, field: AnimationTimeField) {
    if let Some(time) = parse_time_seconds(value) {
        let spec = ensure_animation_spec(style);
        match field {
            AnimationTimeField::Duration => spec.duration = time,
            AnimationTimeField::Delay => spec.delay = time,
        }
    }
}

/// Applies an animation name, clearing animation when set to none.
fn apply_animation_name(style: &mut Style, value: &str) {
    let name = value.trim();
    if name.eq_ignore_ascii_case("none") || name.is_empty() {
        style.animation = None;
        return;
    }

    let spec = ensure_animation_spec(style);
    spec.name = name.to_string();
}

/// Applies an animation duration value.
fn apply_animation_duration(style: &mut Style, value: &str) {
    apply_animation_time(style, value, AnimationTimeField::Duration);
}

/// Applies an animation delay value.
fn apply_animation_delay(style: &mut Style, value: &str) {
    apply_animation_time(style, value, AnimationTimeField::Delay);
}

/// Applies an animation timing function value.
fn apply_animation_timing(style: &mut Style, value: &str) {
    if let Some(timing) = TransitionTiming::from_name(value) {
        let spec = ensure_animation_spec(style);
        spec.timing = timing;
    }
}

/// Applies an animation iteration count value.
fn apply_animation_iterations(style: &mut Style, value: &str) {
    let trimmed = value.trim();
    let spec = ensure_animation_spec(style);

    if trimmed.eq_ignore_ascii_case("infinite") {
        spec.iterations = None;
        return;
    }

    if let Ok(count) = trimmed.parse::<f32>() {
        spec.iterations = Some(count.max(0.0));
    }
}

/// Applies an animation direction value.
fn apply_animation_direction(style: &mut Style, value: &str) {
    if let Some(direction) = AnimationDirection::from_name(value) {
        let spec = ensure_animation_spec(style);
        spec.direction = direction;
    }
}

/// Parses a font-weight value into the corresponding enum.
fn convert_to_font_weight(value: String) -> Option<FontWeight> {
    let in_value = value.trim();
    if in_value.is_empty() {
        return None;
    }

    // 1) Try numeric first (CSS-style: 1–1000, we clamp logically)
    if let Ok(num) = in_value.parse::<u16>() {
        return FontWeight::from_number(num);
    }

    // 2) Try name-based (case-insensitive)
    FontWeight::from_name(in_value)
}

struct MathParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> MathParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_expression(&mut self) -> Option<CalcValue> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_ws();
            if self.consume('+') {
                let right = self.parse_term()?;
                left = add_values(left, right)?;
            } else if self.consume('-') {
                let right = self.parse_term()?;
                left = sub_values(left, right)?;
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_term(&mut self) -> Option<CalcValue> {
        let mut left = self.parse_factor()?;
        loop {
            self.skip_ws();
            if self.consume('*') {
                let right = self.parse_factor()?;
                left = mul_values(left, right)?;
            } else if self.consume('/') {
                let right = self.parse_factor()?;
                left = div_values(left, right)?;
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_factor(&mut self) -> Option<CalcValue> {
        self.skip_ws();
        if self.consume('+') {
            return self.parse_factor();
        }
        if self.consume('-') {
            let inner = self.parse_factor()?;
            return Some(CalcValue::new(-inner.value, inner.unit));
        }
        if self.consume('(') {
            let value = self.parse_expression()?;
            self.expect(')')?;
            return Some(value);
        }

        if let Some(name) = self.parse_identifier() {
            self.skip_ws();
            if self.consume('(') {
                return self.parse_function(&name);
            }
            return None;
        }

        self.parse_number()
    }

    fn parse_function(&mut self, name: &str) -> Option<CalcValue> {
        match name {
            "calc" => {
                let value = self.parse_expression()?;
                self.expect(')')?;
                Some(value)
            }
            "min" | "max" => {
                let mut values = Vec::new();
                loop {
                    let value = self.parse_expression()?;
                    values.push(value);
                    self.skip_ws();
                    if self.consume(',') {
                        continue;
                    }
                    break;
                }
                self.expect(')')?;
                reduce_min_max(name == "min", &values)
            }
            "sin" => {
                let value = self.parse_expression()?;
                self.expect(')')?;
                let radians = to_radians(value)?;
                Some(CalcValue::new(radians.sin(), CalcUnit::None))
            }
            "minmax" => {
                let _ = self.parse_expression()?;
                self.skip_ws();
                if self.consume(',') {
                    let _ = self.parse_expression()?;
                }
                self.expect(')')?;
                None
            }
            _ => None,
        }
    }

    fn parse_number(&mut self) -> Option<CalcValue> {
        self.skip_ws();
        let start = self.pos;
        let mut has_digit = false;
        let mut seen_dot = false;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                has_digit = true;
                self.pos += ch.len_utf8();
            } else if ch == '.' && !seen_dot {
                seen_dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }

        if !has_digit {
            return None;
        }

        let number_str = &self.input[start..self.pos];
        let value = number_str.parse::<f32>().ok()?;

        let unit = if self.consume('%') {
            CalcUnit::Percent
        } else if let Some(unit_str) = self.parse_unit() {
            match unit_str.as_str() {
                "px" => CalcUnit::Px,
                "rem" => CalcUnit::Rem,
                "vw" => CalcUnit::Vw,
                "vh" => CalcUnit::Vh,
                "vmin" => CalcUnit::VMin,
                "vmax" => CalcUnit::VMax,
                "deg" => CalcUnit::Deg,
                "rad" => CalcUnit::Rad,
                "turn" => CalcUnit::Turn,
                "fr" => CalcUnit::Fr,
                _ => return None,
            }
        } else {
            CalcUnit::None
        };

        Some(CalcValue::new(value, unit))
    }

    fn parse_identifier(&mut self) -> Option<String> {
        self.skip_ws();
        let start = self.pos;
        let mut found = false;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphabetic() || ch == '-' {
                found = true;
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        if !found {
            return None;
        }
        Some(self.input[start..self.pos].to_ascii_lowercase())
    }

    fn parse_unit(&mut self) -> Option<String> {
        let start = self.pos;
        let mut found = false;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphabetic() {
                found = true;
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        if !found {
            return None;
        }
        Some(self.input[start..self.pos].to_ascii_lowercase())
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.pos += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: char) -> Option<()> {
        self.skip_ws();
        if self.consume(expected) {
            Some(())
        } else {
            None
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

fn parse_math_value(input: &str) -> Option<CalcValue> {
    let mut parser = MathParser::new(input);
    let value = parser.parse_expression()?;
    parser.skip_ws();
    if parser.is_eof() { Some(value) } else { None }
}

fn add_values(a: CalcValue, b: CalcValue) -> Option<CalcValue> {
    if a.unit == b.unit {
        Some(CalcValue::new(a.value + b.value, a.unit))
    } else {
        None
    }
}

fn sub_values(a: CalcValue, b: CalcValue) -> Option<CalcValue> {
    if a.unit == b.unit {
        Some(CalcValue::new(a.value - b.value, a.unit))
    } else {
        None
    }
}

fn mul_values(a: CalcValue, b: CalcValue) -> Option<CalcValue> {
    match (a.unit, b.unit) {
        (CalcUnit::None, unit) => Some(CalcValue::new(a.value * b.value, unit)),
        (unit, CalcUnit::None) => Some(CalcValue::new(a.value * b.value, unit)),
        _ => None,
    }
}

fn div_values(a: CalcValue, b: CalcValue) -> Option<CalcValue> {
    if b.value == 0.0 {
        return None;
    }
    match (a.unit, b.unit) {
        (unit, CalcUnit::None) => Some(CalcValue::new(a.value / b.value, unit)),
        _ => None,
    }
}

fn reduce_min_max(is_min: bool, values: &[CalcValue]) -> Option<CalcValue> {
    let first = values.first().copied()?;
    let mut best = first;
    for value in values.iter().copied().skip(1) {
        if value.unit != best.unit {
            return None;
        }
        if is_min {
            if value.value < best.value {
                best = value;
            }
        } else if value.value > best.value {
            best = value;
        }
    }
    Some(best)
}

fn to_radians(value: CalcValue) -> Option<f32> {
    match value.unit {
        CalcUnit::None => Some(value.value),
        CalcUnit::Deg => Some(value.value.to_radians()),
        CalcUnit::Rad => Some(value.value),
        CalcUnit::Turn => Some(value.value * std::f32::consts::TAU),
        _ => None,
    }
}

struct CalcParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> CalcParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_expression(&mut self) -> Option<CalcExpr> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_ws();
            if self.consume('+') {
                let right = self.parse_term()?;
                left = CalcExpr::Add(Box::new(left), Box::new(right));
            } else if self.consume('-') {
                let right = self.parse_term()?;
                left = CalcExpr::Sub(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_term(&mut self) -> Option<CalcExpr> {
        let mut left = self.parse_factor()?;
        loop {
            self.skip_ws();
            if self.consume('*') {
                let right = self.parse_factor()?;
                left = CalcExpr::Mul(Box::new(left), Box::new(right));
            } else if self.consume('/') {
                let right = self.parse_factor()?;
                left = CalcExpr::Div(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_factor(&mut self) -> Option<CalcExpr> {
        self.skip_ws();
        if self.consume('+') {
            return self.parse_factor();
        }
        if self.consume('-') {
            let inner = self.parse_factor()?;
            return Some(CalcExpr::Mul(
                Box::new(CalcExpr::Value(CalcValue::new(-1.0, CalcUnit::None))),
                Box::new(inner),
            ));
        }
        if self.consume('(') {
            let expr = self.parse_expression()?;
            self.expect(')')?;
            return Some(expr);
        }

        if let Some(name) = self.parse_identifier() {
            self.skip_ws();
            if self.consume('(') {
                return self.parse_function(&name);
            }
            return None;
        }

        self.parse_number()
    }

    fn parse_function(&mut self, name: &str) -> Option<CalcExpr> {
        match name {
            "calc" => {
                let expr = self.parse_expression()?;
                self.expect(')')?;
                Some(expr)
            }
            "min" | "max" => {
                let mut values = Vec::new();
                loop {
                    let expr = self.parse_expression()?;
                    values.push(expr);
                    self.skip_ws();
                    if self.consume(',') {
                        continue;
                    }
                    break;
                }
                self.expect(')')?;
                if name == "min" {
                    Some(CalcExpr::Min(values))
                } else {
                    Some(CalcExpr::Max(values))
                }
            }
            "sin" => {
                let expr = self.parse_expression()?;
                self.expect(')')?;
                Some(CalcExpr::Sin(Box::new(expr)))
            }
            "minmax" => {
                let _ = self.parse_expression()?;
                self.skip_ws();
                if self.consume(',') {
                    let _ = self.parse_expression()?;
                }
                self.expect(')')?;
                None
            }
            _ => None,
        }
    }

    fn parse_number(&mut self) -> Option<CalcExpr> {
        self.skip_ws();
        let start = self.pos;
        let mut has_digit = false;
        let mut seen_dot = false;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                has_digit = true;
                self.pos += ch.len_utf8();
            } else if ch == '.' && !seen_dot {
                seen_dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }

        if !has_digit {
            return None;
        }

        let number_str = &self.input[start..self.pos];
        let value = number_str.parse::<f32>().ok()?;

        let unit = if self.consume('%') {
            CalcUnit::Percent
        } else if let Some(unit_str) = self.parse_unit() {
            match unit_str.as_str() {
                "px" => CalcUnit::Px,
                "rem" => CalcUnit::Rem,
                "vw" => CalcUnit::Vw,
                "vh" => CalcUnit::Vh,
                "vmin" => CalcUnit::VMin,
                "vmax" => CalcUnit::VMax,
                "deg" => CalcUnit::Deg,
                "rad" => CalcUnit::Rad,
                "turn" => CalcUnit::Turn,
                "fr" => CalcUnit::Fr,
                _ => return None,
            }
        } else {
            CalcUnit::None
        };

        Some(CalcExpr::Value(CalcValue::new(value, unit)))
    }

    fn parse_identifier(&mut self) -> Option<String> {
        self.skip_ws();
        let start = self.pos;
        let mut found = false;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphabetic() || ch == '-' {
                found = true;
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        if !found {
            return None;
        }
        Some(self.input[start..self.pos].to_ascii_lowercase())
    }

    fn parse_unit(&mut self) -> Option<String> {
        let start = self.pos;
        let mut found = false;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphabetic() {
                found = true;
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        if !found {
            return None;
        }
        Some(self.input[start..self.pos].to_ascii_lowercase())
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.pos += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: char) -> Option<()> {
        self.skip_ws();
        if self.consume(expected) {
            Some(())
        } else {
            None
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

fn parse_calc_expr(input: &str) -> Option<CalcExpr> {
    let mut parser = CalcParser::new(input);
    let expr = parser.parse_expression()?;
    parser.skip_ws();
    if parser.is_eof() { Some(expr) } else { None }
}

fn apply_length_property(value: &str, dest: &mut Option<Val>, dest_calc: &mut Option<CalcExpr>) {
    if let Some(val) = convert_to_val(value.to_string()) {
        *dest = Some(val);
        *dest_calc = None;
        return;
    }

    if let Some(expr) = parse_calc_expr(value) {
        *dest = None;
        *dest_calc = Some(expr);
    }
}

/// Converts a string representation of a CSS value into a Bevy [`Val`].
///
/// # Supported Formats
/// - `"100px"` → `Val::Px(100.0)`
/// - `"75%"` → `Val::Percent(75.0)`
/// - `"10vw"` → `Val::Vw(10.0)` (and `vh`/`vmin`/`vmax`)
///
/// # Parameters
/// - `value`: A [`String`] representing a dimension value (e.g. `"20px"`, `"50%"`).
///
/// # Returns
/// - `Some(Val)` if parsing succeeds.
/// - `None` if the format is invalid or cannot be parsed.
///
pub fn convert_to_val(value: String) -> Option<Val> {
    let parsed = parse_math_value(value.trim())?;
    match parsed.unit {
        CalcUnit::Px => Some(Val::Px(parsed.value)),
        CalcUnit::Percent => Some(Val::Percent(parsed.value)),
        CalcUnit::Vw => Some(Val::Vw(parsed.value)),
        CalcUnit::Vh => Some(Val::Vh(parsed.value)),
        CalcUnit::VMin => Some(Val::VMin(parsed.value)),
        CalcUnit::VMax => Some(Val::VMax(parsed.value)),
        CalcUnit::None if parsed.value == 0.0 => Some(Val::Px(0.0)),
        _ => None,
    }
}

/// Parses a two-value length into a `Val2`, defaulting the second to zero.
fn parse_val2(value: &str) -> Option<Val2> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [x] => convert_to_val(x.to_string()).map(|x_val| Val2::new(x_val, Val::Px(0.0))),
        [x, y, ..] => {
            let x_val = convert_to_val((*x).to_string())?;
            let y_val = convert_to_val((*y).to_string())?;
            Some(Val2::new(x_val, y_val))
        }
        _ => None,
    }
}

/// Applies parsed transform functions to the given `TransformStyle`.
fn apply_transform_functions(value: &str, transform: &mut TransformStyle) {
    let mut remainder = value.trim();
    while let Some(open_idx) = remainder.find('(') {
        let name = remainder[..open_idx].trim().to_ascii_lowercase();
        let after_open = &remainder[open_idx + 1..];
        let Some(close_idx) = after_open.find(')') else {
            break;
        };
        let args = after_open[..close_idx].trim();
        let normalized = args.replace(',', " ");

        match name.as_str() {
            "translate" | "translation" => {
                if let Some(val) = parse_val2(normalized.as_str()) {
                    transform.translation = Some(val);
                }
            }
            "translatex" => {
                if let Some(val) = convert_to_val(normalized.clone()) {
                    transform.translation_x = Some(val);
                }
            }
            "translatey" => {
                if let Some(val) = convert_to_val(normalized.clone()) {
                    transform.translation_y = Some(val);
                }
            }
            "scale" => {
                if let Some(val) = parse_scale_vec2(normalized.as_str()) {
                    transform.scale = Some(val);
                }
            }
            "scalex" => {
                if let Some(val) = parse_scale_value(normalized.as_str()) {
                    transform.scale_x = Some(val);
                }
            }
            "scaley" => {
                if let Some(val) = parse_scale_value(normalized.as_str()) {
                    transform.scale_y = Some(val);
                }
            }
            "rotate" | "rotation" => {
                if let Some(val) = parse_rotation(normalized.as_str()) {
                    transform.rotation = Some(val);
                }
            }
            _ => {}
        }

        remainder = after_open[close_idx + 1..].trim_start();
    }
}

/// Parses a scale value into an `f32`.
fn parse_scale_value(value: &str) -> Option<f32> {
    parse_math_value(value)
        .filter(|val| val.unit == CalcUnit::None)
        .map(|val| val.value)
}

/// Parses scale values into a `Vec2`.
fn parse_scale_vec2(value: &str) -> Option<Vec2> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [x] => parse_scale_value(x).map(|v| Vec2::splat(v)),
        [x, y, ..] => {
            let x_val = parse_scale_value(x)?;
            let y_val = parse_scale_value(y)?;
            Some(Vec2::new(x_val, y_val))
        }
        _ => None,
    }
}

/// Parses a rotation value into radians.
fn parse_rotation(value: &str) -> Option<f32> {
    let parsed = parse_math_value(value.trim())?;
    let radians = match parsed.unit {
        CalcUnit::None | CalcUnit::Deg => parsed.value.to_radians(),
        CalcUnit::Rad => parsed.value,
        CalcUnit::Turn => parsed.value * std::f32::consts::TAU,
        _ => return None,
    };
    Some(nudge_problematic_rotation(radians))
}

/// Nudges rotations that are very close to quadrant boundaries to avoid artifacts.
fn nudge_problematic_rotation(rad: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let half_pi = std::f32::consts::FRAC_PI_2;
    let quarter_pi = std::f32::consts::FRAC_PI_4;

    // Normalize to [0, TAU)
    let a = rad.rem_euclid(tau);

    // Distance to nearest (45° + k*90°)
    let m = (a - quarter_pi).rem_euclid(half_pi);
    let dist = m.min(half_pi - m);

    let eps = 1e-6;
    if dist <= eps { a + 1e-6 } else { rad }
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
pub fn convert_to_i32(value: String) -> Option<i32> {
    let trimmed = value.trim();

    let re = Regex::new(r"^-?\d+$").unwrap();

    if re.is_match(trimmed) {
        trimmed.parse::<i32>().ok()
    } else {
        None
    }
}

/// Parses a numeric string into a floating-point value.
pub fn convert_to_f32(value: String) -> Option<f32> {
    parse_math_value(value.trim())
        .filter(|val| val.unit == CalcUnit::None)
        .map(|val| val.value)
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
pub fn convert_to_font_size(value: String) -> Option<FontVal> {
    let parsed = parse_math_value(value.trim())?;
    match parsed.unit {
        CalcUnit::Px => Some(FontVal::Px(parsed.value)),
        CalcUnit::Rem => Some(FontVal::Rem(parsed.value)),
        CalcUnit::None if parsed.value == 0.0 => Some(FontVal::Px(0.0)),
        _ => None,
    }
}

/// Converts a CSS color string into a Bevy [`Color`].
///
/// # Supported Formats
/// - Named colors (e.g. `"red"`, `"white"`, `"transparent"`)
/// - Hex colors (e.g. `"#ff00ff"`, `"#00000000"` for transparent)
/// - RGB: `"rgb(255, 0, 0)"`
/// - RGBA: `"rgba(255, 0, 0, 0.5)"`, `"rgba(255, 0, 0, 128)"`
///
/// # Parameters
/// - `value`: A [`String`] representing a color in any CSS-compatible format.
///
/// # Returns
/// - `Some(Color)` if parsing succeeds.
/// - `None` if the format is invalid or unsupported.
///
pub fn convert_to_color(value: String) -> Option<Color> {
    let mut color = None;
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("transparent") || trimmed.eq_ignore_ascii_case("none") {
        return color;
    }

    if trimmed.starts_with("#") {
        if trimmed.eq("#00000000") {
            color = Some(Color::NONE);
        } else {
            color = Some(Colored::hex_to_color(trimmed));
        }
    } else if let Some(parts) = parse_color_components(trimmed, "rgb", 3) {
        color = Some(Color::srgb_u8(parts[0], parts[1], parts[2]));
    } else if let Some((r, g, b, a)) = parse_rgba_components(trimmed) {
        color = Some(Color::srgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a,
        ));
    } else {
        color = Colored::named(trimmed);
    }

    color
}

fn parse_color_components(value: &str, name: &str, expected: usize) -> Option<Vec<u8>> {
    let inner = parse_color_function_inner(value, name)?;
    let parts: Vec<_> = inner.split(',').map(str::trim).collect();
    if parts.len() != expected {
        return None;
    }

    let mut values = Vec::with_capacity(expected);
    for part in parts {
        values.push(part.parse::<u8>().ok()?);
    }
    Some(values)
}

fn parse_rgba_components(value: &str) -> Option<(u8, u8, u8, f32)> {
    let inner = parse_color_function_inner(value, "rgba")?;
    let parts: Vec<_> = inner.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return None;
    }

    let r = parts[0].parse::<u8>().ok()?;
    let g = parts[1].parse::<u8>().ok()?;
    let b = parts[2].parse::<u8>().ok()?;
    let raw_alpha = parts[3].parse::<f32>().ok()?;
    let alpha = if raw_alpha > 1.0 {
        (raw_alpha / 255.0).clamp(0.0, 1.0)
    } else {
        raw_alpha.clamp(0.0, 1.0)
    };

    Some((r, g, b, alpha))
}

fn parse_color_function_inner<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}(");
    value.strip_prefix(&prefix)?.strip_suffix(')')
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
pub fn convert_to_background(value: String, all_types: bool) -> Option<Background> {
    let trimmed = value.trim();

    let mut background = Background::default();
    let mut has_any = false;

    if let Some(image_or_gradient) = parse_background_image_layer(trimmed) {
        background.image = image_or_gradient.image;
        background.gradient = image_or_gradient.gradient;
        has_any = true;
    }

    if all_types {
        if let Some(color) = parse_background_color_layer(trimmed) {
            background.color = color;
            has_any = true;
        }
    }

    if has_any { Some(background) } else { None }
}

fn apply_background_color(style: &mut Style, value: &str) {
    let Some(color) = convert_to_color(value.to_string()) else {
        return;
    };

    let mut background = style.background.clone().unwrap_or_default();
    background.color = color;
    style.background = Some(background);
}

fn apply_background_image(style: &mut Style, value: &str) {
    if value.trim().eq_ignore_ascii_case("none") {
        if let Some(mut background) = style.background.clone() {
            background.image = None;
            background.gradient = None;
            style.background = if background.color == Color::NONE {
                None
            } else {
                Some(background)
            };
        }
        return;
    }

    let Some(parsed) = parse_background_image_layer(value.trim()) else {
        return;
    };

    let mut background = style.background.clone().unwrap_or_default();
    background.image = parsed.image;
    background.gradient = parsed.gradient;
    style.background = Some(background);
}

fn parse_background_image_layer(value: &str) -> Option<Background> {
    if let Some(gradient_fn) = extract_css_function(value, "linear-gradient") {
        if let Some(gradient) = parse_linear_gradient(gradient_fn.as_str()) {
            return Some(Background {
                gradient: Some(gradient),
                ..default()
            });
        }
    }

    if let Some(url_fn) = extract_css_function(value, "url") {
        let inner = &url_fn[4..url_fn.len() - 1];
        let image = inner
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !image.is_empty() {
            return Some(Background {
                image: Some(image),
                ..default()
            });
        }
    }

    None
}

fn parse_background_color_layer(value: &str) -> Option<Color> {
    if let Some(color) = convert_to_color(value.to_string()) {
        return Some(color);
    }

    let stripped = strip_known_background_functions(value);
    for token in stripped.split_whitespace() {
        let candidate = token.trim().trim_end_matches(',').trim_matches('/');
        if candidate.is_empty() {
            continue;
        }

        if let Some(color) = convert_to_color(candidate.to_string()) {
            return Some(color);
        }
    }

    None
}

fn strip_known_background_functions(value: &str) -> String {
    let mut output = value.to_string();
    loop {
        let mut removed = false;
        for name in ["linear-gradient", "url"] {
            if let Some(call) = extract_css_function(output.as_str(), name) {
                output = output.replacen(call.as_str(), " ", 1);
                removed = true;
            }
        }

        if !removed {
            break;
        }
    }

    output
}

fn extract_css_function(input: &str, function_name: &str) -> Option<String> {
    let lower = input.to_ascii_lowercase();
    let needle = format!("{function_name}(");
    let start = lower.find(&needle)?;
    let mut depth = 0i32;
    let mut end = None;

    for (idx, ch) in input[start..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(start + idx + 1);
                    break;
                }
            }
            _ => {}
        }
    }

    end.map(|end_idx| input[start..end_idx].to_string())
}

fn parse_background_position(value: &str) -> Option<BackgroundPosition> {
    let segment = value.split(',').next().unwrap_or(value);
    let tokens: Vec<&str> = segment.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut x: Option<BackgroundPositionValue> = None;
    let mut y: Option<BackgroundPositionValue> = None;

    for token in tokens {
        let lower = token.to_ascii_lowercase();
        match lower.as_str() {
            "left" => x = Some(BackgroundPositionValue::Percent(0.0)),
            "right" => x = Some(BackgroundPositionValue::Percent(100.0)),
            "top" => y = Some(BackgroundPositionValue::Percent(0.0)),
            "bottom" => y = Some(BackgroundPositionValue::Percent(100.0)),
            "center" => {
                if x.is_none() {
                    x = Some(BackgroundPositionValue::Percent(50.0));
                } else if y.is_none() {
                    y = Some(BackgroundPositionValue::Percent(50.0));
                }
            }
            _ => {
                if let Some(val) = parse_position_value(token) {
                    if x.is_none() {
                        x = Some(val);
                    } else if y.is_none() {
                        y = Some(val);
                    }
                }
            }
        }
    }

    let x = x.unwrap_or(BackgroundPositionValue::Percent(50.0));
    let y = y.unwrap_or(BackgroundPositionValue::Percent(50.0));

    Some(BackgroundPosition { x, y })
}

fn parse_position_value(value: &str) -> Option<BackgroundPositionValue> {
    let val = convert_to_val(value.to_string())?;
    match val {
        Val::Px(px) => Some(BackgroundPositionValue::Px(px)),
        Val::Percent(percent) => Some(BackgroundPositionValue::Percent(percent)),
        _ => None,
    }
}

fn parse_background_size(value: &str) -> Option<BackgroundSize> {
    let segment = value.split(',').next().unwrap_or(value);
    let tokens: Vec<&str> = segment.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    if tokens.len() == 1 {
        let lower = tokens[0].to_ascii_lowercase();
        match lower.as_str() {
            "cover" => return Some(BackgroundSize::Cover),
            "contain" => return Some(BackgroundSize::Contain),
            "auto" => return Some(BackgroundSize::Auto),
            _ => {}
        }

        let width = parse_size_value(tokens[0])?;
        return Some(BackgroundSize::Explicit(width, BackgroundSizeValue::Auto));
    }

    if tokens.len() >= 2 {
        let width = parse_size_value(tokens[0])?;
        let height = parse_size_value(tokens[1])?;
        return Some(BackgroundSize::Explicit(width, height));
    }

    None
}

fn parse_size_value(value: &str) -> Option<BackgroundSizeValue> {
    let lower = value.trim().to_ascii_lowercase();
    if lower == "auto" {
        return Some(BackgroundSizeValue::Auto);
    }

    let val = convert_to_val(value.to_string())?;
    match val {
        Val::Px(px) => Some(BackgroundSizeValue::Px(px)),
        Val::Percent(percent) => Some(BackgroundSizeValue::Percent(percent)),
        _ => None,
    }
}

fn parse_background_attachment(value: &str) -> Option<BackgroundAttachment> {
    let trimmed = value.trim().to_ascii_lowercase();
    match trimmed.as_str() {
        "scroll" => Some(BackgroundAttachment::Scroll),
        "fixed" => Some(BackgroundAttachment::Fixed),
        "local" => Some(BackgroundAttachment::Local),
        _ => None,
    }
}

fn parse_backdrop_filter(value: &str) -> Option<BackdropFilter> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("blur(") || !trimmed.ends_with(')') {
        return None;
    }

    let start = trimmed.find('(')?;
    let end = trimmed.rfind(')')?;
    if end <= start + 1 {
        return None;
    }

    let inner = &trimmed[start + 1..end];
    let parsed = parse_math_value(inner.trim())?;
    let radius = match parsed.unit {
        CalcUnit::Px => parsed.value,
        CalcUnit::None if parsed.value == 0.0 => 0.0,
        _ => return None,
    };

    Some(BackdropFilter::Blur(radius.max(0.0)))
}

fn parse_linear_gradient(value: &str) -> Option<LinearGradient> {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    let open = lower.find('(')?;
    let name = lower[..open].trim();
    if name != "linear-gradient" || !trimmed.ends_with(')') {
        return None;
    }

    let inner = &trimmed[open + 1..trimmed.len() - 1];
    let mut parts = split_top_level_commas(inner);
    if parts.len() < 2 {
        return None;
    }

    let mut angle = None;
    if let Some(candidate) = parts.first() {
        if let Some(parsed) = parse_gradient_direction(candidate) {
            angle = Some(parsed);
            parts.remove(0);
        }
    }

    let angle = angle.unwrap_or(180.0);
    let mut stops = Vec::new();
    for part in parts {
        let mut parsed = parse_gradient_stop(&part)?;
        stops.append(&mut parsed);
    }

    if stops.is_empty() {
        return None;
    }

    Some(LinearGradient { angle, stops })
}

fn split_top_level_commas(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;

    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            ',' if depth == 0 => {
                let segment = input[start..idx].trim();
                if !segment.is_empty() {
                    parts.push(segment.to_string());
                }
                start = idx + 1;
            }
            _ => {}
        }
    }

    let tail = input[start..].trim();
    if !tail.is_empty() {
        parts.push(tail.to_string());
    }

    parts
}

fn parse_gradient_direction(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();

    if let Some(angle) = parse_angle_degrees(&lower) {
        return Some(normalize_angle(angle));
    }

    let lower = lower.strip_prefix("to ")?;
    let mut dx: f32 = 0.0;
    let mut dy = 0.0;

    for token in lower.split_whitespace() {
        match token {
            "left" => dx = -1.0,
            "right" => dx = 1.0,
            "top" => dy = -1.0,
            "bottom" => dy = 1.0,
            _ => return None,
        }
    }

    if dx == 0.0 && dy == 0.0 {
        return None;
    }

    let angle = dx.atan2(-dy).to_degrees();
    Some(normalize_angle(angle))
}

fn parse_angle_degrees(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    if let Some(num) = trimmed.strip_suffix("deg") {
        return num.trim().parse::<f32>().ok();
    }
    if let Some(num) = trimmed.strip_suffix("rad") {
        let radians = num.trim().parse::<f32>().ok()?;
        return Some(radians.to_degrees());
    }
    if let Some(num) = trimmed.strip_suffix("turn") {
        let turns = num.trim().parse::<f32>().ok()?;
        return Some(turns * 360.0);
    }
    None
}

fn normalize_angle(angle: f32) -> f32 {
    angle.rem_euclid(360.0)
}

fn parse_gradient_stop(value: &str) -> Option<Vec<GradientStop>> {
    let (color_text, rest) = split_color_and_positions(value)?;
    let color = convert_to_color(color_text)?;
    let positions = parse_stop_positions(rest);

    let mut stops = Vec::new();
    match positions.len() {
        0 => stops.push(GradientStop {
            color,
            position: None,
        }),
        1 => stops.push(GradientStop {
            color,
            position: Some(positions[0].clone()),
        }),
        _ => {
            stops.push(GradientStop {
                color,
                position: Some(positions[0].clone()),
            });
            stops.push(GradientStop {
                color,
                position: Some(positions[1].clone()),
            });
        }
    }

    Some(stops)
}

fn split_color_and_positions(value: &str) -> Option<(String, &str)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    let (color, rest) = if lower.starts_with("rgb(") || lower.starts_with("rgba(") {
        let end = trimmed.find(')')?;
        let color = trimmed[..=end].to_string();
        (color, &trimmed[end + 1..])
    } else if trimmed.starts_with('#') {
        let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        (trimmed[..end].to_string(), &trimmed[end..])
    } else {
        let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        (trimmed[..end].to_string(), &trimmed[end..])
    };

    Some((color, rest))
}

fn parse_stop_positions(value: &str) -> Vec<GradientStopPosition> {
    value
        .split_whitespace()
        .filter_map(parse_stop_position)
        .collect()
}

fn parse_stop_position(value: &str) -> Option<GradientStopPosition> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(num) = trimmed.strip_suffix('%') {
        return num
            .trim()
            .parse::<f32>()
            .ok()
            .map(GradientStopPosition::Percent);
    }
    if let Some(num) = trimmed.strip_suffix("px") {
        return num.trim().parse::<f32>().ok().map(GradientStopPosition::Px);
    }
    if trimmed == "0" {
        return Some(GradientStopPosition::Px(0.0));
    }
    None
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
pub fn convert_to_position(value: String) -> Option<PositionType> {
    let trimmed = value.trim();
    match trimmed {
        "relative" => Some(PositionType::Relative),
        "absolute" => Some(PositionType::Absolute),
        _ => Some(PositionType::Relative),
    }
}

/// Converts a CSS `box-sizing` value into a Bevy `BoxSizing`.
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
/// - 2 values → top-left and top-right / bottom-right and bottom-left
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
/// - 2 values → top and bottom / left and right
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
/// Supports values in pixels (e.g., `"10px"`), percentages (e.g., `"50%"`), viewport units (e.g., `"10vw"`),
/// or CSS color formats
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
pub fn convert_to_bevy_box_shadow(value: String) -> Option<BoxShadow> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut vals = vec![];
    let mut color = Colored::TRANSPARENT;

    for part in parts {
        let trimmed = part.trim();
        if let Some(val) = convert_to_val(trimmed.to_string()) {
            vals.push(val);
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
/// Supports common border shorthand variants such as:
/// - `"2px #ccd2de"`
/// - `"2px solid #ccd2de"`
/// - `"2px rgba(192, 198, 210, 0.95)"`
/// - `"2px solid rgba(192, 198, 210, 0.95)"`
///
/// The first parsable length token is used as border width. Color is optional and defaults to transparent.
///
/// # Parameters
/// - `value`: CSS border shorthand string.
///
/// # Returns
/// - `Some((UiRect, Color))` on successful parsing, where `UiRect` sets all borders to the given width.
/// - `None` if the width is missing or cannot be parsed.
///
pub fn convert_css_border(value: String) -> Option<(UiRect, Color)> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let mut rect_val: Option<Val> = None;
    let mut color: Option<Color> = None;

    for (index, part) in parts.iter().enumerate() {
        if rect_val.is_none() {
            rect_val = convert_to_val((*part).to_string());
            if rect_val.is_some() {
                continue;
            }
        }

        if color.is_none() {
            let color_candidate = parts[index..].join(" ");
            if let Some(parsed_color) = convert_to_color(color_candidate) {
                color = Some(parsed_color);
                break;
            }
        }
    }

    let rect_val = rect_val?;
    let rect = UiRect::all(rect_val);
    let color = color.unwrap_or(Colored::TRANSPARENT);

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
/// Converts a string into a `JustifyContent` value.
pub fn convert_to_bevy_justify_content(value: String) -> Option<JustifyContent> {
    let trimmed = value.trim();
    match trimmed {
        "start" => Some(JustifyContent::Start),
        "flex-start" => Some(JustifyContent::FlexStart),
        "end" => Some(JustifyContent::End),
        "flex-end" => Some(JustifyContent::FlexEnd),
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
/// Converts a string into an `AlignItems` value.
pub fn convert_to_bevy_align_items(value: String) -> Option<AlignItems> {
    let trimmed = value.trim();
    match trimmed {
        "start" => Some(AlignItems::Start),
        "flex-start" => Some(AlignItems::FlexStart),
        "end" => Some(AlignItems::End),
        "flex-end" => Some(AlignItems::FlexEnd),
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
/// Converts a string into a `FlexDirection` value.
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
/// Converts a string into a `FlexWrap` value.
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
/// Converts a string into a `LineBreak` value.
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
pub fn convert_to_bevy_pick_able(value: String) -> Option<Pickable> {
    let trimmed = value.trim();
    match trimmed {
        "none" => Some(Pickable::IGNORE),
        _ => Some(Pickable::default()),
    }
}

/// Converts a CSS cursor value into a Bevy [`CursorStyle`].
///
/// Supports `custom("...")` and `url("...")` as custom cursors, plus standard CSS keywords
/// like `default`, `pointer`, `text`, `move`, and resize cursors. If multiple cursors are
/// provided (e.g. `custom(...), pointer`), the first supported entry wins.
pub fn convert_to_cursor_style(value: String) -> Option<CursorStyle> {
    for raw in value.split(',') {
        if let Some(path) = parse_cursor_path(raw) {
            return Some(CursorStyle::Custom(path));
        }

        if let Some(icon) = parse_cursor_keyword(raw) {
            return Some(CursorStyle::System(icon));
        }
    }

    None
}

/// Maps a CSS cursor keyword to a system cursor icon.
fn parse_cursor_keyword(token: &str) -> Option<SystemCursorIcon> {
    let token = token.trim().trim_matches('"').trim_matches('\'').trim();

    if token.is_empty() {
        return None;
    }

    match token.to_ascii_lowercase().as_str() {
        "auto" | "default" | "none" => Some(SystemCursorIcon::Default),
        "context-menu" => Some(SystemCursorIcon::ContextMenu),
        "help" => Some(SystemCursorIcon::Help),
        "pointer" => Some(SystemCursorIcon::Pointer),
        "progress" => Some(SystemCursorIcon::Progress),
        "wait" => Some(SystemCursorIcon::Wait),
        "cell" => Some(SystemCursorIcon::Cell),
        "crosshair" => Some(SystemCursorIcon::Crosshair),
        "text" => Some(SystemCursorIcon::Text),
        "vertical-text" => Some(SystemCursorIcon::VerticalText),
        "alias" => Some(SystemCursorIcon::Alias),
        "copy" => Some(SystemCursorIcon::Copy),
        "move" => Some(SystemCursorIcon::Move),
        "no-drop" => Some(SystemCursorIcon::NoDrop),
        "not-allowed" => Some(SystemCursorIcon::NotAllowed),
        "grab" => Some(SystemCursorIcon::Grab),
        "grabbing" => Some(SystemCursorIcon::Grabbing),
        "e-resize" => Some(SystemCursorIcon::EResize),
        "n-resize" => Some(SystemCursorIcon::NResize),
        "ne-resize" => Some(SystemCursorIcon::NeResize),
        "nw-resize" => Some(SystemCursorIcon::NwResize),
        "s-resize" => Some(SystemCursorIcon::SResize),
        "se-resize" => Some(SystemCursorIcon::SeResize),
        "sw-resize" => Some(SystemCursorIcon::SwResize),
        "w-resize" => Some(SystemCursorIcon::WResize),
        "ew-resize" => Some(SystemCursorIcon::EwResize),
        "ns-resize" => Some(SystemCursorIcon::NsResize),
        "nesw-resize" => Some(SystemCursorIcon::NeswResize),
        "nwse-resize" => Some(SystemCursorIcon::NwseResize),
        "col-resize" => Some(SystemCursorIcon::ColResize),
        "row-resize" => Some(SystemCursorIcon::RowResize),
        "all-scroll" => Some(SystemCursorIcon::AllScroll),
        "zoom-in" => Some(SystemCursorIcon::ZoomIn),
        "zoom-out" => Some(SystemCursorIcon::ZoomOut),
        _ => None,
    }
}

/// Extracts a custom cursor path from a token if present.
fn parse_cursor_path(token: &str) -> Option<String> {
    let trimmed = token.trim();
    let lower = trimmed.to_ascii_lowercase();

    let mut inner = if lower.starts_with("custom(") {
        &trimmed["custom(".len()..]
    } else if lower.starts_with("url(") {
        &trimmed["url(".len()..]
    } else {
        return None;
    };

    if let Some(end) = inner.rfind(')') {
        inner = &inner[..end];
    }

    let mut path = inner
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();
    if path.is_empty() {
        return None;
    }

    if path.starts_with('/') {
        path.remove(0);
    }

    Some(path)
}

/**
 * Converts a string into a `GridAutoFlow` enum value.
 *
 * Recognized values include: "row", "column", "row-dense", and "column-dense".
 *
 * @param value The CSS grid-auto-flow value as a string slice.
 * @return Some(GridAutoFlow) if the value is recognized, None otherwise.
 */
/// Converts a string into a `GridAutoFlow` value.
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
 * - "start/end" (two positive integers separated by a slash)
 * - Or a single positive integer (start).
 *
 * @param value is The CSS grid placement string as a string slice.
 * @return Some(GridPlacement) if the value is valid and parsed, None otherwise.
 */
/// Converts a string into a `GridPlacement` value.
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
/// Converts a string into a list of `GridTrack` values.
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
/// Converts a string into a list of `RepeatedGridTrack` values.
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
 * - fixed sizes with units: "100px", "50%", "10vw"
 *
 * @param input is The CSS grid track string.
 * @return Some(GridTrack) if parsing succeeds, None otherwise.
 */
/// Parses a single grid track definition.
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
        _ => match parse_math_value(input) {
            Some(value) => match value.unit {
                CalcUnit::Px => Some(GridTrack::px(value.value)),
                CalcUnit::Percent => Some(GridTrack::percent(value.value)),
                CalcUnit::Fr => Some(GridTrack::fr(value.value)),
                CalcUnit::Vw => Some(GridTrack::vw(value.value)),
                CalcUnit::Vh => Some(GridTrack::vh(value.value)),
                CalcUnit::VMin => Some(GridTrack::vmin(value.value)),
                CalcUnit::VMax => Some(GridTrack::vmax(value.value)),
                _ => None,
            },
            None => None,
        },
    }
}

/**
 * Parses a CSS min track sizing function from a string.
 *
 * Recognized values:
 * - "auto"
 * - "min-content"
 * - "max-content"
 * - fixed size in px or viewport units, e.g. "100px", "10vw"
 *
 * @param input is The CSS min track sizing string.
 * @return Some(MinTrackSizingFunction) if parsing succeeds, None otherwise.
 */
/// Parses a minimum track sizing function.
fn parse_min_sizing(input: &str) -> Option<MinTrackSizingFunction> {
    match input {
        "auto" => Some(MinTrackSizingFunction::Auto),
        "min-content" => Some(MinTrackSizingFunction::MinContent),
        "max-content" => Some(MinTrackSizingFunction::MaxContent),
        _ => match parse_math_value(input) {
            Some(value) => match value.unit {
                CalcUnit::Px => Some(MinTrackSizingFunction::Px(value.value)),
                CalcUnit::Percent => Some(MinTrackSizingFunction::Percent(value.value)),
                CalcUnit::Vw => Some(MinTrackSizingFunction::Vw(value.value)),
                CalcUnit::Vh => Some(MinTrackSizingFunction::Vh(value.value)),
                CalcUnit::VMin => Some(MinTrackSizingFunction::VMin(value.value)),
                CalcUnit::VMax => Some(MinTrackSizingFunction::VMax(value.value)),
                _ => None,
            },
            None => None,
        },
    }
}

/**
 * Parses a CSS max track sizing function from a string.
 *
 * Recognized values:
 * - "auto"
 * - "min-content"
 * - "max-content"
 * - fixed size in px or viewport units, e.g. "100px", "10vw"
 * - fractional units, e.g. "1 fr"
 *
 *  @param input is The CSS max track sizing string.
 * @return Some(MaxTrackSizingFunction) if parsing succeeds, None otherwise.
 */
/// Parses a maximum track sizing function.
fn parse_max_sizing(input: &str) -> Option<MaxTrackSizingFunction> {
    match input {
        "auto" => Some(MaxTrackSizingFunction::Auto),
        "min-content" => Some(MaxTrackSizingFunction::MinContent),
        "max-content" => Some(MaxTrackSizingFunction::MaxContent),
        _ => match parse_math_value(input) {
            Some(value) => match value.unit {
                CalcUnit::Px => Some(MaxTrackSizingFunction::Px(value.value)),
                CalcUnit::Percent => Some(MaxTrackSizingFunction::Percent(value.value)),
                CalcUnit::Fr => Some(MaxTrackSizingFunction::Fraction(value.value)),
                CalcUnit::Vw => Some(MaxTrackSizingFunction::Vw(value.value)),
                CalcUnit::Vh => Some(MaxTrackSizingFunction::Vh(value.value)),
                CalcUnit::VMin => Some(MaxTrackSizingFunction::VMin(value.value)),
                CalcUnit::VMax => Some(MaxTrackSizingFunction::VMax(value.value)),
                _ => None,
            },
            None => None,
        },
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
 * @param value is The CSS overflow value string.
 * @param which The axis specifier ("x", "y", "all", etc.).
 * @return Some (Overflow) if valid input, None otherwise.
 */
/// Converts a CSS overflow value into a Bevy `Overflow`.
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
 * - viewport units (vw/vh/vmin/vmax), e.g. "10vw"
 * - zero ("0") without a unit
 *
 * @param value is The CSS radius string.
 * @return Some(Vec<Val>) if parsing succeeds, None otherwise.
 */
/// Parses radius values into a list of `Val`.
fn parse_radius_values(value: &str) -> Option<Vec<Val>> {
    let mut vals = Vec::new();
    for part in value.split_whitespace() {
        let val = convert_to_val(part.trim().to_string())?;
        vals.push(val);
    }
    Some(vals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_base_and_media_variants_for_same_selector() {
        let parsed = load_css(
            r#"
            .panel { width: 200px; }
            @media (max-width: 900px) {
                .panel { display: none; }
            }
        "#,
        );

        let mut base_count = 0usize;
        let mut media_count = 0usize;

        for pair in parsed.styles.values() {
            if pair.selector != ".panel" {
                continue;
            }

            if pair.media.is_some() {
                media_count += 1;
            } else {
                base_count += 1;
            }
        }

        assert_eq!(base_count, 1);
        assert_eq!(media_count, 1);
    }

    #[test]
    fn evaluates_max_width_breakpoint() {
        let parsed = load_css(
            r#"
            @media (max-width: 800px) {
                .hide-me { display: none; }
            }
        "#,
        );

        let pair = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".hide-me")
            .expect("missing parsed .hide-me style");

        let media = pair.media.as_ref().expect("missing media condition");
        assert!(media.matches_viewport(Vec2::new(640.0, 360.0)));
        assert!(!media.matches_viewport(Vec2::new(920.0, 360.0)));
    }

    #[test]
    fn background_image_gradient_preserves_existing_color() {
        let mut style = Style::default();
        apply_property_to_style(&mut style, "background-color", "rgba(16, 24, 40, 45)");
        apply_property_to_style(
            &mut style,
            "background-image",
            "linear-gradient(to right, #ffffff, #000000)",
        );

        let background = style.background.expect("missing background");
        assert!(background.gradient.is_some());
        assert_eq!(background.image, None);
        assert_ne!(background.color, Color::NONE);
    }

    #[test]
    fn background_shorthand_parses_gradient_with_extra_tokens() {
        let mut style = Style::default();
        apply_property_to_style(
            &mut style,
            "background",
            "linear-gradient(to bottom left, #ff6336, #4f00b1) no-repeat center",
        );

        let background = style.background.expect("missing background");
        assert!(background.gradient.is_some());
    }

    #[test]
    fn background_color_after_gradient_keeps_gradient() {
        let mut style = Style::default();
        apply_property_to_style(
            &mut style,
            "background",
            "linear-gradient(to right, #ff0000, #00ff00)",
        );
        apply_property_to_style(&mut style, "background-color", "#112233");

        let background = style.background.expect("missing background");
        assert!(background.gradient.is_some());
        assert_ne!(background.color, Color::NONE);
    }

    #[test]
    fn parses_body_flex_layout_properties() {
        let parsed = load_css(
            r#"
            body {
                width: 100vw;
                height: 100vh;
                display: flex;
                flex-direction: column;
                justify-content: flex-start;
                align-items: flex-start;
                flex-wrap: wrap;
                background: linear-gradient(to bottom right, #6a00ff, #ff006a);
            }
        "#,
        );

        let pair = parsed
            .styles
            .values()
            .find(|pair| pair.selector == "body")
            .expect("missing parsed body style");

        assert_eq!(pair.normal.display, Some(Display::Flex));
        assert_eq!(pair.normal.flex_direction, Some(FlexDirection::Column));
        assert_eq!(pair.normal.justify_content, Some(JustifyContent::FlexStart));
        assert_eq!(pair.normal.align_items, Some(AlignItems::FlexStart));
        assert_eq!(pair.normal.flex_wrap, Some(FlexWrap::Wrap));
    }

    #[test]
    fn parses_border_shorthand_rgba_with_spaces() {
        let parsed =
            convert_css_border("2px rgba(192, 198, 210, 0.95)".to_string()).expect("border");

        assert_eq!(parsed.0, UiRect::all(Val::Px(2.0)));
        assert_eq!(
            parsed.1,
            convert_to_color("rgba(192, 198, 210, 0.95)".to_string()).expect("color")
        );
    }

    #[test]
    fn parses_border_shorthand_with_style_token() {
        let parsed =
            convert_css_border("2px solid rgba(192, 198, 210, 0.95)".to_string()).expect("border");

        assert_eq!(parsed.0, UiRect::all(Val::Px(2.0)));
        assert_eq!(
            parsed.1,
            convert_to_color("rgba(192, 198, 210, 0.95)".to_string()).expect("color")
        );
    }
}
