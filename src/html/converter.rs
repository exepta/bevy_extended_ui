use bevy::asset::AssetEvent;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::reflect::serde::TypedReflectDeserializer;
use kuchiki::{Attributes, NodeRef, traits::TendrilSink};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::de::DeserializeSeed;
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
#[cfg(feature = "extended-framework")]
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::ExtendedUiConfiguration;
#[cfg(feature = "extended-dialog")]
use crate::dialog::{DialogProvider, DialogWidget, DialogWidgetType};
#[cfg(feature = "extended-framework")]
use crate::framework::{ExtendedFrameworkConfiguration, compile_framework_template};
use crate::html::{
    HtmlDirty, HtmlEventBindings, HtmlID, HtmlInnerContent, HtmlMeta, HtmlSource, HtmlStates,
    HtmlStructureMap, HtmlStyle, HtmlSystemSet, HtmlWidgetNode,
};
use crate::io::{CssAsset, DefaultCssHandle, HtmlAsset};
use crate::lang::{
    UILang, UiLangState, UiLangVariables, UiSharedValues, localize_html, shared_values_fingerprint,
    vars_fingerprint,
};
#[cfg(feature = "providers")]
use crate::providers::{
    ProviderChildPolicy, ProviderResolveContext, ThemeProviderState, UiProvider, UiProviderRegistry,
};
use crate::styles::IconPlace;
use crate::styles::parser::convert_to_color;
use crate::widgets::Button;
use crate::widgets::*;

/// Legacy identifier for the built-in embedded default stylesheet.
pub const DEFAULT_UI_CSS: &str = "embedded/default_style";

static INNER_BINDING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)\{\{\s*([^{}]+?)\s*\}\}").unwrap());
static FOR_HEADER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*([A-Za-z_][A-Za-z0-9_]*)(?:\s*,\s*([A-Za-z_][A-Za-z0-9_]*))?\s+in\s+(.+?)\s*$")
        .unwrap()
});
static SLIDER_RANGE_VALUE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+))\s*-\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+))\s*$")
        .unwrap()
});

/// Plugin that parses HTML assets into widget trees.
pub struct HtmlConverterSystem;

impl Plugin for HtmlConverterSystem {
    /// Registers the HTML conversion system.
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_html_ui.in_set(HtmlSystemSet::Convert));
    }
}

/// Marker component used for HtmlSource entities whose HtmlAsset wasn't ready yet.
/// This enables robust re-try parsing on later frames (important for WASM async loading).
#[derive(Component, Debug, Default)]
/// Marker component for HtmlSource entities that need a retry parse.
struct PendingHtmlParse;

/// Bundled query params for `update_html_ui`, keeping total system-param count ≤ 16.
#[derive(SystemParam)]
struct HtmlSourceQueries<'w, 's> {
    added: Query<'w, 's, (Entity, &'static HtmlSource), Added<HtmlSource>>,
    pending: Query<'w, 's, Entity, With<PendingHtmlParse>>,
    all: Query<'w, 's, (Entity, &'static HtmlSource)>,
}

/// Bundled template data inputs for converter systems.
#[derive(SystemParam)]
struct TemplateInputs<'w> {
    lang_vars: Res<'w, UiLangVariables>,
    shared_values: Res<'w, UiSharedValues>,
}

/// Converts HtmlAsset content into HtmlStructureMap entries.
/// Also resolves <link rel="stylesheet" href="..."> into Handle<CssAsset>.
/// Updates parsed HTML structures from assets and language state.
fn update_html_ui(
    mut commands: Commands,
    mut structure_map: ResMut<HtmlStructureMap>,
    mut html_dirty: ResMut<HtmlDirty>,
    mut ui_lang: ResMut<UILang>,
    mut lang_state: ResMut<UiLangState>,

    template_inputs: TemplateInputs,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    html_assets: Res<Assets<HtmlAsset>>,
    mut html_asset_events: MessageReader<AssetEvent<HtmlAsset>>,
    default_css: Res<DefaultCssHandle>,
    #[cfg(feature = "extended-framework")] framework_config: Option<
        Res<ExtendedFrameworkConfiguration>,
    >,
    #[cfg(feature = "providers")] provider_registry: Option<Res<UiProviderRegistry>>,
    #[cfg(feature = "providers")] mut theme_provider_state: Option<ResMut<ThemeProviderState>>,

    type_registry: Res<AppTypeRegistry>,

    queries: HtmlSourceQueries,
) {
    let type_registry = type_registry.read();
    #[cfg(feature = "extended-framework")]
    let mut framework_config = framework_config.as_deref().cloned().unwrap_or_default();
    #[cfg(feature = "extended-framework")]
    {
        framework_config.assets_component_root = config.framework_components_path.clone();

        // Keep explicit custom roots untouched. If still on the legacy default,
        // derive the rust component root from the configured assets component path.
        if framework_config.rust_component_root == "src/packages" {
            framework_config.rust_component_root =
                PathBuf::from(&framework_config.asset_root_fs_path)
                    .join(trim_path_separators(&config.framework_components_path))
                    .to_string_lossy()
                    .to_string();
        }
    }
    let resolved = ui_lang.resolved().map(|lang| lang.to_string());
    let mut lang_dirty = false;
    let vars_hash = vars_fingerprint(&template_inputs.lang_vars);

    if lang_state.last_resolved != resolved {
        lang_state.last_resolved = resolved;
        lang_dirty = true;
    }

    if lang_state.last_language_path.as_deref() != Some(config.language_path.as_str()) {
        lang_state
            .last_language_path
            .replace(config.language_path.clone());
        lang_dirty = true;
    }

    if lang_state.last_vars_fingerprint != Some(vars_hash) {
        lang_state.last_vars_fingerprint = Some(vars_hash);
        lang_dirty = true;
    }

    let shared_hash = shared_values_fingerprint(&template_inputs.shared_values);
    if lang_state.last_shared_fingerprint != Some(shared_hash) {
        lang_state.last_shared_fingerprint = Some(shared_hash);
        lang_dirty = true;
    }

    // Entities that need reparse (new HtmlSource, pending retry, or changed HtmlAsset).
    let mut dirty_entities: Vec<Entity> = queries.added.iter().map(|(e, _)| e).collect();
    dirty_entities.extend(queries.pending.iter());

    // If an HtmlAsset changed OR was added later (async), find all entities referencing it.
    for ev in html_asset_events.read() {
        let id = match ev {
            AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                *id
            }
            _ => continue,
        };

        for (entity, src) in queries.all.iter() {
            if src.handle.id() == id {
                dirty_entities.push(entity);
            }
        }
    }

    dirty_entities.sort();
    dirty_entities.dedup();

    if lang_dirty {
        dirty_entities.extend(queries.all.iter().map(|(e, _)| e));
        dirty_entities.sort();
        dirty_entities.dedup();
    }

    if dirty_entities.is_empty() {
        return;
    }

    for entity in dirty_entities {
        let Ok((_entity, html)) = queries.all.get(entity) else {
            continue;
        };

        let Some(html_asset) = html_assets.get(&html.handle) else {
            // Asset isn't ready yet (common on WASM). Mark entity for retry.
            commands.entity(entity).insert(PendingHtmlParse);
            continue;
        };

        // Asset is ready -> ensure we don't keep a retry flag.
        commands.entity(entity).remove::<PendingHtmlParse>();

        let source_path = html.get_source_path();
        let raw_content = html_asset.html.clone();
        #[cfg(feature = "extended-framework")]
        let framework_compiled =
            compile_framework_template(&raw_content, &source_path, &framework_config);
        #[cfg(feature = "extended-framework")]
        let content = framework_compiled.html.clone();
        #[cfg(not(feature = "extended-framework"))]
        let content = raw_content;

        let raw_document = kuchiki::parse_html().one(content.clone());
        let html_lang = raw_document
            .select_first("html")
            .ok()
            .and_then(|node| node.attributes.borrow().get("lang").map(|s| s.to_string()));

        ui_lang.apply_html_lang(html_lang.as_deref());
        lang_state.last_resolved = ui_lang.resolved().map(|lang| lang.to_string());

        let localized = localize_html(
            &content,
            ui_lang.resolved(),
            &config.language_path,
            &template_inputs.lang_vars,
        );
        #[cfg(feature = "extended-framework")]
        let component_local_type_names = {
            let mut names = component_local_type_names(&source_path, &framework_config);
            names.extend(component_local_type_names_from_rust_paths(
                &framework_compiled.component_controllers,
            ));
            names.sort();
            names.dedup();
            names
        };
        #[cfg(not(feature = "extended-framework"))]
        let component_local_type_names = Vec::<String>::new();

        let template_resolved = preprocess_template_directives_with_shared_and_local_types(
            &localized,
            &template_inputs.lang_vars,
            &template_inputs.shared_values,
            &component_local_type_names,
        );
        let document = if template_resolved == content {
            raw_document
        } else {
            kuchiki::parse_html().one(template_resolved.clone())
        };

        #[cfg(feature = "providers")]
        if let Some(provider_registry) = provider_registry.as_ref() {
            unwrap_provider_nodes(&document, provider_registry);
        }

        // Extract unique UI key from <meta name="...">
        let Some(meta_key) = document
            .select_first("head meta[name]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("name").map(|s| s.to_string()))
        else {
            error!("Missing <meta name=...> tag in <head>");
            continue;
        };

        let ui_key = if html.source_id.is_empty() {
            meta_key.clone()
        } else {
            html.source_id.clone()
        };

        if ui_key != meta_key {
            debug!(
                "Using registry key '{}' instead of meta name '{}'",
                ui_key, meta_key
            );
        }

        // Optional controller path from <meta controller="...">
        let meta_controller = document
            .select_first("head meta[controller]")
            .ok()
            .and_then(|m| {
                m.attributes
                    .borrow()
                    .get("controller")
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();

        // Load all CSS handles from <link rel="stylesheet" href="...">
        let mut css_handles: Vec<Handle<CssAsset>> = document
            .select("head link[href]")
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|node| {
                let attrs = node.attributes.borrow();

                let rel = attrs.get("rel")?.to_string();
                if rel != "stylesheet" {
                    return None;
                }

                let raw_href = attrs.get("href")?.to_string();
                drop(attrs);

                // Resolve href relative to the HTML file location inside assets/
                let resolved = resolve_relative_asset_path(&source_path, &raw_href);

                Some(asset_server.load::<CssAsset>(resolved))
            })
            .collect();

        css_handles = with_default_css_first(&default_css, css_handles);
        #[cfg(feature = "providers")]
        if let Some(provider_registry) = provider_registry.as_ref() {
            let mut provider_css = resolve_provider_css_handles(
                &template_resolved,
                provider_registry,
                html,
                &asset_server,
                theme_provider_state.as_deref_mut(),
            );
            css_handles.append(&mut provider_css);
            css_handles = dedup_css_handles(css_handles);
        }

        // Parse body. If multiple <body> nodes exist (invalid HTML), prefer the one
        // with the most descendants so provider-wrapped templates still resolve.
        let body_node = select_primary_body_node(&document);
        let Some(body_node) = body_node else {
            error!("Missing <body> tag!");
            continue;
        };

        let mut effective_controller = html.controller.clone();
        if effective_controller.is_none() && !meta_controller.is_empty() {
            effective_controller = Some(meta_controller.clone());
        }
        #[cfg(feature = "extended-framework")]
        if effective_controller.is_none() {
            effective_controller = framework_compiled.inferred_controller.clone();
        }

        let parse_source = HtmlSource {
            controller: effective_controller,
            ..html.clone()
        };

        debug!("Create UI for HTML with key [{:?}]", ui_key);
        if let Some(controller) = parse_source.controller.as_ref() {
            debug!("UI controller [{:?}]", controller);
        }

        let label_map = collect_labels_by_for(&body_node);

        if let Some(body_widget) = parse_html_node(
            &body_node,
            &css_handles,
            &label_map,
            &ui_key,
            &parse_source,
            &type_registry,
            "0",
        ) {
            structure_map
                .html_map
                .insert(ui_key.clone(), vec![body_widget]);
            #[cfg(feature = "extended-framework")]
            {
                let active = structure_map.active.get_or_insert_with(Vec::new);
                if !active.iter().any(|existing| existing == &ui_key) {
                    active.push(ui_key.clone());
                }
            }

            // IMPORTANT: Explicitly mark the affected UI key as dirty so the builder rebuilds.
            html_dirty.0 = true;
            html_dirty.1.insert(ui_key);
        } else {
            error!("Failed to parse <body> node.");
        }
    }
}

/// Handles `trim_path_separators` in the extended UI workflow.
#[cfg(feature = "extended-framework")]
fn trim_path_separators(path: &str) -> String {
    path.trim_matches('/').trim_matches('\\').to_string()
}

/// Builds a deterministic HtmlID from logical position in the parsed HTML tree.
fn stable_html_id(key: &str, path: &str, tag: &str) -> HtmlID {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in key
        .as_bytes()
        .iter()
        .chain(path.as_bytes().iter())
        .chain(tag.as_bytes().iter())
    {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    HtmlID(hash as usize)
}

/// Parses a DOM node into HtmlWidgetNode.
fn parse_html_node(
    node: &NodeRef,
    css_sources: &Vec<Handle<CssAsset>>,
    label_map: &HashMap<String, String>,
    key: &String,
    html: &HtmlSource,
    type_registry: &TypeRegistry,
    path: &str,
) -> Option<HtmlWidgetNode> {
    let element = node.as_element()?;
    let tag = element.name.local.to_string();
    let attributes = element.attributes.borrow();

    let meta = HtmlMeta {
        css: css_sources.clone(),
        id: attributes.get("id").map(|s| s.to_string()),
        class: attributes
            .get("class")
            .map(|s| s.split_whitespace().map(str::to_string).collect()),
        style: attributes.get("style").map(HtmlStyle::from_str),
        validation: parse_validation_attributes(&attributes),
        inner_content: parse_inner_content(node),
    };

    let states = HtmlStates {
        disabled: attributes.contains("disabled"),
        readonly: attributes.contains("readonly"),
        hidden: attributes.contains("hidden"),
    };

    let functions = bind_html_func(&attributes);

    let widget = Widget(html.controller.clone());
    let node_id = stable_html_id(key, path, &tag);

    match tag.as_str() {
        "body" => {
            let children = node
                .children()
                .enumerate()
                .filter_map(|(index, child)| {
                    let child_path = format!("{path}.{index}");
                    parse_html_node(
                        &child,
                        css_sources,
                        label_map,
                        key,
                        html,
                        type_registry,
                        child_path.as_str(),
                    )
                })
                .collect();

            Some(HtmlWidgetNode::Body(
                Body {
                    html_key: Some(key.clone()),
                    ..default()
                },
                meta,
                states,
                children,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "button" => {
            let (icon_path, icon_place) = parse_icon_and_text(node);
            let text = node.text_contents().trim().to_string();
            let button_type = attributes
                .get("type")
                .and_then(ButtonType::from_str)
                .unwrap_or_default();
            Some(HtmlWidgetNode::Button(
                Button {
                    text,
                    icon_path,
                    icon_place,
                    button_type,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "checkbox" => {
            let label = node.text_contents().trim().to_string();
            let icon_path = attributes.get("icon");
            let icon = icon_path.map(String::from);
            Some(HtmlWidgetNode::CheckBox(
                CheckBox {
                    label,
                    icon_path: icon,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "colorpicker" => {
            let value = attributes
                .get("value")
                .and_then(|v| convert_to_color(v.to_string()))
                .unwrap_or_else(|| Color::srgb_u8(0x42, 0x85, 0xF4));

            let srgba = value.to_srgba();
            let mut alpha = (srgba.alpha * 255.0).round() as u8;

            if let Some(alpha_attr) = attributes.get("alpha") {
                let parsed = alpha_attr.trim().parse::<f32>().ok().map(|value| {
                    if value <= 1.0 {
                        (value * 255.0).round().clamp(0.0, 255.0) as u8
                    } else {
                        value.round().clamp(0.0, 255.0) as u8
                    }
                });
                if let Some(parsed) = parsed {
                    alpha = parsed;
                }
            }

            Some(HtmlWidgetNode::ColorPicker(
                ColorPicker::from_rgba_u8(
                    (srgba.red * 255.0).round() as u8,
                    (srgba.green * 255.0).round() as u8,
                    (srgba.blue * 255.0).round() as u8,
                    alpha,
                ),
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "div" => {
            let mut children = Vec::new();
            for (index, child) in node.children().enumerate() {
                let child_path = format!("{path}.{index}");
                if let Some(parsed) = parse_html_node(
                    &child,
                    css_sources,
                    label_map,
                    key,
                    html,
                    type_registry,
                    child_path.as_str(),
                ) {
                    children.push(parsed);
                }
            }

            Some(HtmlWidgetNode::Div(
                Div::default(),
                meta,
                states,
                children,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "form" => {
            let mut children = Vec::new();
            for (index, child) in node.children().enumerate() {
                let child_path = format!("{path}.{index}");
                if let Some(parsed) = parse_html_node(
                    &child,
                    css_sources,
                    label_map,
                    key,
                    html,
                    type_registry,
                    child_path.as_str(),
                ) {
                    children.push(parsed);
                }
            }

            let action = attributes
                .get("action")
                .or_else(|| attributes.get("onsubmit"))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let validate_mode = attributes
                .get("validate")
                .and_then(FormValidationMode::from_str)
                .unwrap_or_default();

            Some(HtmlWidgetNode::Form(
                Form {
                    action,
                    validate_mode,
                    ..default()
                },
                meta,
                states,
                children,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        #[cfg(feature = "extended-dialog")]
        "dialog" => {
            let mut children = Vec::new();
            for (index, child) in node.children().enumerate() {
                let child_path = format!("{path}.{index}");
                if let Some(parsed) = parse_html_node(
                    &child,
                    css_sources,
                    label_map,
                    key,
                    html,
                    type_registry,
                    child_path.as_str(),
                ) {
                    children.push(parsed);
                }
            }

            let trigger = attributes
                .get("trigger")
                .or_else(|| attributes.get("triggger"))
                .map(|raw| raw.trim().trim_start_matches('#').to_string())
                .filter(|value| !value.is_empty());
            let renderer = attributes
                .get("renderer")
                .and_then(DialogProvider::from_attr)
                .unwrap_or(DialogProvider::BevyApp);
            let dialog_type = attributes
                .get("type")
                .and_then(DialogWidgetType::from_attr)
                .unwrap_or(DialogWidgetType::Info);
            let content_text = node
                .text_contents()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            Some(HtmlWidgetNode::Dialog(
                DialogWidget {
                    trigger,
                    renderer,
                    dialog_type,
                    content_text,
                    open: false,
                },
                meta,
                states,
                children,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        #[cfg(feature = "extended-dialog")]
        "dialog-header" => {
            let mut children = Vec::new();
            for (index, child) in node.children().enumerate() {
                let child_path = format!("{path}.{index}");
                if let Some(parsed) = parse_html_node(
                    &child,
                    css_sources,
                    label_map,
                    key,
                    html,
                    type_registry,
                    child_path.as_str(),
                ) {
                    children.push(parsed);
                }
            }

            let mut meta = meta;
            ensure_meta_class(&mut meta, "dialog-header");

            Some(HtmlWidgetNode::Div(
                Div::default(),
                meta,
                states,
                children,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        #[cfg(feature = "extended-dialog")]
        "dialog-body" => {
            let mut children = Vec::new();
            for (index, child) in node.children().enumerate() {
                let child_path = format!("{path}.{index}");
                if let Some(parsed) = parse_html_node(
                    &child,
                    css_sources,
                    label_map,
                    key,
                    html,
                    type_registry,
                    child_path.as_str(),
                ) {
                    children.push(parsed);
                }
            }

            let mut meta = meta;
            ensure_meta_class(&mut meta, "dialog-body");

            Some(HtmlWidgetNode::Div(
                Div::default(),
                meta,
                states,
                children,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        #[cfg(feature = "extended-dialog")]
        "dialog-footer" => {
            let mut children = Vec::new();
            for (index, child) in node.children().enumerate() {
                let child_path = format!("{path}.{index}");
                if let Some(parsed) = parse_html_node(
                    &child,
                    css_sources,
                    label_map,
                    key,
                    html,
                    type_registry,
                    child_path.as_str(),
                ) {
                    children.push(parsed);
                }
            }

            let mut meta = meta;
            ensure_meta_class(&mut meta, "dialog-footer");

            Some(HtmlWidgetNode::Div(
                Div::default(),
                meta,
                states,
                children,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "divider" => {
            let alignment = attributes.get("alignment").unwrap_or("horizontal");
            Some(HtmlWidgetNode::Divider(
                Divider {
                    alignment: DividerAlignment::from_str(alignment).unwrap_or_default(),
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "fieldset" => {
            let allow_none =
                attributes.get("allow-none").map(|s| s.to_string()) == Some("true".to_string());
            let mode = attributes.get("mode").unwrap_or("single").to_string();
            let mut children = Vec::new();

            let mut radio_nodes: Vec<(usize, NodeRef, bool)> = Vec::new();
            for (index, child) in node.children().enumerate() {
                if let Some(el) = child.as_element() {
                    if el.name.local.eq("radio") {
                        let selected_attr = el.attributes.borrow().contains("selected");
                        radio_nodes.push((index, child.clone(), selected_attr));
                        continue;
                    }
                }
                let child_path = format!("{path}.{index}");
                if let Some(parsed) = parse_html_node(
                    &child,
                    css_sources,
                    label_map,
                    key,
                    html,
                    type_registry,
                    child_path.as_str(),
                ) {
                    children.push(parsed);
                }
            }

            let any_selected_attr = radio_nodes.iter().any(|(_, _, sel)| *sel);
            let mut selected_used = false;
            let mut first_radio_seen = false;

            for (radio_index, radio_node, had_selected_attr) in radio_nodes {
                let element = radio_node.as_element().unwrap();
                let attrs = element.attributes.borrow();

                let value_str = attrs.get("value").unwrap_or("").to_string();
                let value_type = attrs
                    .get("internal-value-type")
                    .unwrap_or("")
                    .trim()
                    .to_ascii_lowercase();
                let value = parse_option_internal_value(&value_str, &value_type, type_registry);
                let label = radio_node.text_contents().trim().to_string();

                let selected = if any_selected_attr {
                    if had_selected_attr && !selected_used {
                        selected_used = true;
                        true
                    } else {
                        false
                    }
                } else if !selected_used && !allow_none && !first_radio_seen {
                    selected_used = true;
                    true
                } else {
                    false
                };

                first_radio_seen = true;

                let child_meta = HtmlMeta {
                    css: meta.css.clone(),
                    id: attrs.get("id").map(|s| s.to_string()),
                    class: attrs
                        .get("class")
                        .map(|s| s.split_whitespace().map(str::to_string).collect()),
                    style: attrs.get("style").map(HtmlStyle::from_str),
                    validation: parse_validation_attributes(&attrs),
                    inner_content: parse_inner_content(&radio_node),
                };

                let child_states = HtmlStates {
                    disabled: attrs.contains("disabled"),
                    readonly: attrs.contains("readonly"),
                    hidden: attrs.contains("hidden"),
                };

                let child_functions = bind_html_func(&attrs);

                children.push(HtmlWidgetNode::RadioButton(
                    RadioButton {
                        label,
                        value,
                        selected,
                        ..default()
                    },
                    child_meta,
                    child_states,
                    child_functions,
                    widget.clone(),
                    stable_html_id(key, &format!("{path}.{radio_index}"), "radio"),
                ));
            }

            Some(HtmlWidgetNode::FieldSet(
                FieldSet {
                    allow_none,
                    field_mode: FieldMode::from_str(mode.as_str()).unwrap_or(FieldMode::Single),
                    ..default()
                },
                meta,
                states,
                children,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let h_type = match tag.as_str() {
                "h1" => HeadlineType::H1,
                "h2" => HeadlineType::H2,
                "h3" => HeadlineType::H3,
                "h4" => HeadlineType::H4,
                "h5" => HeadlineType::H5,
                "h6" => HeadlineType::H6,
                _ => HeadlineType::H3,
            };

            let text = node.text_contents().trim().to_string();

            Some(HtmlWidgetNode::Headline(
                Headline {
                    text,
                    h_type,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "a" => {
            let text = node.text_contents().trim().to_string();
            let href = attributes
                .get("href")
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            let open_modal = parse_bool_attribute(&attributes, "open-modal");
            let browsers = match attributes.get("browsers") {
                Some(raw) => HyperLinkBrowsers::from_str(raw).unwrap_or_else(|| {
                    warn!("Invalid hyperlink browsers attribute; falling back to system.");
                    HyperLinkBrowsers::System
                }),
                None => HyperLinkBrowsers::System,
            };

            Some(HtmlWidgetNode::HyperLink(
                HyperLink {
                    text,
                    href,
                    browsers,
                    open_modal,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "img" => {
            let src_raw = attributes.get("src").unwrap_or("").to_string();
            let alt = attributes.get("alt").unwrap_or("").to_string();
            let preview = attributes
                .get("preview")
                .map(str::trim)
                .map(|value| value.trim_start_matches('#').to_string())
                .filter(|value| !value.is_empty());

            let src_resolved = if src_raw.trim().is_empty() {
                None
            } else {
                Some(resolve_relative_asset_path(
                    &html.get_source_path(),
                    &src_raw,
                ))
            };

            Some(HtmlWidgetNode::Img(
                Img {
                    src: src_resolved,
                    alt,
                    preview,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "input" => {
            let id = attributes.get("id").map(|s| s.to_string());
            let name = attributes
                .get("name")
                .map(str::to_string)
                .or_else(|| id.clone())
                .unwrap_or_default();
            let label = id
                .as_ref()
                .and_then(|id| label_map.get(id))
                .cloned()
                .unwrap_or_default();

            let text = attributes.get("value").unwrap_or("").to_string();
            let placeholder = attributes.get("placeholder").unwrap_or("").to_string();
            let input_type = attributes.get("type").unwrap_or("text").to_string();
            let date_format = attributes
                .get("format")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let folder = parse_bool_attribute(&attributes, "folder");
            let extensions = parse_extensions_attribute(attributes.get("extensions"));
            let show_size = parse_bool_attribute(&attributes, "show-size");
            let max_size_bytes = parse_max_size_attribute(attributes.get("max-size"));
            let icon_path = attributes.get("icon").unwrap_or("");
            let icon: Option<String> = if !icon_path.is_empty() {
                Some(String::from(icon_path))
            } else {
                None
            };

            let cap = match attributes.get("maxlength") {
                Some(value) if value.trim().eq_ignore_ascii_case("auto") => InputCap::CapAtNodeSize,
                Some(value) if value.trim().is_empty() => InputCap::NoCap,
                Some(value) => {
                    let length = value.trim().parse::<usize>().unwrap_or(0);
                    InputCap::CapAt(length)
                }
                None => InputCap::NoCap,
            };

            Some(HtmlWidgetNode::Input(
                InputField {
                    name,
                    label,
                    placeholder,
                    text,
                    input_type: InputType::from_str(&input_type).unwrap_or_default(),
                    date_format,
                    folder,
                    extensions,
                    show_size,
                    max_size_bytes,
                    icon_path: icon,
                    cap_text_at: cap,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "date-picker" => {
            let for_id = parse_for_attribute_id(&attributes);
            let id = attributes.get("id").map(|s| s.to_string());
            let name = attributes
                .get("name")
                .map(str::to_string)
                .or_else(|| id.clone())
                .unwrap_or_default();
            let label = attributes
                .get("label")
                .map(str::to_string)
                .or_else(|| id.as_ref().and_then(|id| label_map.get(id)).cloned())
                .unwrap_or_else(|| "Date".to_string());
            let placeholder = attributes.get("placeholder").unwrap_or("").to_string();
            let value = attributes.get("value").unwrap_or("").to_string();
            let min = attributes.get("min").map(str::to_string);
            let max = attributes.get("max").map(str::to_string);
            let format_pattern = attributes
                .get("format")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let format = format_pattern
                .as_deref()
                .and_then(DateFormat::from_str)
                .unwrap_or_default();
            let mut meta = meta;
            if for_id.is_some() {
                let classes = meta.class.get_or_insert_with(Vec::new);
                if !classes.iter().any(|class| class == "date-picker-bound") {
                    classes.push("date-picker-bound".to_string());
                }
            }

            Some(HtmlWidgetNode::DatePicker(
                DatePicker {
                    for_id,
                    name,
                    label,
                    placeholder,
                    value,
                    min,
                    max,
                    format_pattern,
                    format,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "p" => {
            let text = node.text_contents().trim().to_string();
            Some(HtmlWidgetNode::Paragraph(
                Paragraph { text, ..default() },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "tool-tip" => {
            let text = node
                .text_contents()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let for_id = parse_for_attribute_id(&attributes);
            let variant = attributes
                .get("variant")
                .and_then(ToolTipVariant::from_str)
                .unwrap_or_default();
            let prio = attributes
                .get("prio")
                .and_then(ToolTipPriority::from_str)
                .unwrap_or_default();
            let alignment = attributes
                .get("alignment")
                .and_then(ToolTipAlignment::from_str)
                .unwrap_or_default();
            let trigger = attributes
                .get("trigger")
                .map(parse_tooltip_triggers)
                .unwrap_or_else(|| vec![ToolTipTrigger::Hover]);

            Some(HtmlWidgetNode::ToolTip(
                ToolTip {
                    text,
                    for_id,
                    variant,
                    prio,
                    alignment,
                    trigger,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "badge" => {
            let for_id = parse_for_attribute_id(&attributes);
            let max = attributes
                .get("max")
                .and_then(|raw| raw.trim().parse::<u32>().ok())
                .unwrap_or(99);
            let value = attributes
                .get("value")
                .or_else(|| attributes.get("count"))
                .and_then(|raw| raw.trim().parse::<u32>().ok())
                .or_else(|| node.text_contents().trim().parse::<u32>().ok())
                .unwrap_or(0);
            let anchor = attributes
                .get("anchor")
                .and_then(BadgeAnchor::from_str)
                .unwrap_or_default();

            Some(HtmlWidgetNode::Badge(
                Badge {
                    value,
                    max,
                    for_id,
                    anchor,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "progressbar" => {
            let min = attributes
                .get("min")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.0);

            let max = attributes
                .get("max")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(100.0);

            let value = attributes
                .get("value")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(min);

            Some(HtmlWidgetNode::ProgressBar(
                ProgressBar {
                    value,
                    min,
                    max,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "radio" => {
            let value_str = attributes.get("value").unwrap_or("").to_string();
            let value_type = attributes
                .get("internal-value-type")
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            let value = parse_option_internal_value(&value_str, &value_type, type_registry);
            let label = node.text_contents().trim().to_string();
            let selected_attr = attributes.contains("selected");

            Some(HtmlWidgetNode::RadioButton(
                RadioButton {
                    label,
                    value,
                    selected: selected_attr,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "scroll" => {
            let alignment = attributes.get("alignment").unwrap_or("vertical");
            let mut vertical = true;
            if alignment.eq_ignore_ascii_case("horizontal") {
                vertical = false;
            }
            Some(HtmlWidgetNode::Scrollbar(
                Scrollbar {
                    vertical,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "select" => {
            let mut options = Vec::new();
            let mut selected_value = None;

            for child in node.children() {
                if let Some(option_el) = child.as_element() {
                    if option_el.name.local.eq("option") {
                        let attrs = option_el.attributes.borrow();
                        let value = attrs.get("value").unwrap_or("").to_string();
                        let value_type = attrs
                            .get("internal-value-type")
                            .unwrap_or("")
                            .trim()
                            .to_ascii_lowercase();
                        let icon = attrs.get("icon").unwrap_or("").to_string();
                        let text = child.text_contents().trim().to_string();

                        let icon_path = if icon.trim().is_empty() {
                            None
                        } else {
                            Some(icon)
                        };

                        let value = parse_option_internal_value(&value, &value_type, type_registry);

                        let option = ChoiceOption {
                            text: text.clone(),
                            value,
                            icon_path,
                        };

                        if attrs.contains("selected") {
                            selected_value = Some(option.clone());
                        }

                        options.push(option);
                    }
                }
            }

            let value =
                selected_value.unwrap_or_else(|| options.first().cloned().unwrap_or_default());

            Some(HtmlWidgetNode::ChoiceBox(
                ChoiceBox {
                    value,
                    options,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "slider" => {
            let min_raw = attributes
                .get("min")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.0);

            let max_raw = attributes
                .get("max")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(100.0);

            let (min, max) = if max_raw >= min_raw {
                (min_raw, max_raw)
            } else {
                (max_raw, min_raw)
            };

            let step = attributes
                .get("step")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(1.0);

            let slider_type = attributes
                .get("type")
                .and_then(SliderType::from_str)
                .unwrap_or_default();

            let dots = attributes
                .get("dots")
                .and_then(|v| v.trim().parse::<i32>().ok())
                .map(|v| if v <= 1 { 1 } else { v as u32 });

            let show_labels = parse_bool_attribute(&attributes, "show-labels");
            let show_tip = attributes
                .get("tip")
                .map(|value| {
                    let normalized = value.trim().to_ascii_lowercase();
                    normalized.is_empty() || normalized == "true"
                })
                .unwrap_or(true);

            let dot_anchor = attributes
                .get("dot-anchor")
                .and_then(SliderDotAnchor::from_str)
                .unwrap_or_default();

            let (value, range_start, range_end) =
                parse_slider_values(attributes.get("value"), min, max, slider_type);

            Some(HtmlWidgetNode::Slider(
                Slider {
                    slider_type,
                    value,
                    range_start,
                    range_end,
                    min,
                    max,
                    step,
                    dots,
                    show_labels,
                    show_tip,
                    dot_anchor,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "switch" => {
            let text = node.text_contents().trim().to_string();
            let icon_attr = attributes.get("icon").unwrap_or("");
            let icon = if icon_attr.is_empty() {
                None
            } else {
                Some(icon_attr.to_string())
            };
            let selected = if attributes.contains("value") {
                parse_bool_attribute(&attributes, "value")
            } else {
                parse_bool_attribute(&attributes, "checked")
            };

            Some(HtmlWidgetNode::SwitchButton(
                SwitchButton {
                    label: text,
                    icon,
                    selected,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "toggle" => {
            let value = attributes.get("value").unwrap_or("").to_string();
            let (icon_path, icon_place) = parse_icon_and_text(node);
            let text = node.text_contents().trim().to_string();
            let selected_attr = attributes.contains("selected");
            Some(HtmlWidgetNode::ToggleButton(
                ToggleButton {
                    label: text.clone(),
                    icon_path,
                    value: WidgetValue::new(value),
                    icon_place,
                    selected: selected_attr,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        "listbox" => {
            let mut options = Vec::new();
            let mut selected_values = Vec::new();
            let multiselect = attributes.contains("multiselect");

            for child in node.children() {
                if let Some(option_el) = child.as_element() {
                    if option_el.name.local.eq("option") {
                        let attrs = option_el.attributes.borrow();
                        let value = attrs.get("value").unwrap_or("").to_string();
                        let value_type = attrs
                            .get("internal-value-type")
                            .unwrap_or("")
                            .trim()
                            .to_ascii_lowercase();
                        let icon = attrs.get("icon").unwrap_or("").to_string();
                        let text = child.text_contents().trim().to_string();

                        let icon_path = if icon.trim().is_empty() {
                            None
                        } else {
                            Some(icon)
                        };

                        let value = parse_option_internal_value(&value, &value_type, type_registry);

                        let option = ChoiceOption {
                            text: text.clone(),
                            value,
                            icon_path,
                        };

                        if attrs.contains("selected") {
                            selected_values.push(option.clone());
                        }

                        options.push(option);
                    }
                }
            }

            Some(HtmlWidgetNode::ListBox(
                ListBox {
                    options,
                    values: selected_values,
                    multiselect,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                node_id.clone(),
            ))
        }

        _ => None,
    }
}

/// Extracts HTML event bindings from element attributes.
fn bind_html_func(attributes: &Attributes) -> HtmlEventBindings {
    HtmlEventBindings {
        onclick: attributes.get("onclick").map(|s| s.to_string()),
        onmousedown: attributes.get("onmousedown").map(|s| s.to_string()),
        onmouseup: attributes.get("onmouseup").map(|s| s.to_string()),
        onmouseover: attributes
            .get("onmouseover")
            .or_else(|| attributes.get("onmouseenter"))
            .map(|s| s.to_string()),
        onmouseout: attributes
            .get("onmouseout")
            .or_else(|| attributes.get("onmouseleave"))
            .map(|s| s.to_string()),
        onchange: attributes.get("onchange").map(|s| s.to_string()),
        oninit: attributes.get("oninit").map(|s| s.to_string()),
        onfoucs: attributes
            .get("onfoucs")
            .or_else(|| attributes.get("onfocus"))
            .map(|s| s.to_string()),
        onscroll: attributes.get("onscroll").map(|s| s.to_string()),
        onwheel: attributes
            .get("onwheel")
            .or_else(|| attributes.get("onmousewheel"))
            .map(|s| s.to_string()),
        onkeydown: attributes.get("onkeydown").map(|s| s.to_string()),
        onkeyup: attributes.get("onkeyup").map(|s| s.to_string()),
        ondragstart: attributes.get("ondragstart").map(|s| s.to_string()),
        ondrag: attributes.get("ondrag").map(|s| s.to_string()),
        ondragstop: attributes
            .get("ondragstop")
            .or_else(|| attributes.get("ondragend"))
            .map(|s| s.to_string()),
        ontouchstart: attributes.get("ontouchstart").map(|s| s.to_string()),
        ontouchmove: attributes.get("ontouchmove").map(|s| s.to_string()),
        ontouchend: attributes.get("ontouchend").map(|s| s.to_string()),
    }
}

/// Parses slider values from the `value` attribute.
fn parse_slider_values(
    raw_value: Option<&str>,
    min: f32,
    max: f32,
    slider_type: SliderType,
) -> (f32, f32, f32) {
    let single = raw_value
        .and_then(|v| v.trim().parse::<f32>().ok())
        .unwrap_or(min)
        .clamp(min, max);

    if slider_type != SliderType::Range {
        return (single, min, max);
    }

    let (start_raw, end_raw) = match raw_value.and_then(parse_slider_range_attribute) {
        Some((start, end)) => (start, end),
        None => {
            if raw_value.is_some() {
                (min, single)
            } else {
                (min, max)
            }
        }
    };

    let mut start = start_raw.clamp(min, max);
    let mut end = end_raw.clamp(min, max);
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }

    (single, start, end)
}

/// Parses a `start - end` slider range value pair.
fn parse_slider_range_attribute(raw: &str) -> Option<(f32, f32)> {
    let captures = SLIDER_RANGE_VALUE_RE.captures(raw.trim())?;
    let start = captures.get(1)?.as_str().parse::<f32>().ok()?;
    let end = captures.get(2)?.as_str().parse::<f32>().ok()?;
    Some((start, end))
}

/// Parses validation rules from element attributes.
fn parse_validation_attributes(attributes: &Attributes) -> Option<ValidationRules> {
    let mut rules = attributes
        .get("validation")
        .and_then(ValidationRules::from_attribute);

    if attributes.contains("required") {
        let mut merged = rules.unwrap_or_default();
        merged.required = true;
        rules = Some(merged);
    }

    rules
}

/// Parses boolean attributes with `true`/`false` semantics.
fn parse_bool_attribute(attributes: &Attributes, key: &str) -> bool {
    if !attributes.contains(key) {
        return false;
    }

    let Some(value) = attributes.get(key) else {
        // Standalone boolean attribute without explicit value (`checked`, `disabled`, ...)
        return true;
    };

    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return true;
    }

    if matches!(normalized.as_str(), "false" | "0" | "no" | "off") {
        return false;
    }

    matches!(
        normalized.as_str(),
        "true" | "1" | "yes" | "on" | "checked" | "selected"
    )
}

/// Parses `extensions` values from either a single token or list syntax.
fn parse_extensions_attribute(raw: Option<&str>) -> Vec<String> {
    let Some(value) = raw else {
        return Vec::new();
    };

    let value = value.trim();
    if value.is_empty() {
        return Vec::new();
    }

    let inner = value
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(value);

    inner
        .split(',')
        .map(str::trim)
        .map(|token| token.trim_matches('"').trim_matches('\''))
        .map(|token| token.trim_start_matches('.'))
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

/// Parses `max-size` values like `1KB`, `2MB`, `0.5GB` into bytes.
fn parse_max_size_attribute(raw: Option<&str>) -> Option<u64> {
    let value = raw?.trim();
    if value.is_empty() {
        return None;
    }

    let compact = value
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let normalized = compact.to_ascii_uppercase();

    let (number, multiplier) = if let Some(number) = normalized.strip_suffix("KB") {
        (number, 1024_f64)
    } else if let Some(number) = normalized.strip_suffix("MB") {
        (number, 1024_f64 * 1024_f64)
    } else if let Some(number) = normalized.strip_suffix("GB") {
        (number, 1024_f64 * 1024_f64 * 1024_f64)
    } else {
        return None;
    };

    let amount = number.parse::<f64>().ok()?;
    if !amount.is_finite() || amount < 0.0 {
        return None;
    }

    Some((amount * multiplier).round() as u64)
}

/// Handles `ensure_meta_class` in the extended UI workflow.
#[cfg(feature = "extended-dialog")]
fn ensure_meta_class(meta: &mut HtmlMeta, class_name: &str) {
    let classes = meta.class.get_or_insert_with(Vec::new);
    if !classes.iter().any(|class| class == class_name) {
        classes.push(class_name.to_string());
    }
}

/// Parses a `for` attribute into a normalized optional id (`#foo` => `foo`).
fn parse_for_attribute_id(attributes: &Attributes) -> Option<String> {
    attributes
        .get("for")
        .map(str::trim)
        .map(|value| value.trim_start_matches('#').to_string())
        .filter(|value| !value.is_empty())
}

/// Collects mappings from <label for="..."> to its label text.
fn collect_labels_by_for(node: &NodeRef) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for label_node in node.select("label").unwrap() {
        let element = label_node.as_node().as_element().unwrap();
        if let Some(for_id) = element.attributes.borrow().get("for") {
            let label_text = label_node.text_contents().trim().to_string();
            map.insert(for_id.to_string(), label_text);
        }
    }

    map
}

/// Chooses the most content-rich `<body>` node from a document.
fn select_primary_body_node(document: &NodeRef) -> Option<NodeRef> {
    let mut best: Option<NodeRef> = None;
    let mut best_score = 0usize;
    let mut count = 0usize;

    let selected = document.select("body").ok()?;
    for body in selected {
        count += 1;
        let node = body.as_node().clone();
        let score = node.descendants().count();
        if score >= best_score {
            best_score = score;
            best = Some(node);
        }
    }

    if count > 1 {
        warn!(
            "HTML contains {} <body> tags. Only one body is supported; using the most content-rich one.",
            count
        );
    }

    best
}

/// Resolves a CSS href found inside an HTML document to a path that the AssetServer understands.
pub fn resolve_relative_asset_path(html_path: &str, href: &str) -> String {
    let mut href = href.replace('\\', "/");

    if let Some(rest) = href.strip_prefix("assets/") {
        href = rest.to_string();
    }

    let path = std::path::Path::new(&href);
    if path.is_absolute() && (path.exists() || path.parent().is_some_and(std::path::Path::exists)) {
        return href;
    }

    if let Some(rest) = href.strip_prefix('/') {
        return rest.to_string();
    }

    if href.starts_with("./") || href.starts_with("../") {
        return href;
    }

    let base = std::path::Path::new(html_path)
        .parent()
        .unwrap_or(std::path::Path::new(""));

    base.join(href).to_string_lossy().replace('\\', "/")
}

/// Ensures the default CSS handle is the first in the list.
fn with_default_css_first(
    default_css: &DefaultCssHandle,
    mut css: Vec<Handle<CssAsset>>,
) -> Vec<Handle<CssAsset>> {
    let default_handle = default_css.0.clone();

    // Remove default if it already exists somewhere
    css.retain(|h| h.id() != default_handle.id());

    // Insert default at position 0
    css.insert(0, default_handle);

    css
}

/// Handles `dedup_css_handles` in the extended UI workflow.
#[cfg(feature = "providers")]
fn dedup_css_handles(css: Vec<Handle<CssAsset>>) -> Vec<Handle<CssAsset>> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(css.len());

    for handle in css {
        if seen.insert(handle.id()) {
            out.push(handle);
        }
    }

    out
}

/// Handles `resolve_provider_css_handles` in the extended UI workflow.
#[cfg(feature = "providers")]
fn resolve_provider_css_handles(
    html_content: &str,
    provider_registry: &UiProviderRegistry,
    html: &HtmlSource,
    asset_server: &AssetServer,
    mut theme_provider_state: Option<&mut ThemeProviderState>,
) -> Vec<Handle<CssAsset>> {
    let mut handles = Vec::new();
    let source_path = html.get_source_path();

    for provider in provider_registry.iter() {
        let matches = collect_provider_matches(html_content, provider.tag());

        for provider_match in matches {
            let mut theme_known: Option<HashSet<String>> = None;
            let mut theme_asset_dir: Option<String> = None;
            let mut theme_fallback: Option<String> = None;
            let mut theme_active: Option<String> = None;

            if provider.tag().eq_ignore_ascii_case("theme-provider") {
                if let Some(default_theme) = provider_match.attributes.get("default") {
                    if let Some(state) = theme_provider_state.as_deref_mut() {
                        state.set_default_theme(default_theme);
                    }
                }

                if let Some(state) = theme_provider_state.as_deref() {
                    theme_known = Some(state.known_themes());
                    theme_asset_dir = Some(state.themes_asset_dir().to_string());
                    theme_fallback = state.default_theme().map(str::to_string);
                    theme_active = state.active_theme().map(str::to_string);
                }
            }

            let direct_children = extract_direct_child_tags(&provider_match.inner_html);
            if let Err(reason) =
                validate_provider_rules(provider.as_ref(), &direct_children, provider_match.in_head)
            {
                warn!(
                    "Provider <{}> ignored due to rule mismatch: {}",
                    provider.tag(),
                    reason
                );
                continue;
            }

            let mut ctx =
                ProviderResolveContext::new(&provider_match.attributes, source_path.as_str());
            ctx = ctx.with_theme_scope(
                theme_asset_dir.as_deref(),
                theme_known.as_ref(),
                theme_fallback.as_deref(),
                theme_active.as_deref(),
            );
            match provider.resolve(ctx) {
                Ok(effect) => {
                    for css_path in effect.extra_css_paths {
                        let resolved = resolve_relative_asset_path(source_path.as_str(), &css_path);
                        handles.push(asset_server.load::<CssAsset>(resolved));
                    }
                }
                Err(error) => {
                    warn!(
                        "Provider <{}> failed to resolve effects: {}",
                        provider.tag(),
                        error
                    );
                }
            }
        }
    }

    handles
}

/// Handles `validate_provider_rules` in the extended UI workflow.
#[cfg(feature = "providers")]
fn validate_provider_rules(
    provider: &dyn UiProvider,
    child_tags: &[String],
    in_head: bool,
) -> Result<(), String> {
    let rules = provider.rules();

    if in_head && !rules.allow_in_head {
        return Err("<head> placement is not allowed".to_string());
    }

    if rules.requires_body_child
        && !child_tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case("body"))
    {
        return Err("expected a direct <body> child".to_string());
    }

    match rules.child_policy {
        ProviderChildPolicy::Any => {}
        ProviderChildPolicy::Only(allowed) => {
            for child in child_tags {
                if !allowed.iter().any(|tag| tag.eq_ignore_ascii_case(&child)) {
                    return Err(format!("child <{}> is not allowed", child));
                }
            }
        }
    }

    Ok(())
}

/// Represents the `ProviderMatch` data structure used by the extended UI system.
#[cfg(feature = "providers")]
#[derive(Debug)]
struct ProviderMatch {
    attributes: HashMap<String, String>,
    inner_html: String,
    in_head: bool,
}

/// Handles `collect_provider_matches` in the extended UI workflow.
#[cfg(feature = "providers")]
fn collect_provider_matches(html_content: &str, tag: &str) -> Vec<ProviderMatch> {
    let head_ranges = collect_head_ranges(html_content);
    let pattern = format!(
        r"(?is)<\s*{}\b(?P<attrs>[^>]*)>(?P<inner>.*?)</\s*{}\s*>",
        regex::escape(tag),
        regex::escape(tag)
    );
    let Ok(regex) = Regex::new(&pattern) else {
        return Vec::new();
    };

    regex
        .captures_iter(html_content)
        .map(|captures| {
            let start = captures.get(0).map_or(0, |m| m.start());
            let in_head = head_ranges
                .iter()
                .any(|(range_start, range_end)| start >= *range_start && start < *range_end);

            ProviderMatch {
                attributes: parse_provider_attributes(
                    captures.name("attrs").map_or("", |m| m.as_str()),
                ),
                inner_html: captures
                    .name("inner")
                    .map_or_else(String::new, |m| m.as_str().to_string()),
                in_head,
            }
        })
        .collect()
}

/// Handles `collect_head_ranges` in the extended UI workflow.
#[cfg(feature = "providers")]
fn collect_head_ranges(html_content: &str) -> Vec<(usize, usize)> {
    let Ok(head_regex) = Regex::new(r"(?is)<\s*head\b[^>]*>.*?</\s*head\s*>") else {
        return Vec::new();
    };

    head_regex
        .captures_iter(html_content)
        .filter_map(|captures| captures.get(0).map(|m| (m.start(), m.end())))
        .collect()
}

/// Handles `unwrap_provider_nodes` in the extended UI workflow.
#[cfg(feature = "providers")]
fn unwrap_provider_nodes(document: &NodeRef, provider_registry: &UiProviderRegistry) {
    for provider in provider_registry.iter() {
        let selector = provider.tag();
        let provider_nodes: Vec<NodeRef> = document
            .select(selector)
            .ok()
            .into_iter()
            .flatten()
            .map(|node| node.as_node().clone())
            .collect();

        for provider_node in provider_nodes {
            if provider_node.parent().is_none() {
                continue;
            }

            let children: Vec<NodeRef> = provider_node.children().collect();
            for child in children {
                provider_node.insert_before(child);
            }
            provider_node.detach();
        }
    }
}

/// Handles `parse_provider_attributes` in the extended UI workflow.
#[cfg(feature = "providers")]
fn parse_provider_attributes(raw_attrs: &str) -> HashMap<String, String> {
    let Ok(regex) =
        Regex::new(r#"([A-Za-z_:][A-Za-z0-9_:\-\.]*)\s*=\s*("([^"]*)"|'([^']*)'|([^\s"'=<>`]+))"#)
    else {
        return HashMap::new();
    };

    let mut out = HashMap::new();
    for capture in regex.captures_iter(raw_attrs) {
        let Some(key) = capture.get(1).map(|m| m.as_str().to_ascii_lowercase()) else {
            continue;
        };
        let value = capture
            .get(3)
            .or_else(|| capture.get(4))
            .or_else(|| capture.get(5))
            .map_or("", |m| m.as_str());
        out.insert(key, value.to_string());
    }

    out
}

/// Handles `extract_direct_child_tags` in the extended UI workflow.
#[cfg(feature = "providers")]
fn extract_direct_child_tags(inner_html: &str) -> Vec<String> {
    let Ok(tag_regex) = Regex::new(r"(?is)<\s*(/)?\s*([A-Za-z][A-Za-z0-9:-]*)[^>]*?>") else {
        return Vec::new();
    };

    let mut depth: i32 = 0;
    let mut direct_children = Vec::new();

    for capture in tag_regex.captures_iter(inner_html) {
        let is_closing = capture.get(1).is_some();
        let Some(name_match) = capture.get(2) else {
            continue;
        };

        let tag_name = name_match.as_str().to_ascii_lowercase();
        let full_match = capture.get(0).map_or("", |m| m.as_str());
        let self_closing = full_match.trim_end().ends_with("/>");

        if is_closing {
            depth = (depth - 1).max(0);
            continue;
        }

        if depth == 0 {
            direct_children.push(tag_name.clone());
        }

        if !self_closing && !is_void_html_tag(&tag_name) {
            depth += 1;
        }
    }

    direct_children
}

const TEMPLATE_MAX_RECURSION: usize = 64;
static TEMPLATE_USE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)^[ \t]*@use[ \t]+"([^"]+)"(?:[ \t]+as[ \t]+([A-Za-z_][A-Za-z0-9_]*|\*))?[ \t]*;[ \t]*\r?$"#)
        .unwrap()
});
static TEMPLATE_USE_ITEM_ALIAS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?s)^\s*(.*?)\s+as\s+([A-Za-z_][A-Za-z0-9_]*|\*)\s*$"#).unwrap());
#[cfg(feature = "extended-framework")]
static COMPONENT_LOCAL_TYPE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)^\s*(?:pub\s+)?(?:struct|enum)\s+([A-Za-z_][A-Za-z0-9_]*)\b"#).unwrap()
});

#[derive(Debug, Clone)]
struct TemplateUseDirective {
    target: String,
    alias: String,
    wildcard: bool,
    path_wildcard: bool,
}

#[derive(Debug, Clone)]
struct ExpandedUseTarget {
    target: String,
    alias: Option<String>,
}

#[derive(Debug, Clone)]
struct TemplateValueContext {
    values: HashMap<String, JsonValue>,
}

impl TemplateValueContext {
    fn from_sources(
        vars: &UiLangVariables,
        shared: &UiSharedValues,
        use_directives: &[TemplateUseDirective],
        local_use_aliases: &HashMap<String, String>,
    ) -> Self {
        let mut values = HashMap::new();
        for (key, value) in &vars.vars {
            values.insert(key.clone(), parse_template_context_value(value));
        }

        let mut auto_use_entries: Vec<_> = shared.auto_use_aliases.iter().collect();
        auto_use_entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (alias, target) in auto_use_entries {
            if let Some(shared_value) = shared.values.get(target) {
                if values.contains_key(alias) {
                    warn!(
                        "Auto use alias '{}' skipped because value already exists in template context",
                        alias
                    );
                } else {
                    values.insert(alias.clone(), shared_value.clone());
                }

                // Keep fields from #[html_use] values available as direct placeholders.
                // This preserves legacy {{name}}-style access while retaining alias usage.
                if let Some(map) = shared_value.as_object() {
                    for (field, field_value) in map {
                        if values.contains_key(field) {
                            continue;
                        }
                        values.insert(field.clone(), field_value.clone());
                    }
                }
            }
        }

        let mut local_use_entries: Vec<_> = local_use_aliases.iter().collect();
        local_use_entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (alias, target) in local_use_entries {
            if let Some((_resolved_target, shared_value)) =
                resolve_shared_use_target(shared, target)
            {
                if values.contains_key(alias) {
                    warn!(
                        "Component local alias '{}' skipped because value already exists in template context",
                        alias
                    );
                } else {
                    values.insert(alias.clone(), shared_value.clone());
                }
            }
        }

        for directive in use_directives {
            if directive.path_wildcard {
                import_shared_path_wildcard(&mut values, shared, &directive.target);
                continue;
            }

            let Some((resolved_target, shared_value)) =
                resolve_shared_use_target(shared, &directive.target)
            else {
                if !shared_use_target_known(shared, &directive.target) {
                    warn!("Unknown @use target '{}'", directive.target);
                }
                continue;
            };

            if directive.wildcard {
                let Some(map) = shared_value.as_object() else {
                    warn!(
                        "@use \"{}\" as * requires an object/struct value",
                        directive.target
                    );
                    continue;
                };

                for (field, field_value) in map {
                    if values.contains_key(field) {
                        warn!(
                            "Duplicate @use field '{}' ignored (target '{}')",
                            field, directive.target
                        );
                        continue;
                    }
                    values.insert(field.clone(), field_value.clone());
                }
            } else {
                if shared
                    .auto_use_aliases
                    .get(&directive.alias)
                    .is_some_and(|target| shared_targets_match(target, &resolved_target))
                {
                    // Redundant explicit import for an already auto-imported alias.
                    continue;
                }
                if local_use_aliases
                    .get(&directive.alias)
                    .is_some_and(|target| shared_targets_match(target, &resolved_target))
                {
                    // Redundant explicit import for a type from this component.rs.
                    continue;
                }
                if values.contains_key(&directive.alias) {
                    warn!(
                        "Alias '{}' from @use \"{}\" already exists in template context",
                        directive.alias, directive.target
                    );
                }
                values.insert(directive.alias.clone(), shared_value.clone());
            }
        }

        Self { values }
    }

    fn with_iteration(
        &self,
        item_name: &str,
        item_value: JsonValue,
        index_name: Option<&str>,
        index: usize,
    ) -> Self {
        let mut values = self.values.clone();
        values.insert(item_name.to_string(), item_value);
        if let Some(index_name) = index_name {
            values.insert(
                index_name.to_string(),
                JsonValue::Number(serde_json::Number::from(index as u64)),
            );
        }
        Self { values }
    }
}

fn parse_template_context_value(raw: &str) -> JsonValue {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return JsonValue::String(String::new());
    }

    serde_json::from_str(trimmed).unwrap_or_else(|_| JsonValue::String(raw.to_string()))
}

#[derive(Debug, Clone)]
struct TemplateCursor<'a> {
    src: &'a str,
    idx: usize,
}

impl<'a> TemplateCursor<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, idx: 0 }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.src.len()
    }

    fn starts_with(&self, value: &str) -> bool {
        self.src[self.idx..].starts_with(value)
    }

    fn remaining(&self) -> &'a str {
        &self.src[self.idx..]
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.idx += ch.len_utf8();
        Some(ch)
    }

    fn consume_str(&mut self, value: &str) -> bool {
        if self.starts_with(value) {
            self.idx += value.len();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if !ch.is_whitespace() {
                break;
            }
            self.next_char();
        }
    }
}

/// Expands control-flow template directives inside HTML source before DOM parsing.
///
/// Supported directives:
/// - `@if(<expr>) { ... }`
/// - `@if(<expr>) { ... } @else { ... }`
/// - `@for(item in list) { ... }`
/// - `@for(item, index in list) { ... }`
///
/// Expressions support:
/// - object paths (`data.user.name`)
/// - unary negation (`!state`)
/// - equality (`==`)
/// - enum variant literals (`DataState::Inactive`)
/// - logical `&&` / `||`
/// - method calls on values: `startsWidth`, `endsWidth`, `contains`
///   (`startsWith` / `endsWith` aliases are accepted too)
pub fn preprocess_template_directives(template: &str, vars: &UiLangVariables) -> String {
    let shared = UiSharedValues::default();
    preprocess_template_directives_with_shared(template, vars, &shared)
}

/// Preprocesses template directives with typed shared values and `@use` imports.
pub fn preprocess_template_directives_with_shared(
    template: &str,
    vars: &UiLangVariables,
    shared: &UiSharedValues,
) -> String {
    preprocess_template_directives_with_shared_and_local_types(
        template,
        vars,
        shared,
        &Vec::<String>::new(),
    )
}

/// Preprocesses template directives with typed shared values, `@use` imports,
/// and implicit aliases for types declared in the matching `*.component.rs`.
pub fn preprocess_template_directives_with_shared_and_local_types<T: AsRef<str>>(
    template: &str,
    vars: &UiLangVariables,
    shared: &UiSharedValues,
    local_type_names: &[T],
) -> String {
    let (use_directives, cleaned_template) = extract_template_use_directives(template);
    let local_use_aliases = build_local_use_aliases(shared, local_type_names);
    let context =
        TemplateValueContext::from_sources(vars, shared, &use_directives, &local_use_aliases);
    render_template_with_context(&cleaned_template, &context, 0)
}

fn render_template_with_context(
    template: &str,
    context: &TemplateValueContext,
    depth: usize,
) -> String {
    if depth > TEMPLATE_MAX_RECURSION {
        return template.to_string();
    }

    let mut cursor = TemplateCursor::new(template);
    render_template_segment(&mut cursor, context, depth, false)
}

fn render_template_segment(
    cursor: &mut TemplateCursor,
    context: &TemplateValueContext,
    depth: usize,
    stop_on_block_end: bool,
) -> String {
    let mut output = String::new();

    while !cursor.is_eof() {
        if stop_on_block_end && cursor.starts_with("}") {
            cursor.next_char();
            break;
        }

        if cursor.starts_with("{{") {
            output.push_str(&render_moustache(cursor, context));
            continue;
        }

        if is_template_directive(cursor, "@if") {
            if let Some(rendered) = parse_if_directive(cursor, context, depth + 1) {
                output.push_str(&rendered);
                continue;
            }
        }

        if is_template_directive(cursor, "@for") {
            if let Some(rendered) = parse_for_directive(cursor, context, depth + 1) {
                output.push_str(&rendered);
                continue;
            }
        }

        if let Some(ch) = cursor.next_char() {
            output.push(ch);
        } else {
            break;
        }
    }

    output
}

fn is_template_directive(cursor: &TemplateCursor, keyword: &str) -> bool {
    if !cursor.starts_with(keyword) {
        return false;
    }

    let next = cursor.remaining()[keyword.len()..].chars().next();
    !matches!(next, Some(ch) if ch.is_alphanumeric() || ch == '_')
}

fn parse_if_directive(
    cursor: &mut TemplateCursor,
    context: &TemplateValueContext,
    depth: usize,
) -> Option<String> {
    let checkpoint = cursor.idx;
    if !cursor.consume_str("@if") {
        return None;
    }
    cursor.skip_whitespace();
    let condition_raw = match extract_group_content(cursor, '(', ')') {
        Some(value) => value,
        None => {
            cursor.idx = checkpoint;
            return None;
        }
    };
    cursor.skip_whitespace();
    let then_source = match extract_block_content(cursor) {
        Some(value) => value,
        None => {
            cursor.idx = checkpoint;
            return None;
        }
    };

    let mut else_source = None;
    cursor.skip_whitespace();
    if is_template_directive(cursor, "@else") {
        cursor.consume_str("@else");
        cursor.skip_whitespace();
        else_source = match extract_block_content(cursor) {
            Some(value) => Some(value),
            None => {
                cursor.idx = checkpoint;
                return None;
            }
        };
    }

    let condition = evaluate_condition_expression(condition_raw.as_str(), context);
    let selected = if condition {
        then_source
    } else {
        else_source.unwrap_or_default()
    };

    Some(render_template_with_context(&selected, context, depth))
}

fn parse_for_directive(
    cursor: &mut TemplateCursor,
    context: &TemplateValueContext,
    depth: usize,
) -> Option<String> {
    let checkpoint = cursor.idx;
    if !cursor.consume_str("@for") {
        return None;
    }
    cursor.skip_whitespace();
    let header = match extract_group_content(cursor, '(', ')') {
        Some(value) => value,
        None => {
            cursor.idx = checkpoint;
            return None;
        }
    };
    cursor.skip_whitespace();
    let block_source = match extract_block_content(cursor) {
        Some(value) => value,
        None => {
            cursor.idx = checkpoint;
            return None;
        }
    };

    let (item_name, index_name, iterable_expression) = match parse_for_header(&header) {
        Some(value) => value,
        None => {
            cursor.idx = checkpoint;
            return None;
        }
    };
    let Some(iterable_value) = evaluate_expression(&iterable_expression, context) else {
        warn!("Failed to evaluate @for expression: {iterable_expression}");
        return Some(String::new());
    };

    let mut rendered = String::new();
    for (index, item) in iterable_items(iterable_value).into_iter().enumerate() {
        let iteration_context =
            context.with_iteration(&item_name, item, index_name.as_deref(), index);
        let nested = render_template_with_context(&block_source, &iteration_context, depth);
        rendered.push_str(&interpolate_inline_placeholders(
            &nested,
            &iteration_context,
        ));
    }

    Some(rendered)
}

fn parse_for_header(header: &str) -> Option<(String, Option<String>, String)> {
    let captures = FOR_HEADER_RE.captures(header.trim())?;
    let item_name = captures.get(1)?.as_str().to_string();
    let index_name = captures.get(2).map(|value| value.as_str().to_string());
    let iterable_expression = captures.get(3)?.as_str().trim().to_string();

    Some((item_name, index_name, iterable_expression))
}

fn extract_template_use_directives(template: &str) -> (Vec<TemplateUseDirective>, String) {
    let mut directives = Vec::new();

    for captures in TEMPLATE_USE_RE.captures_iter(template) {
        let Some(target) = captures.get(1).map(|m| m.as_str().trim().to_string()) else {
            continue;
        };
        let line_alias = captures.get(2).map(|m| m.as_str().trim().to_string());
        let expanded_targets = expand_use_targets(&target);

        if line_alias.is_some() && expanded_targets.len() > 1 {
            warn!(
                "@use \"{}\" imports multiple targets; ignoring shared alias",
                target
            );
        }

        for expanded_target in expanded_targets {
            let path_wildcard = is_path_wildcard_use_target(&expanded_target.target);
            if path_wildcard && (line_alias.is_some() || expanded_target.alias.as_ref().is_some()) {
                warn!(
                    "@use \"{}\" imports a path wildcard; ignoring alias",
                    expanded_target.target
                );
            }
            let alias = if path_wildcard {
                String::new()
            } else {
                expanded_target
                    .alias
                    .or_else(|| {
                        line_alias
                            .clone()
                            .filter(|_| !is_grouped_use_target(&target))
                    })
                    .unwrap_or_else(|| default_use_alias(&expanded_target.target))
            };

            directives.push(TemplateUseDirective {
                target: expanded_target.target,
                wildcard: alias == "*" && !path_wildcard,
                path_wildcard,
                alias,
            });
        }
    }

    let cleaned = TEMPLATE_USE_RE.replace_all(template, "").into_owned();
    (directives, cleaned)
}

fn expand_use_targets(target: &str) -> Vec<ExpandedUseTarget> {
    let target = target.trim();
    let Some((prefix, group)) = split_grouped_use_target(target) else {
        return vec![ExpandedUseTarget {
            target: target.to_string(),
            alias: None,
        }];
    };

    let expanded = group
        .split(',')
        .filter_map(|item| parse_grouped_use_item(prefix, item))
        .collect::<Vec<_>>();

    if expanded.is_empty() {
        vec![ExpandedUseTarget {
            target: target.to_string(),
            alias: None,
        }]
    } else {
        expanded
    }
}

fn parse_grouped_use_item(prefix: &str, item: &str) -> Option<ExpandedUseTarget> {
    let item = item.trim();
    if item.is_empty() {
        return None;
    }

    let (item_target, alias) = split_use_item_alias(item);
    let item_target = item_target.trim();
    if item_target.is_empty() {
        return None;
    }

    let target = if item_target == "self" {
        prefix.to_string()
    } else {
        format!("{prefix}::{item_target}")
    };

    Some(ExpandedUseTarget { target, alias })
}

fn split_use_item_alias(item: &str) -> (&str, Option<String>) {
    let Some(captures) = TEMPLATE_USE_ITEM_ALIAS_RE.captures(item) else {
        return (item, None);
    };
    let Some(target) = captures.get(1).map(|m| m.as_str()) else {
        return (item, None);
    };
    let Some(alias) = captures.get(2).map(|m| m.as_str().trim().to_string()) else {
        return (item, None);
    };

    (target, Some(alias))
}

fn split_grouped_use_target(target: &str) -> Option<(&str, &str)> {
    let target = target.trim();
    let group_end = target.rfind('}')?;
    if !target[group_end + 1..].trim().is_empty() {
        return None;
    }

    let group_start = target[..group_end].rfind('{')?;
    let prefix = target[..group_start].trim();
    let prefix = prefix.strip_suffix("::")?.trim();
    let group = &target[group_start + 1..group_end];

    if prefix.is_empty() {
        None
    } else {
        Some((prefix, group))
    }
}

fn is_grouped_use_target(target: &str) -> bool {
    split_grouped_use_target(target).is_some()
}

fn is_path_wildcard_use_target(target: &str) -> bool {
    wildcard_use_prefix(target).is_some()
}

fn wildcard_use_prefix(target: &str) -> Option<&str> {
    let prefix = target.trim().strip_suffix("::*")?.trim();
    if prefix.is_empty() {
        None
    } else {
        Some(prefix)
    }
}

fn import_shared_path_wildcard(
    values: &mut HashMap<String, JsonValue>,
    shared: &UiSharedValues,
    target: &str,
) {
    let mut matches = shared_path_wildcard_targets(shared, target);
    if matches.is_empty() {
        warn!("Unknown @use wildcard target '{}'", target);
        return;
    }

    matches.sort_by(|(left, _), (right, _)| left.cmp(right));
    for (registered, value) in matches {
        let alias = default_use_alias(&registered);
        if values.contains_key(&alias) {
            warn!(
                "Duplicate @use wildcard alias '{}' ignored (target '{}')",
                alias, registered
            );
            continue;
        }
        values.insert(alias, value.clone());
    }
}

fn shared_path_wildcard_targets<'a>(
    shared: &'a UiSharedValues,
    target: &str,
) -> Vec<(String, &'a JsonValue)> {
    let Some(prefix) = wildcard_use_prefix(target) else {
        return Vec::new();
    };

    shared
        .values
        .iter()
        .filter(|(registered, _)| shared_path_prefix_matches(registered, prefix))
        .map(|(registered, value)| (registered.clone(), value))
        .collect()
}

fn shared_path_prefix_matches(registered: &str, requested_prefix: &str) -> bool {
    let Some((registered_prefix, _type_name)) = registered.trim().rsplit_once("::") else {
        return false;
    };

    if registered_prefix == requested_prefix {
        return true;
    }

    let Some(requested_without_crate) = requested_prefix.trim().strip_prefix("crate::") else {
        return false;
    };

    registered_prefix == requested_without_crate
        || registered_prefix
            .trim()
            .ends_with(format!("::{requested_without_crate}").as_str())
}

fn build_local_use_aliases<T: AsRef<str>>(
    shared: &UiSharedValues,
    local_type_names: &[T],
) -> HashMap<String, String> {
    let mut aliases = HashMap::new();

    for type_name in local_type_names {
        let type_name = simple_type_name(type_name.as_ref()).trim();
        if type_name.is_empty() || resolve_shared_use_target(shared, type_name).is_none() {
            continue;
        }

        aliases
            .entry(default_use_alias(type_name))
            .or_insert_with(|| type_name.to_string());
    }

    aliases
}

#[cfg(feature = "extended-framework")]
fn component_local_type_names(
    source_path: &str,
    config: &ExtendedFrameworkConfiguration,
) -> Vec<String> {
    let Some(component_path) = component_rust_path_for_template(source_path, config) else {
        return Vec::new();
    };
    let Ok(component_source) = fs::read_to_string(&component_path) else {
        return Vec::new();
    };

    let mut names = COMPONENT_LOCAL_TYPE_RE
        .captures_iter(&component_source)
        .filter_map(|captures| captures.get(1).map(|matched| matched.as_str().to_string()))
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

#[cfg(feature = "extended-framework")]
fn component_local_type_names_from_rust_paths<T: AsRef<str>>(paths: &[T]) -> Vec<String> {
    let mut names = Vec::new();
    for path in paths {
        let Ok(component_source) = fs::read_to_string(path.as_ref()) else {
            continue;
        };
        names.extend(
            COMPONENT_LOCAL_TYPE_RE
                .captures_iter(&component_source)
                .filter_map(|captures| captures.get(1).map(|matched| matched.as_str().to_string())),
        );
    }
    names.sort();
    names.dedup();
    names
}

#[cfg(feature = "extended-framework")]
fn component_rust_path_for_template(
    source_path: &str,
    config: &ExtendedFrameworkConfiguration,
) -> Option<PathBuf> {
    let source = normalize_component_scope_path(source_path);
    let root = normalize_component_scope_path(&config.assets_component_root);
    if root.is_empty() {
        return None;
    }

    let root_prefix = format!("{root}/");
    let relative_template = source.strip_prefix(&root_prefix).or_else(|| {
        let nested_prefix = format!("/{root}/");
        source
            .split_once(&nested_prefix)
            .map(|(_, relative)| relative)
    })?;

    let relative_rust = relative_template
        .strip_suffix(".component.html")
        .map(|base| format!("{base}.component.rs"))?;

    Some(Path::new(&config.rust_component_root).join(relative_rust))
}

#[cfg(feature = "extended-framework")]
fn normalize_component_scope_path(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_string();
    }
    if let Some(rest) = normalized.strip_prefix("assets/") {
        normalized = rest.to_string();
    }
    normalized.trim_matches('/').to_string()
}

fn resolve_shared_use_target<'a>(
    shared: &'a UiSharedValues,
    target: &str,
) -> Option<(String, &'a JsonValue)> {
    if let Some(value) = shared.values.get(target) {
        return Some((target.to_string(), value));
    }

    if target.contains("::") {
        let mut path_matches: Vec<_> = shared
            .values
            .iter()
            .filter(|(registered, _)| shared_paths_match(registered, target))
            .collect();
        path_matches.sort_by(|(left, _), (right, _)| left.cmp(right));

        if let Some((registered, value)) = path_matches.first() {
            if path_matches.len() > 1 {
                warn!(
                    "Ambiguous @use target '{}' matched multiple shared paths; using '{}'",
                    target, registered
                );
            }
            return Some(((*registered).clone(), *value));
        }
    }

    let simple_name = simple_type_name(target);
    shared
        .values
        .get(simple_name)
        .map(|value| (simple_name.to_string(), value))
}

fn shared_use_target_known(shared: &UiSharedValues, target: &str) -> bool {
    shared.known_types.contains(target)
        || shared
            .known_types
            .iter()
            .any(|registered| shared_paths_match(registered, target))
        || shared.known_types.contains(simple_type_name(target))
}

fn shared_targets_match(left: &str, right: &str) -> bool {
    left == right
        || shared_paths_match(left, right)
        || simple_type_name(left) == simple_type_name(right)
}

fn shared_paths_match(registered: &str, requested: &str) -> bool {
    if registered == requested {
        return true;
    }

    let Some(requested_without_crate) = requested.trim().strip_prefix("crate::") else {
        return false;
    };

    registered == requested_without_crate
        || registered
            .trim()
            .ends_with(format!("::{requested_without_crate}").as_str())
}

fn default_use_alias(target: &str) -> String {
    to_template_alias(simple_type_name(target))
}

fn simple_type_name(target: &str) -> &str {
    target.trim().rsplit("::").next().unwrap_or(target).trim()
}

fn to_template_alias(type_name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in type_name.chars().enumerate() {
        if ch.is_uppercase() {
            if index != 0 {
                out.push('_');
            }
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn iterable_items(value: JsonValue) -> Vec<JsonValue> {
    match value {
        JsonValue::Array(items) => items,
        JsonValue::Object(map) => map.into_values().collect(),
        _ => Vec::new(),
    }
}

fn render_moustache(cursor: &mut TemplateCursor, context: &TemplateValueContext) -> String {
    let (raw, expression) = consume_moustache(cursor);
    let Some(expression) = expression else {
        return raw;
    };

    let expression = expression.trim();
    if expression.is_empty() {
        return raw;
    }

    let Some(evaluated) = evaluate_expression(expression, context) else {
        return raw;
    };

    value_to_inline_string(&evaluated).unwrap_or(raw)
}

fn consume_moustache(cursor: &mut TemplateCursor) -> (String, Option<String>) {
    let mut raw = String::new();
    let mut expression = String::new();

    if !cursor.consume_str("{{") {
        return (raw, None);
    }

    raw.push_str("{{");
    while !cursor.is_eof() {
        if cursor.starts_with("}}") {
            cursor.consume_str("}}");
            raw.push_str("}}");
            return (raw, Some(expression));
        }
        if let Some(ch) = cursor.next_char() {
            raw.push(ch);
            expression.push(ch);
        }
    }

    (raw, None)
}

fn extract_group_content(cursor: &mut TemplateCursor, open: char, close: char) -> Option<String> {
    if cursor.peek_char()? != open {
        return None;
    }

    cursor.next_char();
    let mut content = String::new();
    let mut depth = 1usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    while let Some(ch) = cursor.next_char() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            content.push(ch);
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            content.push(ch);
            continue;
        }

        if ch == open {
            depth += 1;
            content.push(ch);
            continue;
        }

        if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(content);
            }
            content.push(ch);
            continue;
        }

        content.push(ch);
    }

    None
}

fn extract_block_content(cursor: &mut TemplateCursor) -> Option<String> {
    extract_group_content(cursor, '{', '}')
}

fn interpolate_inline_placeholders(value: &str, context: &TemplateValueContext) -> String {
    INNER_BINDING_RE
        .replace_all(value, |caps: &regex::Captures| {
            let Some(expression) = caps.get(1).map(|m| m.as_str()) else {
                return caps
                    .get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string();
            };

            let Some(evaluated) = evaluate_expression(expression, context) else {
                return caps
                    .get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string();
            };

            value_to_inline_string(&evaluated).unwrap_or_else(|| {
                caps.get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string()
            })
        })
        .into_owned()
}

fn evaluate_condition_expression(expression: &str, context: &TemplateValueContext) -> bool {
    evaluate_expression(expression, context)
        .map(|value| value_truthy(&value))
        .unwrap_or(false)
}

fn value_truthy(value: &JsonValue) -> bool {
    match value {
        JsonValue::Bool(value) => *value,
        JsonValue::Null => false,
        JsonValue::Number(value) => value.as_f64().is_some_and(|number| number != 0.0),
        JsonValue::String(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            if normalized == "true" {
                true
            } else if normalized == "false" || normalized.is_empty() {
                false
            } else {
                true
            }
        }
        JsonValue::Array(items) => !items.is_empty(),
        JsonValue::Object(map) => !map.is_empty(),
    }
}

fn value_to_inline_string(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Bool(value) => Some(value.to_string()),
        JsonValue::Number(value) => Some(value.to_string()),
        JsonValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ExprToken {
    Identifier(String),
    String(String),
    Number(f64),
    Bool(bool),
    LParen,
    RParen,
    Dot,
    ColonColon,
    Comma,
    LBracket,
    RBracket,
    EqEq,
    BangEq,
    AndAnd,
    OrOr,
    Bang,
}

fn tokenize_expression(expression: &str) -> Option<Vec<ExprToken>> {
    let mut tokens = Vec::new();
    let mut chars = expression.chars().peekable();

    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        if ch == '&' {
            chars.next();
            if chars.next()? != '&' {
                return None;
            }
            tokens.push(ExprToken::AndAnd);
            continue;
        }

        if ch == '|' {
            chars.next();
            if chars.next()? != '|' {
                return None;
            }
            tokens.push(ExprToken::OrOr);
            continue;
        }

        if ch == '=' {
            chars.next();
            if chars.next()? != '=' {
                return None;
            }
            tokens.push(ExprToken::EqEq);
            continue;
        }

        if ch == '!' {
            chars.next();
            if chars.peek() == Some(&'=') {
                chars.next();
                tokens.push(ExprToken::BangEq);
                continue;
            }
            tokens.push(ExprToken::Bang);
            continue;
        }

        if ch == '(' {
            chars.next();
            tokens.push(ExprToken::LParen);
            continue;
        }

        if ch == ')' {
            chars.next();
            tokens.push(ExprToken::RParen);
            continue;
        }

        if ch == '.' {
            chars.next();
            tokens.push(ExprToken::Dot);
            continue;
        }

        if ch == ':' {
            chars.next();
            if chars.next()? != ':' {
                return None;
            }
            tokens.push(ExprToken::ColonColon);
            continue;
        }

        if ch == ',' {
            chars.next();
            tokens.push(ExprToken::Comma);
            continue;
        }

        if ch == '[' {
            chars.next();
            tokens.push(ExprToken::LBracket);
            continue;
        }

        if ch == ']' {
            chars.next();
            tokens.push(ExprToken::RBracket);
            continue;
        }

        if ch == '\'' || ch == '"' {
            let quote = ch;
            chars.next();

            let mut value = String::new();
            let mut escaped = false;

            for current in chars.by_ref() {
                if escaped {
                    value.push(current);
                    escaped = false;
                    continue;
                }

                if current == '\\' {
                    escaped = true;
                    continue;
                }

                if current == quote {
                    break;
                }

                value.push(current);
            }

            tokens.push(ExprToken::String(value));
            continue;
        }

        if ch.is_ascii_digit()
            || (ch == '-'
                && chars
                    .clone()
                    .nth(1)
                    .is_some_and(|next| next.is_ascii_digit()))
        {
            let mut raw = String::new();
            raw.push(ch);
            chars.next();

            while let Some(next) = chars.peek().copied() {
                if !(next.is_ascii_digit() || next == '.') {
                    break;
                }
                raw.push(next);
                chars.next();
            }

            let number = raw.parse::<f64>().ok()?;
            tokens.push(ExprToken::Number(number));
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut ident = String::new();
            ident.push(ch);
            chars.next();

            while let Some(next) = chars.peek().copied() {
                if !(next.is_ascii_alphanumeric() || next == '_') {
                    break;
                }
                ident.push(next);
                chars.next();
            }

            let lower = ident.to_ascii_lowercase();
            if lower == "true" {
                tokens.push(ExprToken::Bool(true));
            } else if lower == "false" {
                tokens.push(ExprToken::Bool(false));
            } else {
                tokens.push(ExprToken::Identifier(ident));
            }
            continue;
        }

        return None;
    }

    Some(tokens)
}

fn evaluate_expression(expression: &str, context: &TemplateValueContext) -> Option<JsonValue> {
    let tokens = tokenize_expression(expression)?;
    let mut parser = ExpressionParser {
        tokens: &tokens,
        index: 0,
        context,
    };

    let value = parser.parse_or()?;
    if parser.index != parser.tokens.len() {
        return None;
    }

    Some(value)
}

struct ExpressionParser<'a> {
    tokens: &'a [ExprToken],
    index: usize,
    context: &'a TemplateValueContext,
}

impl<'a> ExpressionParser<'a> {
    fn peek(&self) -> Option<&ExprToken> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<&ExprToken> {
        let token = self.tokens.get(self.index)?;
        self.index += 1;
        Some(token)
    }

    fn consume(&mut self, expected: ExprToken) -> bool {
        if self.peek() == Some(&expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn parse_or(&mut self) -> Option<JsonValue> {
        let mut left = self.parse_and()?;
        while self.consume(ExprToken::OrOr) {
            let right = self.parse_and()?;
            left = JsonValue::Bool(value_truthy(&left) || value_truthy(&right));
        }
        Some(left)
    }

    fn parse_and(&mut self) -> Option<JsonValue> {
        let mut left = self.parse_equality()?;
        while self.consume(ExprToken::AndAnd) {
            let right = self.parse_equality()?;
            left = JsonValue::Bool(value_truthy(&left) && value_truthy(&right));
        }
        Some(left)
    }

    fn parse_equality(&mut self) -> Option<JsonValue> {
        let mut left = self.parse_unary()?;
        loop {
            if self.consume(ExprToken::EqEq) {
                let right = self.parse_unary()?;
                left = JsonValue::Bool(json_values_equal(&left, &right));
                continue;
            }

            if self.consume(ExprToken::BangEq) {
                let right = self.parse_unary()?;
                left = JsonValue::Bool(!json_values_equal(&left, &right));
                continue;
            }

            break;
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<JsonValue> {
        if self.consume(ExprToken::Bang) {
            let value = self.parse_unary()?;
            return Some(JsonValue::Bool(!value_truthy(&value)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Option<JsonValue> {
        match self.peek()? {
            ExprToken::LParen => {
                self.next();
                let inner = self.parse_or()?;
                if !self.consume(ExprToken::RParen) {
                    return None;
                }
                Some(inner)
            }
            ExprToken::Bool(value) => {
                let value = *value;
                self.next();
                Some(JsonValue::Bool(value))
            }
            ExprToken::Number(value) => {
                let value = *value;
                self.next();
                Some(
                    serde_json::Number::from_f64(value)
                        .map(JsonValue::Number)
                        .unwrap_or(JsonValue::Null),
                )
            }
            ExprToken::String(value) => {
                let value = value.clone();
                self.next();
                Some(JsonValue::String(value))
            }
            ExprToken::Identifier(_) => self.parse_identifier_chain(),
            _ => None,
        }
    }

    fn parse_identifier_chain(&mut self) -> Option<JsonValue> {
        let base_name = match self.next()? {
            ExprToken::Identifier(name) => name.clone(),
            _ => return None,
        };

        if self.peek() == Some(&ExprToken::ColonColon) {
            return self.parse_rust_path_literal(base_name);
        }

        let mut current = self
            .context
            .values
            .get(&base_name)
            .cloned()
            .unwrap_or(JsonValue::Null);

        loop {
            if self.consume(ExprToken::Dot) {
                let name = match self.next()? {
                    ExprToken::Identifier(name) => name.clone(),
                    _ => return None,
                };

                if self.consume(ExprToken::LParen) {
                    let args = self.parse_call_arguments()?;
                    current = evaluate_value_method(&current, &name, &args);
                    continue;
                }

                current = resolve_json_property(&current, &name);
                continue;
            }

            if self.consume(ExprToken::LBracket) {
                let token = self.next()?.clone();
                let index_value = match token {
                    ExprToken::Number(number) => {
                        serde_json::Number::from_f64(number).map(JsonValue::Number)?
                    }
                    ExprToken::String(value) => JsonValue::String(value),
                    ExprToken::Identifier(value) => self
                        .context
                        .values
                        .get(&value)
                        .cloned()
                        .unwrap_or(JsonValue::Null),
                    _ => return None,
                };

                if !self.consume(ExprToken::RBracket) {
                    return None;
                }

                current = resolve_json_index(&current, &index_value);
                continue;
            }

            break;
        }

        Some(current)
    }

    fn parse_rust_path_literal(&mut self, base_name: String) -> Option<JsonValue> {
        let mut last_segment = base_name;

        while self.consume(ExprToken::ColonColon) {
            last_segment = match self.next()? {
                ExprToken::Identifier(name) => name.clone(),
                _ => return None,
            };
        }

        Some(JsonValue::String(last_segment))
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<JsonValue>> {
        let mut args = Vec::new();
        if self.consume(ExprToken::RParen) {
            return Some(args);
        }

        loop {
            args.push(self.parse_or()?);

            if self.consume(ExprToken::Comma) {
                continue;
            }

            if self.consume(ExprToken::RParen) {
                break;
            }

            return None;
        }

        Some(args)
    }
}

fn resolve_json_property(source: &JsonValue, key: &str) -> JsonValue {
    match source {
        JsonValue::Object(map) => map.get(key).cloned().unwrap_or(JsonValue::Null),
        _ => JsonValue::Null,
    }
}

fn resolve_json_index(source: &JsonValue, index: &JsonValue) -> JsonValue {
    match (source, index) {
        (JsonValue::Array(list), JsonValue::Number(number)) => {
            let Some(index) = number.as_u64() else {
                return JsonValue::Null;
            };
            list.get(index as usize).cloned().unwrap_or(JsonValue::Null)
        }
        (JsonValue::Object(map), JsonValue::String(key)) => {
            map.get(key).cloned().unwrap_or(JsonValue::Null)
        }
        _ => JsonValue::Null,
    }
}

fn evaluate_value_method(target: &JsonValue, method: &str, args: &[JsonValue]) -> JsonValue {
    let method = method.to_ascii_lowercase();
    match method.as_str() {
        "startswidth" | "startswith" => {
            let Some(prefix) = args.first().and_then(json_as_string) else {
                return JsonValue::Bool(false);
            };
            let Some(value) = json_as_string(target) else {
                return JsonValue::Bool(false);
            };
            JsonValue::Bool(value.starts_with(prefix.as_str()))
        }
        "endswidth" | "endswith" => {
            let Some(suffix) = args.first().and_then(json_as_string) else {
                return JsonValue::Bool(false);
            };
            let Some(value) = json_as_string(target) else {
                return JsonValue::Bool(false);
            };
            JsonValue::Bool(value.ends_with(suffix.as_str()))
        }
        "contains" => {
            let Some(value) = args.first() else {
                return JsonValue::Bool(false);
            };

            match target {
                JsonValue::String(text) => {
                    let Some(needle) = json_as_string(value) else {
                        return JsonValue::Bool(false);
                    };
                    JsonValue::Bool(text.contains(needle.as_str()))
                }
                JsonValue::Array(items) => {
                    JsonValue::Bool(items.iter().any(|item| json_values_equal(item, value)))
                }
                JsonValue::Object(map) => {
                    let Some(key) = json_as_string(value) else {
                        return JsonValue::Bool(false);
                    };
                    JsonValue::Bool(map.contains_key(key.as_str()))
                }
                _ => JsonValue::Bool(false),
            }
        }
        _ if args.is_empty() => evaluate_zero_arg_getter(target, method.as_str()),
        _ => JsonValue::Null,
    }
}

fn evaluate_zero_arg_getter(target: &JsonValue, method: &str) -> JsonValue {
    let Some(field_name) = method.strip_prefix("get_") else {
        return JsonValue::Null;
    };

    resolve_json_property(target, field_name)
}

fn json_as_string(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(value) => Some(value.clone()),
        JsonValue::Number(value) => Some(value.to_string()),
        JsonValue::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn json_values_equal(left: &JsonValue, right: &JsonValue) -> bool {
    match (left, right) {
        (JsonValue::Number(left), JsonValue::Number(right)) => {
            let (Some(left), Some(right)) = (left.as_f64(), right.as_f64()) else {
                return false;
            };
            (left - right).abs() <= f64::EPSILON
        }
        _ => left == right,
    }
}

/// Handles `is_void_html_tag` in the extended UI workflow.
#[cfg(feature = "providers")]
fn is_void_html_tag(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// Parses an optional icon source and determines its placement relative to text.
fn parse_icon_and_text(node: &NodeRef) -> (Option<String>, IconPlace) {
    let mut icon_path = None;
    let mut icon_place = IconPlace::Left;
    let mut found_text = false;

    for child in node.children() {
        if let Some(el) = child.as_element() {
            if el.name.local.eq("icon") {
                if let Some(src) = el.attributes.borrow().get("src") {
                    icon_path = Some(src.to_string());
                    if found_text {
                        icon_place = IconPlace::Right;
                    }
                }
            }
        } else if child
            .as_text()
            .map(|t| !t.borrow().trim().is_empty())
            .unwrap_or(false)
        {
            found_text = true;
        }
    }

    (icon_path, icon_place)
}

/// Builds the per-widget inner content payload.
pub fn parse_inner_content(node: &NodeRef) -> HtmlInnerContent {
    let inner_text = node.text_contents();
    let inner_html = node
        .children()
        .map(|child| child.to_string())
        .collect::<String>();
    let inner_bindings = extract_inner_bindings(&inner_html);

    HtmlInnerContent::new(inner_text, inner_html, inner_bindings)
}

/// Parses tooltip trigger attribute values like `"hover | click"`.
fn parse_tooltip_triggers(value: &str) -> Vec<ToolTipTrigger> {
    let mut out = Vec::new();

    for token in value.split(['|', ',', ' ']) {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(trigger) = ToolTipTrigger::from_str(trimmed) {
            if !out.contains(&trigger) {
                out.push(trigger);
            }
        }
    }

    if out.is_empty() {
        out.push(ToolTipTrigger::Hover);
    }

    out
}

/// Extracts unique `{{...}}` placeholders from serialized content.
pub fn extract_inner_bindings(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for caps in INNER_BINDING_RE.captures_iter(content) {
        let Some(raw) = caps.get(0).map(|m| m.as_str()) else {
            continue;
        };

        let key = raw.trim().to_string();
        if seen.insert(key.clone()) {
            out.push(key);
        }
    }

    out
}

/// Converts an HTML option's `value` string into a [`WidgetValue`] using the
/// `internal-value-type` attribute hint.
///
/// # Primitive types (parsed directly from the string)
/// `bool`, `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `string` / `str`
///
/// # Structured types (deserialized via Bevy reflection)
/// Any other type name — looked up in the Bevy `TypeRegistry` by short path (e.g. `"MyStruct"`)
/// then deserialized from the JSON value string using `TypedReflectDeserializer` and wrapped in
/// a [`ReflectedValue`]. Falls back to `serde_json::Value` if the type is not registered, and
/// to a plain `String` if that also fails.
///
/// # Fallback
/// An empty or unrecognised `type_hint` stores the value as a plain `String`.
fn parse_option_internal_value(
    value: &str,
    type_hint: &str,
    type_registry: &TypeRegistry,
) -> WidgetValue {
    use crate::widgets::ReflectedValue;

    if value.is_empty() {
        return WidgetValue::default();
    }

    match type_hint {
        "bool" => match value.trim() {
            "true" | "1" | "yes" => WidgetValue::new(true),
            _ => WidgetValue::new(false),
        },
        "i8" => WidgetValue::new(value.trim().parse::<i8>().unwrap_or(0)),
        "i16" => WidgetValue::new(value.trim().parse::<i16>().unwrap_or(0)),
        "i32" => WidgetValue::new(value.trim().parse::<i32>().unwrap_or(0)),
        "i64" => WidgetValue::new(value.trim().parse::<i64>().unwrap_or(0)),
        "u8" => WidgetValue::new(value.trim().parse::<u8>().unwrap_or(0)),
        "u16" => WidgetValue::new(value.trim().parse::<u16>().unwrap_or(0)),
        "u32" => WidgetValue::new(value.trim().parse::<u32>().unwrap_or(0)),
        "u64" => WidgetValue::new(value.trim().parse::<u64>().unwrap_or(0)),
        "f32" => WidgetValue::new(value.trim().parse::<f32>().unwrap_or(0.0)),
        "f64" => WidgetValue::new(value.trim().parse::<f64>().unwrap_or(0.0)),
        "" | "string" | "str" => WidgetValue::new(value.to_string()),
        type_name => {
            // Try Bevy reflection first (short path, then full path).
            let registration = type_registry
                .get_with_short_type_path(type_name)
                .or_else(|| type_registry.get_with_type_path(type_name));

            if let Some(registration) = registration {
                let deserializer = TypedReflectDeserializer::new(registration, type_registry);
                let mut json_de = serde_json::Deserializer::from_str(value);
                match deserializer.deserialize(&mut json_de) {
                    Ok(reflected) => WidgetValue::new(ReflectedValue(reflected)),
                    Err(_) => WidgetValue::new(value.to_string()),
                }
            } else {
                // Type not registered — fall back to serde_json::Value, then String.
                match serde_json::from_str::<JsonValue>(value) {
                    Ok(json) => WidgetValue::new(json),
                    Err(_) => WidgetValue::new(value.to_string()),
                }
            }
        }
    }
}
