use bevy::prelude::*;

use crate::html::{
    AllWidgetsSpawned, HtmlDirty, HtmlEventBindings, HtmlID, HtmlMeta, HtmlStates, HtmlStructureMap,
    HtmlWidgetNode, NeedHidden, ShowWidgetsTimer,
};
use crate::styles::{CssClass, CssID, CssSource};
use crate::widgets::{Body, UIWidgetState, Widget};

/// Plugin that spawns Bevy UI entities from parsed HTML node structures.
pub struct HtmlBuilderSystem;

impl Plugin for HtmlBuilderSystem {
    fn build(&self, app: &mut App) {
        app.add_message::<AllWidgetsSpawned>();
        app.insert_resource(ShowWidgetsTimer::default());

        // Do NOT rely on resource_changed<HtmlStructureMap>().
        // Use an explicit dirty flag instead.
        app.add_systems(Update, build_html_source);
        app.add_systems(Update, show_all_widgets_start.after(build_html_source));
        app.add_systems(Update, show_all_widgets_finish.after(show_all_widgets_start));
    }
}

/// Builds the active HTML structure into Bevy UI entities.
///
/// Runs when HtmlDirty is set. On rebuild, it despawns the old active Body tree
/// and spawns a fresh one from HtmlStructureMap.
pub fn build_html_source(
    mut commands: Commands,
    structure_map: Res<HtmlStructureMap>,
    mut html_dirty: ResMut<HtmlDirty>,
    asset_server: Res<AssetServer>,
    mut event_writer: MessageWriter<AllWidgetsSpawned>,
    body_query: Query<(Entity, &Body)>,
) {
    // Only rebuild if marked dirty.
    if !html_dirty.0 {
        return;
    }
    html_dirty.0 = false;

    let Some(active) = structure_map.active.clone() else {
        return;
    };

    // Despawn the old active UI (recursive).
    for (entity, body) in body_query.iter() {
        if body.html_key.as_deref() == Some(active.as_str()) {
            commands.entity(entity).despawn();
        }
    }

    // Spawn the new active UI from the structure map.
    spawn_structure_for_active(
        &mut commands,
        &active,
        &structure_map,
        &asset_server,
        &mut event_writer,
    );
}

fn spawn_structure_for_active(
    commands: &mut Commands,
    active: &str,
    structure_map: &Res<HtmlStructureMap>,
    asset_server: &Res<AssetServer>,
    event_writer: &mut MessageWriter<AllWidgetsSpawned>,
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
    mut events: MessageReader<AllWidgetsSpawned>,
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
    current_body: Query<&Body>,
    structure_map: Res<HtmlStructureMap>,
) {
    if timer.active && timer.timer.tick(time.delta()).is_finished() {
        if let Some(active) = structure_map.active.clone() {
            for body in current_body.iter() {
                if let Some(bind) = body.html_key.clone() {
                    if bind.eq(&active) {
                        if let Some(map_nodes) = structure_map.html_map.get(active.as_str()) {
                            let mut valid_ids = Vec::new();
                            collect_html_ids(map_nodes, &mut valid_ids);

                            for (mut visibility, widget_id) in query.iter_mut() {
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
            | HtmlWidgetNode::CheckBox(_, _, _, _, _, id)
            | HtmlWidgetNode::ChoiceBox(_, _, _, _, _, id)
            | HtmlWidgetNode::Headline(_, _, _, _, _, id)
            | HtmlWidgetNode::Img(_, _, _, _, _, id)
            | HtmlWidgetNode::Input(_, _, _, _, _, id)
            | HtmlWidgetNode::Paragraph(_, _, _, _, _, id)=> {
                ids.push(id.clone());
            }
            HtmlWidgetNode::Body(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
            }
            HtmlWidgetNode::Div(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
            }
        }
    }
}

/// Recursively spawns entities for a HtmlWidgetNode and its children.
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
        HtmlWidgetNode::CheckBox(checkbox, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, checkbox.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::ChoiceBox(choice_box, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, choice_box.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Headline(headline, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, headline.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Img(img, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, img.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Input(input, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, input.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Paragraph(paragraph, meta, states, functions, widget, id) => {
            spawn_with_meta(commands, paragraph.clone(), meta, states, functions, widget, id)
        }
        HtmlWidgetNode::Body(body, meta, states, children, functions, widget, id) => {
            let entity = spawn_with_meta(commands, body.clone(), meta, states, functions, widget, id);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }

        HtmlWidgetNode::Div(div, meta, states, children, functions, widget, id) => {
            let entity = spawn_with_meta(commands, div.clone(), meta, states, functions, widget, id);
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

/// Spawns a single UI entity and attaches metadata components.
fn spawn_with_meta<T: Component>(
    commands: &mut Commands,
    component: T,
    meta: &HtmlMeta,
    states: &HtmlStates,
    functions: &HtmlEventBindings,
    widget: &Widget,
    id: &HtmlID,
) -> Entity {
    let mut ui_state = UIWidgetState::default();
    ui_state.readonly = states.readonly;
    ui_state.disabled = states.disabled;

    let entity = commands
        .spawn((
            component,
            functions.clone(),
            widget.clone(),
            id.clone(),
            Node::default(),
            CssSource(meta.css.clone()),
            CssClass(meta.class.clone().unwrap_or_default()),
            CssID(meta.id.clone().unwrap_or_default()),
            ui_state,
            Visibility::Hidden,
        ))
        .id();

    if states.hidden {
        commands.entity(entity).insert(NeedHidden);
    }

    entity
}
