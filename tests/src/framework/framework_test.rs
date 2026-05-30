#[cfg(test)]
mod unit_tests {
    use super::super::*;

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
    }
}
