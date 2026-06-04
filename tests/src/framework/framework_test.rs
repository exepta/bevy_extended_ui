#[cfg(test)]
mod unit_tests {
    use super::super::*;
    use bevy::prelude::{App, MinimalPlugins};
    use bevy_extended_ui::BeuStore;
    use bevy_extended_ui::lang::{UiSharedValues, refresh_shared_values};
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
}
