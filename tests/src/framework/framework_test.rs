#[cfg(test)]
mod unit_tests {
    use super::super::*;
    use bevy::prelude::{App, MinimalPlugins};
    use bevy_extended_ui::BeuStore;
    use bevy_extended_ui::html::converter::preprocess_template_directives_with_shared;
    use bevy_extended_ui::lang::{
        UiLangVariables, UiSharedValues, refresh_shared_values, serde_json::json,
    };
    use bevy_extended_ui::routing::{Router, Routes};
    use serde::Serialize;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(BeuStore, Clone, Default, PartialEq, Serialize)]
    struct Player {
        name: String,
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("bevy_extended_ui_framework_{prefix}_{stamp}"))
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir parent");
        }
        fs::write(path, content).expect("write");
    }

    fn write_route_component(
        asset_root: &Path,
        rust_root: &Path,
        name: &str,
        tag: &str,
        html: &str,
    ) {
        write_file(
            &rust_root.join(format!("{name}.component.rs")),
            &format!(
                r#"
                #[bevy_extended_ui_macros::ui_component]
                const COMPONENT: Component = Component {{
                    template_name: "{tag}",
                    template_file: "{name}.component.html",
                    styles: ["{name}.component.css"],
                }};
                "#
            ),
        );
        write_file(
            &asset_root
                .join("components")
                .join(format!("{name}.component.html")),
            html,
        );
        write_file(
            &asset_root
                .join("components")
                .join(format!("{name}.component.css")),
            "div { color: white; }",
        );
    }

    fn route_test_config(asset_root: &Path, rust_root: &Path) -> ExtendedFrameworkConfiguration {
        ExtendedFrameworkConfiguration {
            assets_component_root: "components".to_string(),
            rust_component_root: rust_root.to_string_lossy().to_string(),
            asset_root_fs_path: asset_root.to_string_lossy().to_string(),
            index_html_file: "index.html".to_string(),
        }
    }

    #[test]
    fn infer_component_controller_path_matches_angular_like_layout() {
        let cfg = ExtendedFrameworkConfiguration::default();
        let inferred =
            infer_component_controller_path("assets/ui/bevy_ang/hud/hud.component.html", &cfg);

        assert_eq!(inferred.as_deref(), Some("src/packages/hud.component.rs"));
    }

    #[test]
    fn infer_component_controller_path_ignores_non_component_templates() {
        let cfg = ExtendedFrameworkConfiguration::default();
        let inferred = infer_component_controller_path("assets/ui/bevy_ang/hud/hud.html", &cfg);

        assert_eq!(inferred, None);
    }

    #[test]
    fn compile_framework_template_keeps_html_and_infers_controller() {
        let mut cfg = ExtendedFrameworkConfiguration::default();
        cfg.index_html_file = "root.html".to_string();
        let result = compile_framework_template(
            "<html><body><p>Hello</p></body></html>",
            "ui/bevy_ang/menu/menu.component.html",
            &cfg,
        );

        assert_eq!(result.html, "<html><body><p>Hello</p></body></html>");
        assert_eq!(
            result.inferred_controller.as_deref(),
            Some("src/packages/menu.component.rs")
        );
        assert!(result.component_controllers.is_empty());
    }

    #[test]
    fn compile_framework_template_reports_inlined_component_controllers() {
        let base = unique_temp_dir("inlined_controllers");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");

        write_file(
            &rust_root.join("main.component.rs"),
            r#"
            #[bevy_extended_ui_macros::ui_component]
            const MAIN: Component = Component {
                template_name: "app-main",
                template_file: "main.component.html",
                styles: ["main.component.css"],
            };
            "#,
        );
        write_file(
            &asset_root.join("components/main.component.html"),
            "<section>{{ player.name }}</section>",
        );
        write_file(
            &asset_root.join("components/main.component.css"),
            "section { color: white; }",
        );

        let cfg = ExtendedFrameworkConfiguration {
            assets_component_root: "components".to_string(),
            rust_component_root: rust_root.to_string_lossy().to_string(),
            asset_root_fs_path: asset_root.to_string_lossy().to_string(),
            index_html_file: "index.html".to_string(),
        };

        let result = compile_framework_template(
            "<html><head></head><body><app-main /></body></html>",
            "index.html",
            &cfg,
        );

        assert!(result.html.contains("{{ player.name }}"));
        assert_eq!(
            result.component_controllers,
            vec![
                rust_root
                    .join("main.component.rs")
                    .to_string_lossy()
                    .to_string()
            ]
        );

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_preserves_inline_dollar_functions() {
        let base = unique_temp_dir("inline_dollar_functions");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");

        write_route_component(
            &asset_root,
            &rust_root,
            "main",
            "app-main",
            r#"<button onclick="$add(info.value, 1)">Increase</button>
<input onchange="$set(player.name, $event.value)">"#,
        );

        let cfg = route_test_config(&asset_root, &rust_root);
        let router = {
            let mut router = Router::default();
            router.configure(Routes::new().route("/", "app-main"));
            router
        };

        let result = compile_framework_template_with_router(
            "<html><head></head><body><router-outlet /></body></html>",
            "index.html",
            &cfg,
            Some(&router),
        );

        assert!(result.html.contains(r#"onclick="$add(info.value, 1)""#));
        assert!(
            result
                .html
                .contains(r#"onchange="$set(player.name, $event.value)""#)
        );
        assert!(!result.html.contains(r#"onclick="(info.value, 1)""#));
        assert!(!result.html.contains(r#"onchange="(player.name, .value)""#));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_resolves_router_outlet_to_active_route_component() {
        let base = unique_temp_dir("router_outlet");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");

        write_file(
            &rust_root.join("help.component.rs"),
            r#"
            #[bevy_extended_ui_macros::ui_component]
            const HELP: Component = Component {
                template_name: "app-help",
                template_file: "help.component.html",
                styles: ["help.component.css"],
            };
            "#,
        );
        write_file(
            &asset_root.join("components/help.component.html"),
            "<section>Help route</section>",
        );
        write_file(
            &asset_root.join("components/help.component.css"),
            "section { color: white; }",
        );

        let cfg = ExtendedFrameworkConfiguration {
            assets_component_root: "components".to_string(),
            rust_component_root: rust_root.to_string_lossy().to_string(),
            asset_root_fs_path: asset_root.to_string_lossy().to_string(),
            index_html_file: "index.html".to_string(),
        };

        let mut router = Router::default();
        router.configure(Routes::new().route("/help", "app-help"));
        router.navigate("/help");

        let result = compile_framework_template_with_router(
            "<html><head></head><body><router-outlet></router-outlet></body></html>",
            "index.html",
            &cfg,
            Some(&router),
        );

        assert!(result.html.contains("Help route"));
        assert!(!result.html.contains("router-outlet"));
        assert_eq!(
            result.component_controllers,
            vec![
                rust_root
                    .join("help.component.rs")
                    .to_string_lossy()
                    .to_string()
            ]
        );

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_resolves_self_closing_router_outlet() {
        let base = unique_temp_dir("router_outlet_self_closing");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");
        write_route_component(
            &asset_root,
            &rust_root,
            "main",
            "app-main",
            "<div>Home route</div>",
        );

        let cfg = route_test_config(&asset_root, &rust_root);
        let router = {
            let mut router = Router::default();
            router.configure(Routes::new().route("/", "app-main"));
            router
        };

        let result = compile_framework_template_with_router(
            "<html><head></head><body><router-outlet /></body></html>",
            "index.html",
            &cfg,
            Some(&router),
        );

        assert!(result.html.contains("Home route"));
        assert!(!result.html.contains("router-outlet"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_skips_router_work_without_outlet() {
        let base = unique_temp_dir("router_no_outlet");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");
        write_route_component(
            &asset_root,
            &rust_root,
            "main",
            "app-main",
            "<div>Home route</div>",
        );

        let cfg = route_test_config(&asset_root, &rust_root);
        let mut router = Router::default();
        router.configure(Routes::new().route("/", "app-main"));

        let result = compile_framework_template_with_router(
            "<html><head></head><body><app-main></app-main></body></html>",
            "index.html",
            &cfg,
            Some(&router),
        );

        assert!(result.html.contains("Home route"));
        assert!(!result.html.contains("beu-route"));
        assert!(!result.html.contains("router-outlet"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_keeps_active_load_route_visible() {
        let base = unique_temp_dir("router_keep_alive_active");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");
        write_route_component(
            &asset_root,
            &rust_root,
            "main",
            "app-main",
            "<div>Home route</div>",
        );

        let cfg = route_test_config(&asset_root, &rust_root);
        let mut router = Router::default();
        router.configure(Routes::new().route("/", bevy_extended_ui::load!("app-main")));

        let result = compile_framework_template_with_router(
            "<html><head></head><body><router-outlet></router-outlet></body></html>",
            "index.html",
            &cfg,
            Some(&router),
        );

        assert!(result.html.contains("Home route"));
        assert!(result.html.contains("beu-route-active"));
        assert!(!result.html.contains("beu-route-cached"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_keeps_component_use_directives_at_line_start() {
        let base = unique_temp_dir("router_keep_alive_use_directive");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");
        write_route_component(
            &asset_root,
            &rust_root,
            "main",
            "app-main",
            r#"@use "crate::data_structs::*";
<div>
  <p>Selected DataState: {{ data_state }}</p>
  <p>DataPack embedded state: {{ data_pack.state }}</p>
</div>"#,
        );

        let cfg = route_test_config(&asset_root, &rust_root);
        let mut router = Router::default();
        router.configure(Routes::new().route("/", bevy_extended_ui::load!("app-main")));

        let result = compile_framework_template_with_router(
            "<html><head></head><body><router-outlet></router-outlet></body></html>",
            "index.html",
            &cfg,
            Some(&router),
        );

        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "bevy_extended_ui_tests::data_structs::DataPack".to_string(),
            bevy_extended_ui::lang::serde_json::from_str(r#"{"state":"Inactive"}"#).unwrap(),
        );
        shared.values.insert(
            "bevy_extended_ui_tests::data_structs::DataState".to_string(),
            bevy_extended_ui::lang::serde_json::Value::String("Inactive".to_string()),
        );

        let rendered = preprocess_template_directives_with_shared(
            &result.html,
            &UiLangVariables::default(),
            &shared,
        );

        assert!(rendered.contains("Selected DataState: Inactive"));
        assert!(rendered.contains("DataPack embedded state: Inactive"));
        assert!(!rendered.contains("{{ data_state }}"));
        assert!(!rendered.contains("{{ data_pack.state }}"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_keeps_load_routes_in_router_outlet() {
        let base = unique_temp_dir("router_keep_alive");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");

        for (name, tag, content) in [
            ("main", "app-main", "Home route"),
            ("help", "app-help", "Help route"),
        ] {
            write_file(
                &rust_root.join(format!("{name}.component.rs")),
                &format!(
                    r#"
                    #[bevy_extended_ui_macros::ui_component]
                    const COMPONENT: Component = Component {{
                        template_name: "{tag}",
                        template_file: "{name}.component.html",
                        styles: ["{name}.component.css"],
                    }};
                    "#
                ),
            );
            write_file(
                &asset_root
                    .join("components")
                    .join(format!("{name}.component.html")),
                &format!("<div>{content}</div>"),
            );
            write_file(
                &asset_root
                    .join("components")
                    .join(format!("{name}.component.css")),
                "div { color: white; }",
            );
        }

        let cfg = ExtendedFrameworkConfiguration {
            assets_component_root: "components".to_string(),
            rust_component_root: rust_root.to_string_lossy().to_string(),
            asset_root_fs_path: asset_root.to_string_lossy().to_string(),
            index_html_file: "index.html".to_string(),
        };

        let mut router = Router::default();
        router.configure(
            Routes::new()
                .route("/", bevy_extended_ui::load!("app-main"))
                .route("/help", "app-help"),
        );
        router.navigate("/help");

        let result = compile_framework_template_with_router(
            "<html><head></head><body><router-outlet></router-outlet></body></html>",
            "index.html",
            &cfg,
            Some(&router),
        );

        assert!(result.html.contains("Home route"));
        assert!(result.html.contains("Help route"));
        assert!(result.html.contains("beu-route-cached"));
        assert!(result.html.contains("display: none"));
        assert!(result.html.contains("beu-route-active"));
        assert_eq!(result.component_controllers.len(), 2);

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_panics_for_missing_active_route_component() {
        let base = unique_temp_dir("router_missing_active");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");
        write_route_component(
            &asset_root,
            &rust_root,
            "main",
            "app-main",
            "<div>Home route</div>",
        );

        let cfg = route_test_config(&asset_root, &rust_root);
        let mut router = Router::default();
        router.configure(Routes::new().route("/", "app-missing"));

        let result = std::panic::catch_unwind(|| {
            compile_framework_template_with_router(
                "<html><head></head><body><router-outlet></router-outlet></body></html>",
                "index.html",
                &cfg,
                Some(&router),
            );
        });

        assert!(result.is_err());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_panics_for_missing_keep_alive_component() {
        let base = unique_temp_dir("router_missing_keep_alive");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");
        write_route_component(
            &asset_root,
            &rust_root,
            "help",
            "app-help",
            "<div>Help route</div>",
        );

        let cfg = route_test_config(&asset_root, &rust_root);
        let mut router = Router::default();
        router.configure(
            Routes::new()
                .route("/", bevy_extended_ui::load!("app-missing"))
                .route("/help", "app-help"),
        );
        router.navigate("/help");

        let result = std::panic::catch_unwind(|| {
            compile_framework_template_with_router(
                "<html><head></head><body><router-outlet></router-outlet></body></html>",
                "index.html",
                &cfg,
                Some(&router),
            );
        });

        assert!(result.is_err());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_framework_template_preloads_all_route_component_styles() {
        let base = unique_temp_dir("router_style_preload");
        let asset_root = base.join("assets");
        let rust_root = base.join("src/packages");

        for (name, tag) in [("main", "app-main"), ("help", "app-help")] {
            write_file(
                &rust_root.join(format!("{name}.component.rs")),
                &format!(
                    r#"
                    #[bevy_extended_ui_macros::ui_component]
                    const COMPONENT: Component = Component {{
                        template_name: "{tag}",
                        template_file: "{name}.component.html",
                        styles: ["{name}.component.css"],
                    }};
                    "#
                ),
            );
            write_file(
                &asset_root
                    .join("components")
                    .join(format!("{name}.component.html")),
                &format!("<section>{name}</section>"),
            );
            write_file(
                &asset_root
                    .join("components")
                    .join(format!("{name}.component.css")),
                "section { color: white; }",
            );
        }

        let cfg = ExtendedFrameworkConfiguration {
            assets_component_root: "components".to_string(),
            rust_component_root: rust_root.to_string_lossy().to_string(),
            asset_root_fs_path: asset_root.to_string_lossy().to_string(),
            index_html_file: "index.html".to_string(),
        };

        let mut router = Router::default();
        router.configure(
            Routes::new()
                .route("/", "app-main")
                .route("/help", "app-help")
                .fallback("app-main"),
        );

        let result = compile_framework_template_with_router(
            "<html><head></head><body><router-outlet></router-outlet></body></html>",
            "index.html",
            &cfg,
            Some(&router),
        );

        assert!(
            result
                .html
                .contains("href=\"components/main.component.css\"")
        );
        assert!(
            result
                .html
                .contains("href=\"components/help.component.css\"")
        );

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn beu_store_derive_registers_type_on_framework_startup() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ExtendedFrameworkPlugin));
        app.update();

        let store = app.world().resource::<UiBindingStore>();

        assert!(store.contains_key("Player"));
        assert!(store.get_store::<Player>().is_some());
        assert!(store.known_types().any(|known| known == "Player"));
        assert!(store.known_types().any(|known| known.ends_with("::Player")));
    }

    #[test]
    fn ui_binding_store_revision_changes_only_for_different_values() {
        let mut store = UiBindingStore::default();

        assert!(store.set("score", 1_u32));
        let first_revision = store.revision();

        assert!(!store.set("score", 1_u32));
        assert_eq!(store.revision(), first_revision);

        assert!(store.set("score", 2_u32));
        assert_ne!(store.revision(), first_revision);
        assert_eq!(store.get::<u32>("score"), Some(&2));
    }

    #[test]
    fn ui_binding_store_path_json_keeps_direct_primitive_types() {
        let mut store = UiBindingStore::default();
        store.set("flag", false);
        store.set("small", 1_u8);
        store.set("wide", 1_u128);
        store.set("signed", -1_isize);
        store.set("float", 1.0_f64);
        store.set("text", String::from("old"));

        assert!(store.set_path_json("flag", json!(true)));
        assert!(store.set_path_json("small", json!(7)));
        assert!(store.set_path_json("wide", json!("340282366920938463463374607431768211455")));
        assert!(store.set_path_json("signed", json!("-12")));
        assert!(store.set_path_json("float", json!("2.5")));
        assert!(store.set_path_json("text", json!(42)));

        assert_eq!(store.get::<bool>("flag"), Some(&true));
        assert_eq!(store.get::<u8>("small"), Some(&7_u8));
        assert_eq!(store.get::<u128>("wide"), Some(&u128::MAX));
        assert_eq!(store.get::<isize>("signed"), Some(&-12_isize));
        assert_eq!(store.get::<f64>("float"), Some(&2.5_f64));
        assert_eq!(store.get::<String>("text"), Some(&String::from("42")));
    }

    #[test]
    fn ui_binding_store_path_json_resolves_template_aliases() {
        let mut store = UiBindingStore::default();
        store.set_store(Player {
            name: String::from("Ada"),
        });

        assert!(store.set_path_json("player.name", json!("Grace")));
        assert_eq!(store.json_path("player.name"), Some(json!("Grace")));
        assert_eq!(store.json_path("Player.name"), Some(json!("Grace")));
    }

    #[test]
    fn ui_binding_store_values_are_exposed_to_template_shared_values() {
        let mut app = App::new();
        app.init_resource::<UiSharedValues>();
        app.init_resource::<UiBindingStore>();

        app.world_mut()
            .resource_mut::<UiBindingStore>()
            .set("score", 7_u32);

        refresh_shared_values(app.world_mut());
        sync_ui_binding_store_values(app.world_mut());

        let shared = app.world().resource::<UiSharedValues>();
        assert_eq!(
            shared.values.get("score").and_then(|value| value.as_u64()),
            Some(7)
        );
    }

    #[test]
    fn ui_binding_store_register_type_and_entry_getters_work() {
        #[derive(Clone, PartialEq)]
        struct RawOnly(u32);

        let mut store = UiBindingStore::default();
        assert!(store.register_type::<Player>("player", "crate::Player"));
        let first_revision = store.revision();
        assert!(!store.register_type::<Player>("player", "crate::Player"));
        assert_eq!(store.revision(), first_revision);

        assert!(store.register_type::<Player>("player", "crate::models::Player"));
        assert!(store.revision() > first_revision);
        assert!(
            store
                .known_types()
                .any(|known| known == "crate::models::Player")
        );

        assert!(store.set_raw("raw", RawOnly(7)));
        let entry = store.data.get("raw").expect("raw entry");
        assert_eq!(entry.type_name(), "RawOnly");
        assert!(entry.type_path().ends_with("RawOnly"));
        assert!(entry.has_value());
        assert!(entry.revision() > 0);
        assert_eq!(entry.get::<RawOnly>().map(|value| value.0), Some(7));
        assert!(entry.json().is_none());

        let value = store
            .data
            .get("raw")
            .and_then(|entry| entry.get::<RawOnly>());
        assert_eq!(value.map(|value| value.0), Some(7));
    }

    #[test]
    fn ui_binding_store_direct_json_conversion_edges_work() {
        let mut store = UiBindingStore::default();
        store.set("flag", false);
        store.set("small", 1_u8);
        store.set("signed", 1_i8);
        store.set("float32", 1.0_f32);
        store.set("float64", 1.0_f64);
        store.set("text", String::from("old"));
        store.set("json", json!({"old": true}));

        assert!(store.set_path_json("flag", json!("yes")));
        assert_eq!(store.get::<bool>("flag"), Some(&true));
        assert!(!store.set_path_json("flag", json!("maybe")));

        assert!(!store.set_path_json("small", json!(999)));
        assert_eq!(store.get::<u8>("small"), Some(&1_u8));
        assert!(store.set_path_json("signed", json!("-5")));
        assert_eq!(store.get::<i8>("signed"), Some(&-5_i8));

        assert!(store.set_path_json("float32", json!("2.25")));
        assert_eq!(store.get::<f32>("float32"), Some(&2.25_f32));
        assert!(store.set_path_json("float64", json!(false)));
        assert_eq!(store.get::<f64>("float64"), Some(&0.0_f64));

        assert!(store.set_path_json("text", json!([1, 2])));
        assert_eq!(store.get::<String>("text"), Some(&"[1,2]".to_string()));

        assert!(store.set_path_json("json", json!(["new"])));
        assert_eq!(
            store.get::<bevy_extended_ui::lang::serde_json::Value>("json"),
            Some(&json!(["new"]))
        );
    }

    #[test]
    fn ui_binding_store_path_edge_cases_work() {
        let mut store = UiBindingStore::default();

        assert!(!store.set_path_json("", json!(1)));
        assert!(!store.set_path_json("info.", json!(1)));
        assert_eq!(store.json_path(""), None);
        assert_eq!(store.json_path(".value"), None);

        assert!(store.set_path_json("info.value", json!(1)));
        assert!(!store.set_path_json("info.value", json!(1)));
        assert_eq!(store.json_path("info.value"), Some(json!(1)));

        assert!(store.set_path_json("info.items", json!(["zero", "one"])));
        assert_eq!(store.json_path("info.items.1"), Some(json!("one")));
        assert_eq!(store.json_path("info.items.x"), None);
        assert_eq!(store.json_path("info.items.3"), None);
        assert_eq!(store.json_path("info.value.missing"), None);
    }
}
