#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ExtendedUiConfiguration;
    use crate::html::builder;
    use crate::html::converter::{self, HtmlConverterSystem};
    use crate::html::reload::{CssDirty, HtmlReloadPlugin};
    use crate::io::{CssAsset, DefaultCssHandle, HtmlAsset};
    use crate::lang::{UILang, UiLangState, UiLangVariables};
    use crate::styles::{CssClass, CssID, CssSource, IconPlace};
    use crate::widgets::{
        Body, Button, ButtonType, DateFormat, FieldMode, FormValidationMode, InputCap, InputField,
        InputType, Paragraph, RadioButton, Scrollbar, ToggleButton, ToolTipAlignment,
        ToolTipPriority, ToolTipTrigger, ToolTipVariant, UIWidgetState,
    };
    use bevy::asset::{AssetEvent, AssetPlugin};
    use bevy::ecs::message::Messages;
    use bevy::ecs::system::SystemId;
    use bevy::prelude::*;

    fn build_test_html_event(world: &mut World) -> SystemId<In<HtmlEvent>, ()> {
        world.register_system(|In(_event): In<HtmlEvent>| {})
    }
    fn build_test_html_click(world: &mut World) -> SystemId<In<HtmlClick>, ()> {
        world.register_system(|In(_event): In<HtmlClick>| {})
    }
    fn build_test_html_change(world: &mut World) -> SystemId<In<HtmlChange>, ()> {
        world.register_system(|In(_event): In<HtmlChange>| {})
    }
    fn build_test_html_submit(world: &mut World) -> SystemId<In<HtmlSubmit>, ()> {
        world.register_system(|In(_event): In<HtmlSubmit>| {})
    }
    fn build_test_html_init(world: &mut World) -> SystemId<In<HtmlInit>, ()> {
        world.register_system(|In(_event): In<HtmlInit>| {})
    }
    fn build_test_html_out(world: &mut World) -> SystemId<In<HtmlMouseOut>, ()> {
        world.register_system(|In(_event): In<HtmlMouseOut>| {})
    }
    fn build_test_html_over(world: &mut World) -> SystemId<In<HtmlMouseOver>, ()> {
        world.register_system(|In(_event): In<HtmlMouseOver>| {})
    }
    fn build_test_html_focus(world: &mut World) -> SystemId<In<HtmlFocus>, ()> {
        world.register_system(|In(_event): In<HtmlFocus>| {})
    }
    fn build_test_html_scroll(world: &mut World) -> SystemId<In<HtmlScroll>, ()> {
        world.register_system(|In(_event): In<HtmlScroll>| {})
    }
    fn build_test_html_keydown(world: &mut World) -> SystemId<In<HtmlKeyDown>, ()> {
        world.register_system(|In(_event): In<HtmlKeyDown>| {})
    }
    fn build_test_html_keyup(world: &mut World) -> SystemId<In<HtmlKeyUp>, ()> {
        world.register_system(|In(_event): In<HtmlKeyUp>| {})
    }
    fn build_test_html_dragstart(world: &mut World) -> SystemId<In<HtmlDragStart>, ()> {
        world.register_system(|In(_event): In<HtmlDragStart>| {})
    }
    fn build_test_html_drag(world: &mut World) -> SystemId<In<HtmlDrag>, ()> {
        world.register_system(|In(_event): In<HtmlDrag>| {})
    }
    fn build_test_html_dragstop(world: &mut World) -> SystemId<In<HtmlDragStop>, ()> {
        world.register_system(|In(_event): In<HtmlDragStop>| {})
    }

    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlEvent {
            name: "__unit_html_event",
            build: build_test_html_event,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlClick {
            name: "__unit_html_click",
            build: build_test_html_click,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlChange {
            name: "__unit_html_change",
            build: build_test_html_change,
        }
    }
    inventory::submit! {
        HtmlFnRegistration::HtmlSubmit {
            name: "__unit_html_submit",
            build: build_test_html_submit,
        }
    }
    inventory::submit! {
        HtmlFnRegistration::HtmlInit {
            name: "__unit_html_init",
            build: build_test_html_init,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlMouseOut {
            name: "__unit_html_out",
            build: build_test_html_out,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlMouseOver {
            name: "__unit_html_over",
            build: build_test_html_over,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlFocus {
            name: "__unit_html_focus",
            build: build_test_html_focus,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlScroll {
            name: "__unit_html_scroll",
            build: build_test_html_scroll,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlKeyDown {
            name: "__unit_html_keydown",
            build: build_test_html_keydown,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlKeyUp {
            name: "__unit_html_keyup",
            build: build_test_html_keyup,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlDragStart {
            name: "__unit_html_dragstart",
            build: build_test_html_dragstart,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlDrag {
            name: "__unit_html_drag",
            build: build_test_html_drag,
        }
    }
    crate::html::inventory::submit! {
        HtmlFnRegistration::HtmlDragStop {
            name: "__unit_html_dragstop",
            build: build_test_html_dragstop,
        }
    }

    fn setup_converter_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());

        app.init_asset::<HtmlAsset>();
        app.init_asset::<CssAsset>();

        app.insert_resource(ExtendedUiConfiguration::default());
        app.init_resource::<HtmlStructureMap>();
        app.init_resource::<HtmlDirty>();
        app.init_resource::<UILang>();
        app.init_resource::<UiLangState>();
        app.init_resource::<UiLangVariables>();

        let default_css = {
            let mut css_assets = app.world_mut().resource_mut::<Assets<CssAsset>>();
            css_assets.add(CssAsset {
                text: String::from("/* default */"),
            })
        };
        app.insert_resource(DefaultCssHandle(default_css));

        app.add_plugins(HtmlConverterSystem);
        app
    }

    fn add_html_source(
        app: &mut App,
        source_path: &str,
        html_code: &str,
        source_id: &str,
        controller: Option<&str>,
    ) -> Handle<HtmlAsset> {
        let handle = app
            .world()
            .resource::<AssetServer>()
            .load::<HtmlAsset>(source_path.to_string());

        app.world_mut()
            .resource_mut::<Assets<HtmlAsset>>()
            .insert(
                handle.id(),
                HtmlAsset {
                    html: html_code.to_string(),
                    stylesheets: Vec::new(),
                },
            )
            .expect("failed to insert HtmlAsset");

        app.world_mut().spawn(HtmlSource {
            handle: handle.clone(),
            source_id: source_id.to_string(),
            controller: controller.map(str::to_string),
        });

        handle
    }

    fn collect_nodes<'a>(nodes: &'a [HtmlWidgetNode], out: &mut Vec<&'a HtmlWidgetNode>) {
        for node in nodes {
            out.push(node);
            match node {
                HtmlWidgetNode::Body(_, _, _, children, _, _, _)
                | HtmlWidgetNode::Div(_, _, _, children, _, _, _)
                | HtmlWidgetNode::Form(_, _, _, children, _, _, _)
                | HtmlWidgetNode::FieldSet(_, _, _, children, _, _, _) => {
                    collect_nodes(children, out);
                }
                _ => {}
            }
        }
    }

    fn html_fixture() -> &'static str {
        r##"
        <html lang="de-DE">
          <head>
            <meta name="meta-key" controller="meta::controller" />
            <link rel="stylesheet" href="assets/examples/base.css" />
            <link rel="stylesheet" href="/examples/overlay.css" />
            <link rel="preload" href="ignored.css" />
          </head>
          <body id="root" class="app main" style="width: 320px; height 180px" onclick="root_click" hidden>
            <h1 id="title">Headline</h1>
            <div id="container" class="box" readonly>
              <label for="email">E-Mail</label>
              <input id="email" name="email_name" type="email" value="a@b.c" placeholder="mail" maxlength="12" required validation="length(2, 64)&pattern('^.+@.+$')" />
              <input id="auto-cap" type="text" maxlength="auto" />
              <input id="empty-cap" type="text" maxlength="" />
              <date-picker id="birthday" for="#email" placeholder="Datum" value="2026-02-20" min="2025-01-01" max="2027-01-01" format="yyyy-mm-dd"></date-picker>
              <tool-tip for="#email" variant="point" prio="top" alignment="vertical" trigger="hover | click, drag">  Tip text  </tool-tip>
              <p>{{ user.name }}</p>
              <img src="./images/avatar.png" alt="avatar" />
              <divider alignment="horizontal"></divider>
              <checkbox icon="check.png">Ich stimme zu</checkbox>
              <colorpicker value="#112233" alpha="0.5"></colorpicker>
              <progressbar min="10" max="20" value="15"></progressbar>
              <scroll alignment="horizontal"></scroll>
              <slider min="1" max="9" value="4" step="0.5"></slider>
              <switch icon="switch.png">Enable</switch>
              <button type="submit" onmouseenter="over_fn" onmouseleave="out_fn">Senden <icon src="send.png"/></button>
              <button type="button"><icon src="before.png"/> Vorne</button>
              <toggle value="t1" selected>Text zuerst <icon src="after.png"/></toggle>
              <toggle value="t2"><icon src="before.png"/> Text danach</toggle>
              <select>
                <option value="one">One</option>
                <option value="two" icon="two.png" selected>Two</option>
              </select>
              <form action="submit_fn" validate="always" onsubmit="ignored_submit_alias">
                <radio value="x" selected>Radio X</radio>
                <radio value="y">Radio Y</radio>
              </form>
              <fieldset allow-none="false" mode="single">
                <radio value="a">A</radio>
                <radio value="b">B</radio>
              </fieldset>
              <fieldset allow-none="true" mode="multi">
                <radio value="m1" selected>M1</radio>
                <radio value="m2" selected>M2</radio>
                <radio value="m3">M3</radio>
              </fieldset>
              <unknown-tag>ignored</unknown-tag>
            </div>
          </body>
        </html>
        "##
    }

    #[test]
    fn html_style_from_str_parses_colon_and_whitespace_separators() {
        let style = HtmlStyle::from_str("width: 25px; height 10px; invalid-token; ");

        assert_eq!(style.0.width, Some(Val::Px(25.0)));
        assert_eq!(style.0.height, Some(Val::Px(10.0)));
        assert_eq!(style.0.z_index, None);
    }

    #[test]
    fn html_inner_content_getters_and_setters_work() {
        let mut content =
            HtmlInnerContent::new("Text", "<b>Text</b>", vec!["{{ user.name }}".into()]);

        assert_eq!(content.inner_text(), "Text");
        assert_eq!(content.inner_html(), "<b>Text</b>");
        assert_eq!(content.inner_bindings(), &["{{ user.name }}".to_string()]);

        content.set_inner_text("A");
        content.set_inner_html("<i>B</i>");
        content.set_inner_bindings(vec!["{{ user.id }}".into(), "{{ user.role }}".into()]);

        assert_eq!(content.inner_text(), "A");
        assert_eq!(content.inner_html(), "<i>B</i>");
        assert_eq!(
            content.inner_bindings(),
            &["{{ user.id }}".to_string(), "{{ user.role }}".to_string()]
        );
    }

    #[test]
    fn html_id_default_is_monotonic() {
        let first = HtmlID::default();
        let second = HtmlID::default();

        assert!(second.0 > first.0);
    }

    #[test]
    fn resolve_relative_asset_path_normalizes_paths() {
        assert_eq!(
            converter::resolve_relative_asset_path("examples/test.html", "assets/ui/main.css"),
            "examples/ui/main.css"
        );
        assert_eq!(
            converter::resolve_relative_asset_path("examples/test.html", "/ui/global.css"),
            "ui/global.css"
        );
        assert_eq!(
            converter::resolve_relative_asset_path(
                "examples\\nested\\test.html",
                "..\\styles\\a.css"
            ),
            "../styles/a.css"
        );
    }

    #[test]
    fn html_source_from_handle_and_path_resolution_work() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<HtmlAsset>();

        let handle = app
            .world()
            .resource::<AssetServer>()
            .load::<HtmlAsset>("examples/test.html");
        let source = HtmlSource::from_handle(handle);

        assert_eq!(source.source_id, "");
        assert_eq!(source.controller, None);
        assert_eq!(source.get_source_path(), "examples/test.html");
    }

    #[test]
    fn register_html_fns_populates_untyped_and_typed_registry_maps() {
        let mut app = App::new();
        app.init_resource::<HtmlFunctionRegistry>();

        register_html_fns(app.world_mut());

        let registry = app.world().resource::<HtmlFunctionRegistry>();

        assert!(registry.click.contains_key("__unit_html_event"));
        assert!(registry.change.contains_key("__unit_html_event"));
        assert!(registry.submit.contains_key("__unit_html_event"));
        assert!(registry.init.contains_key("__unit_html_event"));
        assert!(registry.out.contains_key("__unit_html_event"));
        assert!(registry.over.contains_key("__unit_html_event"));
        assert!(registry.focus.contains_key("__unit_html_event"));
        assert!(registry.scroll.contains_key("__unit_html_event"));
        assert!(registry.keydown.contains_key("__unit_html_event"));
        assert!(registry.keyup.contains_key("__unit_html_event"));
        assert!(registry.dragstart.contains_key("__unit_html_event"));
        assert!(registry.drag.contains_key("__unit_html_event"));
        assert!(registry.dragstop.contains_key("__unit_html_event"));

        assert!(registry.click_typed.contains_key("__unit_html_click"));
        assert!(registry.change_typed.contains_key("__unit_html_change"));
        assert!(registry.submit_typed.contains_key("__unit_html_submit"));
        assert!(registry.init_typed.contains_key("__unit_html_init"));
        assert!(registry.out_typed.contains_key("__unit_html_out"));
        assert!(registry.over_typed.contains_key("__unit_html_over"));
        assert!(registry.focus_typed.contains_key("__unit_html_focus"));
        assert!(registry.scroll_typed.contains_key("__unit_html_scroll"));
        assert!(registry.keydown_typed.contains_key("__unit_html_keydown"));
        assert!(registry.keyup_typed.contains_key("__unit_html_keyup"));
        assert!(
            registry
                .dragstart_typed
                .contains_key("__unit_html_dragstart")
        );
        assert!(registry.drag_typed.contains_key("__unit_html_drag"));
        assert!(registry.dragstop_typed.contains_key("__unit_html_dragstop"));
    }

    #[test]
    fn converter_parses_complex_html_fixture() {
        let mut app = setup_converter_app();

        add_html_source(
            &mut app,
            "examples/complex_fixture.html",
            html_fixture(),
            "forced-key",
            Some("test::controller"),
        );

        app.update();

        let default_css_id = app.world().resource::<DefaultCssHandle>().0.id();

        let structure_map = app.world().resource::<HtmlStructureMap>();
        let nodes = structure_map
            .html_map
            .get("forced-key")
            .expect("expected parsed html structure for forced-key");

        assert_eq!(nodes.len(), 1);

        let HtmlWidgetNode::Body(
            body,
            body_meta,
            body_states,
            _body_children,
            body_bindings,
            widget,
            _,
        ) = &nodes[0]
        else {
            panic!("expected body as root node");
        };

        assert_eq!(body.html_key.as_deref(), Some("forced-key"));
        assert_eq!(widget.0.as_deref(), Some("test::controller"));
        assert_eq!(body_meta.id.as_deref(), Some("root"));
        assert_eq!(
            body_meta.class.as_deref(),
            Some(&["app".to_string(), "main".to_string()][..])
        );
        assert!(body_states.hidden);
        assert_eq!(body_bindings.onclick.as_deref(), Some("root_click"));
        assert_eq!(body_meta.css.len(), 3);
        assert_eq!(body_meta.css[0].id(), default_css_id);

        let mut all = Vec::new();
        collect_nodes(nodes, &mut all);

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Input(
                InputField {
                    label,
                    name,
                    input_type,
                    cap_text_at,
                    ..
                },
                HtmlMeta {
                    validation: Some(validation),
                    ..
                },
                _,
                _,
                _,
                _
            ) if label == "E-Mail"
                && name == "email_name"
                && *input_type == InputType::Email
                && *cap_text_at == InputCap::CapAt(12)
                && validation.required
                && validation.min_length == Some(2)
                && validation.max_length == Some(64)
                && validation.pattern.as_deref() == Some("^.+@.+$")
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Input(
                InputField {
                    cap_text_at: InputCap::CapAtNodeSize,
                    ..
                },
                _,
                _,
                _,
                _,
                _
            )
        )));
        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Input(
                InputField {
                    cap_text_at: InputCap::NoCap,
                    ..
                },
                _,
                _,
                _,
                _,
                _
            )
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::DatePicker(
                date_picker,
                HtmlMeta { class: Some(classes), .. },
                _,
                _,
                _,
                _
            ) if date_picker.for_id.as_deref() == Some("email")
                && date_picker.format == DateFormat::YearMonthDay
                && classes.iter().any(|c| c == "date-picker-bound")
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::ToolTip(tool_tip, _, _, _, _, _)
                if tool_tip.for_id.as_deref() == Some("email")
                    && tool_tip.variant == ToolTipVariant::Point
                    && tool_tip.prio == ToolTipPriority::Top
                    && tool_tip.alignment == ToolTipAlignment::Vertical
                    && tool_tip.trigger == vec![ToolTipTrigger::Hover, ToolTipTrigger::Click, ToolTipTrigger::Drag]
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Paragraph(
                Paragraph { text, .. },
                HtmlMeta { inner_content, .. },
                _,
                _,
                _,
                _
            ) if text.contains("{{ user.name }}")
                && inner_content
                    .inner_bindings()
                    .iter()
                    .any(|b| b == "{{ user.name }}")
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Img(img, _, _, _, _, _)
                if img.src.as_deref().is_some_and(|src| src.ends_with("images/avatar.png")) && img.alt == "avatar"
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Scrollbar(Scrollbar { vertical, .. }, _, _, _, _, _)
                if !vertical
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Button(
                Button {
                    button_type: ButtonType::Submit,
                    icon_place: IconPlace::Right,
                    ..
                },
                _,
                _,
                _,
                _,
                _
            )
        )));
        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Button(
                Button {
                    button_type: ButtonType::Button,
                    icon_place: IconPlace::Left,
                    ..
                },
                _,
                _,
                _,
                _,
                _
            )
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::ToggleButton(ToggleButton { value, icon_place: IconPlace::Right, selected: true, .. }, _, _, _, _, _)
                if value == "t1"
        )));
        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::ToggleButton(ToggleButton { value, icon_place: IconPlace::Left, selected: false, .. }, _, _, _, _, _)
                if value == "t2"
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::Form(form, _, _, children, _, _, _)
                if form.action.as_deref() == Some("submit_fn")
                    && form.validate_mode == FormValidationMode::Always
                    && children.iter().filter(|n| matches!(n, HtmlWidgetNode::RadioButton(_, _, _, _, _, _))).count() == 2
        )));

        assert!(all.iter().any(|node| matches!(
            node,
            HtmlWidgetNode::FieldSet(fieldset, _, _, children, _, _, _)
                if fieldset.allow_none
                    && fieldset.field_mode == FieldMode::Multi
                    && children.iter().filter(|n| matches!(n, HtmlWidgetNode::RadioButton(_, _, _, _, _, _))).count() == 3
        )));

        let first_single_fieldset_selected = all.iter().find_map(|node| {
            if let HtmlWidgetNode::FieldSet(fieldset, _, _, children, _, _, _) = node {
                if !fieldset.allow_none {
                    let selected: Vec<bool> = children
                        .iter()
                        .filter_map(|n| match n {
                            HtmlWidgetNode::RadioButton(
                                RadioButton { selected, .. },
                                _,
                                _,
                                _,
                                _,
                                _,
                            ) => Some(*selected),
                            _ => None,
                        })
                        .collect();
                    return Some(selected);
                }
            }
            None
        });

        assert_eq!(first_single_fieldset_selected, Some(vec![true, false]));

        let lang = app.world().resource::<UILang>();
        assert_eq!(lang.forced.as_deref(), Some("de-de"));
    }

    #[test]
    fn converter_retries_when_html_asset_becomes_available_later() {
        let mut app = setup_converter_app();

        let handle = app
            .world()
            .resource::<AssetServer>()
            .load::<HtmlAsset>("examples/pending.html");

        app.world_mut().spawn(HtmlSource {
            handle: handle.clone(),
            source_id: "pending-ui".to_string(),
            controller: None,
        });

        app.update();

        {
            let structure_map = app.world().resource::<HtmlStructureMap>();
            assert!(!structure_map.html_map.contains_key("pending-ui"));
        }

        app.world_mut()
            .resource_mut::<Assets<HtmlAsset>>()
            .insert(
                handle.id(),
                HtmlAsset {
                    html:
                        "<html><head><meta name='pending-ui'/></head><body><p>ok</p></body></html>"
                            .to_string(),
                    stylesheets: vec![],
                },
            )
            .expect("failed to insert pending html asset");

        app.update();

        let structure_map = app.world().resource::<HtmlStructureMap>();
        assert!(structure_map.html_map.contains_key("pending-ui"));
    }

    #[test]
    fn builder_spawns_active_html_and_replaces_existing_body() {
        let mut app = setup_converter_app();
        app.add_message::<HtmlAllWidgetsSpawned>();
        app.add_systems(Update, builder::build_html_source);

        add_html_source(
            &mut app,
            "examples/build_fixture.html",
            html_fixture(),
            "build-key",
            Some("builder::controller"),
        );

        app.update();

        let old_body = app
            .world_mut()
            .spawn(Body {
                entry: 999_999,
                html_key: Some("build-key".to_string()),
            })
            .id();

        {
            let mut map = app.world_mut().resource_mut::<HtmlStructureMap>();
            map.active = Some(vec!["build-key".to_string()]);
        }
        app.world_mut().resource_mut::<HtmlDirty>().0 = true;

        app.update();

        let mut body_query = app.world_mut().query::<(Entity, &Body)>();
        let all_bodies: Vec<(Entity, String)> = body_query
            .iter(app.world())
            .map(|(entity, body)| (entity, body.html_key.clone().unwrap_or_default()))
            .collect();

        assert_eq!(all_bodies.len(), 1);
        assert_eq!(all_bodies[0].1, "build-key");

        let spawned_body_entity = all_bodies[0].0;
        let entity_ref = app.world().entity(spawned_body_entity);
        assert!(entity_ref.contains::<HtmlID>());
        assert!(entity_ref.contains::<HtmlEventBindings>());
        assert!(entity_ref.contains::<HtmlInnerContent>());
        assert!(entity_ref.contains::<CssSource>());
        assert!(entity_ref.contains::<CssClass>());
        assert!(entity_ref.contains::<CssID>());
        assert!(entity_ref.contains::<UIWidgetState>());

        assert!(!app.world().entities().contains(old_body));
    }

    #[test]
    fn builder_noops_when_not_dirty_and_clears_dirty_without_active() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_resource::<HtmlStructureMap>();
        app.init_resource::<HtmlDirty>();
        app.add_message::<HtmlAllWidgetsSpawned>();
        app.add_systems(Update, builder::build_html_source);

        // not dirty -> no spawn
        app.world_mut().resource_mut::<HtmlDirty>().0 = false;
        app.update();
        let body_count = {
            let world = app.world_mut();
            let mut query = world.query::<&Body>();
            query.iter(world).count()
        };
        assert_eq!(body_count, 0);

        // dirty but no active list -> dirty is reset and still no spawn
        app.world_mut().resource_mut::<HtmlDirty>().0 = true;
        app.update();
        assert!(!app.world().resource::<HtmlDirty>().0);
        let body_count = {
            let world = app.world_mut();
            let mut query = world.query::<&Body>();
            query.iter(world).count()
        };
        assert_eq!(body_count, 0);
    }

    #[test]
    fn builder_show_widgets_timer_reveals_visible_nodes_only() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_resource::<HtmlStructureMap>();
        app.init_resource::<HtmlDirty>();
        app.add_plugins(builder::HtmlBuilderSystem);

        let mk_meta = || HtmlMeta {
            css: vec![],
            id: None,
            class: None,
            style: None,
            validation: None,
            inner_content: HtmlInnerContent::default(),
        };
        let mk_bindings = || HtmlEventBindings::default();
        let mk_widget = || crate::widgets::Widget(None);

        let tree = HtmlWidgetNode::Body(
            Body {
                entry: 1,
                html_key: Some("show-key".to_string()),
            },
            mk_meta(),
            HtmlStates::default(),
            vec![
                HtmlWidgetNode::Button(
                    Button::default(),
                    mk_meta(),
                    HtmlStates::default(),
                    mk_bindings(),
                    mk_widget(),
                    HtmlID::default(),
                ),
                HtmlWidgetNode::Paragraph(
                    Paragraph::default(),
                    mk_meta(),
                    HtmlStates {
                        hidden: true,
                        disabled: false,
                        readonly: false,
                    },
                    mk_bindings(),
                    mk_widget(),
                    HtmlID::default(),
                ),
            ],
            mk_bindings(),
            mk_widget(),
            HtmlID::default(),
        );

        {
            let mut map = app.world_mut().resource_mut::<HtmlStructureMap>();
            map.active = Some(vec!["show-key".to_string()]);
            map.html_map.insert("show-key".to_string(), vec![tree]);
        }
        app.world_mut().resource_mut::<HtmlDirty>().0 = true;

        // frame 1: build + start timer (all hidden)
        app.update();

        // frame 2: force timer into finished state to trigger reveal branch.
        {
            let mut timer = app.world_mut().resource_mut::<ShowWidgetsTimer>();
            timer.timer = Timer::from_seconds(0.0, TimerMode::Once);
            timer.active = true;
        }
        app.update();

        let mut button_visible = false;
        let mut hidden_paragraph_still_hidden = false;
        let mut query = app.world_mut().query::<(
            &Visibility,
            Option<&NeedHidden>,
            Option<&Button>,
            Option<&Paragraph>,
        )>();
        for (visibility, hidden_marker, button, paragraph) in query.iter(app.world()) {
            if button.is_some() {
                button_visible = *visibility == Visibility::Inherited;
            }
            if paragraph.is_some() && hidden_marker.is_some() {
                hidden_paragraph_still_hidden = *visibility == Visibility::Hidden;
            }
        }

        assert!(button_visible);
        assert!(hidden_paragraph_still_hidden);
    }

    #[test]
    fn reload_marks_entities_with_changed_css_as_dirty() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), HtmlReloadPlugin));
        app.init_asset::<CssAsset>();

        let changed_handle = {
            let mut css_assets = app.world_mut().resource_mut::<Assets<CssAsset>>();
            css_assets.add(CssAsset {
                text: "a { color: red; }".to_string(),
            })
        };
        let unchanged_handle = {
            let mut css_assets = app.world_mut().resource_mut::<Assets<CssAsset>>();
            css_assets.add(CssAsset {
                text: "b { color: blue; }".to_string(),
            })
        };

        let affected = app
            .world_mut()
            .spawn((CssSource(vec![changed_handle.clone()]),))
            .id();
        let unaffected = app
            .world_mut()
            .spawn((CssSource(vec![unchanged_handle.clone()]),))
            .id();

        app.world_mut()
            .resource_mut::<Messages<AssetEvent<CssAsset>>>()
            .write(AssetEvent::Modified {
                id: changed_handle.id(),
            });

        app.update();

        assert!(app.world().entity(affected).contains::<CssDirty>());
        assert!(!app.world().entity(unaffected).contains::<CssDirty>());
    }

    #[test]
    fn reload_marks_entities_with_removed_css_as_dirty() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), HtmlReloadPlugin));
        app.init_asset::<CssAsset>();

        let removed_handle = {
            let mut css_assets = app.world_mut().resource_mut::<Assets<CssAsset>>();
            css_assets.add(CssAsset {
                text: "a { color: red; }".to_string(),
            })
        };

        let affected = app
            .world_mut()
            .spawn((CssSource(vec![removed_handle.clone()]),))
            .id();

        app.world_mut()
            .resource_mut::<Messages<AssetEvent<CssAsset>>>()
            .write(AssetEvent::Removed {
                id: removed_handle.id(),
            });

        app.update();

        assert!(app.world().entity(affected).contains::<CssDirty>());
    }

    #[test]
    fn plugin_initializes_core_html_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ExtendedUiHtmlPlugin);

        assert!(app.world().contains_resource::<HtmlStructureMap>());
        assert!(app.world().contains_resource::<HtmlFunctionRegistry>());
        assert!(app.world().contains_resource::<HtmlDirty>());
        assert!(app.world().contains_resource::<HtmlInitDelay>());
        assert!(app.world().contains_resource::<UILang>());
        assert!(app.world().contains_resource::<UiLangState>());
        assert!(app.world().contains_resource::<UiLangVariables>());
    }

    #[test]
    fn html_event_target_returns_entity() {
        let entity = Entity::from_raw_u32(42).expect("valid raw entity id");
        let event = HtmlEvent { entity };

        assert_eq!(event.target(), entity);
    }
}
