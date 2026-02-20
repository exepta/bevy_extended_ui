#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::html::HtmlSource;
    use crate::lang::vars_fingerprint;
    use crate::registry::{ExtendedRegistryPlugin, IdPool, UiInitResource, UiRegistry};
    use crate::utils::keycode_to_char;
    use bevy::camera::visibility::RenderLayers;
    use bevy::render::view::Hdr;

    fn source(controller: Option<&str>) -> HtmlSource {
        HtmlSource {
            handle: Handle::default(),
            source_id: String::new(),
            controller: controller.map(str::to_string),
        }
    }

    #[test]
    fn extended_ui_configuration_default_values_are_expected() {
        let cfg = ExtendedUiConfiguration::default();
        assert_eq!(cfg.order, 2);
        assert!(!cfg.hdr_support);
        assert!(matches!(cfg.camera, ExtendedCam::Default));
        assert_eq!(cfg.render_layers, vec![1, 2]);
        assert_eq!(cfg.assets_path, "assets/extended_ui/");
        assert_eq!(cfg.language_path, "assets/lang");
    }

    #[test]
    fn current_widget_state_default_is_zero() {
        assert_eq!(CurrentWidgetState::default().widget_id, 0);
    }

    #[test]
    fn ui_lang_resolution_prefers_forced_then_selected_then_system() {
        let lang = UILang {
            forced: Some("fr".to_string()),
            selected: Some("de".to_string()),
            system: Some("en".to_string()),
        };
        assert_eq!(lang.resolved(), Some("fr"));

        let lang = UILang {
            forced: None,
            selected: Some("de".to_string()),
            system: Some("en".to_string()),
        };
        assert_eq!(lang.resolved(), Some("de"));

        let lang = UILang {
            forced: None,
            selected: None,
            system: Some("en".to_string()),
        };
        assert_eq!(lang.resolved(), Some("en"));
    }

    #[test]
    fn ui_lang_set_selected_and_apply_html_lang_return_change_flags() {
        let mut lang = UILang {
            forced: None,
            selected: None,
            system: None,
        };

        assert!(lang.set_selected(Some("DE_de")));
        assert_eq!(lang.selected.as_deref(), Some("de-de"));
        assert!(!lang.set_selected(Some("de-de")));
        assert!(lang.set_selected(None));
        assert_eq!(lang.selected, None);

        assert!(lang.apply_html_lang(Some("EN_us")));
        assert_eq!(lang.forced.as_deref(), Some("en-us"));
        assert!(!lang.apply_html_lang(Some("en-us")));

        assert!(lang.apply_html_lang(Some("auto")));
        assert_eq!(lang.forced, None);
    }

    #[test]
    fn vars_fingerprint_is_stable_across_insertion_order_and_changes_on_value_change() {
        let mut a = UiLangVariables::default();
        a.set("player", "Alice");
        a.set("rank", "42");

        let mut b = UiLangVariables::default();
        b.set("rank", "42");
        b.set("player", "Alice");

        assert_eq!(vars_fingerprint(&a), vars_fingerprint(&b));

        b.set("rank", "43");
        assert_ne!(vars_fingerprint(&a), vars_fingerprint(&b));
    }

    #[test]
    fn id_pool_reuses_released_ids_in_fifo_order() {
        let mut pool = IdPool::new();
        let id0 = pool.acquire();
        let id1 = pool.acquire();
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);

        pool.release(id0);
        pool.release(id1);

        assert_eq!(pool.acquire(), 0);
        assert_eq!(pool.acquire(), 1);
        assert_eq!(pool.acquire(), 2);
    }

    #[test]
    fn ui_registry_add_get_get_mut_and_remove_work() {
        let mut registry = UiRegistry::new();

        registry.add("main".to_string(), source(Some("controller-a")));
        let main = registry.get("main").expect("main ui missing");
        assert_eq!(main.source_id, "main");
        assert_eq!(main.controller.as_deref(), Some("controller-a"));

        registry
            .get_mut("main")
            .expect("main ui missing")
            .controller = Some("controller-b".to_string());

        let main = registry.get("main").expect("main ui missing");
        assert_eq!(main.controller.as_deref(), Some("controller-b"));

        registry.use_ui("main");
        assert_eq!(registry.current, Some(vec!["main".to_string()]));
        assert!(registry.ui_update);

        registry.remove("main");
        assert!(registry.get("main").is_none());
        assert_eq!(registry.current, None);
    }

    #[test]
    fn ui_registry_add_and_use_multiple_and_switching_work() {
        let mut registry = UiRegistry::new();

        registry.add_and_use("a".to_string(), source(None));
        assert_eq!(registry.current, Some(vec!["a".to_string()]));

        registry.add("b".to_string(), source(Some("ctrl-b")));
        registry.remove_and_use("a", "b");
        assert!(registry.get("a").is_none());
        assert_eq!(registry.current, Some(vec!["b".to_string()]));

        registry.add_and_use_multiple(vec![
            ("x".to_string(), source(None)),
            ("y".to_string(), source(None)),
        ]);
        assert_eq!(registry.current, Some(vec!["x".to_string(), "y".to_string()]));
    }

    #[test]
    fn ui_registry_use_uis_filters_unknown_entries_and_can_clear_current() {
        let mut registry = UiRegistry::new();
        registry.add("ui1".to_string(), source(None));
        registry.add("ui2".to_string(), source(None));

        registry.use_uis(vec!["missing".to_string(), "ui2".to_string()]);
        assert_eq!(registry.current, Some(vec!["ui2".to_string()]));
        assert!(registry.ui_update);

        registry.use_uis(vec!["missing".to_string()]);
        assert_eq!(registry.current, None);
        assert!(registry.ui_update);

        registry.remove_all();
        assert!(registry.collection.is_empty());
        assert_eq!(registry.current, None);
    }

    #[test]
    fn extended_registry_plugin_initializes_registry_resources() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ExtendedRegistryPlugin));

        assert!(app.world().get_resource::<UiRegistry>().is_some());
        assert!(app.world().get_resource::<UiInitResource>().is_some());
    }

    #[test]
    fn keycode_to_char_maps_letters_digits_and_symbols() {
        assert_eq!(keycode_to_char(KeyCode::KeyA, false, false), Some('a'));
        assert_eq!(keycode_to_char(KeyCode::KeyA, true, false), Some('A'));

        // German keyboard style swap in implementation.
        assert_eq!(keycode_to_char(KeyCode::KeyY, true, false), Some('Z'));
        assert_eq!(keycode_to_char(KeyCode::KeyZ, true, false), Some('Y'));

        assert_eq!(keycode_to_char(KeyCode::Digit0, false, true), Some('}'));
        assert_eq!(keycode_to_char(KeyCode::Digit7, true, false), Some('/'));
        assert_eq!(keycode_to_char(KeyCode::IntlBackslash, false, true), Some('|'));
        assert_eq!(keycode_to_char(KeyCode::Space, false, false), Some(' '));
    }

    #[test]
    fn keycode_to_char_returns_none_for_unmapped_keys() {
        assert_eq!(keycode_to_char(KeyCode::Escape, false, false), None);
    }

    #[test]
    fn load_ui_camera_system_default_spawns_camera_with_layers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ExtendedUiConfiguration {
            order: 7,
            hdr_support: false,
            camera: ExtendedCam::Default,
            render_layers: vec![1, 3],
            assets_path: "assets/extended_ui/".to_string(),
            language_path: "assets/lang".to_string(),
        });
        app.add_systems(Update, super::super::load_ui_camera_system);
        app.update();

        let mut query = app.world_mut().query_filtered::<
            (&Camera, &RenderLayers, Has<Hdr>, Has<IsDefaultUiCamera>),
            With<super::super::UiCamera>,
        >();

        let rows: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(rows.len(), 1);

        let (camera, layers, has_hdr, has_default_marker) = rows[0];
        assert_eq!(camera.order, 7);
        assert_eq!(*layers, RenderLayers::from_layers(&[1, 3]));
        assert!(!has_hdr);
        assert!(has_default_marker);
    }

    #[test]
    fn load_ui_camera_system_updates_existing_camera_and_hdr_flag() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ExtendedUiConfiguration::default());
        app.add_systems(Update, super::super::load_ui_camera_system);

        app.update();

        {
            let mut cfg = app.world_mut().resource_mut::<ExtendedUiConfiguration>();
            cfg.order = 42;
            cfg.hdr_support = true;
            cfg.render_layers = vec![2, 4, 6];
        }
        app.update();

        let mut query = app.world_mut().query_filtered::<
            (&Camera, &RenderLayers, Has<Hdr>),
            With<super::super::UiCamera>,
        >();
        let rows: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(rows.len(), 1);
        let (camera, layers, has_hdr) = rows[0];
        assert_eq!(camera.order, 42);
        assert_eq!(*layers, RenderLayers::from_layers(&[2, 4, 6]));
        assert!(has_hdr);

        {
            let mut cfg = app.world_mut().resource_mut::<ExtendedUiConfiguration>();
            cfg.hdr_support = false;
        }
        app.update();

        let mut query = app.world_mut().query_filtered::<Has<Hdr>, With<super::super::UiCamera>>();
        let rows: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(rows, vec![false]);
    }

    #[test]
    fn load_ui_camera_system_simple_replaces_default_camera() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ExtendedUiConfiguration::default());
        app.add_systems(Update, super::super::load_ui_camera_system);

        app.update();

        {
            let mut cfg = app.world_mut().resource_mut::<ExtendedUiConfiguration>();
            cfg.camera = ExtendedCam::Simple;
            cfg.order = 11;
        }
        app.update();

        let mut query = app.world_mut().query_filtered::<
            (&Camera, Has<IsDefaultUiCamera>),
            With<super::super::UiCamera>,
        >();
        let rows: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(rows.len(), 1);
        let (camera, has_default_marker) = rows[0];
        assert_eq!(camera.order, 11);
        assert!(!has_default_marker);
    }

    #[test]
    fn load_ui_camera_system_none_despawns_default_camera() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ExtendedUiConfiguration::default());
        app.add_systems(Update, super::super::load_ui_camera_system);

        app.update();

        {
            let mut cfg = app.world_mut().resource_mut::<ExtendedUiConfiguration>();
            cfg.camera = ExtendedCam::None;
        }
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<super::super::UiCamera>>();
        let count = query.iter(app.world()).count();
        assert_eq!(count, 0);
    }

}
