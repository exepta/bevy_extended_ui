use crate::framework::ExtendedFrameworkConfiguration;
use crate::html::HtmlSource;
use crate::io::HtmlAsset;
use bevy::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

static TEMPLATE_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"template_name\s*:\s*"([^"]+)""#).unwrap());
static TEMPLATE_FILE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"template_file\s*:\s*"([^"]+)""#).unwrap());
static STYLES_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"styles\s*:\s*&?\s*\[([^\]]*)\]"#).unwrap());
static QUOTED_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""([^"]+)""#).unwrap());
static UI_COMPONENT_MARKER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?s)#\s*\[\s*(?:[\w:]+::)?ui_component(?:\s*\([^]]*\))?\s*\]"#).unwrap()
});

/// Parsed metadata from a `*.component.rs` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDefinition {
    pub template_name: String,
    pub template_file: String,
    pub styles: Vec<String>,
    pub source_dir_rel: String,
}

/// Plugin that boots the new component-based entry flow in `extended-framework` mode.
pub struct ExtendedComponentPlugin;

impl Plugin for ExtendedComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_framework_index);
    }
}

/// Loads `index.html` as a mandatory framework entrypoint.
fn load_framework_index(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    framework_config: Option<Res<ExtendedFrameworkConfiguration>>,
) {
    let framework_config = framework_config.as_deref().cloned().unwrap_or_default();

    if framework_config.index_html_file != "index.html" {
        panic!(
            "extended-framework requires `index.html` exactly. Found `{}`.",
            framework_config.index_html_file
        );
    }

    let index_path = PathBuf::from(&framework_config.asset_root_fs_path).join("index.html");
    if !index_path.exists() {
        panic!(
            "extended-framework entrypoint missing: expected `{}`.",
            index_path.display()
        );
    }

    let handle: Handle<HtmlAsset> = asset_server.load("index.html");
    commands.spawn(HtmlSource {
        handle,
        source_id: "framework-index".to_string(),
        controller: None,
    });
}

/// Collects and validates all component definitions from the configured Rust component root.
pub fn load_component_definitions(
    config: &ExtendedFrameworkConfiguration,
) -> Result<Vec<ComponentDefinition>, String> {
    let root = PathBuf::from(&config.rust_component_root);
    if !root.exists() {
        return Err(format!(
            "Rust component root not found: `{}`",
            root.display()
        ));
    }

    let mut rs_files = Vec::new();
    collect_component_rs_files(&root, &mut rs_files)
        .map_err(|err| format!("Failed to read component root `{}`: {err}", root.display()))?;

    let mut defs = Vec::new();
    for file in rs_files {
        defs.push(parse_component_definition_file(&file, &root)?);
    }
    Ok(defs)
}

fn collect_component_rs_files(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_component_rs_files(&path, out)?;
            continue;
        }

        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            if name.ends_with(".component.rs") {
                out.push(path);
            }
        }
    }
    Ok(())
}

fn parse_component_definition_file(
    path: &Path,
    root: &Path,
) -> Result<ComponentDefinition, String> {
    let text = fs::read_to_string(path).map_err(|err| {
        format!(
            "Failed to read component definition `{}`: {err}",
            path.display()
        )
    })?;
    ensure_ui_component_macro_used(path, &text)?;

    let template_name = TEMPLATE_NAME_RE
        .captures(&text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "Missing required field `template_name` in `{}`.",
                path.display()
            )
        })?;

    let template_file = TEMPLATE_FILE_RE
        .captures(&text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "Missing required field `template_file` in `{}`.",
                path.display()
            )
        })?;

    let styles = STYLES_RE
        .captures(&text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .map(|inner| {
            QUOTED_RE
                .captures_iter(&inner)
                .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| format!("Missing required field `styles` in `{}`.", path.display()))?;

    validate_component_name_contract(path, &template_file)?;
    let source_dir_rel = path
        .parent()
        .and_then(|parent| parent.strip_prefix(root).ok())
        .map(|p| normalize_path_like(p.to_string_lossy().as_ref()))
        .unwrap_or_default();

    Ok(ComponentDefinition {
        template_name,
        template_file,
        styles,
        source_dir_rel,
    })
}

fn ensure_ui_component_macro_used(path: &Path, text: &str) -> Result<(), String> {
    if UI_COMPONENT_MARKER_RE.is_match(text) {
        return Ok(());
    }

    Err(format!(
        "Missing required macro marker in `{}`. Add `#[bevy_extended_ui_macros::ui_component]` to the component definition.",
        path.display()
    ))
}

fn validate_component_name_contract(path: &Path, template_file: &str) -> Result<(), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Invalid component path: `{}`", path.display()))?;

    let expected_template = file_name
        .strip_suffix(".rs")
        .ok_or_else(|| format!("Invalid component filename: `{file_name}`"))?
        .to_string()
        + ".html";

    if expected_template != template_file {
        return Err(format!(
            "Component filename mismatch in `{}`. Expected `template_file: \"{}\"` but found `\"{}\"`.",
            path.display(),
            expected_template,
            template_file
        ));
    }

    Ok(())
}

/// Validates that all component HTML/CSS files referenced by definitions exist.
pub fn validate_component_assets(
    defs: &[ComponentDefinition],
    config: &ExtendedFrameworkConfiguration,
) -> Result<(), String> {
    let root =
        PathBuf::from(&config.asset_root_fs_path).join(trim_slashes(&config.assets_component_root));

    let mut seen_tags = HashSet::new();
    for def in defs {
        if !seen_tags.insert(def.template_name.clone()) {
            return Err(format!(
                "Duplicate template_name found: `{}`.",
                def.template_name
            ));
        }

        let template_path = resolve_component_asset_candidate(&root, def, &def.template_file);
        if !template_path.exists() {
            return Err(format!(
                "Component template missing: `{}`.",
                template_path.display()
            ));
        }

        for style in &def.styles {
            let style_path = resolve_component_asset_candidate(&root, def, style);
            if !style_path.exists() {
                return Err(format!(
                    "Component style missing: `{}`.",
                    style_path.display()
                ));
            }
        }
    }

    Ok(())
}

/// Loads one component html snippet by its configured template file.
pub fn load_component_template_html(
    def: &ComponentDefinition,
    config: &ExtendedFrameworkConfiguration,
) -> Result<String, String> {
    let root =
        PathBuf::from(&config.asset_root_fs_path).join(trim_slashes(&config.assets_component_root));
    let path = resolve_component_asset_candidate(&root, def, &def.template_file);
    fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read component html `{}`: {err}", path.display()))
}

fn trim_slashes(path: &str) -> String {
    path.trim_matches('/').trim_matches('\\').to_string()
}

fn normalize_path_like(path: &str) -> String {
    path.replace('\\', "/").trim_matches('/').to_string()
}

fn resolve_component_asset_candidate(
    root: &Path,
    def: &ComponentDefinition,
    file: &str,
) -> PathBuf {
    let normalized = normalize_path_like(file);
    if normalized.contains('/') {
        return root.join(normalized);
    }

    if def.source_dir_rel.is_empty() {
        return root.join(normalized);
    }

    let scoped = root.join(&def.source_dir_rel).join(&normalized);
    if scoped.exists() {
        return scoped;
    }
    root.join(normalized)
}

#[cfg(test)]
mod unit_tests {
    use super::*;
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
