use bevy::prelude::*;
use crate::html::{HtmlMeta, HtmlStructureMap, HtmlWidgetNode};
use crate::styling::convert::{CssClass, CssID, CssSource};

/// A plugin that spawns Bevy UI entities from parsed HTML node structures.
pub struct HtmlBuilderSystem;

impl Plugin for HtmlBuilderSystem {
    /// Registers the HTML builder system to run whenever the HTML structure map resource changes.
    fn build(&self, app: &mut App) {
        app.add_systems(Update, build_html_source.run_if(resource_changed::<HtmlStructureMap>));
    }
}

/// System that builds the active HTML structure into Bevy UI entities.
///
/// This system is triggered when the [`HtmlStructureMap`] resource changes.
/// It looks up the active structure and recursively spawns UI entities for each node.
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

/// Recursively spawns Bevy entities for a given [`HtmlWidgetNode`] and its children.
///
/// Each entity is assigned UI components as well as metadata components like `CssClass` and `CssID`.
///
/// If a parent entity is provided, the new entity is added as a child of that parent.
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
        HtmlWidgetNode::Headline(headline, meta) => {
            spawn_with_meta(commands, headline.clone(), meta)
        }
        HtmlWidgetNode::Paragraph(paragraph, meta) => {
            spawn_with_meta(commands, paragraph.clone(), meta)
        }
        HtmlWidgetNode::CheckBox(checkbox, meta) => {
            spawn_with_meta(commands, checkbox.clone(), meta)
        }
        HtmlWidgetNode::ChoiceBox(choice_box, meta) => {
            spawn_with_meta(commands, choice_box.clone(), meta)
        }
        HtmlWidgetNode::Slider(slider, meta) => {
            spawn_with_meta(commands, slider.clone(), meta)
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

/// Spawns a single UI entity with metadata components based on the provided [`HtmlMeta`].
///
/// The entity will include a [`Node`] component, as well as any CSS metadata like `class`, `id`,
/// and raw CSS source.
///
/// # Type Parameters
/// - `T`: A Bevy component representing a UI element (e.g., `Button`, `Div`, etc.).
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
