#[cfg(test)]
mod tests {
    use super::super::css_service::{CssService, CssUsers};
    use super::super::image_service::{get_or_load_image, pre_load_assets};
    use super::super::state_service::{StateService, update_widget_states};
    use super::super::style_service::{
        LastUiTransform, StyleTransition, propagate_style_inheritance, sync_last_ui_transform,
        update_widget_styles_system,
    };
    use crate::io::CssAsset;
    use crate::styles::components::UiStyle;
    use crate::styles::{CssSource, FontVal, Style, StylePair, TransitionSpec};
    use crate::widgets::{BindToID, UIGenID, UIWidgetState};
    use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
    use bevy::asset::AssetPlugin;
    use bevy::input::ButtonInput;
    use bevy::prelude::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn empty_ui_style() -> UiStyle {
        UiStyle {
            css: Handle::default(),
            styles: HashMap::new(),
            keyframes: HashMap::new(),
            active_style: None,
        }
    }

    fn unique_assets_folder(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        PathBuf::from(format!(
            "assets/{}_{}_{}",
            prefix,
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn update_widget_states_propagates_parent_state_to_bound_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_widget_states);

        let parent = app.world_mut().spawn((
            UIGenID::default(),
            UIWidgetState {
                focused: true,
                hovered: true,
                disabled: true,
                readonly: true,
                checked: true,
                open: false,
                invalid: false,
            },
        ));
        let parent_id = parent.get::<UIGenID>().expect("UIGenID missing").get();

        let child_entity = app
            .world_mut()
            .spawn((BindToID(parent_id), UIWidgetState::default()))
            .id();

        app.update();

        let child = app
            .world()
            .get::<UIWidgetState>(child_entity)
            .expect("child UIWidgetState missing");
        assert!(child.focused);
        assert!(child.hovered);
        assert!(child.disabled);
        assert!(child.readonly);
        assert!(child.checked);
    }

    #[test]
    fn state_service_tabs_focus_across_widgets() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CurrentWidgetState::default());
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_plugins(StateService);

        let e1 = app
            .world_mut()
            .spawn((UIGenID::default(), UIWidgetState::default()))
            .id();
        let e2 = app
            .world_mut()
            .spawn((UIGenID::default(), UIWidgetState::default()))
            .id();

        let (first, second) = {
            let id1 = app.world().get::<UIGenID>(e1).expect("id1 missing").get();
            let id2 = app.world().get::<UIGenID>(e2).expect("id2 missing").get();
            if id1 <= id2 { (e1, e2) } else { (e2, e1) }
        };

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();

        assert!(
            app.world()
                .get::<UIWidgetState>(first)
                .expect("first state missing")
                .focused
        );
        assert!(
            !app.world()
                .get::<UIWidgetState>(second)
                .expect("second state missing")
                .focused
        );

        {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keyboard.reset(KeyCode::Tab);
            keyboard.press(KeyCode::Tab);
        }
        app.update();

        assert!(
            !app.world()
                .get::<UIWidgetState>(first)
                .expect("first state missing")
                .focused
        );
        assert!(
            app.world()
                .get::<UIWidgetState>(second)
                .expect("second state missing")
                .focused
        );
    }

    #[test]
    fn state_service_respects_current_widget_focus_selection() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CurrentWidgetState::default());
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_plugins(StateService);

        let keep = app.world_mut().spawn((
            UIGenID::default(),
            UIWidgetState {
                focused: true,
                ..default()
            },
        ));
        let keep_id = keep.get::<UIGenID>().expect("keep id missing").get();
        let keep_entity = keep.id();

        let clear_entity = app
            .world_mut()
            .spawn((
                UIGenID::default(),
                UIWidgetState {
                    focused: true,
                    ..default()
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<CurrentWidgetState>()
            .widget_id = keep_id;
        app.update();

        assert!(
            app.world()
                .get::<UIWidgetState>(keep_entity)
                .expect("keep state missing")
                .focused
        );
        assert!(
            !app.world()
                .get::<UIWidgetState>(clear_entity)
                .expect("clear state missing")
                .focused
        );
    }

    #[test]
    fn state_service_unfocuses_disabled_widgets() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CurrentWidgetState::default());
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_plugins(StateService);

        let entity = app
            .world_mut()
            .spawn((
                UIGenID::default(),
                UIWidgetState {
                    focused: true,
                    disabled: true,
                    ..default()
                },
            ))
            .id();

        let keep_id = app
            .world()
            .get::<UIGenID>(entity)
            .expect("id missing")
            .get();
        app.world_mut()
            .resource_mut::<CurrentWidgetState>()
            .widget_id = keep_id;
        app.update();

        assert!(
            !app.world()
                .get::<UIWidgetState>(entity)
                .expect("state missing")
                .focused
        );
    }

    #[test]
    fn get_or_load_image_returns_cached_handle_on_second_call() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Image>();
        app.insert_resource(ImageCache::default());

        let path = "service/unit/icon.png";
        let asset_server = app.world().resource::<AssetServer>().clone();

        let first = app
            .world_mut()
            .resource_scope(|world, mut cache: Mut<ImageCache>| {
                let mut images = world.resource_mut::<Assets<Image>>();
                get_or_load_image(path, &mut cache, &mut images, &asset_server)
            });

        let second = app
            .world_mut()
            .resource_scope(|world, mut cache: Mut<ImageCache>| {
                let mut images = world.resource_mut::<Assets<Image>>();
                get_or_load_image(path, &mut cache, &mut images, &asset_server)
            });

        assert_eq!(first.id(), second.id());
        let cache = app.world().resource::<ImageCache>();
        assert_eq!(cache.map.len(), 1);
        assert_eq!(
            cache.map.get(path).expect("cached path missing").id(),
            first.id()
        );
    }

    #[test]
    fn pre_load_assets_does_nothing_for_missing_folder() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Image>();
        app.insert_resource(ImageCache::default());

        let mut config = ExtendedUiConfiguration::default();
        config.assets_path = "assets/this-folder-should-not-exist-services-tests".to_string();
        app.insert_resource(config);
        app.add_systems(Update, pre_load_assets);

        app.update();

        assert!(app.world().resource::<ImageCache>().map.is_empty());
    }

    #[test]
    fn pre_load_assets_loads_supported_extensions_from_assets_folder() {
        let folder = unique_assets_folder("service_preload");
        fs::create_dir_all(&folder).expect("failed to create test assets folder");

        fs::write(folder.join("a.png"), []).expect("failed to write png");
        fs::write(folder.join("b.jpg"), []).expect("failed to write jpg");
        fs::write(folder.join("c.jpeg"), []).expect("failed to write jpeg");
        fs::write(folder.join("ignore.txt"), []).expect("failed to write txt");

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Image>();
        app.insert_resource(ImageCache::default());

        let mut config = ExtendedUiConfiguration::default();
        config.assets_path = folder.to_string_lossy().to_string();
        app.insert_resource(config);
        app.add_systems(Update, pre_load_assets);

        app.update();

        let keys: Vec<String> = app
            .world()
            .resource::<ImageCache>()
            .map
            .keys()
            .cloned()
            .collect();
        assert!(keys.iter().any(|k| k.ends_with("a.png")));
        assert!(keys.iter().any(|k| k.ends_with("b.jpg")));
        assert!(keys.iter().any(|k| k.ends_with("c.jpeg")));
        assert!(!keys.iter().any(|k| k.ends_with("ignore.txt")));

        fs::remove_dir_all(&folder).expect("failed to clean up test assets folder");
    }

    #[test]
    fn css_service_initializes_css_resources() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), CssService));
        app.init_asset::<CssAsset>();

        assert!(
            app.world()
                .contains_resource::<crate::styles::ExistingCssIDs>()
        );
        assert!(app.world().contains_resource::<CssUsers>());
    }

    #[test]
    fn css_service_updates_css_users_index_for_added_and_changed_sources() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), CssService));
        app.init_asset::<CssAsset>();

        let (h1, h2) = {
            let mut assets = app.world_mut().resource_mut::<Assets<CssAsset>>();
            (
                assets.add(CssAsset {
                    text: "button { color: red; }".to_string(),
                }),
                assets.add(CssAsset {
                    text: "button { color: blue; }".to_string(),
                }),
            )
        };

        let entity = app.world_mut().spawn((CssSource(vec![h1.clone()]),)).id();
        app.update();

        {
            let users = app.world().resource::<CssUsers>();
            let set = users
                .users
                .get(&h1.id())
                .expect("missing users for first css");
            assert!(set.contains(&entity));
        }

        app.world_mut()
            .entity_mut(entity)
            .insert(CssSource(vec![h2.clone()]));
        app.update();

        let users = app.world().resource::<CssUsers>();
        let set_new = users
            .users
            .get(&h2.id())
            .expect("missing users for second css");
        assert!(set_new.contains(&entity));
        if let Some(set_old) = users.users.get(&h1.id()) {
            assert!(!set_old.contains(&entity));
        }
    }

    #[test]
    fn sync_last_ui_transform_inserts_and_updates_cached_transform() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, sync_last_ui_transform);

        let entity = app
            .world_mut()
            .spawn((UiTransform::from_translation(Val2::px(10.0, 20.0)),))
            .id();

        app.update();
        let first = app
            .world()
            .get::<LastUiTransform>(entity)
            .expect("missing LastUiTransform after first update");
        assert_eq!(first.0, UiTransform::from_translation(Val2::px(10.0, 20.0)));

        app.world_mut()
            .entity_mut(entity)
            .insert(UiTransform::from_scale(Vec2::splat(2.0)));
        app.update();

        let updated = app
            .world()
            .get::<LastUiTransform>(entity)
            .expect("missing LastUiTransform after update");
        assert_eq!(updated.0, UiTransform::from_scale(Vec2::splat(2.0)));
    }

    #[test]
    fn propagate_style_inheritance_applies_parent_color_and_font_size() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_systems(Update, propagate_style_inheritance);

        let mut root_style = Style::default();
        root_style.color = Some(Color::srgb(1.0, 0.0, 0.0));
        root_style.font_size = Some(FontVal::Px(32.0));

        let mut root_ui = empty_ui_style();
        root_ui.active_style = Some(root_style);

        let root = app.world_mut().spawn((root_ui,)).id();

        let child = app
            .world_mut()
            .spawn((
                empty_ui_style(),
                TextColor(Color::NONE),
                TextFont {
                    font_size: 7.0,
                    ..default()
                },
                ImageNode {
                    color: Color::NONE,
                    ..default()
                },
            ))
            .id();

        app.world_mut().entity_mut(root).add_child(child);
        app.update();

        let child_text = app
            .world()
            .get::<TextColor>(child)
            .expect("missing TextColor");
        let child_font = app
            .world()
            .get::<TextFont>(child)
            .expect("missing TextFont");
        let child_image = app
            .world()
            .get::<ImageNode>(child)
            .expect("missing ImageNode");

        assert_eq!(child_text.0, Color::srgb(1.0, 0.0, 0.0));
        assert_eq!(child_image.color, Color::srgb(1.0, 0.0, 0.0));
        assert_eq!(child_font.font_size, 32.0);
    }

    #[test]
    fn propagate_style_inheritance_respects_local_color_and_size_overrides() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_systems(Update, propagate_style_inheritance);

        let mut parent_style = Style::default();
        parent_style.color = Some(Color::srgb(1.0, 0.0, 0.0));
        parent_style.font_size = Some(FontVal::Px(40.0));
        let mut parent_ui = empty_ui_style();
        parent_ui.active_style = Some(parent_style);

        let root = app.world_mut().spawn((parent_ui,)).id();

        let mut local_style = Style::default();
        local_style.color = Some(Color::srgb(0.0, 0.0, 1.0));
        local_style.font_size = Some(FontVal::Px(13.0));
        let mut child_ui = empty_ui_style();
        child_ui.active_style = Some(local_style);

        let sentinel_color = Color::srgb(0.4, 0.5, 0.6);
        let child = app
            .world_mut()
            .spawn((
                child_ui,
                TextColor(sentinel_color),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                ImageNode {
                    color: sentinel_color,
                    ..default()
                },
            ))
            .id();

        app.world_mut().entity_mut(root).add_child(child);
        app.update();

        let child_text = app
            .world()
            .get::<TextColor>(child)
            .expect("missing TextColor");
        let child_font = app
            .world()
            .get::<TextFont>(child)
            .expect("missing TextFont");
        let child_image = app
            .world()
            .get::<ImageNode>(child)
            .expect("missing ImageNode");

        assert_eq!(child_text.0, sentinel_color);
        assert_eq!(child_image.color, sentinel_color);
        assert_eq!(child_font.font_size, 9.0);
    }

    #[test]
    fn propagate_style_inheritance_uses_transition_current_style_for_descendants() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_systems(Update, propagate_style_inheritance);

        let mut base_parent_style = Style::default();
        base_parent_style.color = Some(Color::srgb(1.0, 0.0, 0.0));
        let mut parent_ui = empty_ui_style();
        parent_ui.active_style = Some(base_parent_style);

        let mut transition_style = Style::default();
        transition_style.color = Some(Color::srgb(0.0, 1.0, 0.0));

        let root = app
            .world_mut()
            .spawn((
                parent_ui,
                StyleTransition {
                    from: Style::default(),
                    to: Style::default(),
                    start_time: 0.0,
                    spec: TransitionSpec::default(),
                    from_transform: None,
                    to_transform: None,
                    current_style: Some(transition_style),
                },
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((empty_ui_style(), TextColor(Color::NONE)))
            .id();

        app.world_mut().entity_mut(root).add_child(child);
        app.update();

        let child_text = app
            .world()
            .get::<TextColor>(child)
            .expect("missing TextColor");
        assert_eq!(child_text.0, Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn update_widget_styles_system_does_not_tint_image_nodes_from_color() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Image>();
        app.insert_resource(ImageCache::default());
        app.add_systems(Update, update_widget_styles_system);

        let mut style = Style::default();
        style.color = Some(Color::srgb(1.0, 0.0, 0.0));

        let mut styles = HashMap::new();
        styles.insert(
            "*".to_string(),
            StylePair {
                normal: style,
                selector: "*".to_string(),
                ..default()
            },
        );

        let entity = app
            .world_mut()
            .spawn((
                UiStyle {
                    css: Handle::default(),
                    styles,
                    keyframes: HashMap::new(),
                    active_style: None,
                },
                Node::default(),
                TextColor(Color::NONE),
                ImageNode {
                    color: Color::srgb(0.2, 0.3, 0.4),
                    ..default()
                },
            ))
            .id();

        app.update();

        let text_color = app
            .world()
            .get::<TextColor>(entity)
            .expect("missing TextColor");
        let image_node = app
            .world()
            .get::<ImageNode>(entity)
            .expect("missing ImageNode");

        assert_eq!(text_color.0, Color::srgb(1.0, 0.0, 0.0));
        assert_eq!(image_node.color, Color::srgb(0.2, 0.3, 0.4));
    }
}
