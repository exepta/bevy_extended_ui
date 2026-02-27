#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::io::CssAsset;
    use crate::styles::components::UiStyle;
    use crate::styles::paint::Colored;
    use bevy::asset::AssetPlugin;
    use bevy::prelude::AppTypeRegistry;
    use bevy::prelude::*;
    use std::any::TypeId;
    use std::collections::{HashMap, HashSet};

    fn style_pair(selector: &str, origin: usize) -> StylePair {
        StylePair {
            selector: selector.to_string(),
            origin,
            ..default()
        }
    }

    #[test]
    fn css_source_helpers_create_and_extend_handles() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<CssAsset>();
        let asset_server = app.world().resource::<AssetServer>().clone();

        let mut source = CssSource::from_path(&asset_server, "styles/a.css");
        assert_eq!(source.0.len(), 1);
        assert_eq!(
            source.0[0]
                .path()
                .expect("handle path missing")
                .path()
                .to_string_lossy(),
            "styles/a.css"
        );

        source.push_path(&asset_server, "styles/b.css");
        assert_eq!(source.0.len(), 2);
        assert_eq!(
            source.0[1]
                .path()
                .expect("handle path missing")
                .path()
                .to_string_lossy(),
            "styles/b.css"
        );

        let from_paths = CssSource::from_paths(&asset_server, vec!["c.css", "d.css"]);
        assert_eq!(from_paths.0.len(), 2);
    }

    #[test]
    fn radius_all_sets_all_corners() {
        let radius = Radius::all(Val::Px(8.0));
        assert_eq!(radius.top_left, Val::Px(8.0));
        assert_eq!(radius.top_right, Val::Px(8.0));
        assert_eq!(radius.bottom_left, Val::Px(8.0));
        assert_eq!(radius.bottom_right, Val::Px(8.0));
    }

    #[test]
    fn font_weight_parsers_and_number_roundtrip_work() {
        assert_eq!(
            super::super::FontWeight::from_name("semi-bold"),
            Some(super::super::FontWeight::SemiBold)
        );
        assert_eq!(
            super::super::FontWeight::from_name("HEAVY"),
            Some(super::super::FontWeight::Black)
        );
        assert_eq!(super::super::FontWeight::from_name("unknown"), None);

        assert_eq!(
            super::super::FontWeight::from_number(700),
            Some(super::super::FontWeight::Bold)
        );
        assert_eq!(
            super::super::FontWeight::from_number(999),
            Some(super::super::FontWeight::Normal)
        );
        assert_eq!(super::super::FontWeight::Bold.as_number(), 700);
    }

    #[test]
    fn font_val_get_resolves_px_and_rem() {
        assert_eq!(FontVal::Px(13.0).get(Some(16.0)), 13.0);
        assert_eq!(FontVal::Rem(2.0).get(Some(12.0)), 24.0);
        assert_eq!(FontVal::Rem(2.0).get(None), 2.0);
    }

    #[test]
    fn calc_expr_eval_length_handles_basic_math_and_units() {
        let ctx = CalcContext {
            base: 200.0,
            viewport: Vec2::new(1000.0, 500.0),
        };

        let expr = CalcExpr::Add(
            Box::new(CalcExpr::Value(CalcValue::new(50.0, CalcUnit::Percent))),
            Box::new(CalcExpr::Value(CalcValue::new(20.0, CalcUnit::Px))),
        );
        assert_eq!(expr.eval_length(ctx), Some(120.0));

        let mul = CalcExpr::Mul(
            Box::new(CalcExpr::Value(CalcValue::new(10.0, CalcUnit::Px))),
            Box::new(CalcExpr::Value(CalcValue::new(3.0, CalcUnit::None))),
        );
        assert_eq!(mul.eval_length(ctx), Some(30.0));

        let vmin = CalcExpr::Value(CalcValue::new(10.0, CalcUnit::VMin));
        assert_eq!(vmin.eval_length(ctx), Some(50.0));

        let vmax = CalcExpr::Value(CalcValue::new(10.0, CalcUnit::VMax));
        assert_eq!(vmax.eval_length(ctx), Some(100.0));
    }

    #[test]
    fn calc_expr_eval_unitless_and_division_edge_cases_work() {
        let unitless = CalcExpr::Div(
            Box::new(CalcExpr::Value(CalcValue::new(9.0, CalcUnit::None))),
            Box::new(CalcExpr::Value(CalcValue::new(3.0, CalcUnit::None))),
        );
        assert_eq!(unitless.eval_unitless(), Some(3.0));

        let div_zero = CalcExpr::Div(
            Box::new(CalcExpr::Value(CalcValue::new(9.0, CalcUnit::None))),
            Box::new(CalcExpr::Value(CalcValue::new(0.0, CalcUnit::None))),
        );
        assert_eq!(div_zero.eval_unitless(), None);
        assert_eq!(
            div_zero.eval_length(CalcContext {
                base: 1.0,
                viewport: Vec2::ONE
            }),
            None
        );

        let not_unitless = CalcExpr::Value(CalcValue::new(10.0, CalcUnit::Px));
        assert_eq!(not_unitless.eval_unitless(), None);
    }

    #[test]
    fn calc_expr_sin_supports_angle_units() {
        let expr = CalcExpr::Sin(Box::new(CalcExpr::Value(CalcValue::new(
            90.0,
            CalcUnit::Deg,
        ))));
        let out = expr.eval_unitless().expect("sin should resolve");
        assert!((out - 1.0).abs() < 0.0001);

        let expr_turn = CalcExpr::Sin(Box::new(CalcExpr::Value(CalcValue::new(
            0.5,
            CalcUnit::Turn,
        ))));
        let out_turn = expr_turn.eval_unitless().expect("sin should resolve");
        assert!(out_turn.abs() < 0.0001);
    }

    #[test]
    fn transition_timing_from_name_and_apply_work() {
        assert_eq!(
            TransitionTiming::from_name("ease-in-out"),
            Some(TransitionTiming::EaseInOut)
        );
        assert_eq!(TransitionTiming::from_name("invalid"), None);

        let t = 0.25;
        assert!((TransitionTiming::Linear.apply(t) - 0.25).abs() < 0.0001);
        assert!((TransitionTiming::EaseIn.apply(t) - 0.0625).abs() < 0.0001);
        assert!((TransitionTiming::EaseOut.apply(t) - 0.4375).abs() < 0.0001);
    }

    #[test]
    fn animation_direction_from_name_and_default_work() {
        assert_eq!(
            AnimationDirection::from_name("alternate-reverse"),
            Some(AnimationDirection::AlternateReverse)
        );
        assert_eq!(AnimationDirection::from_name("bad"), None);
        assert_eq!(AnimationDirection::default(), AnimationDirection::Normal);
    }

    #[test]
    fn media_query_condition_matches_and_cache_key_work() {
        let cond = MediaQueryCondition::And(vec![
            MediaQueryCondition::MinWidth(800.0),
            MediaQueryCondition::Not(Box::new(MediaQueryCondition::OrientationPortrait)),
        ]);
        assert!(cond.matches_viewport(Vec2::new(1280.0, 720.0)));
        assert!(!cond.matches_viewport(Vec2::new(700.0, 1200.0)));

        let or = MediaQueryCondition::Or(vec![
            MediaQueryCondition::Width(320.0),
            MediaQueryCondition::Height(200.0),
        ]);
        assert!(or.matches_viewport(Vec2::new(320.0, 10.0)));
        assert!(or.matches_viewport(Vec2::new(10.0, 200.0)));
        assert!(!or.matches_viewport(Vec2::new(640.0, 480.0)));

        let key = cond.cache_key();
        assert!(key.starts_with("and("));
        assert!(key.contains("minw:800.000"));
    }

    #[test]
    fn transform_style_is_empty_reports_state() {
        let mut transform = TransformStyle::default();
        assert!(transform.is_empty());
        transform.rotation = Some(1.0);
        assert!(!transform.is_empty());
    }

    #[test]
    fn style_merge_prefers_calc_over_value_and_can_switch_back_to_value() {
        let mut base = Style::default();
        base.width = Some(Val::Px(10.0));
        base.color = Some(Color::srgb(1.0, 1.0, 1.0));

        let mut other = Style::default();
        other.width_calc = Some(CalcExpr::Value(CalcValue::new(25.0, CalcUnit::Px)));
        other.color = Some(Color::srgb(0.0, 0.0, 0.0));
        other.transform.scale = Some(Vec2::splat(2.0));

        base.merge(&other);
        assert_eq!(base.width, None);
        assert_eq!(
            base.width_calc,
            Some(CalcExpr::Value(CalcValue::new(25.0, CalcUnit::Px)))
        );
        assert_eq!(base.color, Some(Color::srgb(0.0, 0.0, 0.0)));
        assert_eq!(base.transform.scale, Some(Vec2::splat(2.0)));

        let mut to_value = Style::default();
        to_value.width = Some(Val::Percent(40.0));
        base.merge(&to_value);
        assert_eq!(base.width, Some(Val::Percent(40.0)));
        assert_eq!(base.width_calc, None);
    }

    #[test]
    fn style_merge_updates_transform_fields_incrementally() {
        let mut base = Style::default();
        base.transform.scale_x = Some(1.0);
        base.transform.rotation = Some(0.1);

        let mut other = Style::default();
        other.transform.scale_y = Some(3.0);
        other.transform.translation_x = Some(Val::Px(4.0));

        base.merge(&other);
        assert_eq!(base.transform.scale_x, Some(1.0));
        assert_eq!(base.transform.scale_y, Some(3.0));
        assert_eq!(base.transform.translation_x, Some(Val::Px(4.0)));
        assert_eq!(base.transform.rotation, Some(0.1));
    }

    #[test]
    fn extended_styling_plugin_registers_reflected_types() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ExtendedStylingPlugin);

        let registry = app.world().resource::<AppTypeRegistry>().read();
        assert!(registry.get(TypeId::of::<UiStyle>()).is_some());
        assert!(registry.get(TypeId::of::<CssClass>()).is_some());
        assert!(registry.get(TypeId::of::<CssID>()).is_some());
    }

    #[test]
    fn ui_style_from_asset_handle_returns_empty_for_missing_asset() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<CssAsset>();

        // Use the invalid UUID handle so this test is independent from filesystem
        // paths and global CSS cache state.
        let handle = Handle::<CssAsset>::from(AssetId::<CssAsset>::INVALID_UUID);
        let css_assets = app.world().resource::<Assets<CssAsset>>();

        let ui = UiStyle::from_asset_handle(handle, &css_assets);
        assert!(ui.styles.is_empty());
        assert!(ui.keyframes.is_empty());
        assert_eq!(ui.active_style, None);
    }

    #[test]
    fn ui_style_from_asset_handle_parses_css_and_reload_updates_content() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<CssAsset>();

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<CssAsset>>();
            assets.add(CssAsset {
                text: "div { color: red; }".to_string(),
            })
        };

        let mut ui = {
            let assets = app.world().resource::<Assets<CssAsset>>();
            UiStyle::from_asset_handle(handle.clone(), &assets)
        };
        assert!(ui.styles.values().any(|pair| pair.selector == "div"));

        app.world_mut()
            .resource_mut::<Assets<CssAsset>>()
            .insert(
                handle.id(),
                CssAsset {
                    text: ".next { width: 12px; }".to_string(),
                },
            )
            .expect("failed to replace css asset");

        {
            let assets = app.world().resource::<Assets<CssAsset>>();
            ui.reload_from_assets(&assets);
        }

        let selectors: HashSet<String> = ui
            .styles
            .values()
            .map(|pair| pair.selector.clone())
            .collect();
        assert!(selectors.contains(".next"));
    }

    #[test]
    fn ui_style_filtered_clone_keeps_relevant_selectors() {
        let mut styles = HashMap::new();
        styles.insert("s1".to_string(), style_pair("*", 0));
        styles.insert("s2".to_string(), style_pair(".btn", 1));
        styles.insert("s3".to_string(), style_pair(".btn:focus", 1));
        styles.insert("s4".to_string(), style_pair("#main", 2));
        styles.insert("s5".to_string(), style_pair("div", 3));
        styles.insert("s6".to_string(), style_pair("p", 4));

        let ui = UiStyle {
            css: Handle::default(),
            styles,
            keyframes: HashMap::new(),
            active_style: None,
        };

        let id = CssID("main".to_string());
        let classes = CssClass(vec!["btn".to_string()]);
        let tag = TagName("div".to_string());

        let filtered = ui.filtered_clone(Some(&id), Some(&classes), Some(&tag));
        let selectors: HashSet<String> = filtered
            .styles
            .values()
            .map(|pair| pair.selector.clone())
            .collect();

        assert!(selectors.contains("*"));
        assert!(selectors.contains(".btn"));
        assert!(selectors.contains(".btn:focus"));
        assert!(selectors.contains("#main"));
        assert!(selectors.contains("div"));
        assert!(!selectors.contains("p"));
    }

    #[test]
    fn colored_hex_and_named_parsing_work() {
        let short_hex = Colored::hex_to_color("#0f0");
        let short = short_hex.to_srgba();
        assert!((short.red - 0.0).abs() < 0.001);
        assert!((short.green - 1.0).abs() < 0.001);
        assert!((short.blue - 0.0).abs() < 0.001);
        assert!((short.alpha - 1.0).abs() < 0.001);

        let alpha_hex = Colored::hex_to_color("#ff000080");
        let alpha = alpha_hex.to_srgba();
        assert!((alpha.alpha - (128.0 / 255.0)).abs() < 0.001);

        assert_eq!(Colored::named("darkgrey"), Some(Colored::DARK_GRAY));
        assert_eq!(Colored::named("missing-color"), None);
    }

    #[test]
    fn style_defaults_cover_mod_defaults() {
        let background = Background::default();
        assert_eq!(background.color, Color::NONE);
        assert!(background.image.is_none());
        assert!(background.gradient.is_none());

        let position = BackgroundPosition::default();
        assert_eq!(position.x, BackgroundPositionValue::Percent(0.0));
        assert_eq!(position.y, BackgroundPositionValue::Percent(0.0));
        assert_eq!(BackgroundSize::default(), BackgroundSize::Auto);
        assert_eq!(BackgroundAttachment::default(), BackgroundAttachment::Scroll);
        assert_eq!(IconPlace::default(), IconPlace::Right);
        assert_eq!(FontVal::default(), FontVal::Px(12.0));
        assert_eq!(TransitionTiming::default(), TransitionTiming::EaseInOut);
        assert_eq!(TransitionProperty::default(), TransitionProperty::All);

        let animation = AnimationSpec::default();
        assert_eq!(animation.name, "");
        assert_eq!(animation.duration, 0.0);
        assert_eq!(animation.delay, 0.0);
        assert_eq!(animation.timing, TransitionTiming::Ease);
        assert_eq!(animation.iterations, Some(1.0));
        assert_eq!(animation.direction, AnimationDirection::Normal);

        let parsed = ParsedCss::default();
        assert!(parsed.styles.is_empty());
        assert!(parsed.keyframes.is_empty());

        let pair = StylePair::default();
        assert!(pair.media.is_none());
    }

    #[test]
    fn font_and_transition_parsers_cover_all_named_branches() {
        assert_eq!(
            super::super::FontWeight::from_name("thin"),
            Some(super::super::FontWeight::Thin)
        );
        assert_eq!(
            super::super::FontWeight::from_name("extra-light"),
            Some(super::super::FontWeight::ExtraLight)
        );
        assert_eq!(
            super::super::FontWeight::from_name("light"),
            Some(super::super::FontWeight::Light)
        );
        assert_eq!(
            super::super::FontWeight::from_name("normal"),
            Some(super::super::FontWeight::Normal)
        );
        assert_eq!(
            super::super::FontWeight::from_name("medium"),
            Some(super::super::FontWeight::Medium)
        );
        assert_eq!(
            super::super::FontWeight::from_name("semibold"),
            Some(super::super::FontWeight::SemiBold)
        );
        assert_eq!(
            super::super::FontWeight::from_name("bold"),
            Some(super::super::FontWeight::Bold)
        );
        assert_eq!(
            super::super::FontWeight::from_name("extra-bold"),
            Some(super::super::FontWeight::ExtraBold)
        );
        assert_eq!(
            super::super::FontWeight::from_name("black"),
            Some(super::super::FontWeight::Black)
        );

        assert_eq!(
            super::super::FontWeight::from_number(100),
            Some(super::super::FontWeight::Thin)
        );
        assert_eq!(
            super::super::FontWeight::from_number(200),
            Some(super::super::FontWeight::ExtraLight)
        );
        assert_eq!(
            super::super::FontWeight::from_number(300),
            Some(super::super::FontWeight::Light)
        );
        assert_eq!(
            super::super::FontWeight::from_number(400),
            Some(super::super::FontWeight::Normal)
        );
        assert_eq!(
            super::super::FontWeight::from_number(500),
            Some(super::super::FontWeight::Medium)
        );
        assert_eq!(
            super::super::FontWeight::from_number(600),
            Some(super::super::FontWeight::SemiBold)
        );
        assert_eq!(
            super::super::FontWeight::from_number(700),
            Some(super::super::FontWeight::Bold)
        );
        assert_eq!(
            super::super::FontWeight::from_number(800),
            Some(super::super::FontWeight::ExtraBold)
        );
        assert_eq!(
            super::super::FontWeight::from_number(900),
            Some(super::super::FontWeight::Black)
        );
        assert_eq!(
            super::super::FontWeight::from_number(901),
            Some(super::super::FontWeight::Normal)
        );

        assert_eq!(
            TransitionTiming::from_name("linear"),
            Some(TransitionTiming::Linear)
        );
        assert_eq!(
            TransitionTiming::from_name("ease"),
            Some(TransitionTiming::Ease)
        );
        assert_eq!(
            TransitionTiming::from_name("ease-in"),
            Some(TransitionTiming::EaseIn)
        );
        assert_eq!(
            TransitionTiming::from_name("ease-out"),
            Some(TransitionTiming::EaseOut)
        );
        assert_eq!(
            TransitionTiming::from_name("ease-in-out"),
            Some(TransitionTiming::EaseInOut)
        );

        let ease = TransitionTiming::Ease.apply(0.25);
        assert!((ease - 0.15625).abs() < 0.0001);

        let ease_in_out_left = TransitionTiming::EaseInOut.apply(0.25);
        let ease_in_out_right = TransitionTiming::EaseInOut.apply(0.75);
        assert!((ease_in_out_left - 0.125).abs() < 0.0001);
        assert!((ease_in_out_right - 0.875).abs() < 0.0001);

        assert_eq!(
            AnimationDirection::from_name("normal"),
            Some(AnimationDirection::Normal)
        );
        assert_eq!(
            AnimationDirection::from_name("reverse"),
            Some(AnimationDirection::Reverse)
        );
        assert_eq!(
            AnimationDirection::from_name("alternate"),
            Some(AnimationDirection::Alternate)
        );
        assert_eq!(
            AnimationDirection::from_name("alternate-reverse"),
            Some(AnimationDirection::AlternateReverse)
        );
    }

    #[test]
    fn calc_expr_covers_remaining_math_paths() {
        let ctx = CalcContext {
            base: 200.0,
            viewport: Vec2::new(1000.0, 500.0),
        };

        assert_eq!(
            CalcExpr::Value(CalcValue::new(10.0, CalcUnit::Vw)).eval_length(ctx),
            Some(100.0)
        );
        assert_eq!(
            CalcExpr::Value(CalcValue::new(10.0, CalcUnit::Vh)).eval_length(ctx),
            Some(50.0)
        );
        assert_eq!(
            CalcExpr::Value(CalcValue::new(0.0, CalcUnit::None)).eval_length(ctx),
            Some(0.0)
        );
        assert_eq!(
            CalcExpr::Value(CalcValue::new(5.0, CalcUnit::None)).eval_length(ctx),
            None
        );

        let sub = CalcExpr::Sub(
            Box::new(CalcExpr::Value(CalcValue::new(40.0, CalcUnit::Px))),
            Box::new(CalcExpr::Value(CalcValue::new(10.0, CalcUnit::Px))),
        );
        assert_eq!(sub.eval_length(ctx), Some(30.0));

        let mul_reverse = CalcExpr::Mul(
            Box::new(CalcExpr::Value(CalcValue::new(3.0, CalcUnit::None))),
            Box::new(CalcExpr::Value(CalcValue::new(10.0, CalcUnit::Px))),
        );
        assert_eq!(mul_reverse.eval_length(ctx), Some(30.0));

        let mul_invalid = CalcExpr::Mul(
            Box::new(CalcExpr::Value(CalcValue::new(2.0, CalcUnit::Px))),
            Box::new(CalcExpr::Value(CalcValue::new(5.0, CalcUnit::Px))),
        );
        assert_eq!(mul_invalid.eval_length(ctx), None);

        let div = CalcExpr::Div(
            Box::new(CalcExpr::Value(CalcValue::new(80.0, CalcUnit::Px))),
            Box::new(CalcExpr::Value(CalcValue::new(4.0, CalcUnit::None))),
        );
        assert_eq!(div.eval_length(ctx), Some(20.0));

        let min = CalcExpr::Min(vec![
            CalcExpr::Value(CalcValue::new(20.0, CalcUnit::Px)),
            CalcExpr::Value(CalcValue::new(8.0, CalcUnit::Px)),
        ]);
        assert_eq!(min.eval_length(ctx), Some(8.0));

        let max = CalcExpr::Max(vec![
            CalcExpr::Value(CalcValue::new(20.0, CalcUnit::Px)),
            CalcExpr::Value(CalcValue::new(8.0, CalcUnit::Px)),
        ]);
        assert_eq!(max.eval_length(ctx), Some(20.0));

        let add_unitless = CalcExpr::Add(
            Box::new(CalcExpr::Value(CalcValue::new(1.0, CalcUnit::None))),
            Box::new(CalcExpr::Value(CalcValue::new(2.0, CalcUnit::None))),
        );
        assert_eq!(add_unitless.eval_unitless(), Some(3.0));

        let sub_unitless = CalcExpr::Sub(
            Box::new(CalcExpr::Value(CalcValue::new(10.0, CalcUnit::None))),
            Box::new(CalcExpr::Value(CalcValue::new(4.0, CalcUnit::None))),
        );
        assert_eq!(sub_unitless.eval_unitless(), Some(6.0));

        let mul_unitless = CalcExpr::Mul(
            Box::new(CalcExpr::Value(CalcValue::new(3.0, CalcUnit::None))),
            Box::new(CalcExpr::Value(CalcValue::new(4.0, CalcUnit::None))),
        );
        assert_eq!(mul_unitless.eval_unitless(), Some(12.0));

        let div_unitless = CalcExpr::Div(
            Box::new(CalcExpr::Value(CalcValue::new(8.0, CalcUnit::None))),
            Box::new(CalcExpr::Value(CalcValue::new(2.0, CalcUnit::None))),
        );
        assert_eq!(div_unitless.eval_unitless(), Some(4.0));

        let min_unitless = CalcExpr::Min(vec![
            CalcExpr::Value(CalcValue::new(3.0, CalcUnit::None)),
            CalcExpr::Value(CalcValue::new(7.0, CalcUnit::None)),
        ]);
        assert_eq!(min_unitless.eval_unitless(), Some(3.0));

        let max_unitless = CalcExpr::Max(vec![
            CalcExpr::Value(CalcValue::new(3.0, CalcUnit::None)),
            CalcExpr::Value(CalcValue::new(7.0, CalcUnit::None)),
        ]);
        assert_eq!(max_unitless.eval_unitless(), Some(7.0));

        let sin_none = CalcExpr::Sin(Box::new(CalcExpr::Value(CalcValue::new(
            1.0,
            CalcUnit::None,
        ))));
        assert!((sin_none.eval_unitless().expect("sin none") - 1.0_f32.sin()).abs() < 0.0001);

        let sin_rad = CalcExpr::Sin(Box::new(CalcExpr::Value(CalcValue::new(
            std::f32::consts::FRAC_PI_2,
            CalcUnit::Rad,
        ))));
        assert!((sin_rad.eval_unitless().expect("sin rad") - 1.0).abs() < 0.0001);

        let sin_add = CalcExpr::Sin(Box::new(CalcExpr::Add(
            Box::new(CalcExpr::Value(CalcValue::new(30.0, CalcUnit::Deg))),
            Box::new(CalcExpr::Value(CalcValue::new(30.0, CalcUnit::Deg))),
        )));
        assert!((sin_add.eval_unitless().expect("sin add") - 0.8660254).abs() < 0.0002);

        let sin_sub = CalcExpr::Sin(Box::new(CalcExpr::Sub(
            Box::new(CalcExpr::Value(CalcValue::new(90.0, CalcUnit::Deg))),
            Box::new(CalcExpr::Value(CalcValue::new(30.0, CalcUnit::Deg))),
        )));
        assert!((sin_sub.eval_unitless().expect("sin sub") - 0.8660254).abs() < 0.0002);

        let sin_mul_left = CalcExpr::Sin(Box::new(CalcExpr::Mul(
            Box::new(CalcExpr::Value(CalcValue::new(45.0, CalcUnit::Deg))),
            Box::new(CalcExpr::Value(CalcValue::new(2.0, CalcUnit::None))),
        )));
        assert!((sin_mul_left.eval_unitless().expect("sin mul left") - 1.0).abs() < 0.0001);

        let sin_mul_right = CalcExpr::Sin(Box::new(CalcExpr::Mul(
            Box::new(CalcExpr::Value(CalcValue::new(2.0, CalcUnit::None))),
            Box::new(CalcExpr::Value(CalcValue::new(std::f32::consts::FRAC_PI_4, CalcUnit::Rad))),
        )));
        assert!((sin_mul_right.eval_unitless().expect("sin mul right") - 1.0).abs() < 0.0001);

        let sin_div = CalcExpr::Sin(Box::new(CalcExpr::Div(
            Box::new(CalcExpr::Value(CalcValue::new(180.0, CalcUnit::Deg))),
            Box::new(CalcExpr::Value(CalcValue::new(2.0, CalcUnit::None))),
        )));
        assert!((sin_div.eval_unitless().expect("sin div") - 1.0).abs() < 0.0001);

        let sin_min = CalcExpr::Sin(Box::new(CalcExpr::Min(vec![
            CalcExpr::Value(CalcValue::new(90.0, CalcUnit::Deg)),
            CalcExpr::Value(CalcValue::new(180.0, CalcUnit::Deg)),
        ])));
        assert!((sin_min.eval_unitless().expect("sin min") - 1.0).abs() < 0.0001);

        let sin_max = CalcExpr::Sin(Box::new(CalcExpr::Max(vec![
            CalcExpr::Value(CalcValue::new(90.0, CalcUnit::Deg)),
            CalcExpr::Value(CalcValue::new(180.0, CalcUnit::Deg)),
        ])));
        assert!((sin_max.eval_unitless().expect("sin max") - 0.0).abs() < 0.0001);

        let sin_invalid_unit = CalcExpr::Sin(Box::new(CalcExpr::Value(CalcValue::new(
            10.0,
            CalcUnit::Px,
        ))));
        assert_eq!(sin_invalid_unit.eval_unitless(), None);
    }

    #[test]
    fn media_query_condition_covers_remaining_variants_and_keys() {
        assert!(MediaQueryCondition::Always.matches_viewport(Vec2::new(1.0, 1.0)));
        assert!(!MediaQueryCondition::Never.matches_viewport(Vec2::new(1.0, 1.0)));

        assert!(MediaQueryCondition::MinHeight(720.0).matches_viewport(Vec2::new(1000.0, 720.0)));
        assert!(!MediaQueryCondition::MaxHeight(300.0).matches_viewport(Vec2::new(1000.0, 400.0)));
        assert!(MediaQueryCondition::OrientationLandscape.matches_viewport(Vec2::new(1200.0, 800.0)));

        assert_eq!(MediaQueryCondition::Always.cache_key(), "always");
        assert_eq!(MediaQueryCondition::Never.cache_key(), "never");
        assert_eq!(MediaQueryCondition::Width(300.0).cache_key(), "w:300.000");
        assert_eq!(MediaQueryCondition::MinHeight(300.0).cache_key(), "minh:300.000");
        assert_eq!(MediaQueryCondition::MaxHeight(300.0).cache_key(), "maxh:300.000");
        assert_eq!(MediaQueryCondition::Height(300.0).cache_key(), "h:300.000");
        assert_eq!(
            MediaQueryCondition::OrientationLandscape.cache_key(),
            "orientation:landscape"
        );

        let or_key = MediaQueryCondition::Or(vec![
            MediaQueryCondition::MinWidth(500.0),
            MediaQueryCondition::MaxHeight(400.0),
        ])
        .cache_key();
        assert!(or_key.starts_with("or("));
        assert!(or_key.contains("minw:500.000"));
        assert!(or_key.contains("maxh:400.000"));
    }
}
