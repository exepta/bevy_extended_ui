use bevy::prelude::*;
use crate::html::{HtmlEventBindings, HtmlMeta, HtmlStructureMap, HtmlWidgetNode};
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
        HtmlWidgetNode::Button(button, meta, functions) => {
            spawn_with_meta(commands, button.clone(), meta, functions)
        }
        HtmlWidgetNode::Input(input, meta, functions) => {
            spawn_with_meta(commands, input.clone(), meta, functions)
        }
        HtmlWidgetNode::Headline(headline, meta, functions) => {
            spawn_with_meta(commands, headline.clone(), meta, functions)
        }
        HtmlWidgetNode::Img(img, meta, functions) => {
            spawn_with_meta(commands, img.clone(), meta, functions)
        }
        HtmlWidgetNode::ProgressBar(progress_bar, meta, functions) => {
            spawn_with_meta(commands, progress_bar.clone(), meta, functions)
        }
        HtmlWidgetNode::Paragraph(paragraph, meta, functions) => {
            spawn_with_meta(commands, paragraph.clone(), meta, functions)
        }
        HtmlWidgetNode::CheckBox(checkbox, meta, functions) => {
            spawn_with_meta(commands, checkbox.clone(), meta, functions)
        }
        HtmlWidgetNode::ChoiceBox(choice_box, meta, functions) => {
            spawn_with_meta(commands, choice_box.clone(), meta, functions)
        }
        HtmlWidgetNode::Slider(slider, meta, functions) => {
            spawn_with_meta(commands, slider.clone(), meta, functions)
        }
        HtmlWidgetNode::Div(div, meta, children, functions) => {
            let entity = spawn_with_meta(commands, div.clone(), meta, functions);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
        HtmlWidgetNode::HtmlBody(body, meta, children, functions) => {
            let entity = spawn_with_meta(commands, body.clone(), meta, functions);
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
    functions: &HtmlEventBindings
) -> Entity {
    commands.spawn((
        component,
        functions.clone(),
        Node::default(),
        CssSource(meta.css.clone()),
        CssClass(meta.class.clone().unwrap_or_default()),
        CssID(meta.id.clone().unwrap_or_default()),
    )).id()
}
