use std::collections::HashMap;
use std::fs;
use bevy::prelude::*;
use kuchiki::NodeRef;
use kuchiki::traits::TendrilSink;
use crate::html::{HtmlSource, HtmlMeta, HtmlStructureMap, HtmlWidgetNode, HtmlEventBindings};
use crate::styling::IconPlace;
use crate::widgets::{Button, CheckBox, ChoiceBox, ChoiceOption, Div, Headline, HeadlineType, HtmlBody, Img, InputCap, InputField, InputType, Paragraph, Slider};

/// Plugin that adds a system to convert raw HTML files into Bevy UI entity trees.
pub struct HtmlConverterSystem;

impl Plugin for HtmlConverterSystem {
    /// Registers the HTML update system on Bevy's `Update` stage.
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_html_ui);
    }
}

/// System that reads updated or newly added [`HtmlSource`] components,
/// parses the corresponding HTML file, and converts it into UI widget nodes.
///
/// It extracts metadata, CSS references, and builds a hierarchical
/// [`HtmlWidgetNode`] tree inserted into the global [`HtmlStructureMap`].
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

        // Extract a unique key from <meta name="..."> inside <head>
        let Some(meta_key) = document
            .select_first("head meta[name]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("name").map(|s| s.to_string()))
        else {
            error!("Missing <meta name=...> tag in <head>");
            continue;
        };
        
        let Some(meta_controller) = document
            .select_first("head meta[controller")
            .ok()
            .and_then(|m| m.attributes.borrow().get("controller").map(|s| s.to_string()))
        else {
            error!("Missing <meta name=controller> tag in <head>");
            continue;       
        };

        // Extract CSS source URL from <link href="..."> in <head>, fallback to default
        let css_source = document
            .select_first("head link[href]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("href").map(|s| s.to_string()))
            .unwrap_or_else(|| "assets/css/core.css".to_string());

        structure_map.active = Some(meta_key.clone());

        // Get the <body> element to parse the main UI content
        let Ok(body_node) = document.select_first("body") else {
            error!("Missing <body> tag!");
            continue;
        };

        info!("Create UI for HTML with key [{:?}]", meta_key);
        info!("UI controller [{:?}] try to use...", meta_controller);

        // Collect <label for="..."> mappings for input field labels
        let label_map = collect_labels_by_for(body_node.as_node());

        // Parse the body recursively into the HtmlWidgetNode tree
        if let Some(body_widget) = parse_html_node(
            body_node.as_node(),
            &css_source,
            &label_map,
            &meta_key,
            &meta_controller,
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

/// Recursively parses a Kuchiki DOM node into an [`HtmlWidgetNode`],
/// mapping supported HTML elements into corresponding Bevy widget components.
///
/// # Parameters
/// - `node`: Reference to the current DOM node to parse.
/// - `css_source`: URL or path of the CSS stylesheet for this document.
/// - `label_map`: Map of input IDs to label texts for associating labels with inputs.
/// - `key`: The unique document key extracted from `<meta name=...>`.
/// - `html`: The source [`HtmlSource`] component containing the raw file path.
///
/// # Returns
/// An [`HtmlWidgetNode`] representing the UI widget and its metadata,
/// or `None` if the element is unsupported or parsing failed.
fn parse_html_node(
    node: &NodeRef,
    css_source: &String,
    label_map: &HashMap<String, String>,
    key: &String,
    controller: &String,
    html: &HtmlSource,
) -> Option<HtmlWidgetNode> {
    let element = node.as_element()?;
    let tag = element.name.local.to_string();
    let attributes = element.attributes.borrow();

    // Build HtmlMeta with CSS info and HTML attributes
    let meta = HtmlMeta {
        css: css_source.clone(),
        id: attributes.get("id").map(|s| s.to_string()),
        class: attributes
            .get("class")
            .map(|s| s.split_whitespace().map(str::to_string).collect()),
        style: attributes.get("style").map(|s| s.to_string()),
    };
    
    let functions = HtmlEventBindings {
        onclick: attributes.get("onclick").map(|s| s.to_string()),
        onmouseenter: attributes.get("onmouseenter").map(|s| s.to_string()),
        onmouseleave: attributes.get("onmouseleave").map(|s| s.to_string()),
    };

    match tag.as_str() {
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
            }, meta, functions))
        },
        "input" => {
            // Map <input> to InputField with associated label and attributes
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
            }, meta, functions))
        },
        "checkbox" => {
            // Checkbox with label and optional icon
            let label = node.text_contents().trim().to_string();
            let icon_path = attributes.get("icon").unwrap_or("extended_ui/icons/check-mark.png");
            let icon = Some(String::from(icon_path));
            Some(HtmlWidgetNode::CheckBox(CheckBox {
                label,
                icon_path: icon,
                ..default()
            }, meta, functions))
        },
        "select" => {
            // Parse dropdown options and selected value
            let mut options = Vec::new();
            let mut selected_value = None;

            for child in node.children() {
                if let Some(option_el) = child.as_element() {
                    if option_el.name.local.eq("option") {
                        let attrs = option_el.attributes.borrow();
                        let value = attrs.get("value").unwrap_or("").to_string();
                        let icon = attrs.get("icon").unwrap_or("").to_string();
                        let text = child.text_contents().trim().to_string();

                        let icon_path = if icon.trim().is_empty() { None } else { Some(icon) };

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
                meta, functions))
        },
        "slider" => {
            // Parse slider attributes: min, max, value, step
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
                meta, functions))
        },
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            // Map HTML heading tags to Headline widget with correct level
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
                meta, functions))
        },
        "p" => {
            // Paragraph with text content
            let text = node.text_contents().trim().to_string();
            Some(HtmlWidgetNode::Paragraph(
                Paragraph {
                    text,
                    ..default()
                },
                meta, functions))
        },
        "img" => {
            let src = attributes.get("src").unwrap_or("").to_string();
            let alt = attributes.get("alt").unwrap_or("").to_string();
            
            Some(HtmlWidgetNode::Img(
                Img {
                    src: if src.is_empty() { None } else { Some(String::from(src)) },
                    alt,
                    ..default()
                },
                meta, functions))
        },
        "div" => {
            // Parse children recursively, build Div container
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(parsed) = parse_html_node(&child, css_source, label_map, key, controller, html) {
                    children.push(parsed);
                }
            }
            Some(HtmlWidgetNode::Div(Div::default(), meta, children, functions))
        },
        "body" => {
            // Top-level HtmlBody node with all children parsed
            let children = node.children()
                .filter_map(|child| parse_html_node(&child, css_source, label_map, key, controller, html))
                .collect();

            Some(HtmlWidgetNode::HtmlBody(HtmlBody {
                bind_to_html: Some(key.clone()),
                fn_controller: Some(controller.clone()),
                source: Some(html.clone()),
                ..default()
            }, meta, children, functions))
        }
        // Unsupported or unhandled tags return None
        _ => None,
    }
}

/// Collects mappings from `<label for="id">` attributes to their label text.
///
/// This allows associating form fields with their textual labels during parsing.
///
/// # Arguments
/// - `node`: Root node to search for `<label>` elements.
///
/// # Returns
/// A `HashMap` where keys are the `for` attribute values (input IDs),
/// and values are the trimmed label text contents.
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
