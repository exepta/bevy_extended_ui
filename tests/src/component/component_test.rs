#[cfg(test)]
mod unit_tests {
    use super::super::*;
    use crate::framework::ExtendedFrameworkConfiguration;
    use bevy::asset::AssetPlugin;
    use bevy::prelude::*;
    use bevy_extended_ui::html::HtmlSource;
    use bevy_extended_ui::io::HtmlAsset;
    use std::fs;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("bevy_extended_ui_{prefix}_{stamp}"))
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir parent");
        }
        fs::write(path, content).expect("write");
    }

    fn test_component_source(template_file: &str, styles: &[&str]) -> String {
        let styles_expr = styles
            .iter()
            .map(|style| format!("\"{style}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"
            #[bevy_extended_ui_macros::ui_component]
            const MAIN: Component = Component {{
                template_name: "app-main",
                template_file: "{template_file}",
                styles: [{styles_expr}],
            }};
            "#
        )
    }

    #[test]
    fn parse_component_definition_requires_standard_fields() {
        let base = unique_temp_dir("component_test");
        fs::create_dir_all(&base).expect("mkdir");
        let file = base.join("main.component.rs");
        write_file(
            &file,
            &test_component_source("main.component.html", &["main.component.css"]),
        );

        let cfg = ExtendedFrameworkConfiguration {
            rust_component_root: base.to_string_lossy().to_string(),
            ..Default::default()
        };

        let defs = load_component_definitions(&cfg).expect("definitions");
        assert_eq!(defs.len(), 1);
        let def = &defs[0];
        assert_eq!(def.template_name, "app-main");
        assert_eq!(def.template_file, "main.component.html");
        assert_eq!(def.styles, vec!["main.component.css"]);
        assert_eq!(def.source_dir_rel, "");

        let _ = fs::remove_file(&file);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn load_component_definitions_fails_when_root_missing() {
        let missing_root = unique_temp_dir("missing_component_root");
        let cfg = ExtendedFrameworkConfiguration {
            rust_component_root: missing_root.to_string_lossy().to_string(),
            ..Default::default()
        };

        let err = load_component_definitions(&cfg).expect_err("expected missing-root error");
        assert!(err.contains("Rust component root not found"));
    }

    #[test]
    fn load_component_definitions_fails_without_macro_marker() {
        let base = unique_temp_dir("component_missing_macro");
        let file = base.join("main.component.rs");
        write_file(
            &file,
            r#"
            const MAIN: Component = Component {
                template_name: "app-main",
                template_file: "main.component.html",
                styles: ["main.component.css"],
            };
            "#,
        );

        let cfg = ExtendedFrameworkConfiguration {
            rust_component_root: base.to_string_lossy().to_string(),
            ..Default::default()
        };

        let err = load_component_definitions(&cfg).expect_err("expected missing-macro error");
        assert!(err.contains("Missing required macro marker"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn load_component_definitions_fails_on_template_name_contract_mismatch() {
        let base = unique_temp_dir("component_name_contract");
        let file = base.join("main.component.rs");
        write_file(
            &file,
            &test_component_source("other.component.html", &["main.component.css"]),
        );

        let cfg = ExtendedFrameworkConfiguration {
            rust_component_root: base.to_string_lossy().to_string(),
            ..Default::default()
        };

        let err = load_component_definitions(&cfg).expect_err("expected filename-contract error");
        assert!(err.contains("Component filename mismatch"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn load_component_definitions_reads_nested_component_directory() {
        let base = unique_temp_dir("nested_component_dir");
        let nested = base.join("pages/home");
        let file = nested.join("main.component.rs");
        write_file(
            &file,
            &test_component_source("main.component.html", &["main.component.css"]),
        );

        let cfg = ExtendedFrameworkConfiguration {
            rust_component_root: base.to_string_lossy().to_string(),
            ..Default::default()
        };

        let defs = load_component_definitions(&cfg).expect("definitions");
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].source_dir_rel, "pages/home");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn validate_component_assets_rejects_duplicate_template_name() {
        let assets_root = unique_temp_dir("component_assets_duplicate");
        let component_assets_root = assets_root.join("assets/components");
        write_file(
            &component_assets_root.join("a.component.html"),
            "<div>a</div>",
        );
        write_file(
            &component_assets_root.join("a.component.css"),
            "div { color: red; }",
        );

        let defs = vec![
            ComponentDefinition {
                template_name: "duplicate-tag".to_string(),
                template_file: "a.component.html".to_string(),
                styles: vec!["a.component.css".to_string()],
                source_dir_rel: String::new(),
            },
            ComponentDefinition {
                template_name: "duplicate-tag".to_string(),
                template_file: "b.component.html".to_string(),
                styles: vec!["b.component.css".to_string()],
                source_dir_rel: String::new(),
            },
        ];

        let cfg = ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            assets_component_root: "assets/components".to_string(),
            ..Default::default()
        };
        let err = validate_component_assets(&defs, &cfg).expect_err("expected duplicate error");
        assert!(err.contains("Duplicate template_name"));

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn validate_component_assets_accepts_scoped_template_and_style_files() {
        let assets_root = unique_temp_dir("component_assets_ok");
        let component_assets_root = assets_root.join("assets/components");
        let source_dir_rel = "cards/profile".to_string();
        let scoped_dir = component_assets_root.join(&source_dir_rel);

        write_file(&scoped_dir.join("main.component.html"), "<div>ok</div>");
        write_file(
            &scoped_dir.join("main.component.css"),
            "div { color: red; }",
        );

        let defs = vec![ComponentDefinition {
            template_name: "card-profile".to_string(),
            template_file: "main.component.html".to_string(),
            styles: vec!["main.component.css".to_string()],
            source_dir_rel,
        }];

        let cfg = ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            assets_component_root: "assets/components".to_string(),
            ..Default::default()
        };

        validate_component_assets(&defs, &cfg).expect("expected assets to validate");

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn validate_component_assets_fails_when_style_missing() {
        let assets_root = unique_temp_dir("component_assets_missing_style");
        let component_assets_root = assets_root.join("assets/components");

        write_file(
            &component_assets_root.join("main.component.html"),
            "<div>only template</div>",
        );

        let defs = vec![ComponentDefinition {
            template_name: "main-tag".to_string(),
            template_file: "main.component.html".to_string(),
            styles: vec!["main.component.css".to_string()],
            source_dir_rel: String::new(),
        }];

        let cfg = ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            assets_component_root: "assets/components".to_string(),
            ..Default::default()
        };

        let err = validate_component_assets(&defs, &cfg).expect_err("expected missing-style error");
        assert!(err.contains("Component style missing"));

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn load_component_template_html_reads_scoped_component_template() {
        let assets_root = unique_temp_dir("component_template_read");
        let component_assets_root = assets_root.join("assets/components");
        let scoped_dir = component_assets_root.join("widgets/nav");
        write_file(
            &scoped_dir.join("main.component.html"),
            "<section>content</section>",
        );

        let def = ComponentDefinition {
            template_name: "nav-main".to_string(),
            template_file: "main.component.html".to_string(),
            styles: vec![],
            source_dir_rel: "widgets/nav".to_string(),
        };

        let cfg = ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            assets_component_root: "assets/components".to_string(),
            ..Default::default()
        };

        let html = load_component_template_html(&def, &cfg).expect("template html");
        assert!(html.contains("content"));

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn load_component_definitions_fails_when_template_name_is_missing() {
        let base = unique_temp_dir("component_missing_template_name");
        let file = base.join("main.component.rs");
        write_file(
            &file,
            r#"
            #[bevy_extended_ui_macros::ui_component]
            const MAIN: Component = Component {
                template_file: "main.component.html",
                styles: ["main.component.css"],
            };
            "#,
        );

        let cfg = ExtendedFrameworkConfiguration {
            rust_component_root: base.to_string_lossy().to_string(),
            ..Default::default()
        };

        let err = load_component_definitions(&cfg).expect_err("expected missing-template_name");
        assert!(err.contains("Missing required field `template_name`"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn load_component_definitions_fails_when_template_file_is_missing() {
        let base = unique_temp_dir("component_missing_template_file");
        let file = base.join("main.component.rs");
        write_file(
            &file,
            r#"
            #[bevy_extended_ui_macros::ui_component]
            const MAIN: Component = Component {
                template_name: "app-main",
                styles: ["main.component.css"],
            };
            "#,
        );

        let cfg = ExtendedFrameworkConfiguration {
            rust_component_root: base.to_string_lossy().to_string(),
            ..Default::default()
        };

        let err = load_component_definitions(&cfg).expect_err("expected missing-template_file");
        assert!(err.contains("Missing required field `template_file`"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn validate_component_assets_fails_when_template_file_missing() {
        let assets_root = unique_temp_dir("component_assets_missing_template");
        let component_assets_root = assets_root.join("assets/components");
        write_file(
            &component_assets_root.join("main.component.css"),
            "div { color: red; }",
        );

        let defs = vec![ComponentDefinition {
            template_name: "main-tag".to_string(),
            template_file: "main.component.html".to_string(),
            styles: vec!["main.component.css".to_string()],
            source_dir_rel: String::new(),
        }];

        let cfg = ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            assets_component_root: "assets/components".to_string(),
            ..Default::default()
        };

        let err =
            validate_component_assets(&defs, &cfg).expect_err("expected missing-template error");
        assert!(err.contains("Component template missing"));

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn validate_component_assets_accepts_explicit_relative_paths() {
        let assets_root = unique_temp_dir("component_assets_explicit_paths");
        let component_assets_root = assets_root.join("assets/components");
        write_file(
            &component_assets_root.join("nested/main.component.html"),
            "<div>ok</div>",
        );
        write_file(
            &component_assets_root.join("nested/main.component.css"),
            "div { color: green; }",
        );

        let defs = vec![ComponentDefinition {
            template_name: "nested-main".to_string(),
            template_file: "nested/main.component.html".to_string(),
            styles: vec!["nested/main.component.css".to_string()],
            source_dir_rel: "ignored/by/explicit/path".to_string(),
        }];

        let cfg = ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            assets_component_root: "assets/components".to_string(),
            ..Default::default()
        };

        validate_component_assets(&defs, &cfg).expect("explicit relative paths should resolve");

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn load_component_template_html_fails_for_missing_file() {
        let assets_root = unique_temp_dir("component_template_missing");
        let def = ComponentDefinition {
            template_name: "missing-template".to_string(),
            template_file: "missing.component.html".to_string(),
            styles: vec![],
            source_dir_rel: "widgets/nav".to_string(),
        };

        let cfg = ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            assets_component_root: "assets/components".to_string(),
            ..Default::default()
        };

        let err =
            load_component_template_html(&def, &cfg).expect_err("expected template read failure");
        assert!(err.contains("Failed to read component html"));

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn extended_component_plugin_spawns_framework_index_source_on_startup() {
        let assets_root = unique_temp_dir("component_plugin_startup_ok");
        write_file(
            &assets_root.join("index.html"),
            "<body><h1>Index</h1></body>",
        );

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<HtmlAsset>();
        app.insert_resource(ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            index_html_file: "index.html".to_string(),
            ..Default::default()
        });
        app.add_plugins(ExtendedComponentPlugin);

        app.update();

        let sources = {
            let world = app.world_mut();
            let mut query = world.query::<&HtmlSource>();
            query.iter(world).collect::<Vec<_>>()
        };
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_id, "framework-index");

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn extended_component_plugin_panics_when_index_html_name_is_overridden() {
        let assets_root = unique_temp_dir("component_plugin_bad_index_name");
        write_file(&assets_root.join("index.html"), "<body>ok</body>");

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<HtmlAsset>();
        app.insert_resource(ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            index_html_file: "not-index.html".to_string(),
            ..Default::default()
        });
        app.add_plugins(ExtendedComponentPlugin);

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            app.update();
        }));
        assert!(panic_result.is_err());

        let _ = fs::remove_dir_all(&assets_root);
    }

    #[test]
    fn extended_component_plugin_panics_when_index_file_missing() {
        let assets_root = unique_temp_dir("component_plugin_missing_index");
        fs::create_dir_all(&assets_root).expect("mkdir");

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<HtmlAsset>();
        app.insert_resource(ExtendedFrameworkConfiguration {
            asset_root_fs_path: assets_root.to_string_lossy().to_string(),
            index_html_file: "index.html".to_string(),
            ..Default::default()
        });
        app.add_plugins(ExtendedComponentPlugin);

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            app.update();
        }));
        assert!(panic_result.is_err());

        let _ = fs::remove_dir_all(&assets_root);
    }
}
