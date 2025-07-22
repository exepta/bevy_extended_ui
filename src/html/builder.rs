use bevy::prelude::*;
use crate::html::{AllWidgetsSpawned, HtmlEventBindings, HtmlID, HtmlMeta, HtmlStates, HtmlStructureMap, HtmlWidgetNode, NeedHidden, ShowWidgetsTimer};
use crate::styling::convert::{CssClass, CssID, CssSource};
use crate::UIWidgetState;
use crate::widgets::{HtmlBody, Widget};

/// A plugin that spawns Bevy UI entities from parsed HTML node structures.
pub struct HtmlBuilderSystem;

impl Plugin for HtmlBuilderSystem {
    /// Registers the HTML builder system to run whenever the HTML structure maps resource changes.
    fn build(&self, app: &mut App) {
        app.add_event::<AllWidgetsSpawned>();
        app.insert_resource(ShowWidgetsTimer::default());
        app.add_systems(Update, build_html_source.run_if(resource_changed::<HtmlStructureMap>));
        app.add_systems(Update, show_all_widgets_start.after(build_html_source));
        app.add_systems(Update, show_all_widgets_finish.after(show_all_widgets_start));
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
    mut event_writer: EventWriter<AllWidgetsSpawned>,
    mut query: Query<(&mut Visibility, &HtmlBody), With<HtmlBody>>,
) {
    if let Some(active) = structure_map.active.clone() {
        if query.is_empty() {
            spawn_structure_for_active(&mut commands, &active, &structure_map, &asset_server, &mut event_writer);
        } else {
            let mut found = false;
            for (mut vis, body) in query.iter_mut() {
                if let Some(bind) = body.bind_to_html.clone() {
                    if bind.eq(&active) {
                        *vis = Visibility::Inherited;
                        event_writer.write(AllWidgetsSpawned);
                        found = true;
                    }
                }
            }

            if !found {
                spawn_structure_for_active(&mut commands, &active, &structure_map, &asset_server, &mut event_writer);
            }
        }
    }
}

fn spawn_structure_for_active(
    commands: &mut Commands,
    active: &str,
    structure_map: &Res<HtmlStructureMap>,
    asset_server: &Res<AssetServer>,
    event_writer: &mut EventWriter<AllWidgetsSpawned>,
) {
    if let Some(structure) = structure_map.html_map.get(active) {
        for node in structure {
            spawn_widget_node(commands, node, asset_server, None);
        }
        event_writer.write(AllWidgetsSpawned);
    } else {
        warn!("No structure found for active: {}", active);
    }
}

fn show_all_widgets_start(
    mut events: EventReader<AllWidgetsSpawned>,
    mut timer: ResMut<ShowWidgetsTimer>,
) {
    for _event in events.read() {
        timer.timer = Timer::from_seconds(0.1, TimerMode::Once);
        timer.active = true;
        debug!("Starting 100ms timer before showing widgets");
    }
}

fn show_all_widgets_finish(
    time: Res<Time>,
    mut timer: ResMut<ShowWidgetsTimer>,
    mut query: Query<(&mut Visibility, &HtmlID), (With<Widget>, Without<NeedHidden>)>,
    current_body: Query<&HtmlBody>,
    structure_map: Res<HtmlStructureMap>,
) {
    if timer.active && timer.timer.tick(time.delta()).finished() {
        if let Some(active) = structure_map.active.clone() {
            for body in current_body.iter() {
                if let Some(bind) = body.bind_to_html.clone() {
                    if bind.eq(&active) {
                        if let Some(map_nodes) = structure_map.html_map.get(active.as_str()) {
                            let mut valid_ids = Vec::new();
                            collect_html_ids(map_nodes, &mut valid_ids);
                            //info!("Valid ids: {:?}", valid_ids);

                            info!("Not too times!");
                            for (mut visibility, widget_id) in query.iter_mut() {
                                //info!("Setting visibility for: {:?}", widget_id);
                                if valid_ids.contains(widget_id) {
                                    *visibility = Visibility::Inherited;
                                }
                            }

                            timer.active = false;
                            debug!("All widgets for '{}' are now visible after 100ms delay", active);
                        }
                    }
                }
            }
        }
    }
}
fn collect_html_ids(nodes: &Vec<HtmlWidgetNode>, ids: &mut Vec<HtmlID>) {
    for node in nodes {
        match node {
            HtmlWidgetNode::Button(_, _, _, _, _, id)
            | HtmlWidgetNode::Input(_, _, _, _, _, id)
            | HtmlWidgetNode::CheckBox(_, _, _, _, _, id)
            | HtmlWidgetNode::ChoiceBox(_, _, _, _, _, id)
            | HtmlWidgetNode::Img(_, _, _, _, _, id)
            | HtmlWidgetNode::ProgressBar(_, _, _, _, _, id)
            | HtmlWidgetNode::Headline(_, _, _, _, _, id)
            | HtmlWidgetNode::Paragraph(_, _, _, _, _, id)
            | HtmlWidgetNode::Slider(_, _, _, _, _, id) => {
                ids.push(id.clone());
            }
            HtmlWidgetNode::Div(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
            }
            HtmlWidgetNode::HtmlBody(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
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
        HtmlWidgetNode::Button(button, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, button.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Input(input, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, input.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Headline(headline, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, headline.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Img(img, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, img.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::ProgressBar(progress_bar, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, progress_bar.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Paragraph(paragraph, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, paragraph.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::CheckBox(checkbox, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, checkbox.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::ChoiceBox(choice_box, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, choice_box.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Slider(slider, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, slider.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Div(div, meta, states, children, functions,widget, id) => {
            let entity = spawn_with_meta(commands, div.clone(), meta, states, functions, widget, id);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
        HtmlWidgetNode::HtmlBody(body, meta, states, children, functions, widget, id) => {
            let entity = spawn_with_meta(commands, body.clone(), meta, states, functions, widget, id);
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
    states: &HtmlStates,
    functions: &HtmlEventBindings,
    widget: &Widget,
    id: &HtmlID
) -> Entity {
    let mut ui_state = UIWidgetState::default();
    
    
    ui_state.readonly = states.readonly;
    ui_state.disabled = states.disabled;
    
    let entity = commands.spawn((
        component,
        functions.clone(),
        widget.clone(),
        id.clone(),
        Node::default(),
        CssSource(meta.css.clone()),
        CssClass(meta.class.clone().unwrap_or_default()),
        CssID(meta.id.clone().unwrap_or_default()),
        ui_state,
        Visibility::Hidden
    )).id();
    
    if states.hidden { 
        commands.entity(entity).insert(NeedHidden); 
    }

entity
}
