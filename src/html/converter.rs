use std::fs;
use bevy::prelude::*;
use kuchiki::NodeRef;
use kuchiki::traits::TendrilSink;
use crate::html::{Html, HtmlMeta, HtmlStructureMap, HtmlWidgetNode};
use crate::styling::convert::{CssClass, CssID, CssSource};
use crate::widgets::{Button, CheckBox, Div, InputField, InputType};

pub struct HtmlConverterSystem;

impl Plugin for HtmlConverterSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_html_ui, 
            test_place
                .run_if(resource_changed::<HtmlStructureMap>)
        ).chain());
    }
}

fn test_place(    
    mut commands: Commands,
    structure_map: Res<HtmlStructureMap>,
    asset_server: Res<AssetServer>,
) {
    if let Some(structure) = structure_map.0.get("test") {
        for node in structure {
            spawn_widget_node(&mut commands, node, &asset_server, None);
        }
    }
}

fn update_html_ui(   
    mut structure_map: ResMut<HtmlStructureMap>,
    query: Query<&Html, Or<(Changed<Html>, Added<Html>)>>
) {
    for html in query.iter() {
        let Ok(content) = fs::read_to_string(&html.0) else {
            warn!("Failed to read html file");
            continue;
        };
        
        let document = kuchiki::parse_html().one(content);

        let meta_key = document.select_first("head meta[name]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("name").map(|s| s.to_string()));

        let css_link = document.select_first("head link[href]")
            .ok()
            .and_then(|m| m.attributes.borrow().get("href").map(|s| s.to_string()));

        let Some(key) = meta_key else {
            error!("Missing <meta name=...> tag in head!");
            continue;
        };

        let css_source = css_link.unwrap_or("assets/css/core.css".to_string());
        
        if let Ok(body) = document.select_first("body") {
            info!("Create UI for id [{:?}]", key);
            let mut root_children = Vec::new();
            for child in body.as_node().children() {
                if let Some(node) = parse_html_node(&child, &css_source) {
                    root_children.push(node);
                }
            }

            structure_map.0.insert(key, root_children);
        } else {
            error!("Failed to find body tag, this absolute required!");
        }
    }
}

fn parse_html_node(node: &NodeRef, css_source: &String) -> Option<HtmlWidgetNode> {
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
            let label = attributes.get("label").unwrap_or("").to_string();
            let text = attributes.get("text").unwrap_or("").to_string();
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
                if let Some(parsed) = parse_html_node(&child, css_source) {
                    children.push(parsed);
                }
            }
            Some(HtmlWidgetNode::Div(Div::default(), meta, children))
        }
        _ => None,
    }
}

fn spawn_widget_node(
    commands: &mut Commands,
    node: &HtmlWidgetNode,
    asset_server: &AssetServer,
    parent: Option<Entity>,
) -> Entity {
    let entity = match node {
        HtmlWidgetNode::Button(button, meta) => {
            commands.spawn((
                button.clone(),
                CssSource(meta.css.clone()),
                CssClass(meta.class.clone().unwrap_or_default()),
                CssID(meta.id.clone().unwrap_or_default()),
            )).id()
        }

        HtmlWidgetNode::Input(input, meta) => {
            commands.spawn((
                input.clone(),
                CssSource(meta.css.clone()),
                CssClass(meta.class.clone().unwrap_or_default()),
                CssID(meta.id.clone().unwrap_or_default()),
            )).id()
        }

        HtmlWidgetNode::CheckBox(checkbox, meta) => {
            commands.spawn((
                checkbox.clone(),
                CssSource(meta.css.clone()),
                CssClass(meta.class.clone().unwrap_or_default()),
                CssID(meta.id.clone().unwrap_or_default()),
            )).id()
        }

        HtmlWidgetNode::Div(div, meta, children) => {
            let div_entity = commands.spawn((
                div.clone(),
                CssSource(meta.css.clone()),
                CssClass(meta.class.clone().unwrap_or_default()),
                CssID(meta.id.clone().unwrap_or_default()),
            )).id();

            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(div_entity));
                commands.entity(div_entity).add_child(child_entity);
            }

            div_entity
        }
    };
    
    if let Some(parent) = parent {
        commands.entity(parent).add_child(entity);
    }

    entity
}