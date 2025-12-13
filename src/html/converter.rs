use std::collections::HashMap;

use bevy::asset::AssetEvent;
use bevy::prelude::*;
use kuchiki::{traits::TendrilSink, NodeRef};

use crate::html::{
    HtmlDirty, HtmlEventBindings, HtmlID, HtmlMeta, HtmlSource, HtmlStates, HtmlStructureMap,
    HtmlWidgetNode,
};
use crate::io::{CssAsset, HtmlAsset};
use crate::styles::IconPlace;
use crate::widgets::{Body, Button, Div, Widget};

pub const DEFAULT_UI_CSS: &str = "default/extended_ui.css";

pub struct HtmlConverterSystem;

impl Plugin for HtmlConverterSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_html_ui);
    }
}

/// Converts HtmlAsset content into HtmlStructureMap entries.
/// Also resolves <link rel="stylesheet" href="..."> into Handle<CssAsset>.
fn update_html_ui(
    mut structure_map: ResMut<HtmlStructureMap>,
    mut html_dirty: ResMut<HtmlDirty>,

    asset_server: Res<AssetServer>,
    html_assets: Res<Assets<HtmlAsset>>,
    mut html_asset_events: MessageReader<AssetEvent<HtmlAsset>>,

    query_added: Query<(Entity, &HtmlSource), Added<HtmlSource>>,
    query_all: Query<(Entity, &HtmlSource)>,
) {
    // Entities that need reparse (new HtmlSource or modified HtmlAsset).
    let mut dirty_entities: Vec<Entity> = query_added.iter().map(|(e, _)| e).collect();

    // If an HtmlAsset changed, find all entities referencing it.
    for ev in html_asset_events.read() {
        let id = match ev {
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => *id,
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

    if dirty_entities.is_empty() {
        return;
    }

    for entity in dirty_entities {
        let Ok((_entity, html)) = query_all.get(entity) else { continue };

        let Some(html_asset) = html_assets.get(&html.handle) else {
            // Asset isn't ready yet.
            continue;
        };

        let content = html_asset.html.clone();
        let document = kuchiki::parse_html().one(content);

        // Extract unique UI key from <meta name="...">
        let Some(meta_key) = document
            .select_first("head meta[name]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("name").map(|s| s.to_string()))
        else {
            error!("Missing <meta name=...> tag in <head>");
            continue;
        };

        // Optional controller path from <meta controller="...">
        let meta_controller = document
            .select_first("head meta[controller]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("controller").map(|s| s.to_string()))
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
                let resolved = resolve_relative_path(&html.get_source_path(), &raw_href);

                Some(asset_server.load::<CssAsset>(resolved))
            })
            .collect();

        css_handles = with_default_css_first(&asset_server, css_handles);

        // Mark this UI as active.
        structure_map.active = Some(meta_key.clone());

        // Parse body
        let Ok(body_node) = document.select_first("body") else {
            error!("Missing <body> tag!");
            continue;
        };

        debug!("Create UI for HTML with key [{:?}]", meta_key);
        if !meta_controller.is_empty() {
            debug!("UI controller [{:?}]", meta_controller);
        }

        let label_map = collect_labels_by_for(body_node.as_node());

        if let Some(body_widget) = parse_html_node(
            body_node.as_node(),
            &css_handles,
            &label_map,
            &meta_key,
            html,
        ) {
            structure_map.html_map.insert(meta_key, vec![body_widget]);

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
    _label_map: &HashMap<String, String>,
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
        style: attributes.get("style").map(|s| s.to_string()),
    };

    let states = HtmlStates {
        disabled: attributes.contains("disabled"),
        readonly: attributes.contains("readonly"),
        hidden: attributes.contains("hidden"),
    };

    let functions = HtmlEventBindings {
        onclick: attributes.get("onclick").map(|s| s.to_string()),
        onmouseenter: attributes.get("onmouseenter").map(|s| s.to_string()),
        onmouseleave: attributes.get("onmouseleave").map(|s| s.to_string()),
        onupdate: attributes.get("onupdate").map(|s| s.to_string()),
        onload: attributes.get("onload").map(|s| s.to_string()),
    };

    let widget = Widget(html.controller.clone());

    match tag.as_str() {
        "body" => {
            let children = node
                .children()
                .filter_map(|child| parse_html_node(&child, css_sources, _label_map, key, html))
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
            // Parse button text and optional icon
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
                } else if child.as_text().map(|t| !t.borrow().trim().is_empty()).unwrap_or(false) {
                    found_text = true;
                }
            }

            let text = node.text_contents().trim().to_string();
            Some(HtmlWidgetNode::Button(Button {
                text,
                icon_path,
                icon_place,
                ..default()
            }, meta, states, functions, widget.clone(), HtmlID::default()))
        }
        "div" => {
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_sources, _label_map, key, html) {
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
        _ => None,
    }
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

/// Resolves a CSS href found inside an HTML document to a path that the AssetServer understands.
fn resolve_relative_path(html_path: &str, href: &str) -> String {
    let mut href = href.replace('\\', "/");

    // If a user wrote "assets/..." strip it because AssetServer already roots at assets/.
    if let Some(rest) = href.strip_prefix("assets/") {
        href = rest.to_string();
    }

    // "/..." means "absolute within assets/"
    if let Some(rest) = href.strip_prefix('/') {
        return rest.to_string();
    }

    // If it already looks like assets-root relative (contains folders) and is not ./ ../, keep it.
    if href.contains('/') && !href.starts_with("./") && !href.starts_with("../") {
        return href;
    }

    // Otherwise resolve relative to the folder of the HTML file.
    let base = std::path::Path::new(html_path)
        .parent()
        .unwrap_or(std::path::Path::new(""));

    base.join(href)
        .to_string_lossy()
        .replace('\\', "/")
}

fn with_default_css_first(
    asset_server: &AssetServer,
    mut css: Vec<Handle<CssAsset>>,
) -> Vec<Handle<CssAsset>> {
    let default = asset_server.load::<CssAsset>(DEFAULT_UI_CSS);

    // Remove default if it already exists somewhere
    css.retain(|h| h.id() != default.id());

    // Insert default at position 0
    css.insert(0, default);

    css
}