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
    /// Handles `build` in the extended UI workflow.
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
    let component_root = PathBuf::from(&config.rust_component_root);
    if !component_root.exists() {
        return Err(format!(
            "Rust component root not found: `{}`",
            component_root.display()
        ));
    }

    let mut component_files = Vec::new();
    collect_component_rs_files(&component_root, &mut component_files).map_err(|err| {
        format!(
            "Failed to read component root `{}`: {err}",
            component_root.display()
        )
    })?;

    let mut definitions = Vec::new();
    for component_file in component_files {
        definitions.push(parse_component_definition_file(
            &component_file,
            &component_root,
        )?);
    }
    Ok(definitions)
}

/// Handles `collect_component_rs_files` in the extended UI workflow.
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

/// Handles `parse_component_definition_file` in the extended UI workflow.
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

    let template_name = required_captured_field(&TEMPLATE_NAME_RE, &text, "template_name", path)?;

    let template_file = required_captured_field(&TEMPLATE_FILE_RE, &text, "template_file", path)?;

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

/// Extracts a required capture group (index 1) and trims it.
fn required_captured_field(
    pattern: &Regex,
    text: &str,
    field_name: &str,
    path: &Path,
) -> Result<String, String> {
    pattern
        .captures(text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "Missing required field `{field_name}` in `{}`.",
                path.display()
            )
        })
}

/// Handles `ensure_ui_component_macro_used` in the extended UI workflow.
fn ensure_ui_component_macro_used(path: &Path, text: &str) -> Result<(), String> {
    if UI_COMPONENT_MARKER_RE.is_match(text) {
        return Ok(());
    }

    Err(format!(
        "Missing required macro marker in `{}`. Add `#[bevy_extended_ui_macros::ui_component]` to the component definition.",
        path.display()
    ))
}

/// Handles `validate_component_name_contract` in the extended UI workflow.
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
    definitions: &[ComponentDefinition],
    config: &ExtendedFrameworkConfiguration,
) -> Result<(), String> {
    let assets_root =
        PathBuf::from(&config.asset_root_fs_path).join(trim_slashes(&config.assets_component_root));

    let mut seen_tags = HashSet::new();
    for definition in definitions {
        if !seen_tags.insert(definition.template_name.clone()) {
            return Err(format!(
                "Duplicate template_name found: `{}`.",
                definition.template_name
            ));
        }

        let template_path =
            resolve_component_asset_candidate(&assets_root, definition, &definition.template_file);
        if !template_path.exists() {
            return Err(format!(
                "Component template missing: `{}`.",
                template_path.display()
            ));
        }

        for style_file in &definition.styles {
            let style_path =
                resolve_component_asset_candidate(&assets_root, definition, style_file);
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
    definition: &ComponentDefinition,
    config: &ExtendedFrameworkConfiguration,
) -> Result<String, String> {
    let assets_root =
        PathBuf::from(&config.asset_root_fs_path).join(trim_slashes(&config.assets_component_root));
    let template_path =
        resolve_component_asset_candidate(&assets_root, definition, &definition.template_file);
    fs::read_to_string(&template_path).map_err(|err| {
        format!(
            "Failed to read component html `{}`: {err}",
            template_path.display()
        )
    })
}

/// Handles `trim_slashes` in the extended UI workflow.
fn trim_slashes(path: &str) -> String {
    path.trim_matches('/').trim_matches('\\').to_string()
}

/// Handles `normalize_path_like` in the extended UI workflow.
fn normalize_path_like(path: &str) -> String {
    path.replace('\\', "/").trim_matches('/').to_string()
}

/// Handles `resolve_component_asset_candidate` in the extended UI workflow.
fn resolve_component_asset_candidate(
    root: &Path,
    definition: &ComponentDefinition,
    file_name: &str,
) -> PathBuf {
    let normalized_file_name = normalize_path_like(file_name);
    if normalized_file_name.contains('/') {
        return root.join(normalized_file_name);
    }

    if definition.source_dir_rel.is_empty() {
        return root.join(normalized_file_name);
    }

    let scoped_path = root
        .join(&definition.source_dir_rel)
        .join(&normalized_file_name);
    if scoped_path.exists() {
        return scoped_path;
    }
    root.join(normalized_file_name)
}
