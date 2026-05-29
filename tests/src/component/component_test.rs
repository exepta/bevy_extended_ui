#[cfg(test)]
mod unit_tests {
    use super::super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_component_definition_requires_standard_fields() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("bevy_extended_ui_component_test_{stamp}"));
        fs::create_dir_all(&base).expect("mkdir");
        let file = base.join("main.component.rs");
        fs::write(
            &file,
            r#"
            #[bevy_extended_ui_macros::ui_component]
            const MAIN: Component = Component {
                template_name: "app-main",
                template_file: "main.component.html",
                styles: ["main.component.css"],
            };
            "#,
        )
        .expect("write");

        let def = parse_component_definition_file(&file, &base).expect("definition");
        assert_eq!(def.template_name, "app-main");
        assert_eq!(def.template_file, "main.component.html");
        assert_eq!(def.styles, vec!["main.component.css"]);
        assert_eq!(def.source_dir_rel, "");

        let _ = fs::remove_file(&file);
        let _ = fs::remove_dir_all(&base);
    }
}
