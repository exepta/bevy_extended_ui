use std::collections::HashMap;
use std::fs;
use bevy::prelude::*;
use kuchiki::NodeRef;
use kuchiki::traits::TendrilSink;
use crate::html::{HtmlSource, HtmlMeta, HtmlStructureMap, HtmlWidgetNode};
use crate::widgets::{Button, CheckBox, Div, HtmlBody, InputField, InputType};

pub struct HtmlConverterSystem;

impl Plugin for HtmlConverterSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_html_ui);
    }
}

fn update_html_ui(
    mut structure_map: ResMut<HtmlStructureMap>,
    query: Query<&HtmlSource, Or<(Changed<HtmlSource>, Added<HtmlSource>)>>,
) {
    for html in query.iter() {
        let Ok(content) = fs::read_to_string(&html.0) else {
            warn!("Failed to read HTML file: {:?}", html.0);
            continue;
        };

        let document = kuchiki::parse_html().one(content);

        let Some(meta_key) = document
            .select_first("head meta[name]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("name").map(|s| s.to_string()))
        else {
            error!("Missing <meta name=...> tag in <head>");
            continue;
        };

        let css_source = document
            .select_first("head link[href]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("href").map(|s| s.to_string()))
            .unwrap_or_else(|| "assets/css/core.css".to_string());

        structure_map.active = Some(meta_key.clone());

        let Ok(body_node) = document.select_first("body") else {
            error!("Missing <body> tag!");
            continue;
        };

        info!("Create UI for HTML with key [{:?}]", meta_key);

        let label_map = collect_labels_by_for(body_node.as_node());

        if let Some(body_widget) = parse_html_node(
            body_node.as_node(),
            &css_source,
            &label_map,
            &meta_key,
            &html,
        ) {
            structure_map
                .html_map
                .insert(meta_key, vec![body_widget]);
        } else {
            error!("Failed to parse <body> node.");
        }
    }
}

fn parse_html_node(
    node: &NodeRef, 
    css_source: &String, 
    label_map: &HashMap<String, String>,
    key: &String,
    html: &HtmlSource,
) -> Option<HtmlWidgetNode> {
    let element = node.as_element()?;
    let tag = element.name.local.to_string();
    let attributes = element.attributes.borrow();

    let meta = HtmlMeta {
        css: css_source.clone(),
        id: attributes.get("id").map(|s| s.to_string()),
        class: attributes
            .get("class")
            .map(|s| s.split_whitespace().map(str::to_string).collect()),
        style: attributes.get("style").map(|s| s.to_string()),
    };

    match tag.as_str() {
        "button" => {
            let text = node.text_contents().trim().to_string();
            Some(HtmlWidgetNode::Button(Button {
                text,
                ..default()
            }, meta))
        },
        "input" => {
            let id = attributes.get("id").map(|s| s.to_string());
            let label = id
                .as_ref()
                .and_then(|id| label_map.get(id))
                .cloned()
                .unwrap_or_default();
            
            let text = attributes.get("value").unwrap_or("").to_string();
            let placeholder = attributes.get("placeholder").unwrap_or("").to_string();
            let input_type = attributes.get("type").unwrap_or("text").to_string();

            Some(HtmlWidgetNode::Input(InputField {
                label,
                placeholder,
                text,
                input_type: InputType::from_str(&input_type).unwrap_or_default(),
                ..default()
            }, meta))
        },
        "checkbox" => {
            let label = node.text_contents().trim().to_string();
            Some(HtmlWidgetNode::CheckBox(CheckBox {
                label,
                ..default()
            }, meta))
        },
        "div" => {
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_source, label_map, key, html) {
                    children.push(parsed);
                }
            }
            Some(HtmlWidgetNode::Div(Div::default(), meta, children))
        },
        "body" => {
            let children = node.children()
                .filter_map(|child| parse_html_node(&child, css_source, label_map, key, html))
                .collect();

            Some(HtmlWidgetNode::HtmlBody(HtmlBody {
                bind_to_html: Some(key.clone()),
                source: Some(html.clone()),
                ..default()           
            }, meta, children))
        }
        _ => None,
    }
}

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