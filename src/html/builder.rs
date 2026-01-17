use bevy::prelude::*;

use crate::html::{HtmlAllWidgetsSpawned, HtmlAllWidgetsVisible, HtmlDirty, HtmlEventBindings, HtmlID, HtmlMeta, HtmlStates, HtmlStructureMap, HtmlSystemSet, HtmlWidgetNode, NeedHidden, ShowWidgetsTimer};
use crate::styles::{CssClass, CssID, CssSource};
use crate::widgets::{Body, UIWidgetState, Widget};

/// Plugin that spawns Bevy UI entities from parsed HTML node structures.
pub struct HtmlBuilderSystem;

impl Plugin for HtmlBuilderSystem {
    fn build(&self, app: &mut App) {
        app.add_message::<HtmlAllWidgetsSpawned>();
        app.add_message::<HtmlAllWidgetsVisible>();
        app.insert_resource(ShowWidgetsTimer::default());

        // Do NOT rely on resource_changed<HtmlStructureMap>().
        // Use an explicit dirty flag instead.
        app.add_systems(Update, build_html_source.in_set(HtmlSystemSet::Build));
        app.add_systems(
            Update,
            show_all_widgets_start
                .in_set(HtmlSystemSet::ShowWidgets)
                .after(build_html_source),
        );

        app.add_systems(
            Update,
            show_all_widgets_finish
                .in_set(HtmlSystemSet::ShowWidgets)
                .after(show_all_widgets_start),
        );
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
    mut event_writer: MessageWriter<HtmlAllWidgetsSpawned>,
    body_query: Query<(Entity, &Body)>,
) {
    // Only rebuild if marked dirty.
    if !html_dirty.0 {
        return;
    }
    html_dirty.0 = false;

    let Some(active_list) = structure_map.active.as_ref() else {
        return;
    };

    // Despawn the old active UI (recursive).
    for (entity, body) in body_query.iter() {
        if let Some(key) = body.html_key.as_deref() {
            if active_list.iter().any(|active| active == key) {
                commands.entity(entity).despawn();
            }
        }
    }

    for active in active_list {
        spawn_structure_for_active(
            &mut commands,
            active,
            &structure_map,
            &asset_server,
            &mut event_writer,
        );
    }
}

fn spawn_structure_for_active(
    commands: &mut Commands,
    active: &str,
    structure_map: &Res<HtmlStructureMap>,
    asset_server: &Res<AssetServer>,
    event_writer: &mut MessageWriter<HtmlAllWidgetsSpawned>,
) {
    if let Some(structure) = structure_map.html_map.get(active) {
        for node in structure {
            spawn_widget_node(commands, node, asset_server, None);
        }
        event_writer.write(HtmlAllWidgetsSpawned);
    } else {
        warn!("No structure found for active: {}", active);
    }
}

fn show_all_widgets_start(
    mut events: MessageReader<HtmlAllWidgetsSpawned>,
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
    mut event_writer: MessageWriter<HtmlAllWidgetsVisible>,
) {
    if timer.active && timer.timer.tick(time.delta()).is_finished() {
        let Some(active_list) = structure_map.active.as_ref() else {
            return;
        };

        let mut valid_ids = Vec::new();
        for active in active_list {
            if let Some(map_nodes) = structure_map.html_map.get(active.as_str()) {
                collect_html_ids(map_nodes, &mut valid_ids);
            }
        }

        if valid_ids.is_empty() {
            return;
        }

        for body in current_body.iter() {
            if let Some(bind) = body.html_key.as_ref() {
                if active_list.iter().any(|active| active == bind) {
                    for (mut visibility, widget_id) in query.iter_mut() {
                        if valid_ids.contains(widget_id) {
                            *visibility = Visibility::Inherited;
                        }
                    }

                    timer.active = false;
                    event_writer.write(HtmlAllWidgetsVisible);
                    debug!(
                        "All widgets for '{:?}' are now visible after 100ms delay",
                        active_list
                    );
                    break;
                }
            }
        }
    }
}

fn collect_html_ids(nodes: &Vec<HtmlWidgetNode>, ids: &mut Vec<HtmlID>) {
    for node in nodes {
        match node {
            HtmlWidgetNode::Body(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
            }
            HtmlWidgetNode::Button(_, _, _, _, _, id)
            | HtmlWidgetNode::CheckBox(_, _, _, _, _, id)
            | HtmlWidgetNode::ChoiceBox(_, _, _, _, _, id)
            | HtmlWidgetNode::Divider(_, _, _, _, _, id)
            | HtmlWidgetNode::Headline(_, _, _, _, _, id)
            | HtmlWidgetNode::Img(_, _, _, _, _, id)
            | HtmlWidgetNode::Input(_, _, _, _, _, id)
            | HtmlWidgetNode::Paragraph(_, _, _, _, _, id)
            | HtmlWidgetNode::ProgressBar(_, _, _, _, _, id)
            | HtmlWidgetNode::RadioButton(_, _, _, _, _, id)
            | HtmlWidgetNode::Scrollbar(_, _, _, _, _, id)
            | HtmlWidgetNode::Slider(_, _, _, _, _, id)
            | HtmlWidgetNode::SwitchButton(_, _, _, _, _, id)
            | HtmlWidgetNode::ToggleButton(_, _, _, _, _, id)=> {
                ids.push(id.clone());
            }
            HtmlWidgetNode::Div(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
            }
            HtmlWidgetNode::FieldSet(_, _, _, children, _, _, id) => {
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
        HtmlWidgetNode::Body(body, meta, states, children, functions, widget, id) => {
            let entity =
                spawn_with_meta(commands, body.clone(), meta, states, functions, widget, id);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
        HtmlWidgetNode::Button(button, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            button.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::CheckBox(checkbox, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            checkbox.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::ChoiceBox(choice_box, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            choice_box.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::Div(div, meta, states, children, functions, widget, id) => {
            let entity =
                spawn_with_meta(commands, div.clone(), meta, states, functions, widget, id);
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
        HtmlWidgetNode::Divider(divider, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            divider.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::FieldSet(fieldset, meta, states, children, functions, widget, id) => {
            let entity = spawn_with_meta(
                commands,
                fieldset.clone(),
                meta,
                states,
                functions,
                widget,
                id,
            );
            for child in children {
                let child_entity = spawn_widget_node(commands, child, asset_server, Some(entity));
                commands.entity(entity).add_child(child_entity);
            }
            entity
        }
        HtmlWidgetNode::Headline(headline, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            headline.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::Img(img, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            img.clone(),
            meta,
            states,
            functions,
            widget,
            id
        ),
        HtmlWidgetNode::Input(input, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            input.clone(),
            meta,
            states,
            functions,
            widget,
            id
        ),
        HtmlWidgetNode::Paragraph(paragraph, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            paragraph.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::ProgressBar(progress_bar, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            progress_bar.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::RadioButton(radio_button, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            radio_button.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::Scrollbar(scroll_bar, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            scroll_bar.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::Slider(slider, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            slider.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::SwitchButton(switch_button, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            switch_button.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
        HtmlWidgetNode::ToggleButton(toggle_button, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            toggle_button.clone(),
            meta,
            states,
            functions,
            widget,
            id,
        ),
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

    if let Some(inline_style) = &meta.style {
        commands.entity(entity).insert(inline_style.clone());
    }
    
    if states.hidden {
        commands.entity(entity).insert(NeedHidden);
    }

    entity
}
