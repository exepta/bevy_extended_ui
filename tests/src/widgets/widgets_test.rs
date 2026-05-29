#[cfg(test)]
mod tests {
    use super::super::controls::{ButtonImage, place_icon_if};
    use super::super::validation::update_validation_states;
    use super::super::widget_util::{wheel_delta_x, wheel_delta_y};
    use super::super::*;
    use crate::styles::{CssClass, CssSource, IconPlace};
    use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
    use bevy::asset::AssetPlugin;
    use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
    use bevy::prelude::*;

    fn spawn_icons_once(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut image_cache: ResMut<ImageCache>,
        mut images: ResMut<Assets<Image>>,
    ) {
        let parent = commands.spawn_empty().id();
        commands.entity(parent).with_children(|builder| {
            let css_source = CssSource::default();
            place_icon_if(
                builder,
                IconPlace::Left,
                IconPlace::Left,
                &Some("widgets/test-icon.png".to_string()),
                42,
                &asset_server,
                &mut image_cache,
                &mut images,
                vec!["icon-class".to_string()],
                777,
                3,
                css_source.clone(),
            );
            // Different desired side -> should not spawn.
            place_icon_if(
                builder,
                IconPlace::Left,
                IconPlace::Right,
                &Some("widgets/should-not-spawn.png".to_string()),
                42,
                &asset_server,
                &mut image_cache,
                &mut images,
                vec!["skip-class".to_string()],
                888,
                3,
                css_source,
            );
        });
    }

    #[test]
    fn validation_rules_from_attribute_parses_required_length_and_pattern() {
        let rules =
            ValidationRules::from_attribute("required & length(2, 5) & pattern('^[a-z]+$')")
                .expect("rules should parse");

        assert!(rules.required);
        assert_eq!(rules.min_length, Some(2));
        assert_eq!(rules.max_length, Some(5));
        assert_eq!(rules.pattern.as_deref(), Some("^[a-z]+$"));
    }

    #[test]
    fn validation_rules_from_attribute_returns_none_for_empty_or_unknown() {
        assert!(ValidationRules::from_attribute("").is_none());
        assert!(ValidationRules::from_attribute("unknown(1)").is_none());
    }

    #[test]
    fn widget_id_helpers_return_values() {
        let bind = BindToID(123);
        assert_eq!(bind.get(), 123);

        let first = UIGenID::default();
        let second = UIGenID::default();
        assert!(second.get() > first.get());
    }

    #[test]
    fn form_validation_mode_parser_works() {
        assert_eq!(
            FormValidationMode::from_str("all"),
            Some(FormValidationMode::Always)
        );
        assert_eq!(
            FormValidationMode::from_str("send"),
            Some(FormValidationMode::Send)
        );
        assert_eq!(
            FormValidationMode::from_str("interact"),
            Some(FormValidationMode::Interact)
        );
        assert_eq!(FormValidationMode::from_str("invalid"), None);
    }

    #[test]
    fn button_type_parser_works() {
        assert_eq!(ButtonType::from_str("button"), Some(ButtonType::Button));
        assert_eq!(ButtonType::from_str("submit"), Some(ButtonType::Submit));
        assert_eq!(ButtonType::from_str("reset"), Some(ButtonType::Reset));
        assert_eq!(ButtonType::from_str("other"), None);
    }

    #[test]
    fn divider_alignment_parser_and_display_work() {
        assert_eq!(
            DividerAlignment::from_str("horiz"),
            Some(DividerAlignment::Horizontal)
        );
        assert_eq!(
            DividerAlignment::from_str("vertical"),
            Some(DividerAlignment::Vertical)
        );
        assert_eq!(DividerAlignment::from_str("x"), None);
        assert_eq!(DividerAlignment::Horizontal.to_string(), "horizontal");
        assert_eq!(DividerAlignment::Vertical.to_string(), "vertical");
    }

    #[test]
    fn field_mode_parser_works() {
        assert_eq!(FieldMode::from_str("single"), Some(FieldMode::Single));
        assert_eq!(FieldMode::from_str("multi"), Some(FieldMode::Multi));
        assert_eq!(FieldMode::from_str("count"), Some(FieldMode::Count(0)));
        assert_eq!(FieldMode::from_str("count(2)"), Some(FieldMode::Count(2)));
        assert_eq!(
            FieldMode::from_str("count( 12 )"),
            Some(FieldMode::Count(12))
        );
        assert_eq!(FieldMode::from_str("count(x)"), None);
        assert_eq!(FieldMode::from_str("bad"), None);
    }

    #[test]
    fn choice_option_new_trims_internal_value() {
        let option = ChoiceOption::new("  Hello World  ");
        assert_eq!(option.text, "  Hello World  ");
        assert_eq!(option.value_as_str(), Some("Hello World"));
        assert_eq!(option.icon_path, None);
    }

    #[test]
    fn date_format_parser_works() {
        assert_eq!(DateFormat::from_str("mdy"), Some(DateFormat::MonthDayYear));
        assert_eq!(
            DateFormat::from_str("day-month-year"),
            Some(DateFormat::DayMonthYear)
        );
        assert_eq!(DateFormat::from_str("iso"), Some(DateFormat::YearMonthDay));
        assert_eq!(DateFormat::from_str("bad"), None);
    }

    #[test]
    fn input_type_char_rules_and_parser_work() {
        assert!(InputType::Text.is_valid_char('x'));
        assert!(InputType::Email.is_valid_char('@'));
        assert!(!InputType::Email.is_valid_char('!'));
        assert!(InputType::Number.is_valid_char('*'));
        assert!(!InputType::Number.is_valid_char('a'));
        assert!(InputType::Date.is_valid_char('-'));
        assert!(!InputType::Date.is_valid_char('@'));
        assert!(InputType::Range.is_valid_char(' '));
        assert!(!InputType::File.is_valid_char('a'));
        assert_eq!(InputType::from_str("password"), Some(InputType::Password));
        assert_eq!(InputType::from_str("file"), Some(InputType::File));
        assert_eq!(InputType::from_str("bad"), None);
    }

    #[test]
    fn input_cap_get_value_works() {
        assert_eq!(InputCap::NoCap.get_value(), 0);
        assert_eq!(InputCap::CapAtNodeSize.get_value(), 0);
        assert_eq!(InputCap::CapAt(12).get_value(), 12);
    }

    #[test]
    fn tooltip_parsers_work() {
        assert_eq!(
            ToolTipVariant::from_str("point"),
            Some(ToolTipVariant::Point)
        );
        assert_eq!(ToolTipVariant::from_str("x"), None);

        assert_eq!(
            ToolTipPriority::from_str("left"),
            Some(ToolTipPriority::Left)
        );
        assert_eq!(ToolTipPriority::from_str("x"), None);

        assert_eq!(
            ToolTipAlignment::from_str("vertical"),
            Some(ToolTipAlignment::Vertical)
        );
        assert_eq!(ToolTipAlignment::from_str("x"), None);

        assert_eq!(
            ToolTipTrigger::from_str("hover"),
            Some(ToolTipTrigger::Hover)
        );
        assert_eq!(ToolTipTrigger::from_str("x"), None);

        assert_eq!(
            BadgeAnchor::from_str("top right"),
            Some(BadgeAnchor::TopRight)
        );
        assert_eq!(
            BadgeAnchor::from_str("bottom-left"),
            Some(BadgeAnchor::BottomLeft)
        );
        assert_eq!(BadgeAnchor::from_str("x"), None);
    }

    #[test]
    fn hyperlink_browser_parser_works() {
        assert_eq!(
            HyperLinkBrowsers::from_str("system"),
            Some(HyperLinkBrowsers::System)
        );
        assert_eq!(
            HyperLinkBrowsers::from_str("firefox"),
            Some(HyperLinkBrowsers::Custom(vec!["firefox".to_string()]))
        );
        assert_eq!(
            HyperLinkBrowsers::from_str("[firefox, brave, chrome]"),
            Some(HyperLinkBrowsers::Custom(vec![
                "firefox".to_string(),
                "brave".to_string(),
                "chrome".to_string(),
            ]))
        );
        assert_eq!(
            HyperLinkBrowsers::from_str(""),
            Some(HyperLinkBrowsers::System)
        );
    }

    #[test]
    fn color_picker_conversions_and_formatters_work() {
        let mut picker = ColorPicker::from_rgba_u8(255, 0, 0, 128);
        assert_eq!(picker.hex(), "#FF0000");
        assert_eq!(picker.rgb_string(), "rgb(255, 0, 0)");
        assert_eq!(picker.rgba_string(), "rgba(255, 0, 0, 128)");

        picker.set_hsv(120.0, 1.0, 1.0);
        assert_eq!((picker.red, picker.green, picker.blue), (0, 255, 0));

        picker.set_rgb(0, 0, 255);
        assert!((picker.hue - 240.0).abs() < 0.001);
    }

    #[test]
    fn hsv_to_rgb_handles_gray_and_primary_colors() {
        assert_eq!(hsv_to_rgb_u8(0.0, 0.0, 0.5), (128, 128, 128));
        assert_eq!(hsv_to_rgb_u8(0.0, 1.0, 1.0), (255, 0, 0));
        assert_eq!(hsv_to_rgb_u8(120.0, 1.0, 1.0), (0, 255, 0));
        assert_eq!(hsv_to_rgb_u8(240.0, 1.0, 1.0), (0, 0, 255));
    }

    #[test]
    fn wheel_delta_y_converts_line_and_pixel_units() {
        let line_small = MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 2.0,
            window: Entity::PLACEHOLDER,
        };
        assert_eq!(wheel_delta_y(&line_small, 0.5), 50.0);

        let line_big = MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 12.0,
            window: Entity::PLACEHOLDER,
        };
        assert_eq!(wheel_delta_y(&line_big, 0.5), 6.0);

        let pixel = MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: 0.0,
            y: 30.0,
            window: Entity::PLACEHOLDER,
        };
        assert_eq!(wheel_delta_y(&pixel, 0.5), 15.0);
    }

    #[test]
    fn wheel_delta_x_converts_line_and_pixel_units() {
        let line_small = MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 2.0,
            y: 0.0,
            window: Entity::PLACEHOLDER,
        };
        assert_eq!(wheel_delta_x(&line_small, 0.5), 50.0);

        let line_big = MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 12.0,
            y: 0.0,
            window: Entity::PLACEHOLDER,
        };
        assert_eq!(wheel_delta_x(&line_big, 0.5), 6.0);

        let pixel = MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: 30.0,
            y: 0.0,
            window: Entity::PLACEHOLDER,
        };
        assert_eq!(wheel_delta_x(&pixel, 0.5), 15.0);
    }

    #[test]
    fn evaluate_validation_state_checks_required_lengths_and_pattern() {
        let rules = ValidationRules {
            required: true,
            min_length: Some(2),
            max_length: Some(5),
            pattern: Some("^[a-z]+$".to_string()),
        };
        let state = UIWidgetState::default();

        let valid = InputValue("abc".to_string());
        let invalid_short = InputValue("a".to_string());
        let invalid_pattern = InputValue("ab1".to_string());

        assert!(!evaluate_validation_state(
            &rules,
            &state,
            Some(&valid),
            None,
            None,
            None,
            None,
            None
        ));
        assert!(evaluate_validation_state(
            &rules,
            &state,
            Some(&invalid_short),
            None,
            None,
            None,
            None,
            None
        ));
        assert!(evaluate_validation_state(
            &rules,
            &state,
            Some(&invalid_pattern),
            None,
            None,
            None,
            None,
            None
        ));
    }

    #[test]
    fn evaluate_validation_state_uses_checked_for_non_input_controls() {
        let rules = ValidationRules {
            required: true,
            ..default()
        };

        let checked_state = UIWidgetState {
            checked: true,
            ..default()
        };
        let unchecked_state = UIWidgetState::default();
        let checkbox = CheckBox::default();

        assert!(!evaluate_validation_state(
            &rules,
            &checked_state,
            None,
            None,
            Some(&checkbox),
            None,
            None,
            None
        ));
        assert!(evaluate_validation_state(
            &rules,
            &unchecked_state,
            None,
            None,
            Some(&checkbox),
            None,
            None,
            None
        ));
    }

    #[test]
    fn update_validation_states_interact_mode_validates_on_input_changes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_validation_states);

        let form = app
            .world_mut()
            .spawn(Form {
                validate_mode: FormValidationMode::Interact,
                ..default()
            })
            .id();

        let input = app
            .world_mut()
            .spawn((
                ValidationRules {
                    required: true,
                    min_length: Some(2),
                    ..default()
                },
                UIWidgetState::default(),
                InputValue(String::new()),
            ))
            .id();

        app.world_mut().entity_mut(form).add_child(input);
        app.update();
        assert!(
            app.world()
                .get::<UIWidgetState>(input)
                .expect("missing input state")
                .invalid
        );

        app.world_mut()
            .entity_mut(input)
            .insert(InputValue("ok".to_string()));
        app.update();
        assert!(
            !app.world()
                .get::<UIWidgetState>(input)
                .expect("missing input state")
                .invalid
        );
    }

    #[test]
    fn update_validation_states_send_mode_does_not_auto_validate() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_validation_states);

        let form = app
            .world_mut()
            .spawn(Form {
                validate_mode: FormValidationMode::Send,
                ..default()
            })
            .id();

        let input = app
            .world_mut()
            .spawn((
                ValidationRules {
                    required: true,
                    ..default()
                },
                UIWidgetState::default(),
                InputValue(String::new()),
            ))
            .id();

        app.world_mut().entity_mut(form).add_child(input);
        app.update();

        assert!(
            !app.world()
                .get::<UIWidgetState>(input)
                .expect("missing input state")
                .invalid
        );
    }

    #[test]
    fn update_validation_states_outside_form_clears_invalid_flag() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_validation_states);

        let input = app
            .world_mut()
            .spawn((
                ValidationRules {
                    required: true,
                    ..default()
                },
                UIWidgetState {
                    invalid: true,
                    ..default()
                },
                InputValue(String::new()),
            ))
            .id();

        app.update();

        assert!(
            !app.world()
                .get::<UIWidgetState>(input)
                .expect("missing input state")
                .invalid
        );
    }

    #[test]
    fn controls_place_icon_if_spawns_only_for_matching_side() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Image>();
        app.insert_resource(ImageCache::default());
        app.add_systems(Update, spawn_icons_once);

        app.update();

        let mut q = app.world_mut().query::<(
            &Name,
            &ButtonImage,
            &BindToID,
            &CssClass,
            &CssSource,
            &UIWidgetState,
        )>();

        let rows: Vec<_> = q.iter(app.world()).collect();
        assert_eq!(rows.len(), 1);
        let (name, _marker, bind, css_class, css_source, _state) = rows[0];
        assert!(name.as_str().starts_with("Button-Icon-42"));
        assert_eq!(bind.get(), 777);
        assert_eq!(css_class.0, vec!["icon-class".to_string()]);
        assert!(css_source.0.is_empty());

        let cache = app.world().resource::<ImageCache>();
        assert!(cache.map.contains_key("widgets/test-icon.png"));
        assert!(!cache.map.contains_key("widgets/should-not-spawn.png"));
    }

    #[test]
    fn content_and_controls_plugins_can_be_added() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.insert_resource(CurrentWidgetState::default());
        app.insert_resource(ExtendedUiConfiguration::default());
        app.insert_resource(ImageCache::default());
        app.init_asset::<Image>();

        app.add_plugins((ExtendedContentWidgets, ExtendedControlWidgets));
    }

    #[test]
    fn extended_widget_plugin_can_be_added() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.insert_resource(CurrentWidgetState::default());
        app.insert_resource(ExtendedUiConfiguration::default());
        app.insert_resource(ImageCache::default());
        app.init_asset::<Image>();
        app.add_plugins(ExtendedWidgetPlugin);
    }

    // ---------------------------------------------------------------
    // Typed-value helpers shared by ChoiceOption and RadioButton
    // ---------------------------------------------------------------

    /// Enum used only inside these tests — no Reflect needed since we test
    /// the plain `Arc<dyn Any>` path, not the Bevy-reflection path.
    #[derive(Debug, PartialEq)]
    enum Priority {
        Low,
        Medium,
        High,
    }

    /// Arbitrary struct to cover the "object value" case.
    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    // --- ChoiceOption ---

    #[test]
    fn choice_option_with_integer_value() {
        let opt = ChoiceOption::new("42").with_value(42_i32);

        assert_eq!(opt.get_value::<i32>(), Some(&42_i32));
        assert_eq!(opt.get_value::<u32>(), None); // wrong type
        // value_as_str returns None because the Arc holds an i32, not a String
        assert_eq!(opt.value_as_str(), None);
    }

    #[test]
    fn choice_option_with_boolean_value() {
        let opt_true = ChoiceOption::new("yes").with_value(true);
        let opt_false = ChoiceOption::new("no").with_value(false);

        assert_eq!(opt_true.get_value::<bool>(), Some(&true));
        assert_eq!(opt_false.get_value::<bool>(), Some(&false));
    }

    #[test]
    fn choice_option_with_enum_value() {
        let opt = ChoiceOption::new("High").with_value(Priority::High);

        assert_eq!(opt.get_value::<Priority>(), Some(&Priority::High));
        assert_eq!(opt.get_value::<i32>(), None);
    }

    #[test]
    fn choice_option_with_object_value() {
        let opt = ChoiceOption::new("origin").with_value(Point { x: 0, y: 0 });

        let p = opt.get_value::<Point>().expect("should downcast to Point");
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
    }

    #[test]
    fn choice_option_value_as_str_only_for_string_values() {
        let string_opt = ChoiceOption::new("hello");
        assert_eq!(string_opt.value_as_str(), Some("hello"));

        let int_opt = ChoiceOption::new("1").with_value(1_u32);
        assert_eq!(int_opt.value_as_str(), None);
    }

    #[test]
    fn choice_option_no_value_returns_none() {
        let opt = ChoiceOption {
            text: "empty".to_string(),
            value: WidgetValue::default(),
            icon_path: None,
        };

        assert_eq!(opt.get_value::<i32>(), None);
        assert_eq!(opt.value_as_str(), None);
        assert!(opt.get_reflected().is_none());
    }

    #[test]
    fn choice_option_partial_eq_compares_string_values() {
        let a = ChoiceOption::new("foo");
        let b = ChoiceOption::new("foo");
        let c = ChoiceOption::new("bar");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn choice_option_partial_eq_non_string_uses_ptr_identity() {
        let opt_a = ChoiceOption::new("x").with_value(Priority::Low);
        let opt_b = ChoiceOption::new("x").with_value(Priority::Low);
        // Two independent Arcs holding equal values but different allocations.
        assert_ne!(opt_a, opt_b);

        // Cloning shares the Arc, so ptr_eq holds.
        let opt_c = opt_a.clone();
        assert_eq!(opt_a, opt_c);
    }

    // --- RadioButton ---

    #[test]
    fn radio_button_with_integer_value() {
        let rb = RadioButton::default().with_value(7_i32);

        assert_eq!(rb.get_value::<i32>(), Some(&7_i32));
        assert_eq!(rb.get_value::<u8>(), None);
        assert_eq!(rb.value_as_str(), None);
    }

    #[test]
    fn radio_button_with_boolean_value() {
        let rb = RadioButton::default().with_value(true);

        assert_eq!(rb.get_value::<bool>(), Some(&true));
    }

    #[test]
    fn radio_button_with_enum_value() {
        let rb = RadioButton::default().with_value(Priority::Medium);

        assert_eq!(rb.get_value::<Priority>(), Some(&Priority::Medium));
        assert_eq!(rb.get_value::<bool>(), None);
    }

    #[test]
    fn radio_button_with_object_value() {
        let rb = RadioButton::default().with_value(Point { x: 3, y: -1 });

        let p = rb.get_value::<Point>().expect("should downcast to Point");
        assert_eq!(p.x, 3);
        assert_eq!(p.y, -1);
    }

    #[test]
    fn radio_button_value_as_str_only_for_string_values() {
        let string_rb = RadioButton {
            value: WidgetValue::new("option_a".to_string()),
            ..RadioButton::default()
        };
        assert_eq!(string_rb.value_as_str(), Some("option_a"));

        let int_rb = RadioButton::default().with_value(99_i32);
        assert_eq!(int_rb.value_as_str(), None);
    }

    #[test]
    fn radio_button_no_value_returns_none() {
        let rb = RadioButton {
            value: WidgetValue::default(),
            ..RadioButton::default()
        };

        assert_eq!(rb.get_value::<i32>(), None);
        assert_eq!(rb.value_as_str(), None);
        assert!(rb.get_reflected().is_none());
    }
}
