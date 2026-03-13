use std::collections::{HashMap, HashSet};

use bevy::asset::AssetEvent;
use bevy::prelude::*;
use kuchiki::{Attributes, NodeRef, traits::TendrilSink};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::ExtendedUiConfiguration;
#[cfg(feature = "extended-dialog")]
use crate::dialog::{DialogProvider, DialogWidget, DialogWidgetType};
use crate::html::{
    HtmlDirty, HtmlEventBindings, HtmlID, HtmlInnerContent, HtmlMeta, HtmlSource, HtmlStates,
    HtmlStructureMap, HtmlStyle, HtmlSystemSet, HtmlWidgetNode,
};
use crate::io::{CssAsset, DefaultCssHandle, HtmlAsset};
use crate::lang::{UILang, UiLangState, UiLangVariables, localize_html, vars_fingerprint};
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

/// Converts HtmlAsset content into HtmlStructureMap entries.
/// Also resolves <link rel="stylesheet" href="..."> into Handle<CssAsset>.
/// Updates parsed HTML structures from assets and language state.
fn update_html_ui(
    mut commands: Commands,
    mut structure_map: ResMut<HtmlStructureMap>,
    mut html_dirty: ResMut<HtmlDirty>,
    mut ui_lang: ResMut<UILang>,
    mut lang_state: ResMut<UiLangState>,

    lang_vars: Res<UiLangVariables>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    html_assets: Res<Assets<HtmlAsset>>,
    mut html_asset_events: MessageReader<AssetEvent<HtmlAsset>>,
    default_css: Res<DefaultCssHandle>,
    #[cfg(feature = "providers")] provider_registry: Option<Res<UiProviderRegistry>>,
    #[cfg(feature = "providers")] mut theme_provider_state: Option<ResMut<ThemeProviderState>>,

    query_added: Query<(Entity, &HtmlSource), Added<HtmlSource>>,
    query_pending: Query<Entity, With<PendingHtmlParse>>,
    query_all: Query<(Entity, &HtmlSource)>,
) {
    let resolved = ui_lang.resolved().map(|lang| lang.to_string());
    let mut lang_dirty = false;
    let vars_hash = vars_fingerprint(&lang_vars);

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

    // Entities that need reparse (new HtmlSource, pending retry, or changed HtmlAsset).
    let mut dirty_entities: Vec<Entity> = query_added.iter().map(|(e, _)| e).collect();
    dirty_entities.extend(query_pending.iter());

    // If an HtmlAsset changed OR was added later (async), find all entities referencing it.
    for ev in html_asset_events.read() {
        let id = match ev {
            AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                *id
            }
            _ => continue,
        };

        for (entity, src) in query_all.iter() {
            if src.handle.id() == id {
                dirty_entities.push(entity);
            }
        }
    }

    dirty_entities.sort();
    dirty_entities.dedup();

    if lang_dirty {
        dirty_entities.extend(query_all.iter().map(|(e, _)| e));
        dirty_entities.sort();
        dirty_entities.dedup();
    }

    if dirty_entities.is_empty() {
        return;
    }

    for entity in dirty_entities {
        let Ok((_entity, html)) = query_all.get(entity) else {
            continue;
        };

        let Some(html_asset) = html_assets.get(&html.handle) else {
            // Asset isn't ready yet (common on WASM). Mark entity for retry.
            commands.entity(entity).insert(PendingHtmlParse);
            continue;
        };

        // Asset is ready -> ensure we don't keep a retry flag.
        commands.entity(entity).remove::<PendingHtmlParse>();

        let content = html_asset.html.clone();
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
            &lang_vars,
        );
        let document = if localized == content {
            raw_document
        } else {
            kuchiki::parse_html().one(localized.clone())
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
                let resolved = resolve_relative_asset_path(&html.get_source_path(), &raw_href);

                Some(asset_server.load::<CssAsset>(resolved))
            })
            .collect();

        css_handles = with_default_css_first(&default_css, css_handles);
        #[cfg(feature = "providers")]
        if let Some(provider_registry) = provider_registry.as_ref() {
            let mut provider_css = resolve_provider_css_handles(
                &localized,
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

        debug!("Create UI for HTML with key [{:?}]", ui_key);
        if !meta_controller.is_empty() {
            debug!("UI controller [{:?}]", meta_controller);
        }

        let label_map = collect_labels_by_for(&body_node);

        if let Some(body_widget) =
            parse_html_node(&body_node, &css_handles, &label_map, &ui_key, html)
        {
            structure_map.html_map.insert(ui_key, vec![body_widget]);

            // IMPORTANT: Explicitly mark UI as dirty so the builder rebuilds.
            html_dirty.0 = true;
        } else {
            error!("Failed to parse <body> node.");
        }
    }
}

/// Parses a DOM node into HtmlWidgetNode.
fn parse_html_node(
    node: &NodeRef,
    css_sources: &Vec<Handle<CssAsset>>,
    label_map: &HashMap<String, String>,
    key: &String,
    html: &HtmlSource,
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

    match tag.as_str() {
        "body" => {
            let children = node
                .children()
                .filter_map(|child| parse_html_node(&child, css_sources, label_map, key, html))
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
            ))
        }

        "div" => {
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_sources, label_map, key, html) {
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
                HtmlID::default(),
            ))
        }

        "form" => {
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_sources, label_map, key, html) {
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
                HtmlID::default(),
            ))
        }

        #[cfg(feature = "extended-dialog")]
        "dialog" => {
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_sources, label_map, key, html) {
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
                HtmlID::default(),
            ))
        }

        #[cfg(feature = "extended-dialog")]
        "dialog-header" => {
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_sources, label_map, key, html) {
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
                HtmlID::default(),
            ))
        }

        #[cfg(feature = "extended-dialog")]
        "dialog-body" => {
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_sources, label_map, key, html) {
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
                HtmlID::default(),
            ))
        }

        #[cfg(feature = "extended-dialog")]
        "dialog-footer" => {
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_sources, label_map, key, html) {
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
                HtmlID::default(),
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
                HtmlID::default(),
            ))
        }

        "fieldset" => {
            let allow_none =
                attributes.get("allow-none").map(|s| s.to_string()) == Some("true".to_string());
            let mode = attributes.get("mode").unwrap_or("single").to_string();
            let mut children = Vec::new();

            let mut radio_nodes: Vec<(NodeRef, bool)> = Vec::new();
            for child in node.children() {
                if let Some(el) = child.as_element() {
                    if el.name.local.eq("radio") {
                        let selected_attr = el.attributes.borrow().contains("selected");
                        radio_nodes.push((child.clone(), selected_attr));
                        continue;
                    }
                }
                if let Some(parsed) = parse_html_node(&child, css_sources, label_map, key, html) {
                    children.push(parsed);
                }
            }

            let any_selected_attr = radio_nodes.iter().any(|(_, sel)| *sel);
            let mut selected_used = false;
            let mut first_radio_seen = false;

            for (radio_node, had_selected_attr) in radio_nodes {
                let element = radio_node.as_element().unwrap();
                let attrs = element.attributes.borrow();

                let value = attrs.get("value").unwrap_or("").to_string();
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
                    HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                HtmlID::default(),
            ))
        }

        "radio" => {
            let value = attributes.get("value").unwrap_or("").to_string();
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
                HtmlID::default(),
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
                HtmlID::default(),
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
                        let icon = attrs.get("icon").unwrap_or("").to_string();
                        let text = child.text_contents().trim().to_string();

                        let icon_path = if icon.trim().is_empty() {
                            None
                        } else {
                            Some(icon)
                        };

                        let option = ChoiceOption {
                            text: text.clone(),
                            internal_value: value.clone(),
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
                HtmlID::default(),
            ))
        }

        "slider" => {
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

            let step = attributes
                .get("step")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(1.0);

            Some(HtmlWidgetNode::Slider(
                Slider {
                    value,
                    min,
                    max,
                    step,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                HtmlID::default(),
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

            Some(HtmlWidgetNode::SwitchButton(
                SwitchButton {
                    label: text,
                    icon,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                HtmlID::default(),
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
                    value,
                    icon_place,
                    selected: selected_attr,
                    ..default()
                },
                meta,
                states,
                functions,
                widget.clone(),
                HtmlID::default(),
            ))
        }

        _ => None,
    }
}

/// Extracts HTML event bindings from element attributes.
fn bind_html_func(attributes: &Attributes) -> HtmlEventBindings {
    HtmlEventBindings {
        onclick: attributes.get("onclick").map(|s| s.to_string()),
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
        onkeydown: attributes.get("onkeydown").map(|s| s.to_string()),
        onkeyup: attributes.get("onkeyup").map(|s| s.to_string()),
        ondragstart: attributes.get("ondragstart").map(|s| s.to_string()),
        ondrag: attributes.get("ondrag").map(|s| s.to_string()),
        ondragstop: attributes
            .get("ondragstop")
            .or_else(|| attributes.get("ondragend"))
            .map(|s| s.to_string()),
    }
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
    let Some(value) = attributes.get(key) else {
        return false;
    };

    let normalized = value.trim().to_ascii_lowercase();
    normalized.is_empty() || normalized == "true"
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

    let selected = document.select("body").ok()?;
    for body in selected {
        let node = body.as_node().clone();
        let score = node.descendants().count();
        if score >= best_score {
            best_score = score;
            best = Some(node);
        }
    }

    best
}

/// Resolves a CSS href found inside an HTML document to a path that the AssetServer understands.
pub fn resolve_relative_asset_path(html_path: &str, href: &str) -> String {
    let mut href = href.replace('\\', "/");

    if let Some(rest) = href.strip_prefix("assets/") {
        href = rest.to_string();
    }

    if let Some(rest) = href.strip_prefix('/') {
        return rest.to_string();
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

#[cfg(feature = "providers")]
#[derive(Debug)]
struct ProviderMatch {
    attributes: HashMap<String, String>,
    inner_html: String,
    in_head: bool,
}

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
fn parse_inner_content(node: &NodeRef) -> HtmlInnerContent {
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
fn extract_inner_bindings(content: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_inner_bindings_returns_unique_placeholders() {
        let src = "<p>{{user.name}} {{ user.name }} {{ user.name }} {{user.id}}</p>";
        let bindings = extract_inner_bindings(src);

        assert_eq!(
            bindings,
            vec![
                "{{user.name}}".to_string(),
                "{{ user.name }}".to_string(),
                "{{user.id}}".to_string()
            ]
        );
    }

    #[test]
    fn parse_inner_content_collects_text_html_and_bindings() {
        let doc = kuchiki::parse_html().one("<p>Hello <b>{{ user.name }}</b>!</p>");
        let node = doc.select_first("p").unwrap();
        let content = parse_inner_content(node.as_node());

        assert!(content.inner_text().contains("Hello"));
        assert!(content.inner_html().contains("<b>{{ user.name }}</b>"));
        assert_eq!(content.inner_bindings(), &["{{ user.name }}".to_string()]);
    }
}
