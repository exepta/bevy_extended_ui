use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssSource, TagName};
use crate::widgets::widget_util::find_ancestor_with_component;
use crate::widgets::{
    BindToID, FieldMode, FieldSelectionSingle, FieldSet, InFieldSet, RadioButton, UIGenID,
    UIWidgetState, WidgetId, WidgetKind,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Marker component for initialized radio button widgets.
#[derive(Component)]
struct RadioButtonBase;

/// Marker component for radio button label nodes.
#[derive(Component)]
struct RadioButtonLabel;

/// Marker component for the radio selection dot.
#[derive(Component)]
pub struct RadioButtonDot;

/// Tracks whether a missing field set warning was logged.
#[derive(Resource, Default)]
struct RadioMissingFieldSetWarned(bool);

/// Plugin that registers radio button widget behavior.
pub struct RadioButtonWidget;

impl Plugin for RadioButtonWidget {
    /// Registers systems for radio button setup and selection logic.
    fn build(&self, app: &mut App) {
        app.init_resource::<RadioMissingFieldSetWarned>();
        app.add_systems(
            Update,
            (internal_node_creation_system, update_radio_button_system),
        );
        app.add_systems(Update, ensure_checked_dots_system);
        app.add_systems(Update, ensure_fieldset_selection_system);
    }
}

/// Initializes UI nodes for radio button widgets.
fn internal_node_creation_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &UIGenID,
            &RadioButton,
            Option<&CssSource>,
            &mut UIWidgetState,
        ),
        (With<RadioButton>, Without<RadioButtonBase>),
    >,
    parents: Query<&ChildOf>,
    fieldset_tag_q: Query<(), With<FieldSet>>,
    mut selection_q: Query<Option<&mut FieldSelectionSingle>, With<FieldSet>>,
    config: Res<ExtendedUiConfiguration>,
    mut warned: ResMut<RadioMissingFieldSetWarned>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (entity, id, radio_button, source_opt, mut state) in query.iter_mut() {
        let Some(fieldset_entity) = find_fieldset_ancestor(entity, &parents, &fieldset_tag_q)
        else {
            if !warned.0 {
                warn!(
                    "RadioButton widgets must be placed inside a <fieldset>. \
                 Orphan RadioButtons will be ignored."
                );
                warned.0 = true;
            }
            continue;
        };

        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        // initial checked state from parsed `selected`
        state.checked = radio_button.selected;

        // track initial selection in FieldSet (single)
        if radio_button.selected {
            if let Ok(Some(mut sel)) = selection_q.get_mut(fieldset_entity) {
                if sel.0.is_none() {
                    sel.0 = Some(entity);
                }
            }
        }

        commands
            .entity(entity)
            .insert(InFieldSet(fieldset_entity))
            .insert((
                Name::new(format!("RadioButton-{}", radio_button.entry)),
                Node::default(),
                WidgetId {
                    id: radio_button.entry,
                    kind: WidgetKind::RadioButton,
                },
                BackgroundColor::default(),
                ImageNode::default(),
                BorderColor::default(),
                BoxShadow::new(
                    Colored::TRANSPARENT,
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                ),
                ZIndex::default(),
                Pickable::default(),
                css_source.clone(),
                TagName(String::from("radio")),
                RenderLayers::layer(*layer),
                RadioButtonBase,
                children![
                    (
                        Name::new(format!("Radio-Dot-{}", radio_button.entry)),
                        Node::default(),
                        BackgroundColor::default(),
                        ImageNode::default(),
                        BorderColor::default(),
                        BoxShadow::new(
                            Colored::TRANSPARENT,
                            Val::Px(0.),
                            Val::Px(0.),
                            Val::Px(0.),
                            Val::Px(0.)
                        ),
                        ZIndex::default(),
                        css_source.clone(),
                        UIWidgetState::default(),
                        CssClass(vec!["radio-dot".to_string()]),
                        Pickable::IGNORE,
                        BindToID(id.0),
                        RenderLayers::layer(*layer),
                        RadioButtonDot,
                    ),
                    (
                        Name::new(format!("Radio-Label-{}", radio_button.entry)),
                        Text::new(radio_button.label.clone()),
                        TextColor::default(),
                        TextFont::default(),
                        TextLayout::default(),
                        ZIndex::default(),
                        css_source.clone(),
                        UIWidgetState::default(),
                        CssClass(vec!["radio-text".to_string()]),
                        Pickable::IGNORE,
                        BindToID(id.0),
                        RenderLayers::layer(*layer),
                        RadioButtonLabel
                    )
                ],
            ))
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

fn update_radio_button_system(
    mut radio_q: Query<
        (&RadioButton, &UIGenID, &mut UIWidgetState),
        (
            With<RadioButton>,
            With<RadioButtonBase>,
            Changed<RadioButton>,
        ),
    >,
    mut label_q: Query<(&BindToID, &mut Text), With<RadioButtonLabel>>,
) {
    for (radio_button, id, mut state) in radio_q.iter_mut() {
        state.checked = radio_button.selected;
        set_radio_label_text_for_id(id.0, &radio_button.label, &mut label_q);
    }
}

/// Updates the label text node bound to a radio widget id.
fn set_radio_label_text_for_id(
    bind_id: usize,
    value: &str,
    label_q: &mut Query<(&BindToID, &mut Text), With<RadioButtonLabel>>,
) {
    for (bind_to, mut text) in label_q.iter_mut() {
        if bind_to.0 == bind_id {
            text.0 = value.to_string();
        }
    }
}

/// Handles click events on radio buttons and updates selection state.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut commands: Commands,

    mut radio_q: Query<
        (
            Entity,
            &mut UIWidgetState,
            &UIGenID,
            &mut RadioButton,
            &CssSource,
        ),
        With<RadioButton>,
    >,
    dot_q: Query<(Entity, &BindToID, Option<&Children>, &ComputedNode), With<RadioButtonDot>>,

    parents: Query<&ChildOf>,
    mut fieldset_q: Query<(&FieldSet, Option<&mut FieldSelectionSingle>)>,
    fieldset_tag_q: Query<(), With<FieldSet>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
    config: Res<ExtendedUiConfiguration>,
) {
    let clicked = trigger.entity;

    let Some(fieldset_entity) = find_fieldset_ancestor(clicked, &parents, &fieldset_tag_q) else {
        trigger.propagate(false);
        return;
    };

    let Ok((fieldset, selection_single)) = fieldset_q.get_mut(fieldset_entity) else {
        trigger.propagate(false);
        return;
    };

    if fieldset.field_mode != FieldMode::Single {
        trigger.propagate(false);
        return;
    }

    let (widget_id, radio_entry, css_source, should_check, should_uncheck) = {
        let Ok((_entity, mut widget_state, widget_id, mut radio_button, css_source)) =
            radio_q.get_mut(clicked)
        else {
            trigger.propagate(false);
            return;
        };

        if widget_state.disabled {
            trigger.propagate(false);
            return;
        }

        current_widget_state.widget_id = widget_id.0;

        if widget_state.checked {
            if fieldset.allow_none {
                widget_state.checked = false;
                radio_button.selected = false;
                (
                    widget_id.0,
                    radio_button.entry,
                    css_source.clone(),
                    false,
                    true,
                )
            } else {
                (
                    widget_id.0,
                    radio_button.entry,
                    css_source.clone(),
                    false,
                    false,
                )
            }
        } else {
            widget_state.checked = true;
            radio_button.selected = true;
            (
                widget_id.0,
                radio_button.entry,
                css_source.clone(),
                true,
                false,
            )
        }
    };

    if should_check {
        add_checked_dot_to_radio(
            widget_id,
            radio_entry,
            &css_source,
            &dot_q,
            &mut commands,
            &config,
        );
    } else if should_uncheck {
        remove_checked_dot_by_bind_id(widget_id, &dot_q, &mut commands);
    }

    if should_check {
        if let Some(mut selection) = selection_single {
            selection.0 = Some(clicked);
        }
    } else if should_uncheck {
        if let Some(mut selection) = selection_single {
            if selection.0 == Some(clicked) {
                selection.0 = None;
            }
        }
    }

    if should_check {
        let radio_entities: Vec<Entity> = radio_q
            .iter()
            .map(|(radio_entity, _, _, _, _)| radio_entity)
            .collect();

        for radio_entity in radio_entities {
            if radio_entity == clicked {
                continue;
            }

            let Some(ancestor_fieldset) =
                find_fieldset_ancestor(radio_entity, &parents, &fieldset_tag_q)
            else {
                continue;
            };
            if ancestor_fieldset != fieldset_entity {
                continue;
            }

            if let Ok((
                _other_entity,
                mut other_widget_state,
                other_widget_id,
                mut other_radio_button,
                _other_css_source,
            )) = radio_q.get_mut(radio_entity)
            {
                if other_widget_state.checked {
                    other_widget_state.checked = false;
                    other_radio_button.selected = false;
                    remove_checked_dot_by_bind_id(other_widget_id.0, &dot_q, &mut commands);
                }
            }
        }
    }

    trigger.propagate(false);
}

/// Handles cursor-entered events on radio buttons.
///
/// Sets the `hovered` flag of the corresponding [`UIWidgetState`] to `true`.
/// This enables hover styles (e.g., `:hover`) to apply.
///
/// # Parameters
/// - `trigger`: A [`On<Pointer<Over>>`] when the pointer enters the radio area.
/// - `query`: Query for the UI widget state to be modified.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<RadioButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = !state.disabled;
    }

    trigger.propagate(false);
}

/// Handles cursor-leave events on radio buttons.
///
/// Sets the `hovered` flag of the corresponding [`UIWidgetState`] to `false`,
/// disabling hover styles (e.g., `:hover`).
///
/// # Parameters
/// - `trigger`: A [`On<Pointer<Out>>`] when the pointer leaves the radio area.
/// - `query`: Query for the UI widget state to be modified.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<RadioButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }

    trigger.propagate(false);
}

/// Finds the nearest ancestor field set entity.
fn find_fieldset_ancestor(
    entity: Entity,
    parents: &Query<&ChildOf>,
    fieldsets: &Query<(), With<FieldSet>>,
) -> Option<Entity> {
    find_ancestor_with_component::<FieldSet>(entity, parents, fieldsets)
}

/// Adds a visual dot to the selected radio button.
fn add_checked_dot_to_radio(
    gen_id: usize,
    radio_entry: usize,
    css_source: &CssSource,
    dot_q: &Query<(Entity, &BindToID, Option<&Children>, &ComputedNode), With<RadioButtonDot>>,
    commands: &mut Commands,
    config: &ExtendedUiConfiguration,
) {
    let layer = config.render_layers.first().unwrap_or(&1);

    for (dot_entity, bind, _children, computed) in dot_q.iter() {
        if bind.0 != gen_id {
            continue;
        }

        let width = computed.size.x / 1.5;
        let height = computed.size.y / 1.5;

        commands.entity(dot_entity).with_children(|b| {
            b.spawn((
                Name::new(format!("CheckedDot-{}", radio_entry)),
                Node {
                    width: Val::Px(width),
                    height: Val::Px(height),
                    ..default()
                },
                BackgroundColor::default(),
                BorderColor::default(),
                Pickable::IGNORE,
                css_source.clone(),
                UIWidgetState::default(),
                CssClass(vec!["checked-dot".to_string()]),
                BindToID(gen_id),
                RenderLayers::layer(*layer),
            ));
        });

        break;
    }
}

/// Removes a visual dot from a radio button by bind ID.
fn remove_checked_dot_by_bind_id(
    gen_id: usize,
    dot_q: &Query<(Entity, &BindToID, Option<&Children>, &ComputedNode), With<RadioButtonDot>>,
    commands: &mut Commands,
) {
    for (_, bind, children_opt, _computed) in dot_q.iter() {
        if bind.0 != gen_id {
            continue;
        }

        if let Some(children) = children_opt {
            for child in children.iter() {
                // Only despawn the checked-dot children that belong to this gen_id
                // If you don't have a marker component, you can still despawn all children here.
                commands.entity(child).despawn();
            }
        }
        break;
    }
}

/// Ensures radio buttons reflect their checked state visually.
fn ensure_checked_dots_system(
    mut commands: Commands,
    radio_q: Query<(&UIGenID, &RadioButton, &CssSource, &UIWidgetState)>,
    dot_q: Query<(Entity, &BindToID, Option<&Children>, &ComputedNode), With<RadioButtonDot>>,
    config: Res<ExtendedUiConfiguration>,
) {
    for (gen_id, rb, css, state) in radio_q.iter() {
        if !state.checked {
            continue;
        }
        // Skip if a checked-dot already exists
        let mut has_child = false;
        for (_dot_entity, bind, children_opt, _computed) in dot_q.iter() {
            if bind.0 != gen_id.0 {
                continue;
            }
            if let Some(children) = children_opt {
                if !children.is_empty() {
                    has_child = true;
                }
            }
            if !has_child {
                add_checked_dot_to_radio(gen_id.0, rb.entry, css, &dot_q, &mut commands, &config);
            }
            break;
        }
    }
}

/// Ensures field set selection state stays consistent.
fn ensure_fieldset_selection_system(
    radios: Query<(Entity, &InFieldSet, &UIWidgetState), With<RadioButton>>,
    mut fieldsets: Query<(&FieldSet, Option<&mut FieldSelectionSingle>)>,
) {
    for (radio_entity, in_fs, state) in radios.iter() {
        if !state.checked {
            continue;
        }
        if let Ok((fieldset, selection_opt)) = fieldsets.get_mut(in_fs.0) {
            if fieldset.field_mode != FieldMode::Single {
                continue;
            }
            if let Some(mut selection) = selection_opt {
                if selection.0.is_none() {
                    selection.0 = Some(radio_entity);
                }
            }
        }
    }
}
