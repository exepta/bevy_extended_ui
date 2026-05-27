use bevy::prelude::*;

/// Configuration for the experimental extended framework mode.
///
/// - `assets_component_root`: root folder (inside `assets/`) for Angular-like components.
/// - `rust_component_root`: root folder (inside project `src/`) for component logic files.
#[derive(Resource, Debug, Clone)]
pub struct ExtendedFrameworkConfiguration {
    pub assets_component_root: String,
    pub rust_component_root: String,
}

impl Default for ExtendedFrameworkConfiguration {
    fn default() -> Self {
        Self {
            assets_component_root: "ui/bevy_ang".to_string(),
            rust_component_root: "src/packages".to_string(),
        }
    }
}

/// Result of the framework pre-compile phase.
///
/// For the base implementation this is intentionally a no-op transform for HTML,
/// plus an optional inferred component-controller path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameworkCompileResult {
    pub html: String,
    pub inferred_controller: Option<String>,
}

/// Plugin for initializing resources used by the extended framework.
pub struct ExtendedFrameworkPlugin;

impl Plugin for ExtendedFrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtendedFrameworkConfiguration>();
    }
}

/// Compiles an HTML template in extended framework mode.
///
/// Current base behavior:
/// - HTML passes through unchanged.
/// - If the file looks like `*.component.html` under `assets_component_root`,
///   an inferred Rust component path is returned.
pub fn compile_framework_template(
    template_html: &str,
    source_path: &str,
    config: &ExtendedFrameworkConfiguration,
) -> FrameworkCompileResult {
    FrameworkCompileResult {
        html: template_html.to_string(),
        inferred_controller: infer_component_controller_path(source_path, config),
    }
}

/// Infers a Rust component source path from an HTML component source path.
///
/// Example:
/// - HTML: `assets/ui/bevy_ang/hud/hud.component.html`
/// - Rust: `src/packages/hud.component.rs`
pub fn infer_component_controller_path(
    source_path: &str,
    config: &ExtendedFrameworkConfiguration,
) -> Option<String> {
    let source = normalize_source_path(source_path);
    let root = normalize_root(&config.assets_component_root);

    if root.is_empty() {
        return None;
    }

    let expected_prefix = format!("{root}/");
    if !source.starts_with(&expected_prefix) {
        return None;
    }

    let file_name = source.rsplit('/').next()?;
    if !file_name.ends_with(".component.html") {
        return None;
    }

    let rust_file = file_name
        .strip_suffix(".html")
        .map(|name| format!("{name}.rs"))?;
    let rust_root = normalize_root(&config.rust_component_root);
    if rust_root.is_empty() {
        return Some(rust_file);
    }

    Some(format!("{rust_root}/{rust_file}"))
}

fn normalize_source_path(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_string();
    }
    if let Some(rest) = normalized.strip_prefix("assets/") {
        normalized = rest.to_string();
    }
    normalized.trim_matches('/').to_string()
}

fn normalize_root(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_string();
    }
    if let Some(rest) = normalized.strip_prefix("assets/") {
        normalized = rest.to_string();
    }
    normalized.trim_matches('/').to_string()
}

#[cfg(test)]
mod unit_tests {
    use super::*;

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
        let cfg = ExtendedFrameworkConfiguration::default();
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
