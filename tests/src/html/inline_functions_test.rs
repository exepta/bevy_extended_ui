#[cfg(all(test, feature = "extended-framework"))]
mod tests {
    use super::super::*;
    use crate::framework::UiBindingStore;
    use crate::widgets::{
        CheckBox, ChoiceBox, ChoiceOption, ColorPicker, DatePicker, FieldSelectionMulti,
        FieldSelectionSingle, InputValue, ListBox, ProgressBar, RadioButton, Slider, SwitchButton,
        ToggleButton, WidgetValue,
    };
    use bevy::prelude::*;
    use bevy_extended_ui::html::inline_functions::{HtmlInlineExpr, HtmlInlinePath};
    use bevy_extended_ui::lang::UiSharedValues;
    use bevy_extended_ui::lang::serde_json::{Value as JsonValue, json};

    fn setup_inline_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<HtmlFunctionRegistry>();
        app.init_resource::<UiBindingStore>();
        app.add_plugins(HtmlEventBindingsPlugin);
        app
    }

    fn setup_inline_app_without_store() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<HtmlFunctionRegistry>();
        app.init_resource::<UiSharedValues>();
        app.add_plugins(HtmlEventBindingsPlugin);
        app
    }

    fn inline_change_binding(raw: &str) -> HtmlEventBindings {
        HtmlEventBindings {
            inline: HtmlInlineEventBindings {
                onchange: Some(parse_html_inline_action(raw).expect("inline action")),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn trigger_change(app: &mut App, entity: Entity) {
        app.world_mut().trigger(HtmlChange {
            entity,
            action: HtmlChangeAction::State,
        });
        app.world_mut().flush();
    }

    #[test]
    fn parses_set_event_value() {
        let action = parse_html_inline_action("$set(info.value, $event.value)").unwrap();
        assert_eq!(action.calls().len(), 1);
        assert_eq!(action.calls()[0].function, HtmlInlineFunction::Set);
        assert_eq!(action.calls()[0].target.as_dotted(), "info.value");
    }

    #[test]
    fn parses_multiple_calls_and_literals() {
        let action = parse_html_inline_action("$add(counter, 1); $min(total, '2')").unwrap();
        assert_eq!(action.calls().len(), 2);
        assert_eq!(action.calls()[0].function, HtmlInlineFunction::Add);
        assert_eq!(action.calls()[1].function, HtmlInlineFunction::Min);
    }

    #[test]
    fn inline_bindings_default_and_path_helpers_work() {
        let bindings = HtmlInlineEventBindings::default();
        assert!(bindings.onclick.is_none());
        assert!(bindings.onchange.is_none());
        assert!(bindings.oninit.is_none());

        let path = HtmlInlinePath::new(vec![
            "player".to_string(),
            "profile".to_string(),
            "name".to_string(),
        ])
        .unwrap();
        assert_eq!(path.root(), "player");
        assert_eq!(path.tail(), &["profile".to_string(), "name".to_string()]);
        assert_eq!(path.as_dotted(), "player.profile.name");

        assert!(HtmlInlinePath::new(Vec::new()).is_none());
        assert!(HtmlInlinePath::new(vec!["player".to_string(), String::new()]).is_none());
    }

    #[test]
    fn parse_inline_action_reports_invalid_input() {
        for invalid in [
            "",
            " ; ; ",
            "set(info.value, 1)",
            "$set",
            "$set(info.value, 1",
            "$unknown(info.value, 1)",
            "$set(info.value)",
            "$set(.value, 1)",
            "$set(info.value, $event.)",
            "$set(info.value, 1 + 2)",
        ] {
            assert!(
                parse_html_inline_action(invalid).is_err(),
                "expected `{invalid}` to fail"
            );
        }
    }

    #[test]
    fn parse_inline_action_handles_literals_quotes_and_nested_delimiters() {
        let action = parse_html_inline_action(
            r#"$set(info.text, "hello, \"world\"");
               $set(info.escaped, 'line\nrow\rcol\t\\\'');
               $set(info.flag, true);
               $set(info.false_flag, false);
               $set(info.none, null);
               $set(info.items, [1, 2, {"name": "Ada"}])"#,
        )
        .unwrap();

        assert_eq!(action.calls().len(), 6);
        assert!(matches!(
            &action.calls()[0].value,
            HtmlInlineExpr::Literal(JsonValue::String(value)) if value == "hello, \"world\""
        ));
        assert!(matches!(
            &action.calls()[1].value,
            HtmlInlineExpr::Literal(JsonValue::String(value)) if value == "line\nrow\rcol\t\\'"
        ));
        assert!(matches!(
            action.calls()[2].value,
            HtmlInlineExpr::Literal(JsonValue::Bool(true))
        ));
        assert!(matches!(
            action.calls()[3].value,
            HtmlInlineExpr::Literal(JsonValue::Bool(false))
        ));
        assert!(matches!(
            action.calls()[4].value,
            HtmlInlineExpr::Literal(JsonValue::Null)
        ));
        assert!(matches!(
            action.calls()[5].value,
            HtmlInlineExpr::Literal(JsonValue::Array(_))
        ));
    }

    #[test]
    fn inline_change_action_sets_adds_and_subtracts_store_values() {
        let mut app = setup_inline_app();
        app.world_mut()
            .resource_mut::<UiBindingStore>()
            .set_path_json("info.value", json!(10));
        app.world_mut()
            .resource_mut::<UiBindingStore>()
            .set_path_json("source.copy", json!("copied"));
        let entity = app
            .world_mut()
            .spawn((
                InputValue("Grace".to_string()),
                inline_change_binding(
                    "$set(player.name, $event.value);
                     $set(player.copy, source.copy);
                     $add(info.value, 5);
                     $min(info.value, 3)",
                ),
            ))
            .id();

        trigger_change(&mut app, entity);

        let store = app.world().resource::<UiBindingStore>();
        assert_eq!(store.json_path("player.name"), Some(json!("Grace")));
        assert_eq!(store.json_path("player.copy"), Some(json!("copied")));
        assert_eq!(store.json_path("info.value"), Some(json!(12.0)));
    }

    #[test]
    fn inline_change_action_continues_when_value_or_target_cannot_be_resolved() {
        let mut app = setup_inline_app();
        app.world_mut()
            .resource_mut::<UiBindingStore>()
            .set_path_json("info.value", json!(2));
        let entity = app
            .world_mut()
            .spawn(inline_change_binding(
                "$set(player.name, $event.value); $add(missing.value, 1); $add(info.value, 'bad')",
            ))
            .id();

        trigger_change(&mut app, entity);

        let store = app.world().resource::<UiBindingStore>();
        assert_eq!(store.json_path("player.name"), None);
        assert_eq!(store.json_path("info.value"), Some(json!(2)));
    }

    #[test]
    fn inline_change_reads_common_widget_event_values() {
        let mut app = setup_inline_app();
        let cases: Vec<(Entity, &str, JsonValue)> = vec![
            (
                app.world_mut()
                    .spawn((
                        InputValue("typed".to_string()),
                        inline_change_binding("$set(result.value, $event.value)"),
                    ))
                    .id(),
                "result.value",
                json!("typed"),
            ),
            (
                app.world_mut()
                    .spawn((
                        CheckBox {
                            checked: true,
                            ..Default::default()
                        },
                        inline_change_binding("$set(result.checked, $event.value)"),
                    ))
                    .id(),
                "result.checked",
                json!(true),
            ),
            (
                app.world_mut()
                    .spawn((
                        Slider {
                            value: 42.5,
                            ..Default::default()
                        },
                        inline_change_binding("$set(result.slider, $event.value)"),
                    ))
                    .id(),
                "result.slider",
                json!(42.5),
            ),
            (
                app.world_mut()
                    .spawn((
                        ProgressBar {
                            value: 77.0,
                            ..Default::default()
                        },
                        inline_change_binding("$set(result.progress, $event.value)"),
                    ))
                    .id(),
                "result.progress",
                json!(77.0),
            ),
            (
                app.world_mut()
                    .spawn((
                        ColorPicker::from_rgba_u8(1, 2, 3, 4),
                        inline_change_binding("$set(result.color, $event.value)"),
                    ))
                    .id(),
                "result.color",
                json!("#010203"),
            ),
            (
                app.world_mut()
                    .spawn((
                        ChoiceBox {
                            value: ChoiceOption::new("Choice Text").with_value(9_u32),
                            ..Default::default()
                        },
                        inline_change_binding("$set(result.choice, $event.value)"),
                    ))
                    .id(),
                "result.choice",
                json!(9),
            ),
            (
                app.world_mut()
                    .spawn((
                        ListBox {
                            values: vec![
                                ChoiceOption::new("Alice"),
                                ChoiceOption::new("Answer").with_value(42_i32),
                            ],
                            ..Default::default()
                        },
                        inline_change_binding("$set(result.list, $event.value)"),
                    ))
                    .id(),
                "result.list",
                json!(["Alice", 42]),
            ),
            (
                app.world_mut()
                    .spawn((
                        RadioButton {
                            label: "Radio Label".to_string(),
                            selected: true,
                            value: WidgetValue::new("radio-value".to_string()),
                            ..Default::default()
                        },
                        inline_change_binding("$set(result.radio, $event.value)"),
                    ))
                    .id(),
                "result.radio",
                json!("radio-value"),
            ),
            (
                app.world_mut()
                    .spawn((
                        ToggleButton {
                            label: "Toggle Label".to_string(),
                            selected: true,
                            value: WidgetValue::new(false),
                            ..Default::default()
                        },
                        inline_change_binding("$set(result.toggle, $event.value)"),
                    ))
                    .id(),
                "result.toggle",
                json!(false),
            ),
            (
                app.world_mut()
                    .spawn((
                        SwitchButton {
                            label: "Switch Label".to_string(),
                            selected: true,
                            ..Default::default()
                        },
                        inline_change_binding("$set(result.switch, $event.value)"),
                    ))
                    .id(),
                "result.switch",
                json!(true),
            ),
            (
                app.world_mut()
                    .spawn((
                        DatePicker::default(),
                        InputValue("2026-06-06".to_string()),
                        inline_change_binding("$set(result.date, $event.value)"),
                    ))
                    .id(),
                "result.date",
                json!("2026-06-06"),
            ),
        ];

        for (entity, path, expected) in cases {
            trigger_change(&mut app, entity);
            assert_eq!(
                app.world().resource::<UiBindingStore>().json_path(path),
                Some(expected),
                "path {path}"
            );
        }
    }

    #[test]
    fn inline_change_reads_checked_text_color_and_fieldset_values() {
        let mut app = setup_inline_app();
        let radio = app
            .world_mut()
            .spawn(RadioButton {
                label: "A".to_string(),
                selected: true,
                value: WidgetValue::new("a".to_string()),
                ..Default::default()
            })
            .id();
        let toggle = app
            .world_mut()
            .spawn(ToggleButton {
                label: "B".to_string(),
                selected: false,
                value: WidgetValue::new("b".to_string()),
                ..Default::default()
            })
            .id();
        let single = app
            .world_mut()
            .spawn((
                FieldSelectionSingle(Some(radio)),
                inline_change_binding("$set(result.single, $event.value)"),
            ))
            .id();
        let multi = app
            .world_mut()
            .spawn((
                FieldSelectionMulti(vec![radio, toggle]),
                inline_change_binding("$set(result.multi, $event.value)"),
            ))
            .id();
        let picker = app
            .world_mut()
            .spawn((
                ColorPicker::from_rgba_u8(10, 20, 30, 40),
                inline_change_binding(
                    "$set(color.red, $event.red);
                     $set(color.green, $event.green);
                     $set(color.blue, $event.blue);
                     $set(color.alpha, $event.alpha);
                     $set(color.rgb, $event.rgb);
                     $set(color.rgba, $event.rgba);
                     $set(color.hex, $event.hex)",
                ),
            ))
            .id();
        let checked = app
            .world_mut()
            .spawn((
                ToggleButton {
                    label: "Toggle Label".to_string(),
                    selected: true,
                    value: WidgetValue::new("toggle-value".to_string()),
                    ..Default::default()
                },
                inline_change_binding(
                    "$set(result.checked, $event.checked); $set(result.text, $event.text)",
                ),
            ))
            .id();
        let checkbox_checked = app
            .world_mut()
            .spawn((
                CheckBox {
                    checked: true,
                    ..Default::default()
                },
                inline_change_binding("$set(result.checkbox_checked, $event.checked)"),
            ))
            .id();
        let radio_checked = app
            .world_mut()
            .spawn((
                RadioButton {
                    label: "Radio Checked".to_string(),
                    selected: true,
                    value: WidgetValue::new("checked-radio".to_string()),
                    ..Default::default()
                },
                inline_change_binding("$set(result.radio_checked, $event.selected)"),
            ))
            .id();
        let switch_text_checked = app
            .world_mut()
            .spawn((
                SwitchButton {
                    label: "Switch Text".to_string(),
                    selected: true,
                    ..Default::default()
                },
                inline_change_binding(
                    "$set(result.switch_checked, $event.checked);
                     $set(result.switch_text, $event.text)",
                ),
            ))
            .id();
        let choice_text = app
            .world_mut()
            .spawn((
                ChoiceBox {
                    value: ChoiceOption::new("Choice Text"),
                    ..Default::default()
                },
                inline_change_binding("$set(result.choice_text, $event.text)"),
            ))
            .id();

        for entity in [
            single,
            multi,
            picker,
            checked,
            checkbox_checked,
            radio_checked,
            switch_text_checked,
            choice_text,
        ] {
            trigger_change(&mut app, entity);
        }

        let store = app.world().resource::<UiBindingStore>();
        assert_eq!(store.json_path("result.single"), Some(json!("a")));
        assert_eq!(store.json_path("result.multi"), Some(json!(["a", "b"])));
        assert_eq!(store.json_path("result.checked"), Some(json!(true)));
        assert_eq!(store.json_path("result.text"), Some(json!("Toggle Label")));
        assert_eq!(
            store.json_path("result.checkbox_checked"),
            Some(json!(true))
        );
        assert_eq!(store.json_path("result.radio_checked"), Some(json!(true)));
        assert_eq!(store.json_path("result.switch_checked"), Some(json!(true)));
        assert_eq!(
            store.json_path("result.switch_text"),
            Some(json!("Switch Text"))
        );
        assert_eq!(
            store.json_path("result.choice_text"),
            Some(json!("Choice Text"))
        );
        assert_eq!(store.json_path("color.red"), Some(json!(10)));
        assert_eq!(store.json_path("color.green"), Some(json!(20)));
        assert_eq!(store.json_path("color.blue"), Some(json!(30)));
        assert_eq!(store.json_path("color.alpha"), Some(json!(40)));
        assert_eq!(store.json_path("color.rgb"), Some(json!("rgb(10, 20, 30)")));
        assert_eq!(
            store.json_path("color.rgba"),
            Some(json!("rgba(10, 20, 30, 40)"))
        );
        assert_eq!(store.json_path("color.hex"), Some(json!("#0A141E")));
    }

    #[test]
    fn inline_change_converts_all_widget_value_number_variants() {
        let mut app = setup_inline_app();
        let mut cases = Vec::new();

        macro_rules! spawn_case {
            ($path:literal, $value:expr, $expected:expr) => {{
                let entity = app
                    .world_mut()
                    .spawn((
                        ChoiceBox {
                            value: ChoiceOption::new($path).with_value($value),
                            ..Default::default()
                        },
                        inline_change_binding(concat!("$set(", $path, ", $event.value)")),
                    ))
                    .id();
                cases.push((entity, $path, json!($expected)));
            }};
        }

        spawn_case!("numbers.u8", 1_u8, 1);
        spawn_case!("numbers.u16", 2_u16, 2);
        spawn_case!("numbers.u32", 3_u32, 3);
        spawn_case!("numbers.u64", 4_u64, 4);
        spawn_case!("numbers.u128", 5_u128, 5);
        spawn_case!("numbers.usize", 6_usize, 6);
        spawn_case!("numbers.i8", -1_i8, -1);
        spawn_case!("numbers.i16", -2_i16, -2);
        spawn_case!("numbers.i32", -3_i32, -3);
        spawn_case!("numbers.i64", -4_i64, -4);
        spawn_case!("numbers.i128", -5_i128, -5);
        spawn_case!("numbers.isize", -6_isize, -6);
        spawn_case!("numbers.f32", 1.5_f32, 1.5);
        spawn_case!("numbers.f64", 2.5_f64, 2.5);

        let json_entity = app
            .world_mut()
            .spawn((
                ChoiceBox {
                    value: ChoiceOption::new("json").with_value(json!({"ok": true})),
                    ..Default::default()
                },
                inline_change_binding("$set(numbers.json, $event.value)"),
            ))
            .id();
        cases.push((json_entity, "numbers.json", json!({"ok": true})));

        for (entity, path, expected) in cases {
            trigger_change(&mut app, entity);
            assert_eq!(
                app.world().resource::<UiBindingStore>().json_path(path),
                Some(expected),
                "path {path}"
            );
        }
    }

    #[test]
    fn inline_change_reads_shared_values_when_binding_store_is_absent() {
        let mut app = setup_inline_app_without_store();
        app.world_mut()
            .resource_mut::<UiSharedValues>()
            .values
            .insert(
                "shared".to_string(),
                json!({"value": 4, "bad": "not-number"}),
            );
        let entity = app
            .world_mut()
            .spawn(inline_change_binding(
                "$add(shared.value, 1); $add(shared.bad, 1); $set(result.missing, $event.unknown)",
            ))
            .id();

        trigger_change(&mut app, entity);
    }
}
