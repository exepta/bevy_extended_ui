#[cfg(test)]
mod tests {
    use super::super::parser::*;
    use crate::styles::Style;
    use crate::styles::TextTransform;
    use bevy::prelude::*;
    use bevy::text::LineHeight;
    use std::collections::HashMap;

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
    fn evaluates_width_range_breakpoint_operators() {
        let parsed = load_css(
            r#"
            @media (width > 800px) {
                .gt-width { display: none; }
            }
            @media (width < 800px) {
                .lt-width { display: none; }
            }
        "#,
        );

        let gt = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".gt-width")
            .expect("missing parsed .gt-width style");
        let lt = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".lt-width")
            .expect("missing parsed .lt-width style");

        let gt_media = gt.media.as_ref().expect("missing gt media condition");
        assert!(gt_media.matches_viewport(Vec2::new(960.0, 600.0)));
        assert!(!gt_media.matches_viewport(Vec2::new(700.0, 600.0)));

        let lt_media = lt.media.as_ref().expect("missing lt media condition");
        assert!(lt_media.matches_viewport(Vec2::new(700.0, 600.0)));
        assert!(!lt_media.matches_viewport(Vec2::new(960.0, 600.0)));
    }

    #[test]
    fn evaluates_prefixed_breakpoint_operators() {
        let parsed = load_css(
            r#"
            @media (min-width > 800px) {
                .min-gt { display: none; }
            }
            @media (max-width < 800px) {
                .max-lt { display: none; }
            }
        "#,
        );

        let min_gt = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".min-gt")
            .expect("missing parsed .min-gt style");
        let max_lt = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".max-lt")
            .expect("missing parsed .max-lt style");

        let min_gt_media = min_gt.media.as_ref().expect("missing min-gt media");
        assert!(min_gt_media.matches_viewport(Vec2::new(900.0, 600.0)));
        assert!(!min_gt_media.matches_viewport(Vec2::new(700.0, 600.0)));

        let max_lt_media = max_lt.media.as_ref().expect("missing max-lt media");
        assert!(max_lt_media.matches_viewport(Vec2::new(700.0, 600.0)));
        assert!(!max_lt_media.matches_viewport(Vec2::new(900.0, 600.0)));
    }

    #[test]
    fn evaluates_prefixed_breakpoint_operators_with_equals() {
        let parsed = load_css(
            r#"
            @media (min-width >= 800px) {
                .min-ge { display: none; }
            }
            @media (max-width <= 800px) {
                .max-le { display: none; }
            }
        "#,
        );

        let min_ge = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".min-ge")
            .expect("missing parsed .min-ge style");
        let max_le = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".max-le")
            .expect("missing parsed .max-le style");

        let min_ge_media = min_ge.media.as_ref().expect("missing min-ge media");
        assert!(min_ge_media.matches_viewport(Vec2::new(800.0, 600.0)));
        assert!(!min_ge_media.matches_viewport(Vec2::new(799.0, 600.0)));

        let max_le_media = max_le.media.as_ref().expect("missing max-le media");
        assert!(max_le_media.matches_viewport(Vec2::new(800.0, 600.0)));
        assert!(!max_le_media.matches_viewport(Vec2::new(801.0, 600.0)));
    }

    #[test]
    fn evaluates_height_range_breakpoint_operators() {
        let parsed = load_css(
            r#"
            @media (height > 500px) {
                .gt-height { display: none; }
            }
            @media (max-height < 500px) {
                .max-lt-height { display: none; }
            }
        "#,
        );

        let gt_height = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".gt-height")
            .expect("missing parsed .gt-height style");
        let max_lt_height = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".max-lt-height")
            .expect("missing parsed .max-lt-height style");

        let gt_height_media = gt_height.media.as_ref().expect("missing gt-height media");
        assert!(gt_height_media.matches_viewport(Vec2::new(900.0, 650.0)));
        assert!(!gt_height_media.matches_viewport(Vec2::new(900.0, 480.0)));

        let max_lt_height_media = max_lt_height
            .media
            .as_ref()
            .expect("missing max-lt-height media");
        assert!(max_lt_height_media.matches_viewport(Vec2::new(900.0, 480.0)));
        assert!(!max_lt_height_media.matches_viewport(Vec2::new(900.0, 650.0)));
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
    fn background_gradient_resolves_nested_css_var_tokens() {
        let parsed = load_css(
            r#"
            :root { --btn-bg: #ffd700; }
            .btn-login {
                background: linear-gradient(180deg, var(--btn-bg) 0%, #f0b90b 100%) center;
            }
        "#,
        );

        let pair = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".btn-login")
            .expect("missing parsed .btn-login style");

        let background = pair.normal.background.as_ref().expect("missing background");
        assert!(background.gradient.is_some());
    }

    #[test]
    fn collects_root_css_vars_from_stylesheet() {
        let vars = collect_root_css_vars(
            r#"
            :root {
                --brand: #102030;
                --accent: #445566;
            }
            .panel { color: var(--brand); }
        "#,
        );

        assert_eq!(vars.get("--brand"), Some(&"#102030".to_string()));
        assert_eq!(vars.get("--accent"), Some(&"#456".to_string()));
    }

    #[test]
    fn resolves_vars_from_external_root_scope() {
        let mut root_vars = HashMap::new();
        root_vars.insert("--brand".to_string(), "#112233".to_string());

        let parsed = load_css_with_root_vars(
            r#"
            :root { --brand: #ffeeaa; }
            .panel { background: var(--brand); }
        "#,
            &root_vars,
        );

        let pair = parsed
            .styles
            .values()
            .find(|pair| pair.selector == ".panel")
            .expect("missing parsed .panel style");

        let background = pair.normal.background.as_ref().expect("missing background");
        assert_eq!(background.color, Color::srgb_u8(0x11, 0x22, 0x33));
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

    #[test]
    fn parses_outline_shorthand_with_style_token() {
        let (width, color) = convert_css_outline("3px solid rgba(192, 198, 210, 0.95)".to_string())
            .expect("outline");

        assert_eq!(width, Val::Px(3.0));
        assert_eq!(
            color,
            convert_to_color("rgba(192, 198, 210, 0.95)".to_string()).expect("color")
        );
    }

    #[test]
    fn outline_and_outline_offset_properties_map_to_style_fields() {
        let mut style = Style::default();

        apply_property_to_style(&mut style, "outline", "2px #7ea2ff");
        apply_property_to_style(&mut style, "outline-offset", "5px");

        assert_eq!(style.outline_width, Some(Val::Px(2.0)));
        assert_eq!(style.outline_color, convert_to_color("#7ea2ff".to_string()));
        assert_eq!(style.outline_offset, Some(Val::Px(5.0)));
    }

    #[test]
    fn outline_none_hides_outline() {
        let mut style = Style::default();

        apply_property_to_style(&mut style, "outline", "none");

        assert_eq!(style.outline_width, Some(Val::Px(0.0)));
        assert_eq!(style.outline_color, Some(Color::NONE));
    }

    #[test]
    fn outline_width_and_color_longhands_are_supported() {
        let mut style = Style::default();

        apply_property_to_style(&mut style, "outline-width", "4px");
        apply_property_to_style(&mut style, "outline-color", "#4f83ff");

        assert_eq!(style.outline_width, Some(Val::Px(4.0)));
        assert_eq!(style.outline_color, convert_to_color("#4f83ff".to_string()));
    }

    #[test]
    fn text_align_property_maps_to_text_layout_justify() {
        let mut style = Style::default();
        apply_property_to_style(&mut style, "text-align", "center");
        assert_eq!(style.text_align, Some(Justify::Center));

        apply_property_to_style(&mut style, "text-align", "right");
        assert_eq!(style.text_align, Some(Justify::Right));

        apply_property_to_style(&mut style, "text-align", "justify");
        assert_eq!(style.text_align, Some(Justify::Justified));
    }

    #[test]
    fn text_shadow_property_maps_to_bevy_text_shadow() {
        let mut style = Style::default();
        apply_property_to_style(
            &mut style,
            "text-shadow",
            "3px 2px 8px rgba(10, 20, 30, 0.5)",
        );

        let shadow = style.text_shadow.expect("missing text shadow");
        assert_eq!(shadow.offset, Vec2::new(3.0, 2.0));
        assert_eq!(
            shadow.color,
            convert_to_color("rgba(10, 20, 30, 0.5)".to_string()).expect("color")
        );
    }

    #[test]
    fn text_transform_property_supports_upper_lower_capitalize_and_none() {
        let mut style = Style::default();

        apply_property_to_style(&mut style, "text-transform", "uppercase");
        assert_eq!(style.text_transform, Some(TextTransform::Uppercase));

        apply_property_to_style(&mut style, "text-transform", "lowercase");
        assert_eq!(style.text_transform, Some(TextTransform::Lowercase));

        apply_property_to_style(&mut style, "text-transform", "capitalize");
        assert_eq!(style.text_transform, Some(TextTransform::Capitalize));

        apply_property_to_style(&mut style, "text-transform", "none");
        assert_eq!(style.text_transform, Some(TextTransform::None));
    }

    #[test]
    fn text_transfrom_alias_is_supported() {
        let mut style = Style::default();
        apply_property_to_style(&mut style, "text-transfrom", "lowercase");
        assert_eq!(style.text_transform, Some(TextTransform::Lowercase));
    }

    #[test]
    fn line_height_property_maps_to_bevy_line_height() {
        let mut style = Style::default();
        apply_property_to_style(&mut style, "line-height", "150%");
        assert_eq!(style.line_height, Some(LineHeight::RelativeToFont(1.5)));

        apply_property_to_style(&mut style, "line-height", "20px");
        assert_eq!(style.line_height, Some(LineHeight::Px(20.0)));
    }

    #[test]
    fn box_shadow_parses_rgba_with_spaces() {
        let shadow = convert_to_bevy_box_shadow("2px 4px 6px rgba(192, 198, 210, 0.95)".into())
            .expect("shadow");
        let layer = shadow.0.first().expect("shadow layer");

        assert_eq!(layer.x_offset, Val::Px(2.0));
        assert_eq!(layer.y_offset, Val::Px(4.0));
        assert_eq!(layer.blur_radius, Val::Px(6.0));
        assert_eq!(
            layer.color,
            convert_to_color("rgba(192, 198, 210, 0.95)".to_string()).expect("color")
        );
    }
}
