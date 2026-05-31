use bevy::prelude::*;
use std::collections::HashMap;

use crate::html::{
    HtmlAllWidgetsSpawned, HtmlAllWidgetsVisible, HtmlDirty, HtmlEventBindings, HtmlID, HtmlMeta,
    HtmlStates, HtmlStructureMap, HtmlSystemSet, HtmlWidgetNode, NeedHidden, ShowWidgetsTimer,
};
use crate::styles::{CssClass, CssID, CssSource};
use crate::widgets::body::BodyContentRoot;
use crate::widgets::div::DivContentRoot;
use crate::widgets::{Body, UIWidgetState, Widget};

/// Plugin that spawns Bevy UI entities from parsed HTML node structures.
pub struct HtmlBuilderSystem;

impl Plugin for HtmlBuilderSystem {
    /// Registers systems to build HTML structures into UI entities.
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
    body_query: Query<(Entity, &Body, Option<&HtmlID>)>,
    children_query: Query<&Children>,
    html_id_query: Query<&HtmlID>,
    body_content_root_query: Query<&BodyContentRoot>,
    div_content_root_query: Query<&DivContentRoot>,
) {
    // Only rebuild if marked dirty.
    if !html_dirty.0 {
        return;
    }

    let Some(active_list) = structure_map.active.as_ref() else {
        html_dirty.0 = false;
        html_dirty.1.clear();
        return;
    };

    let rebuild_keys: Vec<String> = if html_dirty.1.is_empty() {
        active_list.clone()
    } else {
        active_list
            .iter()
            .filter(|active| html_dirty.1.contains(*active))
            .cloned()
            .collect()
    };

    html_dirty.0 = false;
    for key in &rebuild_keys {
        html_dirty.1.remove(key);
    }

    if rebuild_keys.is_empty() {
        return;
    }

    for active in &rebuild_keys {
        let mut matching_entities = Vec::new();
        let mut existing_live_root = None;

        for (entity, body, html_id) in body_query.iter() {
            if body.html_key.as_deref() != Some(active.as_str()) {
                continue;
            }

            matching_entities.push(entity);
            if html_id.is_some() {
                existing_live_root = Some(entity);
            }
        }

        if let Some(root) = existing_live_root {
            rebuild_structure_children_for_active(
                &mut commands,
                root,
                active,
                &structure_map,
                &asset_server,
                &children_query,
                &html_id_query,
                &body_content_root_query,
                &div_content_root_query,
            );
            continue;
        }

        for entity in matching_entities {
            commands.entity(entity).despawn();
        }

        spawn_structure_for_active(
            &mut commands,
            active,
            &structure_map,
            &asset_server,
            &mut event_writer,
        );
    }
}

/// Spawns UI nodes for the active HTML key.
fn spawn_structure_for_active(
    commands: &mut Commands,
    active: &str,
    structure_map: &Res<HtmlStructureMap>,
    asset_server: &Res<AssetServer>,
    event_writer: &mut MessageWriter<HtmlAllWidgetsSpawned>,
) {
    if let Some(structure) = structure_map.html_map.get(active) {
        for node in structure {
            spawn_widget_node(commands, node, asset_server, None, true);
        }
        event_writer.write(HtmlAllWidgetsSpawned);
    } else {
        warn!("No structure found for active: {}", active);
    }
}

fn rebuild_structure_children_for_active(
    commands: &mut Commands,
    root: Entity,
    active: &str,
    structure_map: &Res<HtmlStructureMap>,
    asset_server: &Res<AssetServer>,
    children_query: &Query<&Children>,
    html_id_query: &Query<&HtmlID>,
    body_content_root_query: &Query<&BodyContentRoot>,
    div_content_root_query: &Query<&DivContentRoot>,
) {
    let Some(structure) = structure_map.html_map.get(active) else {
        warn!("No structure found for active: {}", active);
        return;
    };

    let Some(HtmlWidgetNode::Body(_, _, _, new_children, _, _, _)) = structure.first() else {
        warn!(
            "No root <body> node found for active '{}' during in-place rebuild",
            active
        );
        return;
    };

    let content_parent = body_content_root_query
        .get(root)
        .map(|content| content.0)
        .unwrap_or(root);

    reconcile_node_children(
        commands,
        content_parent,
        new_children,
        asset_server,
        children_query,
        html_id_query,
        body_content_root_query,
        div_content_root_query,
    );
}

fn reconcile_node_children(
    commands: &mut Commands,
    parent: Entity,
    new_nodes: &Vec<HtmlWidgetNode>,
    asset_server: &AssetServer,
    children_query: &Query<&Children>,
    html_id_query: &Query<&HtmlID>,
    body_content_root_query: &Query<&BodyContentRoot>,
    div_content_root_query: &Query<&DivContentRoot>,
) {
    let existing_children: Vec<Entity> = children_query
        .get(parent)
        .map(|children| children.iter().collect())
        .unwrap_or_default();

    let mut existing_by_id: HashMap<usize, Entity> = HashMap::new();
    let mut existing_without_id: Vec<Entity> = Vec::new();
    for child in &existing_children {
        if let Ok(id) = html_id_query.get(*child) {
            existing_by_id.insert(id.0, *child);
        } else {
            existing_without_id.push(*child);
        }
    }

    let mut final_children: Vec<Entity> =
        Vec::with_capacity(new_nodes.len() + existing_without_id.len());

    for new_node in new_nodes {
        let node_id = get_node_id(new_node).0;
        if let Some(existing_entity) = existing_by_id.remove(&node_id) {
            update_existing_widget_node(commands, existing_entity, new_node, false);

            if let Some(children) = get_node_children(new_node) {
                let nested_parent = resolve_content_parent(
                    existing_entity,
                    body_content_root_query,
                    div_content_root_query,
                );
                reconcile_node_children(
                    commands,
                    nested_parent,
                    children,
                    asset_server,
                    children_query,
                    html_id_query,
                    body_content_root_query,
                    div_content_root_query,
                );
            }

            final_children.push(existing_entity);
        } else {
            let spawned = spawn_widget_node(commands, new_node, asset_server, Some(parent), false);
            final_children.push(spawned);
        }
    }

    for stale in existing_by_id.into_values() {
        commands.entity(stale).despawn();
    }

    final_children.extend(existing_without_id);
    if final_children != existing_children {
        commands.entity(parent).replace_children(&final_children);
    }
}

fn resolve_content_parent(
    entity: Entity,
    body_content_root_query: &Query<&BodyContentRoot>,
    div_content_root_query: &Query<&DivContentRoot>,
) -> Entity {
    if let Ok(content) = body_content_root_query.get(entity) {
        return content.0;
    }
    if let Ok(content) = div_content_root_query.get(entity) {
        return content.0;
    }
    entity
}

fn get_node_children(node: &HtmlWidgetNode) -> Option<&Vec<HtmlWidgetNode>> {
    if let HtmlWidgetNode::Body(_, _, _, children, _, _, _) = node {
        return Some(children);
    }
    if let HtmlWidgetNode::Div(_, _, _, children, _, _, _) = node {
        return Some(children);
    }
    if let HtmlWidgetNode::Form(_, _, _, children, _, _, _) = node {
        return Some(children);
    }
    if let HtmlWidgetNode::FieldSet(_, _, _, children, _, _, _) = node {
        return Some(children);
    }
    #[cfg(feature = "extended-dialog")]
    if let HtmlWidgetNode::Dialog(_, _, _, children, _, _, _) = node {
        return Some(children);
    }
    None
}

fn update_existing_widget_node(
    commands: &mut Commands,
    entity: Entity,
    node: &HtmlWidgetNode,
    start_hidden: bool,
) {
    match node {
        HtmlWidgetNode::Body(body, meta, states, _, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                body.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Button(button, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                button.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::CheckBox(checkbox, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                checkbox.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::ColorPicker(color_picker, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                color_picker.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::ChoiceBox(choice_box, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                choice_box.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::DatePicker(date_picker, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                date_picker.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Div(div, meta, states, _, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                div.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Form(form, meta, states, _, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                form.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        #[cfg(feature = "extended-dialog")]
        HtmlWidgetNode::Dialog(dialog, meta, states, _, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                dialog.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Divider(divider, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                divider.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::FieldSet(fieldset, meta, states, _, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                fieldset.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Headline(headline, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                headline.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::HyperLink(hyper_link, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                hyper_link.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Img(img, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                img.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Input(input, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                input.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Paragraph(paragraph, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                paragraph.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::ToolTip(tooltip, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                tooltip.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Badge(badge, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                badge.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::ProgressBar(progress_bar, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                progress_bar.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::RadioButton(radio_button, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                radio_button.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Scrollbar(scroll_bar, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                scroll_bar.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::Slider(slider, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                slider.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::SwitchButton(switch_button, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                switch_button.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::ToggleButton(toggle_button, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                toggle_button.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
        HtmlWidgetNode::ListBox(list_box, meta, states, functions, widget, id) => {
            update_with_meta(
                commands,
                entity,
                list_box.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
        }
    }
}

fn update_with_meta<T: Component>(
    commands: &mut Commands,
    entity: Entity,
    component: T,
    meta: &HtmlMeta,
    states: &HtmlStates,
    functions: &HtmlEventBindings,
    widget: &Widget,
    id: &HtmlID,
    start_hidden: bool,
) {
    commands.entity(entity).insert((
        component,
        functions.clone(),
        widget.clone(),
        id.clone(),
        meta.inner_content.clone(),
        CssSource(meta.css.clone()),
        CssClass(meta.class.clone().unwrap_or_default()),
        CssID(meta.id.clone().unwrap_or_default()),
        if start_hidden || states.hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        },
    ));

    if let Some(inline_style) = &meta.style {
        commands.entity(entity).insert(inline_style.clone());
    } else {
        commands.entity(entity).remove::<crate::html::HtmlStyle>();
    }

    if let Some(validation) = &meta.validation {
        commands.entity(entity).insert(validation.clone());
    } else {
        commands
            .entity(entity)
            .remove::<crate::widgets::ValidationRules>();
    }

    if states.hidden {
        commands.entity(entity).insert(NeedHidden);
    } else {
        commands.entity(entity).remove::<NeedHidden>();
    }
}

fn get_node_id(node: &HtmlWidgetNode) -> &HtmlID {
    match node {
        HtmlWidgetNode::Body(_, _, _, _, _, _, id)
        | HtmlWidgetNode::Button(_, _, _, _, _, id)
        | HtmlWidgetNode::CheckBox(_, _, _, _, _, id)
        | HtmlWidgetNode::ColorPicker(_, _, _, _, _, id)
        | HtmlWidgetNode::ChoiceBox(_, _, _, _, _, id)
        | HtmlWidgetNode::DatePicker(_, _, _, _, _, id)
        | HtmlWidgetNode::Divider(_, _, _, _, _, id)
        | HtmlWidgetNode::Headline(_, _, _, _, _, id)
        | HtmlWidgetNode::HyperLink(_, _, _, _, _, id)
        | HtmlWidgetNode::Img(_, _, _, _, _, id)
        | HtmlWidgetNode::Input(_, _, _, _, _, id)
        | HtmlWidgetNode::Paragraph(_, _, _, _, _, id)
        | HtmlWidgetNode::ToolTip(_, _, _, _, _, id)
        | HtmlWidgetNode::Badge(_, _, _, _, _, id)
        | HtmlWidgetNode::ProgressBar(_, _, _, _, _, id)
        | HtmlWidgetNode::RadioButton(_, _, _, _, _, id)
        | HtmlWidgetNode::Scrollbar(_, _, _, _, _, id)
        | HtmlWidgetNode::Slider(_, _, _, _, _, id)
        | HtmlWidgetNode::SwitchButton(_, _, _, _, _, id)
        | HtmlWidgetNode::ToggleButton(_, _, _, _, _, id)
        | HtmlWidgetNode::ListBox(_, _, _, _, _, id)
        | HtmlWidgetNode::Div(_, _, _, _, _, _, id)
        | HtmlWidgetNode::Form(_, _, _, _, _, _, id)
        | HtmlWidgetNode::FieldSet(_, _, _, _, _, _, id) => id,
        #[cfg(feature = "extended-dialog")]
        HtmlWidgetNode::Dialog(_, _, _, _, _, _, id) => id,
    }
}

/// Starts the delayed visibility timer after widgets are spawned.
fn show_all_widgets_start(
    mut events: MessageReader<HtmlAllWidgetsSpawned>,
    mut timer: ResMut<ShowWidgetsTimer>,
) {
    for _event in events.read() {
        timer.timer = Timer::from_seconds(0.0, TimerMode::Once);
        timer.active = true;
        debug!("Starting reveal timer before showing widgets");
    }
}

/// Makes all widgets visible after the delay elapses.
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

/// Collects all HTML IDs from a node tree.
fn collect_html_ids(nodes: &Vec<HtmlWidgetNode>, ids: &mut Vec<HtmlID>) {
    for node in nodes {
        match node {
            HtmlWidgetNode::Body(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
            }
            HtmlWidgetNode::Button(_, _, _, _, _, id)
            | HtmlWidgetNode::CheckBox(_, _, _, _, _, id)
            | HtmlWidgetNode::ColorPicker(_, _, _, _, _, id)
            | HtmlWidgetNode::ChoiceBox(_, _, _, _, _, id)
            | HtmlWidgetNode::DatePicker(_, _, _, _, _, id)
            | HtmlWidgetNode::Divider(_, _, _, _, _, id)
            | HtmlWidgetNode::Headline(_, _, _, _, _, id)
            | HtmlWidgetNode::HyperLink(_, _, _, _, _, id)
            | HtmlWidgetNode::Img(_, _, _, _, _, id)
            | HtmlWidgetNode::Input(_, _, _, _, _, id)
            | HtmlWidgetNode::Paragraph(_, _, _, _, _, id)
            | HtmlWidgetNode::ToolTip(_, _, _, _, _, id)
            | HtmlWidgetNode::Badge(_, _, _, _, _, id)
            | HtmlWidgetNode::ProgressBar(_, _, _, _, _, id)
            | HtmlWidgetNode::RadioButton(_, _, _, _, _, id)
            | HtmlWidgetNode::Scrollbar(_, _, _, _, _, id)
            | HtmlWidgetNode::Slider(_, _, _, _, _, id)
            | HtmlWidgetNode::SwitchButton(_, _, _, _, _, id)
            | HtmlWidgetNode::ToggleButton(_, _, _, _, _, id)
            | HtmlWidgetNode::ListBox(_, _, _, _, _, id) => {
                ids.push(id.clone());
            }
            HtmlWidgetNode::Div(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
            }
            #[cfg(feature = "extended-dialog")]
            HtmlWidgetNode::Dialog(_, _, _, children, _, _, id) => {
                ids.push(id.clone());
                collect_html_ids(children, ids);
            }
            HtmlWidgetNode::Form(_, _, _, children, _, _, id) => {
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
    start_hidden: bool,
) -> Entity {
    let entity = match node {
        HtmlWidgetNode::Body(body, meta, states, children, functions, widget, id) => {
            let entity = spawn_with_meta(
                commands,
                body.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
            for child in children {
                spawn_widget_node(commands, child, asset_server, Some(entity), start_hidden);
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
            start_hidden,
        ),
        HtmlWidgetNode::CheckBox(checkbox, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            checkbox.clone(),
            meta,
            states,
            functions,
            widget,
            id,
            start_hidden,
        ),
        HtmlWidgetNode::ColorPicker(color_picker, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                color_picker.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::ChoiceBox(choice_box, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                choice_box.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::DatePicker(date_picker, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                date_picker.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::Div(div, meta, states, children, functions, widget, id) => {
            let entity = spawn_with_meta(
                commands,
                div.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
            for child in children {
                spawn_widget_node(commands, child, asset_server, Some(entity), start_hidden);
            }
            entity
        }
        HtmlWidgetNode::Form(form, meta, states, children, functions, widget, id) => {
            let entity = spawn_with_meta(
                commands,
                form.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
            for child in children {
                spawn_widget_node(commands, child, asset_server, Some(entity), start_hidden);
            }
            entity
        }
        #[cfg(feature = "extended-dialog")]
        HtmlWidgetNode::Dialog(dialog, meta, states, children, functions, widget, id) => {
            let entity = spawn_with_meta(
                commands,
                dialog.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            );
            for child in children {
                spawn_widget_node(commands, child, asset_server, Some(entity), start_hidden);
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
            start_hidden,
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
                start_hidden,
            );
            for child in children {
                spawn_widget_node(commands, child, asset_server, Some(entity), start_hidden);
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
            start_hidden,
        ),
        HtmlWidgetNode::HyperLink(hyper_link, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                hyper_link.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::Img(img, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            img.clone(),
            meta,
            states,
            functions,
            widget,
            id,
            start_hidden,
        ),
        HtmlWidgetNode::Input(input, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            input.clone(),
            meta,
            states,
            functions,
            widget,
            id,
            start_hidden,
        ),
        HtmlWidgetNode::Paragraph(paragraph, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                paragraph.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::ToolTip(tooltip, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            tooltip.clone(),
            meta,
            states,
            functions,
            widget,
            id,
            start_hidden,
        ),
        HtmlWidgetNode::Badge(badge, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            badge.clone(),
            meta,
            states,
            functions,
            widget,
            id,
            start_hidden,
        ),
        HtmlWidgetNode::ProgressBar(progress_bar, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                progress_bar.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::RadioButton(radio_button, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                radio_button.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::Scrollbar(scroll_bar, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                scroll_bar.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::Slider(slider, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            slider.clone(),
            meta,
            states,
            functions,
            widget,
            id,
            start_hidden,
        ),
        HtmlWidgetNode::SwitchButton(switch_button, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                switch_button.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::ToggleButton(toggle_button, meta, states, functions, widget, id) => {
            spawn_with_meta(
                commands,
                toggle_button.clone(),
                meta,
                states,
                functions,
                widget,
                id,
                start_hidden,
            )
        }
        HtmlWidgetNode::ListBox(list_box, meta, states, functions, widget, id) => spawn_with_meta(
            commands,
            list_box.clone(),
            meta,
            states,
            functions,
            widget,
            id,
            start_hidden,
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
    start_hidden: bool,
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
            meta.inner_content.clone(),
            Node::default(),
            CssSource(meta.css.clone()),
            CssClass(meta.class.clone().unwrap_or_default()),
            CssID(meta.id.clone().unwrap_or_default()),
            ui_state,
            if start_hidden || states.hidden {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            },
        ))
        .id();

    if let Some(inline_style) = &meta.style {
        commands.entity(entity).insert(inline_style.clone());
    }

    if let Some(validation) = &meta.validation {
        commands.entity(entity).insert(validation.clone());
    }

    if states.hidden {
        commands.entity(entity).insert(NeedHidden);
    }

    entity
}
