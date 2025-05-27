use bevy::prelude::*;
use crate::html::{HtmlMeta, HtmlStructureMap, HtmlWidgetNode};
use crate::styling::convert::{CssClass, CssID, CssSource};

pub struct HtmlBuilderSystem;

impl Plugin for HtmlBuilderSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, build_html_source.run_if(resource_changed::<HtmlStructureMap>));
    }
}

fn build_html_source(
    mut commands: Commands,
    structure_map: Res<HtmlStructureMap>,
    asset_server: Res<AssetServer>,
) {
    if let Some(active) = structure_map.active.clone() {
        if let Some(structure) = structure_map.html_map.get(active.as_str()) {
            for node in structure {
                spawn_widget_node(&mut commands, node, &asset_server, None);
            }
        }
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
            spawn_with_meta(commands, button.clone(), meta)
        }
        HtmlWidgetNode::Input(input, meta) => {
            spawn_with_meta(commands, input.clone(), meta)
        }
        HtmlWidgetNode::CheckBox(checkbox, meta) => {
            spawn_with_meta(commands, checkbox.clone(), meta)
        }
        HtmlWidgetNode::Div(div, meta, children) => {
            let entity = spawn_with_meta(commands, div.clone(), meta);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
        HtmlWidgetNode::HtmlBody(body, meta, children) => {
            let entity = spawn_with_meta(commands, body.clone(), meta);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
    };

    if let Some(parent) = parent {
        commands.entity(parent).add_child(entity);
    }

    entity
}

fn spawn_with_meta<T: Component>(
    commands: &mut Commands,
    component: T,
    meta: &HtmlMeta,
) -> Entity {
    commands.spawn((
        component,
        Node::default(),
        CssSource(meta.css.clone()),
        CssClass(meta.class.clone().unwrap_or_default()),
        CssID(meta.id.clone().unwrap_or_default()),
    )).id()
}