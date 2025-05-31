use std::collections::HashMap;
use std::fs;
use bevy::prelude::*;
use kuchiki::NodeRef;
use kuchiki::traits::TendrilSink;
use crate::html::{HtmlSource, HtmlMeta, HtmlStructureMap, HtmlWidgetNode};
use crate::styling::IconPlace;
use crate::widgets::{Button, CheckBox, ChoiceBox, ChoiceOption, Div, Headline, HeadlineType, HtmlBody, InputCap, InputField, InputType, Slider};

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
            let mut icon_path = None;
            let mut icon_place = IconPlace::Left;
            let mut found_text = false;

            for child in node.children() {
                if let Some(el) = child.as_element() {
                    if el.name.local.eq("img") {
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
            let icon_path = attributes.get("icon").unwrap_or("");
            let icon: Option<String> = if !icon_path.is_empty() { Some(String::from(icon_path)) } else { None };
            let cap = match attributes.get("maxlength") {
                Some(value) if value.trim().eq_ignore_ascii_case("auto") => InputCap::CapAtNodeSize,
                Some(value) if value.trim().is_empty() => InputCap::NoCap,
                Some(value) => {
                    let length = value.trim().parse::<usize>().unwrap_or(0);
                    InputCap::CapAt(length)
                }
                None => InputCap::NoCap,
            };

            Some(HtmlWidgetNode::Input(InputField {
                label,
                placeholder,
                text,
                input_type: InputType::from_str(&input_type).unwrap_or_default(),
                icon_path: icon,
                cap_text_at: cap,
                ..default()
            }, meta))
        },
        "checkbox" => {
            let label = node.text_contents().trim().to_string();
            let icon_path = attributes.get("icon").unwrap_or("icons/check-mark.png");
            let icon = Some(String::from(icon_path));
            Some(HtmlWidgetNode::CheckBox(CheckBox {
                label,
                icon_path: icon,
                ..default()           
            }, meta))
        },
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

                        let icon_path;
                        if icon.trim().is_empty() {
                            icon_path = None;
                        } else {
                            icon_path = Some(icon);
                        }
                        
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
            
            let value = selected_value.unwrap_or_else(|| {
                options.first().cloned().unwrap_or_default()
            });
            

            Some(HtmlWidgetNode::ChoiceBox(
                ChoiceBox {
                    value,
                    options,
                    ..default()
                },
                meta,
            ))
        },
        "slider" => {
            let min = attributes
                .get("min")
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(0);

            let max = attributes
                .get("max")
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(100);

            let value = attributes
                .get("value")
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(min);

            let step = attributes
                .get("step")
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(1);

            Some(HtmlWidgetNode::Slider(
                Slider {
                    value,
                    min,
                    max,
                    step,
                    ..default()
                },
                meta,
            ))
        },
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
            ))
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