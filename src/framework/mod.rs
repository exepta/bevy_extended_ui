use bevy::prelude::*;
use regex::Regex;
use std::collections::BTreeSet;

use crate::component::{
    load_component_definitions, load_component_template_html, validate_component_assets,
};

/// Configuration for the experimental extended framework mode.
///
/// - `assets_component_root`: root folder (inside `assets/`) for Angular-like components.
/// - `rust_component_root`: root folder (inside project `src/`) for component logic files.
#[derive(Resource, Debug, Clone)]
pub struct ExtendedFrameworkConfiguration {
    pub assets_component_root: String,
    pub rust_component_root: String,
    pub asset_root_fs_path: String,
    pub index_html_file: String,
}

impl Default for ExtendedFrameworkConfiguration {
    /// Handles `default` in the extended UI workflow.
    fn default() -> Self {
        Self {
            assets_component_root: "ui/bevy_ang".to_string(),
            rust_component_root: "src/packages".to_string(),
            asset_root_fs_path: "assets".to_string(),
            index_html_file: "index.html".to_string(),
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
    /// Handles `build` in the extended UI workflow.
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
    let source = normalize_source_path(source_path);
    let mut html = template_html.to_string();
    if source == normalize_source_path(&config.index_html_file) {
        if let Err(err) = compile_index_template(&mut html, config) {
            panic!("extended-framework compile failed for index.html: {err}");
        }
    }

    FrameworkCompileResult {
        html,
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

/// Handles `normalize_source_path` in the extended UI workflow.
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

/// Handles `normalize_root` in the extended UI workflow.
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

/// Handles `compile_index_template` in the extended UI workflow.
fn compile_index_template(
    index_html: &mut String,
    config: &ExtendedFrameworkConfiguration,
) -> Result<(), String> {
    let defs = load_component_definitions(config)?;
    validate_component_assets(&defs, config)?;

    let mut used_style_hrefs: BTreeSet<String> = BTreeSet::new();

    for _ in 0..16 {
        let mut replaced = false;

        for def in &defs {
            let component_html = load_component_template_html(def, config)?;
            let tag_name = regex::escape(&def.template_name);

            let full_tag_re = Regex::new(&format!(
                r"(?is)<\s*{tag}\b[^>]*>.*?</\s*{tag}\s*>",
                tag = tag_name
            ))
            .map_err(|err| format!("invalid regex for tag `{}`: {err}", def.template_name))?;
            let self_closing_re =
                Regex::new(&format!(r"(?is)<\s*{tag}\b[^>]*/\s*>", tag = tag_name)).map_err(
                    |err| format!("invalid regex for tag `{}`: {err}", def.template_name),
                )?;

            if full_tag_re.is_match(index_html) || self_closing_re.is_match(index_html) {
                *index_html = full_tag_re
                    .replace_all(index_html, component_html.as_str())
                    .to_string();
                *index_html = self_closing_re
                    .replace_all(index_html, component_html.as_str())
                    .to_string();
                for style in &def.styles {
                    used_style_hrefs.insert(build_component_style_href(
                        &config.assets_component_root,
                        &def.source_dir_rel,
                        style,
                    ));
                }
                replaced = true;
            }
        }

        if !replaced {
            break;
        }
    }

    inject_component_styles(index_html, used_style_hrefs);
    Ok(())
}

/// Handles `inject_component_styles` in the extended UI workflow.
fn inject_component_styles(html: &mut String, style_hrefs: BTreeSet<String>) {
    if style_hrefs.is_empty() {
        return;
    }

    let mut links = String::new();

    for href in style_hrefs {
        let marker = format!("href=\"{href}\"");
        if html.contains(&marker) {
            continue;
        }
        links.push_str(&format!("<link rel=\"stylesheet\" href=\"{href}\">\n"));
    }

    if links.is_empty() {
        return;
    }

    let lower = html.to_ascii_lowercase();
    if let Some(pos) = lower.find("</head>") {
        html.insert_str(pos, &links);
    } else {
        html.insert_str(0, &links);
    }
}

/// Handles `build_component_style_href` in the extended UI workflow.
fn build_component_style_href(component_root: &str, source_dir_rel: &str, style: &str) -> String {
    let root = normalize_root(component_root);
    let style = normalize_root(style);
    let source_dir_rel = normalize_root(source_dir_rel);

    let relative = if style.contains('/') {
        style
    } else if source_dir_rel.is_empty() {
        style
    } else {
        format!("{source_dir_rel}/{style}")
    };

    if root.is_empty() {
        relative
    } else {
        format!("{root}/{relative}")
    }
}
